# DLA report

Run: `20261110-183531-f614`

Status: INCOMPLETE (4 open)

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 13 enumerated, 13 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 13

Residual: heap 70.29030303030302 bytes/n unattributed of 70.29030303030302 total (threshold 68.04498599999947) ABOVE THRESHOLD, see O1; heap 1.4155844155844155 bytes/n unattributed of 1.4155844155844155 total (threshold 13.222096363637036); heap 14.53181818181818 bytes/n unattributed of 14.53181818181818 total (threshold 21.68976409090909); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0)

Coverage: node 3520 iterations, 25 of 30 entry points exercised

## Open items

- **T4:type:system / WeakArrayList**: column type:system / WeakArrayList of T4 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E24 # column type:system / WeakArrayList`
- **T4:type:system / WeakArrayList.count**: column type:system / WeakArrayList.count of T4 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E25 # column type:system / WeakArrayList.count`
- **T4:type:(code)**: column type:(code) of T4 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E24 # column type:(code)`
- **T4:type:(code).count**: column type:(code).count of T4 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E25 # column type:(code).count`

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.metrics`
- Region: heap
- Site: `src/gateway.js:13`
- Reason: not_exercised
- Remedy: Holder showed no growth in the in-process loopback harness (heap snapshots flat, 0 active timers). Re-test with real net sockets under production traffic and 3+ snapshots in one process (driver mode).
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.router`
- Region: heap
- Site: `src/gateway.js:14`
- Reason: not_exercised
- Remedy: Holder showed no growth in the in-process loopback harness (heap snapshots flat, 0 active timers). Re-test with real net sockets under production traffic and 3+ snapshots in one process (driver mode).
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.sessions`
- Region: heap
- Site: `src/gateway.js:15`
- Writers: `src/gateway.js:22`
- Removers: `src/gateway.js:32`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Holder showed no growth in the in-process loopback harness (heap snapshots flat, 0 active timers). Re-test with real net sockets under production traffic and 3+ snapshots in one process (driver mode).
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `src.gateway.Gateway.accept`
- Region: heap
- Site: `src/gateway.js:24`
- Writers: `src/gateway.js:25`, `src/session.js:12`, `src/session.js:54`
- Removers: `load.js:50`, `src/gateway.js:31`, `src/session.js:67`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Holder showed no growth in the in-process loopback harness (heap snapshots flat, 0 active timers). Re-test with real net sockets under production traffic and 3+ snapshots in one process (driver mode).
- Supporting evidence: E6

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `src.gateway.Gateway.accept`
- Region: heap
- Site: `src/gateway.js:25`
- Writers: `src/gateway.js:24`, `src/session.js:12`, `src/session.js:54`
- Removers: `load.js:50`, `src/gateway.js:31`, `src/session.js:67`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Holder showed no growth in the in-process loopback harness (heap snapshots flat, 0 active timers). Re-test with real net sockets under production traffic and 3+ snapshots in one process (driver mode).
- Supporting evidence: E6

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `src.heartbeat.startHeartbeat`
- Region: non-heap
- Site: `src/heartbeat.js:15`
- Reason: not_exercised
- Remedy: Holder showed no growth in the in-process loopback harness (heap snapshots flat, 0 active timers). Re-test with real net sockets under production traffic and 3+ snapshots in one process (driver mode).
- Supporting evidence: E6

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.loopback.LoopbackSocket.written`
- Region: heap
- Site: `src/loopback.js:11`
- Writers: `src/loopback.js:17`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Holder showed no growth in the in-process loopback harness (heap snapshots flat, 0 active timers). Re-test with real net sockets under production traffic and 3+ snapshots in one process (driver mode).
- Supporting evidence: E6

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.metrics.Metrics.counters`
- Region: heap
- Site: `src/metrics.js:7`
- Writers: `src/metrics.js:12`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Holder showed no growth in the in-process loopback harness (heap snapshots flat, 0 active timers). Re-test with real net sockets under production traffic and 3+ snapshots in one process (driver mode).
- Supporting evidence: E6

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.router.Router.topics`
- Region: heap
- Site: `src/router.js:9`
- Writers: `src/router.js:16`, `src/router.js:19`
- Removers: `src/router.js:26`, `src/router.js:27`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Holder showed no growth in the in-process loopback harness (heap snapshots flat, 0 active timers). Re-test with real net sockets under production traffic and 3+ snapshots in one process (driver mode).
- Supporting evidence: E6

### F10: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `src.router.Router.dispatch`
- Region: heap
- Site: `src/router.js:51`
- Writers: `src/session.js:38`
- Removers: `src/gateway.js:25`, `src/router.js:54`, `src/session.js:33`, `src/session.js:65`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Holder showed no growth in the in-process loopback harness (heap snapshots flat, 0 active timers). Re-test with real net sockets under production traffic and 3+ snapshots in one process (driver mode).
- Supporting evidence: E6

### F11: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.decoder`
- Region: heap
- Site: `src/session.js:15`
- Writers: `src/session.js:30`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Holder showed no growth in the in-process loopback harness (heap snapshots flat, 0 active timers). Re-test with real net sockets under production traffic and 3+ snapshots in one process (driver mode).
- Supporting evidence: E6

### F12: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.topics`
- Region: heap
- Site: `src/session.js:19`
- Reason: not_exercised
- Remedy: Holder showed no growth in the in-process loopback harness (heap snapshots flat, 0 active timers). Re-test with real net sockets under production traffic and 3+ snapshots in one process (driver mode).
- Supporting evidence: E6

### F13: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.recent`
- Region: heap
- Site: `src/session.js:20`
- Writers: `src/session.js:44`
- Removers: `src/session.js:45`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Holder showed no growth in the in-process loopback harness (heap snapshots flat, 0 active timers). Re-test with real net sockets under production traffic and 3+ snapshots in one process (driver mode).
- Supporting evidence: E6

## Ruled out

None.

## Blind spots and unknowns

### Coverage gaps

- node/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- node/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)

### Runtime blind spots

None recorded.

### Entry points not exercisable

- node: `src/metrics.js::Metrics.observe`: no_harness
- node: `src/session.js::Session.describe`: no_harness

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
- E12: limitation: sizes are shallow self sizes of V8 nodes; strings are one row, never named after their text
- E12: limitation: the snapshots are compared in the order given; snapshots of separate processes compare by class only
- E12: limitation: a class histogram ties to no finding: only a holder of a heap-graph series does
- E13: limitation: sizes are shallow self sizes of V8 nodes; strings are one row, never named after their text
- E13: limitation: the snapshots are compared in the order given; snapshots of separate processes compare by class only
- E13: limitation: a class histogram ties to no finding: only a holder of a heap-graph series does
- E13: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E14: partial coverage (2 of 3 examined)
- E14: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E14: limitation: threshold is provisional (not calibrated)
- E15: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E15: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E15: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E15: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E15: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E15: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E15: limitation: points 1 to 9 come from S1; the extra points from S3
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E16: limitation: weak edges are skipped; string contents are never read into the summary
- E17: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E17: limitation: weak edges are skipped; string contents are never read into the summary
- E18: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E18: limitation: weak edges are skipped; string contents are never read into the summary
- E19: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E19: limitation: weak edges are skipped; string contents are never read into the summary
- E20: limitation: dump-series rule: at least 3 snapshots, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E20: limitation: series: per-point; one process per point at n = 440, 1760, 2640, 3960 (harness captures; checked: snapshot format)
- E21: limitation: sites are the first project frame of the stack that created the resource; a resource with no project frame is unattributed
- E21: limitation: active counts come from getActiveResourcesInfo and leave out unref'd timers
- E21: limitation: the files are ordered as given; each is one point
- E22: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E22: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E22: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E22: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E22: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E22: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E22: limitation: points 1 to 9 come from S1; the extra points from S3
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E23: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E23: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E23: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E23: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E23: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E23: limitation: points 1 to 9 come from S1; the extra points from S3
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E24: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E25: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E26: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E27: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E28: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E28: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E29: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E29: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E29: limitation: gate thresholds are provisional (not calibrated)
- E30: partial coverage (2 of 3 examined)
- E30: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E30: limitation: threshold is provisional (not calibrated)
- E31: partial coverage (2 of 3 examined)
- E31: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E31: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | — |
| E2 | run-config | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | probe/probe.json |
| E4 | static-hit | static | eslint 10.14.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | sweep/node-eslint.json |
| E5 | static-hit | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | sweep/node-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | census/node.json |
| E7 | workload | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (bbc288e96ca0) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (a147394aa479) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (a147394aa479) | trend/T1.json |
| E12 | histogram-diff | dump | dla-v8-snapshot 0.1.0 | artifact histograms/H1.json (d2d0ac5ad64e) | histograms/H1.json |
| E13 | growth-series | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (00782ab98591) | series/S2.json |
| E14 | residual | test | dla 0.1.0 | artifact series/S1.json (a147394aa479) | gap/residual-S1-E14.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S4.json (5004c7de2357) | alloc/A1.json |
| E16 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-183531-f614/harness/H1/out/p1-native/capture/9/heap-1.heapsnapshot (a011333fb16a) | summaries/heap-1-a011333f.json |
| E17 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-183531-f614/harness/H1/out/p4-native/capture/34/heap-1.heapsnapshot (9f3c40a7eaff) | summaries/heap-1-9f3c40a7.json |
| E18 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-183531-f614/harness/H1/out/p6-native/capture/50/heap-1.heapsnapshot (ae0872aab111) | summaries/heap-1-ae0872aa.json |
| E19 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-183531-f614/harness/H1/out/p9-native/capture/74/heap-1.heapsnapshot (4cbbc0b5f3be) | summaries/heap-1-4cbbc0b5.json |
| E20 | growth-series | dump | dla-v8-snapshot 0.1.0 | artifact series/S5.json (70ef1801d0e1) | series/S5.json |
| E21 | non-heap | live | node-resources 0.1.0 | artifact nonheap/N1.json (2c31cc94cf11) | nonheap/N1.json |
| E22 | growth-series | test | dla 0.1.0 | artifact series/S4.json (5004c7de2357) | trend/T3.json |
| E23 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (5004c7de2357) | trend/T3.json |
| E24 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (00782ab98591) | trend/T4.json |
| E25 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (00782ab98591) | trend/T4.json |
| E26 | growth-series | test | dla 0.1.0 | artifact series/S3.json (7f092bc775e2) | trend/T5.json |
| E27 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (7f092bc775e2) | trend/T5.json |
| E28 | growth-series | dump | dla 0.1.0 | artifact series/S5.json (70ef1801d0e1) | trend/T6.json |
| E29 | unit-count-series | dump | dla 0.1.0 | artifact series/S5.json (70ef1801d0e1) | trend/T6.json |
| E30 | residual | test | dla 0.1.0 | artifact series/S4.json (5004c7de2357) | gap/residual-S4-E30.json |
| E31 | residual | test | dla 0.1.0 | artifact series/S3.json (7f092bc775e2) | gap/residual-S3-E31.json |
