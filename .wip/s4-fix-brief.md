# S4 follow-up: four review findings (all low), then the dynamic set again

Your S4 work is verified (my two mutations caught, review done). A review with adversarial
verification confirmed four small defects. Fix them, test each, report to
`$sp/u164/report2.md` (sp = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad).

WAIT FIRST: a full test suite of mine is running in the shared `target/`. Do not edit any file and
do not run cargo until `$sp/u165/check.log` contains the line `ALLDONE` (wait with a background
until-loop, not chained sleeps). You may read code and plan meanwhile.

Same rules as before: no commit, no push, no branch switch; `target/` shared; literal rm paths;
kill only your own processes by pid; never weaken a test to pass; mutation marker comments must be
unique and removed.

## F1 plateau(): a constant-count block that rose and then FELL
`crates/dla-cli/src/summarize/rule.rs` `plateau()`: the new branch `stopped && resized && first > 0`
also catches a last interval < 0 (bytes [100,200,150], counts [1,1,1]) and says "ended flat ...
added -50 bytes". Restrict the branch to `last >= 0`; a fall goes to the NonLeak branch (N-WARM,
N-HWM) as before. Test: that series reads NonLeak with those supports; the flat case stays
Inconclusive. Update the doc comment.

## F2 stood_still: which series, and what the advice says
- `rule::stood_still` accepts a fall (e.g. [100,50,300]). Tighten it: constant positive count,
  bytes never fall (every pair `w[1] >= w[0]`), at least one equal pair, last > first.
- hprof.rs and node_heapsnapshot.rs OR it over every failing key; shim_groups.rs over every ranked
  stack. Compute it only for the series the summary is about: hprof/node — the best-failure key
  (the one the reasons already name); shim_groups — when a focus finding is given, the stacks that
  tie to it (use whatever the code already uses to tie a stack to the focus), else the top-ranked
  stack.
- shim_groups' text says "rerun the harness capture, which spaces its points that way": true only
  for a Rust harness's `--capture alloc` (1, 2, 4, 8 times its step); other shim captures still
  use the shared even spacing. Make the text true in every case (simplest: say to take captures
  whose work differs by a factor of two or more, and that a Rust harness's `--capture alloc`
  already does).
- Tests: a fall-then-rise series is not stood_still; a summary whose top stack failed for another
  reason while a small unrelated stack stood still gives the generic text, not the stood-still one.

## F3 summary_harness: every capture must belong to the harness
`crates/dla-core/src/grown.rs` `summary_harness` uses `measured_holders::capture_harnesses`, which
silently drops labels outside any `harness/<H>/`. A summary over three H1 captures plus a file
outside any harness becomes a lead on H1. Add a strict variant (do NOT change `capture_harnesses`'
behavior for its existing callers): no lead unless every capture label maps to a harness and they
are all the same one. Test: three H1 captures plus one outside label is no lead; the existing
positive test stays a lead. Optional, only if trivial: in `dump_grown`, run the cheap checks before
`is_summary_series` loads the series file.

## F4 Rust allocation-stack x when the driver lowers its step
`rust_alloc_points` (crates/dla-workload/src/harness/run.rs) sets `DLA_ITER` = step x rounds and
the shim copies it into x. The Rust driver's `fit()` (the driver template, `dla_harness.rs`
~275-305) lowers `step` when the rounds would not fit the time limit and still exits 0, so the
point's x overstates the work and `gate_at` reads a wrong rate (toward a miss). Fix: after a point
succeeds, read the driver's own dump of that point; when it records that it lowered its step, use
the units it actually did (its records' cumulative count; read the template to find which field)
as the x written into `shim-dump.jsonl`; if the actual count cannot be read, drop the point with a
limitation line that starts with the existing `--capture alloc` prefix ONLY if that is right for
`capture_wrote_no_stacks` semantics (fewer than three points left must still release the
capture-first item; check `dla_core::gate::capture_wrote_no_stacks` and the three existing Rust
limitation lines). Test: a unit test of the x choice from a dump with and without a lowered step.

## Then
- Mutation check for each new test (unique marker, restored, `grep -c` the marker = 0).
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and the test
  targets you touched (`cargo test -p dla-cli --bin dla summarize::`, the grown tests in dla-core,
  dla-workload's run.rs tests, the rust_driver and summarize_native integration tests). Not the
  whole suite: I run it.
- Then the dynamic set once: `./target/debug/dla-bench dynamic --dla-bin ./target/debug/dla
  --parallel 2 --json $sp/u164/dyn2.json` (build dla and dla-bench first). Expected: 63 pass plus
  1 xfail (rust-tree-parent-links); jvm-pool-warmup is a known flake: if it alone fails, run it
  alone up to 3 times and report. Compare confirmed/claim/rule-out/leaks with your last run. Delete
  the dynamic logs afterwards.
- Check free disk before building (`df -h /`); if under 2 GB, `cargo clean -p dla-cli -p dla-core
  -p dla-workload -p dla-bench` (nothing else).

Report: files changed, each fix as implemented, tests and which mutation made each fail, commands
and results, dynamic result, anything unsure.
