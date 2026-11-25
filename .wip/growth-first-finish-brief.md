# Brief X2: finish the change "unexplained heap growth leads the run"

Repository: /home/user/dla (Rust workspace, branch `main`). An earlier builder did most of this
change and stopped before its tests and checks; its work is uncommitted in the tree. Your job is
to finish it. The full brief it worked from is
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/wip-notes/growth-first-brief.md:
read it for the intent, the rules and the checks. Do not read any other file in that wip-notes
directory. Scratch directory: /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u130
(create it; write nothing else outside the repository).

The machine has 4 cores and about 6 GB of free disk. Use `cargo ... -j 3`, one cargo command at a
time. Never touch /root/dla-real. The container can be restarted without warning: keep each
background job short enough to lose, write its output to a file in your scratch directory, and
after a restart look at the tree and the files before you go on.

## What is already in the tree (the earlier builder's report; check it, do not redo it)

- Part 1: `crates/dla-cli/src/gap_cmd.rs` `write_residual_rows` (calls `residual::run`; `Ok(None)`
  on `residual-unavailable`); `crates/dla-cli/src/trend_cmd.rs` writes the residual rows of its
  series at the end of `dla trend`, new key `residual_rows`, and `capture_entry` gives a first
  `next` entry and an `_attention` entry when `gate::uncaptured_residuals` has the series.
- Part 2: `crates/dla-core/src/error.rs` `CAPTURE_FIRST = "capture-first"`;
  `crates/dla-cli/src/harness_cmd.rs` refuses `harness gen --next` with it while
  `uncaptured_residuals` is not empty, unless `--probe` or `--entry` is given.
- Part 3 (done, mutation-tested): `crates/dla-parsers/src/node/snapshot.rs`, `graph.rs`,
  `holders.rs`; test `crates/dla-cli/tests/harness_capture_node.rs::a_heap_summary_names_the_module_array_and_no_path_goes_through_the_weak_ref_job_list`.
- Part 4 (done, mutation-tested): `crates/dla-core/src/queue.rs` `Index::second_looks_used`,
  `crates/dla-cli/src/ledger_queue.rs`; test
  `crates/dla-cli/tests/second_look.rs::close_rest_leaves_the_flat_candidates_for_their_second_look`.
- Bench policy: `bench/runner/src/dynamic/policy.rs` `run_line_capture_first` and test
  `a_next_batch_refused_with_capture_first_takes_the_capture_and_is_repeated` (never built).
- Part 5 texts: `skills/detect-leak-anywhere/references/evidence-and-report.md`, `stages.md`,
  `node.md`; three rows at the end of `docs/VERIFY.md`; test
  `crates/dla-cli/tests/skill_text.rs::the_texts_say_what_a_site_column_counts_and_the_three_growth_first_changes`
  (compiled, never run).
- A per-file run of every dla-cli test file: script
  /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u123/runall.sh,
  results in `u123/summary.txt` (one line per test file, `ALLDONE` at the end) and
  `u123/t-<name>.txt`. Known failure:
  `second_look.rs::one_flat_run_gets_a_second_look_with_a_different_driver_while_budget_lasts`
  (refused with `capture-first`: its Python scene's heap residual row is now written by
  `dla trend`).
- Other files in the tree are earlier checked work (limit.rs, run.rs, stop.rs, dla_pytest_loop.py,
  sitecustomize.py, python.md, SKILL.md and more): leave them, add your own lines only.

## What is left, in order

1. If `u123/summary.txt` does not end with `ALLDONE`, wait for it (it runs on its own; do not start
   cargo while it runs; check with `pgrep -f u123/runall.sh`). If the script is not running and the
   file is incomplete, run the missing test files yourself the same way, one at a time.
2. Every test file that fails: read why. For each failing test caused by the new residual rows or
   the `capture-first` refusal, decide: if the heap growth of its scene is part of what the test
   tests, the scene takes the capture and its summaries before `--next` (as a model is told to);
   if the growth is an accident of the scene, keep the scene under the floor (64 KiB of
   unattributed growth over the series, `MIN_CAPTURE_GROWTH_BYTES` in
   `crates/dla-core/src/gate.rs`) by its own data. Never weaken an assertion and never change the
   rule to make a test pass. A test that fails for another reason: report it with its first lines.
3. The tests of parts 1 and 2 that the full brief lists (the scenes of
   `crates/dla-cli/tests/residual_capture.rs`, Node and Python), and the check that
   `residual_capture` comes before the queue item in `dla gate check --stage classify`, `--final`,
   `dla run status` and the stop hook's text. Each seen failing under a mutation you apply and
   revert (a small script that asserts the old text occurs exactly once). Capture one real
   `dla trend` output (first lines) for part 1.
4. Build dla-bench and run the policy test; then all the checks of the full brief.
5. The WHOLE dynamic acceptance once: `./target/debug/dla-bench dynamic` (all fixtures, 40 to 60
   minutes; background, output in your scratch directory; run nothing heavy beside it). One
   fixture is an expected failure (xfail). `jvm-pool-warmup` fails about one run in three for a
   reason of its own: if it fails, run it alone twice more. A fixture that fails because of the
   new refusal or the residual rows in a way the policy cannot fix without lowering an
   expectation: stop and report it; do not loosen the rule.
6. Delete /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u123/proj
   (a scratch copy, about 267 MB; a literal path).

## Rules

The rules of the full brief hold (no git commands that change state, shared `target/`, no
`rm -rf` on a variable path, no killing by name, no `dla consent`, no model or agent, no
installs, one cargo command at a time, plain English without em dashes, no third-party names in
the repository, no model names or attribution lines). If an action is denied, stop that action
and report it; do not ask anyone to do it for you.

## Report (final message; read by the orchestrator)

Two sentences of result first. Then: each failing existing test with what you did and why; the
tests of parts 1 and 2 with their mutations; one real `dla trend` output; the check results; the
whole table of the dynamic run with its totals line; anything wrong in the earlier work you
found; what you did not verify. Counts only.
