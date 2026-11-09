# DLA report

Run: `20261109-202853-7512`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 13 enumerated, 13 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 13

Residual: heap 58.96666666666667 bytes/n unattributed of 58.96666666666667 total (threshold 53.12714181818136) ABOVE THRESHOLD, see O1; off-heap 0 bytes/n unattributed of 0 total (threshold 0)

Coverage: node 3520 iterations, 25 of 30 entry points exercised

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: discriminator, static
- Holder: container-field
- Owner: `src.gateway.Gateway.metrics`
- Region: heap
- Site: `src/gateway.js:13`
- Reason: not_exercised
- Remedy: Harness (in-process LoopbackSocket, 4k sessions) showed process rss/external growth but no heap holder or timer column grew; reproduce with real net sockets (destroy emits close) over several waves and take 3+ heap snapshots in one process (driver mode / dla attach node-inspector heap-snapshot) to attribute
- Supporting evidence: E6, E19

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: discriminator, static
- Holder: container-field
- Owner: `src.gateway.Gateway.router`
- Region: heap
- Site: `src/gateway.js:14`
- Reason: not_exercised
- Remedy: Harness (in-process LoopbackSocket, 4k sessions) showed process rss/external growth but no heap holder or timer column grew; reproduce with real net sockets (destroy emits close) over several waves and take 3+ heap snapshots in one process (driver mode / dla attach node-inspector heap-snapshot) to attribute
- Supporting evidence: E6, E19

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: discriminator, static
- Holder: container-field
- Owner: `src.gateway.Gateway.sessions`
- Region: heap
- Site: `src/gateway.js:15`
- Writers: `src/gateway.js:22`
- Removers: `src/gateway.js:32`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Harness (in-process LoopbackSocket, 4k sessions) showed process rss/external growth but no heap holder or timer column grew; reproduce with real net sockets (destroy emits close) over several waves and take 3+ heap snapshots in one process (driver mode / dla attach node-inspector heap-snapshot) to attribute
- Supporting evidence: E6, E19

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: discriminator, static
- Holder: registration-table
- Owner: `src.gateway.Gateway.accept`
- Region: heap
- Site: `src/gateway.js:24`
- Writers: `src/gateway.js:25`, `src/session.js:12`, `src/session.js:54`
- Removers: `load.js:50`, `src/gateway.js:31`, `src/session.js:67`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Harness (in-process LoopbackSocket, 4k sessions) showed process rss/external growth but no heap holder or timer column grew; reproduce with real net sockets (destroy emits close) over several waves and take 3+ heap snapshots in one process (driver mode / dla attach node-inspector heap-snapshot) to attribute
- Supporting evidence: E6, E19

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: discriminator, static
- Holder: registration-table
- Owner: `src.gateway.Gateway.accept`
- Region: heap
- Site: `src/gateway.js:25`
- Writers: `src/gateway.js:24`, `src/session.js:12`, `src/session.js:54`
- Removers: `load.js:50`, `src/gateway.js:31`, `src/session.js:67`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Harness (in-process LoopbackSocket, 4k sessions) showed process rss/external growth but no heap holder or timer column grew; reproduce with real net sockets (destroy emits close) over several waves and take 3+ heap snapshots in one process (driver mode / dla attach node-inspector heap-snapshot) to attribute
- Supporting evidence: E6, E19

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: discriminator, static
- Holder: blocked-stack
- Owner: `src.heartbeat.startHeartbeat`
- Region: non-heap
- Site: `src/heartbeat.js:15`
- Reason: not_exercised
- Remedy: Harness (in-process LoopbackSocket, 4k sessions) showed process rss/external growth but no heap holder or timer column grew; reproduce with real net sockets (destroy emits close) over several waves and take 3+ heap snapshots in one process (driver mode / dla attach node-inspector heap-snapshot) to attribute
- Supporting evidence: E6, E19

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: discriminator, static
- Holder: container-field
- Owner: `src.loopback.LoopbackSocket.written`
- Region: heap
- Site: `src/loopback.js:11`
- Writers: `src/loopback.js:17`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Harness (in-process LoopbackSocket, 4k sessions) showed process rss/external growth but no heap holder or timer column grew; reproduce with real net sockets (destroy emits close) over several waves and take 3+ heap snapshots in one process (driver mode / dla attach node-inspector heap-snapshot) to attribute
- Supporting evidence: E6, E19

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: discriminator, static
- Holder: container-field
- Owner: `src.metrics.Metrics.counters`
- Region: heap
- Site: `src/metrics.js:7`
- Writers: `src/metrics.js:12`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Harness (in-process LoopbackSocket, 4k sessions) showed process rss/external growth but no heap holder or timer column grew; reproduce with real net sockets (destroy emits close) over several waves and take 3+ heap snapshots in one process (driver mode / dla attach node-inspector heap-snapshot) to attribute
- Supporting evidence: E6, E19

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: discriminator, static
- Holder: container-field
- Owner: `src.router.Router.topics`
- Region: heap
- Site: `src/router.js:9`
- Writers: `src/router.js:16`, `src/router.js:19`
- Removers: `src/router.js:26`, `src/router.js:27`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Harness (in-process LoopbackSocket, 4k sessions) showed process rss/external growth but no heap holder or timer column grew; reproduce with real net sockets (destroy emits close) over several waves and take 3+ heap snapshots in one process (driver mode / dla attach node-inspector heap-snapshot) to attribute
- Supporting evidence: E6, E19

### F10: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: discriminator, static
- Holder: registration-table
- Owner: `src.router.Router.dispatch`
- Region: heap
- Site: `src/router.js:51`
- Writers: `src/session.js:38`
- Removers: `src/gateway.js:25`, `src/router.js:54`, `src/session.js:33`, `src/session.js:65`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Harness (in-process LoopbackSocket, 4k sessions) showed process rss/external growth but no heap holder or timer column grew; reproduce with real net sockets (destroy emits close) over several waves and take 3+ heap snapshots in one process (driver mode / dla attach node-inspector heap-snapshot) to attribute
- Supporting evidence: E6, E19

### F11: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: discriminator, static
- Holder: container-field
- Owner: `src.session.Session.decoder`
- Region: heap
- Site: `src/session.js:15`
- Writers: `src/session.js:30`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Harness (in-process LoopbackSocket, 4k sessions) showed process rss/external growth but no heap holder or timer column grew; reproduce with real net sockets (destroy emits close) over several waves and take 3+ heap snapshots in one process (driver mode / dla attach node-inspector heap-snapshot) to attribute
- Supporting evidence: E6, E19

### F12: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: discriminator, static
- Holder: container-field
- Owner: `src.session.Session.topics`
- Region: heap
- Site: `src/session.js:19`
- Reason: not_exercised
- Remedy: Harness (in-process LoopbackSocket, 4k sessions) showed process rss/external growth but no heap holder or timer column grew; reproduce with real net sockets (destroy emits close) over several waves and take 3+ heap snapshots in one process (driver mode / dla attach node-inspector heap-snapshot) to attribute
- Supporting evidence: E6, E19

### F13: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: discriminator, static
- Holder: container-field
- Owner: `src.session.Session.recent`
- Region: heap
- Site: `src/session.js:20`
- Writers: `src/session.js:44`
- Removers: `src/session.js:45`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Harness (in-process LoopbackSocket, 4k sessions) showed process rss/external growth but no heap holder or timer column grew; reproduce with real net sockets (destroy emits close) over several waves and take 3+ heap snapshots in one process (driver mode / dla attach node-inspector heap-snapshot) to attribute
- Supporting evidence: E6, E19

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
- E20: partial coverage (2 of 3 examined)
- E20: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E20: limitation: threshold is provisional (not calibrated)

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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (4f4574fe6485) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (3f83d0e35667) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (3f83d0e35667) | trend/T1.json |
| E12 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261109-202853-7512//harness/H1/out/p1-native/capture/9/heap-1.heapsnapshot (6e812c85d52d) | summaries/heap-1-6e812c85.json |
| E13 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261109-202853-7512//harness/H1/out/p4-native/capture/34/heap-1.heapsnapshot (7960f7b070c2) | summaries/heap-1-7960f7b0.json |
| E14 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261109-202853-7512//harness/H1/out/p6-native/capture/50/heap-1.heapsnapshot (8dd075641ac2) | summaries/heap-1-8dd07564.json |
| E15 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261109-202853-7512//harness/H1/out/p9-native/capture/74/heap-1.heapsnapshot (bacbbdc83d4c) | summaries/heap-1-bacbbdc8.json |
| E16 | growth-series | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (7d3516acc373) | series/S2.json |
| E17 | non-heap | live | node-resources 0.1.0 | artifact nonheap/N1.json (bcd842471325) | nonheap/N1.json |
| E18 | growth-series | live | node-resources 0.1.0 | artifact series/S3.json (f6859d5d3d7f) | series/S3.json |
| E19 | discriminator | test | dla 0.1.0 | artifact series/S5.json (67c9f65730ba) | alloc/A1.json |
| E20 | residual | test | dla 0.1.0 | artifact series/S1.json (3f83d0e35667) | gap/residual-S1-E20.json |
