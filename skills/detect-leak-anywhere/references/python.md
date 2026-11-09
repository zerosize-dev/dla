# Python

Worker name and runtime id: `python`. Facts from `harness/python/README.md` and `docs/VERIFY.md`.

## Harness
- Command mode (3.8+; coverage needs 3.12+): `sitecustomize.py` first on `PYTHONPATH`, chain-
  imports the next `sitecustomize`, skips pip/venv/ensurepip/build runs, starts `tracemalloc` and,
  on 3.12+, a `sys.monitoring` coverage collector. Driver mode calls `checkpoint(units, last)` each
  round; fill the hooks. A pytest-memray variant is skipped unless `pytest_memray` imports (a
  recorded gap).
- What the dump measures (after `gc.collect()` twice): `py.traced`, `py.traced_peak`,
  `py.gc_objects`, `py.gc_collections`, the `os.*` set, top-20 types by count with
  `sys.getsizeof`, and probes of module-level containers, `lru_cache` wrappers (`currsize`) and
  class attributes (`PyModule`, `PyGlobal`, `PyAttr`). `values_pre_gc` shows cyclic garbage waiting
  for the collector.
- Purges: `trim` (glibc `malloc_trim` through ctypes) and `gc`.

## Attach backends
| Backend | Tier | Notes |
|---|---|---|
| `dla attach memray --pid <P> --op record [--duration 30s]` | 3 | allocation stacks, not contents; memray must be importable by the target's own interpreter (checked first, refused otherwise); gdb injection below 3.14; 4 s capture took 8.6 s wall on 3.13; the 3.14 `sys.remote_exec` path was not run |
| `dla sample --pid <P> --counters python-remote` | 2 | runs a small generated script in the target; needs Python 3.14 or later |

## Artifacts
- memray capture files and stats land under `attach/P<n>/`. No `dla summarize` adapter reads
  memray files in this release (cut 2). Record them as artifacts; do not parse them by hand.

## Blind spots
- Cycles the collector has not run yet look like leaks; only a forced collection separates a
  pile-up from a surviving cycle.
- Only collector-tracked objects are visible: extension modules, native libraries and pymalloc
  pools returned late are outside object-graph tools.
- Allocation tracing slows the process and changes timing.
- Reference-count errors in C extensions show only as native growth.
- Task and thread leaks in async code are visible only while the tasks can be enumerated.
