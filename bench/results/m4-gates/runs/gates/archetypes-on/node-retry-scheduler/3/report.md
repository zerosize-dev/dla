# DLA report

Run: `20261108-202507-3ab4`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 9 enumerated, 9 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 7

Residual: not measured

Coverage: node 3520 iterations, 17 of 19 entry points exercised

## Findings

### F5: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:src/scheduler.js:shared.latest` 25.97 count/n (steady, gate pass, E11)
- Evidence classes: context, growth, static
- Holder: container-field
- Owner: `src.scheduler.RetryScheduler.latest`
- Region: heap
- Shape: steady
- Site: `src/scheduler.js:15`
- Writers: `src/scheduler.js:32`
- Annotation: heuristic name match over the project's sources
- Annotation: enqueue() links every attempt record to the previous one via record.previous and advances this.latest; nothing ever clears previous, so after the schedule empties the full chain of attempt records (each holding its job and payload rows) stays reachable from the process-lifetime shared scheduler. recent() only needs the last N. Repair class: bound the trail or drop previous links when a record is unlinked. Unlabelled (UNCLASSIFIED): no taxonomy label commands available.
- Sibling hypotheses: F5.n1
- Supporting evidence: E6, E9, E11

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `src/scheduler.js:32`
- Reason: not_exercised
- Remedy: Detector hit duplicates confirmed F5 (same site src/scheduler.js:32).
- Supporting evidence: E5

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.pending.PendingList.head`
- Region: heap
- Site: `src/pending.js:7`
- Writers: `src/pending.js:15`, `src/pending.js:24`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Container drains/bounded (pending emptied, inflight 0, deadLetters capped at 100); growth column tied to F5's chain, not this holder. Re-check if growth persists after fixing F5.
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.pending.PendingList.tail`
- Region: heap
- Site: `src/pending.js:8`
- Writers: `src/pending.js:16`, `src/pending.js:25`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Container drains/bounded (pending emptied, inflight 0, deadLetters capped at 100); growth column tied to F5's chain, not this holder. Re-check if growth persists after fixing F5.
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.scheduler.RetryScheduler.pending`
- Region: heap
- Site: `src/scheduler.js:14`
- Writers: `src/scheduler.js:33`
- Removers: `src/scheduler.js:53`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Container drains/bounded (pending emptied, inflight 0, deadLetters capped at 100); growth column tied to F5's chain, not this holder. Re-check if growth persists after fixing F5.
- Supporting evidence: E6

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.scheduler.RetryScheduler.deadLetters`
- Region: heap
- Site: `src/scheduler.js:16`
- Writers: `src/scheduler.js:72`
- Removers: `src/scheduler.js:73`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Container drains/bounded (pending emptied, inflight 0, deadLetters capped at 100); growth column tied to F5's chain, not this holder. Re-check if growth persists after fixing F5.
- Supporting evidence: E6

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.scheduler.RetryScheduler.inflight`
- Region: heap
- Site: `src/scheduler.js:17`
- Writers: `src/scheduler.js:22`
- Removers: `src/scheduler.js:66`, `src/scheduler.js:71`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Container drains/bounded (pending emptied, inflight 0, deadLetters capped at 100); growth column tied to F5's chain, not this holder. Re-check if growth persists after fixing F5.
- Supporting evidence: E6

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `src.scheduler.shared`
- Region: heap
- Site: `src/scheduler.js:90`
- Writers: `src/scheduler.js:93`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Container drains/bounded (pending emptied, inflight 0, deadLetters capped at 100); growth column tied to F5's chain, not this holder. Re-check if growth persists after fixing F5.
- Supporting evidence: E6

## Ruled out

- **F5.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM: growth is a warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E12
  - Originally supported by: E11

## Blind spots and unknowns

### Coverage gaps

None recorded.

### Runtime blind spots

None recorded.

### Entry points not exercisable

- node: `src/pending.js::PendingList.[Symbol.iterator]`: no_harness

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
| E1 | run-config | static | dla 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | — |
| E2 | run-config | static | dla 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | probe/probe.json |
| E4 | static-hit | static | eslint 10.14.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | sweep/node-eslint.json |
| E5 | static-hit | static | dla 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | sweep/node-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | census/node.json |
| E7 | workload | static | dla 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (6b4063ddbee5) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (204f81c3923a) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (204f81c3923a) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (ad6357e7d820) | alloc/A1.json |
