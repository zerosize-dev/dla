# Where the work stands (paused 2026-11-13)

## Code state
- `d1fca56` on `main` (and `dev/wonderful-mccarthy-algc5s`) holds the whole round as work in
  progress; the round's base is `c65e617`, so `git diff c65e617 d1fca56` is the round.
- The round, from three real runs on Kotlin Spring Boot services:
  - auto time budget sized from runtimes and candidates, `dla run extend`, the `budget`
    object in `dla run status`, the report's Time line;
  - `time_budget` refused while more than max(10 min, 20%) is left, `inconclusive`, fix texts
    that follow the run state, the Findings line split into leaks and explanations;
  - JVM exercised-entry coverage, the coverage warning, the harness step, two final-gate items
    (coverage contradiction, budget left with runnable revisits);
  - JVM non-heap columns (NMT, threads), warm-up, the alloc-test drift rule;
  - Kotlin/Gradle build logic out of scope, CodeQL traced build for Kotlin, Sonar, the driver
    class path;
  - skill text: keep measuring while time is left, say first what was not tested, contents
    lists, a recovery table;
  - Windows verbatim paths; the Windows integration job verified;
  - 34 new dynamic fixtures, 8 of which failed the last full run (see `bench-baseline.md`).
- All of it went through build, independent checks and up to two fix rounds per part. The
  items still open are in `open-items.md` (14: 6 budget/time, 4 coverage steps, 4 JVM non-heap).
- CI on `d1fca56` is expected to be red in places: the new fixtures that fail, and a flaky JVM
  end-to-end test (`alloc_test_jvm::a_driver_that_keeps_daemon_threads_is_not_blamed_on_the_agents_drift`).

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

## Next, in order
1. Fix the items in `open-items.md` (fix, independent check, refix up to twice).
2. Integration check across all changes: shared files hold every part's change; the texts agree
   (one instruction between floor and deadline across SKILL.md, stages.md, recovery.md, the
   Stop hook and the fixes; one harness step in gate fix, Stop hook, report warning and docs).
3. Full acceptance: `cargo test -j 2 --no-fail-fast --workspace` (all test binaries),
   `cargo clippy --workspace --all-targets -j 2 -- -D warnings`, the same for
   `--target x86_64-pc-windows-gnu -p dla-core -p dla-probe -p dla-report`, `cargo fmt --all --check`,
   `target/debug/dla schema --check`, `python3 bench/runner/test_grade.py`, `skill_text`,
   `dla-bench static` with archetypes on and off, `dla-bench dynamic --parallel 2` on all fixtures
   compared with `bench-baseline.md`.
4. Commit to `main` (author Mykyta Hriaznov <17167893+ngriaznov@users.noreply.github.com>, no
   trailers, no attribution), push `main` and `main:dev/wonderful-mccarthy-algc5s`; CI green.
5. Bump to `0.9.0-preview.2` (Cargo.toml and its path dependencies, Cargo.lock,
   `.claude-plugin/plugin.json`, the marketplace manifest), then dispatch `release.yml` on
   `main` with `publish=true`.
6. Later: the 8 failing new fixtures; the benchmark work in `bench-todo.md`; the remaining M7
   phases (calibration, rightsize suite, model baseline); `1.0.0-rc.1` and `1.0.0`, each only
   with the owner's go-ahead.

## Working rules on this machine
- Cargo with `-j 2`, the shared `target/` only; if `df --output=avail -B1M /` is below 3000, run
  `prune-superseded.sh`. Never kill processes by name.
- Every behaviour change gets a test seen failing without it. docs/VERIFY.md rows: 4 cells, name
  every test, "corrected" only when its tests ran, no internal plan names, model names or stale
  byte counts.
- Unfinished work is snapshotted to `wip/progress` (this branch); `main` gets checked work only.
