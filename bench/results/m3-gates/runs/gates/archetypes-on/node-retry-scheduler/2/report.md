# DLA report

Run: `20261108-125316-41ff`

Status: COMPLETE

Findings: 14 enumerated, 14 dispositioned; confirmed 1, strong 0, refuted 4, suspected 0, unresolved 9

Residual: not measured

Coverage: node 3520 iterations, 17 of 19 entry points exercised

## Findings

### F5: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:src/scheduler.js:shared.latest` 25.97 count/n (steady, gate pass, E12)
- Evidence classes: context, growth, static
- Holder: container-field
- Owner: `src.scheduler.RetryScheduler.latest`
- Region: heap
- Shape: steady
- Site: `src/scheduler.js:15`
- Writers: `src/scheduler.js:32`, `src/scheduler.js:31`
- Annotation: heuristic name match over the project's sources
- Annotation: no remover: previous links never cleared on unlink; recent() only reads 10
- Sibling hypotheses: F5.n1
- Supporting evidence: E7, E10, E12

### F1: **UNCLASSIFIED**

- State: unresolved(no_harness)
- Evidence classes: static
- Site: `src/scheduler.js:32`
- Writers: `src/scheduler.js:32`
- Annotation: latest=record; record.previous chain never truncated; each record pins job+payload
- Reason: no_harness
- Remedy: Detector duplicate of F5 (same site scheduler.js:32); covered by confirmed F5.
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: unresolved(no_harness)
- Evidence classes: context, static
- Holder: container-field
- Owner: `src.pending.PendingList.head`
- Region: heap
- Site: `src/pending.js:7`
- Writers: `src/pending.js:15`, `src/pending.js:24`
- Removers: `src/pending.js:24`
- Annotation: heuristic name match over the project's sources
- Annotation: unlink handles head; fine
- Sibling hypotheses: F2.n1
- Reason: no_harness
- Remedy: Drained by tick (pending.length 0, source reviewed); whole-heap discriminator favors leak only because of F5. Add a probe on PendingList to prove it does not grow.
- Supporting evidence: E7, E10

### F2.n1: Hypothesis N-INTENT bounded/drained container (UNCLASSIFIED)

- State: unresolved(no_harness)
- Evidence classes: growth
- Sibling hypothesis of: F2
- Hypothesis: N-INTENT bounded/drained container
- Refuted if: container size grows with iterations
- Reason: no_harness
- Remedy: Sibling of F2; parent already dispositioned, no separate test needed.
- Supporting evidence: E12

### F3: **UNCLASSIFIED**

- State: unresolved(no_harness)
- Evidence classes: context, static
- Holder: container-field
- Owner: `src.pending.PendingList.tail`
- Region: heap
- Site: `src/pending.js:8`
- Writers: `src/pending.js:16`, `src/pending.js:25`
- Removers: `src/pending.js:25`
- Annotation: heuristic name match over the project's sources
- Annotation: unlink handles tail; fine
- Sibling hypotheses: F3.n1
- Reason: no_harness
- Remedy: Drained by tick (pending.length 0, source reviewed); whole-heap discriminator favors leak only because of F5. Add a probe on PendingList to prove it does not grow.
- Supporting evidence: E7, E10

### F3.n1: Hypothesis N-INTENT bounded/drained container (UNCLASSIFIED)

- State: unresolved(no_harness)
- Evidence classes: growth
- Sibling hypothesis of: F3
- Hypothesis: N-INTENT bounded/drained container
- Refuted if: container size grows with iterations
- Reason: no_harness
- Remedy: Sibling of F3; parent already dispositioned, no separate test needed.
- Supporting evidence: E12

### F4.n1: Hypothesis N-INTENT bounded/drained container (UNCLASSIFIED)

- State: unresolved(no_harness)
- Evidence classes: growth
- Sibling hypothesis of: F4
- Hypothesis: N-INTENT bounded/drained container
- Refuted if: container size grows with iterations
- Reason: no_harness
- Remedy: Sibling of F4; parent already dispositioned, no separate test needed.
- Supporting evidence: E12

### F6.n1: Hypothesis N-INTENT bounded/drained container (UNCLASSIFIED)

- State: unresolved(no_harness)
- Evidence classes: growth
- Sibling hypothesis of: F6
- Hypothesis: N-INTENT bounded/drained container
- Refuted if: container size grows with iterations
- Reason: no_harness
- Remedy: Sibling of F6; parent already dispositioned, no separate test needed.
- Supporting evidence: E12

### F7.n1: Hypothesis N-INTENT bounded/drained container (UNCLASSIFIED)

- State: unresolved(no_harness)
- Evidence classes: growth
- Sibling hypothesis of: F7
- Hypothesis: N-INTENT bounded/drained container
- Refuted if: container size grows with iterations
- Reason: no_harness
- Remedy: Sibling of F7; parent already dispositioned, no separate test needed.
- Supporting evidence: E12

### F8: **UNCLASSIFIED**

- State: unresolved(no_harness)
- Evidence classes: static
- Holder: static/global
- Owner: `src.scheduler.shared`
- Region: heap
- Site: `src/scheduler.js:90`
- Writers: `src/scheduler.js:93`
- Annotation: heuristic name match over the project's sources
- Annotation: singleton by design; holds latest chain across runs
- Reason: no_harness
- Remedy: Singleton root that keeps the F5 chain reachable for process life; not a separate leak. Fix is on F5.
- Supporting evidence: E7

## Ruled out

- **F4** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (non-leak) E16
  - Contradicting evidence: E16
  - Originally supported by: E7, E10
- **F5.n1** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (leak) E13
  - Hypothesis: N-WARM growth is warm-up/deferred GC and plateaus
  - Refuted if: heap_used keeps growing after forced GC over extra cycles
  - Contradicting evidence: E13
  - Originally supported by: E12
- **F6** (UNCLASSIFIED): discriminator E17
  - Ruled out by: plateau+purge (non-leak) E17
  - Contradicting evidence: E17
  - Originally supported by: E7, E10
- **F7** (UNCLASSIFIED): discriminator E18
  - Ruled out by: plateau+purge (non-leak) E18
  - Contradicting evidence: E18
  - Originally supported by: E7, E10

## Blind spots and unknowns

### Coverage gaps

None recorded.

### Runtime blind spots

None recorded.

### Entry points not exercisable

- node: `src/pending.js::PendingList.[Symbol.iterator]`: no_harness

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

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | webhook dispatcher RetryScheduler (node) |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | src/*.js, load.js |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | heap grows per batch though schedule empty, inflight 0 |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | node |
| `intake.build` | Does the project build locally? | answered | E1 | no build, plain JS |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | npm run load / load.js --iterations N |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 raw |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 3600s |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | — |
| E2 | run-config | static | dla 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | probe/probe.json |
| E4 | probe | static | dla-probe 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | probe/probe.json |
| E5 | static-hit | static | eslint 10.14.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | sweep/eslint.json |
| E6 | static-hit | static | dla 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | sweep/dla.json |
| E7 | census-holder | static | dla-census 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | census/node.json |
| E8 | workload | static | dla 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | workload/workloads.json |
| E9 | workload | static | dla 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | harness/H1/harness.json |
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (9a0f567f906c) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (57f354dedd8a) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (57f354dedd8a) | trend/T1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S3.json (a8c0973f81aa) | alloc/A1.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S5.json (364fdc14216a) | alloc/A2.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S7.json (32c73e3ebad4) | alloc/A3.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S9.json (e0a59a0f47db) | alloc/A4.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S11.json (5477857e1e3e) | alloc/A5.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S13.json (e682b487dc70) | alloc/A6.json |
| E19 | discriminator | test | dla 0.1.0 | artifact series/S15.json (8e3f7f43304a) | alloc/A7.json |
