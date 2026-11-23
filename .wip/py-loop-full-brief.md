# Brief L: a narrowed Python test loop does not replace the loop over all test files

Repository: /home/user/dla (Rust workspace, branch `main`, clean working tree at the start).
Scratch directory: /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u86.
The `target/` directory is empty: your first cargo command rebuilds everything (15 to 20 minutes).

## What happened (a real unattended run)

An open Python queue of a project with test files starts with the project's own tests in a loop
(`dla harness gen --lang python --pytest`; `pytest_loop_step` in `crates/dla-core/src/queue.rs`).
The loop exists because a leak that nothing holds has no static candidate: only a broad workload
shows the line that grows, and the broadest one is the whole test suite.

In a real run the Python worker ran `dla harness gen --lang python --pytest` (harness H2, 158
test paths), did not run it, then ran
`dla harness gen --lang python --pytest --tests <three test files>` (harness H3), ran H3, read
its trend and went on to the driver batches. The queue treated the first step as done, because
`pytest_loop_step` returns `None` as soon as ANY pytest-loop harness wrote a series. The three
files did not reach the leaking call, so the run never saw its line. An earlier run with the
full loop did.

## The change

1. A pytest-loop harness knows whether it is the full loop (generated without `--tests`: every
   discovered test file of the working directory) or a narrowed one (`--tests` given). Find how the
   manifest of a pytest-loop harness records its selection today (`HarnessManifest` in
   `crates/dla-core/src/harness.rs`, `gen.rs` in `crates/dla-workload/src/harness`); add a field
   only if nothing there tells the two apart, and regenerate the schema files if a schema changes
   (`dla schema --out schemas`, then `dla schema --check`).
2. `pytest_loop_step`: the first step is satisfied only when
   - a FULL loop wrote a series, or
   - an attempt of the FULL loop failed (the refused generation that is already recorded, a
     failed run, a run with no usable point: the cases that end the step today, but only when
     they are the full loop's).
   A narrowed loop that ran or failed changes nothing: while a full loop harness exists and has
   not run, the step is `dla harness run <that harness>`; while none exists, it is
   `dla harness gen --lang python --pytest`. After the full loop failed, a narrowed loop is the
   agent's own fallback and the queue goes on with the driver batches as today.
   Everything that reads the step follows (`dla run status`, the queue's gate item, the
   loop-first gate fixes, `pytest_hint`, `dla ledger list`): check each still prints the full
   loop while it is pending.
3. `dla harness gen --lang python --pytest --tests ...` is not refused. While the full loop is
   still the queue's first step, its summary carries the kept-whole hint (`pytest_hint`, the key
   that `crates/dla-cli/src/out.rs` never cuts): "this loop runs N of the M test files; the first
   step of the queue is the loop over all of them, which reaches code that no candidate names:
   <the step command>; a narrowed loop does not replace it". No hint once the full loop wrote a
   series or its attempt failed.
4. Texts: `skills/detect-leak-anywhere/references/python.md` (the pytest-loop bullet, where
   `--tests a,b` is described): `--tests` narrows the loop and is for a loop that cannot run in
   full (it stopped with a `DLA-PYTEST` line) or for a second look at a part; the queue's first
   step is the loop over all test files and a narrowed loop does not replace it.
   `skills/detect-leak-anywhere/references/stages.md`: one clause in the paragraph that introduces
   the Python loop. `agents/worker.md`: in the sentence that names
   `harness gen --lang python --pytest`, add "without `--tests`". Do NOT touch
   `skills/detect-leak-anywhere/SKILL.md` (size limit). `docs/VERIFY.md`: one new row at the end
   in the style of the last rows (left column: "A test loop narrowed to a few files counted as the
   queue's first step", state `corrected`, the facts above without naming the project).
   Pins in `crates/dla-cli/tests/skill_text.rs`.

## Tests (each seen failing under a mutation you apply and revert)

- Unit tests next to `pytest_loop_step` (the existing ones build pytest-loop harnesses by hand:
  `pytest_harness` in the test module): a narrowed loop that wrote a series leaves the step at the
  full loop (`dla harness gen --lang python --pytest` when no full loop harness exists,
  `dla harness run <H>` when one is generated and not run); a full loop that wrote a series ends
  the step; a full loop whose attempt failed ends the step, and a narrowed loop after that changes
  nothing; a narrowed loop whose run failed leaves the step at the full loop.
- CLI test in `crates/dla-cli/tests/pytest_loop.rs` (read its helpers; the tests there run without
  pytest by default and some skip without it): generate the full loop, generate a narrowed one with
  `--tests`, assert the narrowed one's summary has `pytest_hint` naming `dla harness run <full H>`,
  and that `dla run status` and `dla ledger list` still print the full loop's command. If the
  helpers let a test write a series for a harness by hand (as the queue tests do), also assert that
  a series of the narrowed harness does not end the step; if that needs pytest, guard it the way
  the neighbouring tests do.
- Existing tests that pinned "any pytest loop ends the step" change to the new rule; say which.

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
  library or its APIs in the repository. No model names, no attribution lines.
- If an existing test or golden file changes, say which and why; do not weaken a test. Do not edit
  anything under `bench/fixtures`.
- Disk is short on this machine: do not run the whole workspace suite and do not build targets you
  do not need. A pytest for the pytest-dependent tests is at
  /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u72/pillow/.venv/bin
  (put it first on PATH for those runs only).

## Checks before you report

1. `cargo fmt --all --check` clean.
2. `cargo clippy -j 2 --workspace --all-targets -- -D warnings` clean.
3. `cargo build -j 2 -p dla-cli` then `./target/debug/dla schema --check` clean.
4. `cargo test -j 2 -p dla-core --lib queue`,
   `cargo test -j 2 --no-fail-fast -p dla-cli --test pytest_loop --test queue --test skill_text --test keep_going --test gate_flow`
   (default PATH, then again with the venv first on PATH), and
   `cargo test -j 2 -p dla-workload --test harness_pytest_loop` with the venv first on PATH.

## Report (final message; read by the orchestrator)

Two sentences of result first. Then: files changed; each test with its mutation; existing tests
changed and why; anything in this brief that was wrong; what you did not verify. Counts only.
