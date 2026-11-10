# DLA report

Run: `20261110-175727-5f8f`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 16 enumerated, 16 dispositioned; confirmed 1, strong 1, refuted 2, suspected 0, unresolved 12

Residual: heap 59.630909090909086 bytes/n unattributed of 59.630909090909086 total (threshold 41.15139218181805) ABOVE THRESHOLD, see O1; heap 4422.227272727272 bytes/iteration unattributed of 4422.227272727272 total (threshold 1105.556818181818) ABOVE THRESHOLD, see O1; heap 4078.1195454545455 bytes/iteration unattributed of 4078.1195454545455 total (threshold 1019.5298863636364) ABOVE THRESHOLD, see O1; heap 4331.048551448552 bytes/iteration unattributed of 4331.048551448552 total (threshold 1082.762137862138) ABOVE THRESHOLD, see O1; heap 60.45272727272727 bytes/iteration unattributed of 60.45272727272727 total (threshold 15.113181818181818) ABOVE THRESHOLD, see O3; heap 4343.895292207792 bytes/iteration unattributed of 4343.895292207792 total (threshold 1085.973823051948) ABOVE THRESHOLD, see O3; off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 4524.218181818182 bytes/iteration unattributed of 4524.218181818182 total (threshold 1131.0545454545454) ABOVE THRESHOLD, see O2; off-heap 4325.624242424243 bytes/iteration unattributed of 4325.624242424243 total (threshold 1081.4060606060607) ABOVE THRESHOLD, see O2; off-heap 4524.218181818182 bytes/iteration unattributed of 4524.218181818182 total (threshold 1131.0545454545454) ABOVE THRESHOLD, see O2; off-heap 0 bytes/iteration unattributed of 0 total (threshold 0); off-heap 4524.218181818182 bytes/iteration unattributed of 4524.218181818182 total (threshold 1131.0545454545454) ABOVE THRESHOLD, see O4

Coverage: node 7040 iterations, 25 of 30 entry points exercised

## Findings

### F4: **UNCLASSIFIED**

- State: strong
- Label: UNCLASSIFIED (Gateway.sessions Map (gateway.js:22) keeps each session, its socket and the socket's data/close listeners (gateway.js:24-25) when the session is closed server-side (bye/idle/protocol); sessions.delete runs only from the socket close event, which the destroy() path never raises on the loopback socket)
- Growth: `node.listeners@src/gateway.js:24` 1 count/iteration (steady, gate pass, E22)
- Evidence classes: context, discriminator, growth, intervention, static
- Strong: second class E31 intervention that removed the growth, independent of the growth record E21 (dla)
- Holder: registration-table
- Owner: `src.gateway.Gateway.accept`
- Region: heap
- Shape: steady
- Site: `src/gateway.js:24`
- Writers: `src/gateway.js:25`, `src/session.js:12`, `src/session.js:54`, `src/gateway.js:22`
- Removers: `load.js:50`, `src/gateway.js:31`, `src/session.js:67`, `src/gateway.js:32`
- Annotation: heuristic name match over the project's sources
- Annotation: sessions entry and socket listeners retained for sessions closed server-side (bye/idle/protocol); sessions.delete only runs from the socket close event
- Verified: no
- Sibling hypotheses: F4.n1
- Supporting evidence: E6, E20, E21, E22, E30, E31
- Explanation (inferred): Gateway.accept adds each session to the sessions map (gateway.js:22) and registers socket listeners at gateway.js:24; the entry is removed by release() at gateway.js:32 only when the socket close event fires, so bye, idle and protocol closes leave sessions retained.
- Explanation structure: adds: src/gateway.js:22; removal: conditional at src/gateway.js:32

### F5: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (socket close listener (gateway.js:25) retained with the session in Gateway.sessions for server-side closes; same mechanism as F4)
- Growth: `node.listeners@src/gateway.js:24` 1 count/iteration (steady, gate pass, E36)
- Evidence classes: context, discriminator, growth, static
- Holder: registration-table
- Owner: `src.gateway.Gateway.accept`
- Region: heap
- Shape: steady
- Site: `src/gateway.js:25`
- Writers: `src/gateway.js:24`, `src/session.js:12`, `src/session.js:54`, `src/gateway.js:22`
- Removers: `load.js:50`, `src/gateway.js:31`, `src/session.js:67`, `src/gateway.js:32`
- Annotation: heuristic name match over the project's sources
- Annotation: close listener on retained socket
- Verified: no
- Sibling hypotheses: F5.n1
- Supporting evidence: E6, E20, E35, E36, E39
- Explanation (inferred): Gateway.accept adds each session to the sessions map (gateway.js:22) with the close listener at gateway.js:25; the entry is removed by release() at gateway.js:32 only when the socket close event fires, so server-side closes leave it retained.
- Explanation structure: adds: src/gateway.js:22; removal: conditional at src/gateway.js:32

### F1: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.metrics`
- Region: heap
- Site: `src/gateway.js:13`
- Reason: time_budget
- Remedy: not independently tied to growth; F3 (Gateway.sessions) is the container the F4/F5 leak retains via, and F14/F13 growth is the harness agent / bounded buffer riding on the retained sessions
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.router`
- Region: heap
- Site: `src/gateway.js:14`
- Reason: time_budget
- Remedy: not independently tied to growth; F3 (Gateway.sessions) is the container the F4/F5 leak retains via, and F14/F13 growth is the harness agent / bounded buffer riding on the retained sessions
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

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
- Remedy: not independently tied to growth; F3 (Gateway.sessions) is the container the F4/F5 leak retains via, and F14/F13 growth is the harness agent / bounded buffer riding on the retained sessions
- Supporting evidence: E6

### F6: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `src.heartbeat.startHeartbeat`
- Region: non-heap
- Site: `src/heartbeat.js:15`
- Reason: time_budget
- Remedy: not independently tied to growth; F3 (Gateway.sessions) is the container the F4/F5 leak retains via, and F14/F13 growth is the harness agent / bounded buffer riding on the retained sessions
- Supporting evidence: E6

### F7: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: container-field
- Owner: `src.loopback.LoopbackSocket.written`
- Region: heap
- Site: `src/loopback.js:11`
- Writers: `src/loopback.js:17`
- Annotation: heuristic name match over the project's sources
- Reason: time_budget
- Remedy: not independently tied to growth; F3 (Gateway.sessions) is the container the F4/F5 leak retains via, and F14/F13 growth is the harness agent / bounded buffer riding on the retained sessions
- Supporting evidence: E6

### F8: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: container-field
- Owner: `src.metrics.Metrics.counters`
- Region: heap
- Site: `src/metrics.js:7`
- Writers: `src/metrics.js:12`
- Annotation: heuristic name match over the project's sources
- Reason: time_budget
- Remedy: not independently tied to growth; F3 (Gateway.sessions) is the container the F4/F5 leak retains via, and F14/F13 growth is the harness agent / bounded buffer riding on the retained sessions
- Supporting evidence: E6

### F9: **UNCLASSIFIED**

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
- Remedy: not independently tied to growth; F3 (Gateway.sessions) is the container the F4/F5 leak retains via, and F14/F13 growth is the harness agent / bounded buffer riding on the retained sessions
- Supporting evidence: E6

### F10: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: registration-table
- Owner: `src.router.Router.dispatch`
- Region: heap
- Site: `src/router.js:51`
- Writers: `src/session.js:38`
- Removers: `src/gateway.js:25`, `src/router.js:54`, `src/session.js:33`, `src/session.js:65`
- Annotation: heuristic name match over the project's sources
- Reason: time_budget
- Remedy: not independently tied to growth; F3 (Gateway.sessions) is the container the F4/F5 leak retains via, and F14/F13 growth is the harness agent / bounded buffer riding on the retained sessions
- Supporting evidence: E6

### F11: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.decoder`
- Region: heap
- Site: `src/session.js:15`
- Writers: `src/session.js:30`
- Annotation: heuristic name match over the project's sources
- Reason: time_budget
- Remedy: not independently tied to growth; F3 (Gateway.sessions) is the container the F4/F5 leak retains via, and F14/F13 growth is the harness agent / bounded buffer riding on the retained sessions
- Supporting evidence: E6

### F12: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.topics`
- Region: heap
- Site: `src/session.js:19`
- Reason: time_budget
- Remedy: not independently tied to growth; F3 (Gateway.sessions) is the container the F4/F5 leak retains via, and F14/F13 growth is the harness agent / bounded buffer riding on the retained sessions
- Supporting evidence: E6

### F13: **UNCLASSIFIED**

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
- Remedy: not independently tied to growth; F3 (Gateway.sessions) is the container the F4/F5 leak retains via, and F14/F13 growth is the harness agent / bounded buffer riding on the retained sessions
- Supporting evidence: E6

### F14: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: retention
- Holder: static/global
- Owner: `.dla.20261110-175727-5f8f.harness.H2.agent.dla-preload.listenerRefs`
- Region: heap
- Reason: time_budget
- Remedy: not independently tied to growth; F3 (Gateway.sessions) is the container the F4/F5 leak retains via, and F14/F13 growth is the harness agent / bounded buffer riding on the retained sessions
- Supporting evidence: E26

## Ruled out

- **F4.n1** (UNCLASSIFIED): discriminator E30
  - Ruled out by: plateau+purge (leak) E30
  - Hypothesis: N-OBSERVER: listener growth is an artefact of DLA's agent listenerRefs retaining sockets, not of the gateway
  - Refuted if: growth persists with the agent's refs not the retainer, or intervene stubbing the sessions writer removes it
  - Contradicting evidence: E30
  - Originally supported by: E22
- **F5.n1** (UNCLASSIFIED): discriminator E39
  - Ruled out by: plateau+purge (leak) E39
  - Hypothesis: N-WARM: listener count grows only during warm-up
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E39
  - Originally supported by: E36

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
- E19: limitation: harness written; nothing was run (dla harness run measures it)
- E19: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E20: partial coverage (9 of 9 examined)
- E20: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E20: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds som…
- E20: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that…
- E20: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E21: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E21: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E21: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E21: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E22: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E22: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E22: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E23: limitation: weak edges are skipped; string contents are never read into the summary
- E24: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E24: limitation: weak edges are skipped; string contents are never read into the summary
- E25: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E25: limitation: weak edges are skipped; string contents are never read into the summary
- E26: limitation: retained bytes follow the dominator tree: a holder counts what only it keeps alive
- E26: limitation: weak edges are skipped; string contents are never read into the summary
- E27: limitation: dump-series rule: at least 3 snapshots, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E27: limitation: the snapshots are assumed to come from one process (checked: one pid directory, growing object ids, surviving ids)
- E28: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E29: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E30: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E30: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E30: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E30: limitation: gate and plateau thresholds are provisional (not calibrated)
- E31: limitation: tested in a scratch copy; the project was not changed
- E31: limitation: gate thresholds are provisional (not calibrated); the 0.2 and 0.5 slope fractions are placeholders
- E31: limitation: exercised is null: the harness records entry points, not lines, so whether the edited line ran is not checked
- E31: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E31: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal time...
- E32: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E32: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E32: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E32: limitation: gate thresholds are provisional (not calibrated)
- E33: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap snapshot
- E33: limitation: retained bytes follow the dominator tree; a snapshot records no time
- E33: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E33: limitation: gate thresholds are provisional (not calibrated)
- E34: limitation: x is the snapshot number in capture order; t_ms is the time since the first capture where the inputs record one
- E34: limitation: each point comes from a separate snapshot of one process; a figure absent from a snapshot has no value at that point
- E34: limitation: gate thresholds are provisional (not calibrated)
- E35: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E35: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E35: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E35: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E35: limitation: gate thresholds are provisional (not calibrated)
- E36: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E36: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E36: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E36: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E36: limitation: gate thresholds are provisional (not calibrated)
- E37: limitation: x is the snapshot number in object-id order (ids grow within one process); a snapshot records no time
- E37: limitation: each point comes from a separate heap snapshot of one process; retained bytes follow the dominator tree
- E37: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E37: limitation: gate thresholds are provisional (not calibrated)
- E38: limitation: x is the snapshot number in object-id order (ids grow within one process); a snapshot records no time
- E38: limitation: each point comes from a separate heap snapshot of one process; retained bytes follow the dominator tree
- E38: limitation: holders below 1 KiB retained in a snapshot have no figure at that point
- E38: limitation: gate thresholds are provisional (not calibrated)
- E39: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E39: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E39: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E39: limitation: gate and plateau thresholds are provisional (not calibrated)
- E40: partial coverage (2 of 3 examined)
- E40: limitation: F4: growth evidence is from S4, not S1; not subtracted
- E40: limitation: F5: growth evidence is from S5, not S1; not subtracted
- E40: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E40: limitation: threshold is provisional (not calibrated)
- E41: partial coverage (2 of 3 examined)
- E41: limitation: F4: growth measured as a count; bytes not attributed
- E41: limitation: F5: growth evidence is from S5, not S4; not subtracted
- E41: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E41: limitation: threshold is provisional (not calibrated)
- E42: partial coverage (2 of 3 examined)
- E42: limitation: F4: growth evidence is from S4, not S5; not subtracted
- E42: limitation: F5: growth measured as a count; bytes not attributed
- E42: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E42: limitation: threshold is provisional (not calibrated)
- E43: partial coverage (2 of 3 examined)
- E43: limitation: F4: growth evidence is from S4, not S7; not subtracted
- E43: limitation: F5: growth evidence is from S5, not S7; not subtracted
- E43: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E43: limitation: threshold is provisional (not calibrated)
- E44: partial coverage (2 of 3 examined)
- E44: limitation: F4: growth evidence is from S4, not S8; not subtracted
- E44: limitation: F5: growth evidence is from S5, not S8; not subtracted
- E44: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E44: limitation: threshold is provisional (not calibrated)
- E45: partial coverage (2 of 3 examined)
- E45: limitation: F4: growth evidence is from S4, not S9; not subtracted
- E45: limitation: F5: growth evidence is from S5, not S9; not subtracted
- E45: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E45: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | — |
| E2 | run-config | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | probe/probe.json |
| E4 | static-hit | static | eslint 10.14.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | sweep/node-eslint.json |
| E5 | static-hit | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | sweep/node-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | census/node.json |
| E7 | workload | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (6f1f1d1bcfdd) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (def6f02facd5) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (def6f02facd5) | trend/T1.json |
| E12 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-175727-5f8f/harness/H1/out/p1-native/capture/9/heap-1.heapsnapshot (6fc8d835cad3) | summaries/heap-1-6fc8d835.json |
| E13 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-175727-5f8f/harness/H1/out/p4-native/capture/34/heap-1.heapsnapshot (04047031883d) | summaries/heap-1-04047031.json |
| E14 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-175727-5f8f/harness/H1/out/p6-native/capture/50/heap-1.heapsnapshot (dacba64ef804) | summaries/heap-1-dacba64e.json |
| E15 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-175727-5f8f/harness/H1/out/p9-native/capture/74/heap-1.heapsnapshot (1b69f11c16fb) | summaries/heap-1-1b69f11c.json |
| E16 | growth-series | dump | dla-v8-snapshot 0.1.0 | artifact series/S2.json (703727320b57) | series/S2.json |
| E17 | non-heap | live | node-resources 0.1.0 | artifact nonheap/N1.json (9ecde5a5389f) | nonheap/N1.json |
| E18 | growth-series | live | node-resources 0.1.0 | artifact series/S3.json (f6859d5d3d7f) | series/S3.json |
| E19 | workload | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | harness/H2/harness.json |
| E20 | workload | test | dla 0.1.0 | artifact harness/H2/harness.json (28a635715b5d) | harness/H2/run-S4.json |
| E21 | growth-series | test | dla 0.1.0 | artifact series/S4.json (21815f4d9d21) | trend/T2.json |
| E22 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (21815f4d9d21) | trend/T2.json |
| E23 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-175727-5f8f/harness/H2/out/p1-fresh/capture/11/heap-1.heapsnapshot (69e949dc91b2) | summaries/heap-1-69e949dc.json |
| E24 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-175727-5f8f/harness/H2/out/p1-fresh/capture/11/heap-2.heapsnapshot (dbe37e3e7ff4) | summaries/heap-2-dbe37e3e.json |
| E25 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-175727-5f8f/harness/H2/out/p1-fresh/capture/11/heap-3.heapsnapshot (c7df0f48a5e4) | summaries/heap-3-c7df0f48.json |
| E26 | retention-path | dump | dla-v8-snapshot 0.1.0 | artifact .dla/20261110-175727-5f8f/harness/H2/out/p1-fresh/capture/11/heap-4.heapsnapshot (82da8f2ed36a) | summaries/heap-4-82da8f2e.json |
| E27 | growth-series | dump | dla-v8-snapshot 0.1.0 | artifact series/S6.json (3d11dce14258) | series/S6.json |
| E28 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S6.json (3d11dce14258) | series/S6.json |
| E29 | discriminator | dump | dla-v8-snapshot 0.1.0 | artifact series/S6.json (3d11dce14258) | series/S6.json |
| E30 | discriminator | test | dla 0.1.0 | artifact series/S7.json (0c8aa2241cd1) | alloc/A1.json |
| E31 | intervention | test | dla 0.1.0 | artifact series/S8.json (1095bec10233) | intervene/I1 |
| E32 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (703727320b57) | trend/T5.json |
| E33 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (703727320b57) | trend/T5.json |
| E34 | unit-count-series | dump | dla 0.1.0 | artifact series/S3.json (f6859d5d3d7f) | trend/T6.json |
| E35 | growth-series | test | dla 0.1.0 | artifact series/S5.json (7780eb93f2f2) | trend/T7.json |
| E36 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (7780eb93f2f2) | trend/T7.json |
| E37 | growth-series | dump | dla 0.1.0 | artifact series/S6.json (3d11dce14258) | trend/T8.json |
| E38 | unit-count-series | dump | dla 0.1.0 | artifact series/S6.json (3d11dce14258) | trend/T8.json |
| E39 | discriminator | test | dla 0.1.0 | artifact series/S9.json (e76d3527697f) | alloc/A2.json |
| E40 | residual | test | dla 0.1.0 | artifact series/S1.json (def6f02facd5) | gap/residual-S1-E40.json |
| E41 | residual | test | dla 0.1.0 | artifact series/S4.json (21815f4d9d21) | gap/residual-S4-E41.json |
| E42 | residual | test | dla 0.1.0 | artifact series/S5.json (7780eb93f2f2) | gap/residual-S5-E42.json |
| E43 | residual | test | dla 0.1.0 | artifact series/S7.json (0c8aa2241cd1) | gap/residual-S7-E43.json |
| E44 | residual | test | dla 0.1.0 | artifact series/S8.json (1095bec10233) | gap/residual-S8-E44.json |
| E45 | residual | test | dla 0.1.0 | artifact series/S9.json (e76d3527697f) | gap/residual-S9-E45.json |
