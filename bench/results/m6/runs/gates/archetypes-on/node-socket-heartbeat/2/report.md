# DLA report

Run: `20261110-170838-1982`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 17 enumerated, 17 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 15

Residual: heap 5100.68 bytes/n unattributed of 5100.68 total (threshold 1275.17) ABOVE THRESHOLD, see O1; heap 4978.381060606061 bytes/n unattributed of 4978.381060606061 total (threshold 1244.5952651515152) ABOVE THRESHOLD, see O1; heap 5039.125757575757 bytes/n unattributed of 5039.125757575757 total (threshold 1259.7814393939393) ABOVE THRESHOLD, see O1; off-heap 2544.8772727272726 bytes/n unattributed of 2544.8772727272726 total (threshold 3188.545506818182); off-heap 6144 bytes/n unattributed of 6144 total (threshold 1536) ABOVE THRESHOLD, see O2; off-heap 6144 bytes/n unattributed of 6144 total (threshold 1536) ABOVE THRESHOLD, see O2

Coverage: node 3520 iterations, 25 of 30 entry points exercised

## Findings

### F1: **L-REGISTER**

- State: confirmed
- Label: L-REGISTER (evidence: E11 unit-count-series)
- Growth: `node.timers@src/heartbeat.js:15` 1 count/n (steady, gate pass, E11)
- Evidence classes: context, growth, static
- Holder: registration-table
- Region: non-heap
- Shape: steady
- Site: `src/heartbeat.js:15`
- Writers: `src/heartbeat.js:15`
- Annotation: setInterval per session never cleared in Session.close; timer closure retains session, socket, recent buffer
- Verified: no
- Sibling hypotheses: F1.n1
- Suggested archetype (unconfirmed): L-WORKER
- Supporting evidence: E5, E9, E11
- Explanation (observed): The registration-table of per-session setInterval timers (src/heartbeat.js:15) is only added to: Session.close (src/session.js) and Gateway.release never clearInterval, so each closed session, its socket and listeners stay reachable.
- Explanation structure: adds: src/heartbeat.js:15; removal: never; evidence: E17

### F2: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.metrics`
- Region: heap
- Site: `src/gateway.js:13`
- Reason: time_budget
- Remedy: census lead not tied to the leak; bounded or consequence of F1 (F14 capped at 64, F8 is the in-process loopback test socket, F15 is DLA's own agent). Re-check after fixing F1.
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.router`
- Region: heap
- Site: `src/gateway.js:14`
- Reason: time_budget
- Remedy: census lead not tied to the leak; bounded or consequence of F1 (F14 capped at 64, F8 is the in-process loopback test socket, F15 is DLA's own agent). Re-check after fixing F1.
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.sessions`
- Region: heap
- Site: `src/gateway.js:15`
- Writers: `src/gateway.js:22`
- Removers: `src/gateway.js:32`
- Annotation: heuristic name match over the project's sources
- Reason: time_budget
- Remedy: census lead not tied to the leak; bounded or consequence of F1 (F14 capped at 64, F8 is the in-process loopback test socket, F15 is DLA's own agent). Re-check after fixing F1.
- Supporting evidence: E6

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `src.gateway.Gateway.accept`
- Region: heap
- Site: `src/gateway.js:24`
- Writers: `src/gateway.js:25`, `src/session.js:12`, `src/session.js:54`
- Removers: `load.js:50`, `src/gateway.js:31`, `src/session.js:66`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: listener count on socket at gateway.js:24 is a consequence of F1 retaining sockets; fix F1 and re-measure
- Supporting evidence: E6

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `src.gateway.Gateway.accept`
- Region: heap
- Site: `src/gateway.js:25`
- Writers: `src/gateway.js:24`, `src/session.js:12`, `src/session.js:54`
- Removers: `load.js:50`, `src/gateway.js:31`, `src/session.js:66`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: same as F5: listener on retained socket, consequence of F1
- Supporting evidence: E6

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `src.heartbeat.startHeartbeat`
- Region: non-heap
- Site: `src/heartbeat.js:15`
- Reason: not_exercised
- Remedy: duplicate census view of F1 (same timer at src/heartbeat.js:15); covered by F1
- Supporting evidence: E6

### F8: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: container-field
- Owner: `src.loopback.LoopbackSocket.written`
- Region: heap
- Site: `src/loopback.js:11`
- Writers: `src/loopback.js:17`
- Annotation: heuristic name match over the project's sources
- Reason: time_budget
- Remedy: census lead not tied to the leak; bounded or consequence of F1 (F14 capped at 64, F8 is the in-process loopback test socket, F15 is DLA's own agent). Re-check after fixing F1.
- Supporting evidence: E6

### F9: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: container-field
- Owner: `src.metrics.Metrics.counters`
- Region: heap
- Site: `src/metrics.js:7`
- Writers: `src/metrics.js:12`
- Annotation: heuristic name match over the project's sources
- Reason: time_budget
- Remedy: census lead not tied to the leak; bounded or consequence of F1 (F14 capped at 64, F8 is the in-process loopback test socket, F15 is DLA's own agent). Re-check after fixing F1.
- Supporting evidence: E6

### F10: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: container-field
- Owner: `src.router.Router.topics`
- Region: heap
- Site: `src/router.js:9`
- Writers: `src/router.js:16`, `src/router.js:19`
- Removers: `src/router.js:26`, `src/router.js:27`
- Annotation: heuristic name match over the project's sources
- Reason: time_budget
- Remedy: census lead not tied to the leak; bounded or consequence of F1 (F14 capped at 64, F8 is the in-process loopback test socket, F15 is DLA's own agent). Re-check after fixing F1.
- Supporting evidence: E6

### F11: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: registration-table
- Owner: `src.router.Router.dispatch`
- Region: heap
- Site: `src/router.js:51`
- Writers: `src/session.js:38`
- Removers: `src/gateway.js:25`, `src/router.js:54`, `src/session.js:33`
- Annotation: heuristic name match over the project's sources
- Reason: time_budget
- Remedy: census lead not tied to the leak; bounded or consequence of F1 (F14 capped at 64, F8 is the in-process loopback test socket, F15 is DLA's own agent). Re-check after fixing F1.
- Supporting evidence: E6

### F12: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.decoder`
- Region: heap
- Site: `src/session.js:15`
- Writers: `src/session.js:30`
- Annotation: heuristic name match over the project's sources
- Reason: time_budget
- Remedy: census lead not tied to the leak; bounded or consequence of F1 (F14 capped at 64, F8 is the in-process loopback test socket, F15 is DLA's own agent). Re-check after fixing F1.
- Supporting evidence: E6

### F13: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.topics`
- Region: heap
- Site: `src/session.js:19`
- Reason: time_budget
- Remedy: census lead not tied to the leak; bounded or consequence of F1 (F14 capped at 64, F8 is the in-process loopback test socket, F15 is DLA's own agent). Re-check after fixing F1.
- Supporting evidence: E6

### F14: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.recent`
- Region: heap
- Site: `src/session.js:20`
- Writers: `src/session.js:44`
- Removers: `src/session.js:45`
- Annotation: heuristic name match over the project's sources
- Reason: time_budget
- Remedy: census lead not tied to the leak; bounded or consequence of F1 (F14 capped at 64, F8 is the in-process loopback test socket, F15 is DLA's own agent). Re-check after fixing F1.
- Supporting evidence: E6

### F15: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: retention
- Holder: static/global
- Owner: `.dla.20261110-170838-1982.harness.H1.agent.dla-preload.listenerRefs`
- Region: heap
- Reason: time_budget
- Remedy: census lead not tied to the leak; bounded or consequence of F1 (F14 capped at 64, F8 is the in-process loopback test socket, F15 is DLA's own agent). Re-check after fixing F1.
- Supporting evidence: E21

### F16: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: same Timeout@src/heartbeat.js:15 growth as F1
- Supporting evidence: E26

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM: timers count plateaus after warm-up
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
- E15: limitation: sites are the first project frame of the stack that created the resource; a resource with no project frame is unattributed
- E15: limitation: active counts come from getActiveResourcesInfo and leave out unref'd timers
- E15: limitation: the files are ordered as given; each is one point
- E16: limitation: sites are the first project frame of the stack that created the resource; a resource with no project frame is unattributed
- E16: limitation: active counts come from getActiveResourcesInfo and leave out unref'd timers
- E16: limitation: the files are ordered as given; each is one point
- E17: limitation: sites are the first project frame of the stack that created the resource; a resource with no project frame is unattributed
- E17: limitation: active counts come from getActiveResourcesInfo and leave out unref'd timers
- E17: limitation: the files are ordered as given; each is one point
- E17: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E18: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E18: limitation: weak edges are skipped; string contents are never read into the summary
- E19: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E19: limitation: weak edges are skipped; string contents are never read into the summary
- E20: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E20: limitation: weak edges are skipped; string contents are never read into the summary
- E21: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E21: limitation: weak edges are skipped; string contents are never read into the summary
- E22: limitation: dump-series rule: at least 3 snapshots, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E22: limitation: series: per-point; one process per point at n = 440, 1760, 2640, 3960 (harness captures; checked: snapshot format)
- E23: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E24: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E25: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E26: limitation: x is the snapshot number in capture order; t_ms is the time since the first capture where the inputs record one
- E26: limitation: each point comes from a separate snapshot of one process; a figure absent from a snapshot has no value at that point
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E27: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E27: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E28: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E28: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: partial coverage (2 of 3 examined)
- E29: limitation: F1: growth measured as a count; bytes not attributed
- E29: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E29: limitation: threshold is provisional (not calibrated)
- E30: partial coverage (2 of 3 examined)
- E30: limitation: F1: growth evidence is from S1, not S2; not subtracted
- E30: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E30: limitation: threshold is provisional (not calibrated)
- E31: partial coverage (2 of 3 examined)
- E31: limitation: F1: growth evidence is from S1, not S3; not subtracted
- E31: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E31: limitation: threshold is provisional (not calibrated)
- E32: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E32: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E32: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E32: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E32: limitation: gate thresholds are provisional (not calibrated)
- E33: partial coverage (2 of 3 examined)
- E33: limitation: F1: growth measured as a count; bytes not attributed
- E33: limitation: O2 (off-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E33: limitation: threshold is provisional (not calibrated)
- E34: partial coverage (2 of 3 examined)
- E34: limitation: F1: growth evidence is from S1, not S2; not subtracted
- E34: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E34: limitation: threshold is provisional (not calibrated)
- E35: partial coverage (2 of 3 examined)
- E35: limitation: F1: growth evidence is from S1, not S3; not subtracted
- E35: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E35: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | — |
| E2 | run-config | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | probe/probe.json |
| E4 | static-hit | static | eslint 10.14.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | sweep/node-eslint.json |
| E5 | static-hit | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | sweep/node-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | census/node.json |
| E7 | workload | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (17442449eb81) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (e14d09bfc75a) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (e14d09bfc75a) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (65ae050baf07) | alloc/A1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S2.json (de22d939ef89) | trend/T3.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (de22d939ef89) | trend/T3.json |
| E15 | non-heap | live | node-resources 0.1.0 | artifact nonheap/N1.json (d742e865b174) | nonheap/N1.json |
| E16 | non-heap | live | node-resources 0.1.0 | artifact nonheap/N2.json (f012a59a0214) | nonheap/N2.json |
| E17 | growth-series | live | node-resources 0.1.0 | artifact series/S4.json (1351e1cfef2a) | series/S4.json |
| E18 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-170838-1982/harness/H1/out/p1-native/capture/9/heap-1.heapsnapshot (85fe4ef3657c) | summaries/heap-1-85fe4ef3.json |
| E19 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-170838-1982/harness/H1/out/p4-native/capture/34/heap-1.heapsnapshot (a946054f6c1b) | summaries/heap-1-a946054f.json |
| E20 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-170838-1982/harness/H1/out/p6-native/capture/50/heap-1.heapsnapshot (1becf3e76b3f) | summaries/heap-1-1becf3e7.json |
| E21 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-170838-1982/harness/H1/out/p9-native/capture/74/heap-1.heapsnapshot (1dfac607d0ba) | summaries/heap-1-1dfac607.json |
| E22 | growth-series | dump | dla-v8-snapshot 0.1.0 | artifact series/S5.json (2f137b873b56) | series/S5.json |
| E23 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S5.json (2f137b873b56) | series/S5.json |
| E24 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S5.json (2f137b873b56) | series/S5.json |
| E25 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S5.json (2f137b873b56) | series/S5.json |
| E26 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (1351e1cfef2a) | trend/T4.json |
| E27 | growth-series | dump | dla 0.1.0 | artifact series/S5.json (2f137b873b56) | trend/T5.json |
| E28 | unit-count-series | dump | dla 0.1.0 | artifact series/S5.json (2f137b873b56) | trend/T5.json |
| E29 | residual | test | dla 0.1.0 | artifact series/S1.json (e14d09bfc75a) | gap/residual-S1-E29.json |
| E30 | residual | test | dla 0.1.0 | artifact series/S2.json (de22d939ef89) | gap/residual-S2-E30.json |
| E31 | residual | test | dla 0.1.0 | artifact series/S3.json (65ae050baf07) | gap/residual-S3-E31.json |
| E32 | non-heap | test | dla 0.1.0 | artifact series/S6.json (968b317d223e) | trend/T6.json |
| E33 | residual | test | dla 0.1.0 | artifact series/S1.json (e14d09bfc75a) | gap/residual-S1-E33.json |
| E34 | residual | test | dla 0.1.0 | artifact series/S2.json (de22d939ef89) | gap/residual-S2-E34.json |
| E35 | residual | test | dla 0.1.0 | artifact series/S3.json (65ae050baf07) | gap/residual-S3-E35.json |
