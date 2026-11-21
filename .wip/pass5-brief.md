# Pass 5: trigger verdict honesty, close-rest speed, residual rows in one command

Same agent as passes 3 and 4, same working rules (cargo -j 2, shared target/, no commits, no
stash/reset/checkout of files, never restore a file from a copy, never kill by name, literal
paths in rm, every behaviour change with a test SEEN failing, VERIFY rows at the end of
`## M7`, nothing in the repository names a real project, a model, a plan or an agent). Scratch
`<sp>/u39/` (exists); report with the Bash tool to `<sp>/u39/report.md`.
`<sp>` = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad.

Another builder works at the same time in the Go harness code: `crates/dla-workload/` (gen,
probes, targets, run), `harness/go/`, `crates/dla-core/src/queue.rs`,
`crates/dla-core/src/view.rs`, `crates/dla-cli/src/harness_cmd.rs`, the alloc-test code, and
`skills/.../references/go.md`. Do not touch those files. Your files: `crates/dla-core/src/trigger.rs`
(+ its tests), `crates/dla-cli/src/ledger_queue.rs`, `crates/dla-cli/src/ledger_cmd.rs`,
`crates/dla-core/src/ledger.rs` / `machine.rs` only if item 2 needs it, the gap command, the
report crate, gate.rs only for item 3.

Evidence: a real run on a large Go project; notes in
`<sp>/wip-notes/real-run-plugin-grpc-2.md`; its run directory is
`/root/dla-real/grpc-2/project/.dla/<run>/` (read it, copy files out; do not run `dla`
commands inside it).

## Item 1: "no trigger found" only when the holder was MEASURED flat
`crates/dla-core/src/trigger.rs`: `Trigger::NotFound` is returned when two entry-path
harnesses with different driver digests targeted the finding, were read, and the verdict was
not `grew`. In the real run both entry-path runs had no size reading for the holder (verdict
not `constant`: the holder was unmeasured), and the report said "Trigger: not found ... two
workloads through the public API did not grow it". That sentence is false.
Wanted:
- `not_found` needs two entry-path runs (different harnesses, different driver digests, fresh
  keys, read) whose verdict for the finding is `constant` (a measured, flat holder).
- A new outcome for "ran but nothing measured the holder" (name it as the code style suggests,
  for example `unmeasured`): two entry-path runs as above where at least one verdict is neither
  `grew` nor `constant`. Report line: "Trigger: not checked: the entry-path harnesses H7 and H9
  ran, but nothing measured the holder there (verdicts: executed, unknown)"; the Findings
  header counts it under "trigger not checked". The gate item of kind `trigger` is closed by
  it like by `not_found` (no endless loop), but after ONE unmeasured entry-path run the item's
  fix text must say what is missing: "the entry-path harness H7 ran but did not measure F: give
  it a size reading (a probe of the holder; for Go track the object, see the driver's hook
  comment), then run again", before it asks for a second harness.
- `run status` `triggers`, `ledger list` rows, `dla trend`'s `trigger`/`trigger_next` and the
  bundle's `verdict.json` use the same outcome names. The verifier text (agents/verifier.md)
  already tells the verifier to judge from `verdict.json`; add half a sentence that an
  unmeasured run is no evidence either way.
- Tests in `trigger_tests.rs`, `tests/trigger_items.rs`, dla-report `tests/trigger.rs`,
  and extend `crates/dla-cli/tests/trigger.rs` only if it stays fast.

## Item 2: `dla ledger close-rest` speed
On a ledger of 744 findings (1 700 ledger lines) `close-rest` ran for more than 120 s.
Measure first: build a synthetic ledger of that size in a test or a scratch script (for
example by running the static stage on a generated project with several hundred holders, or
by copying the real run directory to `<sp>/u39/` and running `close-rest` on the COPY) and
time it. Find the cause (likely one full ledger load, lock and view rebuild per closed
finding in `close_rest` going through `resolve_finding`). Wanted: one pass that keeps the
same rules and the same per-finding refusals and records, with the view rebuilt as rarely as
correctness allows; target under 10 s for that size on this machine. Every rule of
`resolve` must still apply per finding (the existing queue tests must pass unchanged); add a
test that closes a few hundred candidates and asserts the outcome is identical to closing
them one by one with `ledger resolve` (same states and reasons), plus a generous time bound
only if it is not flaky (otherwise report the timing and leave the bound out).

## Item 3: residual rows for a whole run in one command
Pass 4 found the per-series residual rule deliberate (a row of one series must not cover
another). Keep the rule. The busywork is the command count: a real run needed one
`dla gap S<n> --residual` per series (176 gate items). Wanted: `dla gap --residual --all`
(or the name that fits the existing CLI) computes and records the residual row of every
series that has none, with exactly what `dla gap S<n> --residual` records for each, prints
one bounded summary (count written, the series above threshold with their O-ids), and the
gate's fix text for a missing residual row names that one command when more than three
series miss a row. No change to what counts as covered. Tests: the rows written by `--all`
equal the rows written one by one (same evidence content), the fix text, a run with zero
series.

## Acceptance
`cargo fmt --all --check`; `cargo clippy --workspace --all-targets -j 2 -- -D warnings` (if
it fails only in files of the other builder, name them and run `--no-deps -p` on your
crates); `cargo test -j 2 --no-fail-fast -p dla-core -p dla-report -p dla-cli`;
`target/debug/dla schema --check`; `python3 bench/runner/test_grade.py`; the `skill_text`
test. Report: per item what changed, the test and that you saw it fail, timings of item 2
before and after, anything not done and why, anything unverified.
