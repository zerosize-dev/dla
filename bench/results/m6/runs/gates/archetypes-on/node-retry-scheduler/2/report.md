# DLA report

Run: `20261110-170228-5697`

Status: INCOMPLETE (2 open)

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 11 enumerated, 11 dispositioned; confirmed 2, strong 0, refuted 2, suspected 0, unresolved 7

Residual: heap 4287.604090909091 bytes/n unattributed of 4287.604090909091 total (threshold 1071.901022727273) ABOVE THRESHOLD, see O2; heap 4267.467424242424 bytes/n unattributed of 4267.467424242424 total (threshold 1066.866856060606) ABOVE THRESHOLD, see O1; heap 4271.493722943724 bytes/n unattributed of 4271.493722943724 total (threshold 1067.873430735931) ABOVE THRESHOLD, see O1; heap 4262.628787878788 bytes/n unattributed of 4262.628787878788 total (threshold 1065.657196969697) ABOVE THRESHOLD, see O1; heap 4270.987662337662 bytes/n unattributed of 4270.987662337662 total (threshold 1067.7469155844155) ABOVE THRESHOLD, see O1; off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0)

Coverage: node 3520 iterations, 17 of 19 entry points exercised

## Open items

- **T8:holder:static:.dla.20261110-170228-5697.harness.H1.work.src.scheduler.shared.count**: column holder:static:.dla.20261110-170228-5697.harness.H1.work.src.scheduler.shared.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E36 # column holder:static:.dla.20261110-170228-5697.harness.H1.work.src.scheduler.shared.count`
- **T8:holder:field:Object.rows.count**: column holder:field:Object.rows.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E36 # column holder:field:Object.rows.count`

## Findings

### F5: **L-COLLECT**

- State: confirmed
- Label: L-COLLECT (evidence: E11 unit-count-series, E6 census-holder)
- Growth: `probe:src/scheduler.js:shared.latest` 25.97 count/n (steady, gate pass, E11)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `src.scheduler.RetryScheduler.latest`
- Region: heap
- Shape: steady
- Site: `src/scheduler.js:15`
- Writers: `src/scheduler.js:32`
- Removers: `src/scheduler.js:32`
- Annotation: heuristic name match over the project's sources
- Annotation: x
- Verified: no
- Sibling hypotheses: F5.n1
- Supporting evidence: E6, E9, E11
- Explanation (inferred): Holder latest (scheduler.js:15) moves on each enqueue at scheduler.js:32, so all records stay reachable unless previous were cleared, which no code does. The remover scheduler.js:32 is a mistaken analyst annotation: it is the add site.
- Explanation structure: adds: src/scheduler.js:32; removal: conditional at src/scheduler.js:32

### F8: **UNCLASSIFIED**

- State: confirmed
- Growth: `probe:src/scheduler.js:shared` 25.95 count/n (steady, gate pass, E16)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `src.scheduler.shared`
- Region: heap
- Shape: steady
- Site: `src/scheduler.js:90`
- Writers: `src/scheduler.js:93`
- Annotation: heuristic name match over the project's sources
- Verified: no
- Sibling hypotheses: F8.n1
- Supporting evidence: E6, E9, E16
- Explanation (inferred): Module-level shared (scheduler.js:90, created at scheduler.js:93) lives for the whole process and holds latest, whose previous chain is never released, keeping every delivered or dead attempt record and its job payload reachable after the batch drains.
- Explanation structure: adds: src/scheduler.js:93; removal: never

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `src/scheduler.js:32`
- Reason: not_exercised
- Remedy: Static detector hit at scheduler.js:32 is the same mechanism as confirmed F5; no separate test run.
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
- Remedy: PendingList head/tail empty after drain (pending=0); no dedicated probe. Add one to confirm flat.
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
- Remedy: PendingList head/tail empty after drain (pending=0); no dedicated probe. Add one to confirm flat.
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
- Remedy: Holder is bounded/emptied per the stats (pending=0, inflight=0, deadLetters capped at 100); its listing is only the shared.latest column. Add a dedicated probe to confirm flat.
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
- Remedy: Holder is bounded/emptied per the stats (pending=0, inflight=0, deadLetters capped at 100); its listing is only the shared.latest column. Add a dedicated probe to confirm flat.
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
- Remedy: Holder is bounded/emptied per the stats (pending=0, inflight=0, deadLetters capped at 100); its listing is only the shared.latest column. Add a dedicated probe to confirm flat.
- Supporting evidence: E6

### F9: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: Heap-graph columns (shared, Object.rows) are the retained chain and job payload rows of confirmed F5/F8; not separately confirmed.
- Supporting evidence: E35

## Ruled out

- **F5.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM: latest chain is a warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E12
  - Originally supported by: E11
- **F8.n1** (UNCLASSIFIED): discriminator E17
  - Ruled out by: plateau+purge (leak) E17
  - Hypothesis: N-WARM: shared singleton count is warm-up
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E17
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
- E9: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E10: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E10: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E10: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E11: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E11: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E12: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
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
- E15: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E15: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E15: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E15: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E15: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E15: limitation: points 1 to 9 come from S1; the extra points from S2
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E16: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E16: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E16: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E16: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E16: limitation: points 1 to 9 come from S1; the extra points from S2
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E17: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E17: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E17: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E17: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E17: limitation: points 1 to 9 come from S1; the extra points from S4
- E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E17: limitation: gate and plateau thresholds are provisional (not calibrated)
- E18: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E18: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E19: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E20: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E20: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E20: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E20: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E20: limitation: points 1 to 9 come from S1; the extra points from S4
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E21: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E21: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E21: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E21: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E21: limitation: points 1 to 9 come from S1; the extra points from S4
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: partial coverage (2 of 3 examined)
- E22: limitation: F5: growth measured as a count; bytes not attributed
- E22: limitation: F8: growth evidence is from S3, not S1; not subtracted
- E22: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E22: limitation: threshold is provisional (not calibrated)
- E23: partial coverage (2 of 3 examined)
- E23: limitation: F5: growth evidence is from S1, not S2; not subtracted
- E23: limitation: F8: growth evidence is from S3, not S2; not subtracted
- E23: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E23: limitation: threshold is provisional (not calibrated)
- E24: partial coverage (2 of 3 examined)
- E24: limitation: F5: growth evidence is from S1, not S3; not subtracted
- E24: limitation: F8: growth measured as a count; bytes not attributed
- E24: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E24: limitation: threshold is provisional (not calibrated)
- E25: partial coverage (2 of 3 examined)
- E25: limitation: F5: growth evidence is from S1, not S4; not subtracted
- E25: limitation: F8: growth evidence is from S3, not S4; not subtracted
- E25: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E25: limitation: threshold is provisional (not calibrated)
- E26: partial coverage (2 of 3 examined)
- E26: limitation: F5: growth evidence is from S1, not S5; not subtracted
- E26: limitation: F8: growth evidence is from S3, not S5; not subtracted
- E26: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E26: limitation: threshold is provisional (not calibrated)
- E27: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E27: limitation: weak edges are skipped; string contents are never read into the summary
- E28: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E28: limitation: weak edges are skipped; string contents are never read into the summary
- E29: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E29: limitation: weak edges are skipped; string contents are never read into the summary
- E30: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E30: limitation: weak edges are skipped; string contents are never read into the summary
- E31: limitation: dump-series rule: at least 3 snapshots, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E31: limitation: series: per-point; one process per point at n = 440, 1760, 2640, 3960 (harness captures; checked: snapshot format)
- E32: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E33: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E34: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E35: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E35: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E35: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E35: limitation: gate thresholds are provisional (not calibrated)
- E36: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E36: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E36: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E36: limitation: gate thresholds are provisional (not calibrated)
- E37: partial coverage (2 of 3 examined)
- E37: limitation: F5: growth measured as a count; bytes not attributed
- E37: limitation: F8: growth evidence is from S3, not S1; not subtracted
- E37: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E37: limitation: threshold is provisional (not calibrated)
- E38: partial coverage (2 of 3 examined)
- E38: limitation: F5: growth measured as a count; bytes not attributed
- E38: limitation: F8: growth evidence is from S3, not S1; not subtracted
- E38: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E38: limitation: threshold is provisional (not calibrated)

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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (bdc9370640c4) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (62f2c0d175b1) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (62f2c0d175b1) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (4dfb4fd74764) | alloc/A1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S2.json (ad09b46c263e) | trend/T3.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (ad09b46c263e) | trend/T3.json |
| E15 | growth-series | test | dla 0.1.0 | artifact series/S3.json (4dfb4fd74764) | trend/T4.json |
| E16 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (4dfb4fd74764) | trend/T4.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S5.json (3394b61636a0) | alloc/A2.json |
| E18 | growth-series | test | dla 0.1.0 | artifact series/S4.json (c750c4061085) | trend/T6.json |
| E19 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (c750c4061085) | trend/T6.json |
| E20 | growth-series | test | dla 0.1.0 | artifact series/S5.json (3394b61636a0) | trend/T7.json |
| E21 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (3394b61636a0) | trend/T7.json |
| E22 | residual | test | dla 0.1.0 | artifact series/S1.json (62f2c0d175b1) | gap/residual-S1-E22.json |
| E23 | residual | test | dla 0.1.0 | artifact series/S2.json (ad09b46c263e) | gap/residual-S2-E23.json |
| E24 | residual | test | dla 0.1.0 | artifact series/S3.json (4dfb4fd74764) | gap/residual-S3-E24.json |
| E25 | residual | test | dla 0.1.0 | artifact series/S4.json (c750c4061085) | gap/residual-S4-E25.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S5.json (3394b61636a0) | gap/residual-S5-E26.json |
| E27 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-170228-5697/harness/H1/out/p1-native/capture/9/heap-1.heapsnapshot (a36a72c6a6f9) | summaries/heap-1-a36a72c6.json |
| E28 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-170228-5697/harness/H1/out/p4-native/capture/34/heap-1.heapsnapshot (0d30e6331ef1) | summaries/heap-1-0d30e633.json |
| E29 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-170228-5697/harness/H1/out/p6-native/capture/50/heap-1.heapsnapshot (09e90e85429d) | summaries/heap-1-09e90e85.json |
| E30 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-170228-5697/harness/H1/out/p9-native/capture/74/heap-1.heapsnapshot (da213044e424) | summaries/heap-1-da213044.json |
| E31 | growth-series | dump | dla-v8-snapshot 0.1.0 | artifact series/S6.json (6cef75c4c53a) | series/S6.json |
| E32 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S6.json (6cef75c4c53a) | series/S6.json |
| E33 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S6.json (6cef75c4c53a) | series/S6.json |
| E34 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S6.json (6cef75c4c53a) | series/S6.json |
| E35 | growth-series | dump | dla 0.1.0 | artifact series/S6.json (6cef75c4c53a) | trend/T8.json |
| E36 | unit-count-series | dump | dla 0.1.0 | artifact series/S6.json (6cef75c4c53a) | trend/T8.json |
| E37 | residual | test | dla 0.1.0 | artifact series/S1.json (62f2c0d175b1) | gap/residual-S1-E37.json |
| E38 | residual | test | dla 0.1.0 | artifact series/S1.json (62f2c0d175b1) | gap/residual-S1-E38.json |
