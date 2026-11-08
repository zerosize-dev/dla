# DLA report

Run: `20261108-194444-4284`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 14 enumerated, 14 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 12

Residual: not measured

Coverage: node 7040 iterations, 25 of 30 entry points exercised

## Findings

### F3: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: not tied to this finding (passing columns of E16: `node.listeners@src/gateway.js:24`, `node.listeners@src/gateway.js:25`, `node.listeners`)
- Evidence classes: context, growth, static
- Holder: container-field
- Owner: `src.gateway.Gateway.sessions`
- Region: heap
- Site: `src/gateway.js:15`
- Writers: `src/gateway.js:22`, `src/gateway.js:21`
- Removers: `src/gateway.js:32`, `src/gateway.js:34`
- Annotation: heuristic name match over the project's sources
- Annotation: Gateway.sessions (container-field) pins Session, socket, decoder, recent buffer and the two socket listeners (gateway.js:24-25). Entry is removed only in release(), which runs only on the socket 'close' event. Server-initiated closes (bye, idle timeout, protocol error) in Session.close clear the heartbeat timer and destroy the socket but never call Gateway.release; with a socket whose destroy() does not emit 'close' (LoopbackSocket) the entry stays forever. Heartbeat timers are cleared correctly (node.timers=0), so the timer is not the holder. Missing pairing: sessions.delete on every close path. Repair class: remove-entry. Plain hang-up workload (load.js) plateaus, so it does not exercise this path. Caveat: trend did not tie heap_used to F3 by probe; growth tied via listeners at gateway.js:24/25 and heap_used slope ~3.4KB/session.
- Sibling hypotheses: F3.n1
- Supporting evidence: E6, E14, E16

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.metrics`
- Region: heap
- Site: `src/gateway.js:13`
- Reason: not_exercised
- Remedy: Not independently exercised; F4/F5 listener growth is the same pinned-session mechanism as F3, others showed no growth column. Add a probe and rerun if suspected.
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.router`
- Region: heap
- Site: `src/gateway.js:14`
- Reason: not_exercised
- Remedy: Not independently exercised; F4/F5 listener growth is the same pinned-session mechanism as F3, others showed no growth column. Add a probe and rerun if suspected.
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
- Remedy: Not independently exercised; F4/F5 listener growth is the same pinned-session mechanism as F3, others showed no growth column. Add a probe and rerun if suspected.
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
- Remedy: Not independently exercised; F4/F5 listener growth is the same pinned-session mechanism as F3, others showed no growth column. Add a probe and rerun if suspected.
- Supporting evidence: E6

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `src.heartbeat.startHeartbeat`
- Region: non-heap
- Site: `src/heartbeat.js:15`
- Reason: not_exercised
- Remedy: Not independently exercised; F4/F5 listener growth is the same pinned-session mechanism as F3, others showed no growth column. Add a probe and rerun if suspected.
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
- Remedy: Not independently exercised; F4/F5 listener growth is the same pinned-session mechanism as F3, others showed no growth column. Add a probe and rerun if suspected.
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
- Remedy: Not independently exercised; F4/F5 listener growth is the same pinned-session mechanism as F3, others showed no growth column. Add a probe and rerun if suspected.
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
- Remedy: Not independently exercised; F4/F5 listener growth is the same pinned-session mechanism as F3, others showed no growth column. Add a probe and rerun if suspected.
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
- Remedy: Not independently exercised; F4/F5 listener growth is the same pinned-session mechanism as F3, others showed no growth column. Add a probe and rerun if suspected.
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
- Remedy: Not independently exercised; F4/F5 listener growth is the same pinned-session mechanism as F3, others showed no growth column. Add a probe and rerun if suspected.
- Supporting evidence: E6

### F12: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.topics`
- Region: heap
- Site: `src/session.js:19`
- Reason: not_exercised
- Remedy: Not independently exercised; F4/F5 listener growth is the same pinned-session mechanism as F3, others showed no growth column. Add a probe and rerun if suspected.
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
- Remedy: Not independently exercised; F4/F5 listener growth is the same pinned-session mechanism as F3, others showed no growth column. Add a probe and rerun if suspected.
- Supporting evidence: E6

## Ruled out

- **F3.n1** (UNCLASSIFIED): discriminator E18
  - Ruled out by: plateau+purge (leak) E18
  - Hypothesis: N-WARM: sessions growth is a warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles of the alloc-test
  - Contradicting evidence: E18
  - Originally supported by: E16

## Blind spots and unknowns

### Coverage gaps

None recorded.

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
- E9: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E9: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E10: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E10: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E11: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: harness written; nothing was run (dla harness run measures it)
- E12: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E13: limitation: harness written; nothing was run (dla harness run measures it)
- E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: partial coverage (9 of 9 examined)
- E14: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E14: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E15: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E15: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E16: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: fresh keys S2 against repeated keys S3
- E17: limitation: no column of the series is tied to the finding, so the comparison is not bound to it
- E18: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E18: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E18: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E18: limitation: gate and plateau thresholds are provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | — |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | — |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | — |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E3 | — |
| `intake.build` | Does the project build locally? | answered | E3 | — |
| `intake.workload` | What can exercise the suspected path? | answered | E3 | — |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (e594dfea1440) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (d5e1fd59905c) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (d5e1fd59905c) | trend/T1.json |
| E12 | workload | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | harness/H2/harness.json |
| E13 | workload | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | harness/H3/harness.json |
| E14 | workload | test | dla 0.1.0 | artifact harness/H3/harness.json (78e3f05a506f) | harness/H3/run-S2.json |
| E15 | growth-series | test | dla 0.1.0 | artifact series/S2.json (56f7b3da4bb7) | trend/T2.json |
| E16 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (56f7b3da4bb7) | trend/T2.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S2.json (56f7b3da4bb7) | trend/T2-keys.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S4.json (67871e26d0e5) | alloc/A1.json |
