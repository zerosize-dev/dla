# Round AA: an abort while work is open needs a failure the run recorded

Repository /home/user/dla on `main` (clean at 92fc0d6). Rules: build with
`export CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_PROFILE_TEST_DEBUG=line-tables-only CARGO_INCREMENTAL=0`;
about 1.8 GB of disk is free: run only the test targets you touch; no commit, push, stash, branch switch,
`cargo clean`; no Claude or model names; never name a third-party project or its APIs in the repository;
every new test fails under a mutation (unique marker, run, restore, `touch`, marker count 0); plain short
English in code comments, refusal texts and docs.

## Why (measured on three real runs)

Three unattended runs on real libraries ended early through `dla run abort` while the budget was far over
its floor and the measurement queue was open. The latest: 11 of 180 minutes used, 25 of 78 ranked candidates
targeted, every harness attempt `ran` with exit 0 (no failure anywhere in the run), and the abort was
accepted with `--blocker tool --reason "Each queue batch needs a hand-written driver; 12 batches ran (25 of
78 candidates targeted, ...), 61 candidates remain and missed targets keep re-queueing. Stopped without
finishing the queue."` The agent itself wrote afterwards that "tool" was a stretch and time was not the
limit. `abort_rule::reason_is_not_a_blocker` (crates/dla-core/src/abort_rule.rs) let it through because the
text has the cue word "without" ("Stopped without finishing"). A second run aborted with "139 ranked
candidates not targeted (each harness batch settles 2-3)". A text check cannot tell a blocker from effort.

## Change

1. `dla run abort` (crates/dla-cli/src/run/abort.rs and crates/dla-core/src/abort_rule.rs), in the branch
   where budget is over the floor and work is open (the branch that already needs a blocker): the blockers
   `tool`, `build`, `environment` and `privilege` need `--evidence <id>` that names a failure this run
   recorded, checked by the CLI:
   - `H<n>`: a harness of the run whose latest attempt (`harness::AttemptOutcome`, `attempt.json`) is
     `Failed`, or whose latest run record has an exit with a non-zero code or a build failure in its
     limitations. Read the existing types; do not invent a new record.
   - a checklist item id (the run's `checklist_answer` records, e.g. `intake.workload`): its answer is
     unresolved or blocked with a reason that names something missing or refused (look at the existing
     reason values such as `env_unavailable`; list the accepted ones in a const with a doc comment).
   Refuse with a new code `blocker-not-recorded` (add it to crates/dla-core/src/error.rs beside the others)
   when `--evidence` is missing, names nothing of the run, or names a harness or item that recorded no
   failure. The refusal text says, in plain words: no command of this run failed for want of a <kind>
   (name what it checked, e.g. "the 16 harness attempts of this run all ran"); a queue that needs many
   batches is the work of the run, not a blocker; the queue's next command (the existing first command of
   the open work); and, when something really fails, run the command that needs it so the failure is
   recorded, then the abort with `--evidence`. The fix line must be a whole command that runs as printed
   once its placeholders are filled (the existing tests in abort.rs check every printed fix this way:
   extend them, keep them passing).
   - `user` is unchanged (no evidence). `refusal` keeps `--code` and needs no `--evidence`.
   - With the budget at its floor, or no open work, nothing changes: no blocker or evidence is needed.
   - Keep `reason_is_not_a_blocker` as a second check; remove the cue word "without" (it matched
     "stopped without finishing"); keep the others.
2. Everywhere the abort command is printed (the `budget-left` refusal, the Stop hook text in
   crates/dla-cli/src/gate_hooks.rs, `abort_rule::ABORT_BLOCKER_COMMAND`, the `run.rs` help text): show
   `--evidence <H<n>|checklist item>` for those four kinds. Grep for `ABORT_BLOCKER_COMMAND` and
   `--blocker` to find them all.
3. Skill text: `skills/detect-leak-anywhere/SKILL.md` (the Stop hook line) and
   `skills/detect-leak-anywhere/references/recovery.md` (the "Stop hook, three stop attempts" section):
   say that a long queue is the run's work (each batch is one driver), and that a blocker must be a failure
   the run recorded, named with `--evidence`. Keep `cargo test -p dla-cli --test skill_text` passing (it
   checks some sentences; if a checked sentence must change, change the test only to follow the new text,
   never to weaken what it checks).
4. `docs/VERIFY.md`: one row `AA1` after the Z4 row, 5 cells like the Z rows (claim | `corrected` | what
   changed | the check | limits). Use the facts above in general words (no project names).

## Tests

dla-cli integration tests near the existing abort tests (grep `budget-left` / `run abort` in
crates/dla-cli/tests): with budget over the floor and an open queue:
- `--blocker tool --reason "<the real reason quoted above>"` and no `--evidence` is refused
  `blocker-not-recorded` (the run stays active);
- `--evidence H1` where H1's attempt ran is refused; where H1's attempt `Failed` (or its run record has a
  non-zero exit) it is accepted;
- a checklist item with a missing-tool answer is accepted; an answered item is refused;
- `--blocker user` and `--blocker refusal --code <c>` are unchanged; at the floor no evidence is needed.
Unit tests for the evidence check in dla-core. Every printed fix runs as printed (extend the existing test).
Mutations for each. Run fmt, clippy -D warnings, `cargo test -p dla-core --lib`, `cargo test -p dla-cli
--bin dla`, the abort/hook integration targets you touch, `cargo test -p dla-cli --test skill_text`, and
`cargo run -q -p dla-cli -- schema --check` if a schema or code list changed.

Final message: files changed, what each change does, tests and mutations, command results, anything not done.
