# Python

Worker name and runtime id: `python`. Facts from `harness/python/README.md` and `docs/VERIFY.md`.

Archetype rows (what to look for, how to confirm, how to rule out): read `references/python-archetypes.md` only after `dla gate check --stage classify` passes.

## Harness
- Command mode (3.8+; coverage needs 3.12+): `sitecustomize.py` first on `PYTHONPATH`, chain-
  imports the next `sitecustomize`, skips pip/venv/ensurepip/build runs, starts `tracemalloc` and,
  on 3.12+, a `sys.monitoring` coverage collector. Driver mode calls `checkpoint(units, last)` each
  round; fill the hooks. Every process runs with `PYTHONDONTWRITEBYTECODE=1` (a value the environment has
  is kept), so no point compiles what a later one loads from a cache and no `__pycache__` is written into
  the project. A pytest-memray variant is skipped unless `pytest_memray` imports (a
  recorded gap).
- What the dump measures (after `gc.collect()` twice): `py.traced`, `py.traced_peak`,
  `py.gc_objects`, `py.gc_collections`, the `os.*` set, top-20 types by count with
  `sys.getsizeof`, and probes of module-level containers, `lru_cache` wrappers (`currsize`) and
  class attributes (`PyModule`, `PyGlobal`, `PyAttr`; a module-level `lru_cache` function is a
  `PyGlobal`). A probe reads only modules the workload already imported and never imports one: a
  module that is not in `sys.modules` at the checkpoint has no column and a limitation line. The agent
  writes its current phase to `<dump file>.phase` for a run killed at its timeout.
  `values_pre_gc` shows cyclic garbage waiting for the collector. The interpreter's internal caches
  are emptied too (`sys._clear_internal_caches()`, else `sys._clear_type_cache()`): its attribute cache
  keeps the name strings of lookups, so a line that builds attribute names at run time would show
  blocks the program does not hold.
- Frozen garbage: after a `gc.freeze()` call the record has `py.gc_frozen@<file:line>` (objects in the
  permanent generation, per call site; the shares sum to `gc.get_freeze_count()`), and a limitation line
  says that `py.gc_objects` and the type counts leave those objects out. Signature of the leak: `py.traced`
  keeps growing after the forced collection while `py.gc_objects` and the type counts stay flat or tiny,
  and `py.gc_frozen@site` grows with the work. The census lists a `gc.freeze()` inside a function
  (kind `gc-freeze`, holder cycle-only); a module-level call is the documented use before a fork and is
  not listed. The label is `UNCLASSIFIED` (no Python row names it). The fix: freeze once in the parent
  before the fork, or `gc.unfreeze()` and `gc.collect()` after the batch; deleting the per-batch call also
  works. The frozen count is a count, so the bytes growth is tied to the site through the group column.
- Allocation sites (driver runs only): at each checkpoint, after the forced collections, the agent counts
  the live blocks `tracemalloc` traces per site: the first frame, from the innermost outward, that lies
  in the project and is neither a test file nor a file of the harness. Every record carries its counts
  (the agent keeps nothing between checkpoints); `dla harness run` gives the column
  `py.blocks@<file:line>` (a count, region heap) to a site whose count at the last point is above its count at
  the first point after the warm-up, with a value at every point (0 where the record lacks the site); at most 12 sites, the
  largest growth first, and a limitation line says how many more grew. Sites that did not grow have no
  column. A column ties to the finding whose own site is that line and to no other, so a leaked object
  that nothing holds shows at the line that made it, for example the line of a Python function that calls
  into an extension module and never releases the reference it was given. By default tracemalloc keeps
  one frame in a driver run, so a block allocated inside a library function that project code called
  has no site (the limitation line says so); `DLA_TRACEMALLOC_FRAMES=<n>` keeps more, at the cost
  in `harness/python/README.md`. Command mode has no such columns. A library installed in editable mode
  runs from its source tree, not from the scratch copy; the agent counts the files of that tree too (the
  run's `.dla`, installed packages and the test files are never counted).
  A growing site that no finding names: `dla trend S<n>` prints `dla ledger add --origin growth-series
  --evidence E --site <file:line> --region heap --holder <kind>`. Choose the holder by reading the line: a
  container of the project that keeps the objects is `static/global`, `container-field` or
  `registration-table`; a line that calls into an extension module or a C function, with nothing in the
  program holding the result, is `native-handle/global-ref` (a reference count that is never released
  keeps a managed object alive, so the finding's region is heap; the archetype table lists `L-OWNERSHIP`
  for the heap too, but the python reference has no row for it, so `dla ledger label` asks for `--note`
  or `UNCLASSIFIED`). Then the route is
  the one of any finding: the non-leak sibling (`N-WARM`), `dla ledger test`, `dla alloc-test H --finding F`
  (it decides by the `py.blocks@` column of the finding's line, not by `py.traced`, which a unit that runs
  a test runner grows on its own), refute the loser, `dla ledger confirm`. The final gate lists a growing
  `py.blocks@` column that ties to no dispositioned finding, and `dla ledger resolve` refuses to close a
  finding whose column passed until an alloc-test is bound to it. After `dla alloc-test` favors leak,
  capture the stacks that lead to the line and the objects that hold it: run the `dla harness run H --mode <m>
  --capture alloc,heap` command it prints first, then the two `dla summarize` commands that run prints
  (`alloc-stack-diff` ties the stacks at the line to the finding, `heap-graph` names who holds the objects),
  then label, `dla bundle` and explain; `dla bundle` is refused with `stacks-not-captured` for a confirmed
  finding at such a line until that capture was made and every summarize command of it was run. The frame
  above the growing line shows who receives the object: for a callback or a wrapper object it is a function
  of an extension module, so read that module's source for the reference it keeps.
- Who holds the objects of a growing line: a `py.blocks@<file:line>` column says where the surviving objects
  were made, not who keeps them, and in a loop over the project's tests several lines often grow because one
  holder keeps them all (a module-level object that stores an exception and raises that same object again keeps
  every raising frame and its locals through the growing traceback). The route is a growing line, then who
  holds it, then one finding at the holder. Run `dla harness run H --mode <m> --capture alloc,heap` as soon as a trend
  lists growing lines that no finding is at (the trend says so in `holders_hint`) and the two `dla summarize`
  commands that run prints (`alloc-stack-diff` for the stacks, `heap-graph` for the holders): one capture serves
  both, and a stack summary made before the finding was added counts for the finding once it exists. Its record lists the holders: `static:<module>.<name>` is a module
  global or a class attribute (static/global), `stack:<function>@<file:line>` a function frame
  (blocked-stack), and a holder keyed by a bare `file:line` with `unreferenced` is held from native code or
  by a reference lost there. Each holder lists the lines it keeps (`sites`: it keeps at least a quarter of the
  objects sampled for the line; the others are `minor_sites` and tie nothing) with their block counts at the
  first and last dump (`first` is null for a file of one dump point), the types of the objects it keeps
  (`types`: the kept object of each chain and a step that repeats; the `py.type:<T>.count` columns of those
  types are the holder's too), the kept objects found and `path`, the chain with the most objects from a kept object to the
  holder (`via` is how the next step refers to it, `at` a place in code, `repeat` a run of the same step, `ats` the distinct places that run names).
  `next` prints the `dla ledger add ... --site <the place nearest the holder> --owner <module.name>` command
  for a holder that no finding names: add one finding there, and it accounts for every line the holder keeps
  (the final gate lists those lines as tied to that finding and no add command for them); then the route of
  the finding as above. When one holder keeps several of the lines, never add a finding per line. A line whose
  block growth is far larger than the objects found for it (under a tenth, for a growth of 20 blocks or more)
  also has the native entry, and a record that says `no_chain` (the command output says `no_chain_sites`) names
  the lines that have kept objects and no chain, with the reason (the time bound or another limit). A holder
  that is native (`unreferenced`: no Python container refers to the blocks, so the test runner, pytest or the
  tracer do not hold them) is explained by the extension function the line calls: read it for the reference
  it keeps, and add the finding at the line. The finding `next` proposes is never at a place outside the project: it is at the first step from the holder that names a project line, preferring a line the holder keeps that grew most, and `dla ledger add` warns when its `--owner` is a holder of a record and its `--site` is none of the lines that holder keeps. The bundle of a finding shows only the holder entries that hold its line
  or are owned by its owner (`other_holders` says how many were left out; an allocation-stack record is cut the same way to the stacks at the finding's line, `other_stacks`), and a retention record stands in for
  the stack only when such an entry has a chain: an `unreferenced` entry does not, so the stack summary is still
  required for a native line. Names of files are relative to the project in the copy and in the source tree of
  an editable install alike; a file outside the project is `<outside>/<name>`. Values are not written (a dict
  key is shown as its type) unless the run's consent allows raw values.
- A table at module level that the project fills at import time through a registration function (plugin
  modules call it, or a wrapper of it, at their top level), and that a driver grows by calling that function
  or its wrapper with new keys, is reported as grown by the registration function itself, not as reproduced
  through the public API; it is a leak only if run-time input reaches that call. Any hook driver whose unit calls the registration function (or a wrapper the census recorded) gives that reading, and `dla ledger confirm` refuses such a finding with `registration-only`; one entry-path harness whose unit does not call it decides both readings (it grows there: confirm; it stays flat: `dla alloc-test` favors non-leak, refute the finding and confirm an `N-INTENT` sibling).
- The project's own tests in a loop (`dla harness gen --lang python --pytest [--tests a,b]`): a leak that
  nothing holds (a reference an extension module never releases) has no static candidate and shows only as an
  allocation line that grows under a broad workload; the broadest workload a project has is its own tests.
  `dla census --lang python` writes the workload discovery result when the run has none and the project has
  test files, so no separate `dla workload discover` is needed. The harness runs every test file the workload discovery lists under the working directory that has the
  most of them (files under another directory are named in a gap line and left out), or the files and
  directories of `--tests` (no `file::test` ids), with `pytest.main` in one process, again and again; the
  driver is hookless (nothing to fill) and declares no target. A round is one run of the tests (step 1,
  1 warm-up and 8 measured runs, 1800 s). The project's addopts are cleared (`-o addopts=`) and
  the cache and xdist plugins are off, so the run stays in this process. After the first run it drops a
  file none of whose tests passed when another file had one (`dropped test file ...`), stops when no test
  passed (`DLA-PYTEST` line, exit 2) or no test was found (exit 3), drops the slowest files until the series fits
  the time limit and gives no point when one file alone does not fit. A run that pytest ends early (an exit code other than 0 and 1, or no test executed, from the second run on) is made again and is not a unit: at most 2 repeats per unit and 4 in the loop, then it counts as a dead unit and the summary line says `cut_units=`; a test file that fails to import costs its own tests only. It needs pytest in the harness
  interpreter (`python3` on PATH: `python3 -m pip install pytest`, or put the `bin` directory of a virtual
  environment that has pytest and the project first on PATH for the `dla` commands; without it the command
  is refused and the queue goes on with the driver batches) and the project importable in place:
  the scratch copy is made when the harness is generated and keeps the extension modules built in place
  (untracked `*.so`, `*.pyd` or `*.dylib` files next to Python sources), so build first
  (`python3 -m pip install -e .`, or `python3 setup.py build_ext --inplace`) and then generate; if
  the tests import the project from somewhere else the loop stops with a `DLA-PYTEST` line that says so.
  The loop runs up to its time limit of 1800 s. Run `dla harness run <H>` for it as a plain foreground command with the tool's longest timeout. When the tool moves the command to the background because it runs long, wait for the notice that it ended and do other read-only work meanwhile. Never start it with the shell's `&` or `nohup` and never put it under `timeout`: a shell background job is killed when the command returns, and a cut run writes no series.
  Read its series with `dla trend`: the columns are the `py.blocks@<file:line>` sites that grew (test files
  and the harness never count), and a site no finding is at gets the `dla ledger add ... --site ...` command; the trend and the final gate name the lines largest growth first, so the first commands are for the lines that grow most.
  A finding made that way is decided with a small driver of its own, not with an alloc-test on the loop (which
  re-runs the whole suite per run): `dla harness gen --lang python --driver --probe F` with a unit that calls the
  function of that line, `dla harness run`, `dla trend`, `dla alloc-test` on that harness. The loop declares no
  candidate, so a candidate of the queue still needs its driver (hooks to fill). The alloc-test runs on a harness
  that targets the finding or measured it, never on another one (`harness-not-about`); a probe driver whose unit does not call the function of the line comes back inconclusive.
  `--tests` narrows the loop: it is for a loop that cannot run in full (it stopped with a `DLA-PYTEST` line) or for a second look at a part. The first step of the queue is the loop over all test files, and a narrowed loop does not replace it: the queue keeps naming the full loop, and the summary of a narrowed loop carries `pytest_hint` while the full loop has not run.
- Purges: `trim` (glibc `malloc_trim` through ctypes) and `gc`.
- Raw captures: `dla harness run H1 --capture histogram,alloc,nonheap,heap`. `histogram` writes
  `py-<n>.types.json` (300 types by instance count), `alloc` writes `py-<n>.alloc.json` (tracemalloc live
  bytes per stack, innermost frame first; frames: 10 with `alloc`, else 1) and `py-1.memray.json` when
  memray imports in the program's own interpreter (`DLA_PY_MEMRAY=0` leaves it out), `nonheap` writes
  `py-<n>.nonheap.json` (threads by name pattern and creation site, pending asyncio tasks, ResourceWarning
  sites), `heap` writes `py-1.retain.json` once, at the last dump point (the holder chains of the objects
  made at the lines that grew: summarize it with `heap-graph`; it costs time and memory in proportion to
  the objects the program tracks, `DLA_HEAP_SECS` bounds it, and `harness/python/README.md` has the
  figures). Run the `dla summarize` commands the harness prints. A run with `alloc` or `heap` is for
  attribution, not for the trend: tracemalloc with several frames, memray and the chain search slow the
  points that take them.

## Census
- Native codec contexts are listed as kind `handle`, holder `native-handle/global-ref`, off-heap, like
  `mmap.mmap` and `sqlite3.connect`: constructors named `compressobj`, `decompressobj`, `BZ2Compressor`,
  `BZ2Decompressor`, `LZMACompressor`, `LZMADecompressor`, `ZstdCompressor`, `ZstdDecompressor` and `ZstdDict`
  (the name must end the callee; `zlib.crc32`, `zstd.compress` and `zstd.decompress` are not listed). The
  entry is where a native object is made, so it names the holder of a value a C extension fails to release
  on an error path (a rejected option, a bad dictionary); it does not say the leak exists.

## Attach backends
| Backend | Tier | Notes |
|---|---|---|
| `dla attach memray --pid <P> --op record [--duration 30s]` | 3 | allocation stacks, not contents; memray must be importable by the target's own interpreter (checked first, refused otherwise); gdb injection below 3.14; 4 s capture took 8.6 s wall on 3.13; the 3.14 `sys.remote_exec` path was not run |
| `dla sample --pid <P> --counters python-remote` | 2 | runs a small generated script in the target; needs Python 3.14 or later |

## Artifacts
- memray capture files and stats land under `attach/P<n>/`; the raw `.bin` is not read by any adapter.
  Harness captures are read by `dla summarize`: `py-tracemalloc` and `py-memray` (`alloc-stack-diff`) tie
  by the stack's first project frame; `py-retain` (`heap-graph`) ties a holder to the finding whose owner
  is the holder (`static:<module>.<name>`) or that is at a line the holder keeps, and the final gate then ties
  every `py.blocks@` column of the lines it keeps to that finding; `py-types` (`type-histogram-diff`) ties only when a growing type is
  the finding's accumulation class and never for built-in containers (a `py.type:*` count that ties to no finding has no residual route: it needs its own disposition); `py-resources` (`non-heap`) ties a
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
- A reference-count error in a C extension that leaks a Python object shows as a growing `py.blocks@`
  site, at the line that calls into the extension (driver runs). Memory the extension takes with `malloc`
  does not: only native growth shows it (`os.rss_anon`).
- Task and thread leaks in async code are visible only while the tasks can be enumerated.
