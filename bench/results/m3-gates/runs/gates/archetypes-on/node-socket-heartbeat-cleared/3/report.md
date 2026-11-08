# DLA report

Run: `20261108-132507-d898`

Status: COMPLETE

Findings: 17 enumerated, 17 dispositioned; confirmed 1, strong 0, refuted 14, suspected 0, unresolved 2

Residual: not measured

Coverage: node 3520 iterations, 25 of 30 entry points exercised

## Findings

### F14.n1: Hypothesis N-HEADROOM heap_used reflects deferred GC (UNCLASSIFIED)

- State: confirmed
- Explained by N-HEADROOM heap_used reflects deferred GC (discriminator E15)
- Evidence classes: context, discriminator, growth
- Sibling hypothesis of: F14
- Hypothesis: N-HEADROOM heap_used reflects deferred GC
- Refuted if: heap_used still rises after forced GC at extra points
- Supporting evidence: E11, E10, E15

### F15: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: context, discriminator, growth
- Region: off-heap
- Shape: steady
- Hypothesis: node.external.pre_gc grows with connections
- Refuted if: post-GC external flat
- Sibling hypotheses: F15.n1
- Reason: env_unavailable
- Remedy: RSS/pre-GC external rise over 440-3000 iterations; scratch ablation (no heartbeat vs heartbeat, 1k->128k iterations) shows RSS plateaus ~122MB with flat post-GC heap/external/timers, i.e. allocator/GC headroom; run a >=100k-iteration harness or soak on the real net.Socket path to settle
- Supporting evidence: E11, E9, E19

### F15.n1: Hypothesis N-ALLOC external is Buffer pool/allocator slack (UNCLASSIFIED)

- State: unresolved(env_unavailable)
- Evidence classes: growth
- Sibling hypothesis of: F15
- Hypothesis: N-ALLOC external is Buffer pool/allocator slack
- Refuted if: external keeps rising after purge
- Reason: env_unavailable
- Remedy: non-leak (allocator/GC headroom) supported by ablation plateau ~122MB but no harness discriminator favors it; needs longer harness
- Supporting evidence: E11

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E22
  - Ruled out by: plateau+purge (non-leak) E22
  - Contradicting evidence: E22
  - Originally supported by: E7
- **F2** (UNCLASSIFIED): discriminator E23
  - Ruled out by: plateau+purge (non-leak) E23
  - Contradicting evidence: E23
  - Originally supported by: E7
- **F3** (UNCLASSIFIED): discriminator E24
  - Ruled out by: plateau+purge (non-leak) E24
  - Contradicting evidence: E24
  - Originally supported by: E7
- **F4** (UNCLASSIFIED): discriminator E25
  - Ruled out by: plateau+purge (non-leak) E25
  - Contradicting evidence: E25
  - Originally supported by: E7
- **F5** (UNCLASSIFIED): discriminator E26
  - Ruled out by: plateau+purge (non-leak) E26
  - Contradicting evidence: E26
  - Originally supported by: E7
- **F6** (UNCLASSIFIED): discriminator E21
  - Ruled out by: plateau+purge (non-leak) E21
  - Contradicting evidence: E21
  - Originally supported by: E7
- **F7** (UNCLASSIFIED): discriminator E27
  - Ruled out by: plateau+purge (non-leak) E27
  - Contradicting evidence: E27
  - Originally supported by: E7
- **F8** (UNCLASSIFIED): discriminator E28
  - Ruled out by: plateau+purge (non-leak) E28
  - Contradicting evidence: E28
  - Originally supported by: E7
- **F9** (UNCLASSIFIED): discriminator E29
  - Ruled out by: plateau+purge (non-leak) E29
  - Contradicting evidence: E29
  - Originally supported by: E7
- **F10** (UNCLASSIFIED): discriminator E30
  - Ruled out by: plateau+purge (non-leak) E30
  - Contradicting evidence: E30
  - Originally supported by: E7
- **F11** (UNCLASSIFIED): discriminator E31
  - Ruled out by: plateau+purge (non-leak) E31
  - Contradicting evidence: E31
  - Originally supported by: E7
- **F12** (UNCLASSIFIED): discriminator E32
  - Ruled out by: plateau+purge (non-leak) E32
  - Contradicting evidence: E32
  - Originally supported by: E7
- **F13** (UNCLASSIFIED): discriminator E33
  - Ruled out by: plateau+purge (non-leak) E33
  - Contradicting evidence: E33
  - Originally supported by: E7
- **F14** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (non-leak) E15
  - Hypothesis: heap_used grows with connections: some per-session object retained after close
  - Refuted if: alloc-test plateau/purge shows heap_used flat after GC
  - Contradicting evidence: E15
  - Originally supported by: E11, E9

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
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E14: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E15: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E15: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E15: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E15: limitation: points 1 to 9 come from S1; the extra points from S2
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E16: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E16: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E16: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E16: limitation: points 1 to 9 come from S1; the extra points from S4
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E17: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E18: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E18: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E18: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E18: limitation: purge 'trim' is not supported by the node agent (only gc)
- E18: limitation: points 1 to 9 come from S1; the extra points from S7
- E18: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E18: limitation: the purge event carries no before/after readings: the purge test was not run
- E18: limitation: gate and plateau thresholds are provisional (not calibrated)
- E19: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E19: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E19: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E19: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E19: limitation: points 1 to 9 come from S1; the extra points from S9
- E19: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E19: limitation: gate and plateau thresholds are provisional (not calibrated)
- E20: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E20: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E20: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E20: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E20: limitation: points 1 to 9 come from S1; the extra points from S11
- E20: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E20: limitation: gate and plateau thresholds are provisional (not calibrated)
- E21: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E21: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E21: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E21: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E21: limitation: points 1 to 9 come from S1; the extra points from S13
- E21: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E21: limitation: gate and plateau thresholds are provisional (not calibrated)
- E22: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E22: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E22: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E22: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E22: limitation: points 1 to 9 come from S1; the extra points from S15
- E22: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E22: limitation: gate and plateau thresholds are provisional (not calibrated)
- E23: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E23: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E23: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E23: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E23: limitation: points 1 to 9 come from S1; the extra points from S17
- E23: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E23: limitation: gate and plateau thresholds are provisional (not calibrated)
- E24: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E24: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E24: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E24: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E24: limitation: points 1 to 9 come from S1; the extra points from S19
- E24: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E24: limitation: gate and plateau thresholds are provisional (not calibrated)
- E25: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E25: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E25: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E25: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E25: limitation: points 1 to 9 come from S1; the extra points from S21
- E25: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E25: limitation: gate and plateau thresholds are provisional (not calibrated)
- E26: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E26: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E26: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E26: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E26: limitation: points 1 to 9 come from S1; the extra points from S23
- E26: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E26: limitation: gate and plateau thresholds are provisional (not calibrated)
- E27: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E27: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E27: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E27: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E27: limitation: points 1 to 9 come from S1; the extra points from S25
- E27: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E27: limitation: gate and plateau thresholds are provisional (not calibrated)
- E28: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E28: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E28: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E28: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E28: limitation: points 1 to 9 come from S1; the extra points from S27
- E28: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E28: limitation: gate and plateau thresholds are provisional (not calibrated)
- E29: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E29: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E29: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E29: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E29: limitation: points 1 to 9 come from S1; the extra points from S29
- E29: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E29: limitation: gate and plateau thresholds are provisional (not calibrated)
- E30: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E30: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E30: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E30: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E30: limitation: points 1 to 9 come from S1; the extra points from S31
- E30: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E30: limitation: gate and plateau thresholds are provisional (not calibrated)
- E31: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E31: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E31: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E31: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E31: limitation: points 1 to 9 come from S1; the extra points from S33
- E31: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E31: limitation: gate and plateau thresholds are provisional (not calibrated)
- E32: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E32: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E32: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E32: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E32: limitation: points 1 to 9 come from S1; the extra points from S35
- E32: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E32: limitation: gate and plateau thresholds are provisional (not calibrated)
- E33: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E33: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E33: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E33: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E33: limitation: points 1 to 9 come from S1; the extra points from S37
- E33: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E33: limitation: gate and plateau thresholds are provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | repo: edge-gateway Node (src/*.js), in-process load.js |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | whole repo src/ |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | RSS grows per wave of connect/disconnect, restart ~2 days |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | node |
| `intake.build` | Does the project build locally? | answered | E1 | no build; node >=22 |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | npm run load / load.js --iterations N |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 raw values |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 3600s |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | — |
| E2 | run-config | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | probe/probe.json |
| E4 | probe | static | dla-probe 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | probe/probe.json |
| E5 | static-hit | static | eslint 10.14.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | sweep/eslint.json |
| E6 | static-hit | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | sweep/dla.json |
| E7 | census-holder | static | dla-census 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | census/node.json |
| E8 | workload | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | workload/workloads.json |
| E9 | workload | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | harness/H1/harness.json |
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (f21d1a0ef58e) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (e1cb9fafc74b) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (e1cb9fafc74b) | trend/T1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S1.json (e1cb9fafc74b) | trend/T2.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (e1cb9fafc74b) | trend/T2.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S3.json (c562c77080bc) | alloc/A1.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S5.json (0af8cbbf671c) | alloc/A2.json |
| E17 | non-heap | test | dla 0.1.0 | artifact series/S6.json (9b3cbb37d2fc) | trend/T5.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S8.json (dd847f424fa9) | alloc/A3.json |
| E19 | discriminator | test | dla 0.1.0 | artifact series/S10.json (8ecfbbbc52ed) | alloc/A4.json |
| E20 | discriminator | test | dla 0.1.0 | artifact series/S12.json (190d94a05fbc) | alloc/A5.json |
| E21 | discriminator | test | dla 0.1.0 | artifact series/S14.json (496215194afc) | alloc/A6.json |
| E22 | discriminator | test | dla 0.1.0 | artifact series/S16.json (cc2b29123ae5) | alloc/A7.json |
| E23 | discriminator | test | dla 0.1.0 | artifact series/S18.json (c7c018a234dd) | alloc/A8.json |
| E24 | discriminator | test | dla 0.1.0 | artifact series/S20.json (2c3a33ce8668) | alloc/A9.json |
| E25 | discriminator | test | dla 0.1.0 | artifact series/S22.json (0bf4b7f119a3) | alloc/A10.json |
| E26 | discriminator | test | dla 0.1.0 | artifact series/S24.json (5d625568fad0) | alloc/A11.json |
| E27 | discriminator | test | dla 0.1.0 | artifact series/S26.json (4ee44d451dd5) | alloc/A12.json |
| E28 | discriminator | test | dla 0.1.0 | artifact series/S28.json (32741d9119a7) | alloc/A13.json |
| E29 | discriminator | test | dla 0.1.0 | artifact series/S30.json (005647c9fd84) | alloc/A14.json |
| E30 | discriminator | test | dla 0.1.0 | artifact series/S32.json (bb53ac7fe28b) | alloc/A15.json |
| E31 | discriminator | test | dla 0.1.0 | artifact series/S34.json (de2e3b4722ea) | alloc/A16.json |
| E32 | discriminator | test | dla 0.1.0 | artifact series/S36.json (245bc5fc9f9f) | alloc/A17.json |
| E33 | discriminator | test | dla 0.1.0 | artifact series/S38.json (c4cd39f53be9) | alloc/A18.json |
