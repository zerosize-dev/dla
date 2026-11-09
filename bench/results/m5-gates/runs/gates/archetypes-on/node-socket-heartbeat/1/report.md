# DLA report

Run: `20261109-195357-8b6e`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 15 enumerated, 15 dispositioned; confirmed 1, strong 0, refuted 1, suspected 13, unresolved 0

Residual: not measured

Coverage: node 3520 iterations, 25 of 30 entry points exercised

## Findings

### F1: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `group:handle/Timeout@src/heartbeat.js:15` 1173 count/dump (steady, gate pass, E13)
- Evidence classes: context, discriminator, growth, static
- Holder: closure/continuation
- Owner: `src.heartbeat.startHeartbeat`
- Region: non-heap
- Shape: steady
- Site: `src/heartbeat.js:15`
- Writers: `src/heartbeat.js:15`
- Annotation: startHeartbeat creates a setInterval per session (gateway.js:25), handle stored in session.heartbeat; Session.close()/Gateway.release never clearInterval. beat() returns early when closed but the interval stays registered, so each closed session (socket, decoder, recent[64], topics) stays reachable via the timer's closure for process lifetime. unref() only stops it keeping the loop alive. Missing pairing: clearInterval(session.heartbeat) on close.
- Sibling hypotheses: F1.n1
- Suggested archetype (unconfirmed): L-WORKER
- Supporting evidence: E5, E9, E13, E14, E11

### F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.metrics`
- Region: heap
- Site: `src/gateway.js:13`
- Reason: not_exercised
- Remedy: static census lead; no growth column tied to it in the harness (sessions/topics are removed on close). Add a driver probe if doubt remains
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.router`
- Region: heap
- Site: `src/gateway.js:14`
- Reason: not_exercised
- Remedy: static census lead; no growth column tied to it in the harness (sessions/topics are removed on close). Add a driver probe if doubt remains
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.sessions`
- Region: heap
- Site: `src/gateway.js:15`
- Writers: `src/gateway.js:22`
- Removers: `src/gateway.js:32`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: static census lead; no growth column tied to it in the harness (sessions/topics are removed on close). Add a driver probe if doubt remains
- Supporting evidence: E6

### F5: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `src.gateway.Gateway.accept`
- Region: heap
- Site: `src/gateway.js:24`
- Writers: `src/gateway.js:25`, `src/session.js:12`, `src/session.js:54`
- Removers: `load.js:50`, `src/gateway.js:31`, `src/session.js:66`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: listener growth at gateway.js:24 tracks the same leaked sessions retained by F1 (timer closure keeps socket); not independently confirmed. Re-run after clearInterval fix and check node.listeners
- Supporting evidence: E6

### F6: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `src.gateway.Gateway.accept`
- Region: heap
- Site: `src/gateway.js:25`
- Writers: `src/gateway.js:24`, `src/session.js:12`, `src/session.js:54`
- Removers: `load.js:50`, `src/gateway.js:31`, `src/session.js:66`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: listener growth at gateway.js:24 tracks the same leaked sessions retained by F1 (timer closure keeps socket); not independently confirmed. Re-run after clearInterval fix and check node.listeners
- Supporting evidence: E6

### F7: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `src.heartbeat.startHeartbeat`
- Region: non-heap
- Site: `src/heartbeat.js:15`
- Reason: not_exercised
- Remedy: same site as confirmed F1 (heartbeat.js:15); reported under F1
- Supporting evidence: E6

### F8: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.loopback.LoopbackSocket.written`
- Region: heap
- Site: `src/loopback.js:11`
- Writers: `src/loopback.js:17`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: static census lead; no growth column tied to it in the harness (sessions/topics are removed on close). Add a driver probe if doubt remains
- Supporting evidence: E6

### F9: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.metrics.Metrics.counters`
- Region: heap
- Site: `src/metrics.js:7`
- Writers: `src/metrics.js:12`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: static census lead; no growth column tied to it in the harness (sessions/topics are removed on close). Add a driver probe if doubt remains
- Supporting evidence: E6

### F10: **UNCLASSIFIED**

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
- Remedy: static census lead; no growth column tied to it in the harness (sessions/topics are removed on close). Add a driver probe if doubt remains
- Supporting evidence: E6

### F11: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `src.router.Router.dispatch`
- Region: heap
- Site: `src/router.js:51`
- Writers: `src/session.js:38`
- Removers: `src/gateway.js:25`, `src/router.js:54`, `src/session.js:33`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: static census lead; no growth column tied to it in the harness (sessions/topics are removed on close). Add a driver probe if doubt remains
- Supporting evidence: E6

### F12: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.decoder`
- Region: heap
- Site: `src/session.js:15`
- Writers: `src/session.js:30`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: static census lead; no growth column tied to it in the harness (sessions/topics are removed on close). Add a driver probe if doubt remains
- Supporting evidence: E6

### F13: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.topics`
- Region: heap
- Site: `src/session.js:19`
- Reason: not_exercised
- Remedy: static census lead; no growth column tied to it in the harness (sessions/topics are removed on close). Add a driver probe if doubt remains
- Supporting evidence: E6

### F14: **UNCLASSIFIED**

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
- Remedy: static census lead; no growth column tied to it in the harness (sessions/topics are removed on close). Add a driver probe if doubt remains
- Supporting evidence: E6

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (leak) E14
  - Hypothesis: N-WARM: timers plateau after warm-up
  - Refuted if: Timeout count at heartbeat.js:15 stops growing over extra cycles
  - Contradicting evidence: E14
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
- E12: limitation: sites are the first project frame of the stack that created the resource; a resource with no project frame is unattributed
- E12: limitation: active counts come from getActiveResourcesInfo and leave out unref'd timers
- E12: limitation: the files are ordered as given; each is one point
- E13: limitation: sites are the first project frame of the stack that created the resource; a resource with no project frame is unattributed
- E13: limitation: active counts come from getActiveResourcesInfo and leave out unref'd timers
- E13: limitation: the files are ordered as given; each is one point
- E13: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E14: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E14: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E14: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E14: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E14: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E14: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E14: limitation: points 1 to 9 come from S1; the extra points from S3
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | — |
| E2 | run-config | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | probe/probe.json |
| E4 | static-hit | static | eslint 10.14.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | sweep/node-eslint.json |
| E5 | static-hit | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | sweep/node-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | census/node.json |
| E7 | workload | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (20cbb05990ef) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (f9651d098639) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (f9651d098639) | trend/T1.json |
| E12 | non-heap | live | node-resources 0.1.0 | artifact nonheap/N1.json (d83b8b50ac90) | nonheap/N1.json |
| E13 | growth-series | live | node-resources 0.1.0 | artifact series/S2.json (b4a2b681ae42) | series/S2.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S4.json (80211054d399) | alloc/A1.json |
