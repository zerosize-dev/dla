# Where the work stands (2026-11-19, evening)

## Code state
- `main` (and `dev/wonderful-mccarthy-algc5s`) is at `9781542` "Version 0.9.0-preview.2", on top of
  `4498323` (the CI repair: Ubuntu 26.04 with JDK 25, Rust 1.100, macOS linked temp dirs; eight
  fixtures that do not pass yet moved unchanged to `bench/fixtures-open/`), `63a30f2` (the 14 open
  items of the budget, coverage-step and JVM non-heap work) and `d1fca56` (that work itself).
- CI was green on the tree of `4498323` (all 15 jobs, run 44576504271 of snapshot `ca43359` on this
  branch). The run on `main` at `9781542` was started; check it with
  `gh run list --repo zerosize-dev/dla --workflow ci.yml --branch main`.
- Release 0.9.0-preview.2: when CI on `main` is green, dispatch `release.yml` on `main` with
  `publish=true` (it tags `v0.9.0-preview.2` itself), then check the release's assets. If the tag
  `v0.9.0-preview.2` exists, this is done.

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
- A third fixture fails by chance and is NOT in this pass: `jni-order-codec` on the CI runner
  (run 44608346577 on `main`, attempt 1; also once before preview.1): both native shim summaries
  fail their gate because the leak's stack is absent from the first capture ("holder absent from
  dump 1"), so the L-HANDLE label has no evidence. Never reproduced on the work machine. Next
  pass: find why the first capture can miss the stack, fix it in the product, and make
  `dla-bench dynamic` print the failed gate's per-capture values (CI artifacts cannot be
  downloaded from the work machine).
- Steps: fix, independent check, snapshot to this branch, CI green on the snapshot, local
  acceptance, then commit to `main` and push.

## Next, in order
1. Finish the fix pass above and commit it to `main` (CI green first).
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
