# DLA report

Run: `20261110-190909-08e5`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 14 enumerated, 14 dispositioned; confirmed 0, strong 0, refuted 0, suspected 13, unresolved 1

Residual: heap 59.775757575757574 bytes/n unattributed of 59.775757575757574 total (threshold 47.16978545454478) ABOVE THRESHOLD, see O2; heap 13.36969696969697 bytes/n unattributed of 13.36969696969697 total (threshold 9.873730909090462) ABOVE THRESHOLD, see O2; heap 21.700000000000003 bytes/n unattributed of 21.700000000000003 total (threshold 13.493007818181818) ABOVE THRESHOLD, see O2; off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0)

Coverage: node 3520 iterations, 25 of 30 entry points exercised

## Findings

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: growth, static
- Holder: container-field
- Owner: `src.gateway.Gateway.metrics`
- Region: heap
- Site: `src/gateway.js:13`
- Reason: not_exercised
- Remedy: No growth tied to this holder in harness H1 (heap snapshots flat, 0 live Timeouts, process slope ~11KB/point). Run a real-socket soak with idle/bye/protocol closes and heap snapshots over many waves to exercise it.
- Supporting evidence: E6, E16

### F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: growth, static
- Holder: container-field
- Owner: `src.gateway.Gateway.router`
- Region: heap
- Site: `src/gateway.js:14`
- Reason: not_exercised
- Remedy: No growth tied to this holder in harness H1 (heap snapshots flat, 0 live Timeouts, process slope ~11KB/point). Run a real-socket soak with idle/bye/protocol closes and heap snapshots over many waves to exercise it.
- Supporting evidence: E6, E16

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: growth, static
- Holder: container-field
- Owner: `src.gateway.Gateway.sessions`
- Region: heap
- Site: `src/gateway.js:15`
- Writers: `src/gateway.js:22`
- Removers: `src/gateway.js:32`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: No growth tied to this holder in harness H1 (heap snapshots flat, 0 live Timeouts, process slope ~11KB/point). Run a real-socket soak with idle/bye/protocol closes and heap snapshots over many waves to exercise it.
- Supporting evidence: E6, E16

### F4: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: growth, static
- Holder: registration-table
- Owner: `src.gateway.Gateway.accept`
- Region: heap
- Site: `src/gateway.js:24`
- Writers: `src/gateway.js:25`, `src/session.js:12`, `src/session.js:54`
- Removers: `load.js:50`, `src/gateway.js:31`, `src/session.js:67`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: No growth tied to this holder in harness H1 (heap snapshots flat, 0 live Timeouts, process slope ~11KB/point). Run a real-socket soak with idle/bye/protocol closes and heap snapshots over many waves to exercise it.
- Supporting evidence: E6, E16

### F5: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: growth, static
- Holder: registration-table
- Owner: `src.gateway.Gateway.accept`
- Region: heap
- Site: `src/gateway.js:25`
- Writers: `src/gateway.js:24`, `src/session.js:12`, `src/session.js:54`
- Removers: `load.js:50`, `src/gateway.js:31`, `src/session.js:67`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: No growth tied to this holder in harness H1 (heap snapshots flat, 0 live Timeouts, process slope ~11KB/point). Run a real-socket soak with idle/bye/protocol closes and heap snapshots over many waves to exercise it.
- Supporting evidence: E6, E16

### F6: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: growth, static
- Holder: blocked-stack
- Owner: `src.heartbeat.startHeartbeat`
- Region: non-heap
- Site: `src/heartbeat.js:15`
- Reason: not_exercised
- Remedy: No growth tied to this holder in harness H1 (heap snapshots flat, 0 live Timeouts, process slope ~11KB/point). Run a real-socket soak with idle/bye/protocol closes and heap snapshots over many waves to exercise it.
- Supporting evidence: E6, E16

### F7: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: growth, static
- Holder: container-field
- Owner: `src.loopback.LoopbackSocket.written`
- Region: heap
- Site: `src/loopback.js:11`
- Writers: `src/loopback.js:17`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: No growth tied to this holder in harness H1 (heap snapshots flat, 0 live Timeouts, process slope ~11KB/point). Run a real-socket soak with idle/bye/protocol closes and heap snapshots over many waves to exercise it.
- Supporting evidence: E6, E16

### F8: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: growth, static
- Holder: container-field
- Owner: `src.metrics.Metrics.counters`
- Region: heap
- Site: `src/metrics.js:7`
- Writers: `src/metrics.js:12`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: No growth tied to this holder in harness H1 (heap snapshots flat, 0 live Timeouts, process slope ~11KB/point). Run a real-socket soak with idle/bye/protocol closes and heap snapshots over many waves to exercise it.
- Supporting evidence: E6, E16

### F9: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: growth, static
- Holder: container-field
- Owner: `src.router.Router.topics`
- Region: heap
- Site: `src/router.js:9`
- Writers: `src/router.js:16`, `src/router.js:19`
- Removers: `src/router.js:26`, `src/router.js:27`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: No growth tied to this holder in harness H1 (heap snapshots flat, 0 live Timeouts, process slope ~11KB/point). Run a real-socket soak with idle/bye/protocol closes and heap snapshots over many waves to exercise it.
- Supporting evidence: E6, E16

### F10: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: growth, static
- Holder: registration-table
- Owner: `src.router.Router.dispatch`
- Region: heap
- Site: `src/router.js:51`
- Writers: `src/session.js:38`
- Removers: `src/gateway.js:25`, `src/router.js:54`, `src/session.js:33`, `src/session.js:65`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: No growth tied to this holder in harness H1 (heap snapshots flat, 0 live Timeouts, process slope ~11KB/point). Run a real-socket soak with idle/bye/protocol closes and heap snapshots over many waves to exercise it.
- Supporting evidence: E6, E16

### F11: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: growth, static
- Holder: container-field
- Owner: `src.session.Session.decoder`
- Region: heap
- Site: `src/session.js:15`
- Writers: `src/session.js:30`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: No growth tied to this holder in harness H1 (heap snapshots flat, 0 live Timeouts, process slope ~11KB/point). Run a real-socket soak with idle/bye/protocol closes and heap snapshots over many waves to exercise it.
- Supporting evidence: E6, E16

### F12: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: growth, static
- Holder: container-field
- Owner: `src.session.Session.topics`
- Region: heap
- Site: `src/session.js:19`
- Reason: not_exercised
- Remedy: No growth tied to this holder in harness H1 (heap snapshots flat, 0 live Timeouts, process slope ~11KB/point). Run a real-socket soak with idle/bye/protocol closes and heap snapshots over many waves to exercise it.
- Supporting evidence: E6, E16

### F13: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: growth, static
- Holder: container-field
- Owner: `src.session.Session.recent`
- Region: heap
- Site: `src/session.js:20`
- Writers: `src/session.js:44`
- Removers: `src/session.js:45`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: No growth tied to this holder in harness H1 (heap snapshots flat, 0 live Timeouts, process slope ~11KB/point). Run a real-socket soak with idle/bye/protocol closes and heap snapshots over many waves to exercise it.
- Supporting evidence: E6, E16

### F14: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: V8-internal type columns ((code), WeakArrayList, (string)) grow slightly: JIT/engine warm-up, no project holder; verify with a longer soak that they plateau
- Supporting evidence: E29, E30

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
- E17: limitation: sites are the first project frame of the stack that created the resource; a resource with no project frame is unattributed
- E17: limitation: active counts come from getActiveResourcesInfo and leave out unref'd timers
- E17: limitation: the files are ordered as given; each is one point
- E18: limitation: sites are the first project frame of the stack that created the resource; a resource with no project frame is unattributed
- E18: limitation: active counts come from getActiveResourcesInfo and leave out unref'd timers
- E18: limitation: the files are ordered as given; each is one point
- E18: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E19: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E19: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E19: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E19: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E19: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E19: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E19: limitation: points 1 to 9 come from S1; the extra points from S4
- E19: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E19: limitation: gate and plateau thresholds are provisional (not calibrated)
- E20: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E20: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E20: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E21: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E21: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: x is the snapshot number in capture order; t_ms is the time since the first capture where the inputs record one
- E22: limitation: each point comes from a separate snapshot of one process; a figure absent from a snapshot has no value at that point
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E23: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E24: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: sizes are shallow self sizes of V8 nodes; strings are one row, never named after their text
- E25: limitation: the snapshots are compared in the order given; snapshots of separate processes compare by class only
- E25: limitation: a class histogram ties to no finding: only a holder of a heap-graph series does
- E26: limitation: sizes are shallow self sizes of V8 nodes; strings are one row, never named after their text
- E26: limitation: the snapshots are compared in the order given; snapshots of separate processes compare by class only
- E26: limitation: a class histogram ties to no finding: only a holder of a heap-graph series does
- E26: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E27: limitation: a sampling heap profile: counts and bytes are estimates, and small allocations may never be sampled
- E27: limitation: each profile covers the objects allocated in its own window that are still alive when it is read; later windows do not hold earlier ones
- E27: limitation: samples in native builtins are charged to the nearest JavaScript frame
- E27: limitation: the profiles are ordered as given
- E27: limitation: the profiles are separate windows, not cumulative: a rise between windows is not retention growth, so no growth column is built from them
- E27: limitation: the profile is sampled: counts and bytes are estimates
- E28: partial coverage (2 of 3 examined)
- E28: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E28: limitation: threshold is provisional (not calibrated)
- E29: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E29: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E29: limitation: gate thresholds are provisional (not calibrated)
- E30: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E30: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E30: limitation: gate thresholds are provisional (not calibrated)
- E31: partial coverage (2 of 3 examined)
- E31: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E31: limitation: threshold is provisional (not calibrated)
- E32: partial coverage (2 of 3 examined)
- E32: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E32: limitation: threshold is provisional (not calibrated)
- E33: partial coverage (2 of 3 examined)
- E33: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E33: limitation: threshold is provisional (not calibrated)

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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (b1224cd2419a) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (f0312eabf5da) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (f0312eabf5da) | trend/T1.json |
| E12 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-190909-08e5/harness/H1/out/p1-native/capture/9/heap-1.heapsnapshot (fcab2f266cfa) | summaries/heap-1-fcab2f26.json |
| E13 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-190909-08e5/harness/H1/out/p4-native/capture/34/heap-1.heapsnapshot (1416bb717173) | summaries/heap-1-1416bb71.json |
| E14 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-190909-08e5/harness/H1/out/p6-native/capture/50/heap-1.heapsnapshot (c41d6c50fc28) | summaries/heap-1-c41d6c50.json |
| E15 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-190909-08e5/harness/H1/out/p9-native/capture/74/heap-1.heapsnapshot (7334173a96ba) | summaries/heap-1-7334173a.json |
| E16 | growth-series | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (c4bdb6311f7f) | series/S2.json |
| E17 | non-heap | live | node-resources 0.1.0 | artifact nonheap/N1.json (cca02c82b4d4) | nonheap/N1.json |
| E18 | growth-series | live | node-resources 0.1.0 | artifact series/S3.json (f6859d5d3d7f) | series/S3.json |
| E19 | discriminator | test | dla 0.1.0 | artifact series/S5.json (7d4f128fa91c) | alloc/A1.json |
| E20 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (c4bdb6311f7f) | trend/T3.json |
| E21 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (c4bdb6311f7f) | trend/T3.json |
| E22 | unit-count-series | dump | dla 0.1.0 | artifact series/S3.json (f6859d5d3d7f) | trend/T4.json |
| E23 | growth-series | test | dla 0.1.0 | artifact series/S4.json (28f507b143b8) | trend/T5.json |
| E24 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (28f507b143b8) | trend/T5.json |
| E25 | histogram-diff | dump | dla-v8-snapshot 0.1.0 | artifact histograms/H1.json (dcd27223eb7e) | histograms/H1.json |
| E26 | growth-series | dump | dla-v8-snapshot 0.1.0 | artifact series/S6.json (578b39c8f671) | series/S6.json |
| E27 | alloc-stack | live | node-heapprofile 0.1.0 | artifact allocs/A1.json (639ef1c0142e) | allocs/A1.json |
| E28 | residual | test | dla 0.1.0 | artifact series/S4.json (28f507b143b8) | gap/residual-S4-E28.json |
| E29 | growth-series | dump | dla 0.1.0 | artifact series/S6.json (578b39c8f671) | trend/T6.json |
| E30 | unit-count-series | dump | dla 0.1.0 | artifact series/S6.json (578b39c8f671) | trend/T6.json |
| E31 | residual | test | dla 0.1.0 | artifact series/S1.json (f0312eabf5da) | gap/residual-S1-E31.json |
| E32 | residual | test | dla 0.1.0 | artifact series/S5.json (7d4f128fa91c) | gap/residual-S5-E32.json |
| E33 | residual | test | dla 0.1.0 | artifact series/S4.json (28f507b143b8) | gap/residual-S4-E33.json |
