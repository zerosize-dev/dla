# Brief T: a capture of the test loop must end by itself inside the agent's time limit, and a cut command must not cost the run

Repository: /home/user/dla (Rust workspace, branch `main`). The working tree holds uncommitted
changes of earlier builders. Leave them as they are; where you must edit one of those files, add
your own lines only. You are the only builder in the tree.
Scratch directory: /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u121
(create it; write nothing else outside the repository).

The machine has 4 cores. Use `cargo ... -j 3`, one cargo command at a time. Never touch
/root/dla-real.

## What happened (a real unattended run on a Python library; the agent that drives the tool may
## run one foreground command for 10 minutes at most)

1. The loop over the project's tests (`dla harness gen --lang python --pytest`, then `dla
   harness run H1 --mode both`) ran and `dla trend S1` showed 11 growing lines. The gate then
   asked, as its first item, for `dla harness run H1 --mode fresh --capture alloc,heap`.
2. The model ran that command in the foreground under `timeout 550`. It was cut. The plain loop
   of this project takes about six and a half minutes on a free machine; the loop fits its
   rounds into `DLA_TIME_LIMIT_SECS`, which `dla harness run` sets from
   `TEST_LOOP_TIMEOUT_SECS = 600` (`crates/dla-workload/src/harness/gen.rs`), and
   `harness/python/dla_pytest_loop.py` plans for 85% of that. Nothing in that plan knows that
   tracing slows every test, that the capture itself took 135 s here, or that the command has
   start-up and reading time around the process. 600 s is also exactly the agent's limit, so
   there is no room at all.
3. To clean up, the model ran `dla run abort --reason "capture run timed out in foreground"`.
   `dla run abort` stops the processes AND marks the run aborted for good. Nothing could be
   recorded after it; the run was lost after 15 minutes of a 3 hour budget.
4. A second try in the background with `--iterations 3` ended: its output said `SERIES TOO
   SHORT` (true and beside the point: the series of this harness exists already, the run was
   for the capture), the allocation diff had 1 profile where its gate needs 3, and the
   retention walk stopped at its 60 s limit after 45,440 of 218,674 objects, so it named no
   holder (`harness/python/sitecustomize.py`, `_HEAP_SECS`, `_heap_scan`, `_heap_groups`).

## The change

### 1. A cut command does not need `dla run abort`

- New command `dla run stop`: stops the processes the run started and undoes what it
  attached, exactly as `dla run abort` does, and leaves the run open. Its output says the run
  goes on. Read `crates/dla-cli/src/run.rs` (abort) and share the code.
- `dla harness run <H>`: at its start it stops the registered processes of an earlier run of
  the same harness that are still alive (a command that was cut left them) and says so in
  `_attention`. Find how processes are registered in `state.json`; stop by process group or
  PID, never by name.
- `dla run abort`: its output and its `--help` say in the first sentence that the run ends
  and nothing more can be recorded in it, and name `dla run stop` for stopping what a cut
  command left. After an abort, the refusal every later command gives names `dla run init`
  as the only way on. Do not change what abort does.
- Texts (see part 4): a command that was cut or timed out is followed by `dla run stop` (or
  simply by running it again), never by `dla run abort`.

### 2. A test-loop run ends by itself well inside ten minutes, capture included

- One number decides: the whole command `dla harness run` of a test-loop harness (Python
  pytest loop, and the JUnit repeat harness where its timeout is set the same way: check
  `JUNIT_TIMEOUT_SECS` and say in the report what you found; change the JVM side only if it is
  the same small change) must end within `DLA_COMMAND_SECS`, default 420 s. From it the tool
  gives the loop its process limit: minus a fixed allowance for start-up and reading (say what
  you measured), and, with `--capture`, minus the time the capture may take (`DLA_HEAP_SECS`
  plus an allowance for the allocation profiles; measure it on the small project below) and
  with the loop's own estimate scaled for tracing: the loop already measures the time of one
  run of the tests in this process, tracing on, so its `fit()` works as it does today once
  the limit it gets is honest. Check that the capture still gets what its summaries need: at
  least 3 allocation profiles and one retention record; when the rounds that fit are fewer
  than that, the loop drops the slowest test files (it does that today) before it gives up
  a profile.
- The output of `dla harness run` for a test-loop harness says in `_attention` what was
  shortened and why, in one short entry (entries are cut at 160 characters when the output is
  over its bound), and says once: this command ends by itself within N minutes, do not wrap
  it in `timeout`.
- A run made with `--capture` on a harness that already has a full series does not print
  `SERIES TOO SHORT`: say instead that this run was for the capture and name the two
  summarize commands.
- Hook drivers (units of milliseconds) are not changed.

### 3. The retention walk names something on a large heap

Read `_heap_scan` and `_heap_groups` in `harness/python/sitecustomize.py` first and measure
where the time goes on a heap of about 200,000 objects (build one in a test: many small
objects kept by a module-level list, and a few thousand made at one "growing" line and kept by
a module-level dict of the project).

- The objects the capture is about come first: the ones made at the growing lines of the run
  (the targets the scan already gets), before the rest of the heap. If the limit comes, the
  record says how many of those were walked and how many were not, and the summary's
  `_attention` says so with the remedy (a smaller driver for one of the named lines).
- If the scan itself (before any walk) is what runs out, make it cheaper or give it its own
  share of the limit so that the walk of the first targets always happens. Say what you
  measured before and after.
- `DLA_HEAP_SECS` stays the bound; do not raise the default above 90.

### 4. Texts

`skills/detect-leak-anywhere/references/python.md`, `references/stages.md` (the part on running
harnesses and on cut runs: find it) and `harness/python/README.md`: the changes in a few
sentences, pinned in `crates/dla-cli/tests/skill_text.rs`. `docs/VERIFY.md`: rows at the end,
state `corrected`, "a Python library the tool had not been run on". Do NOT touch
`skills/detect-leak-anywhere/SKILL.md`.

## Tests (each seen failing under a mutation you apply and revert)

- `dla run stop`: a run with a registered live process: stop kills it, the run status stays
  active, a later `dla ledger` write works; after `dla run abort` the later write is refused
  and the refusal names `dla run init`.
- `dla harness run` stops a leftover process of the same harness.
- The loop's limit: a small project whose tests sleep (a test file of 3 tests that take 2 s
  together): with `DLA_COMMAND_SECS=40` and `--capture alloc,heap` the command ends by itself
  under 40 s with at least 3 profiles and a retention record, and `_attention` says what was
  shortened; without `--capture` it uses more rounds. Use the helper that guards the existing
  Python driver tests.
- The walk: the 200,000-object heap with a short `DLA_HEAP_SECS`: the holder of the objects
  from the growing line is named; the record counts walked and not walked.
- Python agent tests: `python3 -m unittest discover -s harness/python -p 'test_agent_*.py'`
  and the Python files that `crates/dla-cli/tests` run; two known failures on interpreters
  without `gc.freeze` support are not yours (say which fail before your change).
- Existing tests that change: say which and why. Do not weaken a test. Nothing under
  `bench/fixtures` is edited. Run `dla-bench dynamic --only <name>` for two Python fixtures (a
  leak and a non-leak) and report the results.

## Rules

- No `git commit`, `git stash`, `git checkout` (in any form), `git reset`, `git restore` or
  branch change. Leave the tree dirty.
- Shared `target/` only (`cargo` from /home/user/dla). No `rm -rf` on variable paths. Do not
  kill processes by name. Do not run `dla consent`. Do not start any model, agent or verifier.
  No package installs.
- Run each cargo command on its own, in the foreground, piped through `tail` or `grep`. If an
  action is denied, stop that action and report it; do not ask anyone to do it for you and do
  not try another way to the same end.
- Keep to this brief. If you find another defect, report it; do not fix it.
- Plain technical English, full sentences, no em dashes. Never name a real third-party project,
  library or its APIs in the repository. No model names, no attribution lines.
- Mutations by a small script that asserts the old text occurs exactly once. Before you finish,
  `grep -rIl "MUT_" crates harness skills docs bench agents hooks data` must print nothing.
- If the schema of a record changes, update `schemas/` and run `./target/debug/dla schema
  --check`.

## Checks before you report

`cargo fmt --all --check`; `cargo clippy -j 3 --workspace --all-targets -- -D warnings`;
`cargo build -j 3 -p dla-cli -p dla-bench` and `./target/debug/dla schema --check`;
`cargo test -j 3 -p dla-workload --lib`, `cargo test -j 3 -p dla-core --lib`,
`cargo test -j 3 -p dla-cli --bin dla`, the Python driver and test-loop test files of dla-cli
(list them), `--test gate_flow --test skill_text --test holder_first`, and the tests you added.

## Report (final message; read by the orchestrator)

Two sentences of result first. Then per part: files changed, what works, one real output
(first lines); the numbers you measured (start-up allowance, capture time, scan and walk times
before and after); each test with its mutation; the two fixture results; existing tests changed
and why; anything in this brief that was wrong; what you did not verify. Counts only.
