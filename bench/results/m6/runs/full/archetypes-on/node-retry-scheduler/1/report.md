# DLA report

Run: `20261110-202843-d773`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 11 enumerated, 11 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 9

Residual: heap 4287.572727272727 bytes/n unattributed of 4287.572727272727 total (threshold 1071.8931818181818) ABOVE THRESHOLD, see O2; heap 4265.284848484848 bytes/n unattributed of 4265.284848484848 total (threshold 1066.321212121212) ABOVE THRESHOLD, see O2; heap 4271.016883116883 bytes/n unattributed of 4271.016883116883 total (threshold 1067.7542207792208) ABOVE THRESHOLD, see O2; off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0)

Coverage: node 3520 iterations, 17 of 19 entry points exercised

## Findings

### F5: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (RetryScheduler.latest (src/scheduler.js:15, set in enqueue at :34) heads a backward linked list: each attempt record's previous points to the prior record and nothing ever trims it. Every record keeps its job and payload rows alive after delivery/dead-letter, though pending/inflight are empty. The scheduler is a module singleton (shared), so the chain lives for the process.)
- Growth: `holder:field:.dla.20261110-202843-d773.harness.H1.work.src.scheduler.RetryScheduler.latest` 5011275 bytes/n (steady, gate pass, E16)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `src.scheduler.RetryScheduler.latest`
- Region: heap
- Shape: steady
- Site: `src/scheduler.js:15`
- Writers: `src/scheduler.js:32`, `src/scheduler.js:34`
- Annotation: heuristic name match over the project's sources
- Annotation: latest advanced on every enqueue; record.previous links to prior record; never cleared
- Verified: DISAGREES (site: finding `src/scheduler.js:15` vs verifier `src/scheduler.js:30`; holder: finding static/global vs verifier container-field; label: finding UNCLASSIFIED vs verifier L-COLLECT; bundle 8408d4aa3739); finding mechanism: RetryScheduler.latest (scheduler.js:15) is advanced in enqueue at scheduler.js:32; each record's previous link keeps all history and nothing removes it, so jobs and payload rows outlive delivery.; verifier mechanism: enqueue() sets record.previous = this.latest and latest = record, so every attempt record (with its job) stays reachable from RetryScheduler.latest via the previous chain forever, even after delivered/unlinked from pending. recent() reads only 10, but nothing truncates the chain; grows per attempt.
- Sibling hypotheses: F5.n1
- Supporting evidence: E6, E9, E16
- Explanation (observed): RetryScheduler.latest (scheduler.js:15) is advanced in enqueue at scheduler.js:32; each record's previous link keeps all history and nothing removes it, so jobs and payload rows outlive delivery.
- Explanation structure: adds: src/scheduler.js:32; removal: never; evidence: E16

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `src/scheduler.js:32`
- Reason: not_exercised
- Remedy: Detector hit node-chain-head-growth at scheduler.js:32 is the same site as F5; duplicate lead of F5.
- Supporting evidence: E5

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.pending.PendingList.head`
- Region: heap
- Site: `src/pending.js:7`
- Writers: `src/pending.js:15`, `src/pending.js:24`, `src/pending.js:19`
- Removers: `src/pending.js:28`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: PendingList.head: emptied by unlink; pending length returns to 0. Not individually isolated.
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.pending.PendingList.tail`
- Region: heap
- Site: `src/pending.js:8`
- Writers: `src/pending.js:16`, `src/pending.js:25`, `src/pending.js:19`
- Removers: `src/pending.js:28`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: PendingList.tail: reset by unlink to prev; pending 0. Not individually isolated.
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.scheduler.RetryScheduler.pending`
- Region: heap
- Site: `src/scheduler.js:14`
- Writers: `src/scheduler.js:33`, `src/scheduler.js:14`
- Removers: `src/scheduler.js:53`, `src/scheduler.js:57`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Source read: PendingList is emptied by unlink in tick (pending=0 after batch); growth column is the shared.latest chain (F5). Not individually isolated.
- Supporting evidence: E6

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.scheduler.RetryScheduler.deadLetters`
- Region: heap
- Site: `src/scheduler.js:16`
- Writers: `src/scheduler.js:72`
- Removers: `src/scheduler.js:73`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Source read: deadLetters capped at 100 by shift (scheduler.js:73); bounded, growth in the column is F5's chain.
- Supporting evidence: E6

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.scheduler.RetryScheduler.inflight`
- Region: heap
- Site: `src/scheduler.js:17`
- Writers: `src/scheduler.js:22`, `src/scheduler.js:24`
- Removers: `src/scheduler.js:66`, `src/scheduler.js:71`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Source read: inflight deleted on delivery/dead (scheduler.js:66,71); stats show 0 after batch. Growth is F5's chain.
- Supporting evidence: E6

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `src.scheduler.shared`
- Region: heap
- Site: `src/scheduler.js:90`
- Writers: `src/scheduler.js:93`, `src/scheduler.js:90`
- Annotation: heuristic name match over the project's sources
- Annotation: module-level singleton keeps scheduler (and F5 chain) alive for process lifetime
- Reason: not_exercised
- Remedy: Module singleton 'shared' is only the root that keeps F5's chain reachable; not an independent leak. Fix lies in F5's chain, not the singleton.
- Supporting evidence: E6

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Site: `src/jobs.js:9`
- Hypothesis: payload rows arrays retained via record.job through the latest chain (consequence of F5, not an independent holder)
- Refuted if: rows are freed after the chain is cut
- Reason: not_exercised
- Remedy: Rows arrays retained through record.job by F5's chain; consequence, not independent holder.
- Supporting evidence: E21

### F10: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Site: `src/jobs.js:9`
- Hypothesis: count of retained rows arrays; same consequence as F9
- Refuted if: count flat after chain cut
- Reason: not_exercised
- Remedy: Row-count of F9; consequence of F5's chain.
- Supporting evidence: E22

## Ruled out

- **F5.n1** (UNCLASSIFIED): discriminator E18
  - Ruled out by: dump-plateau (leak) E18
  - Hypothesis: N-WARM: growth is a warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles / dump-plateau favors leak
  - Contradicting evidence: E18
  - Originally supported by: E16

## Blind spots and unknowns

### Coverage gaps

- node/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- node/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)

### Runtime blind spots

None recorded.

### Entry points not exercisable

- node: `src/pending.js::PendingList.[Symbol.iterator]`: no_harness

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
- E20: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E20: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E20: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E20: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E20: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E20: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E20: limitation: points 1 to 9 come from S1; the extra points from S3
- E20: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E20: limitation: gate and plateau thresholds are provisional (not calibrated)
- E21: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E21: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E21: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E22: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E22: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E23: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E24: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: partial coverage (2 of 3 examined)
- E25: limitation: F5: growth evidence is from S2, not S1; not subtracted
- E25: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E25: limitation: threshold is provisional (not calibrated)
- E26: partial coverage (2 of 3 examined)
- E26: limitation: F5: growth evidence is from S2, not S3; not subtracted
- E26: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E26: limitation: threshold is provisional (not calibrated)
- E27: partial coverage (2 of 3 examined)
- E27: limitation: F5: growth evidence is from S2, not S4; not subtracted
- E27: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E27: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | — |
| E2 | run-config | static | dla 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | probe/probe.json |
| E4 | static-hit | static | eslint 10.14.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | sweep/node-eslint.json |
| E5 | static-hit | static | dla 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | sweep/node-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | census/node.json |
| E7 | workload | static | dla 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (7f6a376ff04d) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (fbf188460380) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (fbf188460380) | trend/T1.json |
| E12 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-202843-d773/harness/H1/out/p1-native/capture/9/heap-1.heapsnapshot (2f24c7d050e2) | summaries/heap-1-2f24c7d0.json |
| E13 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-202843-d773/harness/H1/out/p4-native/capture/34/heap-1.heapsnapshot (8e49e8da1756) | summaries/heap-1-8e49e8da.json |
| E14 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-202843-d773/harness/H1/out/p6-native/capture/50/heap-1.heapsnapshot (63b7d1482522) | summaries/heap-1-63b7d148.json |
| E15 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-202843-d773/harness/H1/out/p9-native/capture/74/heap-1.heapsnapshot (dad5678454f9) | summaries/heap-1-dad56784.json |
| E16 | growth-series | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (12cc4a034b21) | series/S2.json |
| E17 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (12cc4a034b21) | series/S2.json |
| E18 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (12cc4a034b21) | series/S2.json |
| E19 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (12cc4a034b21) | series/S2.json |
| E20 | discriminator | test | dla 0.1.0 | artifact series/S4.json (5e10051c9675) | alloc/A1.json |
| E21 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (12cc4a034b21) | trend/T3.json |
| E22 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (12cc4a034b21) | trend/T3.json |
| E23 | growth-series | test | dla 0.1.0 | artifact series/S3.json (0838fa3991ec) | trend/T4.json |
| E24 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (0838fa3991ec) | trend/T4.json |
| E25 | residual | test | dla 0.1.0 | artifact series/S1.json (fbf188460380) | gap/residual-S1-E25.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S3.json (0838fa3991ec) | gap/residual-S3-E26.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S4.json (5e10051c9675) | gap/residual-S4-E27.json |
