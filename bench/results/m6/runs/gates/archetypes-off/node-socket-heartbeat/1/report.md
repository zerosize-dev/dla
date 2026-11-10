# DLA report

Run: `20261110-183005-70a8`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 16 enumerated, 16 dispositioned; confirmed 2, strong 0, refuted 2, suspected 0, unresolved 12

Residual: heap 5107.652727272727 bytes/n unattributed of 5107.652727272727 total (threshold 1276.9131818181818) ABOVE THRESHOLD, see O1; heap 4983.459848484848 bytes/n unattributed of 4983.459848484848 total (threshold 1245.864962121212) ABOVE THRESHOLD, see O1; heap 5043.9157024793385 bytes/n unattributed of 5043.9157024793385 total (threshold 1260.9789256198346) ABOVE THRESHOLD, see O1; heap 50.36147186147186 bytes/n unattributed of 50.36147186147186 total (threshold 26.808791298701333) ABOVE THRESHOLD, see O1; heap 4968.184848484849 bytes/n unattributed of 4968.184848484849 total (threshold 1242.0462121212122) ABOVE THRESHOLD, see O1; heap 5039.489090909091 bytes/n unattributed of 5039.489090909091 total (threshold 1259.8722727272727) ABOVE THRESHOLD, see O3; off-heap 6144 bytes/n unattributed of 6144 total (threshold 1536) ABOVE THRESHOLD, see O2; off-heap 6144 bytes/n unattributed of 6144 total (threshold 1536) ABOVE THRESHOLD, see O2; off-heap 6144 bytes/n unattributed of 6144 total (threshold 1536) ABOVE THRESHOLD, see O2; off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 6144 bytes/n unattributed of 6144 total (threshold 1536) ABOVE THRESHOLD, see O2; off-heap 6144 bytes/n unattributed of 6144 total (threshold 1536) ABOVE THRESHOLD, see O4

Coverage: node 3520 iterations, 25 of 30 entry points exercised

## Findings

### F5: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (socket data listener (gateway.js:24) retained on closed sockets: downstream effect of the uncleared heartbeat timer, which keeps session and socket reachable)
- Growth: `node.listeners@src/gateway.js:24` 1 count/n (steady, gate pass, E14)
- Evidence classes: context, discriminator, growth, static
- Holder: registration-table
- Owner: `src.gateway.Gateway.accept`
- Region: heap
- Shape: steady
- Site: `src/gateway.js:24`
- Writers: `src/gateway.js:25`, `src/session.js:12`, `src/session.js:54`, `src/gateway.js:24`
- Removers: `load.js:50`, `src/gateway.js:31`, `src/session.js:66`
- Annotation: heuristic name match over the project's sources
- Annotation: data/close listeners on socket; socket stays reachable via heartbeat timer closure
- Verified: no
- Sibling hypotheses: F5.n1
- Supporting evidence: E6, E9, E14, E16
- Explanation (inferred): Gateway.accept at src/gateway.js:24 adds a data listener to each socket in the registration table; removal (load.js:50 hang-up) works only when the socket is collected, but the uncleared heartbeat timer keeps it reachable.
- Explanation structure: adds: src/gateway.js:24; removal: conditional at load.js:50

### F7: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (setInterval heartbeat timer per session (src/heartbeat.js:15) is never cleared when the session closes; the live timer keeps the session, socket, decoder and recent buffer reachable)
- Growth: `node.timers@src/heartbeat.js:15` 1 count/n (steady, gate pass, E11)
- Evidence classes: context, discriminator, growth, static
- Holder: blocked-stack
- Owner: `src.heartbeat.startHeartbeat`
- Region: non-heap
- Shape: steady
- Site: `src/heartbeat.js:15`
- Writers: `src/heartbeat.js:15`
- Annotation: setInterval per session; no clearInterval anywhere
- Annotation: heartbeat beat() returns early on closed sessions but never clears the interval
- Verified: no
- Sibling hypotheses: F7.n1
- Supporting evidence: E6, E9, E11, E12
- Explanation (inferred): startHeartbeat in src/heartbeat.js:15 registers a setInterval timer per session in the timer table; Session.close never calls clearInterval, so each closed session stays reachable through the timer closure.
- Explanation structure: adds: src/heartbeat.js:15; removal: never

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: blocked-stack
- Site: `src/heartbeat.js:15`
- Writers: `src/heartbeat.js:15`
- Annotation: setInterval per session never cleared on close
- Reason: not_exercised
- Remedy: duplicate of F7 (same site src/heartbeat.js:15); covered there
- Supporting evidence: E5

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.metrics`
- Region: heap
- Site: `src/gateway.js:13`
- Reason: not_exercised
- Remedy: census lead only; no tied growth column in harness H1, holder not shown to grow (bounded or released with session)
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.router`
- Region: heap
- Site: `src/gateway.js:14`
- Reason: not_exercised
- Remedy: census lead only; no tied growth column in harness H1, holder not shown to grow (bounded or released with session)
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
- Remedy: census lead only; no tied growth column in harness H1, holder not shown to grow (bounded or released with session)
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
- Remedy: duplicate of F5 (same registration site src/gateway.js); covered there
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
- Remedy: census lead only; no tied growth column in harness H1, holder not shown to grow (bounded or released with session)
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
- Remedy: census lead only; no tied growth column in harness H1, holder not shown to grow (bounded or released with session)
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
- Remedy: census lead only; no tied growth column in harness H1, holder not shown to grow (bounded or released with session)
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
- Remedy: census lead only; no tied growth column in harness H1, holder not shown to grow (bounded or released with session)
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
- Remedy: census lead only; no tied growth column in harness H1, holder not shown to grow (bounded or released with session)
- Supporting evidence: E6

### F13: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.topics`
- Region: heap
- Site: `src/session.js:19`
- Reason: not_exercised
- Remedy: census lead only; no tied growth column in harness H1, holder not shown to grow (bounded or released with session)
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
- Remedy: census lead only; no tied growth column in harness H1, holder not shown to grow (bounded or released with session)
- Supporting evidence: E6

## Ruled out

- **F5.n1** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (leak) E16
  - Hypothesis: N-WARM: listener count plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E16
  - Originally supported by: E14
- **F7.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM: timer count plateaus after warm-up
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E12
  - Originally supported by: E12

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
- E15: limitation: tested in a scratch copy; the project was not changed
- E15: limitation: gate thresholds are provisional (not calibrated); the 0.2 and 0.5 slope fractions are placeholders
- E15: limitation: exercised is null: the harness records entry points, not lines, so whether the edited line ran is not checked
- E15: limitation: the treated series lacks the control column node.timers@src/heartbeat.js:15: a patch can rename an owner or move a site, so a patch must keep the measured column to be judged
- E15: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E15: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal time...
- E16: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E16: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E16: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E16: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E16: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E16: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E16: limitation: points 1 to 9 come from S1; the extra points from S5
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E17: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E18: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: partial coverage (2 of 3 examined)
- E19: limitation: F5: growth evidence is from S2, not S1; not subtracted
- E19: limitation: F7: growth measured as a count; bytes not attributed
- E19: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E19: limitation: threshold is provisional (not calibrated)
- E20: partial coverage (2 of 3 examined)
- E20: limitation: F5: growth measured as a count; bytes not attributed
- E20: limitation: F7: growth evidence is from S1, not S2; not subtracted
- E20: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E20: limitation: threshold is provisional (not calibrated)
- E21: partial coverage (2 of 3 examined)
- E21: limitation: F5: growth evidence is from S2, not S3; not subtracted
- E21: limitation: F7: growth evidence is from S1, not S3; not subtracted
- E21: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E21: limitation: threshold is provisional (not calibrated)
- E22: partial coverage (2 of 3 examined)
- E22: limitation: F5: growth evidence is from S2, not S4; not subtracted
- E22: limitation: F7: growth evidence is from S1, not S4; not subtracted
- E22: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E22: limitation: O2 (off-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E22: limitation: threshold is provisional (not calibrated)
- E23: partial coverage (2 of 3 examined)
- E23: limitation: F5: growth evidence is from S2, not S5; not subtracted
- E23: limitation: F7: growth evidence is from S1, not S5; not subtracted
- E23: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E23: limitation: threshold is provisional (not calibrated)
- E24: partial coverage (2 of 3 examined)
- E24: limitation: F5: growth evidence is from S2, not S6; not subtracted
- E24: limitation: F7: growth evidence is from S1, not S6; not subtracted
- E24: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E24: limitation: threshold is provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | — |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E2 | — |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E2 | — |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E2 | — |
| `intake.build` | Does the project build locally? | answered | E2 | — |
| `intake.workload` | What can exercise the suspected path? | answered | E2 | — |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E2 | — |

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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (0532fdd21753) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (f2b25eadfb54) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (f2b25eadfb54) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (ffd951161156) | alloc/A1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S2.json (a2dfa3b20bb4) | trend/T3.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (a2dfa3b20bb4) | trend/T3.json |
| E15 | intervention | test | dla 0.1.0 | artifact series/S4.json (d3b28e6295ef) | intervene/I1 |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S6.json (490d3d6b88b3) | alloc/A2.json |
| E17 | growth-series | test | dla 0.1.0 | artifact series/S5.json (2f96445de3bb) | trend/T6.json |
| E18 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (2f96445de3bb) | trend/T6.json |
| E19 | residual | test | dla 0.1.0 | artifact series/S1.json (f2b25eadfb54) | gap/residual-S1-E19.json |
| E20 | residual | test | dla 0.1.0 | artifact series/S2.json (a2dfa3b20bb4) | gap/residual-S2-E20.json |
| E21 | residual | test | dla 0.1.0 | artifact series/S3.json (ffd951161156) | gap/residual-S3-E21.json |
| E22 | residual | test | dla 0.1.0 | artifact series/S4.json (d3b28e6295ef) | gap/residual-S4-E22.json |
| E23 | residual | test | dla 0.1.0 | artifact series/S5.json (2f96445de3bb) | gap/residual-S5-E23.json |
| E24 | residual | test | dla 0.1.0 | artifact series/S6.json (490d3d6b88b3) | gap/residual-S6-E24.json |
