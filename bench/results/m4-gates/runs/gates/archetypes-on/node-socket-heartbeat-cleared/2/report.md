# DLA report

Run: `20261108-201159-1bc6`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 13 enumerated, 13 dispositioned; confirmed 0, strong 0, refuted 13, suspected 0, unresolved 0

Residual: not measured

Coverage: node 7040 iterations, 25 of 30 entry points exercised

## Findings

No findings to report.

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (non-leak) E15
  - Contradicting evidence: E15
  - Originally supported by: E6
- **F2** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (non-leak) E16
  - Contradicting evidence: E16
  - Originally supported by: E6
- **F3** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (non-leak) E13
  - Contradicting evidence: E13
  - Originally supported by: E6
- **F4** (UNCLASSIFIED): discriminator E17
  - Ruled out by: plateau+purge (non-leak) E17
  - Contradicting evidence: E17
  - Originally supported by: E6
- **F5** (UNCLASSIFIED): discriminator E18
  - Ruled out by: plateau+purge (non-leak) E18
  - Contradicting evidence: E18
  - Originally supported by: E6
- **F6** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (non-leak) E14
  - Contradicting evidence: E14
  - Originally supported by: E6
- **F7** (UNCLASSIFIED): discriminator E19
  - Ruled out by: plateau+purge (non-leak) E19
  - Contradicting evidence: E19
  - Originally supported by: E6
- **F8** (UNCLASSIFIED): discriminator E20
  - Ruled out by: plateau+purge (non-leak) E20
  - Contradicting evidence: E20
  - Originally supported by: E6
- **F9** (UNCLASSIFIED): discriminator E21
  - Ruled out by: plateau+purge (non-leak) E21
  - Contradicting evidence: E21
  - Originally supported by: E6
- **F10** (UNCLASSIFIED): discriminator E22
  - Ruled out by: plateau+purge (non-leak) E22
  - Contradicting evidence: E22
  - Originally supported by: E6
- **F11** (UNCLASSIFIED): discriminator E23
  - Ruled out by: plateau+purge (non-leak) E23
  - Contradicting evidence: E23
  - Originally supported by: E6
- **F12** (UNCLASSIFIED): discriminator E24
  - Ruled out by: plateau+purge (non-leak) E24
  - Contradicting evidence: E24
  - Originally supported by: E6
- **F13** (UNCLASSIFIED): discriminator E25
  - Ruled out by: plateau+purge (non-leak) E25
  - Contradicting evidence: E25
  - Originally supported by: E6

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
- E13: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E13: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E13: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E13: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E13: limitation: points 1 to 9 come from S1; the extra points from S4
- E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- E14: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E14: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E14: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E14: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E14: limitation: points 1 to 9 come from S1; the extra points from S6
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E15: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E15: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E15: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E15: limitation: points 1 to 9 come from S1; the extra points from S8
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E16: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E16: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E16: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E16: limitation: points 1 to 9 come from S1; the extra points from S10
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E17: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E17: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E17: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E17: limitation: points 1 to 9 come from S1; the extra points from S12
- E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E17: limitation: gate and plateau thresholds are provisional (not calibrated)
- E18: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E18: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E18: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E18: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E18: limitation: points 1 to 9 come from S1; the extra points from S14
- E18: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E18: limitation: gate and plateau thresholds are provisional (not calibrated)
- E19: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E19: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E19: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E19: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E19: limitation: points 1 to 9 come from S1; the extra points from S16
- E19: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E19: limitation: gate and plateau thresholds are provisional (not calibrated)
- E20: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E20: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E20: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E20: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E20: limitation: points 1 to 9 come from S1; the extra points from S18
- E20: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E20: limitation: gate and plateau thresholds are provisional (not calibrated)
- E21: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E21: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E21: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E21: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E21: limitation: points 1 to 9 come from S1; the extra points from S20
- E21: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E21: limitation: gate and plateau thresholds are provisional (not calibrated)
- E22: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E22: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E22: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E22: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E22: limitation: points 1 to 9 come from S1; the extra points from S22
- E22: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E22: limitation: gate and plateau thresholds are provisional (not calibrated)
- E23: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E23: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E23: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E23: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E23: limitation: points 1 to 9 come from S1; the extra points from S24
- E23: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E23: limitation: gate and plateau thresholds are provisional (not calibrated)
- E24: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E24: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E24: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E24: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E24: limitation: points 1 to 9 come from S1; the extra points from S26
- E24: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E24: limitation: gate and plateau thresholds are provisional (not calibrated)
- E25: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E25: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E25: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E25: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E25: limitation: points 1 to 9 come from S1; the extra points from S28
- E25: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E25: limitation: gate and plateau thresholds are provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | — |
| E2 | run-config | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | probe/probe.json |
| E4 | static-hit | static | eslint 10.14.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | sweep/node-eslint.json |
| E5 | static-hit | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | sweep/node-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | census/node.json |
| E7 | workload | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (6800cc528ceb) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (d7b2731f8ef1) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (d7b2731f8ef1) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (afa519d3e78c) | alloc/A1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S5.json (554c2e0e136b) | alloc/A2.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S7.json (dc02487fa064) | alloc/A3.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S9.json (adc82987fe38) | alloc/A4.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S11.json (9077e0e07b42) | alloc/A5.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S13.json (8b1ac8543733) | alloc/A6.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S15.json (ed2023e0c1ad) | alloc/A7.json |
| E19 | discriminator | test | dla 0.1.0 | artifact series/S17.json (75c541e9719c) | alloc/A8.json |
| E20 | discriminator | test | dla 0.1.0 | artifact series/S19.json (e530da7be300) | alloc/A9.json |
| E21 | discriminator | test | dla 0.1.0 | artifact series/S21.json (32891a86f6c5) | alloc/A10.json |
| E22 | discriminator | test | dla 0.1.0 | artifact series/S23.json (399d2bd95e72) | alloc/A11.json |
| E23 | discriminator | test | dla 0.1.0 | artifact series/S25.json (3ce9f815b63d) | alloc/A12.json |
| E24 | discriminator | test | dla 0.1.0 | artifact series/S27.json (03eca9f955a1) | alloc/A13.json |
| E25 | discriminator | test | dla 0.1.0 | artifact series/S29.json (1e452eac45df) | alloc/A14.json |
