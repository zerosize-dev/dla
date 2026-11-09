"""DLA measurement agent for Python processes.

Placed first on PYTHONPATH by `dla harness run`. It chain-imports the project's own
sitecustomize (if any), starts tracemalloc (DLA_TRACEMALLOC_FRAMES frames: 10 when allocation
captures are asked for, else 1) and, on Python 3.12+, a sys.monitoring
PY_START coverage collector, and writes one JSON record per dump to the file named by
DLA_HARNESS_OUT: an `exit` record from an atexit handler, `checkpoint` records when a driver calls
`checkpoint()`, and optional light `tick` records every DLA_TICK_MS.

Environment: DLA_HARNESS_OUT, DLA_HARNESS_PROBES (probes.json), DLA_PURGE (trim|gc|none),
DLA_PROJECT (scratch root), DLA_TICK_MS, DLA_ITER. Raw captures (`dla harness run --capture`):
DLA_CAPTURE (comma list of histogram, alloc, nonheap), DLA_CAPTURE_DIR, DLA_CAPTURE_DUMPS,
DLA_TRACEMALLOC_FRAMES, DLA_PY_MEMRAY=0 to leave memray out of `alloc`. The module never raises
into the target.
"""

import atexit
import collections
import collections.abc
import gc
import importlib
import importlib.machinery
import importlib.util
import json
import os
import sys
import threading
import time
import types

try:
    import re as _re
except BaseException:  # a broken stdlib must not stop the target
    _re = None

_HERE = os.path.dirname(os.path.abspath(__file__))
_T0 = time.monotonic()
_MAX_OBJECTS = 2_000_000
_COVERAGE_CAP = 20000
_PROBE_DEADLINE_S = 60.0
_MODULE_NAME_CAP = 150
_checkpoint_written = False
_lock = threading.RLock()
_coverage = set()
_coverage_on = False
_chain_error = None


def _env(name):
    value = os.environ.get(name)
    return value if value else None


def _is_tool_process():
    """True for packaging/venv tool runs, which are never measured."""
    argv = list(getattr(sys, "orig_argv", None) or sys.argv)
    for i, arg in enumerate(argv[:-1]):
        if arg == "-m" and argv[i + 1].split(".")[0] in ("pip", "ensurepip", "venv", "setuptools", "build", "wheel"):
            return True
    return False


# ---------------------------------------------------------------------------------------------
# start-up: tracemalloc, coverage, chain import
# ---------------------------------------------------------------------------------------------

_ACTIVE = _env("DLA_HARNESS_OUT") is not None and not _is_tool_process()


_TRACEMALLOC_FRAMES_MAX = 60


def _int_env(name, fallback):
    raw = _env(name)
    try:
        return int(raw) if raw is not None else fallback
    except ValueError:
        return fallback


def _capture_kinds():
    kinds = set()
    for part in (_env("DLA_CAPTURE") or "").split(","):
        part = part.strip().lower()
        if part:
            kinds.add(part)
    return kinds


_CAPTURE_KINDS = _capture_kinds()
_CAPTURE_DIR = _env("DLA_CAPTURE_DIR")


def _start_tracemalloc():
    try:
        import tracemalloc

        if not tracemalloc.is_tracing():
            frames = _int_env("DLA_TRACEMALLOC_FRAMES", 10 if "alloc" in _CAPTURE_KINDS else 1)
            tracemalloc.start(max(1, min(frames, _TRACEMALLOC_FRAMES_MAX)))
    except BaseException:
        pass


def _project_root():
    root = _env("DLA_PROJECT")
    if root is None:
        return None
    return os.path.realpath(root)


def _start_coverage():
    """Collect `module:qualname@file:line` of every project function started (3.12+)."""
    global _coverage_on
    mon = getattr(sys, "monitoring", None)
    root = _project_root()
    if mon is None or root is None:
        return
    prefix = root + os.sep
    decided = {}

    def on_start(code, offset):
        try:
            filename = code.co_filename
            rel = decided.get(filename)
            if rel is None:
                real = os.path.realpath(filename) if not filename.startswith("<") else ""
                rel = os.path.relpath(real, root) if real.startswith(prefix) else ""
                decided[filename] = rel
            if rel:
                module = rel[:-3] if rel.endswith(".py") else rel
                module = module.replace(os.sep, ".")
                if module.endswith(".__init__"):
                    module = module[: -len(".__init__")]
                if len(_coverage) < _COVERAGE_CAP:
                    _coverage.add(f"{module}:{code.co_qualname}@{rel}:{code.co_firstlineno}")
        except BaseException:
            pass
        return mon.DISABLE

    for tool_id in (mon.COVERAGE_ID, 3, 4):
        try:
            mon.use_tool_id(tool_id, "dla")
        except BaseException:
            continue
        try:
            mon.register_callback(tool_id, mon.events.PY_START, on_start)
            mon.set_events(tool_id, mon.events.PY_START)
            _coverage_on = True
        except BaseException:
            pass
        return


def _chain_sitecustomize():
    """Import the next sitecustomize on sys.path (the project's or the system's), if any."""
    global _chain_error
    try:
        paths = [p for p in sys.path if os.path.abspath(p or ".") != _HERE]
        spec = importlib.machinery.PathFinder.find_spec("sitecustomize", paths)
        if spec is None or spec.loader is None:
            return
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
    except BaseException as exc:  # a broken project sitecustomize must not stop the measurement
        _chain_error = f"chained sitecustomize failed: {exc!r}"


# ---------------------------------------------------------------------------------------------
# reading values
# ---------------------------------------------------------------------------------------------


def _os_values():
    out = {}
    keys = {
        "VmRSS": "os.rss",
        "RssAnon": "os.rss_anon",
        "RssFile": "os.rss_file",
        "RssShmem": "os.rss_shmem",
        "VmSwap": "os.swap",
        "VmData": "os.vm_data",
        "VmHWM": "os.vm_hwm",
    }
    try:
        with open("/proc/self/status", encoding="ascii", errors="replace") as fh:
            for line in fh:
                key, _, rest = line.partition(":")
                if key in keys:
                    out[keys[key]] = int(rest.split()[0]) * 1024
                elif key == "Threads":
                    out["os.threads"] = int(rest.split()[0])
    except BaseException:
        pass
    return out


def _traced():
    try:
        import tracemalloc

        if tracemalloc.is_tracing():
            return tracemalloc.get_traced_memory()
    except BaseException:
        pass
    return None


def _type_name(tp):
    mod = getattr(tp, "__module__", "") or ""
    qual = getattr(tp, "__qualname__", getattr(tp, "__name__", str(tp)))
    return qual if mod in ("builtins", "") else f"{mod}.{qual}"


def _types_top(objs, limit=20):
    counts = collections.Counter()
    sizes = collections.Counter()
    for obj in objs:
        tp = type(obj)
        counts[tp] += 1
        try:
            sizes[tp] += sys.getsizeof(obj)
        except BaseException:
            pass
    return [{"name": _type_name(tp), "count": n, "bytes": sizes[tp]} for tp, n in counts.most_common(limit)]


# ---------------------------------------------------------------------------------------------
# bounded reachable walk
# ---------------------------------------------------------------------------------------------

_SKIP_TYPES = tuple(
    t
    for t in (
        type,
        types.ModuleType,
        types.CodeType,
        types.FrameType,
        types.BuiltinFunctionType,
        types.MethodDescriptorType,
        types.WrapperDescriptorType,
        types.GetSetDescriptorType,
        types.MemberDescriptorType,
        types.MethodWrapperType,
        types.ClassMethodDescriptorType,
    )
)
_LEAF_TYPES = (types.GeneratorType, types.CoroutineType, types.AsyncGeneratorType, types.TracebackType)
_SINGLETONS = (None, True, False, Ellipsis, NotImplemented)
# Objects of these stdlib packages are counted but never descended into: they lead to the whole
# interpreter (loggers, event loops, threads) and not to data the program holds.
_OPAQUE_ROOTS = frozenset({"logging", "threading", "asyncio", "concurrent", "multiprocessing", "socket", "ssl", "selectors"})


def _children(obj):
    if isinstance(obj, types.FunctionType):
        out = []
        for cell in obj.__closure__ or ():
            try:
                out.append(cell.cell_contents)
            except ValueError:
                pass
        if obj.__defaults__:
            out.extend(obj.__defaults__)
        if obj.__kwdefaults__:
            out.extend(obj.__kwdefaults__.values())
        return out
    if isinstance(obj, types.MethodType):
        return [obj.__self__]
    return gc.get_referents(obj)


def _opaque(obj):
    root = (getattr(type(obj), "__module__", "") or "").split(".")[0]
    return root in _OPAQUE_ROOTS


def walk(root):
    """Return (objects, shallow_bytes, truncated) reachable from root; iterative and bounded."""
    seen = set()
    stack = [root]
    count = 0
    total = 0
    while stack:
        obj = stack.pop()
        ident = id(obj)
        if ident in seen:
            continue
        seen.add(ident)
        if obj is None or obj is True or obj is False or obj is Ellipsis or obj is NotImplemented:
            continue
        if isinstance(obj, _SKIP_TYPES):
            continue
        if count >= _MAX_OBJECTS:
            return count, total, True
        count += 1
        try:
            total += sys.getsizeof(obj)
        except BaseException:
            pass
        if isinstance(obj, _LEAF_TYPES) or _opaque(obj):
            continue
        try:
            stack.extend(_children(obj))
        except BaseException:
            pass
    return count, total, False


# ---------------------------------------------------------------------------------------------
# probes
# ---------------------------------------------------------------------------------------------

_CONTAINERS = (
    list,
    dict,
    set,
    bytearray,
    collections.deque,
    collections.abc.MutableMapping,
    collections.abc.MutableSequence,
    collections.abc.MutableSet,
)


def _is_cache(obj):
    return callable(getattr(obj, "cache_info", None)) and callable(getattr(obj, "cache_clear", None))


def _size_of(obj):
    try:
        if _is_cache(obj):
            return int(obj.cache_info().currsize)
        if hasattr(obj, "__len__") and not isinstance(obj, (str, bytes, type)):
            return int(len(obj))
    except BaseException:
        pass
    return None


def _record(probe_id, name, path, value, error=None):
    if error is not None:
        return {"id": probe_id, "name": name, "path": path, "size": None, "reachable": None, "bytes": None, "error": error}
    reachable, nbytes, truncated = walk(value)
    rec = {"id": probe_id, "name": name, "path": path, "size": _size_of(value), "reachable": reachable, "bytes": nbytes, "error": None}
    if truncated:
        rec["error"] = f"walk truncated at {_MAX_OBJECTS} objects"
    return rec


def _find_module(name):
    mod = sys.modules.get(name)
    if mod is not None:
        return mod
    main = sys.modules.get("__main__")
    spec = getattr(main, "__spec__", None)
    if spec is not None and spec.name == name:
        return main
    return importlib.import_module(name)


def _is_stdlib_instance(value):
    root = (getattr(type(value), "__module__", "") or "").split(".")[0]
    return root in getattr(sys, "stdlib_module_names", frozenset()) or root == "builtins"


def _module_probes(probe_id, modname, mod, deadline):
    out = []
    items = list(vars(mod).items())
    for name, value in items:
        if len(out) >= _MODULE_NAME_CAP or time.monotonic() > deadline:
            break
        if name.startswith("__") and name.endswith("__"):
            continue
        path = f"{modname}.{name}"
        if isinstance(value, _SKIP_TYPES + (types.FunctionType,)):
            if isinstance(value, type) and getattr(value, "__module__", None) in (modname, getattr(mod, "__name__", None)):
                out.extend(_class_probes(probe_id, modname, value, deadline))
            continue
        if isinstance(value, _CONTAINERS) or _is_cache(value):
            out.append(_record(probe_id, name, path, value))
        elif not isinstance(value, (str, bytes, int, float, complex, bool, tuple, frozenset, type(None))) and not _is_stdlib_instance(value):
            rec = _record(probe_id, name, path, value)
            if rec["reachable"] is not None and rec["reachable"] > 1:
                out.append(rec)
    return out


def _class_probes(probe_id, modname, cls, deadline):
    """Class-level containers and lru_cache wrappers (e.g. a memoised method) of a module's own classes."""
    out = []
    for attr, value in list(vars(cls).items()):
        if time.monotonic() > deadline:
            break
        if attr.startswith("__") and attr.endswith("__"):
            continue
        if isinstance(value, _CONTAINERS) or _is_cache(value):
            out.append(_record(probe_id, f"{cls.__qualname__}.{attr}", f"{modname}.{cls.__qualname__}.{attr}", value))
    return out


def _resolve(root, dotted):
    obj = root
    for part in dotted.split("."):
        obj = getattr(obj, part)
    return obj


def _normalise_kind(raw):
    return str(raw or "").replace("_", "").replace("-", "").lower()


def _probes(limitations):
    path = _env("DLA_HARNESS_PROBES")
    if path is None:
        return []
    try:
        with open(path, encoding="utf-8") as fh:
            specs = json.load(fh)
    except BaseException as exc:
        limitations.append(f"probes.json unreadable: {exc!r}")
        return []
    if isinstance(specs, dict):
        specs = specs.get("probes", [])
    out = []
    deadline = time.monotonic() + _PROBE_DEADLINE_S
    for spec in specs:
        try:
            probe_id = spec.get("id")
            target = spec.get("target") if isinstance(spec.get("target"), dict) else spec
            kind = _normalise_kind(target.get("kind"))
            if not kind.startswith("py"):
                continue  # another runtime's probe
            modname = target.get("module")
            try:
                mod = _find_module(modname)
            except BaseException as exc:
                out.append(_record(probe_id, modname, modname, None, f"module not importable: {exc!r}"))
                continue
            if kind == "pymodule":
                out.extend(_module_probes(probe_id, modname, mod, deadline))
            elif kind == "pyglobal":
                name = target.get("name")
                out.append(_record(probe_id, name, f"{modname}.{name}", getattr(mod, name)))
            elif kind == "pyattr":
                qual = target.get("qualname")
                out.append(_record(probe_id, qual, f"{modname}.{qual}", _resolve(mod, qual)))
        except BaseException as exc:
            out.append({"id": spec.get("id") if isinstance(spec, dict) else None, "name": None, "path": None, "size": None, "reachable": None, "bytes": None, "error": repr(exc)})
    if time.monotonic() > deadline:
        limitations.append(f"probe walk stopped after {_PROBE_DEADLINE_S:.0f} s")
    return out


# ---------------------------------------------------------------------------------------------
# purge
# ---------------------------------------------------------------------------------------------


def _malloc_trim():
    import ctypes
    import ctypes.util

    libc = ctypes.CDLL(ctypes.util.find_library("c") or "libc.so.6")
    return int(libc.malloc_trim(0))


def _purge(kind, before_gc_os, limitations):
    purge = {"kind": kind}
    if kind == "gc":
        purge["before"] = before_gc_os
        purge["after"] = _os_values()
    elif kind in ("trim", "trim-native-heap"):
        # Import ctypes before the first reading: loading it maps libraries and would
        # otherwise show up as RSS the trim "failed" to return.
        try:
            import ctypes  # noqa: F401
            import ctypes.util  # noqa: F401
        except BaseException:
            pass
        purge["before"] = _os_values()
        try:
            import platform

            if platform.libc_ver()[0] != "glibc" and not sys.platform.startswith("linux"):
                raise OSError("malloc_trim needs glibc")
            purge["detail"] = f"malloc_trim(0) -> {_malloc_trim()}"
        except BaseException as exc:
            limitations.append(f"purge trim unavailable: {exc!r}")
        purge["after"] = _os_values()
    else:
        limitations.append(f"purge '{kind}' is not supported by the python agent")
    return purge


# ---------------------------------------------------------------------------------------------
# raw captures (DLA_CAPTURE): allocation stacks, type counters, thread/task/resource sites
# ---------------------------------------------------------------------------------------------
#
# Files go to DLA_CAPTURE_DIR/<pid>/ and are listed in the "captures" array of the record that
# triggered them. DLA_CAPTURE is a comma list of:
#   alloc     py-<n>.alloc.json    tracemalloc: live bytes by allocation stack (innermost first)
#             py-1.memray.json     memray: allocations still live at the end (see below)
#   histogram py-<n>.types.json    gc.get_objects() counts and shallow bytes by type name
#   nonheap   py-<n>.nonheap.json  threads, asyncio tasks and ResourceWarnings by creation site
# Nothing here throws into the target: a failed capture becomes an entry with "error".
# memray runs only when it is importable by THIS interpreter (the target's) and DLA_PY_MEMRAY is
# not 0: the tracker starts at interpreter start-up, stops at the final record, and the helper
# dla_memray_leaks.py (run with this interpreter) turns the capture file into bounded JSON.
# DLA never reads the memray file itself.

_ALLOC_TOP_FIRST = 150
_ALLOC_TOP_LATER = 100
_ALLOC_TOP_SIZE = 50
_TYPES_TOP = 300
_GROUPS_CAP = 200
_FILE_BYTES_CAP = 4 << 20
_CAPTURE_ON = _ACTIVE and _CAPTURE_DIR is not None and bool(_CAPTURE_KINDS)
_cap_taken = 0  # checkpoints seen (driver mode)
_cap_seq = {"alloc": 0, "histogram": 0, "nonheap": 0}
_cap_final = False
_cap_notes = []  # limitations found while starting the captures
_tm_first = {}  # stack key -> size in the first allocation file
_rel_cache = {}
_memray = {"tracker": None, "bin": None, "error": None, "started": False, "stopped": False}
_resource = {"threads": False, "tasks_debug": False, "warnings": collections.Counter(), "warn_total": 0}


def _now_ms():
    return int(time.monotonic() * 1000)


def _project_rel(filename):
    """Path of a project file relative to DLA_PROJECT with '/' separators, else None."""
    if not filename or filename.startswith("<"):
        return None
    got = _rel_cache.get(filename, 0)
    if got != 0:
        return got
    root = _project_root()
    out = None
    if root is not None:
        low = filename.replace("\\", "/")
        if "/site-packages/" not in low and "/dist-packages/" not in low:
            real = os.path.realpath(filename)
            if real.startswith(root + os.sep):
                out = real[len(root) + 1 :].replace(os.sep, "/")
    if len(_rel_cache) < 20000:
        _rel_cache[filename] = out
    return out


def _shown_path(filename):
    """What a capture file says for a frame's file: project-relative, else the plain path."""
    return _project_rel(filename) or filename


def _creation_site(skip):
    """`file:line` of the project frame that created something: the innermost project frame
    that is not an `__init__` (a subclass constructor is not where the object was made)."""
    try:
        frame = sys._getframe(skip)
    except BaseException:
        return None
    fallback = None
    depth = 0
    while frame is not None and depth < 60:
        rel = _project_rel(frame.f_code.co_filename)
        if rel is not None:
            site = f"{rel}:{frame.f_lineno}"
            if frame.f_code.co_name != "__init__":
                return site
            if fallback is None:
                fallback = site
        frame = frame.f_back
        depth += 1
    return fallback


def _entry(kind, role, path, ms, error=None):
    e = {"kind": kind, "role": role, "ms": ms}
    if path is not None:
        e["file"] = os.path.basename(path)
    if error is not None:
        e["error"] = error
    else:
        try:
            e["bytes"] = os.path.getsize(path)
        except BaseException:
            e["bytes"] = 0
    return e


def _capture_dir():
    path = os.path.join(_CAPTURE_DIR, str(os.getpid()))
    os.makedirs(path, exist_ok=True)
    return path


def _write_json(path, doc, list_key=None):
    """Writes `doc` as one line; halves `doc[list_key]` until the file fits the cap."""
    text = json.dumps(doc, separators=(",", ":"), default=str)
    while len(text) > _FILE_BYTES_CAP and list_key and doc.get(list_key):
        doc[list_key] = doc[list_key][: len(doc[list_key]) // 2]
        doc["truncated"] = True
        text = json.dumps(doc, separators=(",", ":"), default=str)
    with open(path, "w", encoding="utf-8") as fh:
        fh.write(text + "\n")


def _dump_here(seq, warmup, rounds, dumps):
    """Whether the checkpoint numbered `seq` takes the heavy captures (same rule as the JVM)."""
    total = warmup + rounds
    if total <= 0 or total <= dumps:
        return True
    first = max(warmup, 1)
    if seq == first or seq == total:
        return True
    interior = dumps - 2
    if interior <= 0 or seq < first or seq > total:
        return False
    return any(seq == first + round((total - first) * j / (interior + 1)) for j in range(1, interior + 1))


# ---- alloc: tracemalloc

def _capture_tracemalloc(out, directory, limitations):
    t0 = _now_ms()
    role = "tracemalloc live bytes by allocation stack"
    path = None
    try:
        import tracemalloc

        if not tracemalloc.is_tracing():
            raise RuntimeError("tracemalloc is not tracing")
        snap = tracemalloc.take_snapshot()
        stats = snap.statistics("traceback")
        del snap
        # Filtering happens here, on the grouped stacks: a tracemalloc Filter over every trace is slow.
        agent_prefix = os.path.realpath(_HERE) + os.sep
        own = {}

        def agent_frame(filename):
            got = own.get(filename)
            if got is None:
                got = not filename.startswith("<") and os.path.realpath(filename).startswith(agent_prefix)
                own[filename] = got
            return got

        skip_files = (tracemalloc.__file__, "<frozen importlib._bootstrap>", "<frozen importlib._bootstrap_external>", "<unknown>")
        stats = [
            st
            for st in stats
            if st.traceback[-1].filename not in skip_files
            and not any(agent_frame(f.filename) for f in st.traceback)
        ]

        def key_of(st):
            return "|".join(f"{f.filename}:{f.lineno}" for f in st.traceback)

        first = not _tm_first
        if first:
            picked = stats[:_ALLOC_TOP_FIRST]
        else:
            grown = sorted(stats, key=lambda st: st.size - _tm_first.get(key_of(st), 0), reverse=True)[:_ALLOC_TOP_LATER]
            picked = list(grown)
            seen = {id(st) for st in grown}
            for st in stats[:_ALLOC_TOP_SIZE]:
                if id(st) not in seen:
                    picked.append(st)
        rows = []
        for st in picked:
            # tracemalloc lists the oldest frame first; the file says innermost first.
            frames = [[_shown_path(f.filename), f.lineno] for f in reversed(st.traceback)]
            rows.append({"size": st.size, "count": st.count, "frames": frames})
            if first:
                _tm_first[key_of(st)] = st.size
        current, peak = tracemalloc.get_traced_memory()
        _cap_seq["alloc"] += 1
        doc = {
            "dla_pyalloc": 1,
            "pid": os.getpid(),
            "t_ms": int((time.monotonic() - _T0) * 1000),
            "frames": tracemalloc.get_traceback_limit(),
            "traced": current,
            "traced_peak": peak,
            "stacks_total": len(stats),
            "stacks": rows,
        }
        path = os.path.join(directory, f"py-{_cap_seq['alloc']}.alloc.json")
        _write_json(path, doc, "stacks")
        out.append(_entry("alloc", role, path, _now_ms() - t0))
    except BaseException as exc:
        limitations.append(f"py alloc capture was not written: {exc!r}")
        out.append(_entry("alloc", role, None, _now_ms() - t0, repr(exc)))


# ---- alloc: memray

def _memray_wanted():
    return "alloc" in _CAPTURE_KINDS and _env("DLA_PY_MEMRAY") != "0"


def _start_memray():
    """Starts the memray tracker for the whole process when memray imports here."""
    if not (_CAPTURE_ON and _memray_wanted()):
        return
    try:
        if importlib.util.find_spec("memray") is None:
            _memray["error"] = "memray is not installed in this interpreter (" + (sys.executable or "python") + "); pip install memray there to add its live-at-exit report"
            return
        import memray

        directory = _capture_dir()
        path = os.path.join(directory, "memray.bin")
        try:
            os.remove(path)
        except OSError:
            pass
        tracker = memray.Tracker(path, native_traces=False, follow_fork=False)
        tracker.__enter__()
        _memray.update(tracker=tracker, bin=path, started=True)
    except BaseException as exc:
        _memray["error"] = f"memray could not start: {exc!r}"


def _stop_memray():
    """Stops the tracker at the start of the final record, so the agent's own work is not in it."""
    tracker = _memray["tracker"]
    if tracker is None:
        return
    _memray["tracker"] = None
    try:
        tracker.__exit__(None, None, None)
        _memray["stopped"] = True
    except BaseException as exc:
        _memray["error"] = f"memray tracker did not stop cleanly: {exc!r}"


def _finish_memray(out, directory, limitations):
    t0 = _now_ms()
    role = "memray allocations still live at the end"
    if not _memray["stopped"]:
        if _memray["error"]:
            out.append(_entry("alloc", role, None, 0, _memray["error"]))
        return
    binary = _memray["bin"]
    target = os.path.join(directory, "py-1.memray.json")
    helper = os.path.join(_HERE, "dla_memray_leaks.py")
    try:
        import subprocess

        if not os.path.isfile(helper):
            raise RuntimeError("dla_memray_leaks.py is missing next to the agent")
        env = {k: v for k, v in os.environ.items() if not k.startswith("DLA_HARNESS") and k != "PYTHONPATH"}
        done = subprocess.run(
            [sys.executable, "-I", helper, binary, target],
            env=env,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
            timeout=180,
        )
        if done.returncode != 0:
            tail = done.stderr.decode("utf-8", "replace").strip().splitlines()[-1:] or ["no message"]
            raise RuntimeError(f"helper exit {done.returncode}: {tail[0][:200]}")
        out.append(_entry("alloc", role, target, _now_ms() - t0))
    except BaseException as exc:
        limitations.append(f"py memray capture was not written: {exc!r}")
        out.append(_entry("alloc", role, None, _now_ms() - t0, repr(exc)))
    finally:
        try:
            os.remove(binary)  # the raw memray file is large and private to memray
        except OSError:
            pass


# ---- histogram: type counters

def _capture_types(out, directory, types_full, count, limitations):
    t0 = _now_ms()
    role = "python object counts by type"
    try:
        _cap_seq["histogram"] += 1
        path = os.path.join(directory, f"py-{_cap_seq['histogram']}.types.json")
        doc = {
            "dla_pytypes": 1,
            "pid": os.getpid(),
            "t_ms": int((time.monotonic() - _T0) * 1000),
            "gc_objects": count,
            "types": types_full,
        }
        _write_json(path, doc, "types")
        out.append(_entry("histogram", role, path, _now_ms() - t0))
    except BaseException as exc:
        limitations.append(f"py types capture was not written: {exc!r}")
        out.append(_entry("histogram", role, None, _now_ms() - t0, repr(exc)))


# ---- nonheap: threads, tasks, resource warnings

_PATTERN_CAP = 80


def _thread_pattern(name):
    """A thread name with every digit run replaced by `*` (`Thread-3 (work)` is `Thread-* (work)`)."""
    name = str(name)[:_PATTERN_CAP]
    return _re.sub(r"\d+", "*", name) if _re is not None else name


def _install_thread_sites():
    """Remembers where each thread was created: Thread.__init__ stores the project frame."""
    try:
        orig = threading.Thread.__init__

        def init(self, *args, **kwargs):
            try:
                self._dla_site = _creation_site(2)
            except BaseException:
                pass
            orig(self, *args, **kwargs)

        init.__wrapped__ = orig
        threading.Thread.__init__ = init
        _resource["threads"] = True
    except BaseException as exc:
        _cap_notes.append(f"thread creation sites unavailable: {exc!r}")


def _install_asyncio_debug():
    """asyncio keeps the creation stack of each task only in debug mode: ask for it."""
    if os.environ.get("PYTHONASYNCIODEBUG"):
        _resource["tasks_debug"] = True
        return
    os.environ["PYTHONASYNCIODEBUG"] = "1"
    _resource["tasks_debug"] = True
    try:
        import logging

        # Debug mode reports slow callbacks on stderr; that is noise in a measured run.
        logging.getLogger("asyncio").setLevel(logging.ERROR)
    except BaseException:
        pass


def _warning_kind(message):
    text = str(message).strip().split("<", 1)[0].strip()
    return text[:60] or "ResourceWarning"


def _install_resource_warnings():
    """Counts ResourceWarnings per allocation site (the warning's source object has a tracemalloc traceback)."""
    try:
        import warnings

        warnings.simplefilter("always", ResourceWarning)

        def count(message, category, source):
            site = None
            try:
                import tracemalloc

                tb = tracemalloc.get_object_traceback(source) if source is not None else None
                if tb is not None:
                    for frame in reversed(tb):  # newest first
                        rel = _project_rel(frame.filename)
                        if rel is not None:
                            site = f"{rel}:{frame.lineno}"
                            break
            except BaseException:
                pass
            _resource["warn_total"] += 1
            if len(_resource["warnings"]) < 500:
                _resource["warnings"][(_warning_kind(message), site)] += 1

        orig_impl = getattr(warnings, "_showwarnmsg_impl", None)
        if orig_impl is not None:

            def impl(msg):
                if issubclass(msg.category, ResourceWarning):
                    count(msg.message, msg.category, getattr(msg, "source", None))
                    return
                return orig_impl(msg)

            warnings._showwarnmsg_impl = impl
        else:
            orig_show = warnings.showwarning

            def show(message, category, filename, lineno, file=None, line=None, source=None):
                if issubclass(category, ResourceWarning):
                    count(message, category, source)
                    return
                return orig_show(message, category, filename, lineno, file, line)

            warnings.showwarning = show
            _cap_notes.append("ResourceWarning sites unavailable on this python (the warnings hook gets no source object)")
    except BaseException as exc:
        _cap_notes.append(f"ResourceWarning counting unavailable: {exc!r}")


def _task_rows(objs):
    asyncio = sys.modules.get("asyncio")
    task_type = getattr(asyncio, "Task", None) if asyncio is not None else None
    if task_type is None:
        return 0, []
    groups = collections.Counter()
    total = 0
    for obj in objs:
        if type(obj) is not task_type and not isinstance(obj, task_type):
            continue
        try:
            if obj.done():
                continue
            coro = obj.get_coro()
            name = getattr(coro, "__qualname__", None) or type(coro).__name__
            frame = getattr(coro, "cr_frame", None) or getattr(coro, "gi_frame", None)
            where = None
            if frame is not None:
                rel = _project_rel(frame.f_code.co_filename)
                where = f"{rel}:{frame.f_lineno}" if rel is not None else None
            site = None
            for fs in reversed(getattr(obj, "_source_traceback", None) or []):  # newest first
                rel = _project_rel(getattr(fs, "filename", None))
                if rel is not None:
                    site = f"{rel}:{fs.lineno}"
                    break
            total += 1
            groups[(name, where, site)] += 1
        except BaseException:
            continue
    rows = [{"coro": k[0], "frame": k[1], "site": k[2], "count": n} for k, n in groups.most_common(_GROUPS_CAP)]
    return total, rows


def _capture_resources(out, directory, x, objs, limitations):
    t0 = _now_ms()
    role = "threads, asyncio tasks and resource warnings by creation site"
    try:
        threads = [t for t in threading.enumerate() if not str(t.name).startswith("dla-")]
        tgroups = collections.Counter()
        for t in threads:
            tgroups[(_thread_pattern(t.name), getattr(t, "_dla_site", None))] += 1
        task_total, task_rows = _task_rows(objs)
        notes = [
            "a thread's site is the project frame that created it (not the frame it runs in)",
            "ResourceWarning counts are cumulative: a warning is raised when an unclosed object is collected",
        ]
        if not _resource["threads"]:
            notes.append("thread creation sites were not recorded")
        if not _resource["tasks_debug"]:
            notes.append("asyncio debug mode was off: tasks have no creation site")
        elif not task_rows:
            notes.append("no pending asyncio task at this point")
        _cap_seq["nonheap"] += 1
        path = os.path.join(directory, f"py-{_cap_seq['nonheap']}.nonheap.json")
        doc = {
            "dla_py_resources": 1,
            "pid": os.getpid(),
            "x": x,
            "t_ms": int((time.monotonic() - _T0) * 1000),
            "threads": {
                "total": len(threads),
                "groups": [{"pattern": k[0], "site": k[1], "count": n} for k, n in tgroups.most_common(_GROUPS_CAP)],
            },
            "tasks": {"total": task_total, "debug": _resource["tasks_debug"], "groups": task_rows},
            "resource_warnings": {
                "total": _resource["warn_total"],
                "groups": [
                    {"what": k[0], "site": k[1], "count": n} for k, n in _resource["warnings"].most_common(_GROUPS_CAP)
                ],
            },
            "limitations": notes,
        }
        _write_json(path, doc)
        out.append(_entry("nonheap", role, path, _now_ms() - t0))
    except BaseException as exc:
        limitations.append(f"py nonheap capture was not written: {exc!r}")
        out.append(_entry("nonheap", role, None, _now_ms() - t0, repr(exc)))


def _capture_now(phase, x, last, types_full, objs, limitations):
    """Takes the captures due at this record; returns the entries for its "captures" array."""
    global _cap_taken, _cap_final
    out = []
    if not _CAPTURE_ON or _cap_final:
        return out
    driver = phase == "checkpoint"
    final = (not driver) or last
    here = True
    if driver:
        _cap_taken += 1
        here = _dump_here(_cap_taken, _int_env("DLA_WARMUP", 0), _int_env("DLA_ROUNDS", 0), _int_env("DLA_CAPTURE_DUMPS", 4))
    try:
        directory = _capture_dir()
    except BaseException as exc:
        limitations.append(f"capture directory unusable: {exc!r}")
        return out
    for kind in sorted(_CAPTURE_KINDS):
        if kind not in ("alloc", "histogram", "nonheap") and final:
            limitations.append(f"--capture {kind} is not written by the python agent (alloc, histogram and nonheap are)")
    if "alloc" in _CAPTURE_KINDS and (here or final):
        _capture_tracemalloc(out, directory, limitations)
    if "histogram" in _CAPTURE_KINDS and (here or final):
        _capture_types(out, directory, types_full, len(objs), limitations)
    if "nonheap" in _CAPTURE_KINDS and (here or final):
        _capture_resources(out, directory, x, objs, limitations)
    if final:
        _cap_final = True
        if "alloc" in _CAPTURE_KINDS and _memray_wanted():
            _finish_memray(out, directory, limitations)
    return out


# ---------------------------------------------------------------------------------------------
# dumping
# ---------------------------------------------------------------------------------------------


def _build(phase, x, last):
    limitations = []
    tick = phase == "tick"
    rec = {
        "dla_harness": 1,
        "phase": phase,
        "pid": os.getpid(),
        "argv": list(getattr(sys, "orig_argv", None) or sys.argv),
        "exe": sys.executable,
        "runtime": "python",
        "x": x,
        "t_ms": int((time.monotonic() - _T0) * 1000),
        "gc_forced": not tick,
    }
    if _CAPTURE_ON and not tick and (phase == "exit" or last):
        _stop_memray()
    purge = _env("DLA_PURGE")
    want_purge = last and not tick and purge not in (None, "none")
    pre = None
    os_before_gc = None
    if not tick:
        pre = {}
        traced = _traced()
        if traced is not None:
            pre["py.traced"] = traced[0]
        pre["py.gc_objects"] = len(gc.get_objects())
        os_now = _os_values()
        if "os.rss" in os_now:
            pre["os.rss"] = os_now["os.rss"]
        os_before_gc = os_now
        gc.collect()
        gc.collect()
    values = {}
    traced = _traced()
    if traced is not None:
        values["py.traced"] = traced[0]
        values["py.traced_peak"] = traced[1]
    else:
        limitations.append("tracemalloc not tracing: py.traced unavailable")
    objs = None if tick else gc.get_objects()
    values["py.gc_objects"] = len(objs) if objs is not None else len(gc.get_objects())
    values["py.gc_collections"] = sum(s.get("collections", 0) for s in gc.get_stats())
    values.update(_os_values())
    rec["values"] = values
    if pre is not None:
        rec["values_pre_gc"] = pre
    if want_purge:
        rec["purge"] = _purge(purge, os_before_gc, limitations)
    if not tick:
        types_full = _types_top(objs, _TYPES_TOP if _CAPTURE_ON and "histogram" in _CAPTURE_KINDS else 20)
        rec["types"] = types_full[:20]
        if _CAPTURE_ON:
            captures = _capture_now(phase, x, last, types_full, objs, limitations)
            if captures:
                rec["captures"] = captures
        del objs, types_full
        rec["probes"] = _probes(limitations)
        if phase == "exit":
            if _coverage_on:
                rec["coverage"] = {"kind": "functions", "items": sorted(_coverage)}
                if len(_coverage) >= _COVERAGE_CAP:
                    limitations.append(f"coverage capped at {_COVERAGE_CAP} functions")
            else:
                limitations.append("function coverage unavailable (sys.monitoring needs Python 3.12+)")
    if _chain_error:
        limitations.append(_chain_error)
    limitations.extend(_cap_notes)
    limitations.append("py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)")
    rec["limitations"] = limitations
    return rec


def dump(phase="exit", x=None, last=True):
    """Write one record to DLA_HARNESS_OUT. Never raises."""
    out = _env("DLA_HARNESS_OUT")
    if out is None:
        return
    with _lock:
        try:
            rec = _build(phase, x, last)
        except BaseException as exc:
            rec = {"dla_harness": 1, "phase": phase, "pid": os.getpid(), "runtime": "python", "x": x, "limitations": [f"dump failed: {exc!r}"]}
        try:
            line = json.dumps(rec, separators=(",", ":"), default=str) + "\n"
            parent = os.path.dirname(out)
            if parent:
                os.makedirs(parent, exist_ok=True)
            fd = os.open(out, os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o644)
            try:
                os.write(fd, line.encode("utf-8"))
            finally:
                os.close(fd)
        except BaseException:
            pass


def checkpoint(x, last=False):
    """Driver entry: write a checkpoint record; x is the number of units completed."""
    global _checkpoint_written
    _checkpoint_written = True
    dump("checkpoint", x, last)


def _exit_dump():
    try:
        if _checkpoint_written:
            return  # the driver owns its final record
        try:
            x = int(_env("DLA_ITER")) if _env("DLA_ITER") else None
        except ValueError:
            x = None
        dump("exit", x, True)
    except BaseException:
        pass


def _ticker(interval):
    while True:
        time.sleep(interval)
        try:
            dump("tick", None, False)
        except BaseException:
            pass


def _init():
    if not _ACTIVE:
        return
    _start_tracemalloc()
    if _CAPTURE_ON and "nonheap" in _CAPTURE_KINDS:
        _install_thread_sites()
        _install_asyncio_debug()
        _install_resource_warnings()
    _start_memray()
    _start_coverage()
    atexit.register(_exit_dump)
    tick = _env("DLA_TICK_MS")
    if tick and tick.isdigit() and int(tick) > 0:
        thread = threading.Thread(target=_ticker, args=(int(tick) / 1000.0,), name="dla-tick", daemon=True)
        thread.start()


if __name__ != "__main__":
    try:
        _init()
    except BaseException:
        pass
    _chain_sitecustomize()
