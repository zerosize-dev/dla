# DLA report

Run: `20261110-194032-1513`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 16 enumerated, 16 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 14

Residual: heap 5139.404545454545 bytes/n unattributed of 5139.404545454545 total (threshold 1284.8511363636362) ABOVE THRESHOLD, see O1; heap 4984.214848484848 bytes/n unattributed of 4984.214848484848 total (threshold 1246.053712121212) ABOVE THRESHOLD, see O1; heap 5046.075303030303 bytes/n unattributed of 5046.075303030303 total (threshold 1261.5188257575758) ABOVE THRESHOLD, see O1; off-heap 6144 bytes/n unattributed of 6144 total (threshold 1536) ABOVE THRESHOLD, see O2; off-heap 6144 bytes/n unattributed of 6144 total (threshold 1536) ABOVE THRESHOLD, see O2; off-heap 6144 bytes/n unattributed of 6144 total (threshold 1536) ABOVE THRESHOLD, see O2

Coverage: node 3520 iterations, 25 of 30 entry points exercised

## Findings

### F1: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Per-session setInterval heartbeat timer (src/heartbeat.js:15) is never clearInterval'd when the session closes; the active timer's closure retains Session, socket, decoder, recent frames and listeners, so every closed connection stays resident.)
- Growth: `node.timers@src/heartbeat.js:15` 1 count/n (steady, gate pass, E12)
- Evidence classes: context, discriminator, growth, static
- Holder: blocked-stack
- Owner: `src.heartbeat.startHeartbeat`
- Region: non-heap
- Shape: steady
- Site: `src/heartbeat.js:15`
- Writers: `src/heartbeat.js:15`
- Annotation: setInterval per session never cleared on close
- Verified: no
- Sibling hypotheses: F1.n1
- Supporting evidence: E6, E10, E12, E13
- Explanation (inferred): startHeartbeat at src/heartbeat.js:15 creates a setInterval per session; Session.close and Gateway.release never clearInterval it, so the live timer (blocked-stack holder) retains each closed Session.
- Explanation structure: adds: src/heartbeat.js:15; removal: never

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.metrics`
- Region: heap
- Site: `src/gateway.js:13`
- Reason: not_exercised
- Remedy: Census lead only; no growth column tied to this holder except as a consequence of F1 (retained closed sessions). Re-measure after F1 is fixed.
- Supporting evidence: E7

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.router`
- Region: heap
- Site: `src/gateway.js:14`
- Reason: not_exercised
- Remedy: Census lead only; no growth column tied to this holder except as a consequence of F1 (retained closed sessions). Re-measure after F1 is fixed.
- Supporting evidence: E7

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.sessions`
- Region: heap
- Site: `src/gateway.js:15`
- Writers: `src/gateway.js:22`
- Removers: `src/gateway.js:32`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Census lead only; no growth column tied to this holder except as a consequence of F1 (retained closed sessions). Re-measure after F1 is fixed.
- Supporting evidence: E7

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: discriminator, static
- Holder: registration-table
- Owner: `src.gateway.Gateway.accept`
- Region: heap
- Site: `src/gateway.js:24`
- Writers: `src/gateway.js:25`, `src/session.js:12`, `src/session.js:54`
- Removers: `load.js:50`, `src/gateway.js:31`, `src/session.js:66`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Growth is downstream of F1 (closed sessions and their sockets/listeners/recent frames stay reachable via the live heartbeat timer); not independently isolated. Re-measure after the timer is cleared on session close.
- Supporting evidence: E7, E13

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: discriminator, static
- Holder: registration-table
- Owner: `src.gateway.Gateway.accept`
- Region: heap
- Site: `src/gateway.js:25`
- Writers: `src/gateway.js:24`, `src/session.js:12`, `src/session.js:54`
- Removers: `load.js:50`, `src/gateway.js:31`, `src/session.js:66`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Growth is downstream of F1 (closed sessions and their sockets/listeners/recent frames stay reachable via the live heartbeat timer); not independently isolated. Re-measure after the timer is cleared on session close.
- Supporting evidence: E7, E13

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: discriminator, static
- Holder: blocked-stack
- Owner: `src.heartbeat.startHeartbeat`
- Region: non-heap
- Site: `src/heartbeat.js:15`
- Reason: not_exercised
- Remedy: Growth is downstream of F1 (closed sessions and their sockets/listeners/recent frames stay reachable via the live heartbeat timer); not independently isolated. Re-measure after the timer is cleared on session close.
- Supporting evidence: E7, E13

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.loopback.LoopbackSocket.written`
- Region: heap
- Site: `src/loopback.js:11`
- Writers: `src/loopback.js:17`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Census lead only; no growth column tied to this holder except as a consequence of F1 (retained closed sessions). Re-measure after F1 is fixed.
- Supporting evidence: E7

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.metrics.Metrics.counters`
- Region: heap
- Site: `src/metrics.js:7`
- Writers: `src/metrics.js:12`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Census lead only; no growth column tied to this holder except as a consequence of F1 (retained closed sessions). Re-measure after F1 is fixed.
- Supporting evidence: E7

### F10: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.router.Router.topics`
- Region: heap
- Site: `src/router.js:9`
- Writers: `src/router.js:16`, `src/router.js:19`
- Removers: `src/router.js:26`, `src/router.js:27`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Census lead only; no growth column tied to this holder except as a consequence of F1 (retained closed sessions). Re-measure after F1 is fixed.
- Supporting evidence: E7

### F11: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `src.router.Router.dispatch`
- Region: heap
- Site: `src/router.js:51`
- Writers: `src/session.js:38`
- Removers: `src/gateway.js:25`, `src/router.js:54`, `src/session.js:33`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Census lead only; no growth column tied to this holder except as a consequence of F1 (retained closed sessions). Re-measure after F1 is fixed.
- Supporting evidence: E7

### F12: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.decoder`
- Region: heap
- Site: `src/session.js:15`
- Writers: `src/session.js:30`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Census lead only; no growth column tied to this holder except as a consequence of F1 (retained closed sessions). Re-measure after F1 is fixed.
- Supporting evidence: E7

### F13: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.topics`
- Region: heap
- Site: `src/session.js:19`
- Reason: not_exercised
- Remedy: Census lead only; no growth column tied to this holder except as a consequence of F1 (retained closed sessions). Re-measure after F1 is fixed.
- Supporting evidence: E7

### F14: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.recent`
- Region: heap
- Site: `src/session.js:20`
- Writers: `src/session.js:44`
- Removers: `src/session.js:45`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Census lead only; no growth column tied to this holder except as a consequence of F1 (retained closed sessions). Re-measure after F1 is fixed.
- Supporting evidence: E7

### F15: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: discriminator, growth
- Reason: not_exercised
- Remedy: Growth is downstream of F1 (closed sessions and their sockets/listeners/recent frames stay reachable via the live heartbeat timer); not independently isolated. Re-measure after the timer is cleared on session close.
- Supporting evidence: E24, E25, E13

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (leak) E13
  - Hypothesis: N-WARM: timers count plateaus after warm-up
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E13
  - Originally supported by: E12

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

- E5: partial coverage
- E5: limitation: rxjs-x rules need rxjs to resolve from the project's node_modules for type information
- E7: partial coverage
- E7: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E7: limitation: writers and removers are textual name matches
- E8: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E8: limitation: workloads were discovered by reading files; none was run
- E9: limitation: harness written; nothing was run (dla harness run measures it)
- E9: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E10: partial coverage (9 of 9 examined)
- E10: limitation: command-mode heap dumps come from separate processes: dla summarize heap-graph compares dumps of one process only, so use a driver (dla harness gen --driver) for a dump series
- E10: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E10: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds som…
- E10: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that…
- E10: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E11: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E11: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E11: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E11: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E12: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E12: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E12: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E13: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E13: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E13: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E13: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E13: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E13: limitation: points 1 to 9 come from S1; the extra points from S2
- E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- E14: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E14: limitation: weak edges are skipped; string contents are never read into the summary
- E15: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E15: limitation: weak edges are skipped; string contents are never read into the summary
- E16: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E16: limitation: weak edges are skipped; string contents are never read into the summary
- E17: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E17: limitation: weak edges are skipped; string contents are never read into the summary
- E18: limitation: dump-series rule: at least 3 snapshots, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E18: limitation: series: per-point; one process per point at n = 440, 1760, 2640, 3960 (harness captures; checked: snapshot format)
- E19: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E20: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E21: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E22: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E22: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E23: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E23: limitation: gate thresholds are provisional (not calibrated)
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
| E1 | run-config | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | — |
| E2 | run-config | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | probe/probe.json |
| E4 | probe | static | dla-probe 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | probe/probe.json |
| E5 | static-hit | static | eslint 10.14.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | sweep/node-eslint.json |
| E6 | static-hit | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | sweep/node-dla.json |
| E7 | census-holder | static | dla-census 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | census/node.json |
| E8 | workload | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | workload/workloads.json |
| E9 | workload | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | harness/H1/harness.json |
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (25ddac1df167) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (30995cdb4f37) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (30995cdb4f37) | trend/T1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S3.json (eb4274dcbb7c) | alloc/A1.json |
| E14 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-194032-1513/harness/H1/out/p1-native/capture/9/heap-1.heapsnapshot (de3d64dba8db) | summaries/heap-1-de3d64db.json |
| E15 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-194032-1513/harness/H1/out/p4-native/capture/34/heap-1.heapsnapshot (89345dc6e407) | summaries/heap-1-89345dc6.json |
| E16 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-194032-1513/harness/H1/out/p6-native/capture/50/heap-1.heapsnapshot (d6408b259935) | summaries/heap-1-d6408b25.json |
| E17 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-194032-1513/harness/H1/out/p9-native/capture/74/heap-1.heapsnapshot (6da433a89388) | summaries/heap-1-6da433a8.json |
| E18 | growth-series | dump | dla-v8-snapshot 0.1.0 | artifact series/S4.json (9894021f2a3f) | series/S4.json |
| E19 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S4.json (9894021f2a3f) | series/S4.json |
| E20 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S4.json (9894021f2a3f) | series/S4.json |
| E21 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S4.json (9894021f2a3f) | series/S4.json |
| E22 | growth-series | test | dla 0.1.0 | artifact series/S2.json (3bccc56b082f) | trend/T3.json |
| E23 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (3bccc56b082f) | trend/T3.json |
| E24 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (9894021f2a3f) | trend/T4.json |
| E25 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (9894021f2a3f) | trend/T4.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S1.json (30995cdb4f37) | gap/residual-S1-E26.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S2.json (3bccc56b082f) | gap/residual-S2-E27.json |
| E28 | residual | test | dla 0.1.0 | artifact series/S3.json (eb4274dcbb7c) | gap/residual-S3-E28.json |
