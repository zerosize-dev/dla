# DLA report

Run: `20261110-200015-16af`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 12 enumerated, 12 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 10

Residual: heap 10112.75090909091 bytes/n unattributed of 10112.75090909091 total (threshold 2528.187727272727) ABOVE THRESHOLD, see O1; heap 10104 bytes/n unattributed of 10104 total (threshold 2526) ABOVE THRESHOLD, see O1; heap 10104.004090909091 bytes/n unattributed of 10104.004090909091 total (threshold 2526.001022727273) ABOVE THRESHOLD, see O2; off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0)

Coverage: node 3520 iterations, 10 of 11 entry points exercised

## Findings

### F5: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Board.publish: changeNote closure captures 'previous' (this.latest) and snapshot.describe closure captures 'snapshot'; V8 allocates one shared context for both, so each live snapshot's describe retains the context holding previous -> chain of every snapshot and its ticks/averages. Retention path: module exports.publish.context -> Board.latest -> snapshot -> describe -> context -> previous ... Closure-capture pattern, label not accepted by ledger (retention-path not tied to F5's owner).)
- Growth: `holder:closure:.dla.20261110-200015-16af.harness.H1.work.src.board.previous` 11855360 bytes/n (steady, gate pass, E16)
- Evidence classes: context, discriminator, growth, retention
- Holder: closure/continuation
- Owner: `.dla.20261110-200015-16af.harness.H1.work.src.board.previous`
- Region: heap
- Shape: steady
- Site: `src/board.js:13`
- Writers: `src/board.js:12`
- Annotation: previous (this.latest) captured by changeNote closure; shares V8 context with snapshot.describe, so every snapshot retains its predecessor -> unbounded chain
- Verified: DISAGREES (label: finding UNCLASSIFIED vs verifier L-CAPTURE; bundle e1ef814b4971); finding mechanism: src/board.js:12 stores previous=this.latest; the changeNote closure at src/board.js:13 captures it and describe (:26) shares its V8 context, so Board.latest -> describe -> context -> previous chains every snapshot; removal never happens.; verifier mechanism: publish() creates changeNote closure capturing previous; V8 shares one context with the snapshot's describe() closure, so each retained snapshot keeps its previous snapshot (and ticks) alive via Board.latest -> describe -> context -> previous, forming an unbounded chain even when verbose is off
- Sibling hypotheses: F5.n1
- Supporting evidence: E15, E9, E16, E22
- Explanation (observed): src/board.js:12 stores previous=this.latest; the changeNote closure at src/board.js:13 captures it and describe (:26) shares its V8 context, so Board.latest -> describe -> context -> previous chains every snapshot; removal never happens.
- Explanation structure: adds: src/board.js:12; removal: never; evidence: E16

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.board.Board.latest`
- Region: heap
- Site: `src/board.js:7`
- Writers: `src/board.js:31`
- Removers: `src/board.js:40`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Not independently confirmed; growth is a downstream column of the F5 closure-context snapshot chain (Board.publish, src/board.js:13). Fixing F5 would show whether it remains.
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.feed.Feed.random`
- Region: heap
- Site: `src/feed.js:17`
- Reason: not_exercised
- Remedy: Not independently confirmed; growth is a downstream column of the F5 closure-context snapshot chain (Board.publish, src/board.js:13). Fixing F5 would show whether it remains.
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.feed.Feed.prices`
- Region: heap
- Site: `src/feed.js:19`
- Writers: `src/feed.js:30`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Not independently confirmed; growth is a downstream column of the F5 closure-context snapshot chain (Board.publish, src/board.js:13). Fixing F5 would show whether it remains.
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: retention
- Holder: closure/continuation
- Owner: `.dla.20261110-200015-16af.harness.H1.work.src.board.board`
- Region: heap
- Reason: not_exercised
- Remedy: Not independently confirmed; growth is a downstream column of the F5 closure-context snapshot chain (Board.publish, src/board.js:13). Fixing F5 would show whether it remains.
- Supporting evidence: E15

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: Not independently confirmed; growth is a downstream column of the F5 closure-context snapshot chain (Board.publish, src/board.js:13). Fixing F5 would show whether it remains.
- Supporting evidence: E34

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: Not independently confirmed; growth is a downstream column of the F5 closure-context snapshot chain (Board.publish, src/board.js:13). Fixing F5 would show whether it remains.
- Supporting evidence: E35

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: Not independently confirmed; growth is a downstream column of the F5 closure-context snapshot chain (Board.publish, src/board.js:13). Fixing F5 would show whether it remains.
- Supporting evidence: E35

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: Not independently confirmed; growth is a downstream column of the F5 closure-context snapshot chain (Board.publish, src/board.js:13). Fixing F5 would show whether it remains.
- Supporting evidence: E35

### F10: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: Not independently confirmed; growth is a downstream column of the F5 closure-context snapshot chain (Board.publish, src/board.js:13). Fixing F5 would show whether it remains.
- Supporting evidence: E38

### F11: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: Not independently confirmed; growth is a downstream column of the F5 closure-context snapshot chain (Board.publish, src/board.js:13). Fixing F5 would show whether it remains.
- Supporting evidence: E39

## Ruled out

- **F5.n1** (UNCLASSIFIED): discriminator E22
  - Ruled out by: plateau+purge (leak) E22
  - Hypothesis: N-WARM: growth is cache warm-up/fill that plateaus
  - Refuted if: dump-plateau / alloc-test shows retained count keeps growing after extra cycles and purge
  - Contradicting evidence: E22
  - Originally supported by: E16

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
- E9: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E10: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E10: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E10: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E11: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
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
- E20: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E21: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E22: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E22: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E22: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E22: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E22: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E22: limitation: points 1 to 9 come from S1; the extra points from S3
- E22: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E22: limitation: gate and plateau thresholds are provisional (not calibrated)
- E23: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E23: limitation: weak edges are skipped; string contents are never read into the summary
- E24: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E24: limitation: weak edges are skipped; string contents are never read into the summary
- E25: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E25: limitation: weak edges are skipped; string contents are never read into the summary
- E26: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E26: limitation: weak edges are skipped; string contents are never read into the summary
- E27: limitation: dump-series rule: at least 3 snapshots, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E27: limitation: series: per-point; one process per point at n = 440, 1760, 2640, 3960 (harness captures; checked: snapshot format)
- E28: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E29: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E30: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E31: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E32: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E33: limitation: a sampling heap profile: counts and bytes are estimates, and small allocations may never be sampled
- E33: limitation: each profile covers the objects allocated in its own window that are still alive when it is read; later windows do not hold earlier ones
- E33: limitation: samples in native builtins are charged to the nearest JavaScript frame
- E33: limitation: the profiles are ordered as given
- E33: limitation: the profiles are separate windows, not cumulative: a rise between windows is not retention growth, so no growth column is built from them
- E33: limitation: the profile is sampled: counts and bytes are estimates
- E34: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E34: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E34: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E34: limitation: gate thresholds are provisional (not calibrated)
- E35: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E35: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E35: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E35: limitation: gate thresholds are provisional (not calibrated)
- E36: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E36: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E36: limitation: gate thresholds are provisional (not calibrated)
- E37: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E37: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E37: limitation: gate thresholds are provisional (not calibrated)
- E38: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E38: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E38: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E38: limitation: gate thresholds are provisional (not calibrated)
- E39: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E39: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E39: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E39: limitation: gate thresholds are provisional (not calibrated)
- E40: partial coverage (2 of 3 examined)
- E40: limitation: F5: growth evidence is from S2, not S1; not subtracted
- E40: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E40: limitation: threshold is provisional (not calibrated)
- E41: partial coverage (2 of 3 examined)
- E41: limitation: F5: growth evidence is from S2, not S3; not subtracted
- E41: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E41: limitation: threshold is provisional (not calibrated)
- E42: partial coverage (2 of 3 examined)
- E42: limitation: F5: growth evidence is from S2, not S4; not subtracted
- E42: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
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
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E1 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo d42e192d932296cb766bd14caf0ede25d1806902 | — |
| E2 | run-config | static | dla 0.1.0 | repo d42e192d932296cb766bd14caf0ede25d1806902 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo d42e192d932296cb766bd14caf0ede25d1806902 | probe/probe.json |
| E4 | static-hit | static | eslint 10.14.0 | repo d42e192d932296cb766bd14caf0ede25d1806902 | sweep/node-eslint.json |
| E5 | static-hit | static | dla 0.1.0 | repo d42e192d932296cb766bd14caf0ede25d1806902 | sweep/node-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo d42e192d932296cb766bd14caf0ede25d1806902 | census/node.json |
| E7 | workload | static | dla 0.1.0 | repo d42e192d932296cb766bd14caf0ede25d1806902 | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo d42e192d932296cb766bd14caf0ede25d1806902 | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (782150325272) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (20089b83dc96) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (20089b83dc96) | trend/T1.json |
| E12 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-200015-16af/harness/H1/out/p1-native/capture/9/heap-1.heapsnapshot (5e3251ea2320) | summaries/heap-1-5e3251ea.json |
| E13 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-200015-16af/harness/H1/out/p4-native/capture/34/heap-1.heapsnapshot (e150a4bdbc38) | summaries/heap-1-e150a4bd.json |
| E14 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-200015-16af/harness/H1/out/p6-native/capture/50/heap-1.heapsnapshot (a15424292cea) | summaries/heap-1-a1542429.json |
| E15 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-200015-16af/harness/H1/out/p9-native/capture/74/heap-1.heapsnapshot (ff43eaca8b4c) | summaries/heap-1-ff43eaca.json |
| E16 | growth-series | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (13deb0cfd493) | series/S2.json |
| E17 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (13deb0cfd493) | series/S2.json |
| E18 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (13deb0cfd493) | series/S2.json |
| E19 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (13deb0cfd493) | series/S2.json |
| E20 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (13deb0cfd493) | series/S2.json |
| E21 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (13deb0cfd493) | series/S2.json |
| E22 | discriminator | test | dla 0.1.0 | artifact series/S4.json (26b8e36d5b91) | alloc/A1.json |
| E23 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-200015-16af/harness/H1/out/p1-native/capture/9/heap-1.heapsnapshot (5e3251ea2320) | summaries/heap-1-5e3251ea.json |
| E24 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-200015-16af/harness/H1/out/p4-native/capture/34/heap-1.heapsnapshot (e150a4bdbc38) | summaries/heap-1-e150a4bd.json |
| E25 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-200015-16af/harness/H1/out/p6-native/capture/50/heap-1.heapsnapshot (a15424292cea) | summaries/heap-1-a1542429.json |
| E26 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-200015-16af/harness/H1/out/p9-native/capture/74/heap-1.heapsnapshot (ff43eaca8b4c) | summaries/heap-1-ff43eaca.json |
| E27 | growth-series | dump | dla-v8-snapshot 0.1.0 | artifact series/S5.json (f18ce93cc152) | series/S5.json |
| E28 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S5.json (f18ce93cc152) | series/S5.json |
| E29 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S5.json (f18ce93cc152) | series/S5.json |
| E30 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S5.json (f18ce93cc152) | series/S5.json |
| E31 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S5.json (f18ce93cc152) | series/S5.json |
| E32 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S5.json (f18ce93cc152) | series/S5.json |
| E33 | alloc-stack | live | node-heapprofile 0.1.0 | artifact allocs/A1.json (c3bc7029718f) | allocs/A1.json |
| E34 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (13deb0cfd493) | trend/T3.json |
| E35 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (13deb0cfd493) | trend/T3.json |
| E36 | growth-series | test | dla 0.1.0 | artifact series/S3.json (b5d06514c08a) | trend/T4.json |
| E37 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (b5d06514c08a) | trend/T4.json |
| E38 | growth-series | dump | dla 0.1.0 | artifact series/S5.json (f18ce93cc152) | trend/T5.json |
| E39 | unit-count-series | dump | dla 0.1.0 | artifact series/S5.json (f18ce93cc152) | trend/T5.json |
| E40 | residual | test | dla 0.1.0 | artifact series/S1.json (20089b83dc96) | gap/residual-S1-E40.json |
| E41 | residual | test | dla 0.1.0 | artifact series/S3.json (b5d06514c08a) | gap/residual-S3-E41.json |
| E42 | residual | test | dla 0.1.0 | artifact series/S4.json (26b8e36d5b91) | gap/residual-S4-E42.json |
