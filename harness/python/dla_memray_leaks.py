"""Reads a memray capture file and writes bounded JSON: the allocations still live when the
tracker stopped, grouped by stack, and memray's own summary.

Run with the interpreter that has memray installed (the one that made the capture file):

    python -I dla_memray_leaks.py <capture.bin> <out.json> [--top N] [--site DIR]

--site is the directory memray was imported from in the target (a user site or PYTHONPATH
install that -I would hide); it is put on sys.path before memray is imported.

DLA never parses the capture file itself (its format is private to memray); this script is the
only reader. Frames are written innermost first as [function, file, line]; a file inside
DLA_PROJECT is written relative to it. Exit status 0 on success, 2 when memray cannot be imported
or the file cannot be read (the reason is on stderr).
"""

import json
import os
import subprocess
import sys

MAX_RECORDS = 200
MAX_FRAMES = 24
FILTER_FRAMES = 512
MAX_BYTES = 4 << 20
STATS_TIMEOUT_S = 60


def _project():
    root = os.environ.get("DLA_PROJECT")
    return os.path.realpath(root) if root else None


def _rel(filename, root, cache):
    got = cache.get(filename)
    if got is not None:
        return got
    out = filename
    if root and filename and not filename.startswith("<"):
        low = filename.replace("\\", "/")
        if "/site-packages/" not in low and "/dist-packages/" not in low:
            real = os.path.realpath(filename)
            prefix = root + os.sep
            if real.startswith(prefix):
                out = real[len(prefix):].replace(os.sep, "/")
    cache[filename] = out
    return out


def _frames(rec, limit=FILTER_FRAMES):
    """The record's frames, innermost first. memray's default `max_stacks` is not reliable
    (it can return one frame or none), so the limit is always given."""
    try:
        return list(rec.stack_trace(max_stacks=limit))
    except BaseException:
        return []


_SITE = None
_RUN_MEMRAY = (
    "import sys, runpy; sys.path.insert(0, sys.argv[1]); sys.argv = ['memray'] + sys.argv[2:]; "
    "runpy.run_module('memray', run_name='__main__')"
)


def _stats_command(bin_path, tmp):
    args = ["stats", "--json", "--force", "-o", tmp, bin_path]
    if _SITE:
        return [sys.executable, "-I", "-c", _RUN_MEMRAY, _SITE] + args
    return [sys.executable, "-I", "-m", "memray"] + args


def _stats(bin_path, out_path):
    """memray's own `stats --json`, reduced to a few figures. Never raises."""
    tmp = out_path + ".stats"
    try:
        subprocess.run(
            _stats_command(bin_path, tmp),
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            timeout=STATS_TIMEOUT_S,
        )
        with open(tmp, encoding="utf-8") as fh:
            raw = json.load(fh)
        keep = ("total_num_allocations", "total_bytes_allocated", "peak_memory", "allocator_type_distribution")
        return {k: raw[k] for k in keep if k in raw}
    except BaseException as exc:
        return {"error": f"memray stats failed: {type(exc).__name__}"}
    finally:
        try:
            os.remove(tmp)
        except OSError:
            pass


def main(argv):
    if len(argv) < 3:
        sys.stderr.write("usage: dla_memray_leaks.py <capture.bin> <out.json> [--top N]\n")
        return 2
    global _SITE
    bin_path, out_path = argv[1], argv[2]
    if "--site" in argv:
        try:
            _SITE = argv[argv.index("--site") + 1] or None
        except IndexError:
            pass
    if _SITE and os.path.isdir(_SITE) and _SITE not in sys.path:
        sys.path.insert(0, _SITE)
    top = MAX_RECORDS
    if "--top" in argv:
        try:
            top = max(1, min(MAX_RECORDS, int(argv[argv.index("--top") + 1])))
        except (IndexError, ValueError):
            pass
    try:
        import memray
    except BaseException as exc:
        sys.stderr.write(f"memray cannot be imported by {sys.executable}: {exc!r}\n")
        return 2
    try:
        reader = memray.FileReader(bin_path)
        records = list(reader.get_leaked_allocation_records(merge_threads=True))
        meta = reader.metadata
    except BaseException as exc:
        sys.stderr.write(f"memray cannot read {bin_path}: {exc!r}\n")
        return 2
    root = _project()
    cache = {}
    # Allocations made by the DLA agent itself (its snapshots, its file writes) are not the program's.
    agent_dir = os.path.dirname(os.path.realpath(__file__)) + os.sep
    own = {}

    def agent_frame(filename):
        got = own.get(filename)
        if got is None:
            got = not str(filename).startswith("<") and os.path.realpath(str(filename)).startswith(agent_dir)
            own[filename] = got
        return got

    records = [r for r in records if not any(agent_frame(f[1]) for f in _frames(r))]
    records.sort(key=lambda r: (-r.size, -r.n_allocations))
    total_bytes = sum(r.size for r in records)
    total_blocks = sum(r.n_allocations for r in records)
    rows = []
    for rec in records[:top]:
        frames = _frames(rec, MAX_FRAMES)
        try:
            allocator = memray.AllocatorType(rec.allocator).name.lower()
        except BaseException:
            allocator = str(rec.allocator)
        rows.append(
            {
                "size": int(rec.size),
                "n": int(rec.n_allocations),
                "allocator": allocator,
                "frames": [[f[0], _rel(f[1], root, cache), int(f[2])] for f in list(frames)[:MAX_FRAMES]],
            }
        )
    doc = {
        "dla_memray": 1,
        "memray": getattr(memray, "__version__", None),
        "python": sys.version.split()[0],
        "records": len(records),
        "leaked_bytes": int(total_bytes),
        "leaked_blocks": int(total_blocks),
        "total_allocations": int(getattr(meta, "total_allocations", 0) or 0),
        "peak_memory": int(getattr(meta, "peak_memory", 0) or 0),
        "native_traces": bool(getattr(meta, "has_native_traces", False)),
        "stats": _stats(bin_path, out_path),
        "stacks": rows,
    }
    text = json.dumps(doc, separators=(",", ":"))
    while len(text) > MAX_BYTES and doc["stacks"]:
        doc["stacks"] = doc["stacks"][: len(doc["stacks"]) // 2]
        doc["truncated"] = True
        text = json.dumps(doc, separators=(",", ":"))
    with open(out_path, "w", encoding="utf-8") as fh:
        fh.write(text + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
