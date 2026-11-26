# Builder R: abort is no free exit while budget is left, and an abort does not skip the verifier

Repository: /home/user/dla. Work in the working tree. Do NOT commit, push or switch branches.
Another builder (Q) works in the same tree at the same time on confirmation rules
(crates/dla-core/src/machine.rs, finding.rs, data/archetypes.toml, the ledger commands, a new
final-gate item in crates/dla-core/src/gate.rs and the report's Findings line). Do not edit
machine.rs, finding.rs or archetypes.toml. In gate.rs and the report code, add your own functions
and do not reformat or move Q's; re-read a file just before you write it. The tree also holds an
uncommitted one-line test fix in crates/dla-sample/tests/os.rs: leave it. Add one row per cause at
the end of docs/VERIFY.md (re-read it just before you write; Q adds rows too). Do not touch
harness/, bench/fixtures/ or any fixture README or answer key.

## What real runs showed (do not name the projects in the repository)

1. An unattended run on a Go library confirmed the library's documented leak and then ran
   `dla run abort` after 6 of 180 minutes, with the reason "unattended run: 4 leak hypotheses
   confirmed plus 2 SDK caches measured; 139 ranked candidates not targeted (each harness batch
   settles 2-3) ...". The run had a measurement queue with 139 targets and 174 minutes left.
2. An unattended run on a JVM library ran `dla run abort` after 76 of 180 minutes because it
   believed nothing it could do would decide its open items (that cause is fixed now). Its 35
   confirmed findings went without the verifier: the abort ended the run, and the verifier step
   (`dla bundle F`, one `dla:verifier` subagent, `dla ledger verify`) was never run for 32 of them.

The Stop hook holds a stop while the final gate fails, budget is over the floor and a queue or an
undecided growth is open (`held_item`/`held_note` in crates/dla-cli/src/gate_hooks.rs), and its
note offers `dla run abort --reason "<why>"` as the way to end the run on purpose. SKILL.md line
106 says the same. An unattended model takes it as the exit. `dla run abort`
(crates/dla-cli/src/run/abort.rs) accepts any reason at any time.

## The change

1. **Abort while budget is left needs a blocker.** `dla run abort` is refused with a new code
   `budget-left` while `gate::budget_over_floor` is Some for the run AND there is work the run can
   still do: an open queue (find how `held_item` decides "a queue is open"), an undecided growth
   (`grown_unmeasured`), or a final-gate item whose fix is a measurement. It passes with
   `--blocker <kind>` where kind is one of:
   - `user`: the user asked to stop (always accepted; the report says "aborted at the user's
     request");
   - `tool`, `build`, `environment`, `privilege`: a missing tool, a build that does not compile, an
     environment or permission the run cannot change; `--reason` must then name it (non-empty,
     and not only a count of work left);
   - `refusal`: a refusal of DLA the run cannot satisfy; `--code <refusal code>` is required. If
     the run keeps any record of the refusals it printed (find out: logs, state, ledger), require
     that the code is one of them; if it does not, accept the code and record it, and say so in
     your report.
   The refusal of a plain abort says how much budget is left, what work is open (the queue line
   and the count of undecided growths), and its fix is the next measurement: the queue's next
   batch command (`dla harness gen --next` or whatever the queue prints now) or the held item's
   first command. Then `or dla run abort --blocker <user|tool|build|environment|privilege|refusal>
   --reason "<what blocks the run>"`. With budget under the floor, nothing open, or a worker's
   abort (already refused), behave as today. The blocker kind and reason go into the state and
   the report's incomplete reason.
2. **The Stop hook and the skill text stop advertising abort as the exit.** In `held_note`
   replace the abort sentence with: the next measurement first, and "dla run abort --blocker
   <kind> --reason ..." only for a blocker the run cannot pass. Update SKILL.md line 106 (that one
   line only; keep SKILL.md under its size limit, find the test that pins it,
   crates/dla-cli/tests/skill_text.rs, and update the pin) and the paragraph of
   skills/detect-leak-anywhere/references/recovery.md that explains the held stop.
3. **An abort does not skip the verifier.** After `dla run abort` (any kind), the confirmed
   findings without a verification record must still be verifiable and must be asked for:
   - Check that `dla bundle F` and `dla ledger verify` work on an aborted run and that their
     records count in the report. `ENDED` in abort.rs says "nothing more can be recorded in it
     that counts": find what that means in the code, keep it true for measurements and
     transitions, and make verification the exception if it is not already.
   - The abort's output lists the confirmed findings with no verification record and the command
     to run for each (`dla bundle F`, then one `dla:verifier` subagent, then `dla ledger verify`).
   - The Stop hook, on an aborted run, holds the stop (within its existing STOP_LIMIT) while a
     confirmed finding has no verification record, with that fix. Find how the hook treats an
     aborted run today and say.
   - The report says, per finding, "not verified: the run was aborted before the verifier ran"
     when that is so, and the Findings line counts confirmed-unverified apart (find the existing
     wording for unverified and follow it).
4. **Bench.** dla-bench dynamic and its policy (bench/runner/src) must still pass. If anything
   there runs `dla run abort`, make it pass `--blocker` as fits.
5. **Docs.** stages.md or recovery.md: one sentence each for 1 and 3. VERIFY rows.

## Tests

- CLI (find the abort tests: crates/dla-cli/tests, `run abort`): with budget over the floor and an
  open queue, a plain abort is refused `budget-left` and the fix names the queue's next command;
  `--blocker user` passes; `--blocker tool` with an empty reason is refused; `--blocker refusal`
  without `--code` is refused; with budget under the floor a plain abort passes; with nothing open
  a plain abort passes.
- gate_hooks tests (crates/dla-cli/tests/gate_hooks.rs): the held note no longer offers a plain
  abort; it names the next measurement and the blocker form.
- An aborted run with two confirmed findings and one verification record: the abort output lists
  the other; `dla ledger verify` on it after the abort is accepted and counts; the Stop hook holds
  while one is unverified and passes when both are; the report marks the unverified one.
- skill_text pin updated to the new line 106.
- Mutation checks: drop the budget check (the refusal test fails); drop the post-abort verifier
  hold (its hook test fails).

## Checks to run

`cargo test -p dla-core --lib`, `cargo test -p dla-cli --bin dla`, the dla-cli test files you
touch plus `gate_hooks`, `skill_text`, `gate_flow`, `report` tests (whichever exist), `cargo test
-p dla-bench --lib`, then `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt
--all --check`. Then the dynamic acceptance on a sample that runs the whole flow to the end:
`./target/debug/dla-bench dynamic --dla-bin ./target/debug/dla --parallel 2 --only go- --json
<scratch file>` and the same with `--only python-` (Q runs the whole set; do not run it in
parallel with Q's full run: check `pgrep -af dla-bench` first and wait for it if one is running).

## Rules

Shared `target/` only (Q builds in it too; cargo waits on the lock, that is fine). Disk is tight:
before building check `df -h /`. Under 3 GB free, delete stale hash dirs older than 120 minutes
under the literal paths /home/user/dla/target/debug/build/dla-cli, .../dla-core, .../dla-workload
and .../dla-bench with `find <literal path> -mindepth 1 -maxdepth 1 -type d -mmin +120 -exec rm
-rf {} +`, and NEVER while a cargo test or a dla-bench run of yours or Q's is running. No
`rm -rf "$VAR/..."`. Never kill processes by name. Never skip or weaken a test. No project names,
attribution lines or model names in the repository. Scratch:
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u151/.

## Report (final message)

Files changed; the rule as implemented for each item (the exact conditions, codes, blocker kinds
and their checks, whether the run records refusals, the hook and abort texts as printed); tests
with what each proves; mutation checks; every command with pass/fail counts; dynamic results;
existing tests whose expectations changed and why; anything you are unsure of.
