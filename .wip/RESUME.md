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
