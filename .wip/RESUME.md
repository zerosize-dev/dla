# Where the work stands (2026-11-19, 23:10 UTC)

## Code state
- `main` (and `dev/wonderful-mccarthy-algc5s`) is at `9781542` "Version 0.9.0-preview.2", on top of
  `4498323` (the CI repair: Ubuntu 26.04 with JDK 25, Rust 1.100, macOS linked temp dirs; eight
  fixtures that do not pass yet moved unchanged to `bench/fixtures-open/`), `63a30f2` (the 14 open
  items of the budget, coverage-step and JVM non-heap work) and `d1fca56` (that work itself).
- CI is green on `main` at `9781542` (run 44608346577; its first attempt failed by chance on
  `jni-order-codec`, the re-run of that job passed).
- Release 0.9.0-preview.2 is published (tag `v0.9.0-preview.2` on `9781542`, pre-release, five
  archives with `.sha256` files and `SHA256SUMS`; release run 44631010965). Nothing to do here.

## In flight: the last fix pass (brief `fix-pass-9-brief.md`, items `fix-pass-9-open-items.json`)
- Two non-leak fixtures fail now and then with "no rule-out on a non-leak fixture"
  (go-gc-tuned-batch once on the CI runner, python-graph-churn once in a full local run). Cause:
  a column that passes the first trend by chance votes in the unbound alloc-test and reads
  plateau + purge retained = inconclusive. Decisions: the purge test uses the plateau test's
  minimum rise (growth at or below it is `no-growth`); `os.rss_file` joins the columns a purge
  cannot release in `crates/dla-cli/src/alloc/pick.rs`. `judge::combine` does not change.
- 14 items the package checks left open: the JVM warm-up test's assertions fail about 1 in 12
  under load, the JNI test's guard on JDK 25, the flight recorder's `JFR Recording Scheduler`
  thread, a link case of the JVM coverage agent, VERIFY rows (a stale count, titles, plan names).
- Known and not handled: under heavy load on the work machine, single failures of
  `sample::node_inspect_backend_reads_memory_and_active_resources`,
  `session::tests::a_backend_that_never_works_is_blocked_and_sampling_continues` (dla-sample) and
  one Go analyzer test of `dla-backends` `real_tools`; none failed in CI.
- State at 23:10 UTC: N2 fixed, checked (7 minor), refixed, second check running. N1 fixed and
  checked: 3 problems (with `os.rss_file` never voting, a real file-backed leak can read
  non-leak: keep its plateau vote and drop only its purge reading; the Python trim step reads
  `before` after its own `import ctypes`, which adds about 2 MB to `os.rss`, so a flat Python run
  still reads inconclusive through `os.rss`: fix in `harness/python/sitecustomize.py`; one false
  sentence in a VERIFY row) and 3 minor; its refix is running. Then the workflow goes on by
  itself: CI on a snapshot of this branch, full local acceptance.
- Steps: fix, independent check, snapshot to this branch, CI green on the snapshot, local
  acceptance, then commit to `main` and push.

## In flight beside it: the `jni-order-codec` chance failure (brief `fix-pass-10-brief.md`)
- Cause, measured on capture files a local run left behind: the fixture's `build.sh run {n}`
  starts the project's test JVM and then the program; both write a shim exit record with stacks
  into `harness/H1/out/p<k>-native/shim-dump.jsonl`, and the reader (`shim_run` in
  `crates/dla-cli/src/summarize/shim_groups.rs`) keeps "the one with most live bytes". At the
  first point the two totals were 41.9 MB (test JVM) and 47.8 MB (program); the test JVM's total
  moves by tens of MB between runs. When it is the larger, capture 1 is the wrong process and
  the leak's stack is "absent from dump 1".
- Decision: choose by identity, not size. The harness writes only the measured process's shim
  record (`shim_dump_commands` in `crates/dla-workload/src/harness/capture.rs`); the reader uses
  the rule of `select_measured` (`dumps.rs`) for other files and uses size only as a named
  fallback; `dla-bench dynamic` prints the failed gate's per-capture values. The gate in
  `summarize/rule.rs` and the fixture do not change.
- Runs as its own small workflow (fix, check, refix, check) in the same tree as the pass above;
  its changes go into the same commit when both are checked.

## Next, in order
1. Finish the two fix passes above and commit them to `main` together (CI green on a snapshot
   first, then the full local acceptance).
2. The eight fixtures in `bench/fixtures-open/` (move each back when it passes, and restore the
   `pair` line of its partner: go-ticker-modern, node-signal-compose-22, python-zstd-options-3145).
3. The benchmark work in `bench-todo.md`.
4. The remaining M7 phases (calibration, rightsize suite, model baseline); `1.0.0-rc.1` and
   `1.0.0`, each only with the owner's go-ahead.

## How to commit and release
- Author Mykyta Hriaznov <17167893+ngriaznov@users.noreply.github.com>, no trailers, no attribution.
  Push `main` and `main:dev/wonderful-mccarthy-algc5s`.
- A version bump touches `Cargo.toml` (workspace version and nine path dependencies), `Cargo.lock`
  (`cargo update -w --offline`), `.claude-plugin/plugin.json`, `.claude-plugin/marketplace.json`.
- The work machine has Rust 1.100 and JDK 25 at `/usr/lib/jvm/java-25-openjdk-amd64` (default
  `java` 21); a recycled container needs `rustup update stable` and
  `apt-get update && apt-get install openjdk-25-jdk-headless`, then the default set back to 21.

## Decisions that the open items must keep
1. Budget settling in a dispatched run: each worker's first passing gate at sweep or later
   re-estimates from every registered worker's slice plus the main ledger and may only raise
   the budget (deadlines shift forward); once every registered worker has settled, it is final.
2. Candidates are counted the same way in every mode (distinct-site, non-duplicate, top-level,
   any state) and called "candidates".
3. Between the floor and the deadline: while more than the floor is left, never ask whether to
   continue and never close `time_budget`. At or under the floor (or after a refusal with nothing
   smaller that fits, of the core's own command), the core of an interactive run asks the user
   once whether to extend, and closes `time_budget` only if the user declines or does not answer;
   a worker or an unattended run closes `time_budget` with its remedy. A worker's own refusal
   never moves the core to the floor.
4. Harness steps: a JUnit repeat harness has no hooks; when it is the only one that ran, the step
   is a hook driver (`dla harness gen --lang jvm --driver`, fill, run). With no harness, the step
   follows what `dla harness gen` would do (`--driver` when it needs a workload). A recorded
   blocker on intake.workload is named instead of a step. A harness whose build failed points at
   the build error or `build_fails`, but never hides a newer driver that built and ran.
5. JVM drift: the agent-drift override applies only when no required column grows (no
   `jvm.nmt.*` category, no thread count, no class count); a growing column is named and `next`
   points at the capture that decides it (threads for thread columns, NMT for `jvm.nmt.*`), never
   `--extra` for a thread leak. The drift line scales with the extra window (2x margin over the
   measured idle drift). Command-mode harnesses get no override; a JUnit harness keeps `--junit`.

## Working rules on this machine
- Cargo with `-j 2`, the shared `target/` only; if `df --output=avail -B1M /` is below 3000, run
  `prune-superseded.sh`. Never kill processes by name.
- Every behaviour change gets a test seen failing without it. docs/VERIFY.md rows: 4 cells, name
  every test, "corrected" only when its tests ran, no internal plan names, model names or stale
  byte counts.
- Unfinished work is snapshotted to `wip/progress` (this branch); `main` gets checked work only.
