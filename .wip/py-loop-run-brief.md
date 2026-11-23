# Brief M: a test loop whose run was cut is not a blocker, and the texts stop telling the agent to use the shell's background

Repository: /home/user/dla (Rust workspace, branch `main`, a dirty working tree that holds one
unfinished change; all of it is to stay). Scratch directory:
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u87.
The `target/` directory is empty: your first cargo command rebuilds everything (15 to 20 minutes).

## What happened (a real unattended run)

The first step of an open Python queue is the project's own tests in a loop
(`dla harness gen --lang python --pytest`, then `dla harness run <H>`). On a real library one run
of the suite takes about two minutes inside the loop and the whole series about twenty.

The Python worker generated the full loop (H2) and started it like this, in one tool call:

    dla ... harness run H2 --iterations 3 > $TMPDIR/h2.out 2>&1 &
    sleep 90; tail -c 800 $TMPDIR/h2.out

The shell background job died when the tool call returned: the harness directory has ticks in
its dump, no checkpoint, and no `attempt.json`. The worker saw an empty file, tried
`dla ledger resolve <F> --as unresolved --reason time_budget` (refused with `harness-not-run`:
"harness H2 (python) was generated and never run"), and then took the other way that refusal
offers:

    dla ledger resolve intake.workload --status unresolved --reason env_unavailable \
      --remedy "pytest-loop harness H2 ... did not finish a series inside the worker deadline; ..." --runtime python

That was accepted. It closed the Python queue, `dla ledger close-rest` closed 277 findings as
`not_exercised`, and the run ended in 28 minutes with nothing measured. The worker's deadline was
two hours and twenty-five minutes away.

Two causes.

1. A text told it to. `skills/detect-leak-anywhere/references/python.md` says "start it in the
   background and wait for it" and `skills/detect-leak-anywhere/references/stages.md` says "start
   `dla harness run <H>` for it in the background (or let the tool move it there) and wait for
   it". An agent reads "in the background" as the shell's `&`, and that job is killed when the
   command returns. In an earlier run, before those sentences existed, the worker ran the loop as
   a plain foreground command, the tool moved the long command to the background by itself and
   reported its end, and the loop wrote its series.
2. The blocker was accepted although the only thing that had happened was a run cut before its
   first result. A test loop needs nothing written, so "no harness can be made" cannot be true
   for it, and nothing says the environment cannot run the workload until the loop has run to an
   end.

## The change

a. Texts. Replace the two sentences. The new wording, in both files (python.md pytest-loop bullet;
   stages.md next to the 600000 ms sentence): the Python test loop runs up to its time limit of
   1800 s. Run `dla harness run <H>` for it as a plain foreground command with the tool's longest
   timeout. When the tool moves the command to the background because it runs long, wait for the
   notice that it ended and do other read-only work meanwhile. Never start it with the shell's
   `&` or `nohup` and never put it under `timeout`: a shell background job is killed when the
   command returns, and a cut run writes no series. `agents/worker.md`: one sentence with the same
   content where the loop command is named. Do NOT touch `skills/detect-leak-anywhere/SKILL.md`
   (size limit). Update the pins in `crates/dla-cli/tests/skill_text.rs` (one of them pins the old
   sentence) and add a pin that the words "in the background and wait" are gone from both files.
b. The blocker. `dla ledger resolve intake.workload --status unresolved --reason <a blocker reason>`
   for a runtime (the `--runtime` of the answer, or every runtime when it has none) is refused
   while that runtime has a test-loop harness (`HarnessManifest::is_test_loop()`: the JUnit loop
   and the pytest loop, full or narrowed) that was generated and has neither a series nor a
   recorded finished attempt (find what `dla harness run` leaves in `harness/<H>/attempt.json`
   and when; a run that was cut leaves nothing or an unfinished record). New refusal code, for
   example `loop-not-finished`; add it where the codes live and to the refusal table in
   `skills/detect-leak-anywhere/references/evidence-and-report.md`. The message says: the test
   loop <H> of <runtime> needs nothing written, and its run did not reach an end (never started,
   or cut before its first result), so nothing shows yet that the environment cannot run the
   workload. The fix: `dla harness run <H>` as a plain foreground command with the tool's longest
   timeout, wait for its end (it stops by itself within its time limit); a run that fails records
   why and the queue goes on with the driver batches; then, if nothing can run, record the
   blocker.
   Once the loop has a finished attempt (a series, or a failure the tool recorded), today's rules
   apply unchanged (`workload-ran` when a series exists, the blocker accepted after a recorded
   failure).
c. The existing refusal `harness-not-run` (find it: "was generated and never run") offers "or its
   blocker recorded on intake.workload". For a test-loop harness that offer is now wrong: its fix
   text names only the run (the same wording as in b). For a hook driver the text stays as it is.
d. `docs/VERIFY.md`: one new row at the end in the style of the last rows (left column: "A test
   loop cut by the shell was recorded as a blocker and closed the queue", state `corrected`, the
   facts above without naming the project, the change, the tests). In the same file, the row whose
   left column starts "A Python registration table grown by the driver's own keys" or the row
   about the test loop mentions the background sentence if it was recorded there: search for "in
   the background" and correct any sentence that describes the old advice as current.

## Tests (each seen failing under a mutation you apply and revert)

- CLI tests (find the tests of `workload-ran` and of the blocker per runtime:
  `crates/dla-cli/tests/workload_runtime.rs`; and the pytest-loop tests in
  `crates/dla-cli/tests/pytest_loop.rs`; reuse their helpers):
  1. a Python run with a generated pytest loop that never ran: the blocker for python is refused
     with the new code and the fix names `dla harness run <H>`; (generating the loop needs pytest:
     if the helpers cannot build a pytest-loop harness without pytest, build the manifest by hand
     the way the queue unit tests do, or guard with `have_pytest()` and say so);
  2. the same with a loop whose run failed and left a recorded failed attempt: the blocker is
     accepted as today;
  3. a runtime with only a hook driver that was generated and never run: the blocker is accepted
     as today (pin the unchanged behaviour);
  4. `harness-not-run` for a test loop prints no blocker offer; for a hook driver it prints it as
     today.
- skill_text pins for item a.

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
- Do not run the whole workspace suite. A pytest for the pytest-dependent tests is at
  /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u72/pillow/.venv/bin
  (put it first on PATH for those runs only).

## Checks before you report

1. `cargo fmt --all --check` clean.
2. `cargo clippy -j 2 --workspace --all-targets -- -D warnings` clean.
3. `cargo build -j 2 -p dla-cli` then `./target/debug/dla schema --check` clean.
4. `cargo test -j 2 --no-fail-fast -p dla-cli --test workload_runtime --test pytest_loop --test skill_text --test queue --test keep_going --test gate_flow`
   (default PATH, then again with the venv first on PATH) and `cargo test -j 2 -p dla-core --lib`.

## Report (final message; read by the orchestrator)

Two sentences of result first. Then: files changed; each test with its mutation; existing tests
changed and why; anything in this brief that was wrong; what you did not verify. Counts only.
