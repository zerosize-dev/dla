# Builder Q4: three defects a third review found in the record-order and abort fixes

Repository: /home/user/dla. Work in the working tree. Do NOT commit, push or switch branches.
The tree holds uncommitted work of earlier builders (the abort rule, the release rules, the
record-order key `RecordOrder { ts, at }` in crates/dla-core/src/ledger.rs with millisecond line
stamps `line_ts()`, the worker-gate fixes in gate.rs, the Python `unavailable` lines file, the
abort command constants `ABORT_BLOCKER_COMMAND` / `ABORT_REFUSAL_COMMAND` in
crates/dla-core/src/abort_rule.rs). Build on it; do not revert it. Do not touch harness/,
bench/fixtures/, any fixture README or answer key, or skills/detect-leak-anywhere/SKILL.md. Read
each file just before you edit it.

A timing-sensitive `dla-bench dynamic` run of mine may still be going. Before your first cargo
command, check `pgrep -af "dla-bench dynamic"`; while one runs, read and edit only, and wait for
it with a background until-loop (no foreground sleep). Do not run dla-bench yourself: CI runs the
dynamic set.

## The three defects (each traced by a reviewer and a skeptic)

1. **A time refusal written in the same second as a finding no longer counts.** Ledger lines are
   now stamped to the millisecond (`line_ts`), but `note_time_refusal`
   (crates/dla-core/src/toolctx.rs ~456) stamps the time-refusals file with `now_utc()`
   (crates/dla-core/src/state.rs, whole seconds, so always `.000`). `refused_since` (~94-130)
   tests `ts >= since` with `since` = the finding's line `ts` (callers: gate.rs ~3197 and
   crates/dla-cli/src/ledger_cmd.rs ~863, through `finding_added_at`). A finding added at
   12:00:00.400 and a refusal at 12:00:00.900 (stamped 12:00:00.000) now compares as "refusal
   before the finding": `dla ledger resolve F --as unresolved --reason time_budget` is refused
   `time-left` and the gate lists the finding as closed early. Before the millisecond stamps both
   were whole seconds and it counted.
   Fix: compare at the refusal file's own precision. Truncate `since` to the whole second in
   `refused_since` (that restores the old rule exactly; keep the extension rule
   "a refusal at the very instant of the extension predates it" as it is), with a comment that
   says why. Do not change the refusal file's format.
   Also: `finding_added_at` and `open_item_added_at` (ledger.rs ~2835) search `self.lines`, which
   in a worker's view holds only the slice's lines. Make `finding_added_at` read the index key
   (`finding_order(id).map(|k| k.ts)` gives the same line `ts` in every view); check the callers of
   `open_item_added_at` and do the same only if one runs in a worker view (say which).

2. **A wall clock that steps backward inverts two records of one writer.** `RecordOrder` sorts by
   `ts` first, and `line_ts()` reads `Utc::now()` with no floor, so after an NTP step or a VM
   resume a later record of the same ledger sorts before an earlier one (newest alloc-test,
   `drivers_of`, `explained_since`, gate helpers all read the key).
   Fix: a fresh line's `ts` never goes below the newest `ts` the ledger has committed: keep the
   latest committed `ts` in the index (updated in `commit`), and stamp a fresh line with
   `max(line_ts(), latest)`; within one batch of `append` (and the trial commit in
   `append_all_held`) each fresh line is at least the one before it. Lines given an explicit `ts`
   (`p.ts` Some: merged slice lines keep their worker's stamp) are not changed. Make the clamp a
   small pure function so it can be tested without moving the clock.

3. **The fix printed for an unknown `--blocker` kind does not run as printed.**
   crates/dla-core/src/abort_rule.rs ~133 returns
   `dla run abort --blocker tool|build|environment|privilege --reason "<what blocks the run>"`:
   the choice has no `<...>` (a shell reads `|` as a pipe) and it leaves out `user` and
   `refusal`. Make it print `ABORT_BLOCKER_COMMAND` and `ABORT_REFUSAL_COMMAND` joined by
   `   or   ` (the separator the `budget-left` fix uses), and check every other fix
   `check_blocker` returns the same way. The existing test
   `every_fix_of_a_failed_blocker_check_runs_as_printed` (crates/dla-cli/src/run/abort.rs) takes
   the first alternative of any word with `|`, so it passed with the bare choice: make it require
   that every `a|b|c` choice sits inside `<...>`, that it tries every alternative (not only the
   first), and that the kinds offered by the unknown-kind fix cover every entry of
   `BLOCKER_KINDS`.

## Tests (each must fail with its fix taken out: check it and report)

- Same-second refusal: a finding added, then a time refusal stamped as `note_time_refusal` stamps
  it in the same second (write the line the way the code does, with the finding's second and
  `.000`); `time_refused_since` / `nothing_fits_since` with `since` = the finding's line `ts`
  says true, and the CLI path (`dla ledger resolve ... --reason time_budget` after a refusal, find
  the existing tests of `time-left` in crates/dla-cli/tests) is accepted. A refusal one second
  before the finding still does not count.
- Clamp: the pure function returns the floor when the clock is behind it and the clock otherwise;
  and a ledger whose last line carries a `ts` an hour in the future (write the fixture line by
  hand) gets a next fresh line stamped no earlier than it, ordered after it by `evidence_order`.
- Abort: the unknown-kind fix parses with the real clap `Cli` for every alternative and passes
  `check_blocker`; the stricter test fails on the old bare `tool|build|...` text.
- Worker view: `finding_added_at` of a main-ledger finding in a worker's view returns its `ts`
  (only if you changed it).

## Checks to run (once no dla-bench dynamic runs)

`cargo test -p dla-core`, `cargo test -p dla-cli --no-fail-fast` (the whole suite: one earlier
round missed a failing test by running only some files), `cargo test -p dla-workload
--no-fail-fast`, `cargo test -p dla-report`, `cargo test -p dla-bench --lib`, then
`cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all --check`. Run the
tests AFTER your last edit: a test result from before an edit proves nothing about the edited
text.

## Rules

Shared `target/` only. Check `df -h /` before building (about 3 GB free); do not delete build
directories; if a build fails for lack of space, stop and report it. No `rm -rf "$VAR/..."`.
Never kill processes by name. Never skip or weaken a test. No project names, attribution lines or
model names in the repository. Update the docs/VERIFY.md record-order row (the last one) for the
clamp and the same-second rule (re-read the file just before you write). Scratch:
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u157/.

## Report (final message)

Files changed; for each defect the fix as implemented (the exact comparison now used, the clamp
function and every append path it covers, the printed unknown-kind fix verbatim); which callers
of `open_item_added_at` you checked; tests with what each proves and its mutation check (seen
failing with the fix taken out); every command with pass/fail counts, run after the last edit;
existing tests whose expectations changed and why; anything you are unsure of.
