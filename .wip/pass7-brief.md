# Pass 7: census speed, the stop hook during a running command, the stamp and the queue

Same agent as passes 3 to 6, same working rules (cargo -j 2, shared target/, no commits, no
stash/reset/checkout of files, never restore a file from a copy, never kill by name, literal
paths in rm, every behaviour change with a test SEEN failing; for a pure speed change an
equality check plus timings; VERIFY rows at the end of `## M7`; nothing in the repository
names a real project, a model, a plan or an agent). Scratch `<sp>/u45/` (exists); report with
the Bash tool to `<sp>/u45/report.md`.
`<sp>` = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad.

Two other builders edit the tree now: Rust harness (allocation stacks: Rust parts of
`crates/dla-workload/`, the native shim, summarize alloc-stack code) and the statistics and
alloc-test rules (`crates/dla-stats/`, `crates/dla-cli/src/alloc*`). Stay out of those.
Re-read a file right before each edit; exact replacements; if the build is broken by a
half-finished edit, wait a minute and retry. The machine is heavily loaded at times: give
wall AND CPU seconds, and rerun a timing-sensitive test alone before calling it a failure.

Evidence: `<sp>/wip-notes/real-run-plugin-grpc-3-axios-2.md` (read it first) and
`<sp>/u36/grpc-3/` (state.json, ledger, transcript of that run).

## Item 1: census speed
`dla census --lang go` on a repository of 1 083 Go files takes 2 min 8 s of CPU (3 min 33 s
wall on the loaded machine) with the current build; the caller's foreground limit is 120 s.
Reproduce on a copy: `cp -r <sp>/u18/grpc/proj <sp>/u45/g`, in it `dla run init --entry
repository --budget auto` with environment `DLA_CONSENT_TIERS=1-4 DLA_CONSENT_RAW_VALUES=1`
and `DLA_PLUGIN_ROOT=/home/user/dla`, then `dla census --lang go` (your built binary; never
run `dla consent`). Find where the time goes (sample the running process with `gdb -p <pid>
-batch -ex bt` a few times as you did for trend, or timing prints in a scratch build that
you remove again) and fix the cause without changing the census output: the candidates, the
annotations and the per-site facts must be identical (compare the census JSON and the ledger
without timestamps before and after; the facts code is `crates/dla-census/src/{sites,events,
usage}.rs`, `lib.rs`; likely suspects: a tree-sitter parse or a whole-file scan per holder or
per site instead of once per file, regexes compiled in a loop, the start-up reach walking
callers per site, name matching over all files per holder). Target: under 20 s of CPU for
that repository. Then time `dla census` for the other languages on the trees under
`<sp>/u18/` (aiohttp/proj python, spring: `/root/dla-real/src/spring-data-commons` jvm,
axios/proj node, tokio/proj rust) and `dla sweep` for each; report the table; fix anything
else above 30 s of CPU if the cause is of the same kind. Equality test: a unit or
integration test that the census of a small multi-file fixture is byte-identical to a stored
expectation is fine if one exists already; otherwise report the before/after comparison on
the real trees and add a focused unit test for whatever you memoised.

## Item 2: the stop hook while a dla command is running
The stop hook (`dla gate stop-hook`, `hooks/hooks.json`; code in `crates/dla-cli/src/` gate
or hook module, and `crates/dla-core/src/gate.rs`) counts stop attempts and, at the third
attempt with no ledger change, lets the stop through and stamps the run INCOMPLETE. In the
real run a long `dla census` had been moved to the background by the caller; the model
answered three times that it was waiting; the hook stamped the run while that command was
still running.
Wanted: when a `dla` command of this run is still running (find how a running command can be
known: `state.json` registers processes the tool started, see the `running` list of `dla run
status`; if the main `dla` process itself is not registered, register every `dla` command
that can take long (census, sweep, harness run, trend, alloc-test, report) for its lifetime
in the existing registry, with its pid and start time, and treat a registered pid that no
longer exists as not running), the stop hook blocks the stop with a message that names the
running command, since when, and says to wait for it; such an attempt does NOT count toward
the three and never stamps. Bound it: if the same command has been running longer than a
generous limit (say 30 minutes; put the constant next to the other hook limits), the hook
falls back to today's behaviour so that a hung command cannot block a stop forever. Tests:
hook with a live registered command (use a real sleeping child or the test hooks the
existing stop-hook tests use; look at them first), with a dead pid, past the limit.

## Item 3: an INCOMPLETE stamp must not close the queue of a run that goes on
`gate::budget_over_floor` returns None for a run whose status is Incomplete, so the
measurement queue reads `closed: floor` and `dla ledger close-rest` closes every candidate
with reason `time_budget` although most of the budget is left (in the real run: 45 of 60
minutes).
Wanted:
- When a state-changing `dla` command runs after a stop-hook stamp (the ledger gets a new
  record), the stamp is lifted: the run is going again, the lifting is recorded (a ledger or
  state record like the one the stamp wrote; the report still says the run was once stamped,
  as an entry in its history, not as its status). Find how the stamp is stored and what else
  reads it (`RunStatus::Incomplete`, the report's status line, `run status`'s first blocking
  item "run is incomplete: stop hook let the stop through...") and make them consistent. An
  `Aborted` run stays aborted.
- Independently of that, the reason a closed queue gives must be true: `floor` only when the
  time left is at or below the floor. A queue that is closed because the run is over (aborted
  or still stamped) says so with its own value, and `close-rest` then uses a reason that is
  true for it (not `time_budget`); pick from the existing reasons or stop and report if none
  fits.
Tests for each.

## Item 4: the auto budget settles on its own
`dla run init --budget auto` starts with 60 minutes and is meant to settle to
`clamp(20 + 15*R + 3*C, 60, 180)` minutes once the candidates are known. In the real run it
stayed unsettled (`settled: false`, 60 minutes) because the one command that settles it was
never reached. Find where it settles today and make every command that needs the budget
after `sweep` and `census` have run for a runtime settle it first (at least `harness gen`,
`run status`, `ledger close-rest`, `gate check`), exactly once, with the same formula and the
same record as today. Tests.

## Acceptance
`cargo fmt --all --check`; `cargo clippy --workspace --all-targets -j 2 -- -D warnings` (if it
fails only in files of the other builders, name them and run `--no-deps -p` on your crates);
`cargo test -j 2 --no-fail-fast -p dla-census -p dla-core -p dla-cli`;
`target/debug/dla schema --check`; `python3 bench/runner/test_grade.py`;
`dla-bench static` (default mode) with your binary (copy `target/debug/dla` and
`target/debug/dla-bench` to `<sp>/u45/bin/`; run from /home/user/dla with
`DLA_PLUGIN_ROOT=/home/user/dla`): 64 of 64 and the candidate counts per fixture unchanged
(compare its JSON with a run of the binary before your census change).
Report: per item what changed (files, functions), the timing table before and after, the
equality checks, tests and that you saw each fail, what you did not do and why, what is
unverified.
