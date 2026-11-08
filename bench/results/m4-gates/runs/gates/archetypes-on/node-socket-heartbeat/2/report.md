# DLA report

Run: `20261108-200852-8cce`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 15 enumerated, 15 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 13

Residual: not measured

Coverage: node 3520 iterations, 25 of 30 entry points exercised

## Findings

### F1: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `node.timers@src/heartbeat.js:15` 1 count/n (steady, gate pass, E11)
- Evidence classes: context, growth, static
- Holder: blocked-stack
- Region: non-heap
- Shape: steady
- Site: `src/heartbeat.js:15`
- Writers: `src/gateway.js:23`
- Annotation: startHeartbeat's setInterval handle is stored on session.heartbeat but never clearInterval'd; Session.close() and Gateway.release() do not stop it. The (unref'd) timer is a libuv-rooted timer whose closure captures the session, so every closed session (socket, decoder, recent[64] messages, topics) stays reachable and its callback keeps firing every period, forever. Missing pairing: clearInterval(session.heartbeat) in close(). Consequence: socket 'data' listener count (gateway.js:24) also grows because sockets are retained.
- Sibling hypotheses: F1.n1
- Suggested archetype (unconfirmed): L-WORKER
- Supporting evidence: E5, E9, E11

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.metrics`
- Region: heap
- Site: `src/gateway.js:13`
- Reason: not_exercised
- Remedy: Census/detector lead with no growing column of its own in harness S1; alloc-test per candidate not run within this pass. Re-run dla alloc-test H1 --finding F2 if growth remains after fixing F1.
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.router`
- Region: heap
- Site: `src/gateway.js:14`
- Reason: not_exercised
- Remedy: Census/detector lead with no growing column of its own in harness S1; alloc-test per candidate not run within this pass. Re-run dla alloc-test H1 --finding F3 if growth remains after fixing F1.
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
- Remedy: Census/detector lead with no growing column of its own in harness S1; alloc-test per candidate not run within this pass. Re-run dla alloc-test H1 --finding F4 if growth remains after fixing F1.
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
- Remedy: Listener/timer column grows but is explained by F1 (retained sessions/sockets via the heartbeat timer); re-measure after clearInterval is added to Session.close to see whether anything remains.
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
- Remedy: Listener/timer column grows but is explained by F1 (retained sessions/sockets via the heartbeat timer); re-measure after clearInterval is added to Session.close to see whether anything remains.
- Supporting evidence: E6

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `src.heartbeat.startHeartbeat`
- Region: non-heap
- Site: `src/heartbeat.js:15`
- Reason: not_exercised
- Remedy: Listener/timer column grows but is explained by F1 (retained sessions/sockets via the heartbeat timer); re-measure after clearInterval is added to Session.close to see whether anything remains.
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
- Remedy: Census/detector lead with no growing column of its own in harness S1; alloc-test per candidate not run within this pass. Re-run dla alloc-test H1 --finding F8 if growth remains after fixing F1.
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
- Remedy: Census/detector lead with no growing column of its own in harness S1; alloc-test per candidate not run within this pass. Re-run dla alloc-test H1 --finding F9 if growth remains after fixing F1.
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
- Remedy: Census/detector lead with no growing column of its own in harness S1; alloc-test per candidate not run within this pass. Re-run dla alloc-test H1 --finding F10 if growth remains after fixing F1.
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
- Remedy: Census/detector lead with no growing column of its own in harness S1; alloc-test per candidate not run within this pass. Re-run dla alloc-test H1 --finding F11 if growth remains after fixing F1.
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
- Remedy: Census/detector lead with no growing column of its own in harness S1; alloc-test per candidate not run within this pass. Re-run dla alloc-test H1 --finding F12 if growth remains after fixing F1.
- Supporting evidence: E6

### F13: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.topics`
- Region: heap
- Site: `src/session.js:19`
- Reason: not_exercised
- Remedy: Census/detector lead with no growing column of its own in harness S1; alloc-test per candidate not run within this pass. Re-run dla alloc-test H1 --finding F13 if growth remains after fixing F1.
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
- Remedy: Census/detector lead with no growing column of its own in harness S1; alloc-test per candidate not run within this pass. Re-run dla alloc-test H1 --finding F14 if growth remains after fixing F1.
- Supporting evidence: E6

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM: timers count plateaus once the 32-session window fills
  - Refuted if: timer count stops growing over extra cycles / after purge
  - Contradicting evidence: E12
  - Originally supported by: E11

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
- E12: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E12: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E12: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E12: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E12: limitation: points 1 to 9 come from S1; the extra points from S2
- E12: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E12: limitation: gate and plateau thresholds are provisional (not calibrated)

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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (d0f92a0eae24) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (00b93f995c74) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (00b93f995c74) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (ad6c0869117d) | alloc/A1.json |
