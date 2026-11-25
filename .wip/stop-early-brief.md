# Builder K: a run with budget and an open queue does not stop by giving up three times

Repository: /home/user/dla. Work in the working tree. Do NOT commit, push or switch branches.
The tree has uncommitted, checked work from other builders that is in CI now. Do not revert or
reformat files you did not write. Add one row at the end of docs/VERIFY.md (re-read it first).

## What four real runs showed (do not name the projects in the repository)

Four unattended runs on real libraries (Node, Rust, Python, Go), each with a 3-hour budget,
ended after 47, 3, 39 and 7 minutes. Each time the queue still had ranked candidates the run
had never measured: 75, 0 (closed), 5 and 139. In the Go run, the documented leak's holder was
candidate 60 of 140, and at its pace the run would have reached it within about 15 minutes.

The model tried to stop again and again. The stop hook blocked each try with
"172 of 180 budget minutes remain: measure what is open instead of stopping". The model answered
"I'm stopping, so the stop hook will probably block again" and stopped three times in a row with
no ledger change. The hook then lets the stop through (crates/dla-cli/src/gate_hooks.rs,
`stop_core`, `STOP_LIMIT = 3`) and stamps the run INCOMPLETE.

SKILL.md even teaches the way out (skills/detect-leak-anywhere/SKILL.md, under "Before you stop"):
"A Stop hook may block your stop while `gate check --final` fails. Do not rely on it: it lets the
third attempt go if the ledger did not change, stamping the run INCOMPLETE."

## The change

1. **The hook.** In `stop_core`, and in `stop_worker` for a worker's slice, while the budget is
   over its floor (`gate::budget_over_floor(state, worker)`), nothing was refused for time with
   nothing smaller that fits (`dla_core::toolctx::nothing_fits_since`, as in `budget_note`),
   and the final gate's missing items include an open `queue` (kind `queue`), a stop attempt is
   not counted toward `STOP_LIMIT`. The hook blocks without stamping. Every other case keeps
   today's rule: at or under the floor, nothing fits, or no open queue (only items that need a
   close or a resolve). The block reason says it in one short line, after the budget line:
   "This stop is not counted: <N> of <T> budget minutes remain and the <rt> queue is open. Run
   its next batch (<command>). To end the run on purpose, dla run abort --reason "<why>"
   (the report says ABORTED and why)". Use the queue's own command from the missing item's fix.
   Keep the total under `MAX_REASON`.
   Check: `dla run abort --reason` exists (crates/dla-cli/src/run/abort.rs) and the report shows
   the reason. Check that a run whose queue closed for a blocker (`intake.workload` resolved
   unresolved) is not held, because then no queue is open.
2. **SKILL.md.** Replace the sentence above with one that is true and no longer advertises an
   escape, for example: "The Stop hook blocks your stop while `gate check --final` fails; while
   budget is over the floor and a queue is open, it never lets the stop through: run the batch,
   or end the run on purpose with `dla run abort --reason "<why>"`." The skill's byte budget is
   pinned in crates/dla-cli/tests/skill_text.rs. The new text must keep SKILL.md within that
   budget and the "Before you stop" offset under its limit. Shorten other words in the same
   bullet if you must, without changing their meaning, and run that test.
3. **Report.** If the run ends INCOMPLETE while budget was left, the report's Time line should
   already say so. Check what it says and leave it if it is plain.

## Tests (crates/dla-cli/tests/gate_hooks.rs has the stop-hook tests: see the ones that assert `stop_blocks`)

1. Budget over the floor and an open queue: four stop attempts with no ledger change are all
   blocked, `stop_blocks` stays 0 (or is not counted), the run is not stamped, and the reason
   names the queue command and `dla run abort --reason`.
2. The same with the budget at or under the floor (set the deadline near): the third attempt is
   let through, as today.
3. Budget left, but no open queue (only findings to close): the third attempt is let through,
   as today.
4. Budget left and an open queue, but a command was refused for time with nothing smaller: as
   today.
5. A worker's slice gets the same rule (if `stop_worker` has its own counter).
6. `dla run abort --reason "x"` while held: the stop is then allowed, because the run is no
   longer active.

Mutation check: make the new condition always false, and see test 1 fail.

## Rules

Shared `target/` only. Run `cargo test -p dla-cli --test gate_hooks --test skill_text`, the
dla-core tests if you touch dla-core, `cargo clippy --workspace --all-targets -- -D warnings`
and `cargo fmt --all --check`. If `df -h /` shows less than 3 GB free, delete stale hash dirs
older than 120 minutes under the literal paths /home/user/dla/target/debug/build/dla-cli and
.../dla-core with `find <literal path> -mindepth 1 -maxdepth 1 -type d -mmin +120 -exec rm -rf {} +`.
No `rm -rf "$VAR/..."`. No benchmarks. Never skip or weaken a test. If an existing test expected
the old let-through with budget left and a queue open, change it only as far as the new rule
requires, and say which and why. No project names, attribution lines or model names in the
repository.

## Report (final message)

List the files changed. Give the rule as implemented (conditions and texts), the SKILL.md before
and after with the byte counts, the tests with what each proves, the mutation check, the commands
with pass or fail counts, existing tests whose expectations changed and why, and anything you
are unsure of.
