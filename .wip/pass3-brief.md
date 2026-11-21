# Three small fixes after the first real run with the measurement queue (brief for one fixer)

Repo /home/user/dla, branch main. The working tree holds uncommitted, verified work of other
agents (the measurement queue, the Go capture axis; `git status`). Do NOT commit, push, stash,
reset or revert anything. Touch only what this job needs; never undo a change you did not
write; re-read a shared file right before each edit.
sp = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad
Your scratch dir: `<sp>/u32/` (create it; delete only it, by literal path; never
`rm -rf "$VAR/..."`). Another agent edits the Go harness and Go coverage code at the same time
(`harness/go/`, `crates/dla-workload/src/harness/cover.rs` and Go parts of `build.rs`/`run.rs`):
stay out of the Go-specific code there, keep the tree compiling at all times.

Background: `dla` now ranks the static candidates and makes the agent measure them in batches
(`dla ledger list`, `dla harness gen --lang <rt> --driver --next`, `dla trend` gives each
declared target a verdict: grew, changed, constant, not_loaded, executed, not_executed,
unknown; `dla ledger close-rest` closes what the queue does not hold back). Read
`crates/dla-core/src/queue.rs`, `crates/dla-cli/src/ledger_queue.rs`,
`crates/dla-cli/src/trend/verdicts.rs` and the queue parts of
`skills/detect-leak-anywhere/references/stages.md` and `recovery.md` first. A real run of the
plugin on a Python library (transcript summary: the agent installed the library's
dependencies with `pip install --target $TMPDIR/deps` and ran `PYTHONPATH=$TMPDIR/deps dla
harness run H1 --mode both`) showed the three defects below.

## 1. A reason must say what happened: measured candidates are not "not exercised"
The skill's rule (SKILL.md, "Before you stop"): `not_exercised` means nothing reached the
finding; `inconclusive` means it was measured and the measurement cannot decide (`unresolved`
only, and the ledger wants a measurement tied to the finding, else `measurement-required`).
`dla ledger close-rest` closes every candidate it may close with `not_exercised` (or
`time_budget` at the floor), also one whose best target verdict is `executed`, `changed` or
`constant` ("targeted in H2, its code ran and showed no growth"). That reason is false there.
Build:
- The ledger's notion of "a measurement tied to the finding" (`tied_measurement` in
  `crates/dla-core/src/machine.rs`, used by `supported_reason` and `check_inconclusive`) also
  accepts a target verdict of the finding (or of a member of its family) from a harness that
  DECLARED it a target, when the verdict is `constant`, `changed` or `executed`. The rule reads
  ledger evidence only, so the verdicts of the declared targets must be in the result of the
  evidence record that `dla trend` writes (find what it stores now; add the list of
  `{finding, verdict, declared}` there if it is missing; keep older records readable).
  `not_executed`, `not_loaded` and `unknown` are NOT measurements of the finding.
- `close-rest` picks the reason per candidate with that rule: a measured candidate becomes
  `unresolved` with `inconclusive` (also at the budget floor: the existing rule in recovery.md
  says a finding that was measured and left undecided is `inconclusive`, never `time_budget`);
  the others as now. Its output and its record count `inconclusive` beside `not_exercised` and
  `time_budget`. The remedy text for `unknown` must not claim the code ran ("whether its code
  ran is not known: no coverage data for this run").
- Every printed fix that names a reason for such a finding (status, gate, refusals: they use
  `supported_reason`) follows automatically; check one by test.
- The report: a finding closed this way shows its `Targeted: H2 executed` line as before.
Tests you SEE fail without the change (revert the hunk, run, restore): close-rest on a run
with one target of each verdict gives the right state and reason for each (extend
`crates/dla-cli/tests/queue.rs::verdicts_settle_candidates_and_close_rest_closes_the_rest` or
add a test beside it); `dla ledger resolve F --as unresolved --reason inconclusive` is accepted
for a declared target with verdict `constant` and still refused (`measurement-required`) for
one with `not_executed` and for a finding that only an automatic probe covered without
declaring it (extend `crates/dla-core/tests/inconclusive.rs`).

## 2. A harness is rerun in the environment its dependencies need
`dla harness run H` ran with `PYTHONPATH=<deps>` in the environment; a later
`dla alloc-test H --finding F` typed without it failed at import (the harness log showed
`ModuleNotFoundError`), and the agent had to find out why. Build: `dla harness run` records,
in the run record of the series (`run-<S>.json`), the values of the dependency-path variables
it ran with, for this fixed list only: `PYTHONPATH`, `NODE_PATH`, `CLASSPATH`, `GOFLAGS`,
`GOWORK`, `LD_LIBRARY_PATH`, `DYLD_LIBRARY_PATH`, `VIRTUAL_ENV`. Every command that runs the
harness's processes again (`dla harness run H` again, `dla alloc-test`, `dla intervene`, any
other caller of the same run machinery: find them all) sets, for each variable of the list
that is UNSET in its own environment and was recorded by the latest successful run of that
harness, the recorded value for the child processes, and says so in its output
(`env_reused: ["PYTHONPATH"]`, plus one line in the harness log). A variable that is set in the
current environment always wins. Never record any other variable (no tokens, no PATH).
Schema: regenerate if a serialized type changes. Tests seen failing: a Python driver harness
whose unit imports a module that is only on a `PYTHONPATH` directory: run with the variable,
then `dla alloc-test` (or a second `dla harness run`) without it succeeds and reports
`env_reused`; with a different value set, that value is used. Docs: one sentence in
`harness/README.md` and in the harness section of `references/stages.md` where the run is
described.

## 3. A site on the last line of a function belongs to that function
`crates/dla-cli/src/trend/candidates.rs::enclosing` asks tree-sitter for the node spanning
`(row, indent)` to `(row, usize::MAX / 2)`: a point past the end of the line lies outside a
function whose last line is that line, so the innermost function is missed and the parent (or
nothing) is returned. A Python method often ends on a real statement (`self._cache[key] =
value`), so a growing column whose site is that line is tied to no finding. Fix: end the range
at the line's real end (trailing whitespace and a trailing comment must not push it out
again; check what `crates/dla-cli/src/trend/verdicts.rs` does for its own enclosing-function
code and use ONE helper for both). Tests seen failing: a unit test with a Python function, a
Go function, a JS arrow function and a Java method whose last line is the site; an end-to-end
check through whatever public function uses `enclosing` (a column at the last line of a
function names the finding whose writer is in that function).

## Rules
- Never edit anything under bench/fixtures/ or bench/fixtures-open/, nor EXPECTED_FAILURES in
  bench/runner/src/dynamic_cmd.rs. Never skip, ignore or weaken a test.
- Cargo with `-j 2`, the shared `target/` only. For bench runs copy `target/debug/dla` and
  `target/debug/dla-bench` to `<sp>/u32/bin/` and use the copies
  (`DLA_PLUGIN_ROOT=/home/user/dla` when you call the copy of dla directly). Never kill
  processes by name.
- Disk: if `df --output=avail -B1M /` is below 2500, run `bash <sp>/prune-superseded.sh` and
  delete kept bench work directories of your own finished runs by literal path.
- docs/VERIFY.md: one new row per item at the end of the `## M7` table
  (`| Item | Status | Evidence | Resolution |`, exactly 4 cells, escape pipes as `\|`, never
  nest backticks in one code span, "corrected" only when its tests ran, name every test; no
  model names, no line numbers, no scratch paths, no plan or package names).
- SKILL.md stays within the limits that `cargo test -p dla-cli --test skill_text` checks.
- Before reporting: `cargo test -j 2 --no-fail-fast -p dla-core -p dla-cli -p dla-workload
  -p dla-report`, `cargo clippy --workspace --all-targets -j 2 -- -D warnings`,
  `cargo fmt --all --check` (format only your own files), `target/debug/dla schema --check`,
  `python3 bench/runner/test_grade.py`, `cargo test -j 2 -p dla-cli --test skill_text`,
  `dla-bench dynamic --parallel 2 --only python-` (all pass; baseline
  `<sp>/u23/dynamic.json`; a chance failure is traced, not skipped) and
  `dla-bench static` in both modes (64/64).
- No separate checker follows you: the orchestrator reruns your commands. Write the full report
  to `<sp>/u32/report.md` with the Bash tool (`cat > file <<'EOF'`), and return at most 50
  lines: each item's state, files changed, tests (which you saw fail), every command with its
  result, what is unverified.
