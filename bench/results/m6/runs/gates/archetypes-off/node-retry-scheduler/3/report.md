# DLA report

Run: `20261110-193657-8134`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 11 enumerated, 11 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 9

Residual: heap 4287.6163636363635 bytes/n unattributed of 4287.6163636363635 total (threshold 1071.9040909090909) ABOVE THRESHOLD, see O1; heap 4265.787878787879 bytes/n unattributed of 4265.787878787879 total (threshold 1066.4469696969695) ABOVE THRESHOLD, see O1; heap 4271.163181818181 bytes/n unattributed of 4271.163181818181 total (threshold 1067.790795454545) ABOVE THRESHOLD, see O1; off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0)

Coverage: node 3520 iterations, 17 of 19 entry points exercised

## Findings

### F5: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (RetryScheduler.latest (src/scheduler.js:32, enqueue) chains every attempt record via record.previous and is never trimmed; each record retains its job and payload rows, so the singly linked history grows without bound from the process-lifetime shared scheduler (module variable) even after pending/inflight drain. recent() reads only 10 entries.)
- Growth: `probe:src/scheduler.js:shared.latest` 25.97 count/n (steady, gate pass, E11)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `src.scheduler.RetryScheduler.latest`
- Region: heap
- Shape: steady
- Site: `src/scheduler.js:15`
- Writers: `src/scheduler.js:32`
- Annotation: heuristic name match over the project's sources
- Verified: no
- Sibling hypotheses: F5.n1
- Supporting evidence: E6, E9, E11
- Explanation (observed): RetryScheduler.latest declared at src/scheduler.js:15 is reachable from module variable shared; enqueue() at src/scheduler.js:32 links each record to previous and nothing ever unlinks previous: it is never removed, so every attempt record and its job payload stays retained.
- Explanation structure: adds: src/scheduler.js:32; removal: never; evidence: E19

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `src/scheduler.js:32`
- Reason: not_exercised
- Remedy: Detector hit at src/scheduler.js:32 is the same mechanism as confirmed F5 (previous-chain head never trimmed); not tracked separately.
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
- Remedy: PendingList head/tail drain to empty (pending=0 after each batch; unlink clears node.next); not the holder per heap-graph. Not tested separately.
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
- Remedy: PendingList head/tail drain to empty (pending=0 after each batch; unlink clears node.next); not the holder per heap-graph. Not tested separately.
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
- Remedy: Not separately tested: the column that lists it is the F5 chain (probe shared.latest); this holder drains or is bounded (pending empty, inflight 0, deadLetters capped at 100) and growth is attributed to F5 per heap-graph E19-E21. Re-test after F5 is fixed.
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
- Remedy: Not separately tested: the column that lists it is the F5 chain (probe shared.latest); this holder drains or is bounded (pending empty, inflight 0, deadLetters capped at 100) and growth is attributed to F5 per heap-graph E19-E21. Re-test after F5 is fixed.
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
- Remedy: Not separately tested: the column that lists it is the F5 chain (probe shared.latest); this holder drains or is bounded (pending empty, inflight 0, deadLetters capped at 100) and growth is attributed to F5 per heap-graph E19-E21. Re-test after F5 is fixed.
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
- Remedy: Not separately tested: the column that lists it is the F5 chain (probe shared.latest); this holder drains or is bounded (pending empty, inflight 0, deadLetters capped at 100) and growth is attributed to F5 per heap-graph E19-E21. Re-test after F5 is fixed.
- Supporting evidence: E6

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: Growth column of the heap-graph series T4 (shared / Object.rows retained size) is the F5 chain and its job payload rows; accounted to F5, not tested separately.
- Supporting evidence: E23

### F10: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: Growth column of the heap-graph series T4 (shared / Object.rows retained size) is the F5 chain and its job payload rows; accounted to F5, not tested separately.
- Supporting evidence: E24

## Ruled out

- **F5.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM: latest chain length is warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E12
  - Originally supported by: E11

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
- E13: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E13: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E14: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E15: limitation: weak edges are skipped; string contents are never read into the summary
- E16: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E16: limitation: weak edges are skipped; string contents are never read into the summary
- E17: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E17: limitation: weak edges are skipped; string contents are never read into the summary
- E18: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E18: limitation: weak edges are skipped; string contents are never read into the summary
- E19: limitation: dump-series rule: at least 3 snapshots, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E19: limitation: series: per-point; one process per point at n = 440, 1760, 2640, 3960 (harness captures; checked: snapshot format)
- E20: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E21: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E22: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E23: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E23: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E23: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E24: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E24: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: partial coverage (2 of 3 examined)
- E25: limitation: F5: growth measured as a count; bytes not attributed
- E25: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E25: limitation: threshold is provisional (not calibrated)
- E26: partial coverage (2 of 3 examined)
- E26: limitation: F5: growth evidence is from S1, not S3; not subtracted
- E26: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E26: limitation: threshold is provisional (not calibrated)
- E27: partial coverage (2 of 3 examined)
- E27: limitation: F5: growth evidence is from S1, not S2; not subtracted
- E27: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E27: limitation: threshold is provisional (not calibrated)

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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (f3ad0ea35a55) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (99fd955844a1) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (99fd955844a1) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (e56900f17a50) | alloc/A1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S2.json (0291dcb6d5e4) | trend/T3.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (0291dcb6d5e4) | trend/T3.json |
| E15 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-193657-8134/harness/H1/out/p1-native/capture/9/heap-1.heapsnapshot (335b251354d7) | summaries/heap-1-335b2513.json |
| E16 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-193657-8134/harness/H1/out/p4-native/capture/34/heap-1.heapsnapshot (73e31a3421dc) | summaries/heap-1-73e31a34.json |
| E17 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-193657-8134/harness/H1/out/p6-native/capture/50/heap-1.heapsnapshot (aafe53b6c166) | summaries/heap-1-aafe53b6.json |
| E18 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-193657-8134/harness/H1/out/p9-native/capture/74/heap-1.heapsnapshot (96fec3b4b755) | summaries/heap-1-96fec3b4.json |
| E19 | growth-series | dump | dla-v8-snapshot 0.1.0 | artifact series/S4.json (7b3230d295fa) | series/S4.json |
| E20 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S4.json (7b3230d295fa) | series/S4.json |
| E21 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S4.json (7b3230d295fa) | series/S4.json |
| E22 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S4.json (7b3230d295fa) | series/S4.json |
| E23 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (7b3230d295fa) | trend/T4.json |
| E24 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (7b3230d295fa) | trend/T4.json |
| E25 | residual | test | dla 0.1.0 | artifact series/S1.json (99fd955844a1) | gap/residual-S1-E25.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S3.json (e56900f17a50) | gap/residual-S3-E26.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S2.json (0291dcb6d5e4) | gap/residual-S2-E27.json |
