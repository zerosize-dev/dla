"""DLA measurement agent for Python processes.

Placed first on PYTHONPATH by `dla harness run`. It chain-imports the project's own
sitecustomize (if any), starts tracemalloc (1 frame) and, on Python 3.12+, a sys.monitoring
PY_START coverage collector, and writes one JSON record per dump to the file named by
DLA_HARNESS_OUT: an `exit` record from an atexit handler, `checkpoint` records when a driver calls
`checkpoint()`, and optional light `tick` records every DLA_TICK_MS.

Environment: DLA_HARNESS_OUT, DLA_HARNESS_PROBES (probes.json), DLA_PURGE (trim|gc|none),
DLA_PROJECT (scratch root), DLA_TICK_MS, DLA_ITER. The module never raises into the target.
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


def _start_tracemalloc():
    try:
        import tracemalloc

        if not tracemalloc.is_tracing():
            tracemalloc.start(1)
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
        rec["types"] = _types_top(objs)
        del objs
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
