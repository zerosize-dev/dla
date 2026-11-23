# Brief G: the Python agent must not break the program it measures, a cut test run is not a unit, and a finding at a growing line cannot be closed as not exercised

Repository: /home/user/dla (Rust workspace, branch `main`, clean working tree at the start).
Scratch directory: /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u95.
`target/` is empty: your first cargo command rebuilds everything (15 to 20 minutes).

## What happened (a real unattended run on a Python library, reproduced)

The first step of the Python queue ran the project's own tests in a loop, nine rounds. The
documented leak line grew by 299 live blocks per round. The run still ended COMPLETE with "no
leak confirmed". Three defects, in the order they acted.

1. The agent broke the program. Seven of the nine rounds had test failures of one kind:
   `SystemError: Objects/tupleobject.c:909: bad argument to internal function`. In round 2 the
   error hit while pytest collected a test file, pytest stopped ("Interrupted: 1 error during
   collection", 27 s instead of about 120 s) and the round ran no test at all.
   Cause, reproduced in isolation: the agent's tick thread (`_ticker` in
   `harness/python/sitecustomize.py`, started when `DLA_TICK_MS` is set; `dla harness run` sets
   100) calls `dump("tick", ...)`, and the tick record computes
   `len(gc.get_objects())` (in `_build`: `values["py.gc_objects"] = len(objs) if objs is not None
   else len(gc.get_objects())`). On a large heap that call takes longer than the interpreter's
   switch interval, the tick thread gives up the interpreter lock while it still holds the list,
   and the list refers to a tuple the main thread is building; the main thread's resize of that
   tuple then fails because the tuple has a second reference. Script
   /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u94/race.py
   (`race.py tick 0.1` against `race.py none 0.1`): with 600000 tracked objects and a thread that
   calls `len(gc.get_objects())` every 100 ms, `tuple(generator)` in the main thread raised that
   SystemError 25 times in 20 s on Python 3.12 and 68 times on 3.13; 0 times without the thread.
2. The loop counted the cut run as a unit. The series of every line then had one flat step
   (299, 299, 598, 897, ...), and the counter gate, which needs a rise of at least one at every
   step, failed every `py.blocks@` column with `delta < 1`, although their shape was `steady`.
3. A finding at such a line could be closed unmeasured. The final gate listed the columns as
   "not flat and tied to no finding" (its predicate is `non_flat`: the gate passed, or the shape
   is steady or stair; `crates/dla-core/src/gate.rs`), so findings were added at the lines. But
   the rule that stops a grown finding from being closed unmeasured (`grown_unmeasured` in
   `crates/dla-core/src/grown.rs`) looks only at columns whose gate PASSED. So the findings were
   not "grown": `dla ledger resolve F --as unresolved --reason inconclusive` was refused with
   `measurement-required`, but the gate's own fix for each finding was `dla ledger close-rest`
   (the fallback in `crates/dla-core/src/view.rs` near line 542), and `dla ledger close-rest`
   closed all of them as `unresolved(not_exercised)` with the remedy "not ranked for
   measurement". The line had been exercised and measured eight times.

A copy of that run's state (ledger, trends, series, harness manifests, no sources) is at
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u94/project/.dla/20261123-095603-f16d
(the ledger there is cut right after finding F2 at the leak line was added and F1 was closed;
the full ledger is `u94/ledger.full.jsonl`; `dla gate check` reports the cut as tamper, which you
can clear in a COPY of that directory with `dla run reseal --reason "..."`). Use copies of it for
the replay in "Checks"; never write into it.

## The change

G1. The tick thread never walks the heap. A tick record carries only what can be read without
    creating references to the program's objects: no `gc.get_objects()`, `gc.get_referrers()`,
    `gc.get_referents()`, no tracemalloc snapshot, no walk of module or object dicts of the
    program. Read everything a tick executes (`dump`, `_build` with `tick` true, the probes, the
    type counts, the OS values) and remove or skip each such read for ticks; `py.gc_objects` is
    left out of a tick record (find who reads tick records on the Rust side and make sure a
    missing value is handled: search `"tick"` and `py.gc_objects` in `crates/`). Say in your
    report every read you removed. Checkpoint records (taken by the driver on the main thread)
    are unchanged.
G2. A run of the tests that pytest ended early is not a unit
    (`harness/python/dla_pytest_loop.py`). From the second run on: when the exit code of
    `pytest.main` is neither 0 (all passed) nor 1 (some tests failed), or the run executed no
    test, the run is not counted: one note says which run, the exit code and the first error
    line; the same run is made again. At most 2 repeats for one unit and 4 in the whole loop;
    after that the run counts as today's "dead unit" and the summary line and the limitations say
    how many units were cut. The time fitting (`fit`) must see the time the repeats took. The
    first run keeps today's rules. Add `--continue-on-collection-errors` to `PYTEST_ARGS`, so that
    one test file that fails to import costs its own tests and not the whole run; the files
    whose collection failed are named once in a note.
G3. A leak-side finding at the site of a non-flat allocation-site column is grown.
    In `grown_unmeasured`, a column `py.blocks@<site>` (any allocation-site column,
    `is_alloc_site_column`) points at the finding at its site (`finding_is_at_site`) when the
    column is non-flat by the gate's own predicate (`crate::gate::non_flat`), not only when its
    gate passed. Everything that follows from "grown" then applies to it: the refusal
    `growth-not-measured` for closing it unmeasured (`not_exercised`, `inconclusive`,
    `time_budget` each go through the rules that exist), `dla ledger close-rest` leaves it and
    says why, `supported_reason` is not `not_exercised`, and the fix the gate and
    `dla run status` print is the route of a grown finding (for a line that grew in a test loop:
    the probe driver, then the alloc-test). Check that the fallback `dla ledger close-rest` in
    view.rs can no longer be printed for a finding that has a non-flat column at its site, and
    that `close-rest` and `resolve` agree for every reason (one predicate; if they use different
    ones today, make them share it and say so).
    Do not change the counter gate (`delta < 1`) or the trend verdicts.
G4. Texts. `skills/detect-leak-anywhere/references/python.md`: one sentence where the test loop
    is described (a run that pytest ends early is made again and is not a unit). `docs/VERIFY.md`:
    three new rows at the end in the style of the last rows, state `corrected`, no project name:
    "The Python agent's tick thread made the program fail" (with the numbers of the script),
    "A test run that pytest cut counted as a round", "A finding at a growing line was closed as
    not exercised". `harness/python/README.md` where ticks or the loop are described. Do NOT
    touch `skills/detect-leak-anywhere/SKILL.md` (size limit).

## Tests (each seen failing under a mutation you apply and revert)

G1:
- A deterministic test, not a race: in `crates/dla-workload/tests/` next to the Python agent
  tests (read `harness_python_blocks.rs` helpers), a driver whose `setup` replaces
  `gc.get_objects` (and `gc.get_referrers`, `gc.get_referents`) with wrappers that record the
  name of the calling thread, runs long enough for several ticks (set the tick interval through
  the environment the helpers use, 20 ms), and writes the recorded thread names to a file the
  test reads: every call came from the main thread, and the dump holds tick records (so the
  thread did run). Mutation: the old read restored in the tick path.
- The same test asserts a tick record has no `py.gc_objects` and a checkpoint record has it.
G2:
- `crates/dla-workload/tests/harness_pytest_loop.rs` (it needs pytest: follow how the existing
  tests guard on it): a project whose `conftest.py` raises at collection on the SECOND run only
  (a counter in a file under the scratch project) : the loop repeats that run, the series has no
  flat step for a line that keeps one object per test run, the note names the cut run, and the
  summary counts no dead unit. A second case that is cut every time from run 2 on: the loop
  stops repeating after the bounds and reports the cut units.
- A test file that fails to import in every run: the other files run in every run, the note
  names the file once.
G3:
- CLI tests in `crates/dla-cli/tests/py_site_columns.rs` (read `synthetic`, `five_sites`): a
  trend whose `py.blocks@` column is `steady` with a FAILED gate (one flat step) and a finding
  added at its line: `dla ledger resolve --as unresolved --reason not_exercised` is refused
  with `growth-not-measured`; `dla ledger close-rest` leaves the finding and names it; the
  final gate's fix for the finding is not `dla ledger close-rest` and names the measuring
  route; after the alloc-test route the finding can be closed as today. A flat column (not
  non-flat) at a finding's site changes nothing: pin it.
- Unit tests next to `grown_unmeasured` for the predicate (gate pass; steady with failed gate;
  stair with failed gate; flat; plateau with failed gate is not non-flat: check what `non_flat`
  says and pin it).
- Existing tests that pinned "only a passed column makes a finding grown" change to the new
  rule: say which. Do not weaken a test. Do not edit anything under `bench/fixtures`.

Mutations by a small script that asserts the old text occurs exactly once. Before you finish,
`grep -rIl "MUT_" crates harness skills docs bench agents hooks data` must print nothing.

## Rules

- No commit, stash, checkout, reset, restore or branch change. Leave the tree dirty.
- Shared `target/` only (`cargo` from /home/user/dla, `-j 2`). No `rm -rf` on variable paths. Do not
  kill processes by name. Do not run `dla consent`. Do not start any model, agent or verifier.
- Run each cargo command on its own, in the foreground, piped through `tail` or `grep`. If an action
  is denied, stop and report it.
- Keep to this brief. If you find another defect, report it; do not fix it.
- Plain technical English, full sentences, no em dashes. Never name a real third-party project,
  library or its APIs in the repository (pytest and the standard library are fine). No model
  names, no attribution lines.
- If a golden file or a schema file changes, say which and why (`./target/debug/dla schema --check`
  must stay clean).
- Do not run the whole workspace suite. The machine's `python3` is 3.13 (no pytest); a 3.12 with
  pytest is at /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u72/pillow/.venv/bin
  (put it first on PATH for the pytest-dependent tests, and run the agent tests under both).

## Checks before you report

1. `cargo fmt --all --check` clean.
2. `cargo clippy -j 2 --workspace --all-targets -- -D warnings` clean.
3. `cargo build -j 2 -p dla-cli` then `./target/debug/dla schema --check` clean.
4. `cargo test -j 2 -p dla-core --lib`, `cargo test -j 2 -p dla-cli --bin dla`,
   `cargo test -j 2 --no-fail-fast -p dla-workload --test harness_python_blocks --test harness_pytest_loop --test harness_capture_python --test harness_python`
   (the last name may differ: run every test file of that crate that starts the Python agent)
   under both interpreters,
   `cargo test -j 2 --no-fail-fast -p dla-cli --test py_site_columns --test pytest_loop --test gate_flow --test gate_harness --test keep_going --test queue --test trigger --test skill_text --test alloc_test`.
5. Replay on a COPY of the real state (see above), with your built binary and the working
   directory set to the copy's `project` directory: `dla gate check --final` prints for F2 a fix
   that is not `dla ledger close-rest`; `dla ledger close-rest` leaves F2 open and says why;
   `dla ledger resolve F2 --as unresolved --reason not_exercised --remedy x` is refused with
   `growth-not-measured`. Quote the three outputs (the relevant fields) in your report.
6. The race script (`u94/race.py`) is not a test of the agent; instead run one real driver of
   your own for 30 s with ticks at 100 ms, a ballast of 600000 tracked objects and a unit that
   builds tuples from generators, under both interpreters, before and after your change, and
   report the number of SystemErrors (before: expect some; after: 0).

## Report (final message; read by the orchestrator)

Two sentences of result first. Then: files changed; every heap read removed from the tick path;
each test with its mutation; existing tests changed and why; the replay outputs; the before and
after counts of check 6; anything in this brief that was wrong; what you did not verify.
