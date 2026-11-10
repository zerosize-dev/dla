# DLA report

Run: `20261110-161003-08fa`

Status: INCOMPLETE (16 open)

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 9 enumerated, 9 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 7

Residual: not measured

Coverage: node 7040 iterations, 10 of 11 entry points exercised

## Open items

- **T1:os.rss**: column os.rss of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:os.rss_anon**: column os.rss_anon of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:os.vm_data**: column os.vm_data of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:os.vm_hwm**: column os.vm_hwm of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:node.heap_total**: column node.heap_total of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:node.heap_used**: column node.heap_used of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:node.heap_total.pre_gc**: column node.heap_total.pre_gc of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:node.heap_used.pre_gc**: column node.heap_used.pre_gc of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:os.rss**: column os.rss of T3 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:os.rss_anon**: column os.rss_anon of T3 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:os.vm_data**: column os.vm_data of T3 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:os.vm_hwm**: column os.vm_hwm of T3 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:node.heap_total**: column node.heap_total of T3 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:node.heap_used**: column node.heap_used of T3 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:node.heap_total.pre_gc**: column node.heap_total.pre_gc of T3 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:node.heap_used.pre_gc**: column node.heap_used.pre_gc of T3 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`

## Findings

### F5: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Board.publish: describe() and the changeNote arrow share one V8 function context that holds 'previous' (src/board.js:15), so each snapshot retains its predecessor, and through it all ticks/averages arrays; the chain grows by one snapshot per publish. Looks like closure-capture but the retention-path record is tied to the board closure (F4) not F5, so L-CAPTURE evidence is not met.)
- Growth: `holder:closure:.dla.20261110-161003-08fa.harness.H1.work.src.board.previous` 16301120 bytes/n (steady, gate pass, E16)
- Evidence classes: context, growth, retention
- Holder: closure/continuation
- Owner: `.dla.20261110-161003-08fa.harness.H1.work.src.board.previous`
- Region: heap
- Shape: steady
- Site: `src/board.js:15`
- Writers: `src/board.js:15`
- Annotation: previous captured by changeNote closure; describe() shares the same V8 context so every snapshot retains its predecessor
- Verified: no
- Sibling hypotheses: F5.n1, F5.n2
- Supporting evidence: E15, E8, E9, E16
- Explanation (observed): Board.publish at src/board.js:15 stores previous=this.latest in a function context shared by describe() and changeNote, so each new snapshot keeps the prior snapshot alive; it is never released, so the whole chain of ticks arrays grows.
- Explanation structure: adds: src/board.js:15; removal: never; evidence: E29

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
- Remedy: Same retention chain as F5 (snapshot->previous snapshot); symptom, fix at F5 src/board.js:15
- Supporting evidence: E4

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.feed.Feed.random`
- Region: heap
- Site: `src/feed.js:17`
- Reason: not_exercised
- Remedy: Feed field; no growth column ties to it in S1/S2; not a cause of the growth
- Supporting evidence: E4

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
- Remedy: Feed field; no growth column ties to it in S1/S2; not a cause of the growth
- Supporting evidence: E4

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: retention
- Holder: closure/continuation
- Owner: `.dla.20261110-161003-08fa.harness.H1.work.src.board.board`
- Region: heap
- Reason: not_exercised
- Remedy: Same retention chain as F5 (snapshot->previous snapshot); symptom, fix at F5 src/board.js:15
- Supporting evidence: E15

### F5.n1: Hypothesis Retention is only the single latest snapshot (Board.latest) held legitimately, growth is a warm-up/GC-lag artifact (UNCLASSIFIED)

- State: unresolved(not_exercised)
- Evidence classes: discriminator
- Sibling hypothesis of: F5
- Hypothesis: Retention is only the single latest snapshot (Board.latest) held legitimately, growth is a warm-up/GC-lag artifact
- Refuted if: retained count of snapshots/ticks keeps growing across dumps after forced GC
- Reason: not_exercised
- Remedy: malformed sibling (no N-* id); superseded by F5.n2
- Supporting evidence: E20

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: closure/continuation
- Owner: `src.board.Board.publish.previous`
- Region: heap
- Site: `src/board.js:15`
- Reason: not_exercised
- Remedy: Same retention chain as F5 (snapshot->previous snapshot); symptom, fix at F5 src/board.js:15
- Supporting evidence: E22, E23

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: closure/continuation
- Owner: `src.board.Board.publish.previous`
- Region: heap
- Site: `src/board.js:15`
- Reason: not_exercised
- Remedy: Same retention chain as F5 (snapshot->previous snapshot); symptom, fix at F5 src/board.js:15
- Supporting evidence: E36, E37

## Ruled out

- **F5.n2** (UNCLASSIFIED): discriminator E24
  - Ruled out by: plateau+purge (leak) E24
  - Hypothesis: N-WARM: growth is warm-up/lazy allocation and plateaus
  - Refuted if: alloc-test shows no plateau: retained bytes keep rising over extra points
  - Contradicting evidence: E24
  - Originally supported by: E20

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
- E4: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E4: limitation: writers and removers are textual name matches
- E5: partial coverage
- E5: limitation: rxjs-x rules need rxjs to resolve from the project's node_modules for type information
- E7: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E7: limitation: workloads were discovered by reading files; none was run
- E8: limitation: harness written; nothing was run (dla harness run measures it)
- E8: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E9: partial coverage (12 of 12 examined)
- E9: limitation: command-mode heap dumps come from separate processes: dla summarize heap-graph compares dumps of one process only, so use a driver (dla harness gen --driver) for a dump series
- E9: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E9: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds som…
- E9: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E10: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E10: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E10: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E11: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
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
- E16: limitation: series: per-point; one process per point at n = 440, 2200, 3520, 5280 (harness captures; checked: snapshot format)
- E17: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E18: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E19: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E20: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E21: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E22: limitation: series: per-point; x is the iteration count n of each point (440, 2200, 3520, 5280); every point is a separate process and heap snapshot
- E22: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E22: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: series: per-point; x is the iteration count n of each point (440, 2200, 3520, 5280); every point is a separate process and heap snapshot
- E23: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E23: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E24: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E24: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E24: limitation: gate and plateau thresholds are provisional (not calibrated)
- E25: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E25: limitation: weak edges are skipped; string contents are never read into the summary
- E26: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E26: limitation: weak edges are skipped; string contents are never read into the summary
- E27: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E27: limitation: weak edges are skipped; string contents are never read into the summary
- E28: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E28: limitation: weak edges are skipped; string contents are never read into the summary
- E29: limitation: dump-series rule: at least 3 snapshots, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E29: limitation: series: per-point; one process per point at n = 440, 2200, 3520, 5280 (harness captures; checked: snapshot format)
- E30: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E31: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E32: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E33: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E34: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E35: limitation: a sampling heap profile: counts and bytes are estimates, and small allocations may never be sampled
- E35: limitation: each profile covers the objects allocated in its own window that are still alive when it is read; later windows do not hold earlier ones
- E35: limitation: samples in native builtins are charged to the nearest JavaScript frame
- E35: limitation: the profiles are ordered as given
- E35: limitation: the profiles are separate windows, not cumulative: a rise between windows is not retention growth, so no growth column is built from them
- E35: limitation: the profile is sampled: counts and bytes are estimates
- E36: limitation: series: per-point; x is the iteration count n of each point (440, 2200, 3520, 5280); every point is a separate process and heap snapshot
- E36: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E36: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E36: limitation: gate thresholds are provisional (not calibrated)
- E37: limitation: series: per-point; x is the iteration count n of each point (440, 2200, 3520, 5280); every point is a separate process and heap snapshot
- E37: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E37: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E37: limitation: gate thresholds are provisional (not calibrated)

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
| E4 | census-holder | static | dla-census 0.1.0 | repo d42e192d932296cb766bd14caf0ede25d1806902 | census/node.json |
| E5 | static-hit | static | eslint 10.14.0 | repo d42e192d932296cb766bd14caf0ede25d1806902 | sweep/node-eslint.json |
| E6 | static-hit | static | dla 0.1.0 | repo d42e192d932296cb766bd14caf0ede25d1806902 | sweep/node-dla.json |
| E7 | workload | static | dla 0.1.0 | repo d42e192d932296cb766bd14caf0ede25d1806902 | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo d42e192d932296cb766bd14caf0ede25d1806902 | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (15a6ba94c424) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (ddd2c0dc8b6a) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (ddd2c0dc8b6a) | trend/T1.json |
| E12 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-161003-08fa/harness/H1/out/p1-native/capture/9/heap-1.heapsnapshot (7bc080d34e8d) | summaries/heap-1-7bc080d3.json |
| E13 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-161003-08fa/harness/H1/out/p5-native/capture/42/heap-1.heapsnapshot (03548a3832c5) | summaries/heap-1-03548a38.json |
| E14 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-161003-08fa/harness/H1/out/p8-native/capture/66/heap-1.heapsnapshot (d3bb821de9fa) | summaries/heap-1-d3bb821d.json |
| E15 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-161003-08fa/harness/H1/out/p12-native/capture/98/heap-1.heapsnapshot (2d74fb4e2b8f) | summaries/heap-1-2d74fb4e.json |
| E16 | growth-series | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (6917d1ce521f) | series/S2.json |
| E17 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (6917d1ce521f) | series/S2.json |
| E18 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (6917d1ce521f) | series/S2.json |
| E19 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (6917d1ce521f) | series/S2.json |
| E20 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (6917d1ce521f) | series/S2.json |
| E21 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (6917d1ce521f) | series/S2.json |
| E22 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (6917d1ce521f) | trend/T2.json |
| E23 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (6917d1ce521f) | trend/T2.json |
| E24 | discriminator | test | dla 0.1.0 | artifact series/S3.json (8edf9844d434) | alloc/A1.json |
| E25 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-161003-08fa/harness/H1/out/p1-native/capture/9/heap-1.heapsnapshot (7bc080d34e8d) | summaries/heap-1-7bc080d3.json |
| E26 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-161003-08fa/harness/H1/out/p5-native/capture/42/heap-1.heapsnapshot (03548a3832c5) | summaries/heap-1-03548a38.json |
| E27 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-161003-08fa/harness/H1/out/p8-native/capture/66/heap-1.heapsnapshot (d3bb821de9fa) | summaries/heap-1-d3bb821d.json |
| E28 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-161003-08fa/harness/H1/out/p12-native/capture/98/heap-1.heapsnapshot (2d74fb4e2b8f) | summaries/heap-1-2d74fb4e.json |
| E29 | growth-series | dump | dla-v8-snapshot 0.1.0 | artifact series/S4.json (9217011769fc) | series/S4.json |
| E30 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S4.json (9217011769fc) | series/S4.json |
| E31 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S4.json (9217011769fc) | series/S4.json |
| E32 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S4.json (9217011769fc) | series/S4.json |
| E33 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S4.json (9217011769fc) | series/S4.json |
| E34 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S4.json (9217011769fc) | series/S4.json |
| E35 | alloc-stack | live | node-heapprofile 0.1.0 | artifact allocs/A1.json (bf0e512b9a5e) | allocs/A1.json |
| E36 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (9217011769fc) | trend/T4.json |
| E37 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (9217011769fc) | trend/T4.json |
