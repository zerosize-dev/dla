# Python agent and driver

| File | Role |
|---|---|
| `sitecustomize.py` | agent; must be first on `PYTHONPATH` (the harness directory) |
| `dla_memray_leaks.py` | helper that turns a memray capture into bounded JSON; run with the target's interpreter, written next to the agent |
| `dla_harness.py` | loop driver template |
| `test_dla_harness.py` | pytest-memray variant of the driver; skipped unless `pytest_memray` imports |

## Run

Command mode (python 3.8+; coverage needs 3.12+):

```
cd <scratch project root>
PYTHONPATH=<harness dir>[:<existing PYTHONPATH>] DLA_HARNESS_OUT=... python3 -m pkg.load --iterations N
```

The agent chain-imports the next `sitecustomize` found on `sys.path` (the project's, or the
system's), skips `pip`/`venv`/`ensurepip`/`build` runs, starts tracemalloc (`DLA_TRACEMALLOC_FRAMES` frames: 10 when `--capture alloc` or `nonheap` is asked for, else 1) and, on
3.12+, a `sys.monitoring` `PY_START` collector (each code object disables itself after its first
event). Coverage items look like `reports.builder:ReportBuilder.render@reports/builder.py:18`
and cover only files under `DLA_PROJECT`.

## What the dump records

- `values` after `gc.collect()` twice: `py.traced` (tracemalloc current bytes), `py.traced_peak`,
  `py.gc_objects` (`len(gc.get_objects())`), `py.gc_collections`, and the `os.*` values from
  `/proc/self/status`. `values_pre_gc` holds `py.traced`, `py.gc_objects`, `os.rss` read before
  the collection (this is how cyclic garbage waiting for the collector shows up).
- `types`: top 20 types by instance count with summed `sys.getsizeof`.
- `groups`, only after the target has called `gc.freeze()`: `py.gc_frozen`, objects in the
  permanent generation by the project call site that froze them (`{"journeys/worker.py:25": 26000}`).
  The agent wraps `gc.freeze` and `gc.unfreeze` (the real call and its errors are the target's; the
  bookkeeping never raises), attributes the rise of `gc.get_freeze_count()` to the innermost project
  frame of the caller (`<non-project>` when none), and writes each site's share of the current count
  in proportion to what it froze, so the shares sum to `gc.get_freeze_count()` (frozen objects freed
  by reference counting leave the count, so a cumulative sum would exceed it). `gc.unfreeze()` sets
  every share to zero; the sites stay in the record. Nothing is written when `freeze` was never
  called. Whenever the freeze count is above zero (whoever froze), a limitation line says that
  `py.gc_objects` and the type counts exclude those objects: the permanent generation is not in
  `gc.get_objects()`. An interpreter without `gc.freeze` or `gc.get_freeze_count` (before 3.7) is left alone.
- `probes`:
  - `PyModule`: every module-level name that is a container (list, dict, set, deque, mutable
    mapping/sequence/set), an `lru_cache`/`cache` wrapper (`size` = `cache_info().currsize`), or an
    instance of a non-stdlib type whose reachable count is above 1; plus, for classes defined in
    that module, class-level containers and cache wrappers, reported as `Class.attr` (this is how
    a memoised method is found).
  - `PyGlobal`: one named module-level value. `PyAttr`: dotted `qualname` resolved from the module
    (`ReportBuilder.category_totals`; an `lru_cache` gives `currsize`).
  - `reachable`: iterative walk over `gc.get_referents` with an `id()` visited set, bounded at
    2,000,000 objects, `bytes` = sum of `sys.getsizeof`. Types, modules, code and frames are not
    followed; functions are followed only into their closure and defaults; bound methods into
    `__self__`; objects of `logging`, `threading`, `asyncio`, `concurrent`, `multiprocessing`,
    `socket`, `ssl` are counted but not followed.
  - A module in the probe that is the running `__main__` is found through `__main__.__spec__`.
  - A probe never imports. A module is read only when the workload has already imported it (it is in
    `sys.modules` when the checkpoint runs); importing runs the module's top level, which can start a
    server or block, and changes the state under measurement. A module that is not loaded gives a probe
    record with an `error` and no values (so no column) and one limitation line naming up to five such
    modules. A module-level `lru_cache` function is read as a `PyGlobal`: its `size` is `currsize`.
- `phase`: the agent writes what the process is doing to `<DLA_HARNESS_OUT>.phase` (one line, overwritten):
  `start-up: the program's own code before its first hook or checkpoint` (the imports of a driver, or the workload of a command) from the moment it loads, `hook setup` and
  `hook unit (round r of n ...)` from the driver (`sitecustomize.mark_phase`), and
  `checkpoint record at x=...: probes` with `checkpoint: probe P3 (pymodule m)` while a checkpoint reads a
  probe. `dla harness run` appends the last line to the note of a point it killed at its timeout.
- `purge`: `trim` calls `malloc_trim(0)` through ctypes (glibc only) and records `os.*` before and
  after, `before` being the record's own reading; ctypes and libc are loaded and `malloc_trim`
  called once at start-up in every process on Linux (the purge is asked only of the process of the
  last point), so the load does not show as memory the purge failed to return; `gc` records the
  `os.*` values before the forced collection and after it.
- `coverage`: `{"kind":"functions","items":[...]}` on 3.12+, else a limitation.

## Raw captures (`dla harness run --capture histogram,alloc,nonheap`)

The agent writes files into `DLA_CAPTURE_DIR/<pid>/` and lists them in the `captures` array of the
record that took them (kind, file, role, bytes, ms, or an `error`). Heavy captures are taken at the
same checkpoints as the JVM's (`DLA_CAPTURE_DUMPS`, default 4: the end of the warm-up, the last round
and rounds in between); command mode takes one set at exit. Nothing here raises into the target.

| `--capture` | File | `dla summarize` | Content |
|---|---|---|---|
| `alloc` | `py-<n>.alloc.json` | `alloc-stack-diff` (`py-tracemalloc`) | live bytes and blocks per allocation stack after a full collection, innermost frame first, as deep as `DLA_TRACEMALLOC_FRAMES`; the largest stacks plus, after the first file, those that grew most since it. Files in a project are written relative to `DLA_PROJECT`. The agent's own frames are left out. Never a pickle or a tracemalloc dump. |
| `alloc` | `py-1.memray.json` | `alloc-stack-diff` (`py-memray`) | allocations still live when the run ended, from memray, when memray imports in the target's interpreter; `DLA_PY_MEMRAY=0` leaves it out |
| `histogram` | `py-<n>.types.json` | `type-histogram-diff` (`py-types`) | the 300 types with the most instances in `gc.get_objects()`, with shallow `sys.getsizeof` bytes |
| `nonheap` | `py-<n>.nonheap.json` | `non-heap` (`py-resources`) | threads by name pattern (digit runs as `*`) and by creation site, pending asyncio tasks by coroutine, frame and creation site, ResourceWarnings by allocation site |

memray: the tracker starts at interpreter start-up (only when `alloc` is captured and memray imports
here) and stops at the start of the final record, so the agent's own work is not in the report. The
helper `dla_memray_leaks.py` then runs with `sys.executable -I` (given `--site <dir>`, the directory memray was imported from, so a user-site or PYTHONPATH install is found) and writes `get_leaked_allocation_records`
(merged threads) plus `memray stats --json` figures as JSON; DLA never reads the memray file. If memray
is not installed in that interpreter, the `alloc` capture list carries an entry with an `error` that
says so and names `pip install memray`; the run records it as a coverage gap. guppy3 and objgraph are
not used.

A run with `--capture alloc` is for attribution, not for the trend: tracemalloc with several frames and
memray add memory and time to the points that take them (the `os.rss` of those points is higher).

Creation sites: with `nonheap` the agent wraps `threading.Thread.__init__` (the site is the innermost
project frame that is not an `__init__`), sets `PYTHONASYNCIODEBUG=1` so tasks keep their creation
stack (and quiets the `asyncio` logger), and counts `ResourceWarning`s through the warnings machinery
with the allocation traceback tracemalloc keeps for the object.

## Driver

`PYTHONPATH=<harness dir>:. python <harness dir>/dla_harness.py` from the scratch project root.
It imports `sitecustomize` (already loaded when the path is set) and calls
`checkpoint(units_done, last)` after every round; the agent then writes no `exit` record.
The `coverage` section then rides on the checkpoints (the first, second, fourth, eighth... and the last),
which the harness merges.

`test_dla_harness.py` imports the hooks from `dla_harness.py` and runs the measured units under
`@pytest.mark.limit_leaks("0B")` from pytest-memray, so the leaking call sites are reported by
pytest-memray; DLA keeps the test output as raw evidence. The standard-library driver is the
fallback, and the missing pytest-memray is a recorded coverage gap.

### Placeholders

| Placeholder | Where | Meaning |
|---|---|---|
| `@@DLA_STEP@@` | `dla_harness.py`, `test_dla_harness.py` | units per round (default for `DLA_STEP`) |
| `@@DLA_WARMUP@@` | both | warm-up rounds (default for `DLA_WARMUP`) |
| `@@DLA_ROUNDS@@` | both | measured rounds (default for `DLA_ROUNDS`) |
| `@@DLA_REPEAT@@` | `dla_harness.py` | repeated key space size (default for `DLA_REPEAT`) |

### Hooks

| Hook | Signature | Fill with |
|---|---|---|
| `DLA-HOOK setup` | `setup()` in `dla_harness.py` | import the project modules and build the objects under test; keep them in `_state` or module globals |
| `DLA-HOOK unit` | `unit(i, key)` in `dla_harness.py` | one unit of the workload for `key`; derive ids from `key` so a repeated key reproduces an earlier request |

The pytest variant has no hooks of its own; it reuses the two above.
