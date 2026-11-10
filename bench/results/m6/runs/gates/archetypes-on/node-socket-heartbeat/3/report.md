# DLA report

Run: `20261110-175109-ff77`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 16 enumerated, 16 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 14

Residual: heap 5099.136363636364 bytes/n unattributed of 5099.136363636364 total (threshold 1274.784090909091) ABOVE THRESHOLD, see O3; heap 4972.726666666667 bytes/n unattributed of 4972.726666666667 total (threshold 1243.1816666666668) ABOVE THRESHOLD, see O3; heap 5043.274696969697 bytes/n unattributed of 5043.274696969697 total (threshold 1260.8186742424243) ABOVE THRESHOLD, see O3; off-heap 6144 bytes/n unattributed of 6144 total (threshold 1536) ABOVE THRESHOLD, see O4; off-heap 6144 bytes/n unattributed of 6144 total (threshold 1536) ABOVE THRESHOLD, see O4; off-heap 6144 bytes/n unattributed of 6144 total (threshold 1536) ABOVE THRESHOLD, see O4

Coverage: node 3520 iterations, 25 of 30 entry points exercised

## Findings

### F1: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Per-session setInterval heartbeat closure captures the session; Session.close() never clearInterval()s it, so each closed session (socket, decoder, recent frames) stays reachable from the timer list)
- Growth: `node.timers@src/heartbeat.js:15` 1 count/n (steady, gate pass, E11)
- Evidence classes: context, discriminator, growth, static
- Holder: closure/continuation
- Region: non-heap
- Shape: steady
- Site: `src/heartbeat.js:15`
- Writers: `src/heartbeat.js:15`
- Annotation: setInterval closure captures session; Session.close never calls clearInterval(session.heartbeat)
- Verified: no
- Sibling hypotheses: F1.n1
- Suggested archetype (unconfirmed): L-WORKER
- Supporting evidence: E5, E9, E11, E12
- Explanation (inferred): startHeartbeat at src/heartbeat.js:15 registers a setInterval whose closure/continuation holds the session; no remover found: Session.close never calls clearInterval, so each closed session stays alive and timers grow per connection.
- Explanation structure: adds: src/heartbeat.js:15; removal: not-found

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.metrics`
- Region: heap
- Site: `src/gateway.js:13`
- Reason: not_exercised
- Remedy: Census lead only: no growth column tied to this holder in the harness (sessions map drains on close); no action unless a later series shows growth
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.router`
- Region: heap
- Site: `src/gateway.js:14`
- Reason: not_exercised
- Remedy: Census lead only: no growth column tied to this holder in the harness (sessions map drains on close); no action unless a later series shows growth
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

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
- Remedy: Census lead only: no growth column tied to this holder in the harness (sessions map drains on close); no action unless a later series shows growth
- Supporting evidence: E6

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `src.gateway.Gateway.accept`
- Region: heap
- Site: `src/gateway.js:24`
- Writers: `src/gateway.js:25`, `src/session.js:12`, `src/session.js:54`
- Removers: `load.js:50`, `src/gateway.js:31`, `src/session.js:66`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Downstream effect of F1: socket data/close listeners are only counted because leaked sessions keep their sockets reachable; re-measure node.listeners after clearing the heartbeat timer on close
- Supporting evidence: E6

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `src.gateway.Gateway.accept`
- Region: heap
- Site: `src/gateway.js:25`
- Writers: `src/gateway.js:24`, `src/session.js:12`, `src/session.js:54`
- Removers: `load.js:50`, `src/gateway.js:31`, `src/session.js:66`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Downstream effect of F1: socket data/close listeners are only counted because leaked sessions keep their sockets reachable; re-measure node.listeners after clearing the heartbeat timer on close
- Supporting evidence: E6

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `src.heartbeat.startHeartbeat`
- Region: non-heap
- Site: `src/heartbeat.js:15`
- Reason: not_exercised
- Remedy: Same timer as F1 (src/heartbeat.js:15), recorded under F1; duplicate census view
- Supporting evidence: E6

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.loopback.LoopbackSocket.written`
- Region: heap
- Site: `src/loopback.js:11`
- Writers: `src/loopback.js:17`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Census lead only: no growth column tied to this holder in the harness (sessions map drains on close); no action unless a later series shows growth
- Supporting evidence: E6

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.metrics.Metrics.counters`
- Region: heap
- Site: `src/metrics.js:7`
- Writers: `src/metrics.js:12`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Census lead only: no growth column tied to this holder in the harness (sessions map drains on close); no action unless a later series shows growth
- Supporting evidence: E6

### F10: **UNCLASSIFIED**

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
- Remedy: Census lead only: no growth column tied to this holder in the harness (sessions map drains on close); no action unless a later series shows growth
- Supporting evidence: E6

### F11: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `src.router.Router.dispatch`
- Region: heap
- Site: `src/router.js:51`
- Writers: `src/session.js:38`
- Removers: `src/gateway.js:25`, `src/router.js:54`, `src/session.js:33`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Census lead only: no growth column tied to this holder in the harness (sessions map drains on close); no action unless a later series shows growth
- Supporting evidence: E6

### F12: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.decoder`
- Region: heap
- Site: `src/session.js:15`
- Writers: `src/session.js:30`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Census lead only: no growth column tied to this holder in the harness (sessions map drains on close); no action unless a later series shows growth
- Supporting evidence: E6

### F13: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.topics`
- Region: heap
- Site: `src/session.js:19`
- Reason: not_exercised
- Remedy: Census lead only: no growth column tied to this holder in the harness (sessions map drains on close); no action unless a later series shows growth
- Supporting evidence: E6

### F14: **UNCLASSIFIED**

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
- Remedy: Census lead only: no growth column tied to this holder in the harness (sessions map drains on close); no action unless a later series shows growth
- Supporting evidence: E6

### F15: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: Session.recent and LoopbackSocket.written grow only because F1 keeps closed sessions reachable; DLA preload listenerRefs is the instrument (N-OBSERVER). Re-measure after fixing F1
- Supporting evidence: E23, E24

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM: timer count is warm-up/steady-state window, not growth
  - Refuted if: timer count plateaus over extra cycles / purge
  - Contradicting evidence: E12
  - Originally supported by: E11

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
- E12: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E12: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E12: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E12: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E12: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E12: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E12: limitation: points 1 to 9 come from S1; the extra points from S2
- E12: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E12: limitation: gate and plateau thresholds are provisional (not calibrated)
- E13: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E13: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E14: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E15: limitation: weak edges are skipped; string contents are never read into the summary
- E16: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E16: limitation: weak edges are skipped; string contents are never read into the summary
- E17: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E17: limitation: weak edges are skipped; string contents are never read into the summary
- E18: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E18: limitation: weak edges are skipped; string contents are never read into the summary
- E19: limitation: dump-series rule: at least 3 snapshots, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E19: limitation: series: per-point; one process per point at n = 440, 1760, 2640, 3960 (harness captures; checked: snapshot format)
- E20: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E21: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E22: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E23: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E23: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E23: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E24: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E24: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: partial coverage (2 of 3 examined)
- E25: limitation: F1: growth measured as a count; bytes not attributed
- E25: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E25: limitation: threshold is provisional (not calibrated)
- E26: partial coverage (2 of 3 examined)
- E26: limitation: F1: growth evidence is from S1, not S2; not subtracted
- E26: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E26: limitation: threshold is provisional (not calibrated)
- E27: partial coverage (2 of 3 examined)
- E27: limitation: F1: growth evidence is from S1, not S3; not subtracted
- E27: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E27: limitation: threshold is provisional (not calibrated)
- E28: partial coverage (2 of 3 examined)
- E28: limitation: F1: growth measured as a count; bytes not attributed
- E28: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E28: limitation: threshold is provisional (not calibrated)
- E29: partial coverage (2 of 3 examined)
- E29: limitation: F1: growth evidence is from S1, not S2; not subtracted
- E29: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E29: limitation: threshold is provisional (not calibrated)
- E30: partial coverage (2 of 3 examined)
- E30: limitation: F1: growth evidence is from S1, not S3; not subtracted
- E30: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E30: limitation: threshold is provisional (not calibrated)

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
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | — |
| E2 | run-config | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | probe/probe.json |
| E4 | static-hit | static | eslint 10.14.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | sweep/node-eslint.json |
| E5 | static-hit | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | sweep/node-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | census/node.json |
| E7 | workload | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (5f89b9d3a62a) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (8d07b44faeba) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (8d07b44faeba) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (41021bb7943b) | alloc/A1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S2.json (2fbacd916b46) | trend/T3.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (2fbacd916b46) | trend/T3.json |
| E15 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-175109-ff77/harness/H1/out/p1-native/capture/9/heap-1.heapsnapshot (08cacc852de8) | summaries/heap-1-08cacc85.json |
| E16 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-175109-ff77/harness/H1/out/p4-native/capture/34/heap-1.heapsnapshot (35bb224a98fc) | summaries/heap-1-35bb224a.json |
| E17 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-175109-ff77/harness/H1/out/p6-native/capture/50/heap-1.heapsnapshot (eed0a5a6e394) | summaries/heap-1-eed0a5a6.json |
| E18 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-175109-ff77/harness/H1/out/p9-native/capture/74/heap-1.heapsnapshot (1d3a724104c5) | summaries/heap-1-1d3a7241.json |
| E19 | growth-series | dump | dla-v8-snapshot 0.1.0 | artifact series/S4.json (a213c9be0687) | series/S4.json |
| E20 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S4.json (a213c9be0687) | series/S4.json |
| E21 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S4.json (a213c9be0687) | series/S4.json |
| E22 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S4.json (a213c9be0687) | series/S4.json |
| E23 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (a213c9be0687) | trend/T4.json |
| E24 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (a213c9be0687) | trend/T4.json |
| E25 | residual | test | dla 0.1.0 | artifact series/S1.json (8d07b44faeba) | gap/residual-S1-E25.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S2.json (2fbacd916b46) | gap/residual-S2-E26.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S3.json (41021bb7943b) | gap/residual-S3-E27.json |
| E28 | residual | test | dla 0.1.0 | artifact series/S1.json (8d07b44faeba) | gap/residual-S1-E28.json |
| E29 | residual | test | dla 0.1.0 | artifact series/S2.json (2fbacd916b46) | gap/residual-S2-E29.json |
| E30 | residual | test | dla 0.1.0 | artifact series/S3.json (41021bb7943b) | gap/residual-S3-E30.json |
