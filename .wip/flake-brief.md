# The dynamic acceptance job fails about one run in three: "no rule-out on a non-leak fixture"

You are a builder on the `dla` tool (Rust workspace at /home/user/dla). `<sp>` below is
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad; your scratch
directory is `<sp>/u43/` (exists). This job is analysis first; change code only as far as the
evidence carries.

## Working rules (binding)
- No git commit, push, stash, reset, checkout of files. Two other builders edit the same tree
  now (the batch/queue code and Go harness: `crates/dla-core/src/queue.rs`,
  `crates/dla-workload/src/harness/`, `crates/dla-cli/src/harness_cmd.rs`, `trend_cmd.rs`
  texts; the Rust harness: Rust parts of dla-workload). Stay in the statistics and alloc-test
  code; exact replacements; re-read a file right before editing; never restore a file from an
  older copy. If the build is broken by someone's half-finished edit, wait a minute and retry.
- cargo always with `-j 2`; only the shared `target/`. Never kill processes by name: a
  background process you start is stopped by the PID you recorded. No `rm -rf` with a
  variable in the path: literal paths only. Keep the disk above 2 GB free.
- NEVER make a check pass by weakening it: no change to a fixture, to an answer key, to the
  pass criteria of `dla-bench dynamic`, to the list of expected failures, and no skipped test.
  The fix, if there is one, is in the measurement rules, with a reason that holds for real
  programs.
- Every behaviour change gets a test you have SEEN fail without the change.
- Nothing in the repository names a real open-source project, a model, a plan or an agent.
  docs/VERIFY.md rows at the end of `## M7` for behaviour changes.
- Report with the Bash tool: `cat > <sp>/u43/report.md <<'EOF' ... EOF`; final message = short
  summary + that path.

## Evidence
CI runs the job "dynamic acceptance on the fixtures" (`dla-bench dynamic` over the 64 bench
fixtures with a fixed mechanical policy; see `.github/workflows/ci.yml` and
`crates/dla-bench/`). Over the last 16 runs it failed 5 times; the same commit passed on one
branch and failed on another, and the last commit failed twice in a row on two different
fixtures. Logs of those two: `<sp>/u43/ci-fail-1.log` (fixture jvm-pool-warmup) and
`<sp>/u43/ci-fail-2.log` (fixture go-buffer-pool). Both say "no rule-out on a non-leak
fixture", with this shape:
- the first trend of the fixed command passes a WHOLE-PROCESS column as a `stair`:
  `os.rss stair slope 6913.120 noise 9000219; os.rss_anon stair slope 7043.591 noise 8806759;
  os.vm_hwm stair slope 8150.333 noise 5236844` (JVM fixture);
  `go.released stair slope 1737.697 noise 1018194` (Go fixture);
- the unbound alloc-test then reads `Inconclusive`: for the Go fixture
  `alloc-test E12: go.released inconclusive (plateau main 1738.000 extra -1300.000 rise
  -4004000.000 noise 1326000.000, purge retained); os.rss non-leak (...); os.vm_hwm non-leak
  (...)` and a second one `E13: go.released leak (growth-continues main 1738.000 extra
  1826.000 rise 5625000.000 noise 1236000.000, purge retained); os.rss non-leak; os.vm_hwm
  non-leak`;
- the policy leaves the growth finding unresolved, so the non-leak fixture has no rule-out.
Local sightings of the same failure under machine load: go-buffer-pool and go-gc-tuned-batch
(logs of a comparison loop may be in `<sp>/u38/`: `bench-gc*.log`, `gc-new-*.log`,
`gc-old-*.log`; read them if they exist, do not run anything in that directory).

## Questions to answer, with numbers
1. What exactly do the columns mean and should each be able to carry a leak verdict at all?
   `go.released` is memory the Go runtime has returned to the operating system: say whether
   growth of it can ever indicate a leak, and what the code does with it today (gate, tie,
   alloc-test basis). Same question for every other whole-process column that passed in these
   logs and in the local logs (`os.vm_data`, `go.total`, `os.vm_hwm`, `os.rss_anon`,
   `jvm.nmt.arena_chunk`, ...): which are high-water marks or virtual sizes that cannot fall,
   and how do the gate and the plateau test treat a column that cannot fall.
2. How does a `stair` whose whole rise is about the size of its noise pass the growth gate?
   Read the gate (`crates/dla-stats/`, `crates/dla-core/src/rule*`, wherever `stair` is
   decided) and compute, from the logs' numbers, the statistic it used. Is the noise figure
   in the log the one the gate compares against?
3. Why is the unbound alloc-test `Inconclusive` in each failing case: which column decided
   and by which branch (plateau verdict, purge verdict, the combination rule)? In E12 the
   extra slope is negative and the rise is minus three times the noise: what makes that
   inconclusive rather than a plateau?
4. Reproduce: run `dla-bench dynamic --only go-buffer-pool` and `--only jvm-pool-warmup`
   repeatedly with a binary you copy to `<sp>/u43/bin/` (from /home/user/dla, with
   `DLA_PLUGIN_ROOT=/home/user/dla`, `--log-dir` under `<sp>/u43/`), some runs on the machine
   as it is and some with extra load that you start and stop yourself by PID (for example two
   `sh -c 'while :; do :; done'` loops). At least 10 runs per fixture and condition if time
   allows; report the failure rate per condition and keep the logs of the failing runs.
   (`--only` takes one text; each call is one fixture or one prefix.)

## Change (only what the answers justify)
Candidates, in the order to consider them; take the ones the evidence supports and say why
the others were not taken:
a. A column that by its meaning cannot show a leak (memory returned to the OS) is recorded and
   shown, and is not a growth signal: it does not pass the gate as growth and is not a basis
   of the alloc-test.
b. A column that cannot fall (a high-water mark, a virtual size) reaching a plateau is a
   plateau: the plateau test must not need it to fall, and a `stair` in it is judged by its
   steps continuing through the extra rounds.
c. A negative extra slope with a rise below minus the noise is a decline, which favors
   non-leak.
d. The gate's stair rule requires what its documentation says it requires; if the code and
   the rule differ, fix the code.
For each change: unit tests with the numbers of the logs (seen failing), then the fixture
loops of question 4 again with the new binary (same conditions, same counts), then
`dla-bench dynamic --only go-` , `--only jvm-` , `--only native-` and `--only python-` once each
(leak fixtures must still be found: a change that turns a leak fixture's result is wrong;
report every fixture whose outcome fields differ from before).

## Acceptance
`cargo fmt --all --check`; `cargo clippy --workspace --all-targets -j 2 -- -D warnings` (if it
fails only in files of the other builders, name them and run `--no-deps -p` on your crates);
`cargo test -j 2 --no-fail-fast -p dla-stats -p dla-core -p dla-cli` (name the crates that
exist; `ls crates/`); `target/debug/dla schema --check`; `python3 bench/runner/test_grade.py`.

## Report
The answers to the four questions with numbers, the failure rates before and after per
fixture and condition, what changed (files, functions) and why each rule holds for real
programs, tests seen failing, fixtures whose outcome changed, what you decided not to change
and why, what is unverified. If the evidence does not support a rule change, say so and
propose what would decide it; do not change code to get a pass.
