# Where the work stands (2026-11-20, 22:30 UTC)

## Working rule from the owner (2026-11-20, 09:25 UTC), above every other habit
"You should be using sonnet 5.5 subagents for tasks and verify yourself, since my limits are
going up too fast." So: every subagent runs on Sonnet (`model: 'sonnet'` in workflow scripts),
no checker, reviewer or design agent on the session model, and as few agents as the work needs.
The orchestrator verifies each agent's report itself: read the diff of its files, rerun its
tests and bench commands, keep the output short. A second agent on the same question only when
the orchestrator cannot settle it.

## Top priority from the owner (2026-11-20, 15:40 UTC)
"I'm not seeing improvements. Skill cannot find leaks in any popular projects you listed. I
don't think this is statistically possible." The owner ran the skill on the popular projects
proposed on 2026-11-19 (known leaks at pinned versions: aiohttp 3.10.10, grpc-go v1.84.0, axios
1.15.1, tokio-util 0.7.1, spring-data-commons 4.1.1, Jetty 12.0.31, Pillow 10.1.0; whether the
pinned versions were used was asked twice and not answered) and it found nothing. Benchmark
fixture work stops after the commit of fix pass 1. The work now:
Measured so far (2026-11-20, 17:45 UTC):
- Static stage on the seven leaking versions (trees and logs under the scratch dir `u18/`):
  the leak's site is among the candidates in 6 of 7 (not Pillow: a C reference-count leak, no
  rule), but real projects give 600 to 1000 candidates in discovery order, so the site sits at
  position 170 to 300 (aiohttp 8 of 922 with the right message; grpc-go 300 of 997;
  spring-data-commons 171 of 623; Jetty 170 of 199; axios 26 of 57; tokio-util 4 of 16).
- A full run of the skill on aiohttp 3.10.10 by a Sonnet agent with the binary of `main`
  (no hooks, no worker dispatch; log `real-run-aiohttp-log.md`, report
  `real-run-aiohttp-report.md`): it FOUND the documented leak (`_build_middlewares`
  `lru_cache(None)`, `aiohttp/web_app.py:82`, confirmed and strong by intervention) in 69
  minutes, plus two more unbounded caches. So the skill can find a real leak; why the owner's
  runs find nothing is still open (asked again which tag was checked out and what the report's
  Status and Findings lines said).
- Defects that run exposed: vendored `vendor/llhttp` and Cython-generated C gave 733 of 916
  candidates, made the probe report three runtimes and sized the budget to 3 h; the driver's
  module probes imported an example script that starts a server and the run hung 580 s without
  a message; one site became two findings (rule and census); no command lists the ledger; each
  follow-up measurement re-opens gate columns (about 10 minutes of busywork); hundreds of
  untested static candidates need one `ledger resolve` call each; the flow never separates "a
  cache unbounded in its key space" from a leak.
State at 22:30 UTC on 2026-11-20: real-project pass 1 is written (`real-project-report-X1.md`,
`real-project-report-X2.md`): vendored directories and generated files are out of scope by
default (`dla run init --include <path>`), Python module probes never import, a killed Python
point names the phase, a rule hit and a census entry at one site are one finding. Verified by
the orchestrator on aiohttp 3.10.10: python only, 187 candidates (922 before), static stage
38 s. The first full dynamic run then showed go-ticker-legacy failing steadily: the join gave
the ticker rule's finding the census entry's holder and region (blocked-stack, non-heap), so
the bound alloc-test read the goroutine count. Corrected: the Go ticker rules and the Node
timer rule declare registration-table and heap, and what a rule declares wins over the census
on a join. A second defect showed in the fixer's reruns and is on `main` since `fbcd6b2`:
`region_primary` preferred `go.heap_live.in_run` whenever the trend passed it, and the noisy
in-run reading made go-ticker-legacy fail about 2 runs in 8; corrected so the exit column
leads when the trend passed it (8 of 8 passes after, go-header-index 3 of 3 by the fixer).
Running: the orchestrator's verification of the corrected tree (workspace tests, lints,
static both modes, full dynamic; results under the scratch dir `u23/`) and CI on snapshot
`d2264d3`. When both are green: one commit on `main` with `commit-message.txt`, push `main`
and the dev branch, then bump to 0.9.0-preview.3 and release (`release.yml`, `publish=true`).
CI on `main` at `fbcd6b2` is green after a re-run: its first attempt failed by chance in the
macOS test job (`a_wrapper_script_that_runs_javac_and_tests_still_yields_the_main_jvm_record`,
`ratio(os.rss) < 4.0`, no values printed); the cause is not found yet (task list).
Next after the release: ordering of candidates and a command that lists them, bulk
dispositions for untested static candidates, the gate treadmill after follow-up measurements,
the keys discriminator in the flow, the macOS test.

## Code state
- `main` (and `dev/wonderful-mccarthy-algc5s`) is at `fbcd6b2` "Find four more kinds of leak;
  move four parked fixtures back", on top of `4aae86f` (the chance failures of the dynamic
  acceptance and the open check items) and `9781542` "Version 0.9.0-preview.2" (published as a
  pre-release; the two later commits are not in it).
- `fbcd6b2`: L-REGISTER accepts growth-series + alloc-stack; Python `gc.freeze` census entry,
  agent group `py.gc_frozen`, native codec vocabulary; Node unmeasurable probe roots and the
  FinalizationRegistry registration holder; Go rules `go-slice-view-retained` and
  `go-chain-head-growth`, holder/owner/region on a static hit; Go command mode reads the heap
  while the workload runs (`go.heap_live.in_run`). Four fixtures moved back (64 in
  `bench/fixtures/`); four stay in `bench/fixtures-open/` (its README says what each needs).
  Its tree is snapshot `41c3bca` of this branch, green on all 15 CI jobs (run 44760385093);
  local: workspace tests 2794 passed, static 64/64 in both modes, dynamic 63 pass and 1
  expected failure.
- CI on `main` at `fbcd6b2` was started by the push at 16:10 UTC on 2026-11-20
  (`gh run list --workflow ci.yml --branch main`).
- The working tree is clean. The container restarts now and then; files on disk survived so
  far, running agents did not.

## Residual chance paths (seen in the last acceptance run, not handled)
- A heap column passes the first trend by chance on a flat Go run; `os.rss` then votes and reads
  retained on a growth inside its own noise (go-ticker-modern, a decoy-control: no cost there).
- Under heavy load `os.rss` of jvm-invoice-render-trimmed passed the first trend and both
  unbound tests read leak (a decoy-control, no claim made).
- A flat Python run can still read inconclusive through `os.rss` now and then (written in the
  VERIFY rows of the alloc-test fix): the purge measures growth against the single point at the
  end of the warm-up.
- Known and not handled: under heavy load on the work machine, single failures of
  `sample::node_inspect_backend_reads_memory_and_active_resources`,
  `session::tests::a_backend_that_never_works_is_blocked_and_sampling_continues` (dla-sample) and
  one Go analyzer test of `dla-backends` `real_tools`; none failed in CI.

Snapshots of this branch: `snapshot.sh "<label>"` (copy here). A label with `[skip ci]` saves
notes without a CI run; a later snapshot without it always gets a commit, so a CI run exists.

## Next, in order
1. Confirm CI on `main` at `4aae86f`. A further preview (0.9.0-preview.3) is not planned unless
   the owner asks; the fixes go out with the next version.
2. The eight fixtures in `bench/fixtures-open/` (move each back when it passes, and restore the
   `pair` line of its partner: go-ticker-modern, node-signal-compose-22, python-zstd-options-3145).
   Diagnosis done (08:30 UTC on 2026-11-20): `open-fixtures-diagnosis.json` holds, per fixture,
   the diagnosis and its independent verification. In short:
   - go-ticker-legacy: only the label is refused; L-REGISTER needs the alternative
     growth-series + alloc-stack in `data/archetypes.toml` (checked end to end on a scratch copy).
   - python-frozen-gc: census entry for `gc.freeze()` in a function, an agent group
     `py.gc_frozen@site`, its catalog entry.
   - node-signal-compose-24: a probe on a class with only private fields reads a constant 1 and
     refutes the true finding; FinalizationRegistry cells need a registration holder.
   - go-header-index, go-feed-merge (second leak), go-trace-context: no static rule for a
     retained slice view or (Go) a chain head; and in command mode the Go agent measures after
     `main` returned, when the state is dead, so nothing grows.
   - python-zstd-options-3144: census has no native codec contexts; and the allocation stack
     lands one frame below the leak's site, so nothing ties.
   - rust-slot-arena: DLA confirms the leak at the holder's declaration `src/arena.rs:24`, which
     the answer key does not list.
   Fix pass 1 (brief `open-fixtures-fix-brief.md`), state at 12:10 UTC on 2026-11-20:
   - F-a (L-REGISTER accepts growth-series + alloc-stack), F-b (Python: `gc.freeze` census
     entry, agent group `py.gc_frozen`, native codec vocabulary), F-c (Node: unmeasurable probe
     roots, FinalizationRegistry registration holder), F-d (Go: `go-slice-view-retained`, Go in
     the chain-head-growth rule, holder/owner/region on a static hit) are written by Sonnet
     agents (`open-fixtures-report-F-*.md`) and verified by the orchestrator on the tree with
     diff hash 4d93f846: workspace tests 2778 passed, 0 failed; clippy (host and Windows
     target), fmt, schema check and grader tests pass; static passes in both modes for scratch
     copies of all eight parked fixtures with no external Go analyzer; dynamic passes for
     go-ticker-legacy, python-frozen-gc and node-signal-compose-24 with their partners.
   - The slice-view rule's false positive (callers matched by method name only) is fixed and
     verified by the orchestrator's own inputs (`open-fixtures-report-F-d2.md`).
   - F-e (Go command mode measures while the workload runs: extra columns
     `go.heap_live.in_run` and `go.heap_objects.in_run`, the in-run heap profile) is built
     (`open-fixtures-report-F-e.md`). The fixer reports go-header-index passing static and
     dynamic; go-trace-context and go-feed-merge stop at the tie rule. Not yet rerun by the
     orchestrator.
   Fix pass 1 is committed as `fbcd6b2`. Fixture work is stopped here by the owner's priority
   (top of this note).
   Waiting for the owner (asked on 2026-11-20): (1) rust-slot-arena: add `src/arena.rs:24` to
   the key's alt_sites? (2) the tie rule: may an allocation stack tie through the nearest caller
   frame that has a finding (python-zstd-options-3144, go-trace-context), or stay strict with
   an explicit `ledger annotate --writer`? (3) go-feed-merge's second leak needs a new kind of
   tie (an intervention: stub the statement, the growth drops): build it, or leave the fixture
   parked? (4) go-trace-context's key names `trace.go:35` where the call is at `trace.go:43`.
   After the packages: move the fixtures that pass back (`git mv`), restore the pair lines,
   update the READMEs and counts, full acceptance, CI on a snapshot, commit.
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
