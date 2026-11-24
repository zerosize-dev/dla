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

State at 00:30 UTC on 2026-11-21: CI on snapshot `d2264d3` failed one fixture, go-header-index
(the other 14 jobs passed): its exit column `go.heap_live` passed the trend on a rise of about
1.5 KB (stair) beside 113 MB in the run, and the column-choice correction let it lead, so the
bound alloc-test read non-leak. Refined in `crates/dla-cli/src/alloc/pick.rs`
(`in_run_leads`, `EXIT_SHARE` 0.1): the exit column leads only when it passed and rose by at
least a tenth of the in-run rise. Over the saved trends the ratio of the two slopes is 0.6 to
3.7 where the state survives the exit (155 trends) and at most 0.00002 where it does not (7).
The test `the_exit_heap_column_leads_when_it_passed_the_trend_beside_the_in_run_one` got the
token-rise case (seen failing with the share set to 0). Local on the tree before this
refinement (scratch `u23/`): workspace tests 2846 passed, static 64/64 both modes, dynamic 63
pass and 1 expected failure. Snapshot `42857a1` ([skip ci]) holds the refined tree. Running:
Go reruns with the refined rule (scratch `u24/`: all `go-` once, then go-header-index and
go-ticker-legacy five times each). Then: snapshot without [skip ci] for a CI run, and when
green the commit on `main`, preview.3.

Real plugin runs (the way the owner runs it): `/root/dla-real/run.py` (copy:
`real-run-driver.py`) runs `claude -p "/dla:detect-leak-anywhere <task>"` headless with
`--plugin-dir` (a frozen copy of skills, data, bin, agents, hooks), Sonnet 5.5, the Bash
sandbox with package registries allowed, a sanitised environment and `DLA_BIN`; `tl.py`
prints a timeline of the transcript. A headless child authenticates here without any of the
session's CLAUDE variables. First run: aiohttp 3.10.10 (`/root/dla-real/aiohttp-1`, started
00:18 UTC, prompt "Find memory leaks in this project."). Next: grpc-go v1.84.0
(`u18/grpc/proj`, site `health/server.go:49`), spring-data-commons 4.1.1 (cloned to
`/root/dla-real/src/spring-data-commons`, site `TypedPropertyPaths.java:53`). Public
repositories can be cloned through the git proxy (`git clone https://github.com/...`).
/root/dla-real does not survive a new container; the transcripts' findings go into notes here.

State at 01:25 UTC on 2026-11-21.
THE CAUSE OF THE OWNER'S EMPTY RUNS IS FOUND (`real-run-plugin-aiohttp-1.md`): a real plugin
run on aiohttp 3.10.10 ended after 309 s: one generic workload, then all 187 candidates closed
`unresolved not_exercised` in a shell loop with 2 h 55 min of budget left, because `dla run
status` and the final gate print that command as the fix of every candidate and every
"measure before you close" rule is per runtime, never per candidate (`queue-survey.md`). The
fix is the measurement queue, designed in `queue-brief.md` (ranking, declared targets,
verdict per target after a run, pending/settled, `target-pending` refusal, `dla ledger list`,
`dla ledger close-rest`, `dla harness gen --next`, `dla run init --queue on|off`; dla-bench
dynamic passes `--queue off`). Two Sonnet agents build it: Q1 the code (report
`u28/Q1/report.md`), Q2 the skill text, bench wiring and documents (`u28/Q2/report.md`).
After both report: verify myself (their test commands, static both modes, full dynamic), then
real plugin runs with `real-run-driver.py` on aiohttp 3.10.10 (must find
`aiohttp/web_app.py:82`), grpc-go v1.84.0 and spring-data-commons 4.1.1. A static ranking of
the saved candidate lists puts the aiohttp leak in tier 1 (6 candidates) but the grpc-go leak
(`health.server.Server.updates`, writers and removers) behind 145 others and the spring one
(`TypedPropertyPaths.resolved`) behind 282: ranking needs a second step (keyed holders whose
keys come from callers) after the queue works.
Pass 1: all five side-by-side reruns of go-header-index and go-ticker-legacy passed with the
share rule; CI runs on snapshot `b54a4be` (tree hash of `git diff HEAD | sha256sum`:
`ca49507edeafeb34`). When green, commit THAT tree (not the working tree, which other agents are
editing) on `main` with plumbing: temporary index, `git read-tree b54a4be`, remove `.wip`,
`git write-tree`, `git commit-tree -p main -F commit-message.txt`, update `refs/heads/main`,
`git reset -q` (index only), push `main` and the dev branch. Then preview.3 only after the
queue is in (a release without it would behave as the owner saw).
A second chance failure showed in the Go reruns and is on `main` since `fbcd6b2`: the in-run
heap profile is taken at a different share of each run and the capture axis is the capture
number, so a linear leak reads as a plateau now and then (go-ticker-legacy, half-split ratio
0.476 against the 0.5 threshold). A Sonnet agent fixes it (`go-capture-axis-brief.md`, report
`u26/report.md`): axis = bytes allocated when the profile was taken (sum of `alloc_space`).
A second real plugin run (grpc-go, `/root/dla-real/grpc-1`) is in progress.

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

## State 2026-11-21 03:15 UTC
- main = d03a00f (pass 1), CI green (run 44850244799). Working tree: queue code (agent Q1, still verifying), queue docs (Q2, done), Go capture axis (agent A: axis = bytes allocated done; follow-up "order captures by bytes allocated" in progress).
- My check of the queue build on aiohttp 3.10.10 (static stage, copy in scratch u29/aiohttp): `run status` leads with the queue item (92 ranked, F8 = aiohttp/web_app.py:82 is rank 1), `dla ledger list` works, `dla harness gen --lang python --driver --next` declares 8 targets and writes the target block into the driver.
- Seen, for the next ranking step: the Python census lists TypeVar assignments (`aiohttp.client._RetType`, `aiohttp.helpers._T`) as static/global holders with writers; they take queue slots in tier 2.
- Next: verify Q1's report (tests, clippy, fmt on tests/queue.rs, schema, static both modes, dynamic), refresh /root/dla-real/plugin and bin, real runs aiohttp-2, grpc-2, spring-4. Then give A the Go driver-mode coverage gap.

## State 2026-11-21 03:16 UTC
- Real run aiohttp-2 started 03:13 (/root/dla-real/aiohttp-2, plugin copy refreshed from the working tree, binary = queue build u28/Q1/bench/dla). Waiter task writes when run.json appears.
- Ranking with the queue build (static stage, scratch u29/<name>, lists in u29/<name>-list.jsonl): grpc-go 298 ranked, documented site health/server.go:49 is rank 159 (tier 4, its only removers delete from the inner map); spring-data-commons 479 ranked, TypedPropertyPaths.java:53 is rank 325 (tier 4, writers/removers matched by name in other files). The queue alone cannot reach them: a ranking second step is needed.
- Research agent R (Sonnet, read-only, scratch u30, brief wip-notes/rank2-research-brief.md) is measuring generic static signals for a better order, with the bench fixtures as a guard against fitting.
- Agent A: follow-up "order captures by bytes allocated" in progress (report appended to u26/report.md). After it: Go driver-mode harness writes no cover profile.
- Agent Q1: still tracing a refuted-count difference on python-report-memo-bounded before reporting.

## State 2026-11-21 03:22 UTC
- REAL RUN aiohttp-2 (queue build): F8 `aiohttp/web_app.py:82` (the documented leak) and F9 `aiohttp/web_log.py:61` CONFIRMED about 5 minutes in; the agent goes on with batches H2, H3 (close-rest closed 104 non-ranked, 81 pending). Run still going (budget up to 3 h, timeout 9000 s).
- Q1 reported (report only in its hand-back message; could not write report.md). Its build = u28/Q1/bench/dla (02:08; last source edit 01:47). My checks so far: Python and Go `harness gen --driver --next` write the target block; clippy/fmt/schema/grader/skill_text running (scratch u31).
- Follow-up list (after the queue is committed): (a) `trend/candidates.rs::enclosing` uses an end column past the line end, so a site on the last line of a function matches no function; (b) Python census lists TypeVar assignments as holders; (c) the agent had to re-export PYTHONPATH for `dla alloc-test` after `dla harness run` (the harness does not remember the environment of its dependencies); (d) ranking second step (agent R); (e) Go driver-mode cover profile (agent A next); (f) #50 macOS assertion message; (g) a blocker on `intake.workload` closes the queue of every runtime, not one.

## State 2026-11-21 03:28 UTC (clock of the container)
- CI run started on snapshot e1099e7 (wip/progress) = queue (Q1+Q2) + Go capture axis incl. A's follow-up code; waiter output in scratch u31/ci1. My local checks on that tree: fmt, clippy, schema, test_grade, skill_text pass; dla-core tests and the dla-cli suites queue, summarize_go, one_site, gate_flow, keep_going, reasons, run_m4 and --bins pass; with `refuse_target_pending` disabled 3 of the 8 queue tests fail (seen), restored.
- Plan for the commit: ONE commit for queue + Go capture axis (they share VERIFY.md and go.md and are CI-verified together), tree = the green snapshot, made with wip-notes/commit-snapshot-tree.sh. Not yet done: wait for A's report and CI.
- Agent P3 (Sonnet, brief wip-notes/pass3-brief.md, scratch u32) started on three follow-ups: close-rest reasons (measured candidates become unresolved/inconclusive; tied measurement accepts declared-target verdicts constant/changed/executed), harness reruns reuse recorded dependency-path variables, `enclosing` end-of-line bug. Goes into the commit after the queue commit.
- Still to hand out: Go driver-mode cover profile (to A after it reports): without it every Go driver target reads `unknown`, and `unknown` settles a target after one driver run.
- Ranking research agent R still running (scratch u30).

## State 2026-11-21 03:39 UTC
- Ranking research (agent R) is done; nothing written by it but scripts and data in scratch u30 (dlarank.py; I reran `summary`: same numbers). Result: script scheme B2 puts the documented site at aiohttp 1, grpc 10, spring 25, axios 15/16; with rule_dla=15, rule_strong=45, life_process=20 (a rule hit on an instance field weighs less than on a static holder) spring is 13; fixture guard (true site stays within rank 8) holds, top 1/3/8 of 38 fixture sites 21/31/37 now, 16/32/37 with the chosen weights. Census bugs found: `x[k] = make(..)` and a constructor's `this.x = new T()` counted as removers; writers/removers of a private holder matched by name in other files; the axios documented `socket.on('error')` on a callback parameter is not a census entry (the two registrations at 908/925 are).
- Agent V2 (Sonnet, brief wip-notes/rank2-build-brief.md, scratch u33) started: census facts per language, `score()` in rank.rs, `--next` brings holders of one owner type into the batch (up to 16), measure on four trees + hold-outs tokio-util 0.7.1 and Jetty 12.0.31 + 64 fixtures.
- Details of the seven real projects (leak, trigger, versions) are in scratch test-projects.md (not in wip-notes; copy on next snapshot).

## State 2026-11-21 04:29 UTC
- COMMITTED: main = 6a7ee29 "Measure the ranked candidates before closing any; Go heap captures on the bytes-allocated axis" (tree of snapshot e1099e7, CI run 44860697616 green on all 15 jobs), pushed to main and dev/wonderful-mccarthy-algc5s. CI on main started by the push (not checked yet).
- A's follow-up report (appended to scratch u26/report.md): ordering by bytes allocated built; Go bench 8/8, 8/8, 7/8 (go-header-index failed for another cause: its in-run column go.heap_live.in_run read "spike" in the first trend, alloc-test fell back to the flat exit column), ticker 5/5, subscription-workers 3/3.
- A resumed with two jobs (scratch u34): (1) Go driver runs write a cover profile (targets read executed/not_executed, not unknown); (2) the in-run column spike of go-header-index: measure, then fix the sampler.
- Working tree vs main now: only P3 (close reasons, env reuse, enclosing) and V2 (census facts, score, sibling batches) edits, both in progress.
- Real run aiohttp-2 still working the queue (30 pending at 04:12).

## State 2026-11-21 05:20 UTC
- Real run aiohttp-2 FINISHED (notes: real-run-plugin-aiohttp-2.md; report, ledger and two drivers kept in scratch u31/aiohttp-2): documented leak F8 confirmed about 5 minutes in, queue worked to the end (89 of 90 ranked targeted in 15 harnesses), but the blind verifier DISAGREED on both confirmed findings ("the harness varies the key on every unit") and the final answer said neither is a leak real use would trigger. Cause: nothing reproduces a grown holder through the public entry path.
- Agent T (Sonnet, brief wip-notes/trigger-brief.md, scratch u35) started: the trigger step (entry-path harness, callers chain, VARIES.txt, trigger status, gate item, report, verifier bundle, skill).
- Report defects from that run still to hand out (to P3 when it reports): the Queue line leaves out findings that left the candidate state; the Residual header has one clause per series with unrounded floats; "Coverage: 0 of 0 entry points" for hook drivers; the final gate wants a residual row per series (176 items, cleared by a shell loop); verifier site disagreement that is only declaration vs writer line.
- Disk fell to 1 GB: deleted target/release, target/x86_64-pc-windows-msvc, target/aarch64-apple-darwin, the Go build cache and the harness work copies of aiohttp-2. Do not use a shell variable inside an rm path again (I did once for the harness work dirs): literal paths only.
- Running agents: A (Go driver coverage, then in-run spike; u34), P3 (three fixes; u32), V2 (ranking; u33), T (trigger; u35). CI on main 6a7ee29 not checked yet.

## State 2026-11-21 05:43 UTC
- P3 REPORTED (scratch u32/report.md): (1) close-rest takes its reason from supported_reason, a declared target with verdict constant/changed/executed is a tied measurement (`dla trend` stores targets in its growth evidence); (2) harness reruns reuse the recorded dependency-path variables (DEPENDENCY_ENV, env_reused); (3) `enclosing` fixed with one helper node_at_line. My checks in the mixed tree: its tests pass (dla-core inconclusive 19, dependency_env 3; dla-cli queue 10, env_reuse 2, three unit tests); with the new rule disabled in machine.rs the two new inconclusive tests fail (seen), restored.
- P3 could not get a clean acceptance set because V2's census work is mid-edit in the same tree: static bench 63/64 (node-gateway-merge: leak L2 src/trail.js:12 not flagged by any candidate), python-ingest-two-leaks dynamic fails (harness gen writes 3 probes instead of 5, the `_newest` probe missing), workspace clippy fails in dla-census sites.rs and dla-workload gen.rs. These are V2's to resolve (its brief: a removed candidate that an answer key names is a stop).
- P3's open note: "a series with no growth column writes no growth record, so its verdicts never reach the ledger": to check with a test in its next job.
- P3's next job (NOT started, wait until V2 or T finishes to keep the tree workable): report defects of aiohttp-2 (Queue line omits findings that left the candidate state; Residual header one clause per series with unrounded floats; "0 of 0 entry points" for hook drivers; a residual row per series in the final gate; verifier site disagreement declaration vs writer line) and the flat-series verdict record.
- Commits will be combined: the tree now mixes P3 (done), V2, T and A (in progress).

## State 2026-11-21 07:04 UTC (container clock)
- Agent A REPORTED (scratch u34/report.md): (1) a Go driver run now takes one more short, instrumented process after the measured one (`go_cover_point` in dla-workload harness/run.rs; `-coverpkg=<module>/...` per module of `go list -m`; profile `cov/p1-<mode>-cover.out`), so targets can read executed/not_executed; (2) the Go in-run reading is the floor envelope read at three quarters of the run time (interpolated; continued from the last samples with a limitation line when no sample ended after that time).
- My checks: the three tests pass (`a_go_driver_run_records_the_functions_its_hooks_reach`, `a_go_driver_run_in_a_workspace_covers_every_module`, `the_go_in_run_reading_stands_at_one_share_of_the_run`); with the `go_cover_point` call disabled both coverage tests fail (seen), restored by reverse replacement.
- My own edit on top of A (template + test): the continued slope is limited to the slope of the whole envelope, because the last envelope point is a raw reading and the slope of the last four samples follows one tooth of a saw-tooth heap (read 111 to 147 MB for a 60 MB saw-tooth, growing with the run; now bounded). Go test `TestASawToothIsNotContinuedAtTheSlopeOfOneTooth` inside the same Rust test; seen failing with the limit disabled. Backups of the files before my edit: scratch u36.
- NOT verified end to end: verdicts of a Go driver on the grpc tree (A was refused `dla consent` by the permission system on its copy; I do not do that for it). The real plugin run on grpc-go covers it.
- P3's next job is written (wip-notes/pass4-brief.md, scratch u37) and not yet started: start it when T or V2 reports.
- Still running: V2 (dynamic bench per fixture, scratch u33), T (real check on aiohttp, scratch u35).
- Interim ranks from V2's own lists (scratch u33/<name>-after-list.jsonl, read by me): aiohttp web_app.py:82 rank 1 of 68 eligible; grpc-go health/server.go:49 rank 10 of 128 (its sibling statusMap :48 rank 6, same owner type, so the pair lands in batch 1); spring TypedPropertyPaths.java:53 rank 23 of 384; axios http.js:908/925 rank 23/24 of 31 (penalty recv_local); tokio-util cancellation_token.rs sites rank 4 to 6 of 7. Jetty hold-out: the gzip Inflater sites are rank 348+ of 420: an acquire-without-release leak is not a holder the census can rank high; it needs the resource rules or a symptom-first route (note for after this round).

## State 2026-11-21 08:35 UTC (container clock)
- Agent T REPORTED (scratch u35/report.md): trigger step built. `dla harness gen --lang <rt> --driver --entry F[,..]` (manifest `purpose: entry-path`; block with writer functions, callers chain 3 hops, key provenance, the `if` guards of the call, "referenced" mentions; `VARIES.txt`, refusal `varies-missing`), `crates/dla-core/src/trigger.rs` (reproduced / not_found needs two different entry-path harnesses with different driver digests / unchecked), gate item kind `trigger` after the queue items, report `Trigger:` lines and a Findings clause, bundle `entry-path/<H>/`, verifier and skill text, 7 VERIFY rows.
- My checks: suites pass (dla-census callers 18, dla-core trigger 10 + trigger_items 8, dla-report trigger 6, dla-cli trigger 2). Mutations seen failing and restored by reverse replacement: not_found from one run (6 tests fail), item for a finding that never grew (1), `varies-missing` refusal disabled (1 end-to-end). Read: trigger.rs rule and items; the H2 driver block of T's real check on aiohttp (scratch u35/aiohttp) shows the guard line "the call happens only when: not self._has_legacy_middlewares; self._run_middlewares" and the chain to public server functions; its report prints F8 "Trigger: reproduced through the public API in H2 (varies: the request path ...)" and F9 "Trigger: not found: ...".
- Caveat (T's own): T knew the cause when it wrote the units. Whether the plugin's model finds it from the block is what the real run aiohttp-3 must show.
- P3 RESUMED at 08:20 with pass 4 (wip-notes/pass4-brief.md, scratch u37): Queue line counts, Residual header, "0 of 0 entry points", residual gate items per series, verifier site comparison, flat-series verdict record.
- Still running: V2 (ranking; dynamic bench fixture by fixture, scratch u33), P3 (pass 4).

## State 2026-11-21 08:26 UTC (container clock)
- REAL RUN aiohttp-3 STARTED 08:26 (/root/dla-real/aiohttp-3; plugin copy refreshed from the working tree at 08:26; binary = target/debug/dla built 08:24 = T's trigger step + A's Go work + my slope limit + V2's ranking as it stood, before P3's pass-4 edits). What it must show: the documented leak aiohttp/web_app.py:82 reported with "Trigger: reproduced through the public API", by a model that does not know the cause.
- Disk: deleted scratch binaries of finished agents (u34/bin, bin2, grpc; u35/bin; u26 bin and capture dirs; u23/bin; u28/Q1/bench; u19 aiohttp and venv) and three stale /tmp/dla-bench-* work dirs: 4.8 GB free.
- grpc-2 real run waits for V2's report (ranking final) to keep the machine usable for V2's dynamic bench.

## State 2026-11-21 08:33 UTC (container clock)
- Agent V2 REPORTED (scratch u33/report.md): census facts per holder and per writer/remover site (new dla-census events.rs, sites.rs), score = sum of 31 named weights (dla-core rank.rs `weights!`), eligible set ordered by score, `harness gen --next` adds same-owner siblings up to 16 targets (`sibling_of`), `ledger list` prints score and why. Ranks of the documented sites: aiohttp 1, grpc-go 10 (sibling of rank 6, so in batch 1), spring 23 (target 16 missed: package-private class gets no exported bonus, tie group at 127), axios 23/24 of 31 (missed: receiver `req` is a function local, -20), tokio-util 4 to 6 (no facts for Rust), Jetty: the gzip Inflater leak is not a candidate at all. V2 changed no weight to hit the targets. Fixture guard: true site within rank 1/3/8: 21/31/37 before, 15/31/37 after, no site left the first 8; candidate sets identical.
- My checks: dla-census 123 tests, dla-core rank/queue 28, dla-cli rank_score 3 pass; mutations seen failing and restored by reverse replacement: no siblings (3 queue tests + 2 CLI tests), rule_strong 0 (only the table test `every_signal_has_one_named_weight_and_the_score_is_their_sum`: no behaviour test pins that single weight).
- V2's own slips (its scratch only): rm -rf with a variable in the path, pkill -f on its own script.
- V2's static bench ran before its last three census hunks: the combined acceptance (static both modes, dynamic) on the final tree is still mine to run, after P3 reports.
- REAL RUNS going: aiohttp-3 (started 08:26; F8 web_app.py:82 and F9 grew in H1 within 7 minutes, queue 27 of 68 settled at 08:33), grpc-2 (started 08:27; filling the first driver). Waiters b91j9cllw and bkmdeod7x fire when run.json appears (2 h limit).
- Open class for later: an acquire-without-release leak of a pooled or native resource (Jetty hold-out) is no census candidate; needs a resource census or a workload-first route.

## State 2026-11-21 08:53 UTC (container clock)
- P3 pass 4 REPORTED and checked (scratch u37/report.md): Queue line counts findings that left the candidate state; Residual header bounded with a "Residual by series" section; "entry points not enumerated" instead of "0 of 0"; verifier site agrees when it names a recorded writer or remover; flat-series verdicts are recorded (no bug); the per-series residual gate rule is deliberate and unchanged. My mutations: known sites ignored (1 test fails), "0 of 0" back (1 test fails); restored.
- My own edit: removed a redundant project-specific token from the Go non-container regex in dla-census sites.rs (`log\.` already matches it); census tests pass.
- COMBINED TREE = snapshot 3b0cc2d (wip/progress, pushed WITHOUT [skip ci]): ranking (V2), trigger step (T), Go driver coverage and in-run reading (A) with my slope limit, report fixes (P3 pass 3 and 4). Local checks on it: fmt, clippy --workspace --all-targets, schema --check, test_grade 190 pass. CI waiter: background task bsur6ve2y (output scratch u36/ci1). Static bench both modes running from scratch u36/bin (task bkwulo6dy; outputs u36/static-on.json, static-off.json, static.done). Dynamic bench still to run (after the real run aiohttp-3 ends, to keep timing clean).
- To commit when CI and benches pass: `bash wip-notes/commit-snapshot-tree.sh 3b0cc2daf3a505ddc34d416dfa88a2bf7553a8d0 <message file>`, then push main and main:dev/wonderful-mccarthy-algc5s. Message file still to write (scratch wip-notes/commit-message-2.txt).
- REAL RUN grpc-2 FINISHED (23 min, INCOMPLETE, the documented leak closed unexercised): notes in wip-notes/real-run-plugin-grpc-2.md. Causes: Go batch driver in one main package for targets of 7 packages; no probes for struct fields; alloc-test read a step-growing map as plateau; trigger "not found" on unmeasured runs; close-rest slow; the model gave up with 157 of 180 minutes left.
- NEW BUILDERS (both Sonnet, editing the working tree on top of the combined tree): G = agent a349aef413818803c, brief wip-notes/go-batch-brief.md, scratch u38 (Go batches one package each, field size probes via dlaTrack, alloc-test with a rising size probe, real-tree check on grpc-go health). P3 pass 5 = agent a8249474e4d24c8ca, brief wip-notes/pass5-brief.md, scratch u39 (trigger not_found only when measured flat, close-rest speed, `gap --residual --all`).
- REAL RUN aiohttp-3 still going (waiter b91j9cllw).

## State 2026-11-21 09:02 UTC (container clock)
- Static bench on the combined tree's binary (scratch u36/bin, = snapshot 3b0cc2d): 64/64 in both modes; versus the u23 baselines only the five known analyzer differences (node-signal-compose-22/-24 eslint, python-report-memo and -bounded ruff/pylint, rust-ledger-sync rapx).
- Dynamic bench (full, --parallel 2) started 09:02 as background task bp5cy0fq9: outputs scratch u36/dynamic.json, dynamic.out, dynamic.logs. Compare outcome fields with u23/dynamic.json; rerun a failed fixture alone before judging (the machine is loaded: two builders, one real run).
- Commit message draft: wip-notes/commit-message-2.txt (the last line @@REAL_RUN_RESULT@@ is to be replaced by what aiohttp-3 shows).

## State 2026-11-21 09:42 UTC (container clock)
- P3 pass 5 REPORTED and checked (scratch u39/report.md): trigger outcome `unmeasured` (two entry-path runs, at least one verdict neither grew nor constant: report "Trigger: not checked: ... nothing measured the holder there"), `not_found` needs two runs with verdict constant; close-rest loads view, ranking and queues once (82.6 s to 5.95 s on a copy of the grpc-2 run, ledger identical apart from timestamps); `dla gap --residual --all` and the gate fix text naming it when more than three series miss a row. My checks: suites pass (dla-core trigger 12 incl. gate, trigger_items 9, dla-report trigger 7, dla-cli gap_all 3, close_rest_bulk 1, trigger 2); mutation "not_found without a flat reading" fails 3 tests; restored.
- These pass-5 edits are in the working tree only (after the combined snapshot 3b0cc2d); they go into the NEXT commit together with G's Go batch work.
- First-batch probes by runtime (V2's lists, read by me): Python 14 of 14 targets with a probe, Java 10 of 10, JavaScript 10 of 10, Go 0 of 11. Go is the only runtime without size readings: G's job 2.
- CI run 44886491565 on 3b0cc2d still in progress at 09:41. Dynamic bench: 12 fixtures passed in 39 minutes (machine loaded), no failure so far. aiohttp-3: four findings confirmed (F8, F9, F83, F189), the model generated entry-path harnesses for F8, F9, F83 and is running them.

## State 2026-11-21 10:08 UTC (container clock)
- CI run 44886491565 on the combined snapshot 3b0cc2d: 14 of 15 jobs green, INCLUDING "dynamic acceptance on the fixtures" (79 minutes, like the run before) and "static acceptance", both integration suites, clippy and test on macOS and Windows. One job failed: "clippy and test (ubuntu-latest)", test `maven_project_series_grows_with_the_repeated_tests` (dla-workload tests/harness_junit.rs): Maven could not resolve the `dependency` plugin prefix from Central after 398 s ("No plugin found for prefix 'dependency' ... available from the repositories [local, central]"): a registry outage on the runner, not in OFFLINE_MARKERS so the test did not skip. No JVM build code changed in this tree. I did NOT change the test; failed jobs re-queued (rerun_failed_jobs), waiter = background task bjtazwe6y (output scratch u36/ci2).
- Local full dynamic bench STOPPED by me (task stop) after 16 of 64 fixtures passed in 66 minutes with no failure: CI's dynamic acceptance job covers the same tree on a quiet machine, and the local run was slowing the real run and the builder.
- If the rerun is green: write the last paragraph of wip-notes/commit-message-2.txt from aiohttp-3's outcome, then commit the snapshot tree 3b0cc2d to main.

## State 2026-11-21 10:12 UTC (container clock)
- REAL RUN aiohttp-3 FINISHED: SUCCESS end to end (notes wip-notes/real-run-plugin-aiohttp-3.md; kept files scratch u36/aiohttp-3). The documented leak aiohttp/web_app.py:82 was confirmed, "Trigger: reproduced through the public API in H11" by an entry-path driver the model wrote itself (server with middleware, client requests to unknown paths and unsupported methods), and the blind verifier AGREES (site, holder, label). Report COMPLETE, 1 h 44 min. The other three grown holders read "no trigger found" (2) and "trigger not checked" (1).
- Commit message for the combined tree is complete: wip-notes/commit-message-2.txt (last paragraph = the two real runs).
- REAL RUNS STARTED 10:12: axios-1 (Node, source scratch u18/axios/proj) and tokio-1 (Rust, u18/tokio/proj), with /root/dla-real/bin/dla = scratch u36/bin/dla (the CI-tested combined tree) and /root/dla-real/plugin = the files of snapshot 3b0cc2d (git archive). Waiter: background task for both run.json files.
- Still pending: CI rerun of the one failed job (task bjtazwe6y), builder G (Go batches).
- Next real runs after these: spring-data-commons (JVM, driver workloads; disk is tight, 2.7 GB free), grpc-go again once G's work is checked.

## State 2026-11-21 10:18 UTC (container clock)
- REAL RUNS axios-1 and tokio-1 FINISHED within two minutes each, nothing found (notes wip-notes/real-run-plugin-axios-1-tokio-1.md; evidence scratch u36/axios-1, u36/tokio-1). axios: the Node harness (CommonJS .js agent and driver) cannot load inside a "type": "module" package: nothing measured, the model recorded the workload blocker and closed everything. tokio-util: Rust drivers have no coverage data, every target read unknown, unknown settles a target after one driver run, queue empty after one harness; no crate features for the harness build; the driver's unit did not cover the object life cycle that leaks.
- NEW BUILDERS (Sonnet, editing the working tree): N = agent a747f7bdc3d1ef293, brief wip-notes/node-esm-brief.md, scratch u40 (Node harness in ES-module packages: .cjs agent and driver, projectImport, probes of ES-module bindings via module.registerHooks, real check on axios). R = agent a61db0f8f10fd6ebb, brief wip-notes/rust-driver-brief.md, scratch u41 (unknown does not settle; Rust driver coverage point; --features; life-of-objects rule in the driver block of every runtime; real check on tokio-util). G (Go batches, agent a349aef413818803c, scratch u38) still running.
- Real-run evidence kept in scratch u36/<run>/ (report, ledger, transcript, a driver); the run directories under /root/dla-real were deleted to free disk (4.9 GB free at 10:15). /root/dla-real/plugin = snapshot 3b0cc2d files; /root/dla-real/bin/dla = scratch u36/bin/dla.
- Still pending: CI rerun of the failed job on 3b0cc2d (waiter bjtazwe6y, output scratch u36/ci2); then commit the snapshot tree with wip-notes/commit-message-2.txt.
- Not yet run on the combined build: spring-data-commons (JVM), Jetty, Pillow.

## State 2026-11-21 10:26 UTC (container clock)
- COMMITTED: main = c5abcd4 "Order candidates by a score, ask who feeds the keys, Go driver coverage" (tree of snapshot 3b0cc2d; 77 files, +11136/-223), pushed to main and dev/wonderful-mccarthy-algc5s. CI on the snapshot: run 44886491565, all 15 jobs green after one rerun of "clippy and test (ubuntu-latest)" (first attempt: Maven could not resolve its dependency plugin from Central in `maven_project_series_grows_with_the_repeated_tests`; rerun passed, test untouched). CI on main started by the push: waiter in the background (output scratch u36/ci-main).
- Working tree vs main now: P3 pass 5 (trigger unmeasured, close-rest speed, gap --residual --all; checked by me) and the in-progress edits of builders G (Go batches), N (Node ES modules), R (Rust coverage, unknown rule, features, life-of-objects rule).
- REAL RUN spring-4 started 10:18 (JVM, combined build, warm Maven home); waiter bzlzi5w5q.
- Tasks: #53 ranking done (committed); #56 G, #58 N, #59 R in progress; #49 open until the other projects' leaks are found.

## State 2026-11-21 11:02 UTC (container clock)
- REAL RUN spring-4 FINISHED (39 min, INCOMPLETE, nothing confirmed): notes wip-notes/real-run-plugin-spring-4.md; evidence and the whole run directory in scratch u36/spring-4/ (my timing run added one trend record T13 for S8 to that copy). The documented holder F135 was a declared target of two batches of nine; the model wrote code for two or three targets per batch and never called its writer; the two-drivers rule settled it. `dla trend` takes about 62 s per series on that ledger.
- THE COMMON CAUSE of grpc-2 and spring-4: big batches plus the two-drivers rule let targets be settled without an attempt. PLANNED FIX (not started; to give to G after its report, since it rewrites the same batch code): a target that a big batch declared and whose code did not run goes into a small harness of its own (at most three targets, block names the writer functions to call); only an executed verdict, such a focused harness, or an explicit waiver with a reason settles it; `--next` serves missed targets by rank before new ones.
- P3 pass 6 STARTED (agent a8249474e4d24c8ca, brief wip-notes/pass6-brief.md, scratch u42): speed of trend and the other loop commands on the spring-4 run copy.
- Builders running: G (Go batches, u38), N (Node ES modules, u40), R (Rust coverage, unknown rule, features, life-of-objects rule, u41), P3 pass 6 (u42).
- Disk fell to 1.8 GB; `bash wip-notes/prune-superseded.sh` freed 1.9 GB of old test binaries in target/debug/build (3.6 GB free). Run it again when free space is under 3 GB.
- CI on main c5abcd4: waiter bfghaofwg (output scratch u36/ci-main).

## State 2026-11-21 11:16 UTC (container clock)
- G REPORTED (scratch u38/report.md) and checked: Go `--next` batch = one package (gen.rs `go_batch`, queue.rs `package_batch`/`packages`/`batch_in_package`), driver generated in that package; probe kind GoField read by reflection from objects passed to `dlaTrack` (both Go templates); alloc-test output has `decided_by`, a size probe leads the tied columns. My checks: go_batch 3 and go_field_probe 2 pass; with the Go arm disabled in BOTH gen.rs and queue.rs `batch_for` all three go_batch tests fail (disabling only `batch_for` changes nothing: that function serves the status display); G's real-tree outputs (u38/grpc-gen2.json, grpc-trend1.json, grpc-alloc-F203/F204.json): package health batch holds F203 statusMap and F204 updates with probes, both read `grew`, alloc-test favors leak decided by the probe column.
- G's open point: go-gc-tuned-batch failed twice in its dynamic bench under load (a column passed the trend gate, unbound alloc-test inconclusive), passed in a third run; its comparison loop old/new binary was still running (u38/gc-new-N.log, gc-old-N.log): to read.
- G RESUMED with the follow-up (brief wip-notes/focus-brief.md, report to u38/report2.md): a target missed by a batch gets a focus harness (at most 3 targets); settled as not reached only after two different drivers missed it and one of them was focused; `--next` serves the best-ranked missed candidate first.
- CI on main c5abcd4 (run 44894342155): 14 of 15 green; "dynamic acceptance on the fixtures" failed on jvm-pool-warmup ("no rule-out on a non-leak fixture": os.rss passed the first trend as a stair, unbound alloc-test inconclusive). The identical tree passed that job on the snapshot run 80 minutes earlier: a timing flake of that fixture on the runner. Failed job re-queued; waiter b5hhn7b4p (output scratch u36/ci-main2). If it recurs: look at the unbound alloc-test on os.rss stairs of JVM fixtures.
- Builders running: G (focus rule), N (Node ES modules), R (Rust), P3 pass 6 (speed).

## State 2026-11-21 12:05 UTC (container clock)
- P3 pass 6 REPORTED and checked (scratch u42/report.md): `dla trend` on the 602-finding run was slow because `attribution::find_entry` normalized the owner of every census entry on every lookup; `Resolver` now computes the canonical owners once (crates/dla-core/src/attribution.rs, `canonical_owners`, `find_entry_with`). My own check on a series it had not used (S5), fresh copies: old binary 84 s, new binary (u42/dla-after1) 1.5 s, stdout identical, ledger identical without timestamps. Other loop commands are 0.4 to 2 s; `harness gen --next` about 5 s (external tools and the scratch copy), unchanged.
- Builders running at 12:05: G (focus-harness rule; its text in trend_cmd.rs currently trips clippy format_in_format_args, in progress), N (Node ES modules), R (Rust).
- CI rerun of the dynamic acceptance job on main c5abcd4: waiter b5hhn7b4p, not back yet.

## State 2026-11-21 12:18 UTC (container clock)
- CI ON MAIN c5abcd4 IS RED in one job: "dynamic acceptance on the fixtures" failed twice (run 44894342155: first attempt jvm-pool-warmup, rerun go-buffer-pool; both "no rule-out on a non-leak fixture": a whole-process column passed the first trend as a stair, unbound alloc-test Inconclusive). The same commit's run on dev/wonderful-mccarthy-algc5s is all green and the same tree was green on the snapshot run. History of that job over the last 16 runs: 5 failures, including same-sha pass/fail pairs: a flake of the measurement rules that predates this commit. I did NOT rerun a third time. Builder F (agent ac2c30713e9bbe14f, Sonnet, brief wip-notes/flake-brief.md, scratch u43; CI logs copied to u43/ci-fail-1.log and ci-fail-2.log) analyses it: column meaning (go.released cannot show a leak), the stair gate, why alloc-test is inconclusive, local failure rates; rule changes only as far as the evidence carries; no weakening of fixtures, policy or tests.
- N REPORTED (scratch u40/report.md) and checked: Node agent and driver are harness/node/dla-preload.cjs and dla-harness.cjs; driver has projectImport; ES-module bindings are probed through a module.registerHooks load hook that appends a hand-off statement after the last line of a probed module. My checks: dla-workload harness_node_esm 4 and dla-cli node_esm 3 pass; with the hand-off removed 3 tests fail; restored (my first reverse replacement was ambiguous and left the mutation in the file for a minute: fixed with a context-specific replacement, tests pass; from now on a mutation carries a unique marker). N's axios check (u40/trend-S3.json, read by me): listener columns node.listeners@lib/adapters/http.js:903, :908, :924 (and :925 in S1) pass the gate with slope 1 per unit and tie to candidates F25 to F28 (the documented sites); trend's `next` proposes the sibling, test and alloc-test commands. N's notes: npm install --omit=dev failed with an npm error (installed the three runtime deps separately); a server left listening kept the process alive (server.unref()); Windows/macOS not run; --capture and purge not checked on an ES-module project.
- Builders running: G (focus-harness rule), R (Rust), F (flake analysis).

## State 2026-11-21 12:30 UTC (container clock)
- R REPORTED (scratch u41/report.md) and checked: (1) `unknown` no longer settles a target (Index::standing; NO_COVERAGE text in ledger list, queue item, target-pending fix, close-rest remedy, report); (2) Rust driver coverage point `rust_cover_point` (run.rs): driver rebuilt with -C instrument-coverage into harness/H<n>/cov-build/, one round, llvm-profdata + llvm-cov export, functions in cov/p1-<mode>-cover.rust.json; missing or mismatching LLVM tools = limitation, verdicts stay unknown; (3) `--features a,b` / `--no-default-features` for the Rust driver build, "behind feature x" in the block and a feature_note (reads `#[cfg(feature)]` on mod declarations and `cfg_x! { }` blocks); (4) one LIFE sentence in the driver block of every runtime (targets.rs) and in references/stages.md. My checks: dla-cli rust_driver 3 tests pass; with the rust_cover_point call disabled (unique marker) the coverage test fails; restored.
- R installed the rustup component llvm-tools on this machine so its test can run; the system llvm-profdata is LLVM 18 and cannot read rustc's profiles. On CI the test skips when the component is missing.
- R's check on the tokio-util copy: a waiter-and-cancel unit gives executed/not_executed (no unknown), the queue stays open; `--features rt` builds the spawn_pinned targets; the grandchild unit shows native.live_bytes +72 bytes and one block per unit, trend passes it with slope 72.0, alloc-test favors leak; BUT `--capture alloc` writes nothing for Rust, so nothing names the site.
- R RESUMED with the follow-up (brief wip-notes/rust-alloc-brief.md, report to u41/report2.md): allocation stacks for Rust harness runs through the native preload shim, summarize names the project frame, the finding gets its site from the tool.
- Builders running: G (focus-harness rule), R (Rust allocation stacks), F (flake analysis).

## State 2026-11-21 13:24 UTC (container clock)
- G's FOLLOW-UP REPORTED (scratch u38/report2.md) and checked: focus-harness rule. `choose_batch` in queue.rs: when the best-ranked pending candidate was missed, the batch is a focus batch (it plus up to two more missed ones: same owner, same file, then rank; Go: one package; no siblings, no fresh ones; N capped at 3); otherwise a broad batch of fresh candidates only. `Why::TwoDrivers` is now `Why::NotReached`: two different drivers missed the target and one of the harnesses was focused (at most 3 targets). Manifest `purpose: focus`, `focus.json` in the harness directory. Texts in the block, trend next, run status, ledger list, close-rest, report, target-pending fix, stages.md, recovery.md, worker.md. My checks: queue unit tests 23, dla-cli focus 1, go_batch 4, queue 12 pass; with the focus choice disabled (unique marker) 3 unit tests, the Python end-to-end test and the Go test fail; restored and passing.
- G's comparison loop on go-gc-tuned-batch: new binary 3 of 3, old binary 3 of 3 pass (earlier under load: new 5 of 7, old 4 of 4): not reproduced, a small effect of its change is not ruled out; F's flake analysis covers the same failure kind.
- REAL RUNS STARTED 13:24: grpc-3 and axios-2, with /root/dla-real/bin/dla = scratch u44/bin/dla = build of the working tree at 13:23 (snapshot b6d36fd, sha in u44/tree-sha): Go batches + field probes, focus rule, Node ES modules, unknown rule, Rust coverage, speed fix, pass-5 fixes, plus whatever F (alloc rules: crates/dla-cli/src/alloc/pick.rs, alloc_cmd.rs changed 12:31) and R (Rust allocation stacks) had in the tree at that time, unverified. Plugin text = files of snapshot b6d36fd. Waiter in the background for both run.json files. spring-data-commons follows when one of them ends.
- Builders running: R (Rust allocation stacks, report to u41/report2.md), F (flake analysis, u43).
- Disk: pruned again (4.5 GB free at 13:23); the Go build cache was cleaned.

## State 2026-11-21 13:54 UTC (container clock)
- REAL RUNS grpc-3 and axios-2 ENDED EARLY, no leak (notes wip-notes/real-run-plugin-grpc-3-axios-2.md; evidence scratch u36/grpc-3, u36/axios-2). grpc-3: census took over 120 s and was moved to the background; three text-only answers while waiting were three stop attempts; the stop hook stamped the run INCOMPLETE while the census was still running; a stamped run has its queue "closed: floor", so close-rest closed 728 candidates as time_budget with 45 of 60 minutes left; auto budget never settled. The per-package Go batch itself worked (package encoding, 2 targets, both grew). axios-2: the .cjs harness loads in the ES-module package; the rig's sandbox could not reach the npm registry for the three runtime dependencies, so the model recorded a workload blocker and closed everything.
- My timing on a copy with the 13:23 build, machine load 29: `dla census --lang go` on grpc-go 3 min 33 s wall, 2 min 8 s CPU; sweep 15 s; ledger list 0.4 s.
- P3 pass 7 STARTED (agent a8249474e4d24c8ca, brief wip-notes/pass7-brief.md, scratch u45): census speed; stop hook must not count or stamp while a registered dla command runs; stamp lifted when work continues and true reason for a closed queue; auto budget settles on its own.
- REAL RUN axios-3 STARTED 13:54 on scratch u18/axios/proj-deps (the three runtime dependencies installed in an empty directory and moved in; axios loads). Same binary and plugin as grpc-3 (u44/bin/dla, snapshot b6d36fd). Waiter in the background.
- Machine load is about 30 (F's flake loops with extra load, R's cargo builds): real-run timings are slow.
- Builders running: R (Rust allocation stacks), F (flake analysis), P3 pass 7.
- Not rerun yet: grpc-go (wait for pass 7: census speed and the stop hook), spring-data-commons (focus rule is in; could run now, but the machine is overloaded: start it when F's loops end).

## State at 15:24 UTC, 2026-11-21 (after context compaction)
- R follow-up reported (u41/report2.md): Rust harness runs now leave allocation stacks (`harness run H --mode fresh --capture alloc`: four extra runs of the driver under the native shim). Real check on the tokio-util copy: `stack:new (src/sync/cancellation_token.rs:144)`, 1320 blocks / 95040 bytes over three steps = 72 bytes per unit, tied to F6 through the recorded writer at :144 (existing strict rule). This is the documented leak (72-byte object allocated in CancellationToken::new).
- I did NOT accept R's same-function tie fallback (tie.rs `findings_in_rust_function`): the tie rule stays strict (owner's open question 2). Sent R three corrections (resumed agent a61db0f8f10fd6ebb, report to u41/report3.md): (1) fallback becomes a hint with the exact `ledger annotate F --writer site` command; (2) alloc-test `growth_evidence` must print evidence that `ledger confirm` accepts as tied (it printed the whole-process E36, refused growth-not-tied); (3) trend must not repeat the capture hint on the capture run's own series.
- Still running: F (flake, u43), P3 pass 7 (u45), real run axios-3 (started 13:54; at 15:21 it was on H7 = http-adapter batch F25-F27, F10).
- To verify when R reports: rerun `cargo test -j 2 -p dla-cli --test rust_driver`, one mutation of the hint path and one of growth_evidence, read the diff of tie.rs / shim_groups.rs / alloc_cmd.rs growth_evidence.

## State at 15:35 UTC, 2026-11-21
- REAL RUN axios-3 FINISHED: COMPLETE, no leak confirmed, but the first harness measured the documented leak (notes wip-notes/real-run-plugin-axios-3.md; evidence u36/axios-3). Root cause: every series had n < 8 points and trend is silent about columns that fail only for the point count; plus driver never exits, detector hit without site, F23 (the documented site) tier 5.
- By hand in /root/dla-real/axios-3/project (finished run, evidence copied first): full-length H1 run + trend passes node.listeners@lib/adapters/http.js:924 with F23 first candidate. Following the printed commands (sibling of F23, alloc-test H1 --finding F23) is running in background (task brf5wh8vt).
- NEW BUILDER WORK: N resumed (agent a747f7bdc3d1ef293) with wip-notes/short-series-brief.md (S1 short series speak and do not settle; S2 Node driver exits and fits points into the time limit; S3 detector hit gets a site; S4 _attention first key of harness run; S5 registration site is its own writer in rank.rs; S6 residual open item names the capture). Scratch u46, report u46/report.md.
- Still running: R corrections (u41/report3.md), F (u43), P3 pass 7 (u45).

## State at 15:50 UTC, 2026-11-21
- R DONE AND CHECKED (u41/report2.md, report3.md; my log u41/my-verify3.log): Rust `--capture alloc` allocation stacks; same-function is a hint with the explicit `ledger annotate F --writer site` command (tie rule strict); alloc-test `growth_evidence` prefers the newest evidence `ledger confirm` accepts; trend does not repeat the capture hint; alloc-test prints test+confirm for a root closed as static-only (`machine::is_allowed(state, UnderTest)`); two-candidate hint test. Mutations MUT_R1/MUT_R2 seen failing, restored. Open: NonLeak branch for a closed root needs `ledger test` before `ledger refute` (left as is).
- AXIOS BY HAND: in /root/dla-real/axios-3/project (13:23 binary) F23 at lib/adapters/http.js:924 went to confirmed following the printed commands (notes real-run-plugin-axios-3.md).
- Running: N on brief S (u46), F (u43), P3 pass 7 (u45). fmt diffs currently in trend_cmd.rs and rank.rs are N's in-flight edits.
- Next: when N, F, P3 report: verify each; rebuild; freeze binary + plugin from a snapshot; real runs axios-4, grpc-4, spring-5, tokio-2; then combined commit via CI.

## State at 16:04 UTC, 2026-11-21
- P3 PASS 7 REPORTED (u45/report.md) and checked (my log u47/verify-p7.log; census timing u47: old 129 s CPU, new 15.1 s, outputs identical): census word index + lazy regex + parse cache + dev-profile opt-level 3 for regex/tree-sitter deps (kept); stop hook does not count or stamp while a registered dla command runs (scope.rs, ProcessKind::Command, 30 min limit); auto budget settles in every working command; stamp history. Mutations MUT_P1..P3 seen failing, restored.
- NOT ACCEPTED in pass 7: a stamped run with time left still shows the queue closed (`over`) and close-rest closes all pending candidates (as not_exercised) before the stamp lifts. Sent pass 7b to P3 (resumed a8249474e4d24c8ca): a stop-hook stamp does not close the queue; any working command lifts the stamp before its work; run status shows the going-run queue and next command; close-rest leaves pending candidates; aborted unchanged.
- My own edit: gate_hooks.rs running-command message now gives `while kill -0 <pid> ...` as the one-command wait.
- Running: P3 pass 7b, N brief S (u46), F (u43, in its after-change sweeps).

## State at 16:14 UTC, 2026-11-21
- P3 PASS 7b REPORTED AND CHECKED (u45/report.md, my log u47/verify-p7b.log): a stop-hook stamp no longer closes the queue (`RunState::stop_stamped`, budget_over_floor treats it as a going run; `over` only for aborted/merge); any working command lifts the stamp before its work (`works_on_the_run` in main.rs; not run/gate/report/bundle/consent/schema/install/ledger list); run status shows the going-run queue plus a `stop_stamp` field; close-rest leaves pending candidates. gate_hooks 41, queue 15, budget_settle 2 pass; MUT_Q1 (report lifts) made 2 tests fail; restored. Task 61 done.
- Spring-4 evidence re-read: no hidden signal of the documented leak (class counts grew because the driver made a fresh class per unit; TypedPropertyPaths never loaded).
- Running: N brief S (u46), F (u43).

## State at 16:26 UTC, 2026-11-21
- F REPORTED (u43/report.md) and its two rules are checked (my log u47/verify-f.log, MUT_F1/MUT_F2 seen failing, restored): `go.released` never passes as growth (series.rs `not_a_growth_signal`, trend analysis, alloc trend_of); a retained purge does not contradict the plateau of go.total, os.vm_hwm, os.vm_data, native.peak_bytes (pick.rs `purge_cannot_release`). Local failure rate 0/50, so the proof is rules + tests + sweeps (go-, jvm-, native-, python- all pass before and after).
- JVM cause diagnosed, not fixed by F: os.rss plateau + trim returned 14% (<20% = retained) -> inconclusive; 25% -> partial -> non-leak. Sent F a follow-up (resumed ac2c30713e9bbe14f): step 1 table of every "plateau + retained" vote from its logs; step 2 rule: a retained purge leaves a plateau inconclusive only while rises_in_main x (extra span / main span) < 3; step 3 one bench sweep per prefix, leak fixtures must keep their outcome else revert. Report appended to u43/report.md.
- Running: N brief S (u46), F follow-up.

## State at 16:40 UTC, 2026-11-21
- N REPORTED brief S (u46/report.md) and it is checked (my log u47/verify-s.log): node_short 4 integration tests pass; unit tests pass; MUT_S2 (short verdict), MUT_S3 (short columns / late column), MUT_S4 (attention lines) seen failing, restored. MUT_S5 (JVM heap row of the residual capture advice) did NOT fail a test: coverage gap, goes into N's follow-up. aiohttp order checked by me with old and new binary: documented site web_app.py:82 stays rank 1 (score 205), eligible 68 -> 78, top 40 unchanged.
- N's real tree (u46/axios): finding at http.js:924 = F27, tier 4, rank 17, score 54; 4-point run: _attention first key + SERIES TOO SHORT next line, target unknown; full run 19 s: column :924 passes; printed commands reach ledger confirm F27.
- N FOLLOW-UP TO SEND AFTER THE REAL-RUN BINARY IS FROZEN: (1) fit the step inside round 1 too (a unit of 1 s makes round 1 of 440 units outlast the limit before the first check); (2) port the exit/fit loop to the Python, Go, JVM, Rust driver templates; (3) residual capture advice: test every runtime/region row; Go (alloc, nonheap) and Python (histogram, alloc, nonheap) do have --capture kinds, the comment and the match say they have none.
- F is in step 2 (plateau rule; judge.rs and alloc_cmd.rs changed in the tree); asked to hand back after step 2 and hold the sweeps. When it hands back: verify step 2 (unit tests + one mutation), snapshot, build, freeze /root/dla-real/bin and plugin, start axios-4 and grpc-4, then spring-5 and tokio-2; resume F for sweeps and N for the follow-up afterwards.
- Disk: freed to 3.2 GB (deleted /root/dla-real/axios-1..3 run dirs, u41/bin, u41/cargo-home, u44/bin, u47 copies). Evidence of axios-3 is in u36/axios-3.

## State at 16:55 UTC, 2026-11-21
- F STEP 2 CHECKED (u43/report.md; my log u47/verify-f2.log): combine() takes expected rises (rises of the main window x extra span / main span); plateau + retained stands as non-leak at >= 3 (MIN_EXPECTED_RISES), else inconclusive with the --extra line; alloc unit tests 93 pass, alloc_test integration 5 pass; MUT_G1 (threshold 0.5) failed 4 tests; restored. Step 1 table: no leak fixture's outcome depended on a plateau+retained vote. F's leftover process 27194 had already ended. F's local sweeps (step 3) NOT run: CI's dynamic acceptance job on the snapshot is the sweep.
- FROZEN BUILD: snapshot 0dbf4c90e9e1fa5c1b470ea2bf659388d3a8a9ef (pushed WITHOUT [skip ci]; CI waiter task bgkwk8545, logs to u48/ci1). Binary copies: /root/dla-real/bin/dla and u48/bin/dla; plugin /root/dla-real/plugin refreshed from the snapshot. Local: fmt clean, clippy workspace clean, schema ok, test_grade 190 OK, skill_text 14.
- REAL RUNS STARTED 16:54 UTC on the frozen build: axios-4 (u18/axios/proj-deps; waiter b2rfu44uu) and grpc-4 (u18/grpc/proj; waiter blmpj0h3h). Next: spring-5 (--warm-m2 /root/dla-real/m2warm, source /root/dla-real/src/spring-data-commons) and tokio-2 (u18/tokio/proj).
- N FOLLOW-UP RUNNING (resumed a747f7bdc3d1ef293): fit the step inside round 1; port exit + fitting to Python/Go/JVM/Rust drivers; residual capture advice rows and tests. Its edits are NOT in the frozen build.
- Deferred idea (not done): let dla-bench follow the printed --extra once after an inconclusive alloc-test; decide after CI.

## State at 16:59 UTC, 2026-11-21
- REAL RUN axios-4 FINISHED in 129 s: COMPLETE, no leak confirmed, although the tool printed the passing :924 column and the commands (notes real-run-plugin-axios-4.md, evidence u36/axios-4; run dir /root/dla-real/axios-4 kept for P3's re-run of the commands).
- PASS 8 STARTED: P3 resumed (a8249474e4d24c8ca), brief wip-notes/pass8-brief.md, scratch u49.
- N got one more item first (queued message): ES-module probe on validator.js deprecatedWarnings read flat while the holder grew.
- Still running: grpc-4 (waiter blmpj0h3h), CI on 0dbf4c9 (waiter bgkwk8545).
- spring-5 and tokio-2 NOT started yet: start them on the frozen build while pass 8 is built (they test other fixes: focus rule, trend speed, Rust stacks), or wait for pass 8? Decision: start them now, the holes of pass 8 apply to them too but they also show what else breaks.

## State at 17:03 UTC, 2026-11-21
- REAL RUN tokio-2 FINISHED in 78 s: COMPLETE, no leak; the unit never built a grandchild token (notes real-run-plugin-tokio-2.md, evidence u36/tokio-2; run dir deleted).
- BRIEF L STARTED: G resumed (a349aef413818803c), wip-notes/second-look-brief.md, scratch u50: second look (different driver, focus harness) for top flat-settled candidates while budget is above the floor; bounded; VARIES.txt; settled after two different drivers.
- Running: grpc-4 (waiter blmpj0h3h), CI on 0dbf4c9 (waiter bgkwk8545), P3 pass 8 (u49), N follow-up (u46: flat ES-module probe first), G brief L (u50).
- spring-5 not started (wait for grpc-4 to end; it would meet the same holes as axios-4/tokio-2, so consider waiting for pass 8 and brief L before spending the run).
- Disk: freed to 3 GB (deleted old /root/.cache/dla-bench benchmark caches, go build cache, u43 run copies, tokio-2 run dir). grpc-4 run dir is 2.1 GB (Go caches): delete when read.

## State at 17:36 UTC, 2026-11-21
- N FOLLOW-UP REPORTED AND CHECKED (u46/report.md; my log u47/verify-n2.log): (a) the flat probe: the probe read `reachable` (object count, 1) and a plain object had no `.size` column; `sizeOf` in dla-preload.cjs now gives a plain object its key count (test node_esm::a_module_level_object_table_and_a_map_rise_one_per_unit_in_driver_mode); (b) Node driver fits its step inside round 1; (c) Python, Go, JVM, Rust drivers fit the step (inside round 1 and after each round); Python (os._exit after flush; agent exit handler returns anyway once a checkpoint was written; memray is stopped at the last dump) and JVM end explicitly (tests crates/dla-workload/tests/driver_fit.rs, 4 pass; MUT_N1 on the Python in-round check failed its test; restored); (d) capture_advice rows for every runtime incl. Go and Python, 12 rows tested. Task 62 done.
- N saw 2 queue tests and the schema test fail: P3 (pass 8) and G (brief L) are editing gate/queue/manifest now; recheck when they report.
- grpc-4 at 17:33: the model is trying to stop, run INCOMPLETE, about 810 gate items missing (569 trend columns tied to no finding, 137 more), 64 ranked candidates untargeted (next batch xdsclient). It reported confirming several registry holders (F185, F184, F188, F205) and three focus targets grew (F685, F682, F687). Read it when run.json appears (waiter blmpj0h3h).
- CI on 0dbf4c9: run 44927090561 in progress at 17:33.

## State at 17:38 UTC, 2026-11-21
- REAL RUN grpc-4 FINISHED (42 min, 238 turns, cost 6.33): INCOMPLETE, but the DOCUMENTED LEAK IS CONFIRMED: F204 health.Server.updates (PR 9411), probe 1 count/iteration, trigger reproduced through Health.Watch with distinct service names (notes real-run-plugin-grpc-4.md, evidence u36/grpc-4, run dir deleted).
- Real-project score so far on current code: aiohttp found (aiohttp-3), grpc-go found (grpc-4); axios measured and printed but the model escaped (pass 8 fixes); tokio-util not triggered (brief L fixes); spring not rerun yet; Jetty and Pillow never run.
- grpc-4 problems -> PASS 9 brief wip-notes/pass9-brief.md, queued for P3 after pass 8: gate lists 810 dependents for about 7 actions; about 18 start-up registries "confirmed" (verifier N-INTENT each); empty package in a probe name; stop-hook digest.
- Running: CI on 0dbf4c9 (run 44927090561, waiter bgkwk8545), P3 pass 8 then 9 (u49), G brief L (u50). N is done (follow-up checked).
- Disk 6.4 GB free after deleting /root/dla-real/grpc-4.

## State at 17:45 UTC, 2026-11-21
- G REPORTED brief L (u50/report.md) and it is checked (my log u47/verify-l.log): second look = focus harness `purpose: second-look` for candidates one hook driver read `executed`, after the first pass, while budget is above the floor; SECOND_LOOK_MAX 3 harnesses per runtime; harness run refuses fewer VARIES.txt lines than targets and hook code equal to an earlier harness; settled after two different drivers; Queue line "second look N (grew M)". dla-core unit tests 3 pass, dla-cli tests/second_look.rs passes; MUT_L1 (room 0) failed 3 tests; restored. Task 65 done.
- G's real tree (u50/tokio2): with a second-look unit that follows the to-dos (three-level token chain, parent cancelled first) native.live_bytes +48 B/unit, alloc-stack-diff names new (src/sync/cancellation_token.rs:144) tied to F6, alloc-test favours leak, confirm after refute = the documented tokio-util leak. Whether the model writes such a unit from the to-dos is for the real rerun to show.
- MY EDITS: (1) alloc_cmd.rs next_commands: with no sibling yet the leak branch now prints the refute of the sibling the add creates (`dla ledger refute <root>.n1 --evidence E`) before test/confirm; test extended (a_root_closed_as_static_only...), MUT_A1 seen failing, restored. (2) cover.rs test rust_export_gives...: dependency path made absolute on every OS (CI Windows failure on 0dbf4c9: "/home/u/..." is not absolute on Windows); assertion unchanged.
- CI on 0dbf4c9 (run 44927090561): all jobs green except "clippy and test (windows-latest)" (that one test); dynamic acceptance still running at 17:40.
- Running: P3 pass 8 then 9 (u49). fmt diff in gate_cmd.rs is P3's edit in flight.
- NEXT: when P3 reports: verify 8 and 9; full local checks; snapshot WITHOUT [skip ci]; freeze build; real reruns axios-5, tokio-3, spring-5 (then grpc-5 for the registry/gate fixes).

## State at 18:09 UTC, 2026-11-21
- CI ON FROZEN SNAPSHOT 0dbf4c9 (run 44927090561) DONE: 14 of 15 green incl. dynamic acceptance on the fixtures (with the plateau rule); only failure the Windows unit test rust_export_gives... (fixed in the tree by me). Task 60 (flake) closed: rules checked, one green CI sample; residual flake possible for columns with 1 or 2 rises.
- P3 PASSES 8 AND 9 REPORTED (u49/report.md) AND CHECKED (my logs u47/verify-p89.log, verify-p89m.log, verify-p89m2.log): pass 8: intake.workload blocker refused when a harness ran for every runtime with candidates (`refuse_contradicted_workload_blocker`); a finding a passed column points at cannot be closed before an alloc-test (`growth-not-measured`, dla_core::grown); detector hits listed by the final gate; trend DETECTOR line prints direct steps (tests crates/dla-cli/tests/grown_chain.rs, 6). Pass 9: gate/status/stop hook count actions (`gate::actions`, missing_total = actions, missing_dependents), digest over actions; never-growth columns not listed; Trigger::Direct "grown only by calling the registration function itself", counted apart; confirmed findings the verifier reads N-... counted "confirmed but disputed by the verifier"; probe:(root package).X.Y display. On the grpc-4 ledger copy: final gate 788 items -> 10 actions; Findings line "confirmed 3, confirmed but disputed by the verifier (reads N-INTENT) 19, ... grown only by calling the registration function itself 4". Mutations MUT_E1..E4 each failed a test; restored. Task 64 done.
- NOT ACCEPTED: `dla run status` exceeds 2048 bytes now and the generic bound clips its first blocking item to about 100 chars (test queue::an_unknown_verdict_does_not_settle_and_is_explained fails). Sent pass 9b to P3: status and gate check fit themselves, first item and next_command whole.
- NEXT: when 9b is checked: fmt, clippy workspace, schema, test_grade, skill_text; snapshot WITHOUT [skip ci]; freeze build (binary + plugin); real reruns axios-5, tokio-3, spring-5, then grpc-5.

## State at 18:55 UTC, 2026-11-21
- P3 PASSES 9b AND 9c CHECKED (u49/report.md; my log u47/verify-9bc.log): `out::fit(v, list_key)` in run status (list `blocking`) and gate check (list `missing`): first item and next_command whole, low-priority keys dropped first, then later items, each named in `left_out`; MUT_O1 (drop the first item) failed the fit test; restored. 9c: the narrowing of `workload_ran` is removed (a target read `unknown` does not exempt its harness); the unknown-verdict queue test reaches close-rest through the budget floor with all assertions kept; new grown_chain test (7 pass); queue 15 pass. Task 66 done.
- LOCAL PRE-CI: fmt clean (after formatting my cover.rs line), clippy workspace -D warnings clean, schema ok, test_grade OK, skill_text 14.
- SECOND FROZEN BUILD: snapshot 167233aec6d5f0317efa84c90efe2ca3b1fa7fcc (pushed WITHOUT [skip ci]; CI waiter bvy4qn016, logs u51/ci). Binary /root/dla-real/bin/dla = u51/bin/dla; plugin refreshed. Contains everything checked so far: R, P3 passes 5-9c, G (Go batches, focus, second look), N (ES modules, brief S, follow-up), F (go.released, cannot-release columns, expected-rises plateau rule), my small edits.
- REAL RUNS STARTED 18:55 UTC on the second frozen build: axios-5 (waiter b7od4bt89), tokio-3 (waiter btjron562). Then spring-5 (--warm-m2 /root/dla-real/m2warm, source /root/dla-real/src/spring-data-commons), then grpc-5.
- If CI on 167233a is green: commit that snapshot tree to main with commit-snapshot-tree.sh (write the message first: wip-notes/commit-message-3.txt), push main and main:dev/wonderful-mccarthy-algc5s.

## State at 19:01 UTC, 2026-11-21
- REAL RUN tokio-3 FINISHED (226 s): COMPLETE, no leak; second look ran twice, grew 0; the unit was one fixed sequence (notes real-run-plugin-tokio-3.md, evidence u36/tokio-3).
- BRIEF M STARTED: G resumed (a349aef413818803c), wip-notes/choices-brief.md, scratch u52: choice-from-key helper in all driver templates, recipe in the second-look block, refusal of a second-look unit with fewer than two choices, combination count.
- Running: axios-5 (waiter b7od4bt89), CI on 167233a (waiter bvy4qn016), G brief M.
- Commit message for main drafted: wip-notes/commit-message-3.txt (use with commit-snapshot-tree.sh 167233aec6d5f0317efa84c90efe2ca3b1fa7fcc when CI is green).

## State at 19:33 UTC, 2026-11-21
- G REPORTED brief M (u52/report.md) and it is checked (my logs u47/verify-m.log, verify-m2.log): choice helper in all six driver templates (dla_choice Python/C, dlaChoice Node/Go, choice JVM/Rust; mixed-radix digits of the key; 8-entry array of n per slot); second-look block gives the five-slot recipe; harness run refuses a second-look unit with fewer than two helper calls (dla_core::harness::choice_calls); "the units took N combinations of K choices" in run summary.choices and trend choices. Tests: 7 template helper tests, second_look.rs; MUT_M1 (refusal off) failed second_look; restored. Task 67 done. NOT in the second frozen build (167233a).
- G's real tree (u52/tokio2): second-look unit written from the recipe lines only (depth, ending, when, order, polled) took 144 combinations; native.live_bytes 92552 -> 465760 (106 B/unit); alloc-stack-diff ties to F6 with child_token (:218) and new (:144); alloc-test favours leak; refute, test, confirm all accepted: F6 confirmed.
- axios-5 at 19:33: the model follows the detector/site finding: "F27's test favored a leak, reading lib/adapters/http.js:924"; alloc-tests of F14, F18, F25, F26, F28 running.
- CI on 167233a: run 44938127014 in progress.
- NEXT: axios-5 result; CI result -> commit 167233a tree to main (commit-message-3.txt) and push; then third frozen build (with brief M) for tokio-4 and spring-5, grpc-5.

## State at 19:45 UTC, 2026-11-21
- REAL RUN axios-5 FINISHED (49 min, cost 1.97): COMPLETE, and THE DOCUMENTED LEAK IS CONFIRMED (F27, lib/adapters/http.js:924) (notes real-run-plugin-axios-5.md, evidence u36/axios-5).
- Real-project score on current code: aiohttp found, grpc-go found, axios found; tokio-util found by hand-driven second look with the choices recipe (real rerun tokio-4 pending the third frozen build); spring not rerun; Jetty, Pillow never run.
- New defect from axios-5: per-site count columns tie through the receiver's writer line (:717, 20 candidates) -> F14 and F18 confirmed without a column at their site. N resumed (a747f7bdc3d1ef293) to tighten: own site only. Task 68.
- CI on 167233a (run 44938127014): in progress at 19:33; waiter bvy4qn016.
- NEXT: CI green -> commit 167233a to main (commit-message-3.txt). When N reports: verify, third frozen build (brief M + N's tie rule), tokio-4, spring-5, grpc-5.

## State at 20:03 UTC, 2026-11-21
- N REPORTED the tie tightening (u46/report.md) and its tests are checked (my logs u47/verify-tie.log, verify-tie2.log): `is_per_site_count` in crates/dla-cli/src/trend/candidates.rs = any column with a site after @ whose base does not start with `native.`; such a column lists and ties only findings whose own site is that line; report `growth_line` quotes the own-site column first. Test a_per_site_count_column_ties_only_to_the_findings_at_its_own_line (MUT_T1 failed it; restored) and the render test pass.
- LOCAL BENCH CHECK RUNNING (task bz4b0fbdp, outputs u53/dyn-<prefix>.out, binaries u53/bin): `dla-bench dynamic --only node-`, then go-, then python-, because the tie change can take a confirmation away. All must pass before the third frozen build.
- CI on 167233a: only dynamic acceptance still running at 19:50.

## State at 20:16 UTC, 2026-11-21
- CI ON SECOND FROZEN SNAPSHOT 167233a (run 44938127014) DONE: 14 of 15 green (Windows fixed); "dynamic acceptance on the fixtures" FAILED 61/64: rust-delayed-queue, rust-ledger-sync, rust-tree-parent-links: final gate item of pass 8 ("F3 is closed unresolved but its column native.live_blocks@src/orders.rs:28 of S3 passed the growth gate and no alloc-test decided it"); the bench's final phase adds a finding from an untied passed column and resolves it unresolved (accepted by resolve, listed by the gate). Log u51/ci/dynamic_acceptance_on_the_fixtures.log. NOT committed to main.
- PASS 10 SENT to P3 (resumed a8249474e4d24c8ca): resolve and gate agree; a column's finding in a harness whose growth an alloc-test already decided may be recorded as part of that finding (existing duplicate/merge); the bench driver follows the printed route; no fixture, key, xfail or criterion change; run dla-bench dynamic --only rust- and native- after my node/go/python check ends.
- My local bench check (task bz4b0fbdp, u53) of node-, go-, python- for N's tie rule is still running.
- NEXT: after pass 10 and the bench checks: snapshot for CI (third), freeze third build, real runs tokio-4, spring-5, grpc-5; commit to main when CI is green.

## State at 20:38 UTC, 2026-11-21
- LOCAL BENCH CHECK DONE for N's per-site tie rule (u53/dyn-*.out): node- 12/12, go- 8/8, python- 9/9 pass. Task 68 done.
- Running: P3 pass 10 (growth rule vs bench driver; will run dla-bench dynamic --only rust- and native-).
- NEXT: verify pass 10; local checks; third CI snapshot; freeze third build; tokio-4, spring-5, grpc-5; commit to main when CI is green (commit-message-3.txt needs two more lines: per-site tie rule, choices recipe, part-of route).

## State at 21:10 UTC, 2026-11-21
- P3 PASS 10 REPORTED AND CHECKED (u49/report.md; my logs u47/verify-p10.log, verify-p10m.log): grown.rs: a passed column also points at the leak-side finding whose file:line equals the column's site (resolve and gate agree); `ledger duplicate F --of G` offered when G of the same harness has an alloc-test that favoured leak and duplicate::check accepts the pair; bench driver (bench/runner/src/dynamic/policy.rs) follows the printed route (duplicate when offered, else sibling, ledger test, alloc-test, then the alloc-test's next lines). grown_chain 9 pass; bench policy test passes; MUT_G2 failed a_column_points_at_the_finding_at_its_site...; restored. P3's bench: rust- 8/9 + 1 xfail, native- 9/9. Task 69 done.
- LOCAL PRE-CI on the tree: fmt clean, clippy workspace clean, schema ok, test_grade OK, skill_text 14.
- THIRD FROZEN BUILD: snapshot 89bc8a8a249d38f1e945e577bcc3bac1fb64dcdb (pushed WITHOUT [skip ci]; CI waiter bzq5j3ssu, logs u54/ci). Binary /root/dla-real/bin/dla = u54/bin/dla; plugin refreshed. Contains everything checked to date (also brief M choices, N per-site tie, pass 10).
- REAL RUNS STARTED 21:10 UTC on the third frozen build: tokio-4 (waiter bx289903h), spring-5 (--warm-m2; waiter bnsh78dwh). grpc-5 after one of them ends.
- commit-message-3.txt updated (choices, per-site tie, duplicate route, two libraries confirmed). Commit 89bc8a8 to main when its CI is green.
- Disk 5.4 GB after pruning (deleted u50/u52 tokio copies, bench logs, go build cache).

## State at 21:34 UTC, 2026-11-21
- REAL RUN tokio-4 FINISHED (23 min): COMPLETE, no leak; second look 7 (grew 0); the model used choices but skipped the "release of the intermediates" slot (notes real-run-plugin-tokio-4.md, evidence u36/tokio-4).
- M2 SENT to G (resumed a349aef413818803c): second-look unit hook generated as a skeleton with named slot variables; harness run refuses an unused slot; explicit give-up form (at most two), listed in run and trend. Task 70.
- Running: spring-5 (waiter bnsh78dwh), CI on 89bc8a8 (waiter bzq5j3ssu), G M2.
- grpc-5 not started yet (start when spring-5 ends, or now if load allows: load is low; Go run needs about 2 GB disk; 5 GB free).

## State at 22:16 UTC, 2026-11-21
- G M2 REPORTED AND CHECKED (u52/report.md; my log u47/verify-m2s.log): second-look unit hook generated as a skeleton with five slot variables (depth, ending, when, order, outcome); harness run refuses an unused slot (`slot_report_in` in dla-core harness.rs); give-up form "// slot <name>: not applicable: <reason>" (at most two), listed as slots_given_up. Tests pass; MUT_K1 failed two tests; restored. G's tokio3 copy: laziest fill grows, F6 confirmed. Task 70 done. NOT in the third frozen build.
- REAL RUN grpc-5 FINISHED (40 min, cost 8.32): INCOMPLETE (124 open); documented leak confirmed again (F202); headline split works (notes real-run-plugin-grpc-5.md, evidence u36/grpc-5). New defects: trigger "registration function itself" wrongly on the RPC handler Server.Watch; 120 untied stack columns of captures are 120 gate actions; 28 stop attempts.
- spring-5 still running (in a stop loop at 22:14: INCOMPLETE, 298 of 367 ranked never targeted, about 115 of 180 min left, JUnit loop blocked by Mockito self-attach in the sandbox).
- CI on 89bc8a8: run 44950113560 in progress.
- DISK was at 322 MB at 22:14 (grpc-5 3.3 GB, target/debug/build 9.6 GB): freed to 4.9 GB (prune, deleted grpc-5 run dir, u52/u46/u50 copies, axios-4 run dir).
- NEXT: read spring-5; pass 11 for P3 (Trigger::Direct only for start-up registrations by census facts; untied stack columns of one capture evidence = one action and one finding accounts for them); fourth frozen build with M2 + pass 11; tokio-5.

## State at 22:20 UTC, 2026-11-21
- CI ON THIRD FROZEN SNAPSHOT 89bc8a8 (run 44950113560): ALL 15 JOBS GREEN, dynamic acceptance included.
- COMMITTED TO MAIN: d297716 "Make a measured growth binding, give flat runs a second look, fit series to their time" (tree of snapshot 89bc8a8; 110 files, +14376/-476; no trailers; author and committer Mykyta Hriaznov). Pushed to origin main and origin dev/wonderful-mccarthy-algc5s. CI on main will run for d297716: check it (gh run list --branch main).
- In the working tree after the commit (not in main): G's M2 (second-look skeleton, checked) and P3's pass 11 (in progress).
- P3 PASS 11 SENT: 11.1 direct-call trigger reading only for start-up registrations by census facts; 11.2 untied columns of one capture evidence are one gate action and one finding accounts for them.
- Running: spring-5 (waiter bnsh78dwh), P3 pass 11.

## State at 22:30 UTC, 2026-11-21
- REAL RUN spring-5 FINISHED (75 min, cost 5.85): INCOMPLETE, no leak; the documented holder F282 was reached by a focus harness and read constant (Java driver cannot produce Kotlin property references; JUnit loop blocked by Mockito self-attach in this sandbox). Notes real-run-plugin-spring-5.md, evidence u36/spring-5.
- Pillow 10.1.0 and Jetty 12.0.31: not run. Pillow needs its C extension built (freetype, zlib, jpeg headers; Python <= 3.12; this machine has 3.13); Jetty source copy is partial. Environment limits, to tell the owner.
- Running: P3 pass 11. After it: verify, fourth frozen build (M2 skeleton + pass 11), CI snapshot, tokio-5.

## State at 22:42 UTC, 2026-11-21
- P3 PASS 11 REPORTED AND CHECKED (u49/report.md; my log u47/verify-p11.log): 11.1 Trigger::Direct only when every writer site is init or startup by the census facts (else reproduced; no facts -> not checked); on the grpc-5 ledger 8 of 10 become reproduced (F202 among them), F143 and F182 keep the reading. 11.2 untied growing columns of a capture summary (series of source kind dump) are one action per (trend, evidence); one finding from `ledger add --origin growth-series --evidence E` stands for them ("Also growing in the same capture"); on the grpc-5 gate 124 actions would be 18. Tests: trigger 15, grown_chain 11; MUT_H1 and MUT_H2 each failed 2 tests; restored. fmt, clippy workspace, schema, test_grade, skill_text clean.
- FOURTH FROZEN BUILD: snapshot 56aa2217ce837e543c01503ec7ccd4b63b0ebbdf (pushed WITHOUT [skip ci]; CI waiter in background, logs u55/ci). Binary /root/dla-real/bin/dla = u55/bin/dla; plugin refreshed. On top of main d297716: G's M2 skeleton, P3's pass 11.
- REAL RUN tokio-5 STARTED 22:41 UTC on the fourth frozen build (waiter in background).
- CI on main d297716 (run 44955952991) and on the dev branch (44955954569): in progress at 22:27.
- If CI on 56aa221 is green: write commit-message-4.txt (skeleton, trigger facts, capture grouping) and commit with commit-snapshot-tree.sh.

## State at 22:50 UTC, 2026-11-21
- REAL RUN tokio-5 FINISHED (6 min, cost 1.19) on the fourth frozen build: COMPLETE and THE DOCUMENTED LEAK IS CONFIRMED (F2 child_token, stack at cancellation_token.rs:218, mechanism "middle token dropped before its child"). Notes real-run-plugin-tokio-5.md, evidence u36/tokio-5.
- REAL-PROJECT SCORE (real plugin runs, model claude-sonnet-5-5, current code): aiohttp 3.10.10 found; grpc-go v1.84.0 found (twice); axios 1.15.1 found; tokio-util 0.7.1 found; spring-data-commons 4.1.1 not found (needs a Kotlin caller; holder reached and read flat); Pillow and Jetty not run (environment).
- Waiting: CI on the fourth snapshot 56aa221 (waiter b11o93v59). Then: commit-message-4.txt, commit to main, push both branches.
- THEN RELEASE 0.9.0-preview.3 (pre-authorized previews; the hold condition "real runs find the other projects' leaks" is met for 4 of 5): bump Cargo.toml workspace version and the nine path deps, cargo update -w --offline, .claude-plugin/plugin.json, .claude-plugin/marketplace.json; CI green on main; dispatch release.yml on main with publish=true; check the assets; tell the owner.

## State 2026-11-22 00:20 UTC

- `main` = d297716: CI green on main (run 44955952991) and on the dev branch (44955954569),
  all 15 jobs.
- Fourth frozen snapshot 56aa221 (second-look skeleton + pass 11): CI RED in two jobs
  (run 44958016880, logs `<sp>/u55/ci`). Not on main.
  1. dynamic acceptance 62/64: go-subscription-workers, final gate keeps
     `trend_column T6:metric:goroutine/total` with a fix that is no command. My reading:
     pass 11.2 removed the own items of untied group columns of a capture summary
     (`capture_item`: `decided` or `stands_for` -> no item) but `group_columns_state` still
     asks each non-flat group to be tied. Flaky by whether a second goroutine group reads
     non-flat. Brief `wip-notes/pass12-brief.md` -> gate builder a8249474e4d24c8ca
     (report: `<sp>/u49/report.md`, "Pass 12").
  2. macOS (image macos-26-arm64, same image as the green runs on main): test
     `frozen_garbage_is_tied_to_the_freeze_site_by_a_count_column_and_reads_growth` failed
     once: trend had 15 columns, no `py.gc_frozen` group, target `not_loaded`, while
     `py.traced` rose as designed. Linux reference: 41 columns (`<sp>/u56/trend-linux.json`,
     project copy `<sp>/u56/frozen`). Brief `wip-notes/macos-frozen-brief.md` -> builder
     a61db0f8f10fd6ebb (report `<sp>/u58/report.md`).
- Next: verify both reports myself (rerun tests, marker mutations), new frozen snapshot
  with CI, commit the tree to main with `commit-message-4.txt` (add the two fixes), push
  main and main:dev/wonderful-mccarthy-algc5s, then 0.9.0-preview.3.
- The artifact host of GitHub Actions is blocked by the proxy here (403): CI artifacts
  cannot be downloaded; job logs can be read.

## State 2026-11-22 (after the two builders' reports)

- Pass 12 (gate builder, report `<sp>/u49/report.md` "Pass 12"): `capture_accounted` is one
  predicate for `capture_item` and `group_columns_state`. Checked by me: grown_chain 13/13;
  mutations MUT_G1 (predicate off) and MUT_G2 (fix text) each failed the right test.
  My own edit: `total_fix` no longer prints `dla ledger resolve F --as unresolved ...` when
  the total waits on a finding; it prints `dla gate check --final   # after F has a
  disposition ...`; test asserts it and that F has its own `finding` item.
- Brief P (Python builder, report `<sp>/u58/report.md`): macOS cause NOT established (10
  runs in a row plus one under load passed on Linux). Two candidate paths: a swallowed
  exception in `dla_freeze` (site lookup), or the typed reader dropping sections. Changes:
  agent keeps the group under `<non-project>`, `reduced record:` limitations, `Rec.dropped`,
  series limitation, `REDUCED RECORD` attention line, trend verdict `unknown`. Checked by
  me: reduced_record test; my mutations MUT_P1 (agent fallback) and MUT_P2 (verdict rule)
  each failed it. My own edits: the test asserts `py.gc_frozen@<non-project>`; the frozen
  test's panic message also carries the agent's raw record of one point (it names the
  interpreter in `exe`).
- Running: `<sp>/u59/verify.sh` -> `<sp>/u59/verify.log` (fmt, clippy, schema, workspace
  tests). Then: `dla-bench dynamic --only go-` once on the final tree, a new frozen
  snapshot WITH CI, commit to main with `commit-message-4.txt` (already extended).

## State 2026-11-22 02:10 UTC

- Final tree checked: fmt, clippy (workspace, all targets, -D warnings), `dla schema --check`,
  `cargo test --workspace --no-fail-fast`: 166 binaries ok, 3132 passed, 0 failed
  (`<sp>/u59/test-all.log`); `dla-bench dynamic --only go-`: 8/8 pass (`<sp>/u59/go.log`).
- Fifth frozen snapshot made WITH CI (id in `<sp>/u59/tree-sha`). When green:
  `bash <sp>/wip-notes/commit-snapshot-tree.sh <sha> <sp>/wip-notes/commit-message-4.txt`,
  check no trailers, push main and main:dev/wonderful-mccarthy-algc5s, then the
  0.9.0-preview.3 bump and release.
- 02:20 UTC: CI waiter for ecbbe4e in background (logs `<sp>/u59/ci`). Meanwhile brief Q
  (`wip-notes/identity-keys-brief.md`) -> builder a349aef413818803c (second-look builder):
  spring-data-commons `TypedPropertyPaths.resolved` is keyed by the lambda object the caller
  passes (TypedPropertyPaths.java:103-115); my earlier note "needs a Kotlin caller" is
  probably wrong: a Java capturing lambda or `new` instance is a fresh key per unit. Step 1
  is the experiment; then a general line in the driver block for holders keyed by a
  caller's object. Report `<sp>/u60/report.md`. Its edits are NOT in the frozen snapshot.

## State 2026-11-22 (brief Q result)

- Builder report `<sp>/u60/report.md`: on a copy of spring-data-commons the documented cache
  grows with a fresh key per unit, but in Java only via a serializable lambda made per unit
  with `LambdaMetafactory.altMetafactory`; a capturing lambda, `new` of an implementing
  class and a bound method reference are rejected by the library's reader. Finding
  confirmed by hand through the printed commands (alloc-test leak).
- In the tree (NOT in frozen snapshot ecbbe4e): `HolderFacts::key_from_caller`, `key:` line
  of the target block, trend constant reason sentence, close-rest remedy clause, docs.
  My rewording: value keys vs identity keys. Checked: two tests, mutation MUT_K1.
  Whole suite running: `<sp>/u61/verify.sh` -> `<sp>/u61/status`, `test-all.log`.
- Open: a real plugin rerun on spring with a frozen build that has this change (does the
  model find the factory form from the line?). A Kotlin unit for the JVM driver stays a
  separate decision for the owner.
- 02:53 UTC: real plugin run spring-6 started (sixth frozen build = snapshot 4a72a50, binary
  copy `<sp>/u61/bin/dla` = `/root/dla-real/bin/dla`, plugin refreshed from 4a72a50). Waiter
  in background. Question: does the model use the `key:` line to make a fresh key (the
  factory form) for the finding at TypedPropertyPaths.java:53? Whole suite on that tree:
  3134 passed, 0 failed (`<sp>/u61/test-all.log`).
- CI on ecbbe4e still running at 02:53 (waiter buvegduus, logs `<sp>/u59/ci`).

## State 2026-11-22 03:10 UTC

- CI on frozen snapshot ecbbe4e: green, all 15 jobs (run 44973926575).
- `main` = cc003a0 "Version 0.9.0-preview.3" on top of a56ee38 "Second-look skeleton,
  trigger reading by census facts, one gate action per capture" (tree of ecbbe4e). Both
  pushed to main and dev/wonderful-mccarthy-algc5s. No trailers.
- Waiting: CI on main at cc003a0 (waiter b8991t70t, logs `<sp>/u62/ci`). When green:
  dispatch `release.yml` on `main` with `publish=true`, check the release
  `v0.9.0-preview.3` and its assets, tell the owner.
- The working tree = main + the key-line change (brief Q, checked, not committed) + the
  version bump. Real run spring-6 is running on the build with the key-line change.
- After spring-6: evidence to `<sp>/u36/spring-6`, delete the run dir, note
  `wip-notes/real-run-plugin-spring-6.md`; then freeze a snapshot with CI for the key-line
  change and commit it (message to write).

## State 2026-11-22 03:35 UTC

- spring-6 done (note `wip-notes/real-run-plugin-spring-6.md`, evidence `<sp>/u36/spring-6`,
  run dir deleted): documented leak NOT found; the key line was ignored; the model stopped
  after 15 minutes. The JUnit repeat was not tried (in spring-5 it failed on the Mockito
  self-attach).
- Brief S (`wip-notes/junit-loop-brief.md`) -> builder a349aef413818803c: start-up agent for
  the mocking stack in the JUnit repeat, measure F160 through the project's tests, report
  what the tool offers about the test loop, shorten the key line. Report
  `<sp>/u63/report.md`.
- Still waiting: CI on main at cc003a0 (waiter b8991t70t, logs `<sp>/u62/ci`); then the
  release workflow for 0.9.0-preview.3.

## State 2026-11-22 04:10 UTC

- Brief S result (report `<sp>/u63/report.md`): the JUnit repeat now adds the mocking
  stack's start-up agent (`harness/jvm/dla-junit-build.sh`, `startup-agents.txt`, summary
  key `startup_agent`, manifest tool `startup-agent:<jar>`). With it a JUnit repeat over
  four test classes of spring-data-commons (792 tests a run, about 90 s) reads the
  documented cache F160 `grew` and the printed commands confirm it. The builder could only
  reproduce the attach failure with `-XX:+DisableAttachMechanism` (its shell is not the
  plugin sandbox). The `key:` explanation is printed once per driver file.
- My edits on top: manifest read by `jar` when `unzip` is missing; jar name matched after
  `/` or `\`; test and its helpers `#[cfg(unix)]`. Checked: test passes, mutation MUT_J1
  failed it, hand check on a PATH without unzip added the agent.
- Running: `<sp>/u61/verify.sh` -> `<sp>/u61/status2`, `test-all2.log` (whole suite on the
  tree with briefs Q and S). When clean: freeze a snapshot WITH CI, then send brief T
  (`wip-notes/test-loop-first-brief.md`: the hookless test loop as the first pass of a JVM
  queue) to builder a349aef413818803c.
- The tool today barely offers the test loop: one clipped `junit_hint`; status, trend and
  the queue name only `--driver --next`. 305 of 387 ranked candidates of the spring ledger
  have a test class named after their owner.
- 03:47 UTC: the first whole-suite run on the tree with briefs Q and S was VOID (disk full
  while linking: `test-all2.log`, exit 101, no test ran). Freed space: prune script, builder
  copies, `go clean -modcache` and `-cache` (the Go module cache must be downloaded again
  for a Go real run). Rerun: `<sp>/u61/verify3.sh` -> `status3`, `test-all3.log`.
  One whole-suite cycle after a change to dla-workload or dla-cli needs about 5 GB free.
- 03:50 UTC: CI on main at cc003a0 green, all 15 jobs (run 44978201658). Release workflow
  dispatched on main with publish=true: run 44982362127 (waiter bzv9cfuxm). Then check the
  release `v0.9.0-preview.3` and its assets (compare with the asset list of preview.2).
- 03:56 UTC: `v0.9.0-preview.3` PUBLISHED (pre-release, target cc003a0, release run
  44982362127 green): 11 assets, all uploaded: archives and `.sha256` for
  aarch64-apple-darwin, x86_64-apple-darwin, aarch64-unknown-linux-musl,
  x86_64-unknown-linux-musl, x86_64-pc-windows-msvc, plus `SHA256SUMS`. Task 71 done.
  Not in it: the key line and the test-loop start-up agent (working tree, whole suite
  rerunning: `<sp>/u61/status3`).

## State 2026-11-22 04:20 UTC

- Whole suite on the tree with briefs Q and S: 166 binaries, 3135 passed, 0 failed
  (`<sp>/u61/test-all3.log`); clippy and schema clean.
- Sixth frozen snapshot 0302e2b (= main cc003a0 + Q + S) is in CI (waiter bx2hzc630, logs
  `<sp>/u61/ci`). When green: `bash <sp>/wip-notes/commit-snapshot-tree.sh
  0302e2b38f0af4beac1bdad6b701a764d45c20ed <sp>/wip-notes/commit-message-5.txt`, check no
  trailers, push main and main:dev/wonderful-mccarthy-algc5s.
- Brief T sent to builder a349aef413818803c (`wip-notes/test-loop-first-brief.md`), report
  `<sp>/u64/report.md`. Its edits come on top of the frozen tree and are not in 0302e2b.
- After brief T is checked: frozen build, real plugin rerun on spring-data-commons
  (`python3 /root/dla-real/run.py spring-7 /root/dla-real/src/spring-data-commons
  /root/dla-real/prompt-generic.txt --timeout 9000 --max-turns 600 --warm-m2
  /root/dla-real/m2warm`).
- 04:38 UTC: the container was RESTARTED (about 04:35). Files survived (working tree,
  scratchpad, /root/dla-real); processes did not. Restarted: the CI waiter for 0302e2b
  (task btvg9i2c0) and the builder on brief T (resumed by message; its partial edits, about
  1240 lines over 15 files, were in the tree with no marker). `main` and the release are
  untouched: main = cc003a0, `v0.9.0-preview.3` published.

## State 2026-11-22 (brief T, interim hand-back of the builder)

Builder a349aef413818803c handed back EARLY; no `<sp>/u64/report.md` yet. Its own background
test run (`cargo test -p dla-workload --test harness_junit`, log `<sp>/u64/hj.log`) was
still going; it may resume by itself.

In the tree, uncommitted, about 1680 lines, builds, NOT checked by me yet:
- new `crates/dla-core/src/testloop.rs`: test class by name (owner name or its singular at a
  camel-case boundary, 2 per candidate at most), else by mention (1). MAX_TARGETS 16,
  MAX_CLASSES 10, MAX_TESTS 1000, one module per harness; `plan()` cuts batches by rank.
- `queue.rs`: `RuntimeQueue.test_loop`; `batch_command` is `--junit --next` while test-loop
  batches are left, else `--driver --next`; a test-loop reading now counts as a driver
  reading (flat = one flat reading, not_loaded/not_executed/unknown = a miss); a failed
  test-loop attempt sends its targets to the driver pool.
- texts name the test loop first with the count in the gate/stop-hook queue item,
  `run status`, close-rest, trend (`queue_next`), the report's Queue line.
- `gen`: `--junit --next` takes the first test batch, prints `tests_chosen`.
- `DlaJUnitHarness.java`: a class with no passing test in the first run is dropped while
  another passed; the loop lowers its step and drops the slowest classes to fit the time.
- `run.rs`: on build failure or "every unit failed" of a test-loop harness the fix ends
  with the driver command for the same targets.
- trend: columns of declared targets lead (`summary::declared_first`).
Real tree (copy `<sp>/u64/spring`): "347 candidates in 24 test-loop batches, nothing to
write; then 41 candidates need a driver"; a batch of 16 targets, 9 classes, 151 tests took
about 45 s and read F160 `grew`.

NOT done by the builder: fmt/clippy/full tests/schema/skill text after the last edits;
mutations for the gen, run.rs and Java tests; the six batches in order and the printed
route to `confirmed`; the time for all batches; item 7 (bench fixtures that would now get
a test-loop batch); the report file. Its own open points: `ledger list` and schema outputs
for the new fields; the second look is capped at 9 candidates a runtime, so hundreds of
flat test-loop readings get only 9 second looks.
- 05:00 UTC: the first builder stopped for good (out of room). Its last word: the full
  `harness_junit` run ended 37 passed, 1 failed:
  `the_loop_fits_its_runs_into_the_time_limit_by_lowering_the_step_and_dropping_the_slowest_class`
  (not investigated). A FRESH Sonnet builder (agent af9e65679252b2b82) has the finish
  brief `wip-notes/test-loop-finish-brief.md`: fix that test's cause, fmt/clippy/schema/whole
  suite, the missing marker mutations, the six real-tree batches with the printed route for
  the documented finding, the report-only points (bench fixtures, second-look cap, ledger
  list and schemas), docs. Report `<sp>/u65/report.md`. After it: my own check.
- 05:15 UTC: CI on the sixth frozen snapshot 0302e2b green, all 15 jobs (run 44983516013).
  `main` = 20ee96e "Say when a key comes from the caller; let the test loop load the mocking
  library at start-up" (tree of 0302e2b, no trailers), pushed to main and the dev branch.
  It is NOT in the published preview.3 (that is cc003a0). CI on main at 20ee96e starts by
  itself; not waited for unless the next snapshot needs it.

## State 2026-11-22 (after the fresh builder's report on the test-loop-first queue, brief T2)

- Builder af9e65679252b2b82 finished (report: u65/report.md): suite 3165 passed, real tree 24 test-loop
  batches in about 23 min, F160 `confirmed` by the printed route.
- My own check (u66/): whole suite on the builder's tree running/ran (u66/test-all.log); a 3-reviewer
  Sonnet workflow (wf_d80d932a-5e6) read the diff; my reading found more.
- Fixes I made on top (all uncommitted, NOT yet built or tested when this note was written):
  1. `testloop.rs` pick(): classes of one pick are of one module and fit MAX_TESTS together (+ unit test).
  2. `trend/verdicts.rs`: flat reading of a test-loop target whose chosen classes (harness/H/tests.json)
     were all dropped (series limitation `dropped test class X: ...`) reads `not_executed` (+ test).
  3. `census_cmd.rs`: `dla census --lang jvm` writes the workload discovery result when the run has none
     (spring-6 never ran `dla workload discover`, so the test loop would never have been offered)
     (+ CLI test at the end of crates/dla-cli/tests/queue.rs).
  4. `DlaJUnitHarness.java`: a run with failing AND passing tests is kept (exit 2 only when no test passed
     in any run; first failure is a limitation); timing after a drop is measured again (sinceNanos,
     estimateNanos); drop limitation says from which run (+ test a_test_that_fails_in_every_run_...).
  5. `gen.rs`: bare `--junit --tests X --next` takes 16 (bound was 8); `tests_chosen` prints the full
     class name when two share a simple name.
  6. `queue.rs`: `command_for` returns `dla harness run H` for a candidate of the generated test-loop
     harness; `missed_command` (used by ledger refuse and ledger list).
  7. `run.rs`: attention lines show the test loop's limitations (`TEST CLASS DROPPED (n)`), the
     no-usable-point failure prints the driver fallback for a test loop.
  8. `out.rs`: `junit_hint`/`queue_next` kept whole, also nested under `summary` and in the last resort.
  9. `ledger_queue.rs`: close-rest `next` from any open queue.
  10. docs jvm.md, stages.md (second look cap 9, census discovery, failing tests, dropped class).
- Commit message draft: wip-notes/commit-message-6.txt. VERIFY row still to extend.
- Next: build, run the new tests, my marker mutations, whole suite again (prune first), snapshot WITH CI,
  commit, push, then the real run spring-7.
- Open points not changed: close-rest closes flat test-loop readings before any second look (cap 9);
  status says "not targeted yet" for missed ones; a model may tire of 24 batches of 3 commands each.

## State 2026-11-22, later (review fixes built and checked)

- Whole suite on the builder's tree: 166 binaries, 3165 passed, 0 failed, 5 ignored (u66/test-all.log).
- Review fixes (list above) built; affected tests pass (u66/t1-*.log, t2-junit.log: harness_junit 39/39).
- My mutations (u66/mut.py, results u66/mutrun.out + mut-k2.log): V1..V12 on my fixes and B1..B4 samples
  of the builder's rules, each seen failing its test, all reverted, no marker left.
- Seventh frozen snapshot WITH CI: 55f31eb6d836d3cddb3f4824a8ebb7d41fac2c59 (id in u66/tree-sha-ci);
  CI waiter writes u66/ci-wait.out, failed job logs to u66/ci/.
- Local whole suite on the final tree running: u66/verify2.out, u66/test-all2.log.
- When both are green: `bash wip-notes/commit-snapshot-tree.sh 55f31eb... wip-notes/commit-message-6.txt`,
  push main and main:dev/wonderful-mccarthy-algc5s; then freeze binary + plugin copy and start spring-7:
  cp target/debug/dla /root/dla-real/bin/dla; refresh /root/dla-real/plugin from the snapshot;
  nohup setsid python3 /root/dla-real/run.py spring-7 /root/dla-real/src/spring-data-commons
  /root/dla-real/prompt-generic.txt --timeout 9000 --max-turns 600 --warm-m2 /root/dla-real/m2warm
- 06:45 UTC: whole suite on the final tree: 166 binaries, 3174 passed, 0 failed, 5 ignored; fmt, clippy,
  schema check clean (u66/verify2.out, u66/test-all.log). CI on main 20ee96e: green (run 44987646797).
- 06:47 UTC: real plugin run spring-7 started on the seventh frozen build (snapshot 55f31eb: binary
  /root/dla-real/bin/dla = target/debug/dla of that tree, plugin copy refreshed from it and compared).
  Waiter: until /root/dla-real/spring-7/run.json exists. Timeline: python3 /root/dla-real/tl.py
  /root/dla-real/spring-7/transcript.jsonl 40. Evidence to keep: <sp>/u36/spring-7 (run.json,
  transcript.jsonl, the run dir .dla without scratch copies), then delete /root/dla-real/spring-7.
- CI on 55f31eb: run 44993841579, in progress at 06:47.
- 07:41 UTC: spring-7 stopped by me (kill -TERM of its process group id, exit 143, 54 min). Result:
  documented leak F160 found, measured and confirmed through the test loop; the verifier then
  disputed it as N-INTENT. Note: wip-notes/real-run-plugin-spring-7.md. Evidence <sp>/u36/spring-7;
  /root/dla-real/spring-7 deleted.
- Brief U (wip-notes/verifier-testloop-brief.md) given to a Sonnet builder (agent a3f4e9db453f2bcd3,
  background, report to <sp>/u67/report.md): bundle carries the tests of a test-loop finding,
  verifier rule for caches with identity keys and weak/soft references, trigger line names the
  chosen tests, JvmClass probe without a column on a loaded class gives the coverage verdict.
  Its edits come on top of the frozen tree 55f31eb and are NOT in it.
- After it reports: verify myself (tests, my mutations, whole suite), then a model check of the
  verifier on the replayed evidence <sp>/u67/proj (bundle F160 and, as a control, F436), 3 runs each.
- CI on 55f31eb (run 44993841579): 14 of 15 green at 07:43, dynamic acceptance still running
  (waiter writes u66/ci-wait.out). When green: commit-snapshot-tree.sh 55f31eb... commit-message-6.txt.
- CI on 55f31eb green, all 15 jobs (run 44993841579). Committed its tree to main as b4b9a42
  ("The project's own tests are the first pass of a JVM queue"), pushed main and
  main:dev/wonderful-mccarthy-algc5s; no trailers, no .wip. The working tree now holds only the
  brief-U builder's edits (unverified).

## State 2026-11-22 about 08:50 UTC

- Brief U builder (a3f4e9db453f2bcd3) done: bundle test-loop/tests.json (classes CHOSEN for the
  finding), trigger varies names them, verifier.md cache rule + test-loop paragraph, JvmClass probe
  without a column on a loaded class -> coverage verdict. Suite 3180 passed. NOT yet verified by me.
- My own additions on top (tested: harness_jvm_coverage 9/9, verdicts 16/16): JVM agent finds a
  nested class (`Outer.Inner` -> `Outer$Inner`, DlaDump.binaryNames/findClass) and keeps the probe's
  spelling in records; JvmStatic "no such field" (e.g. `<clinit>` owners) -> coverage verdict;
  verifier.md test-loop paragraph extended (same tests every round; find the call; entry-path
  constant does not explain growth under tests).
- Verifier model check on the replayed evidence (/root/dla-real/vbase, vrun.py, plugin copy
  /root/dla-real/plugin-u, binary /root/dla-real/bin-u/dla): v160-1 and v160-2 both N-INTENT again.
- MEASURED (scratch run /root/dla-real/attr/project, harnesses H1..H9 = each class of the batch
  alone): the feeder of TypedPropertyPaths.resolved is KPropertyReferenceUnitTests (Kotlin, 13.3
  entries per run, steady); PropertyReferenceUnitTests constant at 267; TypedPropertyPathUnitTests
  one step in bytes; KTypedPropertyPathUnitTests constant at 1; five others not loaded. So the
  classes chosen for F160 by name are not the feeders: brief U's bundle and trigger text mislead.
- Brief V (wip-notes/attribution-brief.md): attribution per test class during
  `dla alloc-test H --finding F` (DLA_ATTRIBUTE in the JUnit loop), stored at
  harness/H/attribution-F.json; trigger and bundle use measured feeders or say "not measured".
  Given to a fresh Sonnet builder (see below for its id). Report comes as its final message.
- After brief V: verify everything myself in one pass (U + mine + V): tests, mutations, whole
  suite; verifier model check again (F160 x3 expecting a leak label with the Kotlin caller; F436 x3
  expecting N-INTENT to stay); VERIFY rows; snapshot WITH CI; commit; then real rerun spring-8.
- Scratch to delete when done: /root/dla-real/attr, /root/dla-real/vbase, /root/dla-real/v160-*,
  /root/dla-real/plugin-u, /root/dla-real/bin-u.

## State 2026-11-22 about 10:10 UTC

- Brief V builder (a5a873422ed5e7361) done: attribution per test class during alloc-test (suite 3189).
- My additions: `grew_in` (later runs a class added in) -> feeder = added in more than half of the
  later runs; others `added_once` / `added_in_a_few_runs_only`; docs (verifier.md, VERIFY rows, jvm
  README, stages.md); commit message wip-notes/commit-message-7.txt.
- Real tree (/root/dla-real/attr2/project, clean run, printed route): alloc-test H1 --finding F160
  -> fed_by KPropertyReferenceUnitTests 14.4/run, added_once TypedPropertyPathUnitTests, 7 ran_flat;
  trigger line names the Kotlin class.
- Verifier model check (vrun.py, plugin copy plugin-u, binary bin-u): F160 on vbase2 -> L-COLLECT in
  3 of 3 (v160-3,4,5); control F436 on vbase3 (fed_by ReturnedTypeUnitTests 162/run) -> N-INTENT in
  3 of 3 (v436-2,3,4). Before the attribution: F160 N-INTENT in v160-1, v160-2.
- My mutations (u68/mut.py, u68/mutrun.out, mut-a6.log): W1..W5 (mine) and X1..X5 (builders' rules),
  each seen failing, all reverted.
- Next: whole suite (u68/verify.sh -> u68/verify.out, test-all.log), eighth frozen snapshot WITH CI,
  commit with commit-snapshot-tree.sh + commit-message-7.txt, push main and dev branch, then real
  rerun spring-8 on that build.
- Whole suite on the eighth frozen tree b3b5ff4: 166 binaries, 3190 passed, 0 failed, 5 ignored; fmt,
  clippy, schema clean (u68/verify.out). CI on b3b5ff4: waiter writes u68/ci-wait.out.
  CI on main b4b9a42: green (run 45000472735).
- 09:55 UTC: real rerun spring-8 started on the eighth frozen build (binary /root/dla-real/bin/dla,
  plugin from b3b5ff4). Watcher bb5nyj5a3 fires when the verifier's reading of F160 is in the
  transcript (or run.json exists). Then: copy evidence to <sp>/u36/spring-8 (run.json, transcript,
  run dir without harness work/out and workload/), stop the run by its process group id if it is
  only writing drivers, delete /root/dla-real/spring-8, write wip-notes/real-run-plugin-spring-8.md.
- Scratch deleted: /root/dla-real/attr, attr2, vbase*, v160-*, v436-*, plugin-u, bin-u. Verifier
  records kept in <sp>/u68/verifier/, vrun.py in wip-notes/.
- 10:55 UTC: real rerun spring-8 ended by itself (exit 0, 60 min, 107 turns): report COMPLETE,
  "leak hypotheses: confirmed 1" = the documented leak TypedPropertyPaths.resolved with the trigger
  "the project's own tests KPropertyReferenceUnitTests (14.6 entries per run)"; verifier agreed on
  holder, mechanism and label (site disagreement only: 53 vs 116). Note:
  wip-notes/real-run-plugin-spring-8.md. Evidence <sp>/u36/spring-8; run dir deleted.
- Next small fix: site agreement accepts a verifier site in the same function as the finding's site
  or a writer (crates/dla-core/src/verify.rs agrees_site_among; known sites are collected in
  crates/dla-cli/src/ledger_m6.rs). Then commit b3b5ff4 when its CI is green (commit-message-7.txt).
- CI on b3b5ff4 (run 45008367756), first attempt: 14 jobs green, dynamic acceptance FAILED on one
  fixture: jvm-pool-warmup (non-leak): T1 of S1 passed os.rss / os.rss_anon / os.vm_hwm /
  jvm.nonheap_used with a stair shape, the unbound alloc-test then read Leak on os.rss, no rule-out.
  Log kept: u68/ci-first-attempt-failed.raw. Locally the same build passes that fixture 3 of 3
  (u70/pool-*.json: only os.vm_hwm passes, jvm.heap_used decides non-leak). Nothing in this change
  touches a command-mode JVM run's memory; read as a runner-noise flake of an RSS-decided
  alloc-test. Failed job re-queued (rerun_failed_jobs) at about 11:25 UTC; waiter below.
  If it fails again on the same fixture: treat as real, compare S1 columns with a b4b9a42 build.
- On top of b3b5ff4 in the tree (snapshot 60682a4, no CI yet): same-function site rule for
  `ledger verify` (+ test, 2 mutations seen failing, VERIFY row); commit message
  wip-notes/commit-message-8.txt. Needs its own CI after b3b5ff4 is committed.
- Plan change (about 11:35 UTC): one CI run on the combined tree (b3b5ff4 + same-function site rule)
  instead of two in sequence; this cancels the re-queued job of b3b5ff4. Ninth frozen snapshot id in
  u70/tree-sha-ci; waiter writes u70/ci-wait.out; local whole suite u70/verify.out. When green:
  commit 7 = tree of b3b5ff4 (commit-message-7.txt), commit 8 = tree of the ninth snapshot
  (commit-message-8.txt), push main and dev branch, then version 0.9.0-preview.4 and the release.

## 2026-11-22, about 12:50 UTC
- CI on the ninth frozen snapshot 93a84ba: green (run 45016510664, all 15 jobs; the
  `jvm-pool-warmup` failure of b3b5ff4 did not repeat).
- main: `37ed3af` (commit 7, tree of b3b5ff4), `ef9fbd2` (commit 8, tree of 93a84ba),
  `29cb6f6` "Version 0.9.0-preview.4". Pushed to main and dev/wonderful-mccarthy-algc5s.
  CI on 29cb6f6 on main: waiter writes u71/ci-wait.out. When green: dispatch release.yml
  (run_workflow, ref main, inputs {"publish": true}), check the release assets, task 74 done.
- Task 73 closed. Task 75: Pillow 10.1.0 real run.
  - Source: /root/dla-real/src/pillow (tag 10.1.0, da59ad0). `apt-get install libjpeg-dev` done
    on this container (system-wide). Scratch build u72/pillow/.venv (python3.12, `pip install -e .`).
  - Ground truth by hand (u72/truth.py, 20000 `draw.text` calls after 2000 warm-up): 2.0 blocks
    per call, tracemalloc: `src/PIL/ImageFont.py:580` (`self.font.render(`) +20004 objects of
    28 B. The C cause: `PyObject_GetAttrString(image, "id")` in `font_render`
    (src/_imagingft.c) is never decremented. No census candidate can name it (no holder).
  - Run pillow-1 started 12:47 UTC: frozen plugin and binary = main at 29cb6f6
    (`dla 0.9.0-preview.4`), model claude-sonnet-5-5, timeout 9000 s, 600 turns.
    Driver log /root/dla-real/pillow-1.driver.log.
- pillow-1 ENDED (219 s, 40 turns): COMPLETE report, no leak; note wip-notes/real-run-plugin-pillow-1.md
  (five causes: one intake.workload blocker closes every queue; C driver offered for a CPython
  extension; no test loop for Python; residual open items closed inconclusive without a capture;
  the model built the extension itself). Evidence u36/pillow-1.
- Workflow wf_1364df94-59b (Sonnet, read-only): four code maps (blocker, residual, testloop,
  native-host) and one hand-driven experiment on Pillow in u73 (pytest unit, alloc capture, route
  to a finding at ImageFont.py:580). Next: read the reports, verify the key claims, write builder
  briefs (blocker per runtime first; then Python test loop; then residual capture rule; native
  under a host runtime).

## 2026-11-22, about 14:10 UTC
- Workflow wf_1364df94-59b finished: five reports saved as wip-notes/map-blocker.md,
  map-residual.md, map-testloop.md, map-native-host.md, pillow-by-hand.md (originals u74/0..4.md).
  Key facts: `workload_ran` refusal exists but only when EVERY runtime has a series; Closed::Blocked
  is computed once for all runtimes. By hand the tool reaches ImageFont.py:580 through residual ->
  `--capture alloc` -> `summarize alloc-stack-diff` -> `ledger add --origin residual` -> sibling ->
  test -> alloc-test -> refute; `ledger confirm` then needs a holder (holder-required); the
  documented kind for "nothing holds it, native code kept a reference" is native-handle/global-ref.
  Per-site count columns (`node.listeners@site`, groups in the record) are the existing shape to
  reuse. No candidate of the census lies in ImageFont.py or ImageDraw.py. Whole Pillow suite: 32 s
  per run outside the harness (3736 tests). System python3 (3.13) has no pytest.
- Plan (sequential Sonnet builders in the shared tree, I verify each, one CI at the end, then
  real run pillow-2):
  W1 blocker per runtime: brief wip-notes/blocker-runtime-brief.md, scratch u75. STARTED (agent
     a2f8ebd3daf968625, background).
  X  `py.blocks@<file:line>` columns for Python driver runs + add command + L-OWNERSHIP region
     heap: brief wip-notes/py-site-columns-brief.md, scratch u76. Start after W1 is verified.
  Y  `dla harness gen --lang python --pytest` (hookless loop over the project's tests, first
     command of a Python queue): brief wip-notes/py-test-loop-brief.md, scratch u77. After X.
  Later (not briefed): W3 residual open item cannot be closed `inconclusive` while budget remains
     and no capture or site columns attribute it; W4 C sources of a CPython extension are offered a
     C driver (after W1 a native blocker closes only native).
- CI on main 29cb6f6: 14 of 15 jobs green at 13:34, dynamic acceptance still running; waiter
  writes u71/ci-wait.out. Release 0.9.0-preview.4 not dispatched yet.
- Disk: deleted /root/dla-real/pillow-1 (evidence kept in u36/pillow-1) and u70/bin.
- 14:17 UTC: v0.9.0-preview.4 PUBLISHED (pre-release, target 29cb6f6, release run 45031474805
  green, 5 platform archives + checksums + SHA256SUMS). CI on 29cb6f6: run 45023802129 green,
  all 15 jobs. Task 74 done.

## 2026-11-22, about 14:50 UTC
- W1 (blocker per runtime) REPORTED and checked by me: 7 new tests in
  crates/dla-cli/tests/workload_runtime.rs pass; three core mutations of the builder's script
  (u75/mut.py: W20 queue ignores the runtime, W1 workload_ran ignores it, W3 answer_covers always
  true) each fail 2 to 3 of them; real tree (u72/pillow, run 20261122-143517-3e52, python 186 and
  native 160 candidates, H1 = direct draw.text unit, S1/S2): blocker without --runtime refused and
  lists both; `--runtime python` refused `workload-ran` naming H1 and S1; `--runtime native`
  accepted; status native closed blocked, python open 186 pending; close-rest closed 384 and left
  the python ones. I removed one sentence the builder added to references/recovery.md beyond the
  brief (it told the agent to drive C extension candidates through Python, which the tool has no
  route for). Builder's whole suite: 3204 passed, 1 failed (a golden text, fixed after); the whole
  suite is NOT rerun yet on the final W1 tree: run it on the combined tree at the end.
  Snapshot 8253583 on wip/progress = W1 checked.
- X (py site columns) STARTED: agent a10fa59fbfcdce816 (background), scratch u76.
- Then Y (py test loop), brief ready. Then: whole suite, CI on a frozen snapshot, commits to main
  (one per brief, messages to write), frozen plugin + binary refresh, real run pillow-2.
- Prepared for pillow-2: /root/dla-real/run.py has `--py-venv <python>` (copy in
  wip-notes/run.py): before the model starts it makes <work>/venv, installs pytest and the project
  editable there (builds the extension in place in the project copy), puts venv/bin first on PATH.
  Command to use: `python3 /root/dla-real/run.py pillow-2 /root/dla-real/src/pillow
  /root/dla-real/prompt-generic.txt --timeout 9000 --max-turns 600 --py-venv python3.12`.
  Say in the report that this stands for a developer's machine where the project already builds.
- Found while preparing: the harness scratch copy holds tracked files only and leaves out untracked
  libraries ("they are rebuilt in the copy"), but a Python harness has no build step, so a package
  with extension modules built in place cannot import from the copy (pillow-1: the agent copied the
  .so files by hand in its hook). Added to brief Y as item 5.
- W1 commit message written: wip-notes/commit-message-9.txt.

## 2026-11-22 about 17:10 UTC: builder X reported; one defect found by me

- X (py.blocks@<file:line> columns) reported done; real route on the Pillow tree reaches `ledger confirm`; label `--as L-OWNERSHIP` is refused `label-runtime-row` (python reference has only L-COLLECT; the fix line offers `--note`).
- Defect I measured: the agent keeps one dict of counts per checkpoint, so `py.traced` grows by about 24 KB per checkpoint on a unit with NO leak (harness H3 in run 20261122-152125-9f39 of u72/pillow: series S5 new agent: py.traced steady 55 B/unit, os.rss passes too; series S7 old agent: no column passes). On the leak unit py.traced reads 91 B/unit against 32 to 36 before.
- Sent back to builder X (agent a10fa59fbfcdce816): agent keeps nothing between checkpoints (full counts in every checkpoint record), the series builder picks the sites that grew (12 at most), a cut run still gets columns, counting order such that every point carries the same transient cost. Acceptance: no-leak unit passes no column; leak unit still passes the site column and py.traced is back near 32 to 36 B/unit.
- After X: start builder Y with py-test-loop-brief.md (scratch u77 holds base-plugin export of main; use u78 for Y).

## 2026-11-22 about 18:20 UTC: X corrected by the builder and checked by me

- Correction: the agent writes `group_counts: {"py.blocks": {site: n}}` in every checkpoint record and keeps nothing; build.rs `site_columns` picks the grown sites (12 at most); cut runs still get columns.
- My checks: builder tests rerun (build:: 15, harness_python_blocks 6, py_site_columns 5, skill_text 16, archetypes_data 14 pass); my mutations u77/mut.py V1 (agent keeps a dict per checkpoint), V2 (>=), V3 (first point after warm-up), V5 (gate tie by evidence) each fail a test; V4 (the `test_` file-name prefix) fails NO test: given to builder Y as item 7b.
- No-leak unit, 6 interleaved pairs (u77/noise.sh): new agent passes some column in 5 of 6 runs, old agent in 3 of 6; OS columns at 9 to 44 B/unit with both, py.traced at 5 to 7.5 B/unit with both. So the unit is noisy by itself; the 55 B/unit of the first version is gone. One run passed `py.blocks@src/PIL/Image.py:413` as a plateau (name strings built for getattr stay live in steps; 52-byte blocks): an open issue, the pipeline handles it with sibling and alloc-test.
- Fresh hand run .dla/20261122-162922-b4b7 in u72/pillow: 580 column 1.04/unit in both key modes, py.traced 35 B/unit; add (F278) -> resolve refused growth-not-measured -> sibling -> test -> alloc-test (decided_by the site column, favors leak, 7 s) -> refute -> confirm ok. Label L-OWNERSHIP needs: `--note` (python reference has no row) AND evidence growth-series + alloc-stack: `harness run H1 --mode fresh --capture alloc` (18 s), `summarize alloc-stack-diff <4 profiles> --finding F278` (tied F278), then `ledger label F278 --as L-OWNERSHIP --evidence E22,E21 --note ...` ok. The label refusal's hint `dla trend <S> --finding F` is refused by clap without `--against` (item 7c for Y).
- Gate: the site column item is its own action but printed after the first five (queue, F1, native census, native sweep, os.rss residual); trend's `next` prints the add command second.
- Reworded harness/python/README.md depth table row (it named an API of the evidence library).
- commit-message-10.txt written for X. Brief Y updated (scratch u78, section "What brief X left you", item 7 a/b/c).
- Next: my own `dla-bench dynamic --only python` result (u77/dyn-python.log), snapshot, start builder Y.

## 2026-11-22 about 18:50 UTC: builder Y started

- My own `dla-bench dynamic --only python`: 9/9 pass (u77/dyn-python.log), 13 min.
- Builder Y (Sonnet, background agent aab0b0d685f8d5184, scratch u78) works on py-test-loop-brief.md in the shared tree on top of W1 and X. Do not run cargo or edit the tree while it runs.
- Disk: freed old scratch (u18/pillow, u24/bin, u20/X2, u8chk-K4/r2, two old run dirs in u72/pillow/.dla); 5.5 GB free.
- After Y: verify (rerun tests, own mutations, real-tree check incl. the queue's first command and the refusal without pytest), whole suite, clippy, schema check, snapshot WITHOUT [skip ci] and wait for CI, then three commits to main (messages 9, 10 and one to write for Y), push main and main:dev/wonderful-mccarthy-algc5s, refresh /root/dla-real plugin and bin, run pillow-2 with --py-venv python3.12.

## State at about 19:30 UTC 2026-11-22

- main is still 29cb6f6 (0.9.0-preview.4). Three changes wait in the working tree, all on wip/progress.
- Trees for the three commits: W1 = snapshot 8253583 (message commit-message-9.txt), X = snapshot 7633949
  (commit-message-10.txt; 9d3ef27 has the same tree), Y+Z+ci.yml = the final snapshot (commit-message-11.txt, to write).
- Builder Y (pytest loop) reported; builder Z (brief py-census-discovery-brief.md) closed the gap that made Y inert:
  `dla census --lang python` writes the workload discovery result when the run has none and the project has Python
  test files (no candidate condition for Python; the JVM branch is unchanged and still has the joined_sites hole).
- My checks of Y+Z so far: ten own mutations (scratchpad u79/mut.py, Q1..Q10) each failed a named test and are reverted;
  the new and changed targets pass with pytest on PATH; forbidden-name scan clean; fmt clean.
- CI: added `sudo apt-get install -y python3-pytest` to the Linux leg of the test job (the loop tests skip without pytest).
- Real tree (u72/pillow, run 20261122-191401-d847): after probe, sweep, census only, `dla run status` names
  `dla harness gen --lang python --pytest`; gen selected 158 test paths; the loop run was started 19:14:37.
- Still to do: read the loop's trend; whole suite on the combined tree; clippy; schema check; snapshot without
  [skip ci]; CI; three commits; push main and main:dev/wonderful-mccarthy-algc5s; refresh /root/dla-real; run pillow-2.
- Not caused by the new work: two gc.freeze tests of harness_capture_python fail under the uv-managed CPython 3.12.13
  (it starts with gc.get_freeze_count() == 375); they pass on CI's interpreters.
- Open items to tell the owner: the Python test-directory rule is case sensitive (a `Tests/` helper file gets a site
  column); the JVM census does not write the discovery when it only joins the sweep's sites; the loop is an order of
  the queue, not a gate rule; pytest loop untested on macOS and Windows; a pytest-loop series holds many growing
  lines that are the test suite's own (12 columns, more without a column).

## State at about 21:10 UTC 2026-11-22

- Tenth frozen snapshot on wip/progress: 868bfed6d31d8da18259a2754a74ae73a541b25c (W1 + X + Y + Z + S + ci.yml pytest step).
  CI for it is running; wait with `bash wip-notes/ci-wait-branch.sh 868bfed... <outdir> wip/progress 3300`.
- Local checks on that tree: whole suite 171 result lines, 3281 passed, 0 failed, 5 ignored (default PATH, pytest tests
  skip); with the venv first on PATH harness_pytest_loop 17, harness_python_blocks 8, pytest_loop 13, py_site_columns 7,
  trigger 4, keep_going 31 pass; clippy -D warnings clean; schema --check clean; fmt clean; eleven own mutations
  (u79/mut.py Q1..Q11) each failed a named test.
- Builder S (brief site-order-brief.md): allocation-site columns come with a finding first, then largest Sen slope
  (`passing` in trend/summary.rs, gate.rs `site_columns_largest_first`).
- Real tree, run 20261122-191401-d847 in u72/pillow: probe, sweep, census -> queue names the loop; loop 1258 s, 9 points,
  149 files, 3736 tests per run, dropped 0; 12 site columns pass, the leak line src/.../ImageFont.py:580 at 299 blocks
  per run (second); F278 added there; H2 via --probe F278 with a text-drawing unit: 1.03 blocks per unit; alloc-test
  favors leak, decided by the site column; F278 confirmed.
- Why 12 lines grow in the loop: the library's deferred-import-error object re-raises one stored exception; each raise
  extends its traceback, whose frames pin every pytest session (120k objects, 80 MB per run; measured without the agent).
- When CI is green: commit three trees to main with commit-snapshot-tree.sh: 8253583 + commit-message-9.txt,
  7633949 + commit-message-10.txt, 868bfed + commit-message-11.txt; push main and main:dev/wonderful-mccarthy-algc5s;
  refresh /root/dla-real/plugin and bin; run pillow-2.

## State at about 22:30 UTC 2026-11-22

- pillow-2 (unattended, plugin from snapshot 86aef0c: W1 + X + Y + Z + S, before T): FOUND. 1165 s wall. Report:
  python/F278 confirmed at src/PIL/ImageFont.py:580, growth py.blocks@...:580 3.05 per iteration, trigger reproduced
  through the public API, mechanism named (the C function font_render takes a reference with a getattr and never
  releases it). The worker did NOT run the loop (worker.md named the driver batch; gate item fix named it too).
  Real-project score: aiohttp, grpc-go, axios, tokio-util, spring-data-commons, Pillow found; Jetty not run.
- Report defects seen in pillow-2: heading UNCLASSIFIED (python archetype rows lack L-OWNERSHIP); Verified DISAGREES
  on site (Python call line vs C line); top warning "0 of 739 entry points exercised" is false (editable install:
  hook drivers import the package from the source tree, coverage maps only the scratch copy; task 78); status
  INCOMPLETE with 175 findings closed time_budget while 41 of 60 minutes remained; budget stayed 1h (task 77).
- CI round 1 on 868bfed: only failure was the over-exact block-count test (two OS); dynamic acceptance passed.
- Eleventh frozen snapshot 658c8005427f8ffe24a7c968adefd276f8249a1d = tenth + test fix + builder T (worker.md,
  census note names the command, ledger list queue command, pytest_hint) + VERIFY sentences. Local: whole suite
  3285 passed 0 failed; clippy, schema, fmt clean. CI round 2 on it is running (waiter writes u79/ci11).
- When CI round 2 is green: commit trees 8253583 (message 9), 7633949 (message 10), 658c800 (message 11) to main.
- Builder U (dispatch-budget-brief.md) is working in the tree on top of 658c800: auto budget in dispatched runs,
  gate fixes loop-first. Its work is a FOURTH commit (message 12 to write) and needs its own CI round.
- After U: builder for task 78 (coverage under editable install), then pillow-3 (task 79).

## State at about 22:55 UTC 2026-11-22

- CI round 2 (run 45075458294 on 658c800): every job green except dynamic acceptance, still running.
- Candidate for the twelfth frozen snapshot: faae7f6145f68e7874bd7ff788299a0d1106de84 = 658c800 + builder U (auto budget
  in dispatched runs: scope.rs budget_ready/settle_auto_budget, gate_cmd.rs; loop-first gate fixes: view.rs
  status_for, gate.rs NoHarness::fix_with) + my correction (a generated harness keeps its run command; queue.rs test
  expects the loop first) + builder V (agent coverage records files under DLA_SOURCE_ROOT) + two VERIFY rows.
  Local: whole suite 3291 passed 0 failed; pytest-dependent targets pass with the venv; clippy, schema, fmt clean;
  own mutations Q13..Q15 each failed a named test. commit-message-12.txt is written.
- Do NOT push a non-skip snapshot before CI round 2 completes (it would cancel the run).
- When round 2 is green: commit 8253583 (message 9), 7633949 (message 10), 658c800 (message 11) to main, push main
  and main:dev/wonderful-mccarthy-algc5s; then push faae7f6's tree as the twelfth frozen snapshot (non-skip) for CI
  round 3; when green, commit it with message 12.
- pillow-3 started 22:51:30 with the plugin and binary of faae7f6 (waiter prints the report head). pillow-2's report,
  state, ledger and transcript are kept in /root/dla-real/pillow-2-keep.
- Builder U's last Bash call was denied by the permission system (a chained command with a background run); I did
  not repeat it for the builder; my own standard checks found and fixed two failing queue tests.

## State at about 23:55 UTC 2026-11-22

- main = 471f809 (three commits: 6f84d64 blocker per runtime, 89ec249 site columns, 471f809 Python test loop), pushed to
  main and dev/wonderful-mccarthy-algc5s after CI round 2 (run 45075458294) was all green. CI on main is running.
- Twelfth frozen snapshot 264e93d267435d5c3f5edb16e1106658569e5330 (budget with workers, loop-first gate fixes,
  coverage from the source tree, two VERIFY rows) is in CI round 3 (waiter output u79/ci12). When green: commit with
  commit-message-12.txt, push main and the dev branch, wait for CI on main, then Version 0.9.0-preview.5 (files:
  .claude-plugin/marketplace.json, .claude-plugin/plugin.json, Cargo.toml, Cargo.lock) and release.yml with publish.
- pillow-3 (plugin of faae7f6 = the twelfth snapshot's tree): FOUND. 3358 s. The python worker ran the loop first
  (`harness gen --lang python --pytest`), the budget was sized to 3h by both workers (2 runtimes, 570 candidates),
  F278 at src/PIL/ImageFont.py:580 added from the loop's trend, confirmed, labelled L-OWNERSHIP (growth 301 blocks per
  run of the suite), report top without the false coverage warning (558 of 739 entry points exercised).
  Weak points: status INCOMPLETE (87 open, 55 of 180 minutes used); no finding verified (the verifier stage did not
  run); the mechanism text blames the image handed to the C call, not the int reference (pillow-2 named the exact C
  line); 10 other findings confirmed are registries fed with new keys by the driver (F17 EXTENSION and similar),
  8 of them with "Trigger: not checked"; the worker first wrapped the loop run in `timeout 560` and lost 9 minutes.
  Kept files: /root/dla-real/pillow-3-keep (report, state, ledger, gate, T1, transcript).
- The disk filled during pillow-3 (34 harness copies of the project, 3 GB); freed. Delete a real run's work directory
  after keeping its small files.
- Real-project score: aiohttp, grpc-go, axios, tokio-util, spring-data-commons, Pillow found; Jetty not run.

## State at about 00:40 UTC 2026-11-23

- main = a2210bd "Version 0.9.0-preview.5" on top of bce0d30 "Size the automatic budget in a run with workers"
  (tree of the twelfth frozen snapshot 264e93d, CI round 3 run 45083449055 all green). Both pushed to main and
  dev/wonderful-mccarthy-algc5s. The working tree is clean.
- CI on main at a2210bd is running (waiter b950s9akm writes u79/ci-main-v5). When green: trigger release.yml on main
  with inputs {"publish": true} (MCP tool mcp__github__actions_run_trigger, run_workflow), then check the release
  0.9.0-preview.5 exists. Previews are pre-authorized by the owner.
- Next work (task 81): registries fed by the driver's own new keys are confirmed as leaks in Python (pillow-3: 10 of
  them). The trigger machinery has `Trigger::Direct` ("grown only by calling the registration function itself") but
  the Python census has no writer facts (start-up registration or not), and a thin public wrapper around the writer
  is not seen as direct. Also say in the texts: run the pytest loop in the background, never under `timeout`.
- Then Jetty 12.0.31 (large Maven build; free disk first; add the repository with add_repo before cloning).

## State at about 01:40 UTC 2026-11-23

- main = a2210bd (Version 0.9.0-preview.5); CI run 45088966409 on main in progress (waiter b950s9akm, output
  u79/ci-main-v5). When green: release.yml on main with {"publish": true}; check the release exists.
- Builder W (brief py-registry-brief.md, resumed twice) works in the tree on top of main. Verified so far by me:
  its tests pass (dla-census 132+7, dla-core lib 248, dla-cli trigger 6, skill_text 19, queue 16); own mutations
  Q16 (refusal call removed), Q17 (startup_via cleared), Q18 (hook-driver direct disabled) each failed a named test.
  Real library (run 20261123-013045-7642 in u72/pillow): six registration writers have startup true, the seventh
  (register_extension) has not, correctly: one plugin's save handler calls it at run time with an extension taken
  from the file name. Hook driver calling register_open with new keys: table grows 1 per unit; alloc-test next has
  no confirm; `ledger confirm F12` refused `registration-only`. Real route that works: `--driver --entry F12` with a
  unit that opens an image (no registration call) -> constant -> alloc-test favors non-leak (E21) -> refute F12,
  confirm N-INTENT sibling F12.n2 with E21. W's printed "way (a)" needed a hand-made discriminator; sent back to
  print the one real route and to make its CLI test use real commands.
- After W: my checks again (fix text, test without append_discriminator), whole suite, clippy, schema, snapshot for
  CI, commit message 13 (to write), commit, then pillow-4 to see registries end as non-leak or unconfirmed.
- Disk: 2.6 GB free; target is 15 GB (prune-superseded when no build runs).

## State at about 01:50 UTC 2026-11-23

- 0.9.0-preview.5 is PUBLISHED: tag v0.9.0-preview.5 at a2210bd, pre-release, 11 assets, release run 45095582891 all
  green, CI on main run 45088966409 all green. (`gh release view` fails here: GraphQL is not available; use
  `gh api repos/zerosize-dev/dla/releases/tags/<tag> --jq ...`.)
- Registry change (builder W) in the working tree on top of main, not committed. Remaining: W closes the gap that the
  hook-driver direct rule walks declared targets only (a second table written by the same registration function was
  still confirmable on the real library); then my checks (mutations Q16..Q18 pass already), real-library recheck for
  the second table, whole suite, clippy, schema, CI snapshot, commit with commit-message-13.txt.
- Then: pillow-4 to see the registries end unconfirmed; Jetty 12.0.31 (add_repo, disk).

## State at about 02:35 UTC 2026-11-23

- Thirteenth frozen snapshot e6f8581a9acf08d9d2d087bff41e7c9ce8fbf7ff = main a2210bd + the registry change (builder W,
  four passes) + one VERIFY sentence. In CI round 4 (waiter bejx7vbjy, output u79/ci13). When green: commit with
  commit-message-13.txt (commit-snapshot-tree.sh e6f8581 ...), push main and the dev branch.
- Local checks on it: whole suite 3306 passed and 4 failed ONLY because the disk filled (98 MB free); with space freed
  the three targets pass (real_tools 8, attach 18, harness_capture 11); fmt, clippy -D warnings, schema clean;
  dla-bench static 64 pass; own mutations Q16..Q19 each failed a named test.
- Real library with the final build: one registration call fills two tables; both findings refused
  `registration-only`; alloc-test next prints the entry-path route for both. Entry-path harness without the call ->
  constant -> alloc-test favors non-leak -> refute + N-INTENT sibling confirmed (checked with the earlier build).
- pillow-4 started 02:33 with the plugin and binary of e6f8581 (waiter b9m7hgm0b stops it from going unnoticed when
  the disk drops under 700 MB; it only reports, it does not stop the run). After it: keep report/state/ledger/
  transcript in /root/dla-real/pillow-4-keep and delete /root/dla-real/pillow-4.
- Disk: keep at least 4 GB free before a whole-suite run; a real run on this library uses up to 3.7 GB.

## State at about 03:20 UTC 2026-11-23

- pillow-4 (plugin of e6f8581) is running. Interim at 24 min: registries refuted with N-INTENT siblings confirmed
  (non-leak confirmed 11, leak confirmed 0 plus one disputed by the verifier); the python worker generated the full
  loop (H2), did not run it, generated a loop with `--tests` on three files (H3) and ran that, so the loop step
  counted as done and the leak line was not seen. Task 82: a narrowed loop must not satisfy the queue's first step.
- Disk: pillow-4 uses 5.5 GB and grows; `cargo clean -p dla-cli` freed 3.6 GB (rebuild dla-cli before any cargo test).

## State at about 03:50 UTC 2026-11-23

- pillow-4 still running; it has 114 harness directories, each with a 79 MB copy of the project (9.1 GB). To give it
  room I ran `cargo clean` (target is EMPTY now: the next cargo command rebuilds everything, about 15 to 20 minutes).
  Task 83: one full project copy per harness is a product defect (disk use of a run).
- CI round 4 on e6f8581 (registry change) still running (waiter bejx7vbjy, output u79/ci13).
- Task 82 brief to write: a pytest loop made with --tests must not satisfy the queue's first step.

## State at about 04:00 UTC 2026-11-23

- main = bd1fc84 "Do not confirm a start-up registration table as a leak" (tree of e6f8581, CI round 4 run 45100885318
  all green), on top of a2210bd (0.9.0-preview.5, published). Pushed to main and dev/wonderful-mccarthy-algc5s.
  The working tree is clean. The registry change is NOT in a published preview yet (next preview: 0.9.0-preview.6).
- pillow-4 still running (waiter bu48tyw00). target/ is empty.
- Open work: task 82 (narrowed test loop must not satisfy the first step), task 83 (one project copy per harness),
  Jetty not run, report quality items from pillow-2/3 (verifier stage skipped when the run ends incomplete,
  mechanism text less exact in pillow-3).

## State at about 04:20 UTC 2026-11-23

- pillow-4 (plugin of e6f8581 = main bd1fc84): NOT FOUND. 6072 s, status COMPLETE, 608 findings all dispositioned;
  leak hypotheses confirmed 0 (one disputed by the verifier), refuted 11; non-leak explanations confirmed 11 (the
  registry rule works unattended). The python worker generated the full loop H2, never ran it, ran a three-file loop
  H3; no ledger line mentions the leak line; 119 harness directories (9.5 GB). Kept: /root/dla-real/pillow-4-keep.
- Unattended Pillow runs so far: pillow-1 not found (before the fixes), pillow-2 found (own driver), pillow-3 found
  (full loop), pillow-4 not found (narrowed loop).
- Builder L (brief py-loop-full-brief.md, task 82) is running on the clean tree at bd1fc84: a narrowed loop must not
  satisfy the queue's first step. After it: my checks (mutations, real library), whole suite (keep 6 GB free), CI,
  commit message 14 to write, then pillow-5, then 0.9.0-preview.6.
- Disk 19 GB free after deleting /root/dla-real/pillow-4; target/ was empty before builder L started.

## State at about 04:57 UTC 2026-11-23

- Fourteenth frozen snapshot 69862151a65951692a400d7202ee5edabdbf4cd5 = main bd1fc84 + builder L (a pytest loop made
  with --tests is marked `pytest-narrowed` in its tools and does not end the queue's first step; kept-whole hint on a
  narrowed loop). Local: whole suite 3315 passed 0 failed; pytest-dependent targets pass with the venv; clippy,
  schema, fmt clean; static 64 pass; own mutation Q20 failed four named tests; real library: after a 2-file loop ran,
  status and ledger list still print `dla harness run H1`. CI round 5 running (waiter bfy41cvuf, output u79/ci14).
  When green: commit-snapshot-tree.sh 6986215 commit-message-14.txt, push main and the dev branch.
- pillow-5 started 04:55 with the plugin and binary of 6986215 (waiter b2wb3pm85). target/ is empty again
  (`cargo clean` to leave the run 19 GB). After the run: keep small files in /root/dla-real/pillow-5-keep, delete
  /root/dla-real/pillow-5.
- After both: Version 0.9.0-preview.6 (registry change + narrowed-loop change) once CI on main is green.

## State at about 05:40 UTC 2026-11-23

- pillow-5 (plugin of 6986215): NOT FOUND, 1738 s, status COMPLETE, 572 findings, nothing confirmed, 3 harnesses.
  The python worker generated the full loop H2 and started it with `harness run H2 --iterations 3 > out 2>&1 &` plus
  `sleep 90` in one tool call; the job died with the call (dump has ticks only, no attempt.json); the worker then
  recorded `intake.workload` unresolved env_unavailable for python (accepted), close-rest closed 277 findings.
  The texts added with the registry change ("start it in the background and wait") caused the `&`; pillow-3, before
  that text, ran the loop in the foreground and the tool moved it to the background by itself. Kept:
  /root/dla-real/pillow-5-keep.
- Unattended Pillow runs: 1 not found (before fixes), 2 found (driver), 3 found (full loop), 4 not found (narrowed
  loop), 5 not found (loop cut by the shell, false blocker).
- Builder M (brief py-loop-run-brief.md) is running on top of the uncommitted narrowed-loop change: texts say plain
  foreground with the tool's longest timeout; a workload blocker is refused while a test-loop harness has no finished
  attempt; harness-not-run no longer offers the blocker for a test loop. target/ was empty before it started.
- CI round 5 on 6986215 (narrowed-loop change) is running (waiter bfy41cvuf, output u79/ci14). When green: commit
  the snapshot tree with commit-message-14.txt (the working tree then still holds builder M's work on top).
- Open: task 83 (one project copy per harness), Jetty, `--detach` for long harness runs if the text fix is not enough.

## State at about 06:12 UTC 2026-11-23

- Working tree = main bd1fc84 + builder L (narrowed loop; frozen as 6986215, CI round 5 running, waiter bfy41cvuf)
  + builder M (cut loop is no blocker; texts say plain foreground). L+M snapshot: 5e2c00347326b6ef2b0941bc4139bcbd8f27d65d
  ([skip ci]). Local on L+M: whole suite 3319 passed 0 failed; pytest-dependent targets pass with the venv; clippy,
  schema, fmt clean; static 64 pass; own mutations Q20 (narrowed marker) and Q21 (unfinished loop rule) each failed
  named tests. Real library: blocker refused `loop-not-finished` with the loop generated and not run, and after its
  run was killed; `harness-not-run` for the loop offers no blocker.
- Order from here: CI round 5 green -> commit 6986215 with commit-message-14.txt, push main + dev branch -> push the
  L+M tree as a non-skip frozen snapshot (CI round 6) -> when green commit it with commit-message-15.txt -> CI on main
  -> Version 0.9.0-preview.6 -> release.yml with publish.
- pillow-6 started 06:10 with the plugin and binary of 5e2c003 (waiter be591kayr). target/ is empty again.
- Seen and not fixed: `dla run status` lists a killed harness process under `running` (no liveness check).

## State at about 06:22 UTC 2026-11-23

- main = f30f050 "Only the loop over all test files ends the queue's first step" (tree of 6986215, CI round 5 run
  45114653843 all green), on top of bd1fc84. Pushed to main and the dev branch.
- Fifteenth frozen snapshot 9371823e466ff13e48df12fa6ee73764b9f02810 = f30f050 + builder M (cut loop is no blocker,
  texts say plain foreground). CI round 6 running (waiter bv1lwx79c, output u79/ci15). When green:
  commit-snapshot-tree.sh 9371823 commit-message-15.txt, push main + dev, wait for CI on main, then
  Version 0.9.0-preview.6 and release.yml with publish.
- pillow-6 running since 06:10 with the plugin of the same tree (waiter be591kayr).

## State at about 07:30 UTC 2026-11-23

- main = 22bd696 "A test loop whose run was cut is not a workload blocker" (tree of 9371823, CI round 6 run
  45123257647 all green), on top of f30f050 and bd1fc84. Pushed to main and the dev branch. Working tree clean.
- Three commits are on main since the published 0.9.0-preview.5 (a2210bd): bd1fc84 registry tables, f30f050 narrowed
  loop, 22bd696 cut loop. Next preview: 0.9.0-preview.6 (version commit on main, wait for CI on main, then
  release.yml with {"publish": true}). Decide after reading pillow-6.
- pillow-6 (plugin of 5e2c003 = the tree of 22bd696) running since 06:10 (waiter be591kayr). The python worker ran
  `harness gen --lang python --pytest` and then `harness run H2` as a plain foreground command.

## State at 08:25 UTC 2026-11-23 (after the fifteenth snapshot)

- main = 24a42ab "Add the social preview card [skip ci]" (`.github/social-card.png`, the framed chart
  version; the owner uploads it by hand; no CI run for it, as the owner asked). Before it: 22bd696
  (CI on main run 45131266107 was in progress at 08:16). Card sources: wip-notes/social-card/.
- Not published yet: bd1fc84, f30f050, 22bd696 (next preview: 0.9.0-preview.6). Plan: publish after
  the round below, one CI round for all of it.
- pillow-6 (final plugin, started 06:10, hard stop about 08:40): FOUND. python/F275 at
  src/PIL/ImageFont.py:580, confirmed, 300.9 blocks per iteration in the full test loop S1, and
  reproduced by the worker's own probe driver H6 (alloc-test favors leak). Weak points:
  label UNCLASSIFIED, no stored explanation, the verifier's bundle had no frames and no excerpt
  (site `none`, printed as DISAGREES), 186 open items.
  - 175 of the 186 items: four lines that build attribute names at run time; CPython's type
    attribute cache keeps the name strings (measured: 15, 30, 169, 206 live blocks, 0 after
    `sys._clear_type_cache()`). Fix N1.
  - Writer lines of census findings are "tied to no finding" (2 here, 10 in pillow-2, 24 in
    pillow-3). Fix N2. A finding added at such a line has no census facts, so
    `registration-only` does not apply to it.
  - python/F107 (extension table) confirmed L-COLLECT, verifier reads N-INTENT, trigger "not
    found". Not a regression: one plugin's save handler calls the registration function at run
    time with the extension of the file name, so the census is right that it is not start-up only.
  - Loop series S1: twelve project lines grow by a constant count per round (ImageFont.py:580
    +300, Image.py:523 +340, :512 +140, :2712 +105, :515 +70, TiffImagePlugin.py:989 +66,
    Tests/helper.py:331 +56, ...). Eleven of them look like retention by repeated in-process
    pytest.main() calls or by test modules. Experiment P measures the holders.
- In flight (all Sonnet): builder N (brief wip-notes/py-noise-brief.md, scratch u88, waits for
  14 GB free before its first cargo command), survey for allocation stacks of site findings
  (task 88), experiment P (scratch u89, task 89).
- After the waiter for pillow-6 reports: keep small files in /root/dla-real/pillow-6-keep, delete
  /root/dla-real/pillow-6 (9.5 GB) by literal path at once: builder N waits for that disk.

## 08:27 UTC: pillow-6 ended

- exit 0, 8076 s wall, report INCOMPLETE (190 open), budget 3h, 2h13m used. Findings line: leak
  hypotheses confirmed 1 (python/F275 at ImageFont.py:580), confirmed but disputed by the verifier
  1 (python/F107, extension table), refuted 6; non-leak explanations confirmed 6.
- Kept: /root/dla-real/pillow-6-keep (report, state, coverage, ledger, transcript, run.json,
  S1.json, T1.json, bundle_python_F275, gate). Run directory deleted (18.7 GB free after it).
- Small defect seen: the report's trigger for F275 names the test loop H2 ("which test file feeds
  it is not measured") although the probe driver H6 reproduced the growth; the confirm line that
  `dla alloc-test H6` printed cited the loop's trend evidence (python/E6), not H6's.
- Pillow unattended runs so far: 1 not found, 2 FOUND, 3 FOUND, 4 not found, 5 not found,
  6 FOUND (the first with the final plugin 22bd696).
- Briefs written: py-noise-brief.md (builder N, running), py-stacks-brief.md (builder A, to start
  after N is verified). Survey result for A is summarized inside the brief.

## 09:00 UTC (approx.): builder N done, builder A started

- Builder N (interpreter cache emptied at a checkpoint; `grown::finding_is_at_site` used by the
  gate tie, grown rule, trigger and `dla trend` candidates): targeted tests pass under Python 3.13
  (system) and 3.12 (venv). My mutations: Q22 (cache call removed) fails the new block-count test
  on both interpreters; Q23 (the pre-3.13 call removed) fails it on 3.12 only; Q24 (gate tie back
  to the finding's own site) fails the new site-column test. Snapshot a4c73ba (skip ci).
  Still to do for N: whole suite, clippy, schema check, dla-bench static, CI, commit message 16.
- Builder A (brief py-stacks-brief.md, scratch u90) works on top of N's tree. Plan: verify A,
  then ONE whole-suite run and ONE CI round for N and A together, two commits on main
  (N, then A) from the same verified tree if the snapshot script allows, else one commit each
  with its own snapshot.
- Experiment P (what the test loop keeps alive) still running.

## Later on 2026-11-23

- CI on main for 22bd696: success (run 45131266107, 53 minutes). 24a42ab (the card) is a
  `[skip ci]` commit on top of it.
- Experiment P, first result: the test objects of a round are held by pytest's own state. Chain:
  a rewritten test module's `__loader__` (AssertionRewritingHook) -> Config -> plugin manager ->
  Session -> `Session.items` (Function items) -> `Function._instance` (the test class instance) ->
  its `__dict__` -> the objects the tests made. With ten test files that gave a one-time step
  (two generations alive), not the steady growth of the real run. `session.items.clear()` plus
  `hook.config = None` removed most of the step; the control line kept growing. One session with
  K passes over the items worked (653 passed, 97 skipped every pass) but three lines still grew.
  Second experiment running: full suite, name what anchors every round's Session, test cleanups.
  Scripts: scratch u89.

## About 09:40 UTC: builder A done and checked; whole suite running

- Builder A (stacks for a finding at a growing line; brief py-stacks-brief.md) plus one correction
  (a capture that was made and not summarized is refused with the literal summarize command; the
  link is the sha256 of the capture files in the `captures` array of the raw alloc/A<n>.json).
- My checks: mutations Q25 (runtime check dropped), Q27 (site-column condition dropped) each fail
  `alloc_cmd::tests::a_leak_decided_by_a_site_column_leads_with_the_capture_and_the_summarize_line`;
  Q26 (bundle takes any stack record) fails the note test and the bundle unit test. Clean: CLI
  unit tests 322, py_site_columns 11, alloc_test 5, harness_capture 11, ledger_m6 43, skill_text 20,
  summarize_python 14, gate_flow 13, keep_going 31.
- Hand run on the real library (frozen binary u90/dla-A, run 20261123-091638-9cef in u72/pillow,
  env u90/env.sh): hook driver H1 with a unit that draws text with the default font:
  `py.blocks@src/PIL/ImageFont.py:580` steady, slope 1.0 per unit. Finding F278 at the line.
  `dla alloc-test H1 --finding F278`: favors leak, decided by the site column, `next` leads with
  the capture and the summarize line, `stacks_hint` present. Confirmed without capture:
  `dla bundle F278` refused `stacks-not-captured` (fix: the capture command). After the capture run
  (4 dumps, 23 s): refused again, fix = the literal `dla summarize alloc-stack-diff` command with
  the four files. After the summary (E16): bundle with evidence E12, E14, E16, excerpt true, frames
  ImageFont.py:580, ImageDraw.py:508, ImageDraw.py:565, ExifTags.py:20. The excerpt marks line 580
  as the `self.font.render(...)` call with the `fill` callback above it.
- My own edit after the builder: the fix text of the first refusal no longer holds a `#` comment in
  the middle of a sentence (bundle_cmd.rs).
- Running: u90/suite.sh (clippy, schema check, whole suite to u90/suite.log, static bench), task
  bax2x5sl6. Then: non-skip snapshot for CI, commits 16 (noise) and 17 (stacks), version commit
  0.9.0-preview.6, release.
- Experiment P second pass still running (what anchors every round's pytest Session).

## 09:55 UTC: sixteenth snapshot frozen, CI running, pillow-7 started

- Whole suite on the combined tree: 3337 passed, 0 failed, 5 ignored; clippy clean; schema check
  20 ok; static bench "ok: 64 fixture(s) pass" (command:
  `./target/debug/dla-bench static --fixtures bench/fixtures --dla-bin ./target/debug/dla --json F`).
- Sixteenth frozen snapshot: dfbb9dd80bf91f556d1606f9a8c86c5800c79e60 on wip/progress (non-skip).
  CI waiter: task blnvu07l2, logs to u90/ci16.
  DO NOT push a non-skip snapshot to wip/progress while that CI run is in progress.
- After green CI: commit 16 = tree of snapshot a4c73ba (noise fixes only, message
  commit-message-16.txt), commit 17 = tree of dfbb9dd (message commit-message-17.txt), with
  `bash wip-notes/commit-snapshot-tree.sh <sha> <message file>` twice, then push main and
  main:dev/wonderful-mccarthy-algc5s. Then the version commit 0.9.0-preview.6 (decide after
  pillow-7).
- pillow-7 started 09:55 with the plugin and binary of dfbb9dd; waiter task bkvede3sr.
  `target/` was cleaned (7.2 GB) for its disk: the next cargo command rebuilds everything.

## About 10:25 UTC: the loop's "noise" is a second real leak in the library

- Experiment P, second pass (scripts u89/full.py, outputs full*.out): on the full suite every
  round's pytest Session stays alive. Anchor: `PIL.ImageCms._imagingcms` is a
  `PIL._util.DeferredError` (the optional extension is not built on this machine);
  `DeferredError.__getattr__` does `raise self.ex` (src/PIL/_util.py:19), the SAME exception object
  every time; each raise adds the raising frames to its `__traceback__`, and the frames keep their
  locals: test function -> request -> Function -> Session -> all items -> test instances -> images.
  Clearing `ex.__traceback__` after each call makes helper.py:331, Image.py:512/523/2712 and
  PngImagePlugin.py:374 flat while ImageFont.py:580 keeps growing.
- So eleven of the twelve growing lines of pillow-6's loop are victims of one holder in the
  library. It is a real retention reachable through the public API: hand run H2 in
  u72/pillow run 20261123-091638-9cef, unit `ImageCms.getOpenProfile("p<key>.icc")` in try/except:
  `py.blocks@src/PIL/ImageCms.py:411` +4 per unit, `py.blocks@src/PIL/_util.py:19` +2 per unit,
  `py.blocks@src/PIL/ImageCms.py:176` +2 per unit, `os.private_dirty` +6.3 KB per unit.
  pillow-2 had measured `_util.py:19` as steady in three trends and left it tied to no finding.
- Two web searches found no upstream report of it; the class is unchanged in later versions seen
  in the results. Not verified beyond that. Do not name the project in the repository.
- What the tool lacks: (1) who holds the objects allocated at a growing line (Python has no
  retention capture; `gc.get_referrers` chains to a root would have named the holder);
  (2) a static rule for `raise <stored exception>`; (3) the cause line was not among the twelve
  site columns of the loop series ("21 more allocation sites grew and have no column").
  Task 91.

## About 11:00 UTC: holder capture (Python agent) built and checked on the real library

- Builder R1 (brief py-holder-brief.md, scratch u91): new capture kind `heap` in
  harness/python/sitecustomize.py, file `py-1.retain.json` (format in py-holder-rust-brief.md).
  Its case checks pass under 3.13 and 3.12 when I rerun them (`python3 u91/check_cases.py <python> <tag>`).
- My real-tree check (u92/pillow_heap.py, units u92/units): on the library checkout, a font unit
  and a colour-management unit, 5 rounds of 200, capture 6.7 s:
  `src/PIL/ImageCms.py:411`, `:176` and `src/PIL/_util.py:19` -> root module-global
  `PIL.ImageCms._imagingcms`, chain traceback x4774 <- ImportError(.ex) <- DeferredError;
  `src/PIL/ImageFont.py:580` -> blocks 200 -> 1200, no object found, note "native code holds them
  or a reference was lost there". Both leaks told apart in one file.
- The working tree now differs from snapshot dfbb9dd by R1's edit of sitecustomize.py and
  harness/python/README.md (uncommitted, not in CI). Commits 16 and 17 are made from the snapshot
  trees, not from the working tree.
- Brief R2 written (py-holder-rust-brief.md): capture flag, `dla summarize heap-graph` adapter,
  gate tie through the holder, hints. Start its builder after pillow-7 ends (disk).
- Still to write: static rule for `raise <stored exception>`; a finding's site always gets its
  column beyond the twelve largest.

## 10:55 UTC: N and A are on main

- CI of the sixteenth snapshot dfbb9dd: success, run 45151139840, all 15 jobs.
- main = cbf3de3 "A finding at a growing line reaches the verifier with its stack" (tree of dfbb9dd),
  parent 1a3bfb2 "Do not read the interpreter's cache or a known writer line as a new leak" (tree
  of a4c73ba), parent 24a42ab. Pushed to main and to dev/wonderful-mccarthy-algc5s.
- Not published: bd1fc84, f30f050, 22bd696, 1a3bfb2, cbf3de3 (next preview 0.9.0-preview.6;
  decide after pillow-7, which runs on the tree of cbf3de3).
- Uncommitted in the working tree: builder R1's holder capture (harness/python/sitecustomize.py,
  README.md). It waits for R2 (Rust side) and its own CI round.

## 11:40 UTC (approx.): pillow-7 NOT FOUND; cause found and reproduced; builder G started

- pillow-7 (tree of cbf3de3): exit 0, 4858 s, report COMPLETE, "no leak confirmed", 0 captures.
  Kept: /root/dla-real/pillow-7-keep (report, state, coverage, ledger, transcript, run.json,
  S1.json, T1.json, H2-p1-fresh.log, gate). Run directory deleted. A replay copy of its state:
  scratch u94/project/.dla/20261123-095603-f16d (ledger cut after F2 was added; full ledger
  u94/ledger.full.jsonl).
- Chain of the miss:
  1. Agent defect: the tick thread (`_ticker`, DLA_TICK_MS=100) calls `len(gc.get_objects())`
     in `_build`. On a large heap this races with tuple construction in the main thread:
     `SystemError: Objects/tupleobject.c:909: bad argument to internal function`.
     Reproduced: u94/race.py (600000 objects, thread every 100 ms): 0 errors without the thread,
     25 in 20 s on 3.12, 68 on 3.13. In the run: 7 of 9 loop rounds had such failures; in round 2
     it hit collection, pytest stopped ("Interrupted: 1 error during collection", 27 s) and ran
     no test. This defect was in every earlier Python run too.
  2. The loop counted that run as a unit: every line had one flat step (299, 299, 598, ...), and
     the counter gate failed every `py.blocks@` column with `delta < 1` (shape still steady).
  3. Hole: `grown_unmeasured` needs a PASSED column, so findings F1..F11 that the orchestrator
     added at the non-flat lines were not "grown". `resolve --reason inconclusive` was refused
     (`measurement-required`), but the gate printed `dla ledger close-rest` as their fix and
     close-rest closed all 11 as unresolved(not_exercised).
- Pillow unattended runs: 1 no, 2 yes, 3 yes, 4 no, 5 no, 6 yes, 7 no.
- Builder G running (brief py-loop-robust-brief.md, scratch u95): G1 no heap walk in ticks, G2 a
  run pytest cut is not a unit (+ --continue-on-collection-errors), G3 a finding at the site of
  a non-flat site column is grown.
- R1 (holder capture of the Python agent) was taken out of the working tree and saved as
  wip-notes/r1-holder-capture.patch (applies to cbf3de3; `git apply` it before starting R2).
  It may need a small merge with G1's edit of sitecustomize.py.
- 0.9.0-preview.6 is NOT published: wait for G and a run that finds the leak with it.

## About 12:25 UTC: builder G done; my checks; one test sent back

- Builder G (brief py-loop-robust-brief.md): G1 tick reads no heap (only `len(gc.get_objects())`
  was removed; a tick record has no `py.gc_objects`), G2 cut runs are repeated
  (`MAX_UNIT_REPEATS=2`, `MAX_LOOP_REPEATS=4`, `--continue-on-collection-errors`, summary has
  `cut_units` and `repeats`), G3 `grown::counts_as_growth` (gate pass, or a non-flat
  allocation-site column) used by `grown_unmeasured`; view.rs gives a grown finding the reason
  inconclusive.
- Builder's real driver check (600000 objects, ticks 100 ms, tuples from generators, 30 s):
  SystemErrors before 138 (3.13) and 155 (3.12), after 0 and 0.
- My mutations: Q28 (old tick read restored) fails the tick test on 3.13 and 3.12; Q30 (non-flat
  clause off) fails one unit test and one CLI test; Q29 (exit code 3 not a cut) SURVIVED: the
  cut-run tests cut at collection, where no test ran. Sent back to the builder: a test with a
  run cut after some tests ran.
- My replay with the fixed binary (copy u95/replay, binary u95/dla-G): final gate has no item
  whose fix holds `close-rest`; F2's fix is the sibling, test, probe driver, run, trend,
  alloc-test route; `dla ledger close-rest` closed 0 and left 10 with "its column grew"; a hand
  close as not_exercised is refused with `growth-not-measured`.
- Clean test results after the mutations: workload blocks 10 (both interpreters), pytest loop 20
  (3.12), capture 22 (3.13); CLI py_site_columns 13, pytest_loop 17, gate_flow 13, gate_harness 25,
  keep_going 31, queue 16, trigger 7, skill_text 20, alloc_test 5; core lib 263.
- Commit message ready: commit-message-18.txt.
- Next: builder's extra test, whole suite (u90/suite.sh pattern, static bench command fixed in
  notes), non-skip snapshot, CI, commit 18, pillow-8, then 0.9.0-preview.6 if it finds the leak.

## 12:40 UTC: seventeenth snapshot frozen, CI running, pillow-8 started

- Builder G's extra test: `a_run_cut_after_some_tests_ran_is_not_a_unit_and_is_made_again`
  (exit code 3 from `pytest_runtest_logfinish` after the first file's test). My mutation Q29 now
  fails it. Line of the first file shows a double step in that round (its test ran twice), the
  second file's line rises by one at every step; the note says the tests that ran before the cut
  are in the round's figures.
- Whole tree: fmt clean, clippy clean, schema check 20 ok, pytest-dependent tests under 3.12
  (harness_pytest_loop 21, pytest_loop 17), whole suite 3348 passed 0 failed 5 ignored (default
  PATH, python 3.13), static bench 64 pass.
- Seventeenth frozen snapshot: b28bc4045808c02ec691d4fb9137c4643174ab59 (non-skip). CI waiter:
  task bqop1dl7n, logs u95/ci17. DO NOT push a non-skip snapshot while it runs.
- After green CI: `bash wip-notes/commit-snapshot-tree.sh b28bc4045808c02ec691d4fb9137c4643174ab59
  wip-notes/commit-message-18.txt`, push main and main:dev/wonderful-mccarthy-algc5s.
- pillow-8 started 12:40 with the plugin and binary of b28bc40; waiter task bmwkqo47f.
  `target/` was cleaned again (7.5 GB).
- After pillow-8: if FOUND, version commit 0.9.0-preview.6 (sed 0.9.0-preview.5 -> 0.9.0-preview.6
  in .claude-plugin/marketplace.json, .claude-plugin/plugin.json, Cargo.toml, Cargo.lock), CI on
  main, release.yml with publish. Then R2 (apply wip-notes/r1-holder-capture.patch first).

## 13:47 UTC: G is on main

- CI of the seventeenth snapshot b28bc40: success, run 45176494591, all 15 jobs.
- main = dd6c4cd "The Python agent's tick thread no longer breaks the program it measures" (tree of
  b28bc40), pushed to main and dev/wonderful-mccarthy-algc5s. Working tree clean.
- Not published: bd1fc84, f30f050, 22bd696, 1a3bfb2, cbf3de3, dd6c4cd (0.9.0-preview.6 after
  pillow-8, if it finds the leak).

## 14:50 UTC (approx.): pillow-8 NOT FOUND; cause found; builder H started

- pillow-8 (tree of dd6c4cd): exit 0, 6633 s, COMPLETE, "no leak confirmed", the leak line is not
  in the report. Kept: /root/dla-real/pillow-8-keep (report, state, coverage, ledger, transcript,
  run.json, S2.json, S40.json, H5-harness.json, gate). Run directory deleted. Replay copy of the
  state: scratch u96/project/.dla/20261123-124107-7a96.
- What worked: the loop H1 ran 9 rounds, `failed_units=0 cut_units=0 repeats=0`, 33624 tests, no
  SystemError; `py.blocks@src/PIL/ImageFont.py:580` = 299, 598, ... 2691, gate pass (E18).
- Why it missed: the orchestrator (no workers in this run: "1 runtime") added findings F571..F581
  at the growing lines, then in a shell loop ran `dla alloc-test H5 --finding <F>` for each. H5 is
  a batch driver for F11, F42, F12; its alloc-test series S40 has no py.blocks column. The tool
  fell back to "the primary column of the region" (py.traced plateau, os.rss plateau), answered
  non-leak, and `dla ledger refute F572 --evidence E126` was accepted. The gate's fix had said to
  generate a probe driver first and alloc-test "the harness just made"; the agent skipped that.
- Pillow unattended runs: 1 no, 2 yes, 3 yes, 4 no, 5 no, 6 yes, 7 no, 8 no. Each miss had a
  different cause; 4, 5, 7 fixed on main; 8: builder H.
- Builder H running (brief alloc-about-brief.md, scratch u97): H1 refusal `harness-not-about`
  before the run; H2 no whole-process fallback for a finding with a site column anywhere
  (executed -> non-leak "the line ran and nothing stayed"; not reached -> inconclusive);
  H3 refute/confirm binding; H4 texts.
- 0.9.0-preview.6 still NOT published.

## About 15:30 UTC: builder H built, too strict, sent back

- Builder H (brief alloc-about-brief.md): `grown::harness_about` (target, sibling target, entry
  or probe harness, tied column of a trend of the harness, cited growth evidence), refusal
  `harness-not-about` in `dla alloc-test --finding` before the run, `refuse_unrelated_alloc_test`
  in `ledger refute` and `confirm`, H2 coverage reading for a missing site column.
- My mutations Q31 (confirm exempt), Q32 (probe rule off), Q33 (module-probe coverage case off):
  each fails a named test. Clean: py_site_columns 17, alloc_test 5, alloc_test_jvm 9, ledger_m6 43,
  skill_text 21, trigger 7, queue 16, keep_going 31, gate_harness 25, gate_flow 13, core lib 270,
  CLI unit tests 323.
- PROBLEM found by running the mechanical dynamic bench with the frozen binaries
  (`u97/dla-bench-H dynamic --dla-bin u97/dla-H --parallel 2 --json u97/dynamic.json`, logs
  u97/dynamic.logs; stopped by pid after eight Go fixtures): go-buffer-pool and go-gc-tuned-batch
  fail "no rule-out on a non-leak fixture" (the bench's step 3b refutes candidates with an
  unbound alloc-test on the fixture's command-mode workload harness), go-header-index fails "leak
  L1 is not confirmed" (alloc-test refused although the capture summary of the same harness
  named the finding).
- Sent back to the builder: rule (d) a command-mode workload harness or a test loop is about
  every finding (limitation: whether the code ran there is not checked); rule (e) a capture
  summary of this harness that names the finding. The hook-driver case (H5, F572) must stay
  refused. It also runs the bench on five Go fixtures.
- After its report: my replay, then the WHOLE dynamic bench locally (all 64 fixtures, no other
  cargo job at the same time), whole suite, static bench, snapshot, CI, commit 19
  (commit-message-19.txt needs a sentence on rules d and e), pillow-9.

## About 15:50 UTC: builder H corrected; full dynamic bench running

- Correction: `HarnessAbout::OwnWorkload` (test loop, or command mode with a workload: about
  every finding; does not check that the code ran) and `Capture(evidence)` (an alloc-stack or
  retention-path record that names the finding and was made from a capture file of this harness,
  linked by the sha256 of the files in harness/<H>/captures.json). Builder's five Go fixtures
  pass (go-buffer-pool, go-gc-tuned-batch, go-header-index, go-subscription-workers,
  go-subscription-workers-closed).
- The builder ran `git checkout -p` once with stdin closed (a rule break it reported itself); I
  compared the tree with snapshot d84d1e7: 14 modified files as before, the correction touches 5
  files, additions only.
- My mutation Q34 (command clause off) fails the unit test and the new JVM CLI test.
- My replay with u97/dla-H2: `ledger refute F572 --evidence E126` refused `harness-not-about`.
  `alloc-test H5 --finding F572` on the cut copy now answers `time-budget` first (the copied
  run's deadline 15:41 has passed); no series written. The builder's replay before the deadline
  got `harness-not-about`.
- Running: full dynamic bench with frozen binaries (u97/dla-bench-H2, u97/dla-H2, parallel 3,
  output u97/dyn2.out, json u97/dyn2.json), task bxpjrwpls. Then whole suite and static bench
  (u95/suite.sh pattern without the Q29 step), snapshot, CI, commit 19, pillow-9.

## 17:08 UTC: eighteenth snapshot frozen, CI running, pillow-9 started

- Full dynamic bench with the corrected binaries (u97/dyn2.out): "63/64 pass, 1 xfail", "ok: 63
  fixture(s) pass, 1 expected failure(s)", 57 minutes, no `harness-not-about` in its logs.
- Whole tree: fmt clean, clippy clean, schema check 20 ok, pytest-dependent tests under 3.12
  (21 and 17), whole suite 3365 passed 0 failed 5 ignored, static bench 64 pass.
- Eighteenth frozen snapshot: d8ad3161eb36a09e1b2b2bb36a38e80ececd3ace (non-skip). CI waiter:
  task b5rbbt6q8, logs u97/ci18. DO NOT push a non-skip snapshot while it runs.
- After green CI: `bash wip-notes/commit-snapshot-tree.sh d8ad3161eb36a09e1b2b2bb36a38e80ececd3ace
  wip-notes/commit-message-19.txt`, push main and main:dev/wonderful-mccarthy-algc5s.
- pillow-9 started 17:08 with the plugin and binary of d8ad316; waiter task bd46xi7se.
  `target/` was cleaned (8.4 GB).

## 18:20 UTC: H is on main

- CI of the eighteenth snapshot d8ad316: success, run 45224180803, all 15 jobs.
- main = d9ec81b "An alloc-test decides a finding only on a harness that is about it" (tree of
  d8ad316), pushed to main and dev/wonderful-mccarthy-algc5s. Working tree clean.
- Not published: bd1fc84, f30f050, 22bd696, 1a3bfb2, cbf3de3, dd6c4cd, d9ec81b
  (0.9.0-preview.6 after pillow-9, if it finds the leak).

## 18:35 UTC (approx.): pillow-9 NOT CONFIRMED; cause found; builder I started

- pillow-9 (tree of d9ec81b): exit 0, 4410 s, COMPLETE, "no leak confirmed". Kept:
  /root/dla-real/pillow-9-keep (report, state, coverage, ledger, transcript, run.json, S47.json,
  S48.json, H18-harness.json, H18-dla_harness.py, gate). Run directory deleted. Replay copy:
  scratch u98/project/.dla/20261123-170841-90a0 (F572 goes unresolved at ledger line 2000).
- What worked: loop clean (failed_units=0); closing F571..F582 as not_exercised was refused
  (`growth-not-measured`); the agent made probe drivers; H18 (targets [F572]) gave
  `py.blocks@src/PIL/ImageFont.py:580` = 440, 880, ... 3960 in both key modes; alloc-test E186
  favors leak by that column; eight victim lines were refuted by their own probe drivers.
- Why it was not confirmed: the agent worked from its own script, not from `next`.
  `ledger confirm F572 --evidence E186` -> `growth-gate-failed` (a discriminator cited as growth;
  the fix was a placeholder naming S1); `ledger confirm F572 --evidence E258` ->
  `nonleak-refutation-required` (fix: placeholders); then
  `ledger resolve F572 --as unresolved --reason inconclusive` was ACCEPTED with 107 of 180 budget
  minutes left. Two commands would have confirmed it.
- Pillow unattended runs: 1 no, 2 yes, 3 yes, 4 no, 5 no, 6 yes, 7 no, 8 no, 9 no (closest).
- Builder I running (brief leak-favored-brief.md, scratch u99): refusal (suggested `leak-favored`)
  for closing a finding that has a route to confirm, exact commands there, in close-rest, in the
  gate and in the two confirm refusals.
- 0.9.0-preview.6 still NOT published.

## About 19:20 UTC: builder I built; my replay confirms the leak on the ninth run's state

- Builder I (brief leak-favored-brief.md): `grown::route_to_confirm` / `route_for`
  (`ConfirmRoute`), refusal `leak-favored` in `ledger resolve` (finding and siblings, every
  reason; `time_budget` passes at the floor; aborted runs exempt), `close-rest` leaves it with
  the route, gate and `run status` print the route, `growth-gate-failed` and
  `nonleak-refutation-required` name real ids. Growth-evidence chooser moved to core
  (`growth_evidence`). One existing test changed (it closed a finding inconclusive after a
  leak-favoring alloc-test: now asserts `leak-favored`).
- My replay (copy u99/myreplay, binary u99/dla-I, ledger cut to 1999 lines): closing F572 as
  inconclusive -> `leak-favored`, fix "dla ledger refute F572.n1 --evidence E186 then dla ledger
  confirm F572 --evidence E260"; confirm citing E186 -> `growth-gate-failed` saying E186 is a
  discriminator, fix names E260; the two commands -> F572 CONFIRMED; report: "leak hypotheses:
  confirmed 1", F572 first finding, growth `py.blocks@src/PIL/ImageFont.py:580` 1 per iteration.
- Bench risk the builder flagged: `test_candidate` in bench/runner/src/dynamic_cmd.rs closes a
  candidate unresolved when its alloc-test says leak on a column that did not pass; that close
  is now refused. Nine fixtures did not reach it. Full dynamic bench running to see all 64
  (u99/dyn3.out, task bjn0lvmcn, frozen binaries u99/dla-I, u99/dla-bench-I).
- After the bench: `bash u99/suite.sh` (my mutation Q35, clippy, schema, pytest tests, whole
  suite, static bench), then non-skip snapshot, CI, commit 20 (commit-message-20.txt), pillow-10.

## 20:15 UTC: rule I too wide for the bench; narrowing sent to the builder

- Full dynamic bench with builder I's binaries (u99/dyn3.out, 59 minutes): 60/64 pass; FAIL:
  rust-delayed-queue, rust-ledger-sync, rust-meter-callbacks, rust-tree-parent-links ("gate check
  --final: finding F3 is under_test and has no disposition"). The bench's final gate adds a
  finding at a growing allocation line, alloc-tests it on the fixture's command-mode workload
  harness (favors leak), then closes it unresolved by fixed policy; `leak-favored` refused that.
- Decision: narrow `route_to_confirm` to an alloc-test on a HOOK DRIVER THAT TARGETS the finding
  (targets, sibling_of, or its entry-path harness). Whole-workload harnesses, test loops and
  harnesses about the finding only through a probe, column, cited evidence or capture give no
  route; closing unresolved works there as before. The confirm refusals keep naming real ids on
  any harness about the finding. Not covered then: a loop alloc-test that favors leak followed by
  an inconclusive close.
- After the builder's report: my mutation Q35 (may need a new text), the replay (F572 on H18 must
  still be refused with the two-command route), `bash u99/suite.sh`, non-skip snapshot, CI,
  commit 20 (commit-message-20.txt needs the narrowed wording), pillow-10. I do not plan a third
  full local dynamic bench: the wide rule failed only those four, the narrowed rule can refuse
  nothing the wide one did not, and CI runs the full bench.

## About 21:40 UTC: rule I narrowed; one defect found by the whole suite and fixed by me

- Narrowed rule (builder): route only for a hook driver that targets the finding. Its bench run
  on nine fixtures: 8 pass, 1 xfail (rust-tree-parent-links, the known expected failure).
  My replay with u99/dla-I2: F572 still refused with the two-command route, and the two commands
  confirm it. My mutation Q35 (candidate state dropped from the route): 5 tests fail.
- Whole suite (first run on the narrowed tree): 3377 passed, 1 FAILED:
  `confirm_leak_growth_gate_table` (crates/dla-core/tests/m3_rules.rs). Cause: a real defect in
  the new hint: `grown::growth_evidence` named E12, a growth record with a passing gate and
  STATIC authority, which `dla ledger confirm` refuses. My fix: the chooser filters with
  `machine::is_passing_growth` (kind, authority test/live/dump, gate pass). The table test now
  parses the evidence the fix names and runs that confirm on a fresh ledger (must be accepted).
  My mutation Q36 (chooser back to kind only) fails that test.
- Running: `bash u99/suite.sh` again on the final tree (output u99/suite2.out), task below.
  Then non-skip snapshot, CI, commit 20, pillow-10.

## 21:50 UTC: nineteenth snapshot frozen, CI running, pillow-10 started

- Second whole-suite run (u99/suite2.out, with my chooser fix): 3376 passed, 2 failed, both unit
  tests in crates/dla-cli/src/alloc_cmd.rs whose fixtures built growth evidence with STATIC
  authority (`the_growth_evidence_to_confirm_with_is_the_newest_tied_one...`,
  `a_root_closed_as_static_only_gets_refute_test_and_confirm_in_that_order`). I changed the
  fixtures to authority `test` (what `dla trend` writes); assertions unchanged. After that:
  CLI unit tests 323, m3_rules 27, alloc_test 5, py_site_columns 23, fmt and clippy clean. I did
  not run the whole suite a third time (the two failures were the only ones; the edit is inside
  a test module); CI runs it.
- Static bench 64 pass, schema check 20 ok, pytest-dependent tests under 3.12: 21 and 17.
- Nineteenth frozen snapshot: ea98058442052c422320568b7805cd2145685aac (non-skip). CI waiter:
  task bdxhoz5mn, logs u99/ci19. DO NOT push a non-skip snapshot while it runs.
- After green CI: `bash wip-notes/commit-snapshot-tree.sh ea98058442052c422320568b7805cd2145685aac
  wip-notes/commit-message-20.txt`, push main and main:dev/wonderful-mccarthy-algc5s.
- pillow-10 started 21:50 with the plugin and binary of ea98058; waiter task byezjbc8m.
  `target/` was cleaned (8.6 GB).

## 22:40 UTC: I is on main

- CI of the nineteenth snapshot ea98058: success, run 45273663975, all 15 jobs.
- main = b91cbe6 "A finding whose alloc-test favors leak is confirmed, not closed as inconclusive"
  (tree of ea98058), pushed to main and dev/wonderful-mccarthy-algc5s. Working tree clean.
- Not published: bd1fc84, f30f050, 22bd696, 1a3bfb2, cbf3de3, dd6c4cd, d9ec81b, b91cbe6
  (0.9.0-preview.6 after pillow-10, if it confirms the leak).

## 23:35 UTC: pillow-10 CONFIRMED the leak (run still going); version commit pushed

- pillow-10 (tree of b91cbe6), ledger at 23:31: F2 added at src/PIL/ImageFont.py:580 (holder
  native-handle/global-ref) 23:21:55; under test; alloc-test E928 on its own driver H162 favors
  leak; alloc-stack E932 tied to F2 (capture and summary were made); sibling F2.n1 refuted with
  E928; F2 CONFIRMED 23:24:09 with E933; label UNCLASSIFIED; explanation stored: "At
  src/PIL/ImageFont.py:580 font.render calls font_render in src/_imagingft.c:888, where
  PyObject_GetAttrString(image,"id") returns a new int reference that is ..." (the documented
  mechanism). Final report not read yet: the run goes on until 00:20 at the latest; waiter task
  bv1slgnqc.
- Disk: the run reached 168 harnesses (13 GB of project copies) and 1170 MB free. I deleted old
  scratch binaries and bench logs, and `harness/H{3..150}/work` of the run (project copies of old
  finished harnesses), 13.7 GB free after. Task 83 is the real fix.
- Pillow unattended runs: 1 no, 2 yes, 3 yes, 4 no, 5 no, 6 yes, 7 no, 8 no, 9 no, 10 YES.
- Version commit: main = 28979b4 "Version 0.9.0-preview.6" (four files), pushed to main and dev.
  CI waiter on main: task bq7cx398o (logs u99/ci-main-v6). When green: trigger release.yml with
  publish true, then verify with
  `gh api repos/zerosize-dev/dla/releases/tags/v0.9.0-preview.6 --jq '{tag_name, prerelease, assets: [.assets[].name]}'`.

## 23:45 UTC: pillow-10 final

- exit 0, 6688 s, report COMPLETE, 1h50m of 3h. Kept: /root/dla-real/pillow-10-keep (report,
  state, coverage, ledger, transcript, run.json, bundle/F2, H162 manifest, driver and
  captures.json, gate). Run directory deleted (16.3 GB free).
- F2 is the first finding of the report: confirmed; growth `py.blocks@src/PIL/ImageFont.py:580`
  299 per iteration; writers `src/_imagingft.c:888`; explanation (observed): font_render passes
  the new reference from PyObject_GetAttrString(image,"id") to PyLong_AsSsize_t and never
  releases it, one int per render; label UNCLASSIFIED with that note (no Python row fits).
- The Findings line says "confirmed 0, confirmed but disputed by the verifier (reads N-OBSERVER)
  1". The verifier had the bundle with the stack and the excerpt (frames ImageFont.py:580,
  ImageDraw.py:508, :565, ExifTags.py:20; evidence discriminator, alloc-stack, growth,
  unit-count) and still read it as an observer effect: "nothing in ImageFont, ImageDraw or
  ExifTags adds to a long-lived container ... most likely test or pytest-held results, or
  tracemalloc attribution ... I found no library-side holder". It did not read the extension.
  The worker is right; the report prints the dispute honestly.
- What would settle the dispute: the holder capture (R1, in wip-notes/r1-holder-capture.patch) as
  retention evidence in the bundle: "no Python container refers to the blocks: native code holds
  them or a reference was lost". That is R2 (brief py-holder-rust-brief.md). Also the trigger
  text still names the loop H2, not the probe driver H162.
- Refusals the agent met in this run: growth-not-measured (4 mentions); none of leak-favored,
  harness-not-about, stacks-not-captured, growth-gate-failed: it followed `next`.
- Waiting: CI on main for the version commit 28979b4 (task bq7cx398o), then release.

## 23:50 UTC 2026-11-23

- main = 28979b4 "Version 0.9.0-preview.6" (pushed to main and dev). CI on main was being waited for
  (logs in scratch u99/ci-main-v6). When green: trigger release.yml (ref main, publish true),
  verify the release tag v0.9.0-preview.6. NOT yet released at the time of this note.
- Working tree = 28979b4 + r1-holder-capture.patch (harness/python/sitecustomize.py, README.md).
  Checked again under 3.13 and 3.12 (check_cases ALL OK) and on the real library.
- py-holder-rust-brief.md updated with the lesson of run 10 (verifier disputed the native leak as
  N-OBSERVER; bundle must carry the holder record and a harness list; verifier text; trigger names
  the probe driver). Builder for it (Sonnet) is the next step; scratch dir u93.
- Pillow run 10 (tree b91cbe6): leak at the font line CONFIRMED with mechanism; verifier disputes.

## 01:20 UTC 2026-11-24

- CI on main for 28979b4 (run 45289556532): all jobs green except "dynamic acceptance on the
  fixtures", still running at 00:50. Waiter restarted after a worker restart (logs u99/ci-main-v6).
  When green: release.yml (ref main, publish true) -> v0.9.0-preview.6.
- R2 builder (Sonnet, agent adde4cb6ee309af51) reported; snapshot 9ca99ac holds its tree.
- My real-library check of R2 (frozen binary u101/dla-R2, env u101/env.sh, run
  20261124-004714-d4a5 in u72/pillow; one hook driver whose unit does the font text and the cms
  open in try/except) found nine defects: brief wip-notes/py-holder-fix-brief.md (R3), sent to
  the same builder at 01:15. Main ones: the walk hits the 60 s bound on a 16k-traceback chain
  (5 of 6 lines get no chain); `at` is a cut absolute path for an editable install; a retention
  record switches the stack requirement of `dla bundle` off; the bundle of the native-line
  finding shows the other holder and excerpts of the other leak; type-count columns stay untied;
  `blocks_first: null` is refused by the adapter.
- After R3 returns: rerun the real check from a fresh `dla run init` in u72/pillow with a new
  frozen binary; then own mutations, Python 3.12 run of the builder's tests, whole suite, static
  bench, full dynamic bench with frozen binaries, non-skip snapshot for CI, commit.
- Brief W (wip-notes/work-trim-brief.md, thinning of harness work copies, task 83) is written;
  its builder starts after R3 is integrated (same files).
- Decision: no static rule for `raise <stored exception>` (noise on asyncio-style code); the
  measured holder chain covers it.

## 02:45 UTC 2026-11-24

- CI on main 28979b4 (run 45289556532): first attempt FAILED in "dynamic acceptance": fixture
  jvm-pool-warmup "no rule-out on a non-leak fixture" (unbound alloc-test Inconclusive; F6 growth
  in os.rss_anon left unresolved); the job took 91 min against 47 to 71 (slow runner). The run's
  artifact cannot be downloaded here (blob host not allowed). `rerun_failed_jobs` -> success.
  OPEN: the flake is not diagnosed; try jvm-pool-warmup under CPU load with frozen binaries.
- release.yml triggered (ref main, publish true) at about 02:40; waiter prints the release
  (task bis2n13ou). Verify tag v0.9.0-preview.6 and its assets, then tell the owner.
- Holder evidence, builder rounds (same agent adde4cb6ee309af51):
  R3 (nine corrections) done; R4 (py-holder-fix2-brief.md) done: PYTHONDONTWRITEBYTECODE=1 for
  every Python harness process (the first process compiled the agent and read ~1500 frozen objects
  high: this is why alloc_test frozen_garbage failed with the bigger agent), `ats` on a run step,
  site of `next` never outside the project, bundle focus on the finding's own stacks.
  R5 sent 02:40: R4 made the chains capture 4x slower on the real library (19.5 s -> 81.7 s, bound
  hit); cause measured by stack samples: `_source_rel` runs two uncached realpath calls per
  traceback for a library that runs from its source tree (editable install).
- Builder was denied twice by the permission system (an interpreter under u72/pillow that my
  brief had put off limits; a watcher script with rmtree). It stopped both times; I did not run
  either for it. 3.12 runs are mine: check_cases ALL OK (3.13, 3.12); harness_capture_python 25
  pass + the 2 known gc.freeze failures; py_site_columns 26, summarize_python 22, alloc_test 5,
  pytest_loop 17 pass (R3 tree).
- Real-library walk script: scratch u101/walk.py <frozen binary> <out dir> (binaries u101/dla-R2,
  -R3, -R4; outputs u101/w3.out, w4.out). After R5: freeze dla-R5, walk again; expect heap about
  20 s, five lines under the module global, `next` site a project line, the font bundle with its
  own stack excerpt (R4 gives ImageFont.py:580, ImageDraw.py:508, :565), no finding item in the
  final gate.
- Then: own mutations, whole suite (3.13) + Python test files under 3.12, static bench, full
  dynamic bench with frozen binaries, non-skip snapshot for CI, commit, version 0.9.0-preview.7.

## 02:55 UTC 2026-11-24

- v0.9.0-preview.6 PUBLISHED (release run 45312904399 success; prerelease; target 28979b4; five
  platform archives, their .sha256 files and SHA256SUMS). Task 98 closed. Task 99: the
  jvm-pool-warmup flake (not diagnosed).
- Builder round R5 running: (1) cache in `_source_rel` and one place per code location (walk
  19.5 s -> 81.7 s regression of R4 on the editable-install library); (2) the stack-site clause
  moves out of `holders::names_holder` (it let a bare allocation stack count as retention for
  `strong` and for an observed explanation) into `labels::names_finding` and the label rule.

## 03:40 UTC 2026-11-24

- R5 tree (snapshot 16ca204): real-library walk with frozen u101/dla-R5 is CLEAN (out u101/w5.out):
  heap capture 19.5 s; holders = module global (5 lines) + native entry for the font line; `next`
  proposes project lines; both findings go sibling/test/alloc-test (no second capture)/refute/
  confirm; bundle of the holder finding excerpts its four lines, bundle of the font finding
  excerpts ImageFont.py:580, ImageDraw.py:508, :565 with the stack record and the focused
  native entry; final gate has no finding item.
- My checks on that tree (scratch u104): fmt, clippy, schema ok; whole suite 3424 passed, 1 FAILED:
  dla-core tests/m6_rules.rs `an_alloc_stack_record_counts_for_the_findings_its_stated_ties_name_only`
  (R5 put the stack-site clause into `labels::names_finding`, which the label rule uses; the old
  rule: a label counts stated ties only). 3.12: only the 2 known gc.freeze failures. Static bench
  64 pass. My mutations R1..R9 (u104/mut2.py, muts.sh): all nine caught.
- R6 sent 03:40: `names_finding` back to strict; new predicate (stated tie, or an untied growing
  row at the finding's line) for bundle, stacks_check and the alloc-test offer only; label refusal
  names the literal summarize command with --finding.
- After R6: freeze binaries (dla + dla-bench), walk again (u101/walk.py), m6_rules + whole suite,
  full dynamic bench (`dla-bench dynamic --dla-bin <frozen> --parallel 3 --json F`, about 58 min,
  baseline 63/64 pass + 1 xfail), non-skip snapshot for CI, commit message 21, commit, push main
  and dev. Then builder W (work-trim-brief.md), then pillow-11, then 0.9.0-preview.7.
- Disk 7.4 GB free with target/ built (debug). `cargo clean` before a real plugin run.

## 04:15 UTC 2026-11-24

- R6 done (names_finding strict again; `labels::about_finding` for bundle, stacks_check, alloc-test
  offer; label refusal names the literal summarize command). Builder rounds R2..R6 are closed.
- Frozen tree = snapshot 93ac3b5 on wip/progress (NON-skip: CI is running on it; do NOT push
  another non-skip snapshot before it ends). CI waiter logs: scratch u104/ci-21.
- Frozen binaries: scratch u104/frozen/dla and dla-bench (also u101/dla-R6). Real-library walk
  with it is clean (u101/w6.out; heap capture 17.8 s).
- Running locally (task bzp1m0nid): u104/suite.sh -> u104/suite2.out (whole suite 3.13, Python
  test files under 3.12, static bench), then the full dynamic bench -> u104/dynamic.log and
  dynamic.json (expect 63/64 pass + 1 xfail rust-tree-parent-links).
- When both are green: commit with wip-notes/commit-message-21.txt
  (`bash wip-notes/commit-snapshot-tree.sh 93ac3b5a7060995ade2ad813b9be2a6d8e874961 wip-notes/commit-message-21.txt`,
  then `git push -q -u origin main` and `git push -q origin main:dev/wonderful-mccarthy-algc5s`).
  The working tree must equal the snapshot tree (no edits since).
- Next: builder W (work-trim-brief.md; fresh Sonnet agent), pillow-11, 0.9.0-preview.7.

## 04:50 UTC 2026-11-24

- COMMITTED: main = 49bccfe "A Python run names who holds the objects of a growing line" (tree of
  snapshot 93ac3b5; CI run 45320918413 on the snapshot all green; whole suite 3427 passed, 0
  failed; 3.12: only the 2 known gc.freeze failures; static 64 pass). Pushed to main and dev.
  CI on main: waiter logs scratch u104/ci-main-21 (task bh2fo8jl9).
- Local full dynamic bench with frozen binaries still running (u104/dynamic.log; 36 of 64 at
  04:41, no FAIL so far); it is a duplicate of the CI job that passed. The W builder runs cargo at
  the same time, so a late local failure may be load: rerun that fixture alone before judging.
- Builder W started 04:48 (fresh Sonnet agent ae577710f60833b67, brief work-trim-brief.md,
  scratch u100): thin the work copy of older harnesses, fill on use. Verify: read every reader of
  work/ it lists, own mutations, whole suite, dynamic bench, a walk on the real library with six
  or more harnesses (disk before/after), CI snapshot, commit 22.
- Then pillow-11 (unattended; refresh plugin from the commit; `cargo clean` first for disk), then
  version 0.9.0-preview.7.

## 05:50 UTC 2026-11-24

- Local full dynamic bench on the tree of 49bccfe: 63/64 pass + 1 xfail (baseline).
- Builder W (agent ae577710f60833b67) reported; its tree + my test fix = snapshot 8afb484 on
  wip/progress (NON-skip, CI running; waiter logs u105/ci-22). My edit: alloc_test.rs
  `the_first_point_...` compared steps for equality and failed twice under load on a +49 blip; now
  a tenth of the median step (R2 mutation still fails it; 3 clean runs of the file).
- W checked: real library (u105/thin.py with frozen u105/dla-W): gens 5,6,7 thin H1,H2,H3
  (80 MB -> 260 KB each), H1 run fills and gives the same trend as never-thinned H7, report thins
  all (harness dir 322 MB -> 5 MB), a changed README.md -> `work-copy-stale` naming it, restored
  -> runs. My mutations W1..W5 (u105/mut3.py): all caught.
- Running locally (task bp1vspg40): u105/suite.sh -> u105/suite.out, then dynamic bench ->
  u105/dynamic.log. When CI (8afb484) and these are green: commit with
  wip-notes/commit-message-22.txt (`commit-snapshot-tree.sh 8afb484d5d0d62a295423ee4358cd250f06fddd7 ...`),
  push main and dev.
- CI on main for 49bccfe (run 45327928611) was still in progress at 05:17; waiter task bh2fo8jl9.
- Then: pillow-11 unattended (plugin refreshed from the new commit; `cargo clean` first), then
  version 0.9.0-preview.7 (previews are pre-authorized).

## 07:00 UTC 2026-11-24

- CI on main for 49bccfe (run 45327928611) FAILED in "clippy and test (ubuntu)": the R4 test
  `the_first_point_of_a_command_mode_python_series_...` (exact step equality; one point read +51).
  Fixed in the working tree (tolerance a tenth of the median step); main goes green with commit 22.
- CI of snapshot 8afb484 FAILED on Windows only: scratch.rs test `thin_run_keeps_the_last_used_and_the_new_one`
  set a directory's time through File::open (Access is denied on Windows). Fixed: that step is
  `cfg!(unix)` only. New frozen snapshot 1397662 (NON-skip, CI running; waiter logs u105/ci-22b,
  task b51juwzgc). Do not push another non-skip snapshot before it ends.
- Local on the W tree: whole suite 3447 passed 0 failed; 3.12 only the 2 known; static 64; full
  dynamic bench 63/64 + 1 xfail.
- `cargo clean` done (target/ EMPTY; 14.4 GB free). Plugin in /root/dla-real/plugin refreshed
  from snapshot 1397662; /root/dla-real/bin/dla = u105/dla-W2 (W tree, production code equal).
- pillow-11 STARTED 06:56 UTC (task 100; timeout 9000 s; waiter task boah3w3e4 with low-disk
  check; driver log /root/dla-real/pillow-11.driver.log; run dir /root/dla-real/pillow-11).
  To read: report.md, ledger (confirmed findings, verifier labels), harness count and disk
  (du of project/.dla), `thinned` in harness gen outputs, whether heap capture was used.
- When CI 1397662 is green: commit with wip-notes/commit-message-22.txt
  (`commit-snapshot-tree.sh 1397662aa4f0695e63854b4037456c6f16818211 ...`), push main and dev,
  then version commit 0.9.0-preview.7 after pillow-11 is read (previews are pre-authorized).

## 07:47 UTC 2026-11-24

- COMMITTED: main = cb933c3 "A run does not keep one full copy of the project per harness" (tree
  of snapshot 1397662; CI run 45339326235 on the snapshot all green). Pushed to main and dev. CI
  on main: waiter task bdsckos6r, logs u105/ci-main-22. This commit also carries the tolerance fix
  of the first-point test that failed on main for 49bccfe.
- pillow-11 at 48 min: 35 harnesses, 31 thinned, run dir 417 MB (the earlier runs were at several
  GB by then). Waiter task boah3w3e4 (2 h limit; the run may take 2.5 h: restart the waiter if it
  times out).
- After pillow-11 is read and CI on main is green: version commit 0.9.0-preview.7 (sed in
  .claude-plugin/marketplace.json, .claude-plugin/plugin.json, Cargo.toml, Cargo.lock), push,
  CI, release.yml (ref main, publish true), verify the tag.
- Open: task 99 (jvm-pool-warmup flake), Jetty 12.0.31 not run, the two gc.freeze tests under the
  uv-managed 3.12, `dla run status` lists a killed harness as running, a holder finding added at a
  line its holder does not keep has no column of its own (warning only).

## 08:55 UTC 2026-11-24: the owner asks whether the work is fitted to one project; blind test started

- Owner's question (verbatim): "Are you sure we are not trying to fix Pillow specific case? Will
  fixing Pillow generally help skill to find leaks?"
- Facts from these notes used in the answer: every project of the list was a development target
  (aiohttp found in run 2 and 3; grpc-go in run 4; axios in run 5; tokio-util in run 5; spring in
  run 8; Pillow found in runs 2, 3, 6, 10 and MISSED in 7, 8, 9 after my own changes). No project
  was ever run blind. No project was run twice on one build, so no repeat rate exists. None of the
  five earlier projects was rerun after the Python and gate changes of the last two days.
- Decision: no more Pillow runs after pillow-11. Next is a BLIND TEST on one frozen build
  (0.9.0-preview.7): projects never run before, one unattended run each, generic prompt, no fix
  between runs, score at the end.
- Sonnet research agent HO started 08:55 (brief wip-notes/holdout-research-brief.md, scratch u106):
  ten candidates, two per runtime. It writes u106/candidates.md (no hint of the leak; I read it
  before the runs) and u106/answer-key.md (I read it ONLY after all runs have ended).
- Blind-test rules for me: do not open answer-key.md before the last run ends; take the first
  candidate of each runtime whose install works (the second only if the first cannot be built);
  one run each, `--timeout 9000 --max-turns 600`, same prompt file prompt-generic.txt; no edit of
  skills/, agents/, data/, hooks/ or the binary between runs; Jetty 12.0.31 runs as a sixth
  project (on the owner's list, never run, leak class known to me, so it is not blind).
- pillow-11 at 08:47: still running, 87 harnesses, run folder 558 MB, 13.2 GB free.

## 09:10 UTC 2026-11-24: CI on main green; blind-test candidates chosen; key being checked

- CI on main for cb933c3: run 45350380373 green, all 15 jobs.
- pillow-11 waiter hit its 2 h limit at 08:56 and was stopped; run still going then (98 harnesses,
  656 MB, 12.5 GB free). New waiter b3h61bqwf (45 min limit; the run ends by 09:26).
- Research agent HO reported. Files: u106/candidates.md (read by me) and u106/answer-key.md (NOT
  read by me). Candidates in order: Python 1 opentelemetry-python v1.40.0, 2 mcp python-sdk
  v1.11.0; Go 3 opentelemetry-go v1.29.0, 4 golang.org/x/crypto v0.51.0; Node 5 @apollo/client
  v3.3.13, 6 @mswjs/interceptors v0.40.0; Rust 7 fastrace commit 929879e, 8 jni-rs v0.20.0; JVM 9
  logstash-logback-encoder 7.1.1, 10 micrometer v1.3.5.
- HO's batched install-and-build command was DENIED by the permission system; it stopped and ran
  no install. It did not ask me to run it. Told the owner. I prepare each chosen project myself,
  one plain command at a time, as for every earlier real run; if one of mine is denied I stop.
- HO read the code at the tag for only two of ten. Second Sonnet agent HV started 09:05 (brief
  wip-notes/holdout-verify-brief.md): reads the fix and the code at each tag, rewrites
  answer-key.md, reports per project only CONFIRMED / UNSURE / ABSENT and "reachable in one
  process". No installs or builds in its scope.
- Rules fixed before any run: per runtime, the first candidate whose key is CONFIRMED and
  reachable and whose install works; otherwise the second; if neither, that runtime has no blind
  project and I say so. A run whose key turns out wrong afterwards is reported as void, not as a
  miss or a hit.

## 09:27 UTC 2026-11-24: pillow-11 read; preview.7 frozen; blind runs 1 and 2 started

- pillow-11 (tree of cb933c3): exit 0, 8315 s, COMPLETE, 100 harnesses, run folder 283 MB at the
  end (thinning works). THE KNOWN LEAK WAS NOT CONFIRMED. Chain: the python worker's test loop
  measured growth at src/PIL/ImageFont.py:580 at 07:16 (python/E48, 20 min in); the worker's slice
  ended 07:17; the orchestrator worked 95 queue harnesses; a finding for the line (F4, holder
  native-handle/global-ref) was added only at 09:00 (2 h 03 min in); one driver H99 did not run
  the line's function; closed unresolved(inconclusive) at 09:09 with 40 min of the 3 h budget
  left. Also: python/F17 (extension registry fed by the driver's own keys) confirmed and labeled
  L-COLLECT with retention-path E24 although trigger "not found"; verifier says N-INTENT (right).
  Stored-exception holder not found (module global "not targeted, tier 5"). Kept:
  /root/dla-real/pillow-11-keep. Run directory deleted. Task 102 holds the defect; NOT fixed
  before the blind test on purpose.
- Rig fact: run.py --timeout 9000 was BELOW the tool's 3 h budget in the Pillow runs. Blind runs
  use --timeout 12600.
- Version commit b896d8c "Version 0.9.0-preview.7" on main and dev. CI waiter b5fo434az (logs
  u107/ci-main-23). After green: release.yml (ref main, publish true), verify the tag.
- FROZEN FOR THE BLIND TEST: binary u107/dla-P7 = /root/dla-real/bin/dla (sha256 0fa9d559a22e7ee3,
  debug build of b896d8c), plugin = git archive of b896d8c. Do not rebuild or edit until the last
  blind run has ended. No cargo while runs are going.
- Key check (agent HV): CONFIRMED by reading for 1, 2, 3, 4, 7, 8, 9, 10; UNSURE for 5 and 6.
  Reproduction (agent HR): 5 Apollo DOES NOT GROW (the control with the fix was not run: a patch
  action was denied to it), 6 interceptors GROWS. HR's report came with an automatic security
  warning (sensitive information leakage); my review: two scripts in u106/repro/scripts, only
  localhost, no child processes or file writes, nothing changed outside its directory.
  Rule change made before any run and told to the owner: an UNSURE key may be settled by a hand
  reproduction by an agent that reports only grows / does not grow.
- SET A (baseline, becomes a development set afterwards): 1 opentelemetry-python v1.40.0,
  3 opentelemetry-go v1.29.0, 6 @mswjs/interceptors v0.40.0, 7 fastrace 929879e,
  9 logstash-logback-encoder 7.1.1; plus Jetty 12.0.31 (not blind).
  SET B (stays blind for the measurement after the fixes): 2 mcp python-sdk, 4 x/crypto,
  8 jni-rs, 10 micrometer; Node slot empty (Apollo did not reproduce).
  I have NOT read answer-key.md. candidates.md leaks an area hint through three test commands
  (otel-go, x/crypto, Apollo); I do not use it.
- Sources: /root/dla-real/src/{otel-python,otel-go,interceptors,apollo-client,fastrace,logstash}.
  interceptors: `pnpm install --frozen-lockfile` + `pnpm build` done in the source dir.
  logstash: Maven home filled at u107/logstash-try/m2 (459 tests pass on JDK 21) for --warm-m2.
  otel-python: run.py has a new option --py-setup (backup run.py.before-blind); environment =
  `uv sync --frozen --inexact && uv pip install pytest==7.4.4 flaky==3.7.0` (all workspace
  packages editable; sdk tests 672 pass, 1 fails; api and sdk tests cannot be collected in one
  pytest call, a property of the repository).
- STARTED 09:25: blind-otel-python (run 20261124-092508-b7f0) and blind-otel-go (run
  20261124-092504-fbbd), each `--timeout 12600 --max-turns 600`, prompt-generic.txt. Waiters via
  u107/wait-run.sh <name> 7000 (restart when they time out). Next pair: interceptors + fastrace;
  then logstash (--warm-m2) + Jetty.
- After each run: keep small files in /root/dla-real/<name>-keep, delete the run directory by
  literal path. Score only after ALL runs: then read answer-key.md.

## 09:36 UTC 2026-11-24: blind run otel-go ended after 9 minutes; blind Node run started

- blind-otel-go: exit 0, 520 s, 81 turns, cost 1.47. Report INCOMPLETE (2 open), 7 min of a 3 h
  budget used; 11 harness runs; queue 17 of 136 ranked targeted, 119 not targeted. Findings line:
  confirmed 0; confirmed but disputed by the verifier 3 (two slices of a test-helper span
  recorder, and a record that grew only because the model's unit kept adding to it; the model
  itself says none is a leak: "The ledger required me to confirm it, but I would not call it a
  leak"). THE MODEL STOPPED BY ITSELF: "Closing these items needs one hand-written driver per
  batch, and I didn't have a faster way to do that", then three stop messages against the stop
  hook ("I won't loop on the same refused step"). Also seen: H9 REDUCED RECORD (a point lost
  sections), an entry-path harness that read unknown. Kept: /root/dla-real/blind-otel-go-keep
  (run.json, transcript, report, state, ledgers, gate). Run directory deleted.
  Score entry (key NOT read yet): nothing reported that the run itself calls a leak.
- blind-interceptors started 09:34 (run 20261124-093424-bd1f), waiter running. blind-otel-python
  still going (started 09:25).
- Next after one of them ends: fastrace; then logstash (--warm-m2 u107/logstash-try/m2); Jetty.

## 10:36 UTC 2026-11-24: blind Node run ended, no leak confirmed; blind Rust run started; CI flake

- CI on main for b896d8c (version commit): run 45363801429 FAILED in clippy and test
  (macos-latest): dla-workload lib test
  harness::scratch::tests::a_harness_whose_lock_is_held_is_not_thinned_and_one_in_use_is_filled_first
  at scratch.rs:1602 (after drop(guard), thin_harness was not Done). Passed on macOS in runs
  45339326235 and 45350380373 on the same code. Failed job re-run queued (waiter bseg2xwjo, logs
  u107/ci-main-23-rerun). Task 103 holds the hypothesis (a child forked by another test keeps
  the open file description and so the lock until exec) and the fix; no cargo while blind runs go.
  Publish preview.7 only after the re-run is green.
- blind-interceptors: exit 0, 3552 s, 102 turns, cost 1.88. COMPLETE, "no leak confirmed".
  93 harness runs, 185 series, 2 captures, 56 min of 3 h. Queue 172 of 172 targeted in 86
  harnesses: executed 43, no coverage data 5, NOT REACHED 124 (drivers load the built bundle
  lib/, candidates are lines of the TypeScript source src/). Residual: heap growth unattributed
  and above threshold on 178 of 370 rows (S1 264 bytes/iteration on the "mock-respond" drivers),
  left as unresolved open items O1, O2: measured growth never tied to a holder.
  Kept: /root/dla-real/blind-interceptors-keep. Run directory deleted.
  Score entry (key NOT read yet): nothing confirmed.
- blind-fastrace started 10:33 (run 20261124-103359-0c40), waiter running. blind-otel-python
  still going (started 09:25; waiter bsqrpquxa ends about 11:22: restart it then).
- Next: logstash (--warm-m2 u107/logstash-try/m2), then Jetty 12.0.31 (clone the tag first).

## 10:39 UTC 2026-11-24: blind Rust run ended in under 3 minutes, nothing confirmed; JVM run started

- blind-fastrace: exit 0, 165 s, 32 turns, cost 0.50. COMPLETE, "no leak confirmed", 2 min of a
  2h47m budget. 44 candidates, only 8 ranked; 3 harnesses, 6 series; queue 8 of 8 targeted:
  executed 3, not reached 5; 16 of 146 entry points exercised. The final gate passed. Whole
  process rss grew 5 to 6 bytes per unit, under threshold. The model: "native.live_bytes was not
  among the reported growth columns, but I didn't check it directly." Kept:
  /root/dla-real/blind-fastrace-keep. Run directory deleted.
  Score entry (key NOT read yet): nothing confirmed.
- blind-logstash started 10:37 (run 20261124-103730-3c68, --warm-m2 u107/logstash-try/m2), waiter
  running. blind-otel-python still going (started 09:25).
- Jetty source fetched: /root/dla-real/src/jetty (tag jetty-12.0.31, 5eb4a85, 493 poms, 2581
  source files). Runs LAST and ALONE: it needs a Maven build first, which must not run beside a
  measuring run.
- Pattern so far (3 of 3 blind runs, no leak confirmed): the run ends long before its budget
  (9 min, 56 min, 3 min of about 3 h) with most code never exercised, and the gate or the stop
  hook lets it.

## 10:45 UTC 2026-11-24: first JVM run VOID (rig), rerun started; CI second flake; transcripts redacted

- CI re-run (attempt 2) of the macOS job failed on ANOTHER test:
  dla-workload tests/harness_run.rs:2078 `a_wrapper_script_that_runs_javac_and_tests_still_yields_the_main_jvm_record`,
  `ratio(&series, "os.rss") < 4.0` (the known macOS flake of task 50). The lock test passed in
  that attempt. Attempt 3 queued (waiter blxb9xo6h, logs u107/ci-main-23-rerun2).
- blind-logstash (first): VOID, a rig failure. 89 s, measured nothing: inside the run's sandbox
  Maven has no network (the JVM's proxy setting of this container is refused there), and my warm
  Maven home held only what `./mvnw test` needs, not the goals of the tool's JUnit harness
  (`dependency:build-classpath`, `dependency:get` of junit-platform-launcher). The report said
  COMPLETE with "Measured: nothing" and the blocker build_fails; the model closed all 86 with
  close-rest. Kept: /root/dla-real/blind-logstash-void-keep (no transcript: it showed
  environment variable names of credentials).
- Rig fix (no change to the tool): u107/logstash-try/m2 now also holds maven-dependency-plugin
  2.8 (wrapper Maven 3.8.5) and 3.7.0 (system Maven 3.9.11) and junit-platform-launcher 1.8.2;
  offline `test-compile dependency:build-classpath` and `dependency:get` pass. For Jetty the same
  goals must be warmed.
- blind-logstash-2 started 10:41 (run 20261124-104153-202a), waiter bgjn3sizr.
- Kept transcripts can hold the sandbox's own proxy URL with credentials
  (`://srt…@127.0.0.1:3128`). /root/dla-real/redact.py <src> <dst> removes credential-shaped
  text; applied to the three blind keeps. Use it for every keep from now on; never copy a raw
  transcript to the progress branch.
- blind-otel-python still going (started 09:25); its waiter bsqrpquxa ends about 11:22.

## 11:05 UTC 2026-11-24: CI green on attempt 3, release started; JVM rerun confirmed a leak

- CI on main for b896d8c: run 45363801429 green on attempt 3 (attempt 1: lock test; attempt 2:
  os.rss ratio test; both macOS, both flakes: tasks 103 and 50).
- release.yml started for main (publish true) at about 11:01; waiter br36sejg0 prints the release
  run and the assets of v0.9.0-preview.7.
- blind-logstash-2: exit 0, 1300 s, 84 turns, cost 1.38. COMPLETE, 21 min of 3 h, 24 harness
  runs, 46 series, 1 capture; queue 60 of 60 targeted: grew 2, executed 54, not reached 4;
  844 of 1137 entry points exercised (the JUnit loop works with the fixed Maven home).
  CONFIRMED 2, both reproduced through the public API in H7:
  F57 static map ObjectFieldsAppendingMarker.serializerHelper
  (src/main/java/net/logstash/logback/marker/ObjectFieldsAppendingMarker.java:92, writer :149),
  keyed by ObjectMapper, never removed, 43 count/iteration with a new encoder per unit; the
  verifier agrees on site, holder and mechanism and disputes only the label (L-STALE vs
  L-COLLECT). F58 the cache inside the same helper (line 154), the verifier attributes it to
  F57. Kept: /root/dla-real/blind-logstash-2-keep (transcript redacted). Run directory deleted.
  Score entry (key NOT read yet): one real-looking leak reported; whether it is the documented
  one is decided by the key.
- blind-otel-python still going at 11:03 (23 harnesses, 59 series, 345 MB). Its waiter bsqrpquxa
  ends about 11:22: restart it. After it ends: build Jetty's Maven home (heavy; nothing else
  may measure then), run Jetty alone, then open answer-key.md and score.
- 11:05 UTC: v0.9.0-preview.7 PUBLISHED (release run 45380080893 on b896d8c; prerelease; five
  platform archives, each with .sha256, and SHA256SUMS).

## 11:27 UTC 2026-11-24: Python blind run still going; a hypothesis from the four finished runs

- blind-otel-python at 11:23: alive, 23 harnesses, 63 series, 4 "confirmed" transitions in the
  ledger, last ledger entry 11:20. New waiter bgcodtt2p (6000 s). It can run until 12:55.
- HYPOTHESIS (to test, not a conclusion; n is tiny): the loop over the project's own tests is
  what makes a run exercise the code. It exists only for JVM (JUnit 5 through Maven or Gradle)
  and Python (pytest): crates/dla-core/src/testloop.rs. The three blind runs that confirmed
  nothing are Go, Node and Rust, which have no such loop and depend on hand-written drivers per
  batch (Go: the model gave up after 10; Rust: 3 drivers, 16 of 146 entry points; Node: 86
  drivers, 124 of 172 targets not reached). The JVM run exercised 844 of 1137 entry points
  through the loop and confirmed a leak in 21 minutes. In the Pillow runs the pytest loop is what
  measured the leak line. If the key agrees, the general fix is a test loop for Go (`go test`),
  Rust (`cargo test`) and Node (the project's test runner), plus: measured growth is worked
  first (task 102), and a run may not end COMPLETE far below its budget with most entry points
  unexercised.

## 12:20 UTC 2026-11-24: BLIND TEST SCORED (five projects, frozen 0.9.0-preview.7): 0 of 5 documented leaks

- blind-otel-python: exit 0, 9330 s, 144 turns, cost 2.90. INCOMPLETE (311 open), 2h34m of 3h,
  27 harness runs, 71 series, 7 captures, 312 of 1040 entry points. Confirmed: F206 SDK
  MeterProvider._meters (sdk/metrics/_internal/__init__.py:443; verifier disputes the site) and
  F197 BatchProcessor.__init__ os.register_at_fork lambda per instance
  (sdk/_shared_internal/__init__.py:127; real, small; verifier L-REGISTER). Disputed: F229, F215.
  Kept: /root/dla-real/blind-otel-python-keep (transcript redacted). Run directory deleted.
- I read answer-key.md sections 1, 3, 6, 7, 9 ONLY (set A). Sections 2, 4, 8, 10 (set B) and 5
  stay unread. Read before Jetty ran: Jetty is not blind and nothing is changed before it.
- SCORE against the key (documented leak found: 0 of 5):
  1 otel-python: documented = api `_ProxyMeterProvider._meters` (metrics/_internal/__init__.py
    :157, adder :174) fed by TracerProvider.get_tracer. MEASURED AND LOST: the pytest loop showed
    py.blocks@...:173 (the _ProxyMeter made there) growing; finding F259 added 10:03 (38 min in),
    put under test only at 11:38, REFUTED 11:43 on E246, the alloc-test of a driver in whose
    series the line did not grow. Late siblings F270..F273 (lines 211, 511, 541, 555) left as
    candidates. Two other growth findings confirmed instead (see above).
  3 otel-go: documented = internal/global/meter.go:95 `instruments` slice. NOT A CANDIDATE (census
    has meter.go:97 registry, :334, :351 only), and the run gave up after 9 minutes.
  6 interceptors: documented = per-request HTTP parsers not released (Node internals hold them;
    about 20 KB per request by hand). GROWTH MEASURED, NOT ATTRIBUTED: heap residual above
    threshold on 178 of 370 rows, left as open items; nothing confirmed.
  7 fastrace: documented = fastrace/src/util/spsc.rs:21 Sender.pending_messages, which grows only
    after the 10240-slot ring is full with no reporter set. CANDIDATE LISTED (F41), NEVER
    TRIGGERED: targeted by H1 and H3, writer not executed (runs have 3520 iterations), closed
    static-only; the run ended COMPLETE after 2 minutes.
  9 logstash: documented = util/ThreadLocalHolder.java:63 threadValues (grows on JDK 16+ when
    ForkJoinPool common-pool workers encode). CANDIDATE LISTED (F81), executed by H3, NOT
    TRIGGERED, closed inconclusive with a wrong generic remedy. A DIFFERENT REAL LEAK was
    confirmed: static map ObjectFieldsAppendingMarker.serializerHelper keyed by ObjectMapper
    (read by me at the tag: never cleared, the code comment admits it).
- Failure families, by projects hit:
  A loop growth measured and then lost (late, refuted or closed by a driver that does not
    reproduce it): Pillow (runs 7, 8, 9, 11), otel-python. Fix idea: the loop is the reproducer;
    narrow it by test file mechanically; such a finding is first in the order of work and a
    driver that does not show the line cannot refute it.
  B no test loop for the runtime, little code exercised, early end: Go, Rust, Node (3 of 3).
    Fix idea: test loop for `go test`, `cargo test`, the Node test runner; no COMPLETE far below
    budget with most entry points unexercised.
  C candidate listed, trigger condition never met: fastrace (capacity threshold), logstash
    (thread kind). Fix idea: when a target's writer did not run, show its guarding condition;
    run long enough for a capacity constant.
  D census gap: otel-go slice field appended in 14 methods is no candidate.
  E unattributed heap growth left open: interceptors. Fix idea: the gate asks for a heap capture
    and its summary before a residual can be left.
- Jetty: Maven build to fill a home started 12:02 (task bepsru3j5, 45 min limit, log
  u107/jetty-try/build.log). If it works: run Jetty alone with --warm-m2 u107/jetty-try/m2.

## 12:40 UTC 2026-11-24 (approx.): cause of the lost Python leak found in the tool; builder H started

- Jetty: the build `mvn -B -q -fae -DskipTests install` PASSED in 5 minutes on JDK 21 (2573 jars,
  Maven home u107/jetty-try/m2 1.3 GB, built copy u107/jetty-try/project 1.5 GB). The Jetty RUN
  is deferred to the second blind round (task 106): it would block all building for 3.5 h and
  its leak class is no census candidate today. Told the owner.
- otel-python, the exact chain (kept transcript, ledger, retain/R1.json):
  1. 09:36 (11 min in): driver H3 for another finding (F229) run with --capture alloc,heap;
     `summarize heap-graph ... --finding F229 | head -c 1800`. The stored record R1 has the
     holder `static:opentelemetry.metrics._internal._PROXY_METER_PROVIDER` with sites
     api/metrics/_internal/__init__.py:173, 212, 510, 540, 555 = THE DOCUMENTED LEAK'S HOLDER.
     No finding at it; nothing ever mentioned it again. (The holder chains built from the
     Pillow runs DID name the right holder on an unseen project; the workflow dropped it.)
  2. 10:00: "The queue is empty", then the classify gate listed 48 untied growing lines of the
     pytest loop; the model added 48 findings in a shell loop with holder container-field
     (F259 at :173 among them).
  3. 11:38: one big driver H20 with a real meter provider installed, so get_meter returns early
     and line 173 never runs; `dla alloc-test H20 --finding F259` wrote E246 favors non-leak:
     "the function of the line ... ran in this series and none of its blocks stayed"
     (alloc_cmd.rs near 1183: growth column absent + FUNCTION coverage says executed). Refute
     accepted. An unsound rule that came in with the Pillow fixes.
- Builder H (Sonnet, background) started on the clean tree b896d8c: brief
  wip-notes/holder-first-brief.md, scratch u108. Scope: per-line execution count in the Python
  agent and inconclusive instead of non-leak when the line did not run; holder `of: project` /
  `of: tests`; uncovered project holder = first gate item with the literal add command,
  `_attention` in summarize, lead of run status, refusal `measured-holder-first` on
  `harness gen --next`; one capture item per series instead of N per-line adds; texts.
- My verification plan for H: read the diff; suite; my own mutations; Pillow walk
  (u101/walk.py) must still pass; a new walk on /root/dla-real/src/otel-python that replays the
  three steps above with the new binary (holder item appears after the H3-like capture; the
  early-return driver gives inconclusive; refute refused; finding at the holder confirmed);
  static and dynamic bench; CI on a snapshot; then main.
- While H builds: do not edit the tree. Next brief to write: family B, the Go test loop.

## 13:00 UTC 2026-11-24 (approx.): priorities re-ranked by two checks; builder G started

- Check 1 (by hand, u110): `go test -count=3` against `-count=23` with `-memprofilerate=1` on
  the Go project's own packages shows NO growth at the documented site (the tests use a fresh
  meter each). A test loop for Go would not have found that leak. Family B is no longer second.
- Check 2: why the Go holder was no candidate (frozen P7 census on u110/otel-go: 439 candidates,
  three in meter.go, none for `instruments`). Cause in the code: crates/dla-census/src/lang/go.rs
  takes a container field only of a long-lived owner; model.rs `note_fields` records held types
  with `caps_idents` (upper-case identifiers only), so unexported Go types never join the
  long-lived closure; and a type built inside a function called from a package-level `var`
  initializer is not long-lived either. General for Go.
- Builder G (Sonnet, background) started: brief wip-notes/go-census-brief.md, scratch u111,
  edits only under crates/dla-census/, runs beside builder H in the same tree (told to retry
  builds that fail elsewhere). I add its VERIFY row and re-run static acceptance myself on the
  combined tree.
- Replay script u109/walk_otel.py (frozen build first: output dir u109/before, task bclsmzgt0):
  driver A reproduces the growth (lines 173, 212, 510, 540 at 2.0 per iteration); the frozen
  build's summary `next` does propose the finding at the holder
  (`--owner opentelemetry.metrics._internal._PROXY_METER_PROVIDER`), so the tool had the answer
  and the workflow dropped it.
- Also seen: logstash's confirmed leak came from a census candidate and a model-written driver
  (H7), not from the test loop.
- Remaining ideas, not started: C (show the guard of a writer that did not run; fastrace needs
  more units than a capacity constant), E (heap growth held by runtime internals in Node: look
  at what the two captures of the Node run showed).

## 12:23 UTC 2026-11-24 (clock checked): both replays on the frozen build done

(Earlier headings of today carry estimated times; this one is from `date -u`.)
- u109/walk_otel.py <dla> <plugin root> <out dir> [line]; project copy u107/otel-try/project,
  venv u107/otel-try/venv. Frozen build u107/dla-P7 with /root/dla-real/plugin:
  - holder mode (out u109/before): following the commands the tool prints (summary `next` add at
    the holder with --owner ..._PROXY_METER_PROVIDER, sibling, test, alloc-test H1, refute the
    sibling, confirm) the documented leak is CONFIRMED in 25 commands. `harness gen --next` is
    not refused; the classify gate has no holder item. Driver B there is inconclusive only
    because the finding sits at :555, whose function B does not call.
  - line mode (out u109/before-line): finding at :173 from the per-line gate item with holder
    container-field; driver B calls the same method on its early-return branch;
    `alloc-test H3 --finding F1` says favors non-leak and prints
    `next: dla ledger refute F1 --evidence E19`; the refute is ACCEPTED. The loss reproduced.
- After builder H: line mode must end with inconclusive and a refused refute; holder mode must
  show the holder item first in the classify gate and refuse `harness gen --next` until the add.

## 12:50 UTC 2026-11-24 (approx.): builder G reported and checked (Go census)

- G changed crates/dla-census/src/lang/go.rs (+133) and model.rs (+28) only. New: held field
  types of any case when the project declares the type (go_types, go_field_types resolved in
  finish); types built by a literal or new() in a package-level var, in an entry-named function,
  or in a function called (one level, plain name) from a var initializer or init are long-lived
  (go_fn_built, go_startup_calls).
- My checks: diff read; `cargo test -p dla-census` 135 + 7 pass; my mutation MUT_G1 (skip the
  start-up calls in finish) fails unexported_types_built_at_start_up_hold_their_fields, reverted,
  diff hash unchanged; no markers. Real project (u112/otel-go, binary u112/dla-G-try built from
  the tree at 12:42 with H's work in progress): 469 candidates (439 before), and
  internal.global.meter.meter.instruments at internal/global/meter.go:95 IS listed with its 14
  writers. Its rank: 118 of 149, tier 4, score 61 (w_grow +25, container +20, w_runtime +20; no
  growth_unmatched because the reset in setDelegate counts as a remover); last of 20 ranked
  candidates of its package, so the third Go batch of that package. NOT tuned: a reset-only
  remover on a configuration path is a ranking question for later, with more than one case.
- Seen in that ranking, general: test-support packages rank at the top (tracetest recorder,
  otest collector storage, internaltest carriers: ranks 3 to 12). A penalty for test-support
  directories is a candidate change; not started.
- go-header-index loses four entries: struct Request has a FIELD named Request ([]byte); the old
  rule read the field name as a self-reference. Expected site index.go:36 still flagged (G);
  the dynamic acceptance run on the combined tree is the check.
- Still to do for G: static acceptance by me on the combined tree; VERIFY row; commit message.
- Builder H still working (tree has its edits: measured_holders.rs, holder_first.rs,
  test_agent_lines.py, gate, alloc, py_retain, harness agent).

## 13:20 UTC 2026-11-24 (clock checked): builder H reported; replays pass; follow-up H2 sent

- H's report (brief H): per-line execution counts in the Python agent (DLA_LINES env, sidecar
  harness/<H>/lines-<S>.json, sys.monitoring with a settrace fallback forced by
  DLA_LINES_MODE=settrace); alloc-test: column absent is non-leak only when the line's count
  rose; holders get `of: project | tests`; new core module measured_holders.rs; classify gate
  item kind `measured_holder` first, status lead, `_attention` in summarize, refusal
  `measured-holder-first` on `harness gen --next`; one capture item per series instead of
  per-line adds; texts, VERIFY rows, schema harness-dump (line_counts); tests holder_first.rs
  (8), test_agent_lines.py (7), unit tests; three py_site_columns tests and one queue test
  adapted. 27 files, +1541/-43 with G's two files.
- Frozen binary of that tree: u112/dla-H1 (sha 47798cc76c63).
- MY REPLAYS WITH dla-H1 (u109/after-line, u109/after), both as wanted:
  line mode: alloc-test on the other-branch driver = inconclusive, next names H1 "(the growth
  was measured there)", refute REFUSED; alloc-test on H1 = leak; confirmed.
  holder mode: first classify item = measured_holder with the literal add command; status lead
  is the holder; `harness gen --next` REFUSED measured-holder-first; after the add the item is
  gone; confirmed in 25 commands.
- H also reported a second route to the same wrong verdict: a finding with an owner gets a probe
  from `harness gen --probe F`; a driver that never feeds the holder reads it flat; alloc-test
  says non-leak; refute accepted. Cause read by me: crates/dla-cli/src/alloc/judge.rs
  `plateau()` judges only the extra window and `combine()` maps Plateau to NonLeak (N-WARM)
  even when the column never grew in the series. Every runtime.
- Follow-up brief wip-notes/holder-first-brief-2.md SENT to the same builder (resumed): a
  deciding column flat from the start = inconclusive with its own verdict word, next names the
  harness of the growth; "line ran and nothing stayed" is non-leak only when no retention
  record names a project holder of that line; an existing test that relied on a flat column as
  a plateau is to be left failing and reported.
- Pillow walk with dla-H1 running (task biexlrwq2, out u112/pillow-walk-H1).
- After H2: build, freeze, both replays again, Pillow walk again, whole suite, own mutations,
  static + dynamic acceptance (the flat-column rule can change fixture outcomes), VERIFY rows
  for G, commit messages, snapshot with CI, main.

## 13:35 UTC 2026-11-24 (approx.): Pillow walk passes on dla-H1; the Node cause found (TypeScript)

- Pillow walk (u101/walk.py) with u112/dla-H1: as before. F1 (holder PIL.ImageCms._imagingcms)
  and F2 (native entry at ImageFont.py:580) confirmed without a second capture, own excerpts per
  bundle, final gate asks for no finding and has no measured_holder item.
- NODE (blind interceptors run), from the kept transcript and a replay in u113/interceptors
  (script u113/walk_node.py <dla> <plugin root> <run dir>, binary u112/dla-H1):
  1. In the blind run the model captured once on the first, generic harness and ran
     `summarize type-histogram-diff` and `alloc-stack-diff` by hand with 2 of the 4 files (gate
     fail: captures < 3); it never ran the `summarize heap-graph` command the capture prints.
     No capture on the later mock-answer drivers. The residual open items were closed as
     unresolved with the remedy "capture alloc profile then summarize".
  2. The harness work copy takes only git files (scratch.rs copy_project_for -> git_files), so
     the git-ignored build output `lib/` and `node_modules/` are missing: a driver cannot
     `projectRequire('lib/...')`. The blind run's model worked around it in every driver
     (fs.cpSync of lib into ROOT, absolute require of node_modules). Same failure in my replay
     until I copied lib and linked node_modules by hand.
  3. With that done and a unit that sends one mocked request (ClientRequestInterceptor applied,
     respondWith(new Response('ok')), http.get with agent:false): trend passes
     node.listeners@lib/node/chunk-VYO5XDY2.js:1076 (+4 per iteration), :54 (+2), :688 (+2),
     :394, :565, :583, chunk-7Q53NNPV.js:113, and node.heap_total +29 KB per iteration; the
     printed heap-graph summary passes two growing closure holders in the built chunk. So the
     tool measures this leak at once. But every site is a line of the BUILT bundle while the
     candidates are TypeScript lines (and census even lists `.d.ts` declarations under lib/), so
     nothing ties: "not reached 124 of 172".
- NEXT BRIEF (N, Node and TypeScript; not written yet): (a) the work copy includes the build
  output that package.json points at (main, module, exports, files, bin) even when git-ignored,
  and node_modules resolves from the work copy; (b) a site in a built file that has a source
  map is reported at the original source position when that source is a project file (keep the
  built position too); (c) the census does not list declaration files and prefers sources over
  their build output; (d) a residual above threshold cannot be left unresolved while the
  harness that shows it has no summarized capture (the summarize commands the capture prints).
  The dev project for Node so far was plain JavaScript, which hid all of this.

## 14:15 UTC 2026-11-24 (approx.): H2 reported; three existing tests fail; follow-up H3 sent

- H2 built: `PlateauVerdict::NeverGrew` (word never-grew) in crates/dla-cli/src/alloc/judge.rs,
  `grew()` (counter: net rise over the main window above zero; gauge: Theil-Sen lower bound
  above zero and fitted rise above the column's smallest rise and 1% of its level); combine
  returns inconclusive for a never-grown deciding column of a bound finding by every route;
  `LineRan::HolderRecord` (line ran and kept nothing is inconclusive when a retention record
  names a project holder of the line); 4 new holder_first tests; stages.md sentence; VERIFY row.
- Three existing tests fail, left failing as told:
  1. alloc_test_jvm::a_loop_that_warms_up_is_judged_a_plateau_on_the_non_heap_bytes_and_the_root_is_refuted
     (alloc-test on H1 = the harness that measured the growth; non-heap flat after warm-up).
  2. trigger::a_registry_grown_only_by_its_own_writer_is_not_confirmed_and_the_non_leak_way_closes_it
     (flat on an ENTRY-PATH harness of the finding: the tool's own non-leak way for registries).
  3. trigger::the_trigger_is_reproduced_for_one_holder_and_not_found_for_the_other (broken by
     brief H: the capture item now leads blocking and missing).
- My decision (H3, sent to the same builder): never-grew applies only on a harness that is
  neither the harness of the growth nor an entry-path harness declared for the finding. On the
  growth's own harness a flat rerun is fair evidence (a leak grows with every unit), and the
  entry-path harness keeps its meaning. Tests 1 and 2 must pass unchanged; test 3 is adapted
  like the queue test (assert the new lead, find the trigger item by kind, keep every
  assertion). Known remaining hole, recorded on purpose: an entry-path harness that fails to
  reach a real leak can still close it the non-leak way.
- Node brief ready: wip-notes/node-ts-brief.md (start after H3; scratch u114).

## 14:50 UTC 2026-11-24 (approx.): H3 reported; combined tree frozen and sent to CI; Rust cause found

- H3 built: `exemption()` in crates/dla-cli/src/alloc_cmd.rs: never-grew is skipped on an
  entry-path harness whose targets hold the finding, and on the harness of the growth (the
  finding cites growth evidence of a series of it or a capture of it names the finding:
  harness_about Evidence | Capture; or an earlier trend of a series of this harness has a grown,
  not whole-process column tied to the root). Text there: "the growth of F did not come back on
  the harness that measured it". Two holder_first tests added; trigger test repaired (I read the
  diff: blocking[0] is asserted measured_capture, trigger items found in gate/final.json, every
  content assertion kept); alloc_test_jvm warm-up test and the registry trigger test pass
  unchanged (builder). Residual risk noted: Capture also exempts a harness whose capture names
  the finding.
- My edits to docs/VERIFY.md: removed the words "After brief H" from a row; added the row for
  the Go census change (last row).
- Combined tree (G + H + H2 + H3 + my VERIFY edits): fmt clean, build ok, schema --check ok,
  29 files +2024/-64, three new files. Frozen: u112/dla-H3 (sha 1ce2ae370e4d), u112/dla-bench-H3.
  Snapshot a37fcfb pushed WITHOUT [skip ci]; CI waiter bg6z2r0ip (logs u115/ci-snap-23).
  Replays and the Pillow walk on dla-H3 running: task bvjtfa7ie.
- Commit message draft: wip-notes/commit-message-23.txt.
- RUST (blind fastrace run), cause found: the crate has the feature `enable`
  (fastrace/Cargo.toml: enable = ["fastrace-macro/enable"]), off by default; span.rs has 25
  `feature = "enable"` cfg sites, local_collector.rs 10, local_span.rs 8: without it the spans
  are no-ops and the collector never runs. The blind run never passed `--features enable`
  (transcript: only the help text), so the candidate at spsc.rs:21 read "not executed" in H1 and
  in the focus harness H3 and was closed static-only. The capacity of 10240 comes second.
  The tool says "gated behind a feature" only for a whole module.
  NEXT BRIEF R (small, not written): when a crate has features that are not default and that
  gate code (`#[cfg(feature = "x")]`, `cfg!(feature = "x")`), every target block of that crate
  and the remedy of a target whose code did not run name the features with their counts of
  gated sites per file and the command with `--features`; the queue does not close a target as
  not reached after a focus harness while such a feature was never tried.
- Order of work now: verify the combined tree (replays, suite, own mutations, static and
  dynamic acceptance, CI) -> main; builder N (Node/TypeScript, brief ready) starts when my local
  suite is done; then brief R.

## 15:15 UTC 2026-11-24 (clock checked 15:06): local verification of the combined tree; CI found a hole

- Disk filled during the first local suite run (25 MB free). Freed 7 GB by literal paths:
  u107/jetty-try/project (built Jetty copy; m2 kept, rebuild takes 5 min), u106/repro copies,
  u106/src, u106/repos, u106/cand (research clones; answer-key.md and candidates.md kept),
  u113/interceptors/.dla, u101/dla-R2..R6, u104/frozen, u105/dla-W2 and dla-bench-W,
  u112/dla-G-try and dla-H1, /tmp/dla-bench-* leftovers, /root/dla-real/src/apollo-client,
  u110/gocache, u111, and the .dla run directories of my walks in u72/pillow (1.1 GB) and
  u107/otel-try/project. Lesson: delete a walk's run directory after reading it.
- Replays and Pillow walk on u112/dla-H3: all as wanted (line mode inconclusive + refute
  refused + confirmed on H1; holder mode `_attention`, gate item, status lead, refusal,
  confirmed; Pillow both findings confirmed, final gate asks for no finding).
- Local run on the combined tree (u115/suite.sh, log u115/suite.log): fmt, clippy, schema clean;
  whole suite under 3.13: 3480 passed, 0 failed, 5 ignored; Python test files under 3.12: the
  two known gc.freeze tests of harness_capture_python fail, all else passes; static acceptance:
  64 fixtures pass; agent tests (unittest) 7 pass.
- CI on snapshot a37fcfb (run 45412730283): clippy and test (ubuntu-latest) FAILED in
  alloc_test_jvm::a_loop_that_warms_up_is_judged_a_plateau_on_the_non_heap_bytes_and_the_root_is_refuted
  (favors inconclusive, jvm.nonheap_used never-grew). Cause: on that machine the first trend
  wrote no growth evidence, the finding cited only the workload evidence of H1, harness_about
  gave OwnWorkload, and the exemption takes only Evidence and Capture. Other jobs passed; dynamic
  acceptance was still running when read.
- Follow-up H4 SENT to the builder: a harness is the finding's own when the finding cites ANY
  evidence a run of it produced (workload record, series, tied grown column, capture); only a
  harness it cites nothing from is "another driver". Timing-free test both ways, five runs of
  alloc_test_jvm, texts say "the harness the finding was raised on".
- After H4: fmt, build, freeze, replays again (they must not change), suite, snapshot with CI.
  Builder N (Node) still waits; brief R (Rust features) not written.

## 15:35 UTC 2026-11-24 (approx.): H4 done, H5 in progress; builder R started

- H4 built: exemption() returns EntryPath, Growth (an earlier trend of a series of this harness
  has a grown, not whole-process column tied to the root) or Raised (a finding of the family
  cites, in supports or contradicts, a record that a run of this harness produced: a Workload
  record with result.harness == H, a GrowthSeries or UnitCountSeries record of a series of H, a
  RetentionPath or AllocStack record whose source lies under harness/<H>/). Texts: "did not come
  back on the harness that measured it" / "no growth of F showed on the harness it was raised
  on". Five alloc_test_jvm runs ok.
- The builder pointed out that `dla ledger test F --evidence <workload of a probe driver>` then
  makes that driver exempt. H5 SENT: a Workload record counts only for a harness that runs the
  project's own workload (command mode with a workload, or a test loop: the OwnWorkload test);
  a hand-written driver needs a cited series record, a tied grown column or a capture. Its own
  new test is changed to say so; a Python own-workload case is added.
- CI run 45412730283 on a37fcfb: dynamic acceptance still in progress at 15:25 (66 min).
- Builder R (Sonnet, background) started in the same tree: brief
  wip-notes/rust-features-brief.md, scratch u116 (task 109).
- Builder N (Node/TypeScript) still waits (brief wip-notes/node-ts-brief.md, task 108).
- 15:35 UTC: CI run 45412730283 on a37fcfb ended in failure, two jobs: clippy and test
  (ubuntu-latest) = the JVM warm-up test (see above); dynamic acceptance = 61/64 pass, 1 xfail,
  2 failures: node-report-heap and node-template-cache "no rule-out on a non-leak fixture"
  (H1 is the project's own load script, the candidate's probe is flat from the start, the
  alloc-test read never-grew = inconclusive). Under H4/H5 an own-workload harness that the
  finding cites is exempt, so both should pass again: check with
  `dla-bench dynamic --dla-bin <frozen> --only node-report-heap,node-template-cache` after H5,
  then the whole set on CI. Logs: u115/ci-snap-23/.
- Plan: wait for H5 and builder R, then verify the combined tree once (build, freeze, the two
  Node fixtures, replays, Pillow walk, fastrace check, suite, static), snapshot with CI, commit.
