# Brief L: one flat run does not end a run that has its budget left (real run tokio-2)

Rules as before: Sonnet only; no commits; shared `target/`, `-j 2`; a real plugin run and two
other builders are on this machine (run no `dla-bench`; small exact edits, no reformatting;
the others are in `ledger_cmd.rs`, `gate.rs`/`gate_cmd.rs`, `trend_cmd.rs` (one line),
`harness/*` templates, `dla-workload/src/harness/run.rs`, `gap/residual.rs`); literal paths
in `rm`; no process kills by name; unique-marker mutations reversed by exact replacement; no
test, fixture or policy weakened; never `dla consent` (unattended-runner environment on
`dla run init`). Free disk is about 3 GB: delete your build outputs and copies as you go.
Scratch `<sp>/u50/`, report `<sp>/u50/report.md`, short final message.
`<sp>` = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad

## What happened (evidence `<sp>/u36/tokio-2/`: run.json, transcript.jsonl, project/.dla)

Real plugin run on tokio-util 0.7.1 (documented leak: a grandchild token,
`parent.child_token().child_token()`, then `parent.cancel()`, leaks the 72-byte state
allocated in `CancellationToken::new`). 78 seconds, 17 turns, Status COMPLETE, "no leak
confirmed", budget 1 h 17 min, 1 minute used.

The model wrote one driver whose unit touched all 7 ranked candidates once: a
`ReusableBoxFuture`, one token with one child (cancelled for every third key), one
`spawn_pinned`. `native.live_bytes` stayed flat (truly: this unit leaks nothing). Every
target read `executed`, the queue was empty, `close-rest` closed them, the gate passed.
The unit never built a second level of tokens, so the leak was never triggered. Your focus
rule did not apply: no target was missed.

A flat reading of one arbitrary unit says that unit does not leak. It says little about a
holder whose leak needs a sequence (derive from a derived object, cancel the parent first,
fail half-way, overlap lifetimes). The run had 76 minutes to try a second unit and the tool
asked for none.

## Wanted: a second look while the budget lasts

1. When the measurement queue of a runtime has no pending candidate, the run is going and
   more than the floor of the budget is left (the existing floor rule), the queue is not
   done: the best-ranked candidates that were settled by a flat reading (`executed`, or
   `constant` from a whole-process or size reading) of ONE driver get a second look, a
   focus harness of at most `FOCUS_MAX` targets, best score first. Bound it so it ends: one
   second look per candidate, at most 3 second-look harnesses per runtime (9 candidates);
   put the numbers next to `FOCUS_MAX` with a one-line reason. A candidate that grew, was
   missed (your existing rules) or already ran in two different drivers is not part of it.
2. A second-look harness must be a different driver. `dla harness gen --driver --next`
   prints, in the driver file's target block, what the first driver did not vary, as
   concrete to-dos (general, not project specific): other call orders; objects derived from
   derived objects, three deep where the API allows it (a child of a child); the failing and
   cancelling paths (cancel or close the parent before the child, drop before completion,
   an error half-way); overlapping lifetimes (start the next before the last one ended);
   reuse of one long-lived object across units next to a fresh one per unit. It names the
   earlier harness so the model can read its unit. `dla harness run` refuses a second-look
   harness whose hook bodies are the same as those of an earlier harness of the same
   targets (compare the filled hook text, whitespace aside; you have `driver_digest` in the
   run file: look at what it covers) and asks for a `VARIES.txt` line per target in the
   harness directory saying what differs (the entry-path harness already has a
   `VARIES.txt` rule: reuse its check and its wording, do not build a second mechanism).
3. A candidate is settled for good by flat readings of two different drivers (the second
   one focused). `dla run status`, `dla ledger list` (`queue`, `settled_by`),
   `dla ledger close-rest` (leaves a candidate that awaits its second look, names the
   batch command), the final gate (a pending second look is a pending queue candidate
   while the budget is above the floor) and the report's Queue line (how many got a second
   look, how many of them grew) all say the same thing. At the floor the existing
   `time_budget` close applies.
4. Keep the first pass first: no second look while first-pass candidates are pending.
5. The bench's mechanical driver (`dla-bench dynamic`) must not be changed in what it
   accepts. Check how it drives the queue: if it would now see pending candidates it does
   not know how to serve, say so and stop before changing anything in `dla-bench`.

## Tests and checks

- Unit tests in `dla-core/src/queue.rs` for each rule (selection order, bounds, not while
  first-pass candidates are pending, settled after two different drivers, grew/missed
  excluded, floor), each seen failing once.
- Integration test (the existing queue tests' style): a run whose first driver settles its
  targets flat with budget left: status and close-rest show the second look and the batch
  command; `harness gen --driver --next` writes a focus harness with the to-dos and the
  earlier harness's name; `harness run` refuses identical hooks and a missing `VARIES.txt`;
  after a different, flat second driver the targets are settled and the gate item is gone;
  at the floor nothing is asked.
- Real tree: copy `<sp>/u18/tokio/proj` to `<sp>/u50/tokio` (never work in the original).
  `dla run init --budget auto`, probe, sweep, census; first driver: the unit of the real run
  (`<sp>/u36/tokio-2/project/.dla/20261121-165936-5aa7/harness/H1/driver/dla_harness.rs`);
  run, trend, then show: `dla run status` asks for the second look; the generated focus
  harness for the cancellation-token candidates carries the to-dos; fill it with a unit
  that follows them (a three-level chain, cancel the parent first, drop in varying order),
  write `VARIES.txt`, run, trend: report what `native.live_bytes` does, what trend prints
  next, and how far the printed commands carry (`--capture alloc`, `summarize
  alloc-stack-diff`, sibling, `ledger test`, `alloc-test`, confirm). Stop and report at the
  first command that is refused or wrong.
- `cargo fmt --all --check`; clippy on the crates you touched; `cargo test -j 2 -p dla-core
  -p dla-cli` (name failures that are not yours); `dla schema --check`; skill text test and
  docs (`skills/.../references/*.md` where the queue is described; `docs/VERIFY.md` rows).

Report: what changed, tests with what was seen failing, the real-tree outputs, what is
unverified, anything wrong you saw outside this brief.
