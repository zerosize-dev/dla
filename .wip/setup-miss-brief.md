# Builder J: a guarded writer is "not reached" only after drivers with different set-ups missed it

Repository: /home/user/dla. Work in the working tree. Do NOT commit, push or switch branches.
The tree has uncommitted, checked work from builders G, H and I and from me (grown.rs, gate.rs,
summarize/node_*, dla-preload.cjs and more). It is in CI now. Do not revert or reformat anything
you did not write. Add rows at the end of docs/VERIFY.md (re-read it just before editing).

## What a real run showed (do not name the project, its types or its files anywhere in the repository)

A real Rust library. Its documented leak is a holder `Sender.pending` (a Vec) whose three adding
lines sit under `if let Err(Full(value)) = self.ring.push(value) { self.pending.push(value) }`.
The ring buffer is full only when nothing drains it, which happens when the library's consumer
(a reporter) is never installed. The run:

1. H2 (a batch) installed a reporter in `setup()` and created spans in `unit()`. Trend verdict:
   `writer_not_executed`. The function ran; the three adding lines ran 0 times.
2. The queue gave the target a focus harness, H3. Its target block said, correctly, that "a
   state that normal set-up removes (a full buffer, a missing configuration, an error path, a
   second thread) needs a unit or a set-up that does not remove it". It listed each adding line
   with the guard lines above it (`if let Err(Full(value)) = ...`).
3. The model filled H3 by copying H2's `setup()` exactly and wrapping H2's `unit()` in
   `for _ in 0..8 { ... }`, so 8x the spans. The reporter still drained the buffer, and the
   lines still ran 0 times.
4. Two different drivers (by `driver_key`), one of them focused, had now missed the target, so
   the queue settled it `NotReached` (crates/dla-core/src/queue.rs, the "Not reached" block near
   `misses`/`driver_key`). `dla ledger close-rest` closed it `suspected_static_only`
   (not_exercised). The run ended COMPLETE after 3 of 167 budget minutes.

The second driver did not test the reason the block itself named: a state the set-up removes. A
change to the unit alone cannot open a guard whose state the set-up fixes.

## The change

In the queue's standing rule, a target whose misses are `writer_not_executed` (the function ran,
the adding lines did not) settles `NotReached` only when the drivers that missed it include two
whose set-ups differ: the code of the `DLA-HOOK setup` body (comments and whitespace removed,
like `harness::driver_code`). Otherwise it stays `Pending`, the same way the Rust
untried-feature rule keeps a target open "however many drivers missed it".

- Get the set-up body for every hook-driver runtime (Python, Node, Rust, Go, JVM). Find the
  `DLA-HOOK setup` marker in each driver template (harness/<rt>/ and the generator in
  crates/dla-workload/src/harness/). The body is the code of the setup function after the marker
  comment, up to the next `DLA-HOOK` marker or the function's end. Choose the simplest reliable
  rule and test it on each runtime's generated driver.
- Store a set-up key next to `driver_key` wherever runs are recorded for the queue. Read how
  `driver_key` is computed and stored, and do the same, so older runs without the key behave as
  today (treat a missing key as "unknown, different": never block on old records).
- Only `writer_not_executed` misses are subject to this. A target whose code never ran
  (`not_executed`, `not_loaded`) keeps today's rule.
- Where the queue explains a pending target (the focus harness's target block, the queue
  description in `dla run status` and the gate, the trend's `next` line after the second
  miss), say it in one sentence. For example: "F41: H2 and H3 missed its adding lines with the
  same set-up; the guard above them (line 37: ...) may need a state the set-up removes: change
  `setup()` too (for example, leave out what consumes or resets that state), or resolve F41
  with the reason the state cannot be reached". Use the guard lines already computed for the
  block (grep "above it, line").
- Do not let this keep a run from ending at the budget floor. When the queue closes for time,
  the target closes as today, with reason `time_budget`. Check that `close-rest` and the floor
  rule do that.

## Tests

- Unit tests in queue.rs next to `one_focused_miss_after_a_broad_miss_settles_and_the_same_driver_twice_does_not`
  (see its `ran(...)` helper, which writes driver text):
  1. Two drivers miss a target `writer_not_executed`, one focused, both with the same set-up
     body and different units: `Pending`.
  2. The same with different set-up bodies: `NotReached`.
  3. Two `not_executed` misses with the same set-up: `NotReached`, as today.
  4. Old runs with no set-up key: as today.
- A CLI test, a Python or Node scene (fast), of the whole flow. A driver misses a guarded writer
  twice with the same `setup` and `close-rest` leaves the target, naming the set-up. A third
  driver with a different setup that still misses lets it close. A test that runs the guard
  state (setup leaves out the consumer) gets `grew`. If a full flow is too slow, do the first
  two steps.
- Mutation check: make the set-up comparison always "different", see tests 1 and the CLI test
  fail, and restore it.

## Docs

- skills/detect-leak-anywhere/references/stages.md (or wherever "not reached" or focus harnesses
  are explained; find them with grep): one sentence. Do not touch SKILL.md.
- docs/VERIFY.md: one row in neutral words ("a real Rust library run").

## Rules

Shared `target/` only. Run `cargo test -p dla-core`, the dla-cli and dla-workload tests that
mention not reached, focus or writer_not_executed (grep), the new tests,
`cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all --check`. The disk
is tight: if `df -h /` shows less than 3 GB free, delete stale hash dirs older than 120 minutes
under the literal paths /home/user/dla/target/debug/build/dla-cli, .../dla-core and
.../dla-workload (`find <literal path> -mindepth 1 -maxdepth 1 -type d -mmin +120 -exec rm -rf {} +`).
No `rm -rf "$VAR/..."`. No benchmarks. Never skip or weaken a test. No project names,
attribution lines or model names in the repository.

## Report (final message)

List the files changed. Give how the set-up body is found per runtime, where the key is stored,
the standing rule as implemented, the texts, the tests with what each proves, the mutation check,
the commands with pass or fail counts, and anything you are unsure of.
