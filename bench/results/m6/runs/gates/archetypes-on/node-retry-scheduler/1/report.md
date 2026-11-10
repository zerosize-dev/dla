# DLA report

Run: `20261110-162923-aa0b`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 10 enumerated, 10 dispositioned; confirmed 1, strong 0, refuted 4, suspected 0, unresolved 5

Residual: heap 4288.30303030303 bytes/n unattributed of 4288.30303030303 total (threshold 1072.0757575757575) ABOVE THRESHOLD, see O1; heap 4271.215909090909 bytes/n unattributed of 4271.215909090909 total (threshold 1067.8039772727273) ABOVE THRESHOLD, see O4; heap 4265.522727272728 bytes/n unattributed of 4265.522727272728 total (threshold 1066.380681818182) ABOVE THRESHOLD, see O2; heap 4270.592424242424 bytes/n unattributed of 4270.592424242424 total (threshold 1067.648106060606) ABOVE THRESHOLD, see O2; heap 4266.345454545455 bytes/n unattributed of 4266.345454545455 total (threshold 1066.5863636363638) ABOVE THRESHOLD, see O2; heap 4271.975064935064 bytes/n unattributed of 4271.975064935064 total (threshold 1067.993766233766) ABOVE THRESHOLD, see O2; heap 4265.128787878788 bytes/n unattributed of 4265.128787878788 total (threshold 1066.282196969697) ABOVE THRESHOLD, see O2; heap 4271.353030303031 bytes/n unattributed of 4271.353030303031 total (threshold 1067.8382575757578) ABOVE THRESHOLD, see O3; heap 4263.746969696969 bytes/n unattributed of 4263.746969696969 total (threshold 1065.9367424242423) ABOVE THRESHOLD, see O2; off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0)

Coverage: node 3520 iterations, 17 of 19 entry points exercised

## Findings

### F5: **L-STALE**

- State: confirmed
- Label: L-STALE (evidence: E16 growth-series, E12 retention-path)
- Growth: `holder:field:.dla.20261110-162923-aa0b.harness.H1.work.src.scheduler.RetryScheduler.latest` 5011275 bytes/n (steady, gate pass, E16)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `src.scheduler.RetryScheduler.latest`
- Region: heap
- Shape: steady
- Site: `src/scheduler.js:15`
- Writers: `src/scheduler.js:32`
- Annotation: heuristic name match over the project's sources
- Annotation: latest advances on every enqueue; record.previous chains each attempt to the one before, never cut
- Annotation: enqueue() links each attempt record to the previous one via record.previous and advances this.latest; nothing ever cuts the chain, so every record and its job payload stays reachable from the shared scheduler
- Verified: no
- Sibling hypotheses: F5.n1
- Supporting evidence: E6, E9, E16, E11
- Explanation (observed): RetryScheduler.latest (src/scheduler.js:15) is advanced on every enqueue (scheduler.js:32) and each record keeps record.previous, so all finished attempts and their jobs stay reachable from the shared scheduler; the chain is never cut and nothing removes it.
- Explanation structure: adds: src/scheduler.js:32; removal: never; evidence: E16

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `src/scheduler.js:32`
- Reason: not_exercised
- Remedy: Duplicate or consequence of F5 (the unbounded record.previous chain from RetryScheduler.latest); not a separate leak. Fix F5.
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
- Remedy: PendingList head/tail: pending list drains (alloc-test of F4 plateau); not exercised as a separate holder.
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
- Remedy: PendingList head/tail: pending list drains (alloc-test of F4 plateau); not exercised as a separate holder.
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
- Remedy: F8 is only the module-level root (sharedScheduler singleton) through which F5's chain is reachable; not an independent leak. Fix F5's previous chain; do not evaluate separately.
- Supporting evidence: E6

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: Duplicate or consequence of F5 (the unbounded record.previous chain from RetryScheduler.latest); not a separate leak. Fix F5.
- Supporting evidence: E21, E22

## Ruled out

- **F4** (UNCLASSIFIED): discriminator E25
  - Ruled out by: plateau+purge (non-leak) E25
  - Contradicting evidence: E25
  - Originally supported by: E6
- **F5.n1** (UNCLASSIFIED): discriminator E20
  - Ruled out by: plateau+purge (leak) E20
  - Hypothesis: N-WARM: growth is a warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles and dump-plateau favors leak
  - Contradicting evidence: E20
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
- E17: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E18: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E19: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E20: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E20: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E20: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E20: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E20: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E20: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E20: limitation: points 1 to 9 come from S1; the extra points from S3
- E20: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E20: limitation: gate and plateau thresholds are provisional (not calibrated)
- E21: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E21: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E21: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E22: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E22: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E23: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E24: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
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
- E30: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E30: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E30: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E30: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E30: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E30: limitation: points 1 to 9 come from S1; the extra points from S3
- E30: limitation: gate thresholds are provisional (not calibrated)
- E31: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E31: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E31: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E31: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E31: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E31: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E31: limitation: points 1 to 9 come from S1; the extra points from S3
- E31: limitation: gate thresholds are provisional (not calibrated)
- E32: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E32: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E32: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E32: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E32: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E32: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E32: limitation: points 1 to 9 come from S1; the extra points from S5
- E32: limitation: gate thresholds are provisional (not calibrated)
- E33: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E33: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E33: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E33: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E33: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E33: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E33: limitation: points 1 to 9 come from S1; the extra points from S5
- E33: limitation: gate thresholds are provisional (not calibrated)
- E34: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E34: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E34: limitation: gate thresholds are provisional (not calibrated)
- E35: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E35: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E35: limitation: gate thresholds are provisional (not calibrated)
- E36: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E36: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E36: limitation: gate thresholds are provisional (not calibrated)
- E37: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E37: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E37: limitation: gate thresholds are provisional (not calibrated)
- E38: partial coverage (2 of 3 examined)
- E38: limitation: F5: growth measured as a count; bytes not attributed
- E38: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E38: limitation: threshold is provisional (not calibrated)
- E39: partial coverage (2 of 3 examined)
- E39: limitation: F5: growth evidence is from S2, not S3; not subtracted
- E39: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E39: limitation: threshold is provisional (not calibrated)
- E40: partial coverage (2 of 3 examined)
- E40: limitation: F5: growth evidence is from S2, not S4; not subtracted
- E40: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E40: limitation: threshold is provisional (not calibrated)
- E41: partial coverage (2 of 3 examined)
- E41: limitation: F5: growth evidence is from S2, not S5; not subtracted
- E41: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E41: limitation: threshold is provisional (not calibrated)
- E42: partial coverage (2 of 3 examined)
- E42: limitation: F5: growth evidence is from S2, not S6; not subtracted
- E42: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E42: limitation: threshold is provisional (not calibrated)
- E43: partial coverage (2 of 3 examined)
- E43: limitation: F5: growth evidence is from S2, not S7; not subtracted
- E43: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E43: limitation: threshold is provisional (not calibrated)
- E44: partial coverage (2 of 3 examined)
- E44: limitation: F5: growth evidence is from S2, not S9; not subtracted
- E44: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E44: limitation: threshold is provisional (not calibrated)
- E45: partial coverage (2 of 3 examined)
- E45: limitation: F5: growth evidence is from S2, not S8; not subtracted
- E45: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E45: limitation: threshold is provisional (not calibrated)
- E46: partial coverage (2 of 3 examined)
- E46: limitation: F5: growth evidence is from S2, not S10; not subtracted
- E46: limitation: off-heap: no gap series for S10; run dla gap S10 first (node.external is used when present)
- E46: limitation: threshold is provisional (not calibrated)

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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (131e0b8122bd) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (c2cb30003d45) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (c2cb30003d45) | trend/T1.json |
| E12 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-162923-aa0b/harness/H1/out/p1-native/capture/9/heap-1.heapsnapshot (619018b2ba2f) | summaries/heap-1-619018b2.json |
| E13 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-162923-aa0b/harness/H1/out/p4-native/capture/34/heap-1.heapsnapshot (03c822b82a77) | summaries/heap-1-03c822b8.json |
| E14 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-162923-aa0b/harness/H1/out/p6-native/capture/50/heap-1.heapsnapshot (50f2c6a401a0) | summaries/heap-1-50f2c6a4.json |
| E15 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-162923-aa0b/harness/H1/out/p9-native/capture/74/heap-1.heapsnapshot (e7caa222d488) | summaries/heap-1-e7caa222.json |
| E16 | growth-series | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (5beb32bacf95) | series/S2.json |
| E17 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (5beb32bacf95) | series/S2.json |
| E18 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (5beb32bacf95) | series/S2.json |
| E19 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (5beb32bacf95) | series/S2.json |
| E20 | discriminator | test | dla 0.1.0 | artifact series/S4.json (984854569da3) | alloc/A1.json |
| E21 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (5beb32bacf95) | trend/T3.json |
| E22 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (5beb32bacf95) | trend/T3.json |
| E23 | growth-series | test | dla 0.1.0 | artifact series/S3.json (00e85df52a27) | trend/T4.json |
| E24 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (00e85df52a27) | trend/T4.json |
| E25 | discriminator | test | dla 0.1.0 | artifact series/S6.json (bc5a22afb67b) | alloc/A2.json |
| E26 | discriminator | test | dla 0.1.0 | artifact series/S8.json (ff11d6ef542f) | alloc/A3.json |
| E27 | discriminator | test | dla 0.1.0 | artifact series/S10.json (33344e1aa9cd) | alloc/A4.json |
| E28 | growth-series | test | dla 0.1.0 | artifact series/S5.json (ac883eea9365) | trend/T8.json |
| E29 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (ac883eea9365) | trend/T8.json |
| E30 | growth-series | test | dla 0.1.0 | artifact series/S4.json (984854569da3) | trend/T9.json |
| E31 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (984854569da3) | trend/T9.json |
| E32 | growth-series | test | dla 0.1.0 | artifact series/S6.json (bc5a22afb67b) | trend/T10.json |
| E33 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (bc5a22afb67b) | trend/T10.json |
| E34 | growth-series | test | dla 0.1.0 | artifact series/S7.json (03268c925613) | trend/T11.json |
| E35 | unit-count-series | test | dla 0.1.0 | artifact series/S7.json (03268c925613) | trend/T11.json |
| E36 | growth-series | test | dla 0.1.0 | artifact series/S9.json (89e4f721f150) | trend/T12.json |
| E37 | unit-count-series | test | dla 0.1.0 | artifact series/S9.json (89e4f721f150) | trend/T12.json |
| E38 | residual | test | dla 0.1.0 | artifact series/S1.json (c2cb30003d45) | gap/residual-S1-E38.json |
| E39 | residual | test | dla 0.1.0 | artifact series/S3.json (00e85df52a27) | gap/residual-S3-E39.json |
| E40 | residual | test | dla 0.1.0 | artifact series/S4.json (984854569da3) | gap/residual-S4-E40.json |
| E41 | residual | test | dla 0.1.0 | artifact series/S5.json (ac883eea9365) | gap/residual-S5-E41.json |
| E42 | residual | test | dla 0.1.0 | artifact series/S6.json (bc5a22afb67b) | gap/residual-S6-E42.json |
| E43 | residual | test | dla 0.1.0 | artifact series/S7.json (03268c925613) | gap/residual-S7-E43.json |
| E44 | residual | test | dla 0.1.0 | artifact series/S9.json (89e4f721f150) | gap/residual-S9-E44.json |
| E45 | residual | test | dla 0.1.0 | artifact series/S8.json (ff11d6ef542f) | gap/residual-S8-E45.json |
| E46 | residual | test | dla 0.1.0 | artifact series/S10.json (33344e1aa9cd) | gap/residual-S10-E46.json |
