# DLA report

Run: `20261110-163920-e46c`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 13 enumerated, 13 dispositioned; confirmed 0, strong 0, refuted 3, suspected 10, unresolved 0

Residual: heap 61.96363636363636 bytes/n unattributed of 61.96363636363636 total (threshold 32.676889090909086) ABOVE THRESHOLD, see O2; heap 1.361038961038961 bytes/n unattributed of 1.361038961038961 total (threshold 12.689020519480035); heap 19.047727272727272 bytes/n unattributed of 19.047727272727272 total (threshold 15.44565940909091) ABOVE THRESHOLD, see O2; heap 15.837878787878788 bytes/n unattributed of 15.837878787878788 total (threshold 13.094534999999777) ABOVE THRESHOLD, see O2; heap 24.983636363636364 bytes/n unattributed of 24.983636363636364 total (threshold 14.655635781817992) ABOVE THRESHOLD, see O2; heap 0 bytes/n unattributed of -8.165151515151516 total (threshold 11.925302727272726); heap 14.491818181818182 bytes/n unattributed of 14.491818181818182 total (threshold 25.512446018181755); heap 3.1363636363636367 bytes/n unattributed of 3.1363636363636367 total (threshold 8.83206); heap 16.767045454545453 bytes/n unattributed of 16.767045454545453 total (threshold 13.539676022727273) ABOVE THRESHOLD, see O2; off-heap 14098.897186147187 bytes/n unattributed of 14098.897186147187 total (threshold 4418.325004285704) ABOVE THRESHOLD, see O3; off-heap 11328.387878787878 bytes/n unattributed of 11328.387878787878 total (threshold 2832.0969696969696) ABOVE THRESHOLD, see O3; off-heap 11680.581818181818 bytes/n unattributed of 11680.581818181818 total (threshold 2920.1454545454544) ABOVE THRESHOLD, see O3; off-heap 10815.30181818182 bytes/n unattributed of 10815.30181818182 total (threshold 2703.8254545454547) ABOVE THRESHOLD, see O3; off-heap 11757.769696969695 bytes/n unattributed of 11757.769696969695 total (threshold 2939.4424242424243) ABOVE THRESHOLD, see O3; off-heap 11409.62077922078 bytes/n unattributed of 11409.62077922078 total (threshold 2852.4051948051947) ABOVE THRESHOLD, see O3; off-heap 11556.460606060606 bytes/n unattributed of 11556.460606060606 total (threshold 2889.1151515151514) ABOVE THRESHOLD, see O3; off-heap 11097.98787878788 bytes/n unattributed of 11097.98787878788 total (threshold 2774.4969696969697) ABOVE THRESHOLD, see O3; off-heap 11459.279338842976 bytes/n unattributed of 11459.279338842976 total (threshold 2864.8198347107436) ABOVE THRESHOLD, see O3

Coverage: node 3520 iterations, 25 of 30 entry points exercised

## Findings

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.metrics`
- Region: heap
- Site: `src/gateway.js:13`
- Reason: not_exercised
- Remedy: no per-holder growth column tied to this holder; whole-process heap plateaus after gc (E24); only add a driver probe if os.rss growth is later tied here
- Supporting evidence: E7

### F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.router`
- Region: heap
- Site: `src/gateway.js:14`
- Reason: not_exercised
- Remedy: no per-holder growth column tied to this holder; whole-process heap plateaus after gc (E24); only add a driver probe if os.rss growth is later tied here
- Supporting evidence: E7

### F5: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `src.gateway.Gateway.accept`
- Region: heap
- Site: `src/gateway.js:25`
- Writers: `src/gateway.js:24`, `src/session.js:12`, `src/session.js:54`
- Removers: `load.js:50`, `src/gateway.js:31`, `src/session.js:67`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: no per-holder growth column tied to this holder; whole-process heap plateaus after gc (E24); only add a driver probe if os.rss growth is later tied here
- Supporting evidence: E7

### F7: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.loopback.LoopbackSocket.written`
- Region: heap
- Site: `src/loopback.js:11`
- Writers: `src/loopback.js:17`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: no per-holder growth column tied to this holder; whole-process heap plateaus after gc (E24); only add a driver probe if os.rss growth is later tied here
- Supporting evidence: E7

### F8: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.metrics.Metrics.counters`
- Region: heap
- Site: `src/metrics.js:7`
- Writers: `src/metrics.js:12`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: no per-holder growth column tied to this holder; whole-process heap plateaus after gc (E24); only add a driver probe if os.rss growth is later tied here
- Supporting evidence: E7

### F9: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.router.Router.topics`
- Region: heap
- Site: `src/router.js:9`
- Writers: `src/router.js:16`, `src/router.js:19`
- Removers: `src/router.js:26`, `src/router.js:27`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: no per-holder growth column tied to this holder; whole-process heap plateaus after gc (E24); only add a driver probe if os.rss growth is later tied here
- Supporting evidence: E7

### F10: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `src.router.Router.dispatch`
- Region: heap
- Site: `src/router.js:51`
- Writers: `src/session.js:38`
- Removers: `src/gateway.js:25`, `src/router.js:54`, `src/session.js:33`, `src/session.js:65`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: no per-holder growth column tied to this holder; whole-process heap plateaus after gc (E24); only add a driver probe if os.rss growth is later tied here
- Supporting evidence: E7

### F11: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.decoder`
- Region: heap
- Site: `src/session.js:15`
- Writers: `src/session.js:30`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: no per-holder growth column tied to this holder; whole-process heap plateaus after gc (E24); only add a driver probe if os.rss growth is later tied here
- Supporting evidence: E7

### F12: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.topics`
- Region: heap
- Site: `src/session.js:19`
- Reason: not_exercised
- Remedy: no per-holder growth column tied to this holder; whole-process heap plateaus after gc (E24); only add a driver probe if os.rss growth is later tied here
- Supporting evidence: E7

### F13: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.recent`
- Region: heap
- Site: `src/session.js:20`
- Writers: `src/session.js:44`
- Removers: `src/session.js:45`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: no per-holder growth column tied to this holder; whole-process heap plateaus after gc (E24); only add a driver probe if os.rss growth is later tied here
- Supporting evidence: E7

## Ruled out

- **F3** (UNCLASSIFIED): discriminator E27
  - Ruled out by: plateau+purge (non-leak) E27
  - Contradicting evidence: E27
  - Originally supported by: E7
- **F4** (UNCLASSIFIED): discriminator E29
  - Ruled out by: plateau+purge (non-leak) E29
  - Contradicting evidence: E29
  - Originally supported by: E7
- **F6** (UNCLASSIFIED): discriminator E28
  - Ruled out by: plateau+purge (non-leak) E28
  - Contradicting evidence: E28
  - Originally supported by: E7

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

- E5: partial coverage
- E5: limitation: rxjs-x rules need rxjs to resolve from the project's node_modules for type information
- E7: partial coverage
- E7: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E7: limitation: writers and removers are textual name matches
- E8: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E8: limitation: workloads were discovered by reading files; none was run
- E9: limitation: harness written; nothing was run (dla harness run measures it)
- E9: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E10: partial coverage (9 of 9 examined)
- E10: limitation: command-mode heap dumps come from separate processes: dla summarize heap-graph compares dumps of one process only, so use a driver (dla harness gen --driver) for a dump series
- E10: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E10: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds som…
- E10: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that…
- E10: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E11: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E11: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E11: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E11: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E12: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E12: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E12: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E13: limitation: weak edges are skipped; string contents are never read into the summary
- E14: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E14: limitation: weak edges are skipped; string contents are never read into the summary
- E15: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E15: limitation: weak edges are skipped; string contents are never read into the summary
- E16: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E16: limitation: weak edges are skipped; string contents are never read into the summary
- E17: limitation: dump-series rule: at least 3 snapshots, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E17: limitation: series: per-point; one process per point at n = 440, 1760, 2640, 3960 (harness captures; checked: snapshot format)
- E18: limitation: sites are the first project frame of the stack that created the resource; a resource with no project frame is unattributed
- E18: limitation: active counts come from getActiveResourcesInfo and leave out unref'd timers
- E18: limitation: the files are ordered as given; each is one point
- E19: limitation: sites are the first project frame of the stack that created the resource; a resource with no project frame is unattributed
- E19: limitation: active counts come from getActiveResourcesInfo and leave out unref'd timers
- E19: limitation: the files are ordered as given; each is one point
- E19: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E20: partial coverage (2 of 3 examined)
- E20: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E20: limitation: threshold is provisional (not calibrated)
- E21: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E21: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E21: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E22: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E22: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: x is the snapshot number in capture order; t_ms is the time since the first capture where the inputs record one
- E23: limitation: each point comes from a separate snapshot of one process; a figure absent from a snapshot has no value at that point
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E24: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E24: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E24: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E24: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E24: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E24: limitation: points 1 to 9 come from S1; the extra points from S4
- E24: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E24: limitation: gate and plateau thresholds are provisional (not calibrated)
- E25: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E25: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E25: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E25: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E25: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E25: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E25: limitation: points 1 to 9 come from S1; the extra points from S4
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E26: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E26: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E26: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E26: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E26: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E26: limitation: points 1 to 9 come from S1; the extra points from S4
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E27: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E27: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E27: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E27: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E27: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E27: limitation: points 1 to 9 come from S1; the extra points from S6
- E27: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E27: limitation: gate and plateau thresholds are provisional (not calibrated)
- E28: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E28: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E28: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E28: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E28: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E28: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E28: limitation: points 1 to 9 come from S1; the extra points from S8
- E28: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E28: limitation: gate and plateau thresholds are provisional (not calibrated)
- E29: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E29: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E29: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E29: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E29: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E29: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E29: limitation: points 1 to 9 come from S1; the extra points from S10
- E29: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E29: limitation: gate and plateau thresholds are provisional (not calibrated)
- E30: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E30: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E30: limitation: gate thresholds are provisional (not calibrated)
- E31: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E31: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E31: limitation: gate thresholds are provisional (not calibrated)
- E32: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E32: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E32: limitation: gate thresholds are provisional (not calibrated)
- E33: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E33: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E33: limitation: gate thresholds are provisional (not calibrated)
- E34: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E34: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E34: limitation: gate thresholds are provisional (not calibrated)
- E35: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E35: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E35: limitation: gate thresholds are provisional (not calibrated)
- E36: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E36: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E36: limitation: gate thresholds are provisional (not calibrated)
- E37: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E37: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E37: limitation: gate thresholds are provisional (not calibrated)
- E38: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E38: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E38: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E38: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E38: limitation: gate thresholds are provisional (not calibrated)
- E39: partial coverage (2 of 3 examined)
- E39: limitation: threshold is provisional (not calibrated)
- E40: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E40: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E40: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E40: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E40: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E40: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E40: limitation: points 1 to 9 come from S1; the extra points from S4
- E40: limitation: gate thresholds are provisional (not calibrated)
- E41: partial coverage (2 of 3 examined)
- E41: limitation: threshold is provisional (not calibrated)
- E42: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E42: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E42: limitation: gate thresholds are provisional (not calibrated)
- E43: partial coverage (2 of 3 examined)
- E43: limitation: threshold is provisional (not calibrated)
- E44: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E44: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E44: limitation: gate thresholds are provisional (not calibrated)
- E45: partial coverage (2 of 3 examined)
- E45: limitation: O2 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E45: limitation: threshold is provisional (not calibrated)
- E46: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E46: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E46: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E46: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E46: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E46: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E46: limitation: points 1 to 9 come from S1; the extra points from S6
- E46: limitation: gate thresholds are provisional (not calibrated)
- E47: partial coverage (2 of 3 examined)
- E47: limitation: O2 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E47: limitation: threshold is provisional (not calibrated)
- E48: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E48: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E48: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E48: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E48: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E48: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E48: limitation: points 1 to 9 come from S1; the extra points from S8
- E48: limitation: gate thresholds are provisional (not calibrated)
- E49: partial coverage (2 of 3 examined)
- E49: limitation: threshold is provisional (not calibrated)
- E50: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E50: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E50: limitation: gate thresholds are provisional (not calibrated)
- E51: partial coverage (2 of 3 examined)
- E51: limitation: O2 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E51: limitation: threshold is provisional (not calibrated)
- E52: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E52: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E52: limitation: gate thresholds are provisional (not calibrated)
- E53: partial coverage (2 of 3 examined)
- E53: limitation: O2 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E53: limitation: threshold is provisional (not calibrated)
- E54: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E54: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E54: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E54: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E54: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E54: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E54: limitation: points 1 to 9 come from S1; the extra points from S10
- E54: limitation: gate thresholds are provisional (not calibrated)
- E55: partial coverage (2 of 3 examined)
- E55: limitation: threshold is provisional (not calibrated)

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
| E4 | probe | static | dla-probe 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | probe/probe.json |
| E5 | static-hit | static | eslint 10.14.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | sweep/node-eslint.json |
| E6 | static-hit | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | sweep/node-dla.json |
| E7 | census-holder | static | dla-census 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | census/node.json |
| E8 | workload | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | workload/workloads.json |
| E9 | workload | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | harness/H1/harness.json |
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (985b5d14e968) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (64ca5001cf63) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (64ca5001cf63) | trend/T1.json |
| E13 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-163920-e46c/harness/H1/out/p1-native/capture/9/heap-1.heapsnapshot (4beab2cd1e81) | summaries/heap-1-4beab2cd.json |
| E14 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-163920-e46c/harness/H1/out/p4-native/capture/34/heap-1.heapsnapshot (97691addf6f2) | summaries/heap-1-97691add.json |
| E15 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-163920-e46c/harness/H1/out/p6-native/capture/50/heap-1.heapsnapshot (4bf663cba79e) | summaries/heap-1-4bf663cb.json |
| E16 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-163920-e46c/harness/H1/out/p9-native/capture/74/heap-1.heapsnapshot (600cd725196a) | summaries/heap-1-600cd725.json |
| E17 | growth-series | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (df5ff8f6bebe) | series/S2.json |
| E18 | non-heap | live | node-resources 0.1.0 | artifact nonheap/N1.json (7f5d8e4f730d) | nonheap/N1.json |
| E19 | growth-series | live | node-resources 0.1.0 | artifact series/S3.json (87019708e5d6) | series/S3.json |
| E20 | residual | test | dla 0.1.0 | artifact series/S1.json (64ca5001cf63) | gap/residual-S1-E20.json |
| E21 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (df5ff8f6bebe) | trend/T2.json |
| E22 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (df5ff8f6bebe) | trend/T2.json |
| E23 | unit-count-series | dump | dla 0.1.0 | artifact series/S3.json (87019708e5d6) | trend/T3.json |
| E24 | discriminator | test | dla 0.1.0 | artifact series/S5.json (6d3c34e65435) | alloc/A1.json |
| E25 | growth-series | test | dla 0.1.0 | artifact series/S5.json (6d3c34e65435) | trend/T5.json |
| E26 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (6d3c34e65435) | trend/T5.json |
| E27 | discriminator | test | dla 0.1.0 | artifact series/S7.json (9763eb996423) | alloc/A2.json |
| E28 | discriminator | test | dla 0.1.0 | artifact series/S9.json (0054040a6d57) | alloc/A3.json |
| E29 | discriminator | test | dla 0.1.0 | artifact series/S11.json (6c64c7df8aad) | alloc/A4.json |
| E30 | growth-series | test | dla 0.1.0 | artifact series/S4.json (69c73eb315c1) | trend/T9.json |
| E31 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (69c73eb315c1) | trend/T9.json |
| E32 | growth-series | test | dla 0.1.0 | artifact series/S6.json (521f0e3b1afe) | trend/T10.json |
| E33 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (521f0e3b1afe) | trend/T10.json |
| E34 | growth-series | test | dla 0.1.0 | artifact series/S8.json (07e5475e8c66) | trend/T11.json |
| E35 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (07e5475e8c66) | trend/T11.json |
| E36 | growth-series | test | dla 0.1.0 | artifact series/S10.json (67b1ce9f0d23) | trend/T12.json |
| E37 | unit-count-series | test | dla 0.1.0 | artifact series/S10.json (67b1ce9f0d23) | trend/T12.json |
| E38 | non-heap | test | dla 0.1.0 | artifact series/S12.json (198970042a04) | trend/T13.json |
| E39 | residual | test | dla 0.1.0 | artifact series/S1.json (64ca5001cf63) | gap/residual-S1-E39.json |
| E40 | non-heap | test | dla 0.1.0 | artifact series/S13.json (d5ca22fe9e78) | trend/T14.json |
| E41 | residual | test | dla 0.1.0 | artifact series/S5.json (6d3c34e65435) | gap/residual-S5-E41.json |
| E42 | non-heap | test | dla 0.1.0 | artifact series/S14.json (3fcb054fc35c) | trend/T15.json |
| E43 | residual | test | dla 0.1.0 | artifact series/S4.json (69c73eb315c1) | gap/residual-S4-E43.json |
| E44 | non-heap | test | dla 0.1.0 | artifact series/S15.json (5609b4ffb122) | trend/T16.json |
| E45 | residual | test | dla 0.1.0 | artifact series/S6.json (521f0e3b1afe) | gap/residual-S6-E45.json |
| E46 | non-heap | test | dla 0.1.0 | artifact series/S16.json (839447b52eea) | trend/T17.json |
| E47 | residual | test | dla 0.1.0 | artifact series/S7.json (9763eb996423) | gap/residual-S7-E47.json |
| E48 | non-heap | test | dla 0.1.0 | artifact series/S17.json (85fa03f2dbc9) | trend/T18.json |
| E49 | residual | test | dla 0.1.0 | artifact series/S9.json (0054040a6d57) | gap/residual-S9-E49.json |
| E50 | non-heap | test | dla 0.1.0 | artifact series/S18.json (ba3c437d7f30) | trend/T19.json |
| E51 | residual | test | dla 0.1.0 | artifact series/S8.json (07e5475e8c66) | gap/residual-S8-E51.json |
| E52 | non-heap | test | dla 0.1.0 | artifact series/S19.json (8873ef8648ad) | trend/T20.json |
| E53 | residual | test | dla 0.1.0 | artifact series/S10.json (67b1ce9f0d23) | gap/residual-S10-E53.json |
| E54 | non-heap | test | dla 0.1.0 | artifact series/S20.json (fd877f61dabd) | trend/T21.json |
| E55 | residual | test | dla 0.1.0 | artifact series/S11.json (6c64c7df8aad) | gap/residual-S11-E55.json |
