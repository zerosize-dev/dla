# DLA report

Run: `20261108-125806-e082`

Status: COMPLETE

Findings: 16 enumerated, 16 dispositioned; confirmed 2, strong 0, refuted 2, suspected 0, unresolved 12

Residual: not measured

Coverage: node 3520 iterations, 25 of 30 entry points exercised

## Findings

### F1: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: not tied to this finding (passing columns of E11: `node.array_buffers`, `node.array_buffers.pre_gc`, `node.external`, `node.external.pre_gc`, `node.heap_used`)
- Evidence classes: context, growth, static
- Holder: closure/continuation
- Owner: `heartbeat setInterval closure in startHeartbeat`
- Site: `src/heartbeat.js:15`
- Writers: `src/heartbeat.js:15`
- Annotation: Gateway.release/Session.close never clearInterval(session.heartbeat); the live interval's closure keeps Session (socket.written[], decoder, recent) reachable after disconnect
- Sibling hypotheses: F1.n1
- Suggested archetype (unconfirmed): L-WORKER
- Supporting evidence: E6, E10, E11

### F5: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: not tied to this finding (passing columns of E11: `node.array_buffers`, `node.array_buffers.pre_gc`, `node.external`, `node.external.pre_gc`, `node.heap_used`)
- Evidence classes: context, growth, static
- Holder: registration-table
- Owner: `src.gateway.Gateway.accept`
- Region: heap
- Site: `src/gateway.js:24`
- Writers: `src/gateway.js:25`, `src/session.js:12`, `src/session.js:54`
- Removers: `load.js:50`, `src/gateway.js:31`, `src/session.js:66`
- Annotation: heuristic name match over the project's sources
- Sibling hypotheses: F5.n1
- Supporting evidence: E7, E10, E11

### F2: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.metrics`
- Region: heap
- Site: `src/gateway.js:13`
- Reason: time_budget
- Remedy: Bounded/cleaned holder per source review (released with session or capped); no growth column tied to it; not individually tested
- Supporting evidence: E7

### F3: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.router`
- Region: heap
- Site: `src/gateway.js:14`
- Reason: time_budget
- Remedy: Bounded/cleaned holder per source review (released with session or capped); no growth column tied to it; not individually tested
- Supporting evidence: E7

### F4: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.sessions`
- Region: heap
- Site: `src/gateway.js:15`
- Writers: `src/gateway.js:22`
- Removers: `src/gateway.js:32`
- Annotation: heuristic name match over the project's sources
- Reason: time_budget
- Remedy: Bounded/cleaned holder per source review (released with session or capped); no growth column tied to it; not individually tested
- Supporting evidence: E7

### F6: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: registration-table
- Owner: `src.gateway.Gateway.accept`
- Region: heap
- Site: `src/gateway.js:25`
- Writers: `src/gateway.js:24`, `src/session.js:12`, `src/session.js:54`
- Removers: `load.js:50`, `src/gateway.js:31`, `src/session.js:66`
- Annotation: heuristic name match over the project's sources
- Reason: time_budget
- Remedy: Sibling listener site of F5 (socket 'close' once-listener); follows same retention as F1
- Supporting evidence: E7

### F7: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth, static
- Holder: blocked-stack
- Owner: `src.heartbeat.startHeartbeat`
- Region: non-heap
- Site: `src/heartbeat.js:15`
- Reason: time_budget
- Remedy: Same timer site as confirmed F1; fixed by clearing the interval on session close
- Supporting evidence: E7, E11

### F8: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: container-field
- Owner: `src.loopback.LoopbackSocket.written`
- Region: heap
- Site: `src/loopback.js:11`
- Writers: `src/loopback.js:17`
- Annotation: heuristic name match over the project's sources
- Reason: time_budget
- Remedy: Bounded/cleaned holder per source review (released with session or capped); no growth column tied to it; not individually tested
- Supporting evidence: E7

### F9: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: container-field
- Owner: `src.metrics.Metrics.counters`
- Region: heap
- Site: `src/metrics.js:7`
- Writers: `src/metrics.js:12`
- Annotation: heuristic name match over the project's sources
- Reason: time_budget
- Remedy: Bounded/cleaned holder per source review (released with session or capped); no growth column tied to it; not individually tested
- Supporting evidence: E7

### F10: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: container-field
- Owner: `src.router.Router.topics`
- Region: heap
- Site: `src/router.js:9`
- Writers: `src/router.js:16`, `src/router.js:19`
- Removers: `src/router.js:26`, `src/router.js:27`
- Annotation: heuristic name match over the project's sources
- Reason: time_budget
- Remedy: Bounded/cleaned holder per source review (released with session or capped); no growth column tied to it; not individually tested
- Supporting evidence: E7

### F11: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: registration-table
- Owner: `src.router.Router.dispatch`
- Region: heap
- Site: `src/router.js:51`
- Writers: `src/session.js:38`
- Removers: `src/gateway.js:25`, `src/router.js:54`, `src/session.js:33`
- Annotation: heuristic name match over the project's sources
- Reason: time_budget
- Remedy: Bounded/cleaned holder per source review (released with session or capped); no growth column tied to it; not individually tested
- Supporting evidence: E7

### F12: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.decoder`
- Region: heap
- Site: `src/session.js:15`
- Writers: `src/session.js:30`
- Annotation: heuristic name match over the project's sources
- Reason: time_budget
- Remedy: Bounded/cleaned holder per source review (released with session or capped); no growth column tied to it; not individually tested
- Supporting evidence: E7

### F13: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.topics`
- Region: heap
- Site: `src/session.js:19`
- Reason: time_budget
- Remedy: Bounded/cleaned holder per source review (released with session or capped); no growth column tied to it; not individually tested
- Supporting evidence: E7

### F14: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.recent`
- Region: heap
- Site: `src/session.js:20`
- Writers: `src/session.js:44`
- Removers: `src/session.js:45`
- Annotation: heuristic name match over the project's sources
- Reason: time_budget
- Remedy: Bounded/cleaned holder per source review (released with session or capped); no growth column tied to it; not individually tested
- Supporting evidence: E7

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (leak) E13
  - Hypothesis: N-WARM timers are a warm-up/pool plateau, not per-connection growth
  - Refuted if: timer count keeps growing over extra cycles after purge
  - Contradicting evidence: E13
  - Originally supported by: E12
- **F5.n1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (leak) E14
  - Hypothesis: N-WARM listener count plateaus
  - Refuted if: listener count keeps growing over extra cycles
  - Contradicting evidence: E14
  - Originally supported by: E12

## Blind spots and unknowns

### Coverage gaps

None recorded.

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
- E10: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E10: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E11: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E11: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E12: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E13: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E13: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E13: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E13: limitation: points 1 to 9 come from S1; the extra points from S2
- E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- E14: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E14: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E14: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E14: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E14: limitation: points 1 to 9 come from S1; the extra points from S4
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | edge-gateway node project in cwd |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | from task text/repo: node, load.js workload, 60m budget; RSS grows per connect/disconnect wave |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | from task text/repo: node, load.js workload, 60m budget; RSS grows per connect/disconnect wave |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | from task text/repo: node, load.js workload, 60m budget; RSS grows per connect/disconnect wave |
| `intake.build` | Does the project build locally? | answered | E1 | from task text/repo: node, load.js workload, 60m budget; RSS grows per connect/disconnect wave |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | from task text/repo: node, load.js workload, 60m budget; RSS grows per connect/disconnect wave |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 raw |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | from task text/repo: node, load.js workload, 60m budget; RSS grows per connect/disconnect wave |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | — |
| E2 | run-config | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | probe/probe.json |
| E4 | probe | static | dla-probe 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | probe/probe.json |
| E5 | static-hit | static | eslint 10.14.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | sweep/eslint.json |
| E6 | static-hit | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | sweep/dla.json |
| E7 | census-holder | static | dla-census 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | census/node.json |
| E8 | workload | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | workload/workloads.json |
| E9 | workload | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | harness/H1/harness.json |
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (5c161f0fa017) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (43aeb726fc3a) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (43aeb726fc3a) | trend/T1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S3.json (42185ad7cc10) | alloc/A1.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S5.json (f80eb93ea17a) | alloc/A2.json |
