# DLA report

Run: `20261108-131945-b100`

Status: COMPLETE

Findings: 28 enumerated, 28 dispositioned; confirmed 3, strong 0, refuted 4, suspected 0, unresolved 21

Residual: not measured

Coverage: node 3520 iterations, 25 of 30 entry points exercised

## Findings

### F1: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `node.timers@src/heartbeat.js:15` 1 count/n (steady, gate pass, E12)
- Evidence classes: context, growth, static
- Holder: blocked-stack
- Region: non-heap
- Shape: steady
- Site: `src/heartbeat.js:15`
- Writers: `src/gateway.js:26`
- Annotation: setInterval created per session in accept; returned timer stored in session.heartbeat; no clearInterval anywhere
- Annotation: timer registered in Node timer list holds closure over session (socket, recent[64], topics)
- Sibling hypotheses: F1.n1
- Suggested archetype (unconfirmed): L-WORKER
- Supporting evidence: E6, E10, E12

### F5: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `node.listeners@src/gateway.js:24` 1 count/n (steady, gate pass, E12)
- Evidence classes: context, growth, static
- Holder: registration-table
- Owner: `src.gateway.Gateway.accept`
- Region: heap
- Shape: steady
- Site: `src/gateway.js:24`
- Writers: `src/gateway.js:25`, `src/session.js:12`, `src/session.js:54`
- Removers: `load.js:50`, `src/gateway.js:31`, `src/session.js:66`
- Annotation: heuristic name match over the project's sources
- Annotation: consequence of F1: socket.on listeners stay because the timer keeps session+socket alive
- Sibling hypotheses: F5.n1
- Supporting evidence: E7, E10, E12

### F7: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `node.timers@src/heartbeat.js:15` 1 count/n (steady, gate pass, E12)
- Evidence classes: context, growth, static
- Holder: blocked-stack
- Owner: `src.heartbeat.startHeartbeat`
- Region: non-heap
- Shape: steady
- Site: `src/heartbeat.js:15`
- Writers: `src/heartbeat.js:15`
- Annotation: no clearInterval in session.close or Gateway.release
- Sibling hypotheses: F7.n1
- Supporting evidence: E7, E10, E12

### F2: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: context, static
- Holder: container-field
- Owner: `src.gateway.Gateway.metrics`
- Region: heap
- Site: `src/gateway.js:13`
- Sibling hypotheses: F2.n1
- Reason: env_unavailable
- Remedy: No growth column of its own; only whole-process RSS grew, which F1 explains (session retained by timer). Re-run harness after adding clearInterval on session close and see whether this holder still grows.
- Supporting evidence: E7, E10

### F2.n1: Hypothesis N-WARM: bounded/plateaus (UNCLASSIFIED)

- State: unresolved(time_budget)
- Evidence classes: growth
- Sibling hypothesis of: F2
- Hypothesis: N-WARM: bounded/plateaus
- Refuted if: growth continues over extra cycles
- Reason: time_budget
- Remedy: Sibling of a finding with no own growth column; untested beyond alloc-test.
- Supporting evidence: E12

### F3: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: context, static
- Holder: container-field
- Owner: `src.gateway.Gateway.router`
- Region: heap
- Site: `src/gateway.js:14`
- Sibling hypotheses: F3.n1
- Reason: env_unavailable
- Remedy: No growth column of its own; only whole-process RSS grew, which F1 explains (session retained by timer). Re-run harness after adding clearInterval on session close and see whether this holder still grows.
- Supporting evidence: E7, E10

### F3.n1: Hypothesis N-WARM: bounded/plateaus (UNCLASSIFIED)

- State: unresolved(time_budget)
- Evidence classes: growth
- Sibling hypothesis of: F3
- Hypothesis: N-WARM: bounded/plateaus
- Refuted if: growth continues over extra cycles
- Reason: time_budget
- Remedy: Sibling of a finding with no own growth column; untested beyond alloc-test.
- Supporting evidence: E12

### F4: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: context, static
- Holder: container-field
- Owner: `src.gateway.Gateway.sessions`
- Region: heap
- Site: `src/gateway.js:15`
- Writers: `src/gateway.js:22`
- Removers: `src/gateway.js:32`
- Annotation: heuristic name match over the project's sources
- Sibling hypotheses: F4.n1
- Reason: env_unavailable
- Remedy: No growth column of its own; only whole-process RSS grew, which F1 explains (session retained by timer). Re-run harness after adding clearInterval on session close and see whether this holder still grows.
- Supporting evidence: E7, E10

### F4.n1: Hypothesis N-WARM: bounded/plateaus (UNCLASSIFIED)

- State: unresolved(time_budget)
- Evidence classes: growth
- Sibling hypothesis of: F4
- Hypothesis: N-WARM: bounded/plateaus
- Refuted if: growth continues over extra cycles
- Reason: time_budget
- Remedy: Sibling of a finding with no own growth column; untested beyond alloc-test.
- Supporting evidence: E12

### F6: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: context, static
- Holder: registration-table
- Owner: `src.gateway.Gateway.accept`
- Region: heap
- Site: `src/gateway.js:25`
- Writers: `src/gateway.js:24`, `src/session.js:12`, `src/session.js:54`
- Removers: `load.js:50`, `src/gateway.js:31`, `src/session.js:66`
- Annotation: heuristic name match over the project's sources
- Sibling hypotheses: F6.n1
- Reason: env_unavailable
- Remedy: No growth column of its own; only whole-process RSS grew, which F1 explains (session retained by timer). Re-run harness after adding clearInterval on session close and see whether this holder still grows.
- Supporting evidence: E7, E10

### F6.n1: Hypothesis N-WARM: bounded/plateaus (UNCLASSIFIED)

- State: unresolved(time_budget)
- Evidence classes: growth
- Sibling hypothesis of: F6
- Hypothesis: N-WARM: bounded/plateaus
- Refuted if: growth continues over extra cycles
- Reason: time_budget
- Remedy: Sibling of a finding with no own growth column; untested beyond alloc-test.
- Supporting evidence: E12

### F8: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: context, static
- Holder: container-field
- Owner: `src.loopback.LoopbackSocket.written`
- Region: heap
- Site: `src/loopback.js:11`
- Writers: `src/loopback.js:17`
- Annotation: heuristic name match over the project's sources
- Sibling hypotheses: F8.n1
- Reason: env_unavailable
- Remedy: No growth column of its own; only whole-process RSS grew, which F1 explains (session retained by timer). Re-run harness after adding clearInterval on session close and see whether this holder still grows.
- Supporting evidence: E7, E10

### F8.n1: Hypothesis N-WARM: bounded/plateaus (UNCLASSIFIED)

- State: unresolved(time_budget)
- Evidence classes: growth
- Sibling hypothesis of: F8
- Hypothesis: N-WARM: bounded/plateaus
- Refuted if: growth continues over extra cycles
- Reason: time_budget
- Remedy: Sibling of a finding with no own growth column; untested beyond alloc-test.
- Supporting evidence: E12

### F9.n1: Hypothesis N-WARM: bounded/plateaus (UNCLASSIFIED)

- State: unresolved(time_budget)
- Evidence classes: growth
- Sibling hypothesis of: F9
- Hypothesis: N-WARM: bounded/plateaus
- Refuted if: growth continues over extra cycles
- Reason: time_budget
- Remedy: Sibling of a finding with no own growth column; untested beyond alloc-test.
- Supporting evidence: E12

### F10: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: context, static
- Holder: container-field
- Owner: `src.router.Router.topics`
- Region: heap
- Site: `src/router.js:9`
- Writers: `src/router.js:16`, `src/router.js:19`
- Removers: `src/router.js:26`, `src/router.js:27`
- Annotation: heuristic name match over the project's sources
- Sibling hypotheses: F10.n1
- Reason: env_unavailable
- Remedy: No growth column of its own; only whole-process RSS grew, which F1 explains (session retained by timer). Re-run harness after adding clearInterval on session close and see whether this holder still grows.
- Supporting evidence: E7, E10

### F10.n1: Hypothesis N-WARM: bounded/plateaus (UNCLASSIFIED)

- State: unresolved(time_budget)
- Evidence classes: growth
- Sibling hypothesis of: F10
- Hypothesis: N-WARM: bounded/plateaus
- Refuted if: growth continues over extra cycles
- Reason: time_budget
- Remedy: Sibling of a finding with no own growth column; untested beyond alloc-test.
- Supporting evidence: E12

### F11: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: context, static
- Holder: registration-table
- Owner: `src.router.Router.dispatch`
- Region: heap
- Site: `src/router.js:51`
- Writers: `src/session.js:38`
- Removers: `src/gateway.js:25`, `src/router.js:54`, `src/session.js:33`
- Annotation: heuristic name match over the project's sources
- Sibling hypotheses: F11.n1
- Reason: env_unavailable
- Remedy: No growth column of its own; only whole-process RSS grew, which F1 explains (session retained by timer). Re-run harness after adding clearInterval on session close and see whether this holder still grows.
- Supporting evidence: E7, E10

### F11.n1: Hypothesis N-WARM: bounded/plateaus (UNCLASSIFIED)

- State: unresolved(time_budget)
- Evidence classes: growth
- Sibling hypothesis of: F11
- Hypothesis: N-WARM: bounded/plateaus
- Refuted if: growth continues over extra cycles
- Reason: time_budget
- Remedy: Sibling of a finding with no own growth column; untested beyond alloc-test.
- Supporting evidence: E12

### F12: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: context, static
- Holder: container-field
- Owner: `src.session.Session.decoder`
- Region: heap
- Site: `src/session.js:15`
- Writers: `src/session.js:30`
- Annotation: heuristic name match over the project's sources
- Sibling hypotheses: F12.n1
- Reason: env_unavailable
- Remedy: No growth column of its own; only whole-process RSS grew, which F1 explains (session retained by timer). Re-run harness after adding clearInterval on session close and see whether this holder still grows.
- Supporting evidence: E7, E10

### F12.n1: Hypothesis N-WARM: bounded/plateaus (UNCLASSIFIED)

- State: unresolved(time_budget)
- Evidence classes: growth
- Sibling hypothesis of: F12
- Hypothesis: N-WARM: bounded/plateaus
- Refuted if: growth continues over extra cycles
- Reason: time_budget
- Remedy: Sibling of a finding with no own growth column; untested beyond alloc-test.
- Supporting evidence: E12

### F13: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: context, static
- Holder: container-field
- Owner: `src.session.Session.topics`
- Region: heap
- Site: `src/session.js:19`
- Sibling hypotheses: F13.n1
- Reason: env_unavailable
- Remedy: No growth column of its own; only whole-process RSS grew, which F1 explains (session retained by timer). Re-run harness after adding clearInterval on session close and see whether this holder still grows.
- Supporting evidence: E7, E10

### F13.n1: Hypothesis N-WARM: bounded/plateaus (UNCLASSIFIED)

- State: unresolved(time_budget)
- Evidence classes: growth
- Sibling hypothesis of: F13
- Hypothesis: N-WARM: bounded/plateaus
- Refuted if: growth continues over extra cycles
- Reason: time_budget
- Remedy: Sibling of a finding with no own growth column; untested beyond alloc-test.
- Supporting evidence: E12

### F14: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: context, static
- Holder: container-field
- Owner: `src.session.Session.recent`
- Region: heap
- Site: `src/session.js:20`
- Writers: `src/session.js:44`
- Removers: `src/session.js:45`
- Annotation: heuristic name match over the project's sources
- Sibling hypotheses: F14.n1
- Reason: env_unavailable
- Remedy: No growth column of its own; only whole-process RSS grew, which F1 explains (session retained by timer). Re-run harness after adding clearInterval on session close and see whether this holder still grows.
- Supporting evidence: E7, E10

### F14.n1: Hypothesis N-WARM: bounded/plateaus (UNCLASSIFIED)

- State: unresolved(time_budget)
- Evidence classes: growth
- Sibling hypothesis of: F14
- Hypothesis: N-WARM: bounded/plateaus
- Refuted if: growth continues over extra cycles
- Reason: time_budget
- Remedy: Sibling of a finding with no own growth column; untested beyond alloc-test.
- Supporting evidence: E12

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (leak) E13
  - Hypothesis: N-WARM: timers count plateaus after warm-up
  - Refuted if: timer count keeps growing over extra cycles and after purge
  - Contradicting evidence: E13
  - Originally supported by: E12
- **F5.n1** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (leak) E15
  - Hypothesis: N-WARM: plateaus after warm-up
  - Refuted if: growth continues over extra cycles
  - Contradicting evidence: E15
  - Originally supported by: E12
- **F7.n1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (leak) E14
  - Hypothesis: N-WARM: plateaus after warm-up
  - Refuted if: growth continues over extra cycles
  - Contradicting evidence: E14
  - Originally supported by: E12
- **F9** (UNCLASSIFIED): discriminator E21
  - Ruled out by: plateau+purge (non-leak) E21
  - Contradicting evidence: E21
  - Originally supported by: E7, E10

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
- E15: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E15: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E15: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E15: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E15: limitation: points 1 to 9 come from S1; the extra points from S6
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E16: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E16: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E16: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E16: limitation: points 1 to 9 come from S1; the extra points from S8
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E17: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E17: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E17: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E17: limitation: points 1 to 9 come from S1; the extra points from S10
- E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E17: limitation: gate and plateau thresholds are provisional (not calibrated)
- E18: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E18: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E18: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E18: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E18: limitation: points 1 to 9 come from S1; the extra points from S12
- E18: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E18: limitation: gate and plateau thresholds are provisional (not calibrated)
- E19: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E19: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E19: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E19: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E19: limitation: points 1 to 9 come from S1; the extra points from S14
- E19: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E19: limitation: gate and plateau thresholds are provisional (not calibrated)
- E20: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E20: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E20: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E20: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E20: limitation: points 1 to 9 come from S1; the extra points from S16
- E20: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E20: limitation: gate and plateau thresholds are provisional (not calibrated)
- E21: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E21: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E21: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E21: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E21: limitation: points 1 to 9 come from S1; the extra points from S18
- E21: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E21: limitation: gate and plateau thresholds are provisional (not calibrated)
- E22: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E22: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E22: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E22: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E22: limitation: points 1 to 9 come from S1; the extra points from S20
- E22: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E22: limitation: gate and plateau thresholds are provisional (not calibrated)
- E23: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E23: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E23: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E23: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E23: limitation: points 1 to 9 come from S1; the extra points from S22
- E23: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E23: limitation: gate and plateau thresholds are provisional (not calibrated)
- E24: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E24: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E24: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E24: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E24: limitation: points 1 to 9 come from S1; the extra points from S24
- E24: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E24: limitation: gate and plateau thresholds are provisional (not calibrated)
- E25: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E25: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E25: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E25: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E25: limitation: points 1 to 9 come from S1; the extra points from S26
- E25: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E25: limitation: gate and plateau thresholds are provisional (not calibrated)
- E26: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E26: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E26: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E26: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E26: limitation: points 1 to 9 come from S1; the extra points from S28
- E26: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E26: limitation: gate and plateau thresholds are provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | repository: edge-gateway (node) |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | whole repo, src/ and load.js |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | RSS grows per connect/disconnect wave, no open sessions, restart ~2 days |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E3 | node only |
| `intake.build` | Does the project build locally? | answered | E4 | no build step (plain JS) |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | load.js --iterations N |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4, raw values |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 60m |

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
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (735390fc3da6) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (59fb4b91e5c0) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (59fb4b91e5c0) | trend/T1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S3.json (0dcf5c33c960) | alloc/A1.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S5.json (414d02babf51) | alloc/A2.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S7.json (e98be037b897) | alloc/A3.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S9.json (c02795d2f849) | alloc/A4.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S11.json (518f860db637) | alloc/A5.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S13.json (50e79b980c39) | alloc/A6.json |
| E19 | discriminator | test | dla 0.1.0 | artifact series/S15.json (0ab64ff4f424) | alloc/A7.json |
| E20 | discriminator | test | dla 0.1.0 | artifact series/S17.json (ed762bed8b98) | alloc/A8.json |
| E21 | discriminator | test | dla 0.1.0 | artifact series/S19.json (9a145ef116c0) | alloc/A9.json |
| E22 | discriminator | test | dla 0.1.0 | artifact series/S21.json (61a9b4822d71) | alloc/A10.json |
| E23 | discriminator | test | dla 0.1.0 | artifact series/S23.json (88be5d0d3be0) | alloc/A11.json |
| E24 | discriminator | test | dla 0.1.0 | artifact series/S25.json (7372259fffbe) | alloc/A12.json |
| E25 | discriminator | test | dla 0.1.0 | artifact series/S27.json (ed6d3e943ab6) | alloc/A13.json |
| E26 | discriminator | test | dla 0.1.0 | artifact series/S29.json (9d4da51e1ff4) | alloc/A14.json |
