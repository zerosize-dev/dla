# DLA report

Run: `20261110-192215-532e`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 7 enumerated, 7 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 5

Residual: heap 10112.886666666667 bytes/n unattributed of 10112.886666666667 total (threshold 2528.221666666667) ABOVE THRESHOLD, see O1; heap 10103.99090909091 bytes/n unattributed of 10103.99090909091 total (threshold 2525.9977272727274) ABOVE THRESHOLD, see O2; heap 10104.010606060603 bytes/n unattributed of 10104.010606060603 total (threshold 2526.0026515151512) ABOVE THRESHOLD, see O3; off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0)

Coverage: node 3520 iterations, 10 of 11 entry points exercised

## Findings

### F5: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Closure context in Board.publish: changeNote closure captures 'previous' (prior snapshot); snapshot.describe shares that V8 context, so every snapshot retains its predecessor -> unbounded linked chain of snapshots with their ticks/averages arrays)
- Growth: `holder:closure:.dla.20261110-192215-532e.harness.H1.work.src.board.previous` 11855360 bytes/n (steady, gate pass, E16)
- Evidence classes: context, growth, retention
- Holder: closure/continuation
- Owner: `.dla.20261110-192215-532e.harness.H1.work.src.board.previous`
- Region: heap
- Shape: steady
- Site: `src/board.js:11`
- Writers: `src/board.js:11`
- Removers: `src/board.js:40`
- Annotation: closure variable previous (Board.publish) captured by changeNote and snapshot.describe share one V8 context, so each snapshot retains its predecessor
- Annotation: no removal: reset() only clears latest, not the previous chain; previous is never nulled
- Verified: no
- Sibling hypotheses: F5.n1
- Supporting evidence: E15, E9, E16
- Explanation (inferred): Closure context of Board.publish at src/board.js:11 holds previous, shared by changeNote and snapshot.describe; each snapshot retains its predecessor; reset at src/board.js:40 is never reached from publish.
- Explanation structure: adds: src/board.js:11; removal: unreachable at src/board.js:40

### F1: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: container-field
- Owner: `src.board.Board.latest`
- Region: heap
- Site: `src/board.js:7`
- Writers: `src/board.js:31`
- Removers: `src/board.js:40`
- Annotation: heuristic name match over the project's sources
- Reason: time_budget
- Remedy: census/heap-graph lead subsumed by F5 (the growth is the previous-snapshot chain via closure); Board.latest, Feed.random, Feed.prices are bounded single/8-entry holders
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: container-field
- Owner: `src.feed.Feed.random`
- Region: heap
- Site: `src/feed.js:17`
- Reason: time_budget
- Remedy: census/heap-graph lead subsumed by F5 (the growth is the previous-snapshot chain via closure); Board.latest, Feed.random, Feed.prices are bounded single/8-entry holders
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: container-field
- Owner: `src.feed.Feed.prices`
- Region: heap
- Site: `src/feed.js:19`
- Writers: `src/feed.js:30`
- Annotation: heuristic name match over the project's sources
- Reason: time_budget
- Remedy: census/heap-graph lead subsumed by F5 (the growth is the previous-snapshot chain via closure); Board.latest, Feed.random, Feed.prices are bounded single/8-entry holders
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: retention
- Holder: closure/continuation
- Owner: `.dla.20261110-192215-532e.harness.H1.work.src.board.board`
- Region: heap
- Reason: time_budget
- Remedy: census/heap-graph lead subsumed by F5 (the growth is the previous-snapshot chain via closure); Board.latest, Feed.random, Feed.prices are bounded single/8-entry holders
- Supporting evidence: E15

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: closure/continuation
- Owner: `Board.publish.previous`
- Region: heap
- Shape: steady
- Site: `src/board.js:11`
- Reason: not_exercised
- Remedy: duplicate of F5 (same column set); covered by F5
- Supporting evidence: E22, E23

## Ruled out

- **F5.n1** (UNCLASSIFIED): discriminator E24
  - Ruled out by: plateau+purge (leak) E24
  - Hypothesis: N-WARM: chain growth is a warm-up fill that plateaus
  - Refuted if: retained snapshot count keeps growing linearly with updates after warm-up and after gc purge
  - Contradicting evidence: E24
  - Originally supported by: E16

## Blind spots and unknowns

### Coverage gaps

- node/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- node/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)

### Runtime blind spots

None recorded.

### Entry points not exercisable

None recorded.

### Evidence limitations and unknowns

- E4: partial coverage
- E4: limitation: rxjs-x rules need rxjs to resolve from the project's node_modules for type information
- E6: partial coverage
- E6: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E6: limitation: writers and removers are textual name matches
- E7: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E7: limitation: workloads were discovered by reading files; none was run
- E8: limitation: harness written; nothing was run (dla harness run measures it)
- E8: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E9: partial coverage (9 of 9 examined)
- E9: limitation: command-mode heap dumps come from separate processes: dla summarize heap-graph compares dumps of one process only, so use a driver (dla harness gen --driver) for a dump series
- E9: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E9: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds som…
- E9: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that…
- E9: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E10: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E10: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E10: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E10: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E11: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E11: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E11: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E12: limitation: weak edges are skipped; string contents are never read into the summary
- E13: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E13: limitation: weak edges are skipped; string contents are never read into the summary
- E14: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E14: limitation: weak edges are skipped; string contents are never read into the summary
- E15: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E15: limitation: weak edges are skipped; string contents are never read into the summary
- E16: limitation: dump-series rule: at least 3 snapshots, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E16: limitation: series: per-point; one process per point at n = 440, 1760, 2640, 3960 (harness captures; checked: snapshot format)
- E17: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E18: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E19: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E20: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E21: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E22: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E22: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E22: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E23: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E23: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E24: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E24: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E24: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E24: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E24: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E24: limitation: points 1 to 9 come from S1; the extra points from S3
- E24: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E24: limitation: gate and plateau thresholds are provisional (not calibrated)
- E25: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E25: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E26: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: partial coverage (2 of 3 examined)
- E27: limitation: F5: growth evidence is from S2, not S1; not subtracted
- E27: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E27: limitation: threshold is provisional (not calibrated)
- E28: partial coverage (2 of 3 examined)
- E28: limitation: F5: growth evidence is from S2, not S3; not subtracted
- E28: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E28: limitation: threshold is provisional (not calibrated)
- E29: partial coverage (2 of 3 examined)
- E29: limitation: F5: growth evidence is from S2, not S4; not subtracted
- E29: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E29: limitation: threshold is provisional (not calibrated)
- E30: partial coverage (2 of 3 examined)
- E30: limitation: F5: growth evidence is from S2, not S4; not subtracted
- E30: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E30: limitation: threshold is provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | — |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | — |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | — |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E3 | — |
| `intake.build` | Does the project build locally? | answered | E3 | — |
| `intake.workload` | What can exercise the suspected path? | answered | E3 | — |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo d42e192d932296cb766bd14caf0ede25d1806902 | — |
| E2 | run-config | static | dla 0.1.0 | repo d42e192d932296cb766bd14caf0ede25d1806902 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo d42e192d932296cb766bd14caf0ede25d1806902 | probe/probe.json |
| E4 | static-hit | static | eslint 10.14.0 | repo d42e192d932296cb766bd14caf0ede25d1806902 | sweep/node-eslint.json |
| E5 | static-hit | static | dla 0.1.0 | repo d42e192d932296cb766bd14caf0ede25d1806902 | sweep/node-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo d42e192d932296cb766bd14caf0ede25d1806902 | census/node.json |
| E7 | workload | static | dla 0.1.0 | repo d42e192d932296cb766bd14caf0ede25d1806902 | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo d42e192d932296cb766bd14caf0ede25d1806902 | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (b6054a5077c5) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (73fda86738b2) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (73fda86738b2) | trend/T1.json |
| E12 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-192215-532e/harness/H1/out/p1-native/capture/9/heap-1.heapsnapshot (714a9087afbe) | summaries/heap-1-714a9087.json |
| E13 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-192215-532e/harness/H1/out/p4-native/capture/34/heap-1.heapsnapshot (af63d68c8bf1) | summaries/heap-1-af63d68c.json |
| E14 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-192215-532e/harness/H1/out/p6-native/capture/50/heap-1.heapsnapshot (db48ae0c5950) | summaries/heap-1-db48ae0c.json |
| E15 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-192215-532e/harness/H1/out/p9-native/capture/74/heap-1.heapsnapshot (f37e6f168c81) | summaries/heap-1-f37e6f16.json |
| E16 | growth-series | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (bf0f33f21e2d) | series/S2.json |
| E17 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (bf0f33f21e2d) | series/S2.json |
| E18 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (bf0f33f21e2d) | series/S2.json |
| E19 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (bf0f33f21e2d) | series/S2.json |
| E20 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (bf0f33f21e2d) | series/S2.json |
| E21 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (bf0f33f21e2d) | series/S2.json |
| E22 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (bf0f33f21e2d) | trend/T2.json |
| E23 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (bf0f33f21e2d) | trend/T2.json |
| E24 | discriminator | test | dla 0.1.0 | artifact series/S4.json (c41aa0cbbf88) | alloc/A1.json |
| E25 | growth-series | test | dla 0.1.0 | artifact series/S3.json (b32b47975721) | trend/T4.json |
| E26 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (b32b47975721) | trend/T4.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S1.json (73fda86738b2) | gap/residual-S1-E27.json |
| E28 | residual | test | dla 0.1.0 | artifact series/S3.json (b32b47975721) | gap/residual-S3-E28.json |
| E29 | residual | test | dla 0.1.0 | artifact series/S4.json (c41aa0cbbf88) | gap/residual-S4-E29.json |
| E30 | residual | test | dla 0.1.0 | artifact series/S4.json (c41aa0cbbf88) | gap/residual-S4-E30.json |
