# DLA report

Run: `20261110-174743-cfb4`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 11 enumerated, 11 dispositioned; confirmed 1, strong 0, refuted 4, suspected 0, unresolved 6

Residual: heap 4287.629999999999 bytes/n unattributed of 4287.629999999999 total (threshold 1071.9074999999998) ABOVE THRESHOLD, see O1; heap 4270.504545454545 bytes/n unattributed of 4270.504545454545 total (threshold 1067.6261363636363) ABOVE THRESHOLD, see O1; heap 4265.812121212121 bytes/n unattributed of 4265.812121212121 total (threshold 1066.4530303030303) ABOVE THRESHOLD, see O1; heap 4270.521103896104 bytes/n unattributed of 4270.521103896104 total (threshold 1067.630275974026) ABOVE THRESHOLD, see O1; heap 4269.571212121213 bytes/n unattributed of 4269.571212121213 total (threshold 1067.3928030303032) ABOVE THRESHOLD, see O1; heap 4271.938888888889 bytes/n unattributed of 4271.938888888889 total (threshold 1067.9847222222222) ABOVE THRESHOLD, see O1; heap 4264.263636363636 bytes/n unattributed of 4264.263636363636 total (threshold 1066.065909090909) ABOVE THRESHOLD, see O1; heap 4270.6797979797975 bytes/n unattributed of 4270.6797979797975 total (threshold 1067.6699494949494) ABOVE THRESHOLD, see O1; heap 4264.522727272728 bytes/n unattributed of 4264.522727272728 total (threshold 1066.130681818182) ABOVE THRESHOLD, see O1; off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0)

Coverage: node 3520 iterations, 17 of 19 entry points exercised

## Findings

### F5: **L-STALE**

- State: confirmed
- Label: L-STALE (evidence: E17 growth-series, E13 retention-path)
- Growth: `probe:src/scheduler.js:shared.latest` 25.97 count/n (steady, gate pass, E11)
- Evidence classes: context, discriminator, growth, static
- Holder: static/global
- Owner: `src.scheduler.RetryScheduler.latest`
- Region: heap
- Shape: steady
- Site: `src/scheduler.js:15`
- Writers: `src/scheduler.js:32`, `src/scheduler.js:33`
- Annotation: heuristic name match over the project's sources
- Annotation: latest/previous chain links every attempt record; never trimmed
- Annotation: RetryScheduler.latest (module singleton via sharedScheduler) heads a previous-linked chain of every attempt record; finished records keep their job and payload alive
- Verified: no
- Sibling hypotheses: F5.n1
- Supporting evidence: E6, E9, E11, E12
- Explanation (inferred): RetryScheduler.latest at src/scheduler.js:15 is a static/global holder reached via sharedScheduler; enqueue (src/scheduler.js:32) links every attempt record to the previous one and nothing removes the link, it is never cleared, so delivered records and their jobs stay reachable.
- Explanation structure: adds: src/scheduler.js:32; removal: never

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `src/scheduler.js:32`
- Reason: not_exercised
- Remedy: detector hit on src/scheduler.js:32 is the same mechanism confirmed as F5; no separate action
- Supporting evidence: E5

### F2: **UNCLASSIFIED**

- State: unresolved(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `src.pending.PendingList.head`
- Region: heap
- Site: `src/pending.js:7`
- Writers: `src/pending.js:15`, `src/pending.js:24`
- Annotation: heuristic name match over the project's sources
- Reason: no_harness
- Remedy: PendingList.head: no probe; pending length returns to zero (F4 plateau), so not a growth source
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: unresolved(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `src.pending.PendingList.tail`
- Region: heap
- Site: `src/pending.js:8`
- Writers: `src/pending.js:16`, `src/pending.js:25`
- Annotation: heuristic name match over the project's sources
- Reason: no_harness
- Remedy: PendingList.tail: no probe; unlink resets tail, pending drains (F4 plateau)
- Supporting evidence: E6

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth, static
- Holder: static/global
- Owner: `src.scheduler.shared`
- Region: heap
- Site: `src/scheduler.js:90`
- Writers: `src/scheduler.js:93`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: F8 is the module singleton that roots F5's chain; fix F5 (stop linking records via previous / clear latest), no separate fix
- Supporting evidence: E6, E17

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: same growth as F5: retention path from shared singleton through latest/previous chain; covered by F5
- Supporting evidence: E23

### F10: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: count column of the same F5 chain; covered by F5
- Supporting evidence: E24

## Ruled out

- **F4** (UNCLASSIFIED): discriminator E25
  - Ruled out by: plateau+purge (non-leak) E25
  - Contradicting evidence: E25
  - Originally supported by: E6
- **F5.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM: growth is a warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E12
  - Originally supported by: E11
- **F6** (UNCLASSIFIED): discriminator E26
  - Ruled out by: plateau+purge (non-leak) E26
  - Contradicting evidence: E26
  - Originally supported by: E6
- **F7** (UNCLASSIFIED): discriminator E27
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
- E13: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E13: limitation: weak edges are skipped; string contents are never read into the summary
- E14: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E14: limitation: weak edges are skipped; string contents are never read into the summary
- E15: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E15: limitation: weak edges are skipped; string contents are never read into the summary
- E16: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E16: limitation: weak edges are skipped; string contents are never read into the summary
- E17: limitation: dump-series rule: at least 3 snapshots, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E17: limitation: series: per-point; one process per point at n = 440, 1760, 2640, 3960 (harness captures; checked: snapshot format)
- E18: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E19: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E20: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E21: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E21: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E22: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E23: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E23: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E24: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E24: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E25: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E25: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E25: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E25: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E25: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E25: limitation: points 1 to 9 come from S1; the extra points from S5
- E25: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E25: limitation: gate and plateau thresholds are provisional (not calibrated)
- E26: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E26: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E26: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E26: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E26: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E26: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E26: limitation: points 1 to 9 come from S1; the extra points from S7
- E26: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E26: limitation: gate and plateau thresholds are provisional (not calibrated)
- E27: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E27: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E27: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E27: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E27: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E27: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E27: limitation: points 1 to 9 come from S1; the extra points from S9
- E27: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E27: limitation: gate and plateau thresholds are provisional (not calibrated)
- E28: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E28: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E29: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E29: limitation: gate thresholds are provisional (not calibrated)
- E30: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E30: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E30: limitation: gate thresholds are provisional (not calibrated)
- E31: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E31: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E31: limitation: gate thresholds are provisional (not calibrated)
- E32: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E32: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E32: limitation: gate thresholds are provisional (not calibrated)
- E33: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E33: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E33: limitation: gate thresholds are provisional (not calibrated)
- E34: partial coverage (2 of 3 examined)
- E34: limitation: F5: growth measured as a count; bytes not attributed
- E34: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E34: limitation: threshold is provisional (not calibrated)
- E35: partial coverage (2 of 3 examined)
- E35: limitation: F5: growth evidence is from S1, not S2; not subtracted
- E35: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E35: limitation: threshold is provisional (not calibrated)
- E36: partial coverage (2 of 3 examined)
- E36: limitation: F5: growth evidence is from S1, not S3; not subtracted
- E36: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E36: limitation: threshold is provisional (not calibrated)
- E37: partial coverage (2 of 3 examined)
- E37: limitation: F5: growth evidence is from S1, not S5; not subtracted
- E37: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E37: limitation: threshold is provisional (not calibrated)
- E38: partial coverage (2 of 3 examined)
- E38: limitation: F5: growth evidence is from S1, not S6; not subtracted
- E38: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E38: limitation: threshold is provisional (not calibrated)
- E39: partial coverage (2 of 3 examined)
- E39: limitation: F5: growth evidence is from S1, not S7; not subtracted
- E39: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E39: limitation: threshold is provisional (not calibrated)
- E40: partial coverage (2 of 3 examined)
- E40: limitation: F5: growth evidence is from S1, not S8; not subtracted
- E40: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E40: limitation: threshold is provisional (not calibrated)
- E41: partial coverage (2 of 3 examined)
- E41: limitation: F5: growth evidence is from S1, not S9; not subtracted
- E41: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E41: limitation: threshold is provisional (not calibrated)
- E42: partial coverage (2 of 3 examined)
- E42: limitation: F5: growth evidence is from S1, not S10; not subtracted
- E42: limitation: off-heap: no gap series for S10; run dla gap S10 first (node.external is used when present)
- E42: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | — |
| E2 | run-config | static | dla 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | probe/probe.json |
| E4 | static-hit | static | eslint 10.14.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | sweep/node-eslint.json |
| E5 | static-hit | static | dla 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | sweep/node-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | census/node.json |
| E7 | workload | static | dla 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (ffe248f790b8) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (4ee230933ab2) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (4ee230933ab2) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (6607df7bfb36) | alloc/A1.json |
| E13 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-174743-cfb4/harness/H1/out/p1-native/capture/9/heap-1.heapsnapshot (adc841dd7744) | summaries/heap-1-adc841dd.json |
| E14 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-174743-cfb4/harness/H1/out/p4-native/capture/34/heap-1.heapsnapshot (437aac8097f1) | summaries/heap-1-437aac80.json |
| E15 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-174743-cfb4/harness/H1/out/p6-native/capture/50/heap-1.heapsnapshot (3700e9e1fab0) | summaries/heap-1-3700e9e1.json |
| E16 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-174743-cfb4/harness/H1/out/p9-native/capture/74/heap-1.heapsnapshot (c5febb861d23) | summaries/heap-1-c5febb86.json |
| E17 | growth-series | dump | dla-v8-snapshot 0.1.0 | artifact series/S4.json (7ec7e737e409) | series/S4.json |
| E18 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S4.json (7ec7e737e409) | series/S4.json |
| E19 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S4.json (7ec7e737e409) | series/S4.json |
| E20 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S4.json (7ec7e737e409) | series/S4.json |
| E21 | growth-series | test | dla 0.1.0 | artifact series/S2.json (56b1434e08ed) | trend/T3.json |
| E22 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (56b1434e08ed) | trend/T3.json |
| E23 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (7ec7e737e409) | trend/T4.json |
| E24 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (7ec7e737e409) | trend/T4.json |
| E25 | discriminator | test | dla 0.1.0 | artifact series/S6.json (6b4847dfe20a) | alloc/A2.json |
| E26 | discriminator | test | dla 0.1.0 | artifact series/S8.json (0692904410df) | alloc/A3.json |
| E27 | discriminator | test | dla 0.1.0 | artifact series/S10.json (71e750551b5b) | alloc/A4.json |
| E28 | growth-series | test | dla 0.1.0 | artifact series/S5.json (d95767d78d2e) | trend/T8.json |
| E29 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (d95767d78d2e) | trend/T8.json |
| E30 | growth-series | test | dla 0.1.0 | artifact series/S7.json (758b00a9a010) | trend/T9.json |
| E31 | unit-count-series | test | dla 0.1.0 | artifact series/S7.json (758b00a9a010) | trend/T9.json |
| E32 | growth-series | test | dla 0.1.0 | artifact series/S9.json (61fafb285aa6) | trend/T10.json |
| E33 | unit-count-series | test | dla 0.1.0 | artifact series/S9.json (61fafb285aa6) | trend/T10.json |
| E34 | residual | test | dla 0.1.0 | artifact series/S1.json (4ee230933ab2) | gap/residual-S1-E34.json |
| E35 | residual | test | dla 0.1.0 | artifact series/S2.json (56b1434e08ed) | gap/residual-S2-E35.json |
| E36 | residual | test | dla 0.1.0 | artifact series/S3.json (6607df7bfb36) | gap/residual-S3-E36.json |
| E37 | residual | test | dla 0.1.0 | artifact series/S5.json (d95767d78d2e) | gap/residual-S5-E37.json |
| E38 | residual | test | dla 0.1.0 | artifact series/S6.json (6b4847dfe20a) | gap/residual-S6-E38.json |
| E39 | residual | test | dla 0.1.0 | artifact series/S7.json (758b00a9a010) | gap/residual-S7-E39.json |
| E40 | residual | test | dla 0.1.0 | artifact series/S8.json (0692904410df) | gap/residual-S8-E40.json |
| E41 | residual | test | dla 0.1.0 | artifact series/S9.json (61fafb285aa6) | gap/residual-S9-E41.json |
| E42 | residual | test | dla 0.1.0 | artifact series/S10.json (71e750551b5b) | gap/residual-S10-E42.json |
