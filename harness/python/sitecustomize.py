"""DLA measurement agent for Python processes.

Placed first on PYTHONPATH by `dla harness run`. It chain-imports the project's own
sitecustomize (if any), starts tracemalloc (DLA_TRACEMALLOC_FRAMES frames: 10 when allocation or non-heap
captures are asked for, else _DRIVER_FRAMES in driver mode (DLA_KEY_MODE is set), else 1) and, on Python 3.12+, a sys.monitoring
PY_START coverage collector, and writes one JSON record per dump to the file named by
DLA_HARNESS_OUT: an `exit` record from an atexit handler, `checkpoint` records when a driver calls
`checkpoint()`, and optional light `tick` records every DLA_TICK_MS.

Environment: DLA_HARNESS_OUT, DLA_HARNESS_PROBES (probes.json), DLA_PURGE (trim|gc|none),
DLA_PROJECT (scratch root), DLA_TICK_MS, DLA_ITER. Raw captures (`dla harness run --capture`):
DLA_CAPTURE (comma list of histogram, alloc, nonheap), DLA_CAPTURE_DIR, DLA_CAPTURE_DUMPS,
DLA_TRACEMALLOC_FRAMES, DLA_PY_MEMRAY=0 to leave memray out of `alloc`. The module never raises
into the target.

Driver mode also counts the live traced blocks per allocation site at every checkpoint (see
"py.blocks" below): each checkpoint record carries its own counts, and the agent keeps nothing
between checkpoints that grows with their number; `dla harness run` picks the columns.
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
_checkpoint_count = 0
_lock = threading.RLock()
_coverage = set()
_coverage_on = False
_chain_error = None


def _env(name):
    value = os.environ.get(name)
    return value if value else None


def mark_phase(text):
    """Records what the process is doing in `<DLA_HARNESS_OUT>.phase` (one line, overwritten), so
    that a point killed at its timeout can say where it was stuck. Never raises."""
    out = _env("DLA_HARNESS_OUT")
    if out is None or not _ACTIVE:
        return
    try:
        with open(out + ".phase", "w", encoding="utf-8") as fh:
            fh.write(f"{text} (at {int((time.monotonic() - _T0) * 1000)} ms)\n")
    except BaseException:
        pass


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
# Frames tracemalloc keeps per block in driver mode when no capture asks for more. The site of a
# block (py.blocks) is its innermost project frame, so a block made inside a library function that
# project code called has a site only when enough frames are kept. Chosen by measuring the unit time
# of a driver run at 1, 2, 4 and 8 frames (harness/python/README.md): the deepest within 1.25 times
# the time at one frame.
_DRIVER_FRAMES = 1


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
            if "alloc" in _CAPTURE_KINDS or "nonheap" in _CAPTURE_KINDS:
                fallback = 10
            else:
                fallback = _DRIVER_FRAMES if _env("DLA_KEY_MODE") is not None else 1
            frames = _int_env("DLA_TRACEMALLOC_FRAMES", fallback)
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
                if real.startswith(prefix):
                    rel = os.path.relpath(real, root)
                else:
                    # A package installed in editable mode runs from the source tree.
                    rel = _source_rel(filename) or ""
                    rel = rel.replace("/", os.sep)
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
    """The module the workload has already imported, or None. A probe never imports: importing
    runs the module's top level, which can start a server or block, and it would change the
    state under measurement."""
    if not isinstance(name, str) or not name:
        return None
    mod = sys.modules.get(name)
    if mod is not None:
        return mod
    main = sys.modules.get("__main__")
    spec = getattr(main, "__spec__", None)
    if spec is not None and getattr(spec, "name", None) == name:
        return main
    return None


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
    not_loaded = []
    deadline = time.monotonic() + _PROBE_DEADLINE_S
    for spec in specs:
        try:
            probe_id = spec.get("id")
            target = spec.get("target") if isinstance(spec.get("target"), dict) else spec
            kind = _normalise_kind(target.get("kind"))
            if not kind.startswith("py"):
                continue  # another runtime's probe
            modname = target.get("module")
            mark_phase(f"checkpoint: probe {probe_id} ({kind} {modname})")
            mod = _find_module(modname)
            if mod is None:
                if str(modname) not in not_loaded:
                    not_loaded.append(str(modname))
                out.append(_record(probe_id, modname, modname, None, "module not loaded by the workload at this checkpoint (a probe never imports it)"))
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
    if not_loaded:
        shown = ", ".join(not_loaded[:5]) + (f" and {len(not_loaded) - 5} more" if len(not_loaded) > 5 else "")
        limitations.append(
            f"{len(not_loaded)} probed module(s) were not loaded by the workload and were not imported, so they have no column: {shown}"
        )
    if time.monotonic() > deadline:
        limitations.append(f"probe walk stopped after {_PROBE_DEADLINE_S:.0f} s")
    return out


# ---------------------------------------------------------------------------------------------
# purge
# ---------------------------------------------------------------------------------------------


_libc = None
_trim_unavailable = None  # the reason malloc_trim cannot be used, found at start-up


def _malloc_trim():
    global _libc
    if _libc is None:
        import ctypes
        import ctypes.util

        _libc = ctypes.CDLL(ctypes.util.find_library("c") or "libc.so.6")
    return int(_libc.malloc_trim(0))


def _preload_trim():
    """Load ctypes and libc and call malloc_trim once at start-up, in every measured process.

    The first use maps libraries and binds the call; done at the purge it would show as
    resident memory (RSS and file-backed pages) that the purge reading "failed" to return.
    The purge is asked only of the process of the last point (`DLA_PURGE` is set for that
    one), so every process loads them: the points of a series then carry the same load and the
    last one does not stand above the others by it. `platform` is imported (and the C library
    looked up) here too: importing it between the two readings of the purge would show as the
    memory the purge failed to return.
    """
    global _trim_unavailable
    try:
        if sys.platform.startswith("linux"):
            import platform

            if platform.libc_ver()[0] != "glibc":
                _trim_unavailable = "malloc_trim needs glibc"
            else:
                _malloc_trim()
    except BaseException:
        pass


def _purge(kind, before_gc_os, own_os, limitations):
    purge = {"kind": kind}
    if kind == "gc":
        purge["before"] = before_gc_os
        purge["after"] = _os_values()
    elif kind in ("trim", "trim-native-heap"):
        # The checkpoint's own reading (the last point of the series), not a new one: reading
        # again after loading ctypes would put the cost of the load in `before`. The load is
        # done at start-up (`_preload_trim`).
        purge["before"] = dict(own_os)
        try:
            if _trim_unavailable:
                raise OSError(_trim_unavailable)
            if not sys.platform.startswith("linux"):
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


def _is_thread_constructor(frame):
    """True when `frame` is an `__init__` running on a `threading.Thread` instance (a subclass
    constructor calling `super().__init__()`), not a plain class that makes a thread."""
    try:
        import threading

        return isinstance(frame.f_locals.get("self"), threading.Thread)
    except BaseException:
        return False


def _creation_site(skip):
    """`file:line` of the project frame that created something: the innermost project frame,
    except a Thread subclass `__init__` (that is not where the thread was made, its caller is).
    An `__init__` of any other class is where the object was made and is reported."""
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
            if frame.f_code.co_name != "__init__" or not _is_thread_constructor(frame):
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
            # The baseline covers every stack, not only the ones written, so a stable stack
            # outside the first top-N is not counted as growth from zero later.
            for st in stats:
                _tm_first[key_of(st)] = st.size
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

        try:
            _memray["site"] = os.path.dirname(os.path.dirname(os.path.realpath(memray.__file__)))
        except BaseException:
            pass
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
            [sys.executable, "-I", helper, binary, target] + (["--site", _memray["site"]] if _memray.get("site") else []),
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
    """A stable, plain label for a ResourceWarning: digit runs (pids, ports) become `*`, and a
    message that is not plain words (it may carry a name, path or secret) is the category."""
    import re

    text = str(message).strip().split("<", 1)[0].strip()
    text = re.sub(r"\d+", "*", text)[:60].strip()
    if not re.fullmatch(r"[A-Za-z][A-Za-z *_]*", text):
        return "ResourceWarning"
    return text


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
# gc.freeze: objects moved to the permanent generation are never collected
# ---------------------------------------------------------------------------------------------

_freeze_lock = threading.Lock()  # not _lock: a dump holds that one for as long as a probe walk
_freeze = {"on": False, "called": False, "sites": {}}  # sites: call site -> objects it froze


def _freeze_count():
    try:
        return gc.get_freeze_count()
    except BaseException:
        return None


def _split_freeze(current):
    """`{site: share}` summing to `current`, each site in proportion to what it froze.
    Objects frozen earlier and since freed by reference counting leave the count, so a plain
    cumulative sum per site would exceed it."""
    sites = _freeze["sites"]
    total = sum(sites.values())
    if current is None or current <= 0 or total <= 0:
        return {site: 0 for site in sites}
    shares = {site: current * n // total for site, n in sites.items()}
    left = current - sum(shares.values())
    for site in sorted(sites, key=lambda k: (-(current * sites[k] % total), k))[:left]:
        shares[site] += 1
    return shares


def _install_gc_freeze():
    """Wraps gc.freeze and gc.unfreeze to attribute the frozen objects to the calling project
    frame. The bookkeeping never raises; the real call's own result and errors are the target's."""
    freeze = getattr(gc, "freeze", None)
    unfreeze = getattr(gc, "unfreeze", None)
    if freeze is None or _freeze_count() is None:
        return

    def dla_freeze(*args, **kwargs):
        before = None
        try:
            with _freeze_lock:
                before = _freeze_count()
                if before is not None:
                    # Rebase on what is still frozen, so the earlier sites keep their true share.
                    _freeze["sites"] = _split_freeze(before)
        except BaseException:
            pass
        result = freeze(*args, **kwargs)
        try:
            with _freeze_lock:
                after = _freeze_count()
                _freeze["called"] = True
                if before is not None and after is not None:
                    try:
                        site = _creation_site(2) or "<non-project>"
                    except BaseException:
                        site = "<non-project>"  # the group is kept, only its site is lost
                    sites = _freeze["sites"]
                    sites[site] = sites.get(site, 0) + max(after - before, 0)
        except BaseException:
            pass
        return result

    def dla_unfreeze(*args, **kwargs):
        result = unfreeze(*args, **kwargs)
        try:
            with _freeze_lock:
                # Keep the sites so the group reads zero afterwards instead of vanishing.
                _freeze["sites"] = {site: 0 for site in _freeze["sites"]}
        except BaseException:
            pass
        return result

    dla_freeze.__name__ = "freeze"
    dla_unfreeze.__name__ = "unfreeze"
    gc.freeze = dla_freeze
    if unfreeze is not None:
        gc.unfreeze = dla_unfreeze
    _freeze["on"] = True


def _freeze_record(rec, limitations):
    """Adds the `py.gc_frozen` group (after any freeze call) and the limitation line (whenever
    the permanent generation holds objects, whoever put them there)."""
    count = _freeze_count()
    if _freeze["on"] and _freeze["called"]:
        with _freeze_lock:
            shares = _split_freeze(count)
        if shares:
            rec.setdefault("groups", {})["py.gc_frozen"] = shares
        elif count:
            limitations.append(
                "reduced record: gc.freeze was called but py.gc_frozen has no site to group by, so the column is missing"
            )
    if count:
        limitations.append(
            f"gc.freeze moved {count} objects into the permanent generation: py.gc_objects and the type counts exclude them"
        )


# ---------------------------------------------------------------------------------------------
# py.blocks: live traced blocks per allocation site (driver mode)
# ---------------------------------------------------------------------------------------------
#
# At each checkpoint, after the forced collections, the live blocks tracemalloc traces are counted
# by site: the first frame, from the innermost outward, that lies in the project and is neither a
# test file nor a file of the harness (the agent, the driver). The record carries these counts as
# `group_counts: {"py.blocks": {site: blocks}}` and nothing is kept for the next checkpoint (but the
# file-name cache, which fills once), so the agent's own bookkeeping cannot read as growth. The
# series builder picks the sites that grew. At most _BLOCKS_SITES_CAP sites are written per record
# (the largest counts); a record that hit the bound lists `py.blocks` in `group_capped`.
#
# Order inside a checkpoint: the snapshot and the grouping run right after the two collections and
# BEFORE py.traced, py.traced_peak and the OS values are read, at every checkpoint alike. The
# snapshot's transient memory (freed at once, but the allocator keeps part of it) is then in every
# reading, the warm-up checkpoints' and the first measured one's included, so the first measured
# point is not cheaper than the later ones. The counts dict of the checkpoint lives until its
# record is written: it weighs the same on every point of a run whose number of sites is stable.

_BLOCKS_SITES_CAP = 5000  # sites kept per checkpoint (the largest counts), so memory stays bounded
_BLOCKS_TEST_DIRS = frozenset(("test", "tests", "__tests__", "__mocks__", "testing", "spec", "specs"))
_blocks = {"own": {}, "files": {}}  # file name -> answer caches: they fill once
# The drivers the harness writes: the generic hook driver and the pytest loop (the project's own tests).
_DRIVER_FILES = ("dla_harness.py", "dla_pytest_loop.py")


def _is_test_rel(rel):
    """The rule `FileInfo.is_test` of the workload discovery applies to a Python file: a file under
    a test directory, `test_*` and `*_test`, `conftest.py`, and the entry files `setup.py` and
    `__main__.py`. `rel` is project-relative with '/' separators."""
    parts = rel.split("/")
    name = parts[-1]
    stem = name.rsplit(".", 1)[0] if "." in name else name
    return (
        any(d in _BLOCKS_TEST_DIRS for d in parts[:-1])
        or stem.startswith("test_")
        or stem.endswith("_test")
        or name == "conftest.py"
        or stem in ("setup", "__main__")
    )


def _harness_file(filename):
    """True for a file of the harness: the agent's directory (which also holds the driver the
    harness copied there) and the main script when it is a driver (`dla_harness.py`, or
    `dla_pytest_loop.py`, the loop over the project's own tests)."""
    got = _blocks["own"].get(filename)
    if got is None:
        real = os.path.realpath(filename)
        got = real.startswith(os.path.realpath(_HERE) + os.sep)
        if not got:
            main = getattr(sys.modules.get("__main__"), "__file__", None)
            got = bool(main) and os.path.basename(main) in _DRIVER_FILES and real == os.path.realpath(main)
        if len(_blocks["own"]) < 20000:
            _blocks["own"][filename] = got
    return got


def _source_rel(filename):
    """Path of a file of the project the scratch copy was made from (DLA_SOURCE_ROOT), relative to
    that project, else None. A library installed in editable mode runs from there, not from the
    copy; the copy keeps the project's layout, so the path names the same line in both. The run's
    own directory (.dla), the repository metadata and installed packages are not the project."""
    root = _env("DLA_SOURCE_ROOT")
    if root is None or not filename or filename.startswith("<"):
        return None
    low = filename.replace("\\", "/")
    if "/site-packages/" in low or "/dist-packages/" in low:
        return None
    root = os.path.realpath(root)
    real = os.path.realpath(filename)
    if not real.startswith(root + os.sep):
        return None
    rel = real[len(root) + 1 :].replace(os.sep, "/")
    return None if rel.split("/", 1)[0] in (".dla", ".git") else rel


def _block_file(filename):
    """The project-relative path of `filename` when blocks made there count, else None."""
    rel = _project_rel(filename) or _source_rel(filename)
    if rel is None or _is_test_rel(rel) or _harness_file(filename):
        return None
    return rel


def _count_blocks():
    """`{site: live traced blocks}` for the project's allocation sites right now."""
    import tracemalloc

    if not tracemalloc.is_tracing():
        raise RuntimeError("tracemalloc is not tracing")
    # The collector is off while the snapshot is grouped: it allocates millions of tuples, and
    # the collections they would trigger are the agent's own work, not the program's.
    was_on = gc.isenabled()
    gc.disable()
    try:
        snap = tracemalloc.take_snapshot()
        stats = snap.statistics("traceback")
        del snap
    finally:
        if was_on:
            gc.enable()
    counts = {}
    files = _blocks["files"]  # file name -> counting path or None, kept across checkpoints
    for st in stats:
        site = None
        for frame in reversed(st.traceback):  # tracemalloc lists the oldest frame first
            name = frame.filename
            rel = files.get(name, 0)
            if rel == 0:
                rel = _block_file(name)
                if len(files) < 20000:
                    files[name] = rel
            if rel is not None:
                site = f"{rel}:{frame.lineno}"
                break
        if site is not None:
            counts[site] = counts.get(site, 0) + st.count
    capped = len(counts) > _BLOCKS_SITES_CAP
    if capped:
        counts = dict(sorted(counts.items(), key=lambda kv: (-kv[1], kv[0]))[:_BLOCKS_SITES_CAP])
    return counts, capped


def _blocks_checkpoint(limitations):
    """Counts the blocks now; returns `(counts, capped)`, or None with a limitation when it cannot."""
    try:
        import tracemalloc

        if tracemalloc.get_traceback_limit() == 1:
            limitations.append(
                "py.blocks sees only the innermost frame of a block (tracemalloc keeps 1 frame in a driver run): a block a library function allocates for project code has no site; DLA_TRACEMALLOC_FRAMES=<n> keeps n frames"
            )
    except BaseException:
        pass
    try:
        counts, capped = _count_blocks()
        if capped:
            limitations.append(
                f"py.blocks keeps the {_BLOCKS_SITES_CAP} sites with the most blocks at this checkpoint: a site below them has no reading here"
            )
        return counts, capped
    except BaseException as exc:
        limitations.append(f"reduced record: py.blocks could not be counted ({exc!r}), so the allocation-site columns are missing")
        return None


# ---------------------------------------------------------------------------------------------
# dumping
# ---------------------------------------------------------------------------------------------


def _clear_interpreter_caches():
    """Empties the interpreter's own caches (the type attribute cache keeps a reference to the
    name string of every attribute lookup, and a name built at run time is a new string each
    time), so that a block it holds is not counted as held by the program. Never raises. When
    the interpreter has no call for it, a limitation line says so, once per process."""
    try:
        clear = getattr(sys, "_clear_internal_caches", None)  # 3.13 and later
        if clear is None:
            clear = getattr(sys, "_clear_type_cache", None)  # before 3.13 (deprecated after)
        if clear is not None:
            clear()
            return
    except BaseException:
        pass
    note(
        "the interpreter's attribute cache could not be emptied: lines that build attribute names at run time can show live blocks that the program does not hold"
    )


def _build(phase, x, last):
    limitations = []
    tick = phase == "tick"
    if not tick:
        mark_phase(f"{phase} record at x={x}: reading the process")
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
        _clear_interpreter_caches()
    values = {}
    # The blocks are counted before any figure of this record is read (see "py.blocks").
    counted = None
    if phase == "checkpoint":
        if _traced() is None:
            limitations.append("tracemalloc not tracing: the py.blocks columns are unavailable")
        else:
            counted = _blocks_checkpoint(limitations)
    traced = _traced()
    if traced is not None:
        values["py.traced"] = traced[0]
        values["py.traced_peak"] = traced[1]
    else:
        limitations.append("tracemalloc not tracing: py.traced unavailable")
    objs = None if tick else gc.get_objects()
    values["py.gc_objects"] = len(objs) if objs is not None else len(gc.get_objects())
    values["py.gc_collections"] = sum(s.get("collections", 0) for s in gc.get_stats())
    os_own = _os_values()
    values.update(os_own)
    rec["values"] = values
    if counted is not None:
        rec["group_counts"] = {"py.blocks": counted[0]}
        if counted[1]:
            rec["group_capped"] = ["py.blocks"]
    if pre is not None:
        rec["values_pre_gc"] = pre
    if want_purge:
        rec["purge"] = _purge(purge, os_before_gc, os_own, limitations)
    if not tick:
        types_full = _types_top(objs, _TYPES_TOP if _CAPTURE_ON and "histogram" in _CAPTURE_KINDS else 20)
        rec["types"] = types_full[:20]
        if _CAPTURE_ON:
            mark_phase(f"{phase} record at x={x}: raw captures")
            captures = _capture_now(phase, x, last, types_full, objs, limitations)
            if captures:
                rec["captures"] = captures
        del objs, types_full
        mark_phase(f"{phase} record at x={x}: probes")
        rec["probes"] = _probes(limitations)
        # A driver writes no `exit` record, so its last checkpoint carries the section, and so do
        # the first, second, fourth, eighth... (a driver cut off by its timeout never writes the
        # last one). The harness takes the union of all of them.
        if phase == "exit" or (phase == "checkpoint" and (last or bin(_checkpoint_count).count("1") == 1)):
            if _coverage_on:
                rec["coverage"] = {"kind": "functions", "items": sorted(_coverage)}
                if len(_coverage) >= _COVERAGE_CAP:
                    limitations.append(f"coverage capped at {_COVERAGE_CAP} functions")
            else:
                limitations.append("function coverage unavailable (sys.monitoring needs Python 3.12+)")
    try:
        _freeze_record(rec, limitations)
    except BaseException as exc:
        limitations.append(f"reduced record: py.gc_frozen could not be read: {exc!r}")
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
    global _checkpoint_written, _checkpoint_count
    _checkpoint_written = True
    _checkpoint_count += 1
    dump("checkpoint", x, last)


def note(text):
    """A line for the limitations of every following record (a driver says what it changed)."""
    line = str(text)
    if line not in _cap_notes:
        _cap_notes.append(line)


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
    mark_phase("start-up: the program's own code before its first hook or checkpoint (the imports of a driver, or the workload of a command)")
    _preload_trim()
    _start_tracemalloc()
    if _CAPTURE_ON and "nonheap" in _CAPTURE_KINDS:
        _install_thread_sites()
        _install_asyncio_debug()
        _install_resource_warnings()
    _start_memray()
    _start_coverage()
    _install_gc_freeze()
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
