# Python agent and driver

| File | Role |
|---|---|
| `sitecustomize.py` | agent; must be first on `PYTHONPATH` (the harness directory) |
| `dla_memray_leaks.py` | helper that turns a memray capture into bounded JSON; run with the target's interpreter, written next to the agent |
| `dla_harness.py` | loop driver template |
| `dla_pytest_loop.py` | loop driver over the project's own tests (`dla harness gen --lang python --pytest`): no hooks |
| `test_dla_harness.py` | pytest-memray variant of the driver; skipped unless `pytest_memray` imports |

## Run

Command mode (python 3.8+; coverage needs 3.12+):

```
cd <scratch project root>
PYTHONPATH=<harness dir>[:<existing PYTHONPATH>] DLA_HARNESS_OUT=... python3 -m pkg.load --iterations N
```

The agent chain-imports the next `sitecustomize` found on `sys.path` (the project's, or the
system's), skips `pip`/`venv`/`ensurepip`/`build` runs, starts tracemalloc (`DLA_TRACEMALLOC_FRAMES` frames: 10 when `--capture alloc` or `nonheap` is asked for, else 1, in command and driver mode alike: see "Allocation sites" for the depth) and, on
3.12+, a `sys.monitoring` `PY_START` collector (each code object disables itself after its first
event). Coverage items look like `reports.builder:ReportBuilder.render@reports/builder.py:18`
and cover only files under `DLA_PROJECT`, plus files under `DLA_SOURCE_ROOT` (a package installed in editable mode runs from there), named by their path relative to it.

## What the dump records

- `values` after `gc.collect()` twice: `py.traced` (tracemalloc current bytes), `py.traced_peak`,
  `py.gc_objects` (`len(gc.get_objects())`), `py.gc_collections`, and the `os.*` values from
  `/proc/self/status`. `values_pre_gc` holds `py.traced`, `py.gc_objects`, `os.rss` read before
  the collection (this is how cyclic garbage waiting for the collector shows up).
- `types`: top 20 types by instance count with summed `sys.getsizeof`.
- `group_counts`, in every checkpoint record of a driver run: `py.blocks`, the live blocks tracemalloc
  traces per allocation site, all counted sites (see "Allocation sites" below); `group_capped` names
  the counters a record cut at the bound of 5000 sites.
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

### Allocation sites (`py.blocks@<file:line>`, driver runs)

At every checkpoint, after the two collections the record already forces, the agent takes a
tracemalloc snapshot, groups it by traceback and counts the live blocks per site. The site of a block
is the first frame, from the innermost outward, that lies in the project (under `DLA_PROJECT`, or under
`DLA_SOURCE_ROOT`, the project the scratch copy was made from, because a library installed in editable
mode runs from there and not from the copy; its `.dla` directory, `.git` and installed packages never
count) and is neither a test file nor a file of the harness (the agent's directory, the driver). A test
file follows the rule `dla workload discover` uses for Python (`FileInfo.is_test`): a file under a
directory named `test`, `tests`, `__tests__`, `__mocks__`, `testing`, `spec` or `specs`, `test_*.py`,
`*_test.py`, `conftest.py`, and the entry files `setup.py` and `__main__.py`.

The agent keeps nothing between checkpoints that grows with their number (only the file-name caches,
which fill once): every checkpoint record carries its own counts as
`group_counts: {"py.blocks": {"<file:line>": <blocks>}}`, at most 5000 sites (the largest counts; a record
that cut the list also has `group_capped: ["py.blocks"]` and a limitation line). `dla harness run` picks the
columns from the records of the whole run: a site is kept when its count at the last point that has counts
is above its count at the first such point after the warm-up (`DLA_WARMUP`); at most 12, the largest growth
first, and `N more allocation sites grew and have no column` says how many more. A kept site reads 0 at a
point whose record lists the counter without it; at a point cut at the bound it is left empty (not 0), a
site with no reading at the first or last point is not judged, and a limitation line says so. A process
cut before its last checkpoint is judged on the records it wrote (two points after the warm-up are
enough). In a process where tracemalloc is not tracing there are no columns and a limitation line. Command
mode and the other runtimes are unchanged.

Order inside a checkpoint: the two collections, then the snapshot and the grouping (with the collector
switched off meanwhile), then `py.traced`, `py.traced_peak` and the OS values. The snapshot's transient
memory is thus in every reading alike, the warm-up checkpoints' and the first measured one's included; read
before it, the first points would be cheaper than the later ones and the difference would read as growth.
The counts dict of a checkpoint lives until its record is written, the same size on every point of a run
whose number of sites is stable.

Depth. With one frame only the innermost frame of a block is known, so a block a library function
allocates for project code has no site. `DLA_TRACEMALLOC_FRAMES` sets the depth (a capture with `alloc`
keeps 10). The default of a driver run was chosen by measuring the time of the run at 1, 2, 4 and 8
frames, the deepest whose time is at most 1.25 times the time at one frame (milliseconds per unit,
fresh keys, 9 points, snapshot and grouping at every checkpoint included):

| Frames | 1 | 2 | 4 | 8 |
|---|---|---|---|---|
| one text-drawing call of an imaging library per unit (mean of 3 runs) | 0.66 | 0.94 (1.42x) | 1.26 (1.90x) | 2.38 (3.6x) |
| two test files of the project per unit (1 run) | 3313 | 4417 (1.33x) | 6133 (1.85x) | 10222 (3.1x) |
| snapshot and grouping, mean per checkpoint, first unit | 78 ms | 111 ms | 165 ms | 423 ms |
| snapshot and grouping, mean per checkpoint, second unit | 680 ms | 966 ms | 1302 ms | 3152 ms |

Both units exceed the bound at two frames, so a driver run keeps one frame and says so in a limitation
line (`py.blocks sees only the innermost frame of a block`).

### Reduced records

A full exit or checkpoint record always has `values_pre_gc`, `types`, `probes` and `limitations`.
`dla harness run` reads each point's record with a typed reader that, when a section cannot be read
(a value of the wrong type), drops the sections one by one until the record parses and keeps its
values; it remembers which it dropped, and a Python record that has `values_pre_gc` but lacks
`types`, `probes` or `limitations` counts as cut short the same way. Either gives the series a limitation
`reduced record: the record of point N lost ...`, the run output an `_attention` line
`REDUCED RECORD: ...`, and a target whose probe has no column reads `unknown` in `dla trend` (with the
command that measures again) instead of `not_loaded`. The agent also says `reduced record: ...` itself when
`gc.freeze` was called and `py.gc_frozen` could not be written (a failed site lookup keeps the group under
`<non-project>`; any other failure of that section is a limitation line).
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

### The pytest loop (`dla_pytest_loop.py`)

`dla harness gen --lang python --pytest [--tests a,b]` writes a driver with nothing to fill. One unit is one
`pytest.main` of the selected test files in this process, run from the working directory of the tests with the
directory first on `sys.path` (and `<cwd>/src` when that directory holds a package or a module). A small plugin
counts, per test file, the tests that passed, failed and were skipped and the time they took. The arguments keep
the run quiet and in this process: `-o addopts=` (the project's own addopts may ask for parallel workers or a
coverage plugin), `-p no:cacheprovider`, `-p no:xdist`, `-q`, `--tb=short` and `--rootdir <cwd>`; what pytest and
the tests print goes to a temporary file and only a bounded head of it reaches the harness log (the first run, and
the first later run with a failure). `SystemExit` of a test is caught. Rounds as the generic driver
(`DLA_WARMUP`, `DLA_ROUNDS`, `DLA_STEP`, a checkpoint after each round) with a step of at least 1; the default step is 1.

After the first run the driver:

1. checks where the code comes from: every top-level package or module the project defines (under the working
   directory or its `src/`) that the tests imported must lie under the scratch copy (`DLA_PROJECT`) or under
   the project it was made from (`DLA_SOURCE_ROOT`); otherwise it stops with exit code 3 and the `DLA-PYTEST the tests import
   <name> from <path>, which is outside the project: ...` line. The scratch copy is made when the harness is generated
   (`dla harness gen`), not at each run, and keeps the built extension modules (untracked `*.so`, `*.pyd` and
   `*.dylib` files in a directory that holds Python sources); build the project in place before generating;
2. drops a file none of whose tests passed when another file had a passing test (limitation `dropped test file
   <path>: none of its tests passed in the first run (first failure: ...)`); no test found stops with exit 3
   (also when every test was skipped), no test passed in any file stops with exit 2; a later run with failing tests is
   counted and its first failure recorded once; exit 2 also when no test passed in any run;
3. estimates the whole series (warm-up plus rounds) from the time of the first run and, when it does not fit
   `DLA_TIME_LIMIT_SECS` (margin 15 percent and 5 seconds), lowers the step (not below 1) and drops the slowest
   files until it fits (limitation `dropped test file <path>: it took <t> s of <unit> s per run and the series would
   not fit <limit> s`); when one file alone does not fit it stops with `the tests are too slow for a series` and exit 3,
   so the run reports no usable point. A reading of a single run holds the one-time costs (the imports), so this stop
   waits for the second run unless one run alone is over the budget. Timing restarts after a drop.

It records the files it runs in `pytest-files.json` next to the dump (the bundle of a finding lists them) and ends
with `DLA-PYTEST summary: units= tests_run= files= dropped= failed_units= first_failure=` on stderr. The driver file
counts as a harness file: its lines have no `py.blocks@` column.

### Placeholders

| Placeholder | Where | Meaning |
|---|---|---|
| `@@DLA_STEP@@` | `dla_harness.py`, `test_dla_harness.py` | units per round (default for `DLA_STEP`) |
| `@@DLA_WARMUP@@` | both | warm-up rounds (default for `DLA_WARMUP`) |
| `@@DLA_ROUNDS@@` | both | measured rounds (default for `DLA_ROUNDS`) |
| `@@DLA_REPEAT@@` | `dla_harness.py` | repeated key space size (default for `DLA_REPEAT`) |
| `@@DLA_TESTS@@` | `dla_pytest_loop.py` | the test files or directories to run, a Python list literal |

### Hooks

| Hook | Signature | Fill with |
|---|---|---|
| `DLA-HOOK setup` | `setup()` in `dla_harness.py` | import the project modules and build the objects under test; keep them in `_state` or module globals |
| `DLA-HOOK unit` | `unit(i, key)` in `dla_harness.py` | one unit of the workload for `key`; derive ids from `key` so a repeated key reproduces an earlier request |

The pytest variant has no hooks of its own; it reuses the two above.
