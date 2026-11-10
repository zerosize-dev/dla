# DLA report

Run: `20261110-202246-6839`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 3 enumerated, 3 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 3

Residual: heap 9.078181818181818 bytes/n unattributed of 9.078181818181818 total (threshold 20.769493090908956); off-heap 0 bytes/n unattributed of 0 total (threshold 0)

Coverage: node 3520 iterations, 10 of 11 entry points exercised

## Findings

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
- Remedy: No growth reproduced by load.js harness (S1 flat, 38 columns) or by a 100k-update verbose publish loop (heapUsed plateau ~3.85MB after gc). Capture heap snapshots from the real service across bursts and run dla summarize heap-graph.
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.feed.Feed.random`
- Region: heap
- Site: `src/feed.js:17`
- Reason: not_exercised
- Remedy: No growth reproduced by load.js harness (S1 flat, 38 columns) or by a 100k-update verbose publish loop (heapUsed plateau ~3.85MB after gc). Capture heap snapshots from the real service across bursts and run dla summarize heap-graph.
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
- Remedy: No growth reproduced by load.js harness (S1 flat, 38 columns) or by a 100k-update verbose publish loop (heapUsed plateau ~3.85MB after gc). Capture heap snapshots from the real service across bursts and run dla summarize heap-graph.
- Supporting evidence: E6

## Ruled out

None.

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
- E12: partial coverage (2 of 3 examined)
- E12: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E12: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo e91c117114645a5643a4ae62870a620311c2969d | — |
| E2 | run-config | static | dla 0.1.0 | repo e91c117114645a5643a4ae62870a620311c2969d | — |
| E3 | probe | static | dla-probe 0.1.0 | repo e91c117114645a5643a4ae62870a620311c2969d | probe/probe.json |
| E4 | static-hit | static | eslint 10.14.0 | repo e91c117114645a5643a4ae62870a620311c2969d | sweep/node-eslint.json |
| E5 | static-hit | static | dla 0.1.0 | repo e91c117114645a5643a4ae62870a620311c2969d | sweep/node-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo e91c117114645a5643a4ae62870a620311c2969d | census/node.json |
| E7 | workload | static | dla 0.1.0 | repo e91c117114645a5643a4ae62870a620311c2969d | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo e91c117114645a5643a4ae62870a620311c2969d | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (e6bd2cb91be7) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (e3fdb61ca94a) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (e3fdb61ca94a) | trend/T1.json |
| E12 | residual | test | dla 0.1.0 | artifact series/S1.json (e3fdb61ca94a) | gap/residual-S1-E12.json |
