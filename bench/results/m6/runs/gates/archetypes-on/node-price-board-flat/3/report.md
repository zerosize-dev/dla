# DLA report

Run: `20261110-174247-8288`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 3 enumerated, 3 dispositioned; confirmed 0, strong 0, refuted 3, suspected 0, unresolved 0

Residual: heap 9.076363636363636 bytes/n unattributed of 9.076363636363636 total (threshold 20.820325090909225); off-heap 0 bytes/n unattributed of 0 total (threshold 0)

Coverage: node 3520 iterations, 10 of 11 entry points exercised

## Findings

No findings to report.

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E20
  - Ruled out by: plateau+purge (non-leak) E20
  - Contradicting evidence: E20
  - Originally supported by: E6
- **F2** (UNCLASSIFIED): discriminator E21
  - Ruled out by: plateau+purge (non-leak) E21
  - Contradicting evidence: E21
  - Originally supported by: E6
- **F3** (UNCLASSIFIED): discriminator E22
  - Ruled out by: plateau+purge (non-leak) E22
  - Contradicting evidence: E22
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
- E12: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E12: limitation: weak edges are skipped; string contents are never read into the summary
- E13: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E13: limitation: weak edges are skipped; string contents are never read into the summary
- E14: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E14: limitation: weak edges are skipped; string contents are never read into the summary
- E15: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E15: limitation: weak edges are skipped; string contents are never read into the summary
- E16: limitation: dump-series rule: at least 3 snapshots, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E16: limitation: series: per-point; one process per point at n = 440, 1760, 2640, 3960 (harness captures; checked: snapshot format)
- E17: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E17: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E17: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E18: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E18: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E19: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E19: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E19: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E19: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E19: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E19: limitation: points 1 to 9 come from S1; the extra points from S3
- E19: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E19: limitation: gate and plateau thresholds are provisional (not calibrated)
- E20: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E20: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E20: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E20: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E20: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E20: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E20: limitation: points 1 to 9 come from S1; the extra points from S5
- E20: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E20: limitation: gate and plateau thresholds are provisional (not calibrated)
- E21: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E21: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E21: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E21: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E21: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E21: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E21: limitation: points 1 to 9 come from S1; the extra points from S7
- E21: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E21: limitation: gate and plateau thresholds are provisional (not calibrated)
- E22: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E22: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E22: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E22: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E22: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E22: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E22: limitation: points 1 to 9 come from S1; the extra points from S9
- E22: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E22: limitation: gate and plateau thresholds are provisional (not calibrated)
- E23: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E23: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E24: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E25: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E26: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E27: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E28: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E29: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E29: limitation: gate thresholds are provisional (not calibrated)
- E30: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E30: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E30: limitation: gate thresholds are provisional (not calibrated)
- E31: partial coverage (2 of 3 examined)
- E31: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E31: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo e91c117114645a5643a4ae62870a620311c2969d | — |
| E2 | run-config | static | dla 0.1.0 | repo e91c117114645a5643a4ae62870a620311c2969d | — |
| E3 | probe | static | dla-probe 0.1.0 | repo e91c117114645a5643a4ae62870a620311c2969d | probe/probe.json |
| E4 | static-hit | static | eslint 10.14.0 | repo e91c117114645a5643a4ae62870a620311c2969d | sweep/node-eslint.json |
| E5 | static-hit | static | dla 0.1.0 | repo e91c117114645a5643a4ae62870a620311c2969d | sweep/node-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo e91c117114645a5643a4ae62870a620311c2969d | census/node.json |
| E7 | workload | static | dla 0.1.0 | repo e91c117114645a5643a4ae62870a620311c2969d | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo e91c117114645a5643a4ae62870a620311c2969d | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (e5705e2ad1fe) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (ecec99956f5a) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (ecec99956f5a) | trend/T1.json |
| E12 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-174247-8288/harness/H1/out/p1-native/capture/16/heap-1.heapsnapshot (424fcee08025) | summaries/heap-1-424fcee0.json |
| E13 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-174247-8288/harness/H1/out/p4-native/capture/40/heap-1.heapsnapshot (88c06c1192ee) | summaries/heap-1-88c06c11.json |
| E14 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-174247-8288/harness/H1/out/p6-native/capture/56/heap-1.heapsnapshot (d38c65434642) | summaries/heap-1-d38c6543.json |
| E15 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-174247-8288/harness/H1/out/p9-native/capture/80/heap-1.heapsnapshot (08a0f245ec1d) | summaries/heap-1-08a0f245.json |
| E16 | growth-series | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (e748e5584a9e) | series/S2.json |
| E17 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (e748e5584a9e) | trend/T2.json |
| E18 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (e748e5584a9e) | trend/T2.json |
| E19 | discriminator | test | dla 0.1.0 | artifact series/S4.json (781b7feeb88f) | alloc/A1.json |
| E20 | discriminator | test | dla 0.1.0 | artifact series/S6.json (d5c5862e2aec) | alloc/A2.json |
| E21 | discriminator | test | dla 0.1.0 | artifact series/S8.json (3ef1d2e08d72) | alloc/A3.json |
| E22 | discriminator | test | dla 0.1.0 | artifact series/S10.json (7224dedf09c0) | alloc/A4.json |
| E23 | growth-series | test | dla 0.1.0 | artifact series/S3.json (3c2561626bc8) | trend/T7.json |
| E24 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (3c2561626bc8) | trend/T7.json |
| E25 | growth-series | test | dla 0.1.0 | artifact series/S5.json (0fa4cbb738b4) | trend/T8.json |
| E26 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (0fa4cbb738b4) | trend/T8.json |
| E27 | growth-series | test | dla 0.1.0 | artifact series/S7.json (ce81c3df91f3) | trend/T9.json |
| E28 | unit-count-series | test | dla 0.1.0 | artifact series/S7.json (ce81c3df91f3) | trend/T9.json |
| E29 | growth-series | test | dla 0.1.0 | artifact series/S9.json (1d7bdab6f0ff) | trend/T10.json |
| E30 | unit-count-series | test | dla 0.1.0 | artifact series/S9.json (1d7bdab6f0ff) | trend/T10.json |
| E31 | residual | test | dla 0.1.0 | artifact series/S1.json (ecec99956f5a) | gap/residual-S1-E31.json |
