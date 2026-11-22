# Brief T: a worker that skips `dla run status` still learns that the Python queue starts with the test loop

Repository: /home/user/dla (Rust workspace, branch `main`, a dirty working tree with unfinished
changes on top of commit 29cb6f6; all of it is to stay). Scratch directory for your files:
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u82 (create it).

## What happened

An open Python queue of a project that has test files starts with the project's own tests in a
loop: `dla run status` prints `dla harness gen --lang python --pytest` as the queue's command
(`RuntimeQueue.py_loop` in `crates/dla-core/src/queue.rs`). It is an order of the queue, not a
gate rule.

In a real unattended run the Python worker never took that step. After `dla sweep` and
`dla census` it ran `dla ledger list` and `dla gate check --stage sweep`, not `dla run status`,
and then ran `dla harness gen --lang python --driver --next`, because `agents/worker.md` names
that command literally as "the batch command". Nothing it read named the loop command:

- the census output said only "the queue of a Python project that has test files starts with the
  project's own tests; dla workload discover repeats it" (no command);
- `dla ledger list` printed `"queue":{"python":{"closed":null,"eligible":186,"open":true,...}}`
  (no command);
- the output of `dla harness gen --lang python --driver --next` had no word about the loop.

The JVM has a precedent for the last point: a hook driver generated on a project with JUnit 5
tests carries `junit_hint` in its summary (`junit_hint_text` in
`crates/dla-workload/src/harness/gen.rs`), and `crates/dla-cli/src/out.rs` never cuts it
(`KEPT_WHOLE`).

## The change (texts and hints only; no command is refused, no behaviour of the queue changes)

1. `agents/worker.md`. The sentence that starts "Work your queue before you close anything."
   must not name `harness gen --lang <runtime> --driver --next` as THE command. It must say: after
   `sweep` and `census` run `dla --run <id> --worker <name> run status`; while it shows an open
   `queue` for your runtime, run the command it prints for that queue (`next_command`, the same
   as `queue.<runtime>.command`), exactly as printed. Then name the two cases in one sentence
   each: for a Python project with test files the first command is
   `dla --run <id> --worker <name> harness gen --lang python --pytest` (the project's own tests in
   a loop, nothing to fill: run it, then `dla trend`, then go back to `run status`); the driver
   batches (`harness gen --lang <runtime> --driver --next`, hooks to fill) come when the queue
   prints them. Keep everything else the sentence says about filling `setup` and `unit`. Read the
   whole file first and change any other place that names the driver batch as the first or only
   queue command (line 16 says "a `queue` item first: its fix is the batch command": make it "its
   fix names the command").
2. Census note. In `crates/dla-cli/src/census_cmd.rs` the Python note must name the command:
   "... the queue of a Python project that has test files starts with the project's own tests:
   dla harness gen --lang python --pytest (nothing to write); dla workload discover repeats the
   discovery". The JVM note is unchanged.
3. `dla ledger list`. The per-runtime queue summary under `totals.queue.<runtime>` gets a
   `command` field: the same string as `dla run status` prints for that queue
   (`RuntimeQueue.command`), present while the queue is open. Find where that summary is built
   (`crates/dla-cli/src/ledger_cmd.rs` or near it) and where its tests are.
4. Hint in the hook driver's output. When `dla harness gen --lang python --driver` (with or
   without `--next`, not with `--pytest`) writes a hook driver while the Python queue's first step
   is still the loop (`py_loop` is `Some` for the queue of this run and worker), the summary
   carries `pytest_hint`: "this project has Python test files and the first step of the queue has
   not run: <the py_loop command> runs the project's own tests in a loop with nothing to fill;
   run it and read its trend before you fill these hooks". Use the py_loop command as the queue
   gives it (`dla harness gen --lang python --pytest`, or `dla harness run <H>` when the loop
   harness exists and has not run). No hint when `py_loop` is `None` (no test files, the loop
   wrote a series, an attempt failed, the queue is closed or blocked) and none for another
   runtime. Add `pytest_hint` to `KEPT_WHOLE` in `crates/dla-cli/src/out.rs` so the 2 KB bound
   never cuts it, exactly as `junit_hint` is kept (summary and last-resort paths).
5. `skills/detect-leak-anywhere/references/stages.md`: where the worker or core is told to run the
   batch command, one clause: the command is the one `dla run status` prints for the queue; a
   hook driver generated before the Python test loop ran carries `pytest_hint`. Do NOT touch
   `skills/detect-leak-anywhere/SKILL.md` (size limit) and do NOT touch `docs/VERIFY.md`.

## Tests (each seen failing under a mutation you apply and revert)

- `crates/dla-cli/tests/pytest_loop.rs` (read its helpers; the plain `project(p, true)` fixture has
  test files):
  a. the census note names `dla harness gen --lang python --pytest`;
  b. after probe, sweep, census: `dla harness gen --lang python --driver --next` succeeds and its
     summary has `pytest_hint` naming `dla harness gen --lang python --pytest`; the same project
     without test files has no `pytest_hint`; after a refused generation of the loop (the default
     PATH of the tests has no pytest; see the existing test
     `without_pytest_the_loop_is_refused_with_the_install_text_and_the_queue_goes_on` for how it
     pins the interpreter) the next hook driver has no `pytest_hint`;
  c. `dla ledger list` prints `totals.queue.python.command` equal to the queue command of
     `dla run status`, before the loop (the loop command) and after the refused generation (the
     driver batch).
- `crates/dla-cli/src/out.rs`: extend the unit tests that pin `junit_hint` so they pin
  `pytest_hint` the same way.
- `crates/dla-cli/tests/skill_text.rs`: pins for worker.md: it says to run the command
  `run status` prints, it names `harness gen --lang python --pytest`, and it does not contain the
  old sentence "run the batch command it prints (`dla --run <id> --worker <name> harness gen
  --lang <runtime> --driver --next`)".

Mutations by a small script that asserts the old text occurs exactly once; see each test fail;
revert. Before you finish, `grep -rIl "MUT_" crates harness skills docs bench agents hooks data`
must print nothing.

## Rules

- No commit, stash, checkout, reset, restore or branch change. Leave the tree dirty.
- Use the shared `target/` directory only (plain `cargo` from /home/user/dla, `-j 2`).
- No `rm -rf` on variable paths. Do not kill processes by name. Do not run `dla consent`. Do not
  start any model, agent or verifier. If an action is denied, stop and report it.
- Keep to this brief. If you find another defect, report it; do not fix it.
- Plain technical English in texts and comments, full sentences, no em dashes. Never name a real
  third-party project, library or its APIs in the repository. No model names, no attribution lines.
- If an existing test or golden file changes, say which and why; do not weaken a test.
- A pytest is at /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u72/pillow/.venv/bin
  (put it first on PATH to run the pytest-dependent tests as well); the default PATH has none.

## Checks before you report

1. `cargo fmt --all --check` clean.
2. `cargo clippy -j 2 --workspace --all-targets -- -D warnings` clean.
3. `cargo build -j 2 -p dla-cli` and `./target/debug/dla schema --check` clean (if a schema file
   changes because of the new field, regenerate it the way the repository does and say so).
4. `cargo test -j 2 --no-fail-fast -p dla-cli --test pytest_loop --test skill_text --test queue --test keep_going --test gate_flow --test py_site_columns`
   under the default PATH and again with the venv first on PATH; `cargo test -j 2 -p dla-cli --bins`;
   `cargo test -j 2 -p dla-workload --test harness_gen --test harness_pytest_loop`.
5. Do NOT run the whole workspace suite; the orchestrator runs it.

## Report (final message; read by the orchestrator)

Two sentences of result first. Then: files changed; each test with its mutation; existing tests or
golden files changed and why; anything in this brief that was wrong; what you did not verify.
Counts only, no command output.
