# Where the work stands (2026-11-20, 05:50 UTC)

## Code state
- `main` (and `dev/wonderful-mccarthy-algc5s`) is at `9781542` "Version 0.9.0-preview.2", on top of
  `4498323` (the CI repair: Ubuntu 26.04 with JDK 25, Rust 1.100, macOS linked temp dirs; eight
  fixtures that do not pass yet moved unchanged to `bench/fixtures-open/`), `63a30f2` (the 14 open
  items of the budget, coverage-step and JVM non-heap work) and `d1fca56` (that work itself).
- CI is green on `main` at `9781542` (run 44608346577; its first attempt failed by chance on
  `jni-order-codec`, the re-run of that job passed).
- Release 0.9.0-preview.2 is published (tag `v0.9.0-preview.2` on `9781542`, pre-release, five
  archives with `.sha256` files and `SHA256SUMS`; release run 44631010965). Nothing to do here.

## In flight: two fix passes in one tree, one commit at the end
The container restarts about every three hours (00:17 and 03:25 UTC on 2026-11-20); files on disk
survive, running agents do not. Reports and checks of every finished agent are the `cont-*` files
here; the briefs are `fix-pass-9-brief.md` and `fix-pass-10-brief.md`.

State of the code (all edits are in the tree and in the snapshot `6e7db3b`):
- Pass A, package N1 (flaky non-leak verdicts in the unbound alloc-test): fixed, checked three
  times, refixed twice. The purge test uses the plateau test's minimum rise; a scatter-scaled
  limit was tried and left out (a slow real leak read non-leak with it); the file-backed column
  keeps its plateau vote and only its purge reading is exempt; the Python trim purge no longer
  reads `before` after its own `import ctypes`. Open by decision and written in the VERIFY rows:
  a flat Python run can still read inconclusive through `os.rss` now and then.
- Pass A, package N2 (14 items earlier checks left open): fixed, checked twice.
- Pass B (`jni-order-codec` failed by chance on the runner because the shim summary read the
  record of the process with the most live bytes, and the fixture's test JVM can hold more than
  the program at the first point): the harness writes the measured process's shim record only,
  the reader chooses by identity and uses size only as a named fallback, `dla-bench dynamic`
  prints the failed gate's limitation lines and per-capture values. Fixed, checked three times,
  refixed twice.
- Ten leftover items of the last checks (`cont-leftover-items.json`; one problem: a VERIFY row of
  N1 claimed more fixture runs than were made) were edited by a fixer that the 03:25 restart cut
  off before its report. Its 13 fixture runs passed and the tests of dla-cli, dla-workload,
  dla-parsers and dla-sample passed; `cargo fmt --all --check` and workspace clippy are clean.

State at 05:50 UTC:
- CI on snapshot `6e7db3b` (the tree with the ten leftover edits): green on all 15 jobs.
- Independent check of the leftover edits (`cont-c3-check-leftovers.json`): no problem, six
  minor items. Five are done by hand (wording in VERIFY rows, the README bullet and one comment
  re-wrapped; "the checker" reworded to "an independent check" in three rows).
- Acceptance on the tree of `6e7db3b` (reports `cont-c3-accept-fixtures.md` and
  `cont-c3-accept-tests.md`): static 60/60 in both modes; dynamic 59 pass and 1 expected failure,
  no failure (python-graph-churn passes; jni-order-codec reads the program's record at all
  nine captures); workspace tests 2744 passed, 0 failed on JDK 21; the JVM-dependent tests pass
  on JDK 25 (four Kotlin tests skip there with the reason printed); clippy (host and Windows
  target), fmt, schema check, grader tests and skill_text pass; the scan of the added lines for
  names that must not be in the repository is clean.
- The sixth item is done and independently checked (no problem): in
  `bench/runner/src/dynamic/ops.rs`, `failed_summary_detail` prints the reasons lines of every
  failed summary first and does not count them against the eight-line cap; the new test fails
  on the old function body. On the final tree `cargo test -p dla-bench`, workspace clippy,
  `cargo fmt --all --check` and `dla schema --check` pass.
- The tree is final. A snapshot without `[skip ci]` was pushed for the last CI run (find it with
  `gh run list --workflow ci.yml --branch wip/progress`).
When that run is green: one commit on `main` with the message in `commit-message.txt` (the
working tree as it is, without `.wip/`; author as below, no trailers), push `main` and
`main:dev/wonderful-mccarthy-algc5s`, confirm CI on `main`. If the working tree was lost, take
the tree of the snapshot: `git checkout <snapshot> -- .` on `main`, then remove `.wip/`.
Residual chance paths seen in the acceptance run, not handled (they cost nothing on the
fixtures where they showed): a heap column passes the first trend by chance on a flat Go run,
`os.rss` then votes and reads retained on a growth inside its own noise (go-ticker-modern, a
decoy-control); under heavy load `os.rss` of jvm-invoice-render-trimmed passed the first trend
and both unbound tests read leak (decoy-control, no claim made).

Snapshots: `snapshot.sh "<label>"` (copy here). A label with `[skip ci]` saves notes without a
CI run; a later snapshot without it always gets a commit, so a CI run exists for it.

Known and not handled: under heavy load on the work machine, single failures of
`sample::node_inspect_backend_reads_memory_and_active_resources`,
`session::tests::a_backend_that_never_works_is_blocked_and_sampling_continues` (dla-sample) and
one Go analyzer test of `dla-backends` `real_tools`; none failed in CI.

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
