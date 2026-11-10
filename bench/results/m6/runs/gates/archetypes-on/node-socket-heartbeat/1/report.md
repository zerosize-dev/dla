# DLA report

Run: `20261110-163258-b202`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 18 enumerated, 18 dispositioned; confirmed 1, strong 0, refuted 1, suspected 10, unresolved 6

Residual: heap 5097.204090909091 bytes/n unattributed of 5097.204090909091 total (threshold 1274.3010227272728) ABOVE THRESHOLD, see O1; heap 4988.537012987013 bytes/n unattributed of 4988.537012987013 total (threshold 1247.1342532467534) ABOVE THRESHOLD, see O1; heap 5041.47803030303 bytes/n unattributed of 5041.47803030303 total (threshold 1260.3695075757576) ABOVE THRESHOLD, see O1; heap 48.2939393939394 bytes/n unattributed of 48.2939393939394 total (threshold 15.737702727272728) ABOVE THRESHOLD, see O1; off-heap 6144 bytes/n unattributed of 6144 total (threshold 1536) ABOVE THRESHOLD, see O2; off-heap 6144 bytes/n unattributed of 6144 total (threshold 1536) ABOVE THRESHOLD, see O2; off-heap 6144 bytes/n unattributed of 6144 total (threshold 1536) ABOVE THRESHOLD, see O2; off-heap 0 bytes/n unattributed of 0 total (threshold 0)

Coverage: node 3520 iterations, 25 of 30 entry points exercised

## Findings

### F1: **L-REGISTER**

- State: confirmed
- Label: L-REGISTER (evidence: E11 unit-count-series)
- Growth: `node.timers@src/heartbeat.js:15` 1 count/n (steady, gate pass, E11)
- Evidence classes: context, discriminator, growth, static
- Holder: registration-table
- Region: non-heap
- Shape: steady
- Site: `src/heartbeat.js:15`
- Writers: `src/heartbeat.js:15`
- Annotation: setInterval timer registered per session, never cleared
- Annotation: setInterval per session, never cleared in Session.close
- Verified: no
- Sibling hypotheses: F1.n1
- Suggested archetype (unconfirmed): L-WORKER
- Supporting evidence: E5, E9, E11, E12
- Explanation (inferred): startHeartbeat in src/heartbeat.js:15 creates a setInterval per accepted session (a registration-table holder in the timer list); Session.close and Gateway.release never clearInterval, so the live timer keeps each closed session and its socket reachable.
- Explanation structure: adds: src/heartbeat.js:15; removal: never

### F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.metrics`
- Region: heap
- Site: `src/gateway.js:13`
- Reason: not_exercised
- Remedy: census name-match only; no probe grew for this holder in S1-S3 and the confirmed leak F1 explains the growth. Run an unbound alloc-test after fixing F1 to check for residual growth
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.router`
- Region: heap
- Site: `src/gateway.js:14`
- Reason: not_exercised
- Remedy: census name-match only; no probe grew for this holder in S1-S3 and the confirmed leak F1 explains the growth. Run an unbound alloc-test after fixing F1 to check for residual growth
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.sessions`
- Region: heap
- Site: `src/gateway.js:15`
- Writers: `src/gateway.js:22`, `src/gateway.js:21`
- Removers: `src/gateway.js:32`, `src/gateway.js:36`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: census name-match only; no probe grew for this holder in S1-S3 and the confirmed leak F1 explains the growth. Run an unbound alloc-test after fixing F1 to check for residual growth
- Supporting evidence: E6

### F8: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.loopback.LoopbackSocket.written`
- Region: heap
- Site: `src/loopback.js:11`
- Writers: `src/loopback.js:17`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: census name-match only; no probe grew for this holder in S1-S3 and the confirmed leak F1 explains the growth. Run an unbound alloc-test after fixing F1 to check for residual growth
- Supporting evidence: E6

### F9: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.metrics.Metrics.counters`
- Region: heap
- Site: `src/metrics.js:7`
- Writers: `src/metrics.js:12`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: census name-match only; no probe grew for this holder in S1-S3 and the confirmed leak F1 explains the growth. Run an unbound alloc-test after fixing F1 to check for residual growth
- Supporting evidence: E6

### F10: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.router.Router.topics`
- Region: heap
- Site: `src/router.js:9`
- Writers: `src/router.js:16`, `src/router.js:19`
- Removers: `src/router.js:26`, `src/router.js:27`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: census name-match only; no probe grew for this holder in S1-S3 and the confirmed leak F1 explains the growth. Run an unbound alloc-test after fixing F1 to check for residual growth
- Supporting evidence: E6

### F11: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `src.router.Router.dispatch`
- Region: heap
- Site: `src/router.js:51`
- Writers: `src/session.js:38`
- Removers: `src/gateway.js:25`, `src/router.js:54`, `src/session.js:33`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: census name-match only; no probe grew for this holder in S1-S3 and the confirmed leak F1 explains the growth. Run an unbound alloc-test after fixing F1 to check for residual growth
- Supporting evidence: E6

### F12: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.decoder`
- Region: heap
- Site: `src/session.js:15`
- Writers: `src/session.js:30`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: census name-match only; no probe grew for this holder in S1-S3 and the confirmed leak F1 explains the growth. Run an unbound alloc-test after fixing F1 to check for residual growth
- Supporting evidence: E6

### F13: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.topics`
- Region: heap
- Site: `src/session.js:19`
- Reason: not_exercised
- Remedy: census name-match only; no probe grew for this holder in S1-S3 and the confirmed leak F1 explains the growth. Run an unbound alloc-test after fixing F1 to check for residual growth
- Supporting evidence: E6

### F14: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.recent`
- Region: heap
- Site: `src/session.js:20`
- Writers: `src/session.js:44`
- Removers: `src/session.js:45`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: census name-match only; no probe grew for this holder in S1-S3 and the confirmed leak F1 explains the growth. Run an unbound alloc-test after fixing F1 to check for residual growth
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
- Remedy: node.listeners@src/gateway.js:24 grows 1 per connection, but only because the closed socket stays reachable through the F1 timer; re-measure after clearing the heartbeat interval on close
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
- Remedy: node.listeners@src/gateway.js:24 grows 1 per connection, but only because the closed socket stays reachable through the F1 timer; re-measure after clearing the heartbeat interval on close
- Supporting evidence: E6

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `src.heartbeat.startHeartbeat`
- Region: non-heap
- Site: `src/heartbeat.js:15`
- Writers: `src/heartbeat.js:15`
- Annotation: setInterval per session; no clearInterval anywhere
- Reason: not_exercised
- Remedy: same site and timer column as confirmed F1 (src/heartbeat.js:15); duplicate census view of F1, no separate test
- Supporting evidence: E6

### F15: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Holder: container-field
- Owner: `src.session.Session.recent`
- Region: heap
- Site: `src/session.js:20`
- Reason: time_budget
- Remedy: Session.recent (64 frames/session) is bounded per session; it grows only because closed sessions stay reachable via the F1 timer. Re-measure after F1 fix
- Supporting evidence: E24, E25

### F16: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Holder: container-field
- Owner: `src.loopback.LoopbackSocket.written`
- Region: heap
- Site: `src/loopback.js:11`
- Reason: time_budget
- Remedy: LoopbackSocket.written is the in-process test stand-in for a TCP socket and keeps every written frame; retained via F1 timer. Re-measure after F1 fix
- Supporting evidence: E24, E25

### F17: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: growth
- Holder: static/global
- Owner: `dla-preload.listenerRefs`
- Region: heap
- Reason: env_unavailable
- Remedy: listenerRefs belongs to DLA's own measurement agent (N-OBSERVER); not project code
- Supporting evidence: E24, E25

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM: timers plateau at sessions-in-window
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
- E23: limitation: tested in a scratch copy; the project was not changed
- E23: limitation: gate thresholds are provisional (not calibrated); the 0.2 and 0.5 slope fractions are placeholders
- E23: limitation: exercised is null: the harness records entry points, not lines, so whether the edited line ran is not checked
- E23: limitation: the treated series lacks the control column node.timers@src/heartbeat.js:15: a patch can rename an owner or move a site, so a patch must keep the measured column to be judged
- E23: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E23: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal time...
- E24: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E24: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E24: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E25: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E25: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: partial coverage (2 of 3 examined)
- E26: limitation: F1: growth measured as a count; bytes not attributed
- E26: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E26: limitation: threshold is provisional (not calibrated)
- E27: partial coverage (2 of 3 examined)
- E27: limitation: F1: growth evidence is from S1, not S2; not subtracted
- E27: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E27: limitation: threshold is provisional (not calibrated)
- E28: partial coverage (2 of 3 examined)
- E28: limitation: F1: growth evidence is from S1, not S3; not subtracted
- E28: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E28: limitation: threshold is provisional (not calibrated)
- E29: partial coverage (2 of 3 examined)
- E29: limitation: F1: growth evidence is from S1, not S5; not subtracted
- E29: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E29: limitation: O2 (off-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E29: limitation: threshold is provisional (not calibrated)

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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (a8c3b75b8b4e) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (53cbb6ee974a) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (53cbb6ee974a) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (18435aa09ee0) | alloc/A1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S2.json (7247fc594294) | trend/T3.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (7247fc594294) | trend/T3.json |
| E15 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-163258-b202/harness/H1/out/p1-native/capture/9/heap-1.heapsnapshot (482cdc69937f) | summaries/heap-1-482cdc69.json |
| E16 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-163258-b202/harness/H1/out/p4-native/capture/34/heap-1.heapsnapshot (97eb4be322b5) | summaries/heap-1-97eb4be3.json |
| E17 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-163258-b202/harness/H1/out/p6-native/capture/50/heap-1.heapsnapshot (a75b32050acc) | summaries/heap-1-a75b3205.json |
| E18 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-163258-b202/harness/H1/out/p9-native/capture/74/heap-1.heapsnapshot (3466fa6e4563) | summaries/heap-1-3466fa6e.json |
| E19 | growth-series | dump | dla-v8-snapshot 0.1.0 | artifact series/S4.json (90c91fe54f8e) | series/S4.json |
| E20 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S4.json (90c91fe54f8e) | series/S4.json |
| E21 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S4.json (90c91fe54f8e) | series/S4.json |
| E22 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S4.json (90c91fe54f8e) | series/S4.json |
| E23 | intervention | test | dla 0.1.0 | artifact series/S5.json (1c561bfabb2d) | intervene/I1 |
| E24 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (90c91fe54f8e) | trend/T5.json |
| E25 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (90c91fe54f8e) | trend/T5.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S1.json (53cbb6ee974a) | gap/residual-S1-E26.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S2.json (7247fc594294) | gap/residual-S2-E27.json |
| E28 | residual | test | dla 0.1.0 | artifact series/S3.json (18435aa09ee0) | gap/residual-S3-E28.json |
| E29 | residual | test | dla 0.1.0 | artifact series/S5.json (1c561bfabb2d) | gap/residual-S5-E29.json |
