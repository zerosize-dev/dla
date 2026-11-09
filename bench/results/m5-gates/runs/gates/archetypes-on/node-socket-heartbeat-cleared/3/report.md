# DLA report

Run: `20261109-205141-0142`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 13 enumerated, 13 dispositioned; confirmed 0, strong 0, refuted 13, suspected 0, unresolved 0

Residual: not measured

Coverage: node 7040 iterations, 25 of 30 entry points exercised

## Findings

No findings to report.

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E21
  - Ruled out by: plateau+purge (non-leak) E21
  - Contradicting evidence: E21
  - Originally supported by: E6
- **F2** (UNCLASSIFIED): discriminator E22
  - Ruled out by: plateau+purge (non-leak) E22
  - Contradicting evidence: E22
  - Originally supported by: E6
- **F3** (UNCLASSIFIED): discriminator E20
  - Ruled out by: plateau+purge (non-leak) E20
  - Contradicting evidence: E20
  - Originally supported by: E6
- **F4** (UNCLASSIFIED): discriminator E29
  - Ruled out by: plateau+purge (non-leak) E29
  - Contradicting evidence: E29
  - Originally supported by: E6
- **F5** (UNCLASSIFIED): discriminator E30
  - Ruled out by: plateau+purge (non-leak) E30
  - Contradicting evidence: E30
  - Originally supported by: E6
- **F6** (UNCLASSIFIED): discriminator E31
  - Ruled out by: plateau+purge (non-leak) E31
  - Contradicting evidence: E31
  - Originally supported by: E6
- **F7** (UNCLASSIFIED): discriminator E28
  - Ruled out by: plateau+purge (non-leak) E28
  - Contradicting evidence: E28
  - Originally supported by: E6
- **F8** (UNCLASSIFIED): discriminator E23
  - Ruled out by: plateau+purge (non-leak) E23
  - Contradicting evidence: E23
  - Originally supported by: E6
- **F9** (UNCLASSIFIED): discriminator E24
  - Ruled out by: plateau+purge (non-leak) E24
  - Contradicting evidence: E24
  - Originally supported by: E6
- **F10** (UNCLASSIFIED): discriminator E32
  - Ruled out by: plateau+purge (non-leak) E32
  - Contradicting evidence: E32
  - Originally supported by: E6
- **F11** (UNCLASSIFIED): discriminator E25
  - Ruled out by: plateau+purge (non-leak) E25
  - Contradicting evidence: E25
  - Originally supported by: E6
- **F12** (UNCLASSIFIED): discriminator E26
  - Ruled out by: plateau+purge (non-leak) E26
  - Contradicting evidence: E26
  - Originally supported by: E6
- **F13** (UNCLASSIFIED): discriminator E27
  - Ruled out by: plateau+purge (non-leak) E27
  - Contradicting evidence: E27
  - Originally supported by: E6

## Blind spots and unknowns

### Coverage gaps

- node/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- node/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)

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
- E17: limitation: sites are the first project frame of the stack that created the resource; a resource with no project frame is unattributed
- E17: limitation: active counts come from getActiveResourcesInfo and leave out unref'd timers
- E17: limitation: the files are ordered as given; each is one point
- E18: limitation: sites are the first project frame of the stack that created the resource; a resource with no project frame is unattributed
- E18: limitation: active counts come from getActiveResourcesInfo and leave out unref'd timers
- E18: limitation: the files are ordered as given; each is one point
- E18: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E19: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E19: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E19: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E19: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E19: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E19: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E19: limitation: points 1 to 9 come from S1; the extra points from S4
- E19: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E19: limitation: gate and plateau thresholds are provisional (not calibrated)
- E20: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E20: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E20: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E20: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E20: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E20: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E20: limitation: points 1 to 9 come from S1; the extra points from S6
- E20: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E20: limitation: gate and plateau thresholds are provisional (not calibrated)
- E21: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E21: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E21: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E21: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E21: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E21: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E21: limitation: points 1 to 9 come from S1; the extra points from S8
- E21: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E21: limitation: gate and plateau thresholds are provisional (not calibrated)
- E22: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E22: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E22: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E22: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E22: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E22: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E22: limitation: points 1 to 9 come from S1; the extra points from S10
- E22: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E22: limitation: gate and plateau thresholds are provisional (not calibrated)
- E23: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E23: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E23: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E23: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E23: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E23: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E23: limitation: points 1 to 9 come from S1; the extra points from S12
- E23: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E23: limitation: gate and plateau thresholds are provisional (not calibrated)
- E24: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E24: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E24: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E24: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E24: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E24: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E24: limitation: points 1 to 9 come from S1; the extra points from S14
- E24: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E24: limitation: gate and plateau thresholds are provisional (not calibrated)
- E25: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E25: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E25: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E25: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E25: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E25: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E25: limitation: points 1 to 9 come from S1; the extra points from S16
- E25: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E25: limitation: gate and plateau thresholds are provisional (not calibrated)
- E26: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E26: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E26: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E26: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E26: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E26: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E26: limitation: points 1 to 9 come from S1; the extra points from S18
- E26: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E26: limitation: gate and plateau thresholds are provisional (not calibrated)
- E27: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E27: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E27: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E27: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E27: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E27: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E27: limitation: points 1 to 9 come from S1; the extra points from S20
- E27: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E27: limitation: gate and plateau thresholds are provisional (not calibrated)
- E28: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E28: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E28: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E28: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E28: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E28: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E28: limitation: points 1 to 9 come from S1; the extra points from S22
- E28: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E28: limitation: gate and plateau thresholds are provisional (not calibrated)
- E29: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E29: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E29: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E29: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E29: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E29: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E29: limitation: points 1 to 9 come from S1; the extra points from S24
- E29: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E29: limitation: gate and plateau thresholds are provisional (not calibrated)
- E30: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E30: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E30: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E30: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E30: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E30: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E30: limitation: points 1 to 9 come from S1; the extra points from S26
- E30: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E30: limitation: gate and plateau thresholds are provisional (not calibrated)
- E31: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E31: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E31: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E31: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E31: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E31: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E31: limitation: points 1 to 9 come from S1; the extra points from S28
- E31: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E31: limitation: gate and plateau thresholds are provisional (not calibrated)
- E32: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E32: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E32: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E32: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E32: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E32: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E32: limitation: points 1 to 9 come from S1; the extra points from S30
- E32: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E32: limitation: gate and plateau thresholds are provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | — |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | — |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | — |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E3 | — |
| `intake.build` | Does the project build locally? | answered | E3 | — |
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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (7ff7de7eb4f1) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (320340a4c057) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (320340a4c057) | trend/T1.json |
| E12 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261109-205141-0142/harness/H1/out/p1-native/capture/9/heap-1.heapsnapshot (b39f811d410c) | summaries/heap-1-b39f811d.json |
| E13 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261109-205141-0142/harness/H1/out/p4-native/capture/34/heap-1.heapsnapshot (431f01dcdffd) | summaries/heap-1-431f01dc.json |
| E14 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261109-205141-0142/harness/H1/out/p6-native/capture/50/heap-1.heapsnapshot (cccd8cfe9d6e) | summaries/heap-1-cccd8cfe.json |
| E15 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261109-205141-0142/harness/H1/out/p9-native/capture/74/heap-1.heapsnapshot (e283d1c78966) | summaries/heap-1-e283d1c7.json |
| E16 | growth-series | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (c466b7b53d32) | series/S2.json |
| E17 | non-heap | live | node-resources 0.1.0 | artifact nonheap/N1.json (2ef79cd737a8) | nonheap/N1.json |
| E18 | growth-series | live | node-resources 0.1.0 | artifact series/S3.json (f6859d5d3d7f) | series/S3.json |
| E19 | discriminator | test | dla 0.1.0 | artifact series/S5.json (ac33d947015e) | alloc/A1.json |
| E20 | discriminator | test | dla 0.1.0 | artifact series/S7.json (580c13f8cb62) | alloc/A2.json |
| E21 | discriminator | test | dla 0.1.0 | artifact series/S9.json (c98bc0c2818b) | alloc/A3.json |
| E22 | discriminator | test | dla 0.1.0 | artifact series/S11.json (4eca80d02dfb) | alloc/A4.json |
| E23 | discriminator | test | dla 0.1.0 | artifact series/S13.json (279bc1a23bcb) | alloc/A5.json |
| E24 | discriminator | test | dla 0.1.0 | artifact series/S15.json (742ba830893b) | alloc/A6.json |
| E25 | discriminator | test | dla 0.1.0 | artifact series/S17.json (fa48de191ac6) | alloc/A7.json |
| E26 | discriminator | test | dla 0.1.0 | artifact series/S19.json (625500cee7e9) | alloc/A8.json |
| E27 | discriminator | test | dla 0.1.0 | artifact series/S21.json (6b574a6b9de1) | alloc/A9.json |
| E28 | discriminator | test | dla 0.1.0 | artifact series/S23.json (f89caee8be30) | alloc/A10.json |
| E29 | discriminator | test | dla 0.1.0 | artifact series/S25.json (06de2bae586f) | alloc/A11.json |
| E30 | discriminator | test | dla 0.1.0 | artifact series/S27.json (653b1daf45fe) | alloc/A12.json |
| E31 | discriminator | test | dla 0.1.0 | artifact series/S29.json (9b78521e2e4a) | alloc/A13.json |
| E32 | discriminator | test | dla 0.1.0 | artifact series/S31.json (3b51fe86ff87) | alloc/A14.json |
