# Brief R4: the failing test explained, and four more points from the second real-library check

Same repository, same rules as R2 and R3, scratch `u93`, Python 3.13 only (no 3.12, nothing under
scratch `u72/pillow` or `u101`). Your R3 tree stays. Do NOT run the watcher that was denied and do
not look for another way to keep that test's directory: the cause is known (item 1).

My checks of R3: `u91/check_cases.py` is ALL OK under 3.13 and 3.12 with your final agent; under
3.12 `harness_capture_python` has only the two known `gc.freeze` failures, and `py_site_columns`
26, `summarize_python` 22, `alloc_test` 5 of 5, `pytest_loop` 17 pass. The real-library walk with
your binary: capture 19 s for the chains (was 60.5 s and cut), five lines under the module
global, the native entry for the sixth, no second capture asked, the stack in the bundle, no
finding item left in the final gate. Four things are still wrong (items 2 to 5).

## 1. Why `frozen_garbage_is_tied_to_the_freeze_site...` fails: the first process compiles

In command mode every point is a process. The first process of a harness finds no bytecode cache
for the agent (and for the project's modules in the fresh copy), compiles them and writes the
cache; every later process loads the cache. A module compiled from source leaves more objects
alive than one loaded from the cache. Measured with an idle interpreter and the agent on the path:
`gc.freeze` then `gc.get_freeze_count()` is 8569 in the first process and 7596 in the second and
third (8546 and 7571 with the agent of the last release). In the failing test the frozen counts of
the nine points are 14312, 14247, 15747, 17247, ... (+1500 per point from the second on): the
first point reads about 1565 high, the first step is negative, and the count column fails the
gate. With the smaller agent the offset was just under the fixture's 1500 per step, so the test
passed by luck. `PYTHONDONTWRITEBYTECODE=1 cargo test -p dla-cli --test alloc_test frozen_garbage`
passes with your agent. It is not `_heap_path`; any growth of the agent file past that edge does
it. The same offset is in every heap column of the first point of a command-mode Python series,
and a library that runs from its source tree gets `__pycache__` written into the user's project.

After: every process a Python harness starts (command mode, drivers, the test loop) runs with
`PYTHONDONTWRITEBYTECODE=1`, so no process writes a cache and every point loads its modules the
same way. Find where the environment of a Python harness process is built
(`crates/dla-workload/src/harness/run.rs`; one place for all modes if there is one). A value the
user's environment already has for that variable is kept.

Tests: the failing test passes untouched. A new test on the same kind of fixture: the first step
of the `py.gc_frozen@` column equals the later steps (the first point lies on the line), and after
the run there is no `__pycache__` directory under the harness's `agent/` and `work/`. Mutation:
the variable not set fails both. Texts: one sentence in `harness/python/README.md` and one in
`references/python.md` where command mode is described; a VERIFY row (left column: "The first
point of a Python series read high"; state `corrected`).

## 2. `next` proposes a site outside the project

The chain of the module global on the real library is
`<object> <- frame@<outside>/dla_harness.py:99 <- traceback@<outside>/dla_harness.py:97 x15653 <- ImportError(.ex) <- <class>(._global)`.
The run of 15 653 tracebacks covers four lines in turn (the driver's and three of the project);
the step shows only the first one's `at`, which is the driver's. `next` printed
`dla ledger add ... --site <outside>/dla_harness.py:97 --owner <module>.<global> ...`; the finding
then has no column of its own and its route was "resolve as unresolved, inconclusive", while its
holder keeps five growing lines.

After:
- Agent: a step with `repeat` also lists the distinct `at` values of the run (`ats`, at most 6,
  in the order met); `at` stays as it is. The adapter keeps `ats` in the record's path and in the
  raw file (clipped like `at`; a frame key the bundle leaves unscrubbed).
- Adapter, the site of the finding `next` proposes: never an `<outside>/...` or `<...>` place.
  From the root side, the first step that names a project line in `at` or `ats`; of those lines
  the one the holder holds with the largest growth, else the first; with no project line in the
  chain, the held site that grew most.
- `dla ledger add` with an `--owner` that a retention record names as a holder and a `--site`
  that is none of the lines that holder keeps: a warning that names up to three of the held lines
  and says that a finding at one of them has a measured column. Not a refusal.

Tests: unit tests of the site choice (a chain whose only `at` is outside; a run with `ats`; no
project line at all), one agent test of `ats`, one CLI test of the warning. Mutations for each.

## 3. The bundle of one finding shows the stacks of another

The stack summary of that capture has four growing stacks; three belong to the other leak. The
bundle of the finding at the native line now holds the record (good) but `excerpt.txt` again shows
four lines of the other leak: the frames are taken from every stack of the record in its order.

After: the copy of an alloc-stack record in a bundle (the `result` in `index.json` and the raw
file) is cut, before the scrub and before frames are collected, to the stacks whose first project
frame is the finding's line, when the finding has a line and at least one stack matches; the
number left out is stated (`other_stacks`). When no stack matches, the record stays whole as
today. Excerpts then start with the frames of the finding's own stack. Also: in the focused copy
of a retention record the top-level totals (`kept`, `sites`) are those of the entries left, not
of the whole record. Test on a record with stacks of two lines: each finding's bundle has its own
stack and excerpt and no frame of the other's; mutation.

## 4. Checks

All checks of brief R2 again with the final tree (fmt, clippy, schema check, the unit and test
files listed there, `alloc_test` now 5 of 5), `python3 u91/check_cases.py python3 <tag>`, the
`MUT_` grep empty. Run also the dla-cli test files you did not rerun after R3: `summarize_node`,
`summarize_go`, `summarize_jvm`, `second_look`, `focus`, `left_out`, `reasons`, `harness_capture`.

## Report

Two sentences first. Then per item 1 to 3: what changed, the tests, the mutations, with real
numbers (for item 1 the nine frozen counts before and after). Then existing tests changed and
why; anything wrong in this brief; what you did not verify.
