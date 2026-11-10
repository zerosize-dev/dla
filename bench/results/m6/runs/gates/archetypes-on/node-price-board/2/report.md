# DLA report

Run: `20261110-164944-1b7b`

Status: INCOMPLETE (11 open)

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 8 enumerated, 8 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 6

Residual: heap 10112.904545454545 bytes/n unattributed of 10112.904545454545 total (threshold 2528.226136363636) ABOVE THRESHOLD, see O2; off-heap 0 bytes/n unattributed of 0 total (threshold 0)

Coverage: node 3520 iterations, 10 of 11 entry points exercised

## Open items

- **O2**: open item O2: unexplained growth: 10113 bytes/n in heap (S1); add a hypothesis (dla ledger add --origin residual --evidence E37 ...) or record it unresolved. Fix: `dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"`
- **T1:os.rss**: column os.rss of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:os.rss_anon**: column os.rss_anon of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:os.rss_file**: column os.rss_file of T1 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:os.vm_data**: column os.vm_data of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:os.vm_hwm**: column os.vm_hwm of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:node.heap_total**: column node.heap_total of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:node.heap_used**: column node.heap_used of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:node.heap_total.pre_gc**: column node.heap_total.pre_gc of T1 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:node.heap_used.pre_gc**: column node.heap_used.pre_gc of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **heap**: residual growth in heap (10112.905 bytes/n, threshold 2528.226) has no resolved open item. Fix: `dla gap --residual then resolve the open item it raises`

## Findings

### F5: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (closure variable 'previous' in Board.publish (src/board.js:14), kept in a V8 context shared by describe() and changeNote(), chains every snapshot to its predecessor)
- Growth: `holder:closure:.dla.20261110-164944-1b7b.harness.H1.work.src.board.previous` 11855360 bytes/n (steady, gate pass, E16)
- Evidence classes: context, growth, retention
- Holder: closure/continuation
- Owner: `.dla.20261110-164944-1b7b.harness.H1.work.src.board.previous`
- Region: heap
- Shape: steady
- Site: `src/board.js:14`
- Writers: `src/board.js:14`, `src/board.js:15`, `src/board.js:20`
- Annotation: closure context shared by changeNote and describe keeps 'previous' snapshot alive
- Annotation: describe() and changeNote() share one V8 context; it holds previous, which holds the prior snapshot, so snapshots chain unboundedly
- Verified: no
- Sibling hypotheses: F5.n1
- Supporting evidence: E15, E9, E16, E11
- Explanation (inferred): Board.publish captures previous (src/board.js:14) in a closure context shared with snapshot.describe; each snapshot keeps its predecessor alive via the closure chain, which is never released.
- Explanation structure: adds: src/board.js:14; removal: never

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.board.Board.latest`
- Region: heap
- Site: `src/board.js:7`
- Writers: `src/board.js:31`
- Removers: `src/board.js:40`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: same retention chain as F5 (or unrelated census lead); fixing F5 closure capture addresses it; no separate growth tied
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.feed.Feed.random`
- Region: heap
- Site: `src/feed.js:17`
- Reason: not_exercised
- Remedy: same retention chain as F5 (or unrelated census lead); fixing F5 closure capture addresses it; no separate growth tied
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.feed.Feed.prices`
- Region: heap
- Site: `src/feed.js:19`
- Writers: `src/feed.js:30`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: same retention chain as F5 (or unrelated census lead); fixing F5 closure capture addresses it; no separate growth tied
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: retention
- Holder: closure/continuation
- Owner: `.dla.20261110-164944-1b7b.harness.H1.work.src.board.board`
- Region: heap
- Reason: not_exercised
- Remedy: same retention chain as F5 (or unrelated census lead); fixing F5 closure capture addresses it; no separate growth tied
- Supporting evidence: E15

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: closure/continuation
- Owner: `Object.describe`
- Reason: not_exercised
- Remedy: same retention chain as F5 (or unrelated census lead); fixing F5 closure capture addresses it; no separate growth tied
- Supporting evidence: E22, E23

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: closure/continuation
- Owner: `Object.describe`
- Reason: not_exercised
- Remedy: same snapshot chain as F5; fixed by releasing the closure capture
- Supporting evidence: E34, E35

## Ruled out

- **F5.n1** (UNCLASSIFIED): discriminator E21
  - Ruled out by: dump-plateau (leak) E21
  - Hypothesis: N-WARM: growth is warm-up fill that plateaus, not a leak
  - Refuted if: dump-plateau discriminator favors leak (growth continues across all intervals)
  - Contradicting evidence: E21
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
- E24: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E24: limitation: weak edges are skipped; string contents are never read into the summary
- E25: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E25: limitation: weak edges are skipped; string contents are never read into the summary
- E26: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E26: limitation: weak edges are skipped; string contents are never read into the summary
- E27: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E27: limitation: weak edges are skipped; string contents are never read into the summary
- E28: limitation: dump-series rule: at least 3 snapshots, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E28: limitation: series: per-point; one process per point at n = 440, 1760, 2640, 3960 (harness captures; checked: snapshot format)
- E29: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E30: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E31: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E32: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E33: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E34: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E34: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E34: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E34: limitation: gate thresholds are provisional (not calibrated)
- E35: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E35: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E35: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E35: limitation: gate thresholds are provisional (not calibrated)
- E36: partial coverage (2 of 3 examined)
- E36: limitation: F5: no passing column is tied to it; nothing attributed
- E36: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E36: limitation: threshold is provisional (not calibrated)
- E37: partial coverage (2 of 3 examined)
- E37: limitation: F5: no passing column is tied to it; nothing attributed
- E37: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E37: limitation: threshold is provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | — |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | — |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | — |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | — |
| `intake.build` | Does the project build locally? | answered | E1 | — |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | — |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E1 | — |
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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (6aaa24d210b7) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (033c25ccd2f7) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (033c25ccd2f7) | trend/T1.json |
| E12 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-164944-1b7b/harness/H1/out/p1-native/capture/9/heap-1.heapsnapshot (04c8fbc719e5) | summaries/heap-1-04c8fbc7.json |
| E13 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-164944-1b7b/harness/H1/out/p4-native/capture/34/heap-1.heapsnapshot (68701857e0ba) | summaries/heap-1-68701857.json |
| E14 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-164944-1b7b/harness/H1/out/p6-native/capture/50/heap-1.heapsnapshot (7b80a9c6b5e8) | summaries/heap-1-7b80a9c6.json |
| E15 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-164944-1b7b/harness/H1/out/p9-native/capture/74/heap-1.heapsnapshot (4c2f85cfb649) | summaries/heap-1-4c2f85cf.json |
| E16 | growth-series | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (a81c33f71257) | series/S2.json |
| E17 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (a81c33f71257) | series/S2.json |
| E18 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (a81c33f71257) | series/S2.json |
| E19 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (a81c33f71257) | series/S2.json |
| E20 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (a81c33f71257) | series/S2.json |
| E21 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (a81c33f71257) | series/S2.json |
| E22 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (a81c33f71257) | trend/T2.json |
| E23 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (a81c33f71257) | trend/T2.json |
| E24 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-164944-1b7b/harness/H1/out/p1-native/capture/9/heap-1.heapsnapshot (04c8fbc719e5) | summaries/heap-1-04c8fbc7.json |
| E25 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-164944-1b7b/harness/H1/out/p4-native/capture/34/heap-1.heapsnapshot (68701857e0ba) | summaries/heap-1-68701857.json |
| E26 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-164944-1b7b/harness/H1/out/p6-native/capture/50/heap-1.heapsnapshot (7b80a9c6b5e8) | summaries/heap-1-7b80a9c6.json |
| E27 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-164944-1b7b/harness/H1/out/p9-native/capture/74/heap-1.heapsnapshot (4c2f85cfb649) | summaries/heap-1-4c2f85cf.json |
| E28 | growth-series | dump | dla-v8-snapshot 0.1.0 | artifact series/S3.json (558ef45a74f1) | series/S3.json |
| E29 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S3.json (558ef45a74f1) | series/S3.json |
| E30 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S3.json (558ef45a74f1) | series/S3.json |
| E31 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S3.json (558ef45a74f1) | series/S3.json |
| E32 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S3.json (558ef45a74f1) | series/S3.json |
| E33 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S3.json (558ef45a74f1) | series/S3.json |
| E34 | growth-series | dump | dla 0.1.0 | artifact series/S3.json (558ef45a74f1) | trend/T3.json |
| E35 | unit-count-series | dump | dla 0.1.0 | artifact series/S3.json (558ef45a74f1) | trend/T3.json |
| E36 | residual | test | dla 0.1.0 | artifact series/S1.json (033c25ccd2f7) | gap/residual-S1-E36.json |
| E37 | residual | test | dla 0.1.0 | artifact series/S1.json (033c25ccd2f7) | gap/residual-S1-E37.json |
