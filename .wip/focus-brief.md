# A target that a batch missed gets a harness of its own

Follow-up for the builder of the Go batches (same working rules as your first brief:
cargo -j 2, shared target/, no commits, no stash/reset/checkout of files, never restore a
file from a copy, never kill by name, literal paths in rm, every behaviour change with a test
SEEN failing, VERIFY rows at the end of `## M7`, nothing in the repository names a real
project, a model, a plan or an agent). Scratch `<sp>/u38/` as before; append your report to
`<sp>/u38/report2.md` with the Bash tool.
`<sp>` = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad.

Three other builders edit the tree now: Node harness (`harness/node/`, Node parts of
dla-workload, dla-bench node policy), Rust harness (Rust parts of dla-workload, crate
features, a coverage point, the shared driver-block text: one "life of the objects" rule; it
has ALREADY changed `Index::standing` so that verdict `unknown` no longer settles a target:
keep that), and a speed pass (trend and other commands on large ledgers: no behaviour change,
it may restructure loading inside `crates/dla-cli/src/trend/` and `ledger_cmd.rs`). Re-read a
file right before each edit and use exact replacements.

## Evidence
Read `<sp>/wip-notes/real-run-plugin-spring-4.md` and again
`<sp>/wip-notes/real-run-plugin-grpc-2.md`. In both runs the documented leak was a declared
target of two batches (9 to 16 targets each). The model wrote code for two or three targets
per batch; the verdict of the others was `not_loaded` or `not_executed`; after the second
batch the two-drivers rule settled them and they were closed as not exercised. Nothing ever
attempted them.

## The rule (hook-driver harnesses of the measurement queue only; entry-path harnesses,
## JUnit repeats and command-mode harnesses keep their present rules)
Definitions
- A run REACHES a target when its verdict is `grew`, `changed` or `executed`.
- A run MISSES a target when its verdict is `constant`, `not_loaded`, `not_executed` or
  `unknown` (the harness ran and its trend was read).
- A harness is FOCUSED when it declares at most 3 targets in all (siblings included).
Standing of a ranked candidate (`Index::standing`, `crates/dla-core/src/queue.rs`)
- `Settled(Moved)` and `Settled(Executed)`: as now.
- Settled as not reached (replace `Why::TwoDrivers`; keep the serialized name stable if
  stored records or tests of other crates read it, else rename to what it now means): the
  target was missed by two different drivers (different driver keys, as now) AND at least one
  of those runs was of a focused harness.
- Otherwise `Pending`, in one of two kinds that the queue exposes: FRESH (no run declared it
  yet, or its harness has not run or was not read) and MISSED (at least one run missed it).
Batches (`--next`, all runtimes; for Go inside the package rule you built)
- If the best-ranked pending candidate is MISSED, the batch is a FOCUS batch: that candidate
  and up to two more MISSED pending candidates, chosen in this order: same owner, then same
  file, then (Go: same package only; other runtimes) next by rank. No sibling expansion, no
  FRESH candidates. `--next N` with N above 3 is capped at 3 for a focus batch, and the output
  says it is a focus batch and why.
- Otherwise the batch is the broad batch as now, of FRESH candidates only (a MISSED candidate
  is never put into a broad batch again).
- The generated harness records that it is a focus batch (`harness.json`, additive field) and
  which earlier harness missed each target.
Texts
- Driver block of a focus batch, above the targets: "Focus harness: these targets were
  declared in H<k> and their code did not run there. This harness exists for them alone.
  Call the writer function of each (named below) with a new key per unit. A target that no
  driver can reach here (it needs a container, a peer, another platform) is not left
  silently: after the run, `dla ledger resolve <F> --as unresolved --reason env_unavailable
  --remedy "<what it needs>"`." Per target: the earlier harness and its verdict with the
  reason text the trend gave.
- Driver block of a broad batch: one line: "A target whose code does not run in this harness
  comes back in a small harness of its own; reach as many as one unit naturally can."
- `dla trend` of a hook-driver series with misses: the `next` list starts with one entry
  that names the missed targets (ids, at most 8, then a count) and the batch command that
  will give them a focus harness. Keep the output bounded.
- `dla run status` queue item and `dla ledger list` rows: the next batch says "focus" and
  names its targets; a MISSED candidate's row says where it was missed and that a focus
  harness is pending; a candidate settled as not reached says in which focused harness.
- `close-rest` record, the report's `Queue:` line and `Targeted:` lines: "not reached" is
  only what the new rule settled; missed-once candidates left pending at the end count as
  "missed once, no focus harness" (name the count; do not fold them into "not reached").
- Skill text: `skills/detect-leak-anywhere/references/stages.md` (the batch loop) and
  `references/recovery.md` (`target-pending`), and the worker agent `agents/worker.md` where
  it describes the loop: say in two or three sentences how misses come back as focus
  harnesses and that a target is given up only with a reason. SKILL.md has word limits
  checked by the `skill_text` test: change it only if a sentence there becomes wrong, and
  keep its size.
Refusals
- `target-pending` (closing a pending candidate with not_exercised, no_harness or
  time_budget while the queue is open) now also holds for a MISSED candidate until the new
  rule settles it; its fix text for a MISSED candidate names the focus batch command.
  Reasons env_unavailable, tool_missing, build_fails, privilege_declined stay accepted as
  today (check what `refuse_target_pending` in `crates/dla-cli/src/ledger_cmd.rs` does with
  them and do not widen or narrow that; say in your report what it does).

## Tests (each seen failing without the change)
Unit tests of `standing` and the batch choice (two big batches missing a target leave it
pending and MISSED; a focus batch is chosen when the best-ranked pending is MISSED, with the
choice order; one focused miss after a broad miss settles; two focused misses with the same
driver key do not; `unknown` is a miss; `constant` is a miss; a reached target is never in a
focus batch). CLI tests on a small synthetic project (Python is the fastest runtime; look at
`crates/dla-cli/tests/queue.rs` and `rank_score.rs`): batch 1 broad with a unit that reaches
only some targets; `trend` names the misses; `--next` gives a focus harness of at most 3 with
the block text; `ledger resolve <missed> --as suspected_static_only --reason not_exercised`
is refused `target-pending` with the focus command; after the focus harness misses again the
candidate is settled and `close-rest` closes it; the report lines. Go: one test that a focus
batch stays inside one package. Update existing tests that assert the two-drivers behaviour
to the new rule; list every existing test you changed and why.

## Acceptance
`cargo fmt --all --check`; `cargo clippy --workspace --all-targets -j 2 -- -D warnings`;
`cargo test -j 2 --no-fail-fast -p dla-core -p dla-workload -p dla-cli -p dla-report`;
`target/debug/dla schema --check` (regenerate if a schema changed);
`python3 bench/runner/test_grade.py`; `cargo test -j 2 -p dla-cli --test skill_text`;
`dla-bench dynamic --only python-` with your binary (the fixed policy runs with the queue
off: it must be unaffected; say if any fixture changes). Also say what the comparison loop on
`go-gc-tuned-batch` that you left running showed (logs `gc-new-N.log` / `gc-old-N.log`).

## Report
What changed (files, functions), the rule as built if it differs from the above and why,
tests and that you saw each fail, existing tests changed, what you did not do and why, what
is unverified.
