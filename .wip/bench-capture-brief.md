# Brief N2: the benchmark's fixed policy follows the new "capture before a heap residual is left" rule

Repository: /home/user/dla (Rust workspace, branch `main`). The working tree holds uncommitted
changes of earlier builders (about 40 files). Leave them as they are; where you must edit one
of those files, add your own lines only. You are the only builder in the tree.
Scratch directory: /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u126
(create it; write nothing else outside the repository).

The machine has 4 cores. Use `cargo ... -j 3`, one cargo command at a time. Never touch
/root/dla-real.

## What is wrong

An uncommitted change in the tree added a rule to the tool: a heap residual above its
threshold is a gate item of kind `residual_capture` ("1 heap residual row of S1 over the
threshold and H1 has no summarized capture (no retention or allocation-stack record under
harness/H1/)", fix `dla harness run <H> --mode <mode> --capture alloc,heap, then the dla
summarize commands it prints`), and `dla ledger resolve <O> --as unresolved` for the open item
of such a residual is refused with `capture-not-summarized` until a capture of that harness
has been summarized (`crates/dla-core/src/gate.rs`: `RESIDUAL_CAPTURE_KIND`,
`uncaptured_residuals`; `crates/dla-cli/src/ledger_cmd.rs`).

The dynamic acceptance run (`dla-bench dynamic`, no model: a fixed policy in
`bench/runner/src/dynamic/policy.rs` and `ops.rs` drives `dla` through each fixture) does not
know the rule. Seen on the current tree, each run alone on a quiet machine:

- `go-subscription-workers` (archetype-leak): FAIL, `gate check --final: 57 item(s) missing:
  residual_capture H1:residual-capture: 1 heap residual row ...`
- `go-subscription-workers-closed` (decoy-control): FAIL, the same item, 4 missing.
- `node-socket-heartbeat` (archetype-leak): FAIL, the same item for H2, 91 missing.
- `node-socket-heartbeat-cleared`, `go-buffer-pool`: pass.

The whole dynamic run has never been done on this tree, so more fixtures may fail the same
way.

## The change

1. The fixed policy does what the gate asks, as a model is told to: when a gate lists a
   `residual_capture` item, or a resolve is refused with `capture-not-summarized`, it runs the
   capture command of the fix for that harness and then every `dla summarize` command that
   run printed (read how `ops.rs` runs captures and summaries today and reuse it), and goes on
   with what it was doing. It does this once per harness. Nothing under `bench/fixtures` is
   edited, no fixture's answer key, README or project changes, and no expectation of the
   benchmark is lowered.
2. Then run the WHOLE dynamic acceptance once: `./target/debug/dla-bench dynamic` (all
   fixtures; it takes 40 to 60 minutes; start it in the background, wait for it, do not run
   anything heavy beside it). Report the table. Known before this work: one fixture is an
   expected failure (xfail), and `jvm-pool-warmup` fails about one run in three for a reason
   of its own (its unbound alloc-test reads inconclusive under load); if it fails, run it
   alone twice more and report all three results.
3. For every fixture that showed the `residual_capture` item, report: the fixture's category
   (leak, non-leak, decoy), the residual and its threshold in bytes per unit, and what the
   summaries of the capture named. This tells the orchestrator whether the rule asks for a
   capture where there is nothing to find (a non-leak fixture with a heap residual over its
   threshold is worth a sentence: say what the heap growth is).
4. If a fixture fails for another reason after your change, report it with the first lines of
   its failure and do not change the tool to make it pass. If the rule itself looks wrong for
   a fixture (for example a capture that cannot be taken for that kind of harness), stop and
   report; do not weaken the rule.

## Tests

- A unit test next to the policy (the file has stand-in `dla` scripts for such tests: see the
  tests at the end of `ops.rs` and `policy.rs`): a gate that lists `residual_capture` makes
  the policy run the capture and the printed summaries once, then the gate again. Seen
  failing under a mutation you apply and revert.
- `cargo test -j 3 -p dla-bench` passes.

## Rules

- No `git commit`, `git stash`, `git checkout` (in any form), `git reset`, `git restore` or
  branch change. Leave the tree dirty.
- Shared `target/` only (`cargo` from /home/user/dla). No `rm -rf` on variable paths. Do not
  kill processes by name. Do not run `dla consent`. Do not start any model, agent or verifier.
  No package installs.
- Run each cargo command on its own, in the foreground, piped through `tail` or `grep` (the
  whole dynamic run is the one exception: background, output to a file in your scratch
  directory). If an action is denied, stop that action and report it; do not ask anyone to do
  it for you and do not try another way to the same end.
- Keep to this brief. If you find another defect, report it; do not fix it.
- Plain technical English, full sentences, no em dashes. No model names, no attribution lines.
- Mutations by a small script that asserts the old text occurs exactly once. Before you finish,
  `grep -rIl "MUT_" crates harness skills docs bench agents hooks data` must print nothing.

## Checks before you report

`cargo fmt --all --check`; `cargo clippy -j 3 --workspace --all-targets -- -D warnings`;
`cargo build -j 3 -p dla-cli -p dla-bench`; `cargo test -j 3 -p dla-bench`.

## Report (final message; read by the orchestrator)

Two sentences of result first. Then: files changed; the policy rule in one paragraph; the
whole table of the dynamic run (fixture, category, result) and its totals line; the list of
part 3; the test and its mutation; anything in this brief that was wrong; what you did not
verify. Counts only.
