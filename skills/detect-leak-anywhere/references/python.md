# Python

Worker name and runtime id: `python`. Facts from `harness/python/README.md` and `docs/VERIFY.md`.

Archetype rows (what to look for, how to confirm, how to rule out): read `references/python-archetypes.md` only after `dla gate check --stage classify` passes.

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
- Raw captures: `dla harness run H1 --capture histogram,alloc,nonheap`. `histogram` writes
  `py-<n>.types.json` (300 types by instance count), `alloc` writes `py-<n>.alloc.json` (tracemalloc live
  bytes per stack, innermost frame first; frames: 10 with `alloc`, else 1) and `py-1.memray.json` when
  memray imports in the program's own interpreter (`DLA_PY_MEMRAY=0` leaves it out), `nonheap` writes
  `py-<n>.nonheap.json` (threads by name pattern and creation site, pending asyncio tasks, ResourceWarning
  sites). Run the `dla summarize` commands the harness prints. A run with `alloc` is for attribution, not
  for the trend: tracemalloc with several frames and memray slow the points that take them.

## Attach backends
| Backend | Tier | Notes |
|---|---|---|
| `dla attach memray --pid <P> --op record [--duration 30s]` | 3 | allocation stacks, not contents; memray must be importable by the target's own interpreter (checked first, refused otherwise); gdb injection below 3.14; 4 s capture took 8.6 s wall on 3.13; the 3.14 `sys.remote_exec` path was not run |
| `dla sample --pid <P> --counters python-remote` | 2 | runs a small generated script in the target; needs Python 3.14 or later |

## Artifacts
- memray capture files and stats land under `attach/P<n>/`; the raw `.bin` is not read by any adapter.
  Harness captures are read by `dla summarize`: `py-tracemalloc` and `py-memray` (`alloc-stack-diff`) tie
  by the stack's first project frame; `py-types` (`type-histogram-diff`) ties only when a growing type is
  the finding's accumulation class and never for built-in containers; `py-resources` (`non-heap`) ties a
  thread or task group to the finding at the line that created it.
- memray missing in the target's interpreter is a coverage gap (`pip install memray` there); the
  tracemalloc capture still runs.

## Blind spots
- PyO3 boundaries (a `#[pymodule]` crate that Python imports) are detected and reported by `dla probe`,
  but the allocator shim measures JNI only: native growth under a PyO3 call shows as `os.rss_anon`.
- Cycles the collector has not run yet look like leaks; only a forced collection separates a
  pile-up from a surviving cycle.
- Only collector-tracked objects are visible: extension modules, native libraries and pymalloc
  pools returned late are outside object-graph tools.
- Allocation tracing slows the process and changes timing.
- Reference-count errors in C extensions show only as native growth.
- Task and thread leaks in async code are visible only while the tasks can be enumerated.
