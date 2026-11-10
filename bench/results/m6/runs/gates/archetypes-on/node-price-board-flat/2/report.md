# DLA report

Run: `20261110-165750-5add`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 3 enumerated, 3 dispositioned; confirmed 0, strong 0, refuted 3, suspected 0, unresolved 0

Residual: heap 8.620909090909091 bytes/n unattributed of 8.620909090909091 total (threshold 20.46738927272754); heap 0 bytes/n unattributed of 0 total (threshold 0); heap 0.012554112554112554 bytes/n unattributed of 0.012554112554112554 total (threshold 0.009352207792239811) ABOVE THRESHOLD, see O1; off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0)

Coverage: node 3520 iterations, 10 of 11 entry points exercised

## Findings

No findings to report.

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (non-leak) E12
  - Contradicting evidence: E12
  - Originally supported by: E6
- **F2** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (non-leak) E13
  - Contradicting evidence: E13
  - Originally supported by: E6
- **F3** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (non-leak) E14
  - Contradicting evidence: E14
  - Originally supported by: E6

## Blind spots and unknowns

### Coverage gaps

- node/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- node/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)

### Runtime blind spots

None recorded.

### Entry points not exercisable

None recorded.

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
- E13: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E13: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E13: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E13: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E13: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E13: limitation: points 1 to 9 come from S1; the extra points from S4
- E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- E14: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E14: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E14: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E14: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E14: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E14: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E14: limitation: points 1 to 9 come from S1; the extra points from S6
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E15: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E16: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E17: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E18: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E19: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E20: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: partial coverage (2 of 3 examined)
- E21: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E21: limitation: threshold is provisional (not calibrated)
- E22: partial coverage (2 of 3 examined)
- E22: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E22: limitation: threshold is provisional (not calibrated)
- E23: partial coverage (2 of 3 examined)
- E23: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E23: limitation: threshold is provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | repo cwd; node price-board; load.js workload; runner consent tiers 1-4 |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | repo cwd; node price-board; load.js workload; runner consent tiers 1-4 |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | repo cwd; node price-board; load.js workload; runner consent tiers 1-4 |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | repo cwd; node price-board; load.js workload; runner consent tiers 1-4 |
| `intake.build` | Does the project build locally? | answered | E1 | repo cwd; node price-board; load.js workload; runner consent tiers 1-4 |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | repo cwd; node price-board; load.js workload; runner consent tiers 1-4 |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E1 | repo cwd; node price-board; load.js workload; runner consent tiers 1-4 |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | repo cwd; node price-board; load.js workload; runner consent tiers 1-4 |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo e91c117114645a5643a4ae62870a620311c2969d | — |
| E2 | run-config | static | dla 0.1.0 | repo e91c117114645a5643a4ae62870a620311c2969d | — |
| E3 | probe | static | dla-probe 0.1.0 | repo e91c117114645a5643a4ae62870a620311c2969d | probe/probe.json |
| E4 | static-hit | static | eslint 10.14.0 | repo e91c117114645a5643a4ae62870a620311c2969d | sweep/node-eslint.json |
| E5 | static-hit | static | dla 0.1.0 | repo e91c117114645a5643a4ae62870a620311c2969d | sweep/node-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo e91c117114645a5643a4ae62870a620311c2969d | census/node.json |
| E7 | workload | static | dla 0.1.0 | repo e91c117114645a5643a4ae62870a620311c2969d | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo e91c117114645a5643a4ae62870a620311c2969d | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (17db1b29848f) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (b471460c237f) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (b471460c237f) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (1e2dec0cacf7) | alloc/A1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S5.json (8876e1b866e3) | alloc/A2.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S7.json (914b616a5bff) | alloc/A3.json |
| E15 | growth-series | test | dla 0.1.0 | artifact series/S2.json (3630897aee36) | trend/T5.json |
| E16 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (3630897aee36) | trend/T5.json |
| E17 | growth-series | test | dla 0.1.0 | artifact series/S4.json (7264d4f1d9b9) | trend/T6.json |
| E18 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (7264d4f1d9b9) | trend/T6.json |
| E19 | growth-series | test | dla 0.1.0 | artifact series/S6.json (c9aeaf9e5b0c) | trend/T7.json |
| E20 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (c9aeaf9e5b0c) | trend/T7.json |
| E21 | residual | test | dla 0.1.0 | artifact series/S1.json (b471460c237f) | gap/residual-S1-E21.json |
| E22 | residual | test | dla 0.1.0 | artifact series/S2.json (3630897aee36) | gap/residual-S2-E22.json |
| E23 | residual | test | dla 0.1.0 | artifact series/S6.json (c9aeaf9e5b0c) | gap/residual-S6-E23.json |
