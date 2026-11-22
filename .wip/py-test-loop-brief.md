# Brief Y: the project's own tests in a loop, first step of a Python queue

Repository: /home/user/dla (Rust workspace: crates dla-core, dla-workload, dla-cli and others;
plugin files under agents/, skills/, harness/, data/). You are a builder. The working tree holds
uncommitted work from two earlier briefs (W1: the workload blocker per runtime; X: allocation-site
count columns `py.blocks@<file:line>` for Python driver runs); build on it, do not revert it.

Scratchpad (sp): /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad
Your scratch directory: <sp>/u77 (create it). You cannot write a report file with the Write tool:
give the report as your final message (you may also write it with a shell redirect to
<sp>/u77/report.md).

## Hard rules

- No `git commit`, `git stash`, `git checkout`, `git reset`, `git restore` in /home/user/dla.
- Never kill a process by name. No `rm -rf "$VAR/..."`: only literal paths, only under <sp>/u77.
- Use the shared `target/` directory only (`cargo ... -j 2`). Disk is tight (about 5 GB free):
  run `bash <sp>/wip-notes/prune-superseded.sh` before a whole-suite run.
- Nothing in the repository may name a model, an AI assistant, or any third-party skill collection.
  No attribution lines. Plain technical English, short sentences. Do not name the project of the
  evidence run in the repository (say "a Python library with C extension modules").
- A marker mutation is an exact text replacement that carries a unique marker such as
  `/*MUT_Y1*/` (`#MUT_Y1` in Python); apply it with a script that asserts the old text occurs once
  (see <sp>/u66/mut.py for the pattern), see the test fail, reverse it with the exact reverse
  replacement. At the end `grep -rIl "MUT_" crates harness skills docs bench agents hooks data`
  must print nothing.
- Do not run `dla consent`, and do not start any model, verifier or agent. A scratch run is
  initialised with `DLA_CONSENT_TIERS=1-4 DLA_CONSENT_RAW_VALUES=1` in the environment of
  `dla run init`.
- If something you are told here is wrong, say so in the report; do not widen the task.
- If a permission check denies a command, do not work around it and do not ask the orchestrator to
  run it for you: report it and go on with the rest.

## Why

A leak that nothing holds (for example a reference a C extension never releases) has no static
candidate. It shows only as an allocation line that grows under a broad workload. The broadest
workload a project has is its own tests. Measured by hand on a Python library with C extension
modules (<sp>/u72: `loop.py`, and the note <sp>/wip-notes/real-run-plugin-pillow-1.md): two of its
test files run with `pytest.main` in a loop in one process show the documented leak as the top
growing project line, about 145 blocks per round, in 17 seconds; the whole suite (157 files, 3736
tests) takes 32 seconds per run outside the harness. In the real unattended run the agent wrote
its own driver units, never called the leaking function, and found nothing.

The JVM side already has "the project's own tests are the first pass of the queue" (a hookless
JUnit loop; code map with file:line and what is JVM-only: <sp>/u74/2.md, read it). This brief adds
the Python counterpart in its simplest useful form: one loop over the project's test files, with
no declared targets, offered first. The by-name choice of test classes per candidate (the JVM
plan) is NOT part of this brief; Python candidate batches stay hook drivers.

## The change

1. `dla harness gen --lang python --pytest [--tests <paths>]`
   (crates/dla-cli/src/harness_cmd.rs, crates/dla-workload/src/harness/gen.rs).
   - Python only (another `--lang` is refused in the style of the `--junit` refusal). It needs no
     candidate and declares no target; `--probe` may still add targets as for any harness.
   - The unit is one pytest run, in the same process, of the selected test files. Without
     `--tests`: every test file of the Python runtime's test workloads (workload discovery already
     lists them: one workload per test file with its cwd) under the working directory that has the
     most of them; test files under another working directory are named in a gap line and left
     out. With `--tests a,b`: those files or directories, relative to that working directory
     (pytest node ids are not supported; refuse `::`).
   - No workload report yet: run the discovery the way `dla census --lang jvm` does when it is
     missing. No Python test file: refuse with a message that says so and names the driver
     command.
   - The harness interpreter is the one Python harnesses use today (`python3` from PATH; see
     crates/dla-workload/src/harness/commands.rs). If pytest does not import in it
     (`python3 -c "import pytest"`), refuse: say which interpreter, that pytest does not import in
     it, the command that installs it (`python3 -m pip install pytest`), and the driver command as
     the other way on. The refusal is recorded as a failed attempt of the test loop (see 3), so
     the queue moves on.
   - Hookless: nothing to fill. Output and `instructions` in the style of the JUnit loop ("nothing
     to fill: the driver runs the project's own tests in a loop; run it"), `next` =
     `dla harness run <H>`.
   - Manifest: mode Driver, key modes [Fresh] only, tool marker `pytest-loop`;
     `HarnessManifest::is_test_loop()` is true for it and `is_hook_driver()` false. The survey
     lists every place that tests the literal `junit-platform-launcher` (harness.rs, gate.rs,
     alloc_cmd.rs, run.rs of dla-workload, trigger.rs): make each read the manifest predicate, or
     handle both markers, and say what you did per place. Step 1, a timeout of 1800 s.
2. The loop driver: a new template harness/python/dla_pytest_loop.py (wire it like the existing
   templates in crates/dla-workload/src/harness/templates.rs; the Python agent sitecustomize.py
   stays the agent and writes the records).
   - Paths: the working directory first, and `<cwd>/src` when that directory holds a package or a
     module (a src layout).
   - One unit: `pytest.main([...])` with a small plugin object that counts, per test file, the
     tests that passed, failed and were skipped, and the time taken. Arguments that keep the run
     in this process and quiet: `-o addopts=` (the project's own addopts may ask for parallel
     workers or a coverage plugin), `-p no:cacheprovider`, `-p no:xdist`, `-q`, a short traceback
     style. `SystemExit` is caught. Output goes to the harness log, bounded.
   - Rounds as the generic driver (DLA_WARMUP, DLA_ROUNDS, DLA_STEP, a checkpoint after each
     round through the agent's `checkpoint`), with a step floor of 1.
   - After the first unit:
     a. Where the code comes from. For every top-level package or module the project defines
        (under the working directory or its `src/`) that is now in `sys.modules`: if its file is
        neither under the scratch copy nor under the project root, stop with exit code 3 and a
        line on stderr: `DLA-PYTEST the tests import <name> from <path>, which is outside the
        project: the loop would measure another copy. Build or install the project in place (for
        example python3 -m pip install -e . , or python3 setup.py build_ext --inplace for
        extension modules) so that <name> imports from the project, then generate the harness
        again`. Find out when the scratch copy is made (at generation or at each run) and word the
        last clause to match.
     b. Files that cannot run: a file none of whose tests passed is dropped when another file had
        a passing test (limitation `dropped test file <path>: none of its tests passed in the
        first run (first failure: ...)`). No test found: exit 3. No test passed in any file: exit
        2. A later unit in which tests fail is counted and its first failure recorded as a
        limitation once, as the JUnit loop does; exit 2 only when no test passed in any unit.
     c. Time: from the first unit, estimate the whole series (warm-up plus rounds). If it does
        not fit the time limit (DLA_TIME_LIMIT_SECS, with the margin the JUnit loop keeps), drop
        the slowest files until it fits (limitation `dropped test file <path>: it took <t> s of
        <unit> s per run and the series would not fit <limit> s`); if one file alone does not
        fit, write `the tests are too slow for a series` and exit so that the run reports no
        usable point. Timing restarts after a drop, as in the JUnit loop.
   - A summary line on stderr at the end (`DLA-PYTEST summary: units= tests_run= files= dropped=
     failed_units= first_failure=`).
   - The Rust side (crates/dla-workload/src/harness/run.rs): the attention lines that exist for
     the JUnit loop (`DRIVER STEP`, `TEST CLASS DROPPED`) also show for `dropped test file`; a
     failed loop prints the reason line (`DLA-PYTEST ...`) in its error, and its fix names the
     in-place build or the driver command, not hooks (this harness has none). A run with no usable
     point falls back to the driver command for the queue, as for the JUnit loop. The time-budget
     estimate of `dla harness run` for this harness is bounded by its timeout (see how the JUnit
     loop is estimated), not by a step the loop lowers itself.
3. Queue (crates/dla-core/src/queue.rs and what prints it: `dla run status`, the gate's queue
   line, `dla ledger close-rest`): for an open Python queue (queue on, budget above the floor, not
   blocked) of a run that has Python test files, the first command is the test loop:
   `dla harness gen --lang python --pytest`, with one sentence in the queue's steps text ("first
   the project's own tests in a loop, nothing to write: ... ; a line of the project that grows
   there gets a finding (the trend prints the command)"). It stays the first command until a
   `pytest-loop` harness of this run wrote a series, or an attempt of it failed (the refusal for a
   missing pytest, a failed run, no usable point); a generated one that has not run yet makes the
   command `dla harness run <H>`. After that the queue goes on with the driver batches exactly as
   today. This is an order, not a new gate rule: do not add a blocking item for it, and do not
   refuse `--driver --next` before it.
   Candidates are not settled by this loop (it declares none). If the trend of its series reads a
   candidate's probe as grown, the existing routes apply.
4. Trend of the loop's series: nothing new beyond brief X (the `py.blocks@<site>` columns and
   their add command). Check the texts a test loop gets elsewhere and make them true for this
   harness: the trigger line of a finding that grew in it ("the project's own tests in <H>"; do
   not print the `dla alloc-test ... ` attribution hint, which measures test classes for the JVM
   only), the bundle's `test-loop/` file (list the test files that ran; no per-file figures), and
   `agents/verifier.md` only if its test-loop paragraph says something false for Python (do not
   touch the archetypes block).
5. Texts: skills/detect-leak-anywhere/references/python.md (the test loop: what it runs, that it
   needs pytest in the harness interpreter and the project importable in place, what the columns
   mean, that candidates still need their drivers), one sentence each in references/stages.md and
   SKILL.md where the JVM test loop is described as the first pass, harness/python/README.md.
   Update the pins in crates/dla-cli/tests/skill_text.rs. docs/VERIFY.md: one row (state
   `corrected` or `added`, in the style of the rows about the JVM test loop): what is verified and
   what is not.

## Tests

- Real loop (crates/dla-workload/tests, next to the tests that run a real Python driver; skip
  with the repository's usual "tool missing" pattern when pytest does not import in `python3`,
  and say in the report whether it ran here): a small project with a package, a function that
  keeps one object per call at a known line, and three test files: one that calls it, one that
  passes without calling it, one whose every test fails. Assert: the harness runs with no edit;
  the series has the `py.blocks@` column of the keeping line, rising per run; the failing file is
  dropped with its limitation and the run still succeeds; a project where every test fails exits
  as a failed harness with the reason; the outside-copy stop (a package of the same name placed
  on the path outside the project) fails with the `DLA-PYTEST` line.
- A slow file is dropped to fit the time limit (a test that sleeps; a small limit through the
  manifest's timeout).
- Generation: `--pytest` refused for another language; refused with the install text when pytest
  does not import (simulate with a `python3` shim first on PATH that fails the import); `--tests`
  with `::` refused; a project without test files refused.
- Queue (crates/dla-cli/tests/queue.rs is the model): the first command of an open Python queue
  with test files is the test loop; after its series exists the command is the driver batch;
  after a failed attempt likewise; a project without test files never shows it; a blocked or
  closed queue never shows it.
- Each new rule seen failing with a marker mutation, then reversed (list rule, mutation, failing
  test).

## Checks you run

- `cargo fmt --all`, `cargo clippy -j 2 --workspace --all-targets -- -D warnings`,
  `./target/debug/dla schema --check`.
- The benchmark's dynamic acceptance for the Python fixtures must still pass with your build:
  `DLA_PLUGIN_ROOT=/home/user/dla ./target/debug/dla-bench dynamic --only python --dla-bin ./target/debug/dla --json <sp>/u77/dynamic-python.json`
  (build `dla-bench` with `cargo build -j 2 -p dla-bench` first if it is not built; if the
  command line differs, read `dla-bench dynamic --help`). Report the result per fixture.
- The whole suite once at the end (prune first):
  `cargo test -j 2 --no-fail-fast --workspace > <sp>/u77/test-all.log 2>&1`; count the
  `^test result:` lines and the failures from the log (about 17 minutes).
- Real tree, no model. The library is at <sp>/u72/pillow (scratch; a git checkout with `.venv`,
  Python 3.12, the library installed editable from this tree, pytest installed). Environment:
  `export DLA_PLUGIN_ROOT=/home/user/dla DLA_BIN=/home/user/dla/target/debug/dla PATH=<sp>/u72/pillow/.venv/bin:$PATH`.
  Start a NEW run (`DLA_CONSENT_TIERS=1-4 DLA_CONSENT_RAW_VALUES=1 dla run init --entry repository
  --budget auto`, answer the intake items with `dla ledger resolve intake.<item> --status answered
  --evidence E1` for target scope symptom runtimes build workload consent budget, `dla probe`,
  `dla sweep --lang python`, `dla census --lang python`). Then:
  1. `dla run status`: show the queue's command (expected: the test loop).
  2. `dla harness gen --lang python --pytest`, `dla harness run <H>`, `dla trend <S>`. Show the
     outputs (heads), the wall time, the time per unit, which files were dropped and why, the
     `py.blocks@` columns that passed. Expected: `py.blocks@src/PIL/ImageFont.py:580` passes.
     If the whole suite does not fit, say what was dropped and whether the files that draw text
     (Tests/test_imagedraw.py, Tests/test_imagefont.py) stayed in.
  3. `dla run status` again: the queue's command is now the driver batch.
  4. The same generation with the system interpreter first on PATH (drop the `.venv/bin` prefix:
     `python3` is then 3.13 without pytest): show the refusal and the queue's command after it.
  If a figure disagrees with what is expected here, report what you see; do not tune anything to
  match.

## Report

Lead with the result. What changed (files), the tests and the mutation each was seen failing
with, the real-tree outputs and times, the benchmark result, the whole-suite counts, what is not
verified.
