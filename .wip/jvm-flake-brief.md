# Task 99: the JVM resident-memory flake (unbound alloc-test reads inconclusive under load)

sp = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad. Scratch: `$sp/u167/`.
Report to `$sp/u167/report.md`.

## The failure
`crates/dla-cli/tests/alloc_test_jvm.rs` (~line 289),
`an_unbound_alloc_test_of_a_command_that_runs_the_projects_workload_can_refute_a_finding`, expects
`favors: non-leak`; under CPU load (CI ubuntu clippy+test job, local full suite) it reads
`inconclusive`. The full answer of one failure is in `$sp/u165/jvm-flake-289.txt`: `os.rss` and
`os.rss_anon` favor leak with plateau inconclusive, while `jvm.nmt_committed`, `os.rss_file`,
`os.vm_data`, `os.vm_hwm` read plateau / non-leak; `decided_by: os.rss`. The fixture's cache is
bounded (cap 5000). The dynamic bench fixture `jvm-pool-warmup` fails the same way now and then
(on CI and locally). It is a precision flake: never a false leak, but it costs CI rounds and, on a
real library, an inconclusive where a non-leak was due.

## What to find (general, not this test)
Why a bounded JVM workload's resident memory still rises over the alloc-test window when the
machine is loaded, and what the tool should read instead. Hypotheses to test, not conclusions:
1. First touch: the JVM commits heap (and metaspace / code cache) up front; RSS rises as pages of
   already committed memory are touched for the first time, so RSS climbs toward the committed
   size while `jvm.nmt_committed` is flat. Under load the warm-up is slower, so the climb overlaps
   the measured window. If so: either the harness's JVM flags (pre-touch, or `-Xms` equal to
   `-Xmx`) remove the climb, or the rule reads RSS growth that stays under a flat committed size
   as not a leak (bounded by committed). Say which is general and cheaper; check the JVM flags the
   harness already passes (`crates/dla-workload/src/harness/`, the JVM agent and launcher).
2. The window: the alloc-test's points under load cover less of the warm-up than unloaded.
3. Something else the data shows.

## How
- Reproduce first: run that test alone in a loop with and without CPU load (e.g. a parallel
  `cargo build` of another crate is NOT allowed in the shared target; use a busy loop like
  `for i in $(seq $(nproc)); do (while :; do :; done) & done` and kill those pids by pid after).
  Record the failure rate before (e.g. 20 runs loaded) and keep each failing answer.
- Fix it in the tool, not in the test. Never weaken the test's assertion. A real JVM leak must
  still read leak: the leak fixtures of the dynamic set (JVM ones) and the alloc-test leak tests
  must still pass.
- After: the same 20 loaded runs, the JVM alloc-test integration tests, the dla-core/dla-stats
  tests you touch, fmt, clippy -D warnings, then the dynamic set once
  (`./target/debug/dla-bench dynamic --dla-bin ./target/debug/dla --parallel 2 --json $sp/u167/dyn.json`;
  expected 63 pass plus 1 xfail rust-tree-parent-links; run jvm-pool-warmup alone 5 times loaded).
  Delete dynamic logs afterwards.
- Mutation check for each new test (unique marker, restore, `grep -c` = 0).

## Rules
No commit, no push, no branch switch. `target/` is shared; free disk with
`cargo clean -p dla-cli -p dla-core -p dla-workload -p dla-bench` only. Literal rm paths. Kill only
your own processes by pid. If docs/VERIFY.md or a skill reference states the old reading, update it.

Report: the cause with the data that shows it, the fix, files, tests and mutations, failure rate
before and after, dynamic result, anything unsure.
