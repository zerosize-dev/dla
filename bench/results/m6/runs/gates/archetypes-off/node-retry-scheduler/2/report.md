# DLA report

Run: `20261110-190208-7894`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 9 enumerated, 9 dispositioned; confirmed 1, strong 0, refuted 4, suspected 0, unresolved 4

Residual: heap 4288.61590909091 bytes/n unattributed of 4288.61590909091 total (threshold 1072.1539772727274) ABOVE THRESHOLD, see O1; heap 4265.301515151516 bytes/n unattributed of 4265.301515151516 total (threshold 1066.325378787879) ABOVE THRESHOLD, see O1; heap 4271.7664646464655 bytes/n unattributed of 4271.7664646464655 total (threshold 1067.9416161616164) ABOVE THRESHOLD, see O1; heap 4268.221212121212 bytes/n unattributed of 4268.221212121212 total (threshold 1067.055303030303) ABOVE THRESHOLD, see O1; heap 4271.996877869606 bytes/n unattributed of 4271.996877869606 total (threshold 1067.9992194674014) ABOVE THRESHOLD, see O1; heap 4262.762121212121 bytes/n unattributed of 4262.762121212121 total (threshold 1065.6905303030303) ABOVE THRESHOLD, see O1; heap 4271.012337662338 bytes/n unattributed of 4271.012337662338 total (threshold 1067.7530844155845) ABOVE THRESHOLD, see O1; heap 4264.918181818182 bytes/n unattributed of 4264.918181818182 total (threshold 1066.2295454545456) ABOVE THRESHOLD, see O1; heap 4271.046060606061 bytes/n unattributed of 4271.046060606061 total (threshold 1067.7615151515151) ABOVE THRESHOLD, see O1; heap 4266.313636363637 bytes/n unattributed of 4266.313636363637 total (threshold 1066.5784090909092) ABOVE THRESHOLD, see O1; heap 4270.629797979798 bytes/n unattributed of 4270.629797979798 total (threshold 1067.6574494949496) ABOVE THRESHOLD, see O1; heap 4265.848484848485 bytes/n unattributed of 4265.848484848485 total (threshold 1066.4621212121212) ABOVE THRESHOLD, see O1; heap 4271.001298701299 bytes/n unattributed of 4271.001298701299 total (threshold 1067.7503246753247) ABOVE THRESHOLD, see O1; heap 4265.812121212121 bytes/n unattributed of 4265.812121212121 total (threshold 1066.4530303030303) ABOVE THRESHOLD, see O1; heap 4270.973376623377 bytes/n unattributed of 4270.973376623377 total (threshold 1067.7433441558442) ABOVE THRESHOLD, see O1; heap 4265.862121212122 bytes/n unattributed of 4265.862121212122 total (threshold 1066.4655303030304) ABOVE THRESHOLD, see O1; heap 4270.996428571429 bytes/n unattributed of 4270.996428571429 total (threshold 1067.7491071428572) ABOVE THRESHOLD, see O1; off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 0 bytes/n unattributed of 0 total (threshold 0)

Coverage: node 3520 iterations, 17 of 19 entry points exercised

## Findings

### F5: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (RetryScheduler.latest heads a record.previous back-chain; enqueue links every attempt record to the prior one and nothing ever clears previous, so each delivered/dead record (and its job payload) stays reachable from latest)
- Growth: `probe:src/scheduler.js:shared.latest` 25.97 count/n (steady, gate pass, E11)
- Evidence classes: context, growth, static
- Holder: container-field
- Owner: `src.scheduler.RetryScheduler.latest`
- Region: heap
- Shape: steady
- Site: `src/scheduler.js:15`
- Writers: `src/scheduler.js:32`, `src/scheduler.js:31`
- Annotation: heuristic name match over the project's sources
- Annotation: latest -> record.previous back-chain never truncated; each record pins its job
- Annotation: no remover of record.previous exists
- Verified: no
- Sibling hypotheses: F5.n1
- Supporting evidence: E6, E9, E11
- Explanation (inferred): RetryScheduler.latest (container-field, scheduler.js:15) is advanced at scheduler.js:32 in enqueue and each record keeps previous, so every attempt record and its job stay reachable; previous is never cleared.
- Explanation structure: adds: src/scheduler.js:32; removal: never

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `src/scheduler.js:32`
- Reason: not_exercised
- Remedy: detector hit at scheduler.js:32 is the same site as confirmed F5; covered by F5
- Supporting evidence: E5

### F2: **UNCLASSIFIED**

- State: unresolved(no_harness)
- Evidence classes: discriminator, static
- Holder: container-field
- Owner: `src.pending.PendingList.head`
- Region: heap
- Site: `src/pending.js:7`
- Writers: `src/pending.js:15`, `src/pending.js:24`
- Annotation: heuristic name match over the project's sources
- Reason: no_harness
- Remedy: PendingList head/tail are emptied by unlink (pending length 0 after each batch); no probe measured them; whole-process growth is explained by F5. Add a probe on pending.head/tail to refute.
- Supporting evidence: E6, E29

### F3: **UNCLASSIFIED**

- State: unresolved(no_harness)
- Evidence classes: discriminator, static
- Holder: container-field
- Owner: `src.pending.PendingList.tail`
- Region: heap
- Site: `src/pending.js:8`
- Writers: `src/pending.js:16`, `src/pending.js:25`
- Annotation: heuristic name match over the project's sources
- Reason: no_harness
- Remedy: PendingList head/tail are emptied by unlink (pending length 0 after each batch); no probe measured them; whole-process growth is explained by F5. Add a probe on pending.head/tail to refute.
- Supporting evidence: E6, E29

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: discriminator, static
- Holder: static/global
- Owner: `src.scheduler.shared`
- Region: heap
- Site: `src/scheduler.js:90`
- Writers: `src/scheduler.js:93`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: F8 is the container singleton whose growth is the F5 back-chain (shared.latest); fix F5 and re-measure
- Supporting evidence: E6, E18

## Ruled out

- **F4** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (non-leak) E15
  - Contradicting evidence: E15
  - Originally supported by: E6
- **F5.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM: growth is warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E12
  - Originally supported by: E11
- **F6** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (non-leak) E16
  - Contradicting evidence: E16
  - Originally supported by: E6
- **F7** (UNCLASSIFIED): discriminator E17
  - Ruled out by: plateau+purge (non-leak) E17
  - Contradicting evidence: E17
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
- E13: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E13: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E14: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E15: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E15: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E15: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E15: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E15: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E15: limitation: points 1 to 9 come from S1; the extra points from S4
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E16: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E16: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E16: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E16: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E16: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E16: limitation: points 1 to 9 come from S1; the extra points from S6
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E17: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E17: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E17: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E17: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E17: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E17: limitation: points 1 to 9 come from S1; the extra points from S8
- E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E17: limitation: gate and plateau thresholds are provisional (not calibrated)
- E18: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E18: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E18: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E18: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E18: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E18: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E18: limitation: points 1 to 9 come from S1; the extra points from S10
- E18: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E18: limitation: gate and plateau thresholds are provisional (not calibrated)
- E19: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E19: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E20: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E21: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E22: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E23: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E24: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E25: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E26: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E27: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E27: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E27: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E27: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E27: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E27: limitation: points 1 to 9 come from S1; the extra points from S12
- E27: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E27: limitation: gate and plateau thresholds are provisional (not calibrated)
- E28: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E28: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E28: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E28: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E28: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E28: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E28: limitation: points 1 to 9 come from S1; the extra points from S14
- E28: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E28: limitation: gate and plateau thresholds are provisional (not calibrated)
- E29: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E29: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E29: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E29: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E29: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E29: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E29: limitation: points 1 to 9 come from S1; the extra points from S16
- E29: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E29: limitation: gate and plateau thresholds are provisional (not calibrated)
- E30: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E30: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E30: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E30: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E30: limitation: gate thresholds are provisional (not calibrated)
- E31: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E31: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E31: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E31: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E31: limitation: gate thresholds are provisional (not calibrated)
- E32: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E32: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E32: limitation: gate thresholds are provisional (not calibrated)
- E33: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E33: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E33: limitation: gate thresholds are provisional (not calibrated)
- E34: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E34: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E34: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E34: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E34: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E34: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E34: limitation: points 1 to 9 come from S1; the extra points from S2
- E34: limitation: gate thresholds are provisional (not calibrated)
- E35: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E35: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E35: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E35: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E35: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E35: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E35: limitation: points 1 to 9 come from S1; the extra points from S2
- E35: limitation: gate thresholds are provisional (not calibrated)
- E36: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E36: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E36: limitation: gate thresholds are provisional (not calibrated)
- E37: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E37: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E37: limitation: gate thresholds are provisional (not calibrated)
- E38: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E38: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E38: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E38: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E38: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E38: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E38: limitation: points 1 to 9 come from S1; the extra points from S4
- E38: limitation: gate thresholds are provisional (not calibrated)
- E39: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E39: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E39: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E39: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E39: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E39: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E39: limitation: points 1 to 9 come from S1; the extra points from S4
- E39: limitation: gate thresholds are provisional (not calibrated)
- E40: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E40: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E40: limitation: gate thresholds are provisional (not calibrated)
- E41: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E41: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E41: limitation: gate thresholds are provisional (not calibrated)
- E42: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E42: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E42: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E42: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E42: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E42: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E42: limitation: points 1 to 9 come from S1; the extra points from S6
- E42: limitation: gate thresholds are provisional (not calibrated)
- E43: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E43: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E43: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E43: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E43: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E43: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E43: limitation: points 1 to 9 come from S1; the extra points from S6
- E43: limitation: gate thresholds are provisional (not calibrated)
- E44: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E44: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E44: limitation: gate thresholds are provisional (not calibrated)
- E45: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E45: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E45: limitation: gate thresholds are provisional (not calibrated)
- E46: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E46: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E46: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E46: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E46: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E46: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E46: limitation: points 1 to 9 come from S1; the extra points from S8
- E46: limitation: gate thresholds are provisional (not calibrated)
- E47: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E47: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E47: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E47: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E47: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E47: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E47: limitation: points 1 to 9 come from S1; the extra points from S8
- E47: limitation: gate thresholds are provisional (not calibrated)
- E48: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E48: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E48: limitation: gate thresholds are provisional (not calibrated)
- E49: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E49: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E49: limitation: gate thresholds are provisional (not calibrated)
- E50: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E50: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E50: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E50: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E50: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E50: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E50: limitation: points 1 to 9 come from S1; the extra points from S10
- E50: limitation: gate thresholds are provisional (not calibrated)
- E51: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E51: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E51: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E51: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E51: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E51: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E51: limitation: points 1 to 9 come from S1; the extra points from S10
- E51: limitation: gate thresholds are provisional (not calibrated)
- E52: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E52: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E52: limitation: gate thresholds are provisional (not calibrated)
- E53: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E53: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E53: limitation: gate thresholds are provisional (not calibrated)
- E54: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E54: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E54: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E54: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E54: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E54: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E54: limitation: points 1 to 9 come from S1; the extra points from S12
- E54: limitation: gate thresholds are provisional (not calibrated)
- E55: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E55: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E55: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E55: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E55: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E55: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E55: limitation: points 1 to 9 come from S1; the extra points from S12
- E55: limitation: gate thresholds are provisional (not calibrated)
- E56: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E56: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E56: limitation: gate thresholds are provisional (not calibrated)
- E57: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E57: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E57: limitation: gate thresholds are provisional (not calibrated)
- E58: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E58: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E58: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E58: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E58: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E58: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E58: limitation: points 1 to 9 come from S1; the extra points from S14
- E58: limitation: gate thresholds are provisional (not calibrated)
- E59: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E59: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E59: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E59: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E59: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E59: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E59: limitation: points 1 to 9 come from S1; the extra points from S14
- E59: limitation: gate thresholds are provisional (not calibrated)
- E60: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E60: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E60: limitation: gate thresholds are provisional (not calibrated)
- E61: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E61: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E61: limitation: gate thresholds are provisional (not calibrated)
- E62: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E62: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E62: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E62: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E62: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E62: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E62: limitation: points 1 to 9 come from S1; the extra points from S16
- E62: limitation: gate thresholds are provisional (not calibrated)
- E63: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E63: limitation: --capture heap enables the inspector Debugger domain for the run to record script ids; scripts.json is informational (no dla adapter reads it, snapshots name their scripts), and the Debugger adds some overhead
- E63: limitation: --capture nonheap tracks resources with async_hooks (init and destroy), which makes promise-heavy targets run up to roughly 10x slower than without the capture; compare timings only between runs that use the same capture
- E63: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E63: limitation: function coverage is written by node itself to NODE_V8_COVERAGE
- E63: limitation: timer and listener counts cover registrations made through the global/`timers` functions and EventEmitter/EventTarget methods; timers/promises and internal timers are not counted
- E63: limitation: points 1 to 9 come from S1; the extra points from S16
- E63: limitation: gate thresholds are provisional (not calibrated)
- E64: partial coverage (2 of 3 examined)
- E64: limitation: F5: growth measured as a count; bytes not attributed
- E64: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E64: limitation: threshold is provisional (not calibrated)
- E65: partial coverage (2 of 3 examined)
- E65: limitation: F5: growth evidence is from S1, not S2; not subtracted
- E65: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E65: limitation: threshold is provisional (not calibrated)
- E66: partial coverage (2 of 3 examined)
- E66: limitation: F5: growth evidence is from S1, not S3; not subtracted
- E66: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E66: limitation: threshold is provisional (not calibrated)
- E67: partial coverage (2 of 3 examined)
- E67: limitation: F5: growth evidence is from S1, not S4; not subtracted
- E67: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E67: limitation: threshold is provisional (not calibrated)
- E68: partial coverage (2 of 3 examined)
- E68: limitation: F5: growth evidence is from S1, not S5; not subtracted
- E68: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E68: limitation: threshold is provisional (not calibrated)
- E69: partial coverage (2 of 3 examined)
- E69: limitation: F5: growth evidence is from S1, not S6; not subtracted
- E69: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E69: limitation: threshold is provisional (not calibrated)
- E70: partial coverage (2 of 3 examined)
- E70: limitation: F5: growth evidence is from S1, not S7; not subtracted
- E70: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E70: limitation: threshold is provisional (not calibrated)
- E71: partial coverage (2 of 3 examined)
- E71: limitation: F5: growth evidence is from S1, not S8; not subtracted
- E71: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E71: limitation: threshold is provisional (not calibrated)
- E72: partial coverage (2 of 3 examined)
- E72: limitation: F5: growth evidence is from S1, not S9; not subtracted
- E72: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E72: limitation: threshold is provisional (not calibrated)
- E73: partial coverage (2 of 3 examined)
- E73: limitation: F5: growth evidence is from S1, not S10; not subtracted
- E73: limitation: off-heap: no gap series for S10; run dla gap S10 first (node.external is used when present)
- E73: limitation: threshold is provisional (not calibrated)
- E74: partial coverage (2 of 3 examined)
- E74: limitation: F5: growth evidence is from S1, not S11; not subtracted
- E74: limitation: off-heap: no gap series for S11; run dla gap S11 first (node.external is used when present)
- E74: limitation: threshold is provisional (not calibrated)
- E75: partial coverage (2 of 3 examined)
- E75: limitation: F5: growth evidence is from S1, not S12; not subtracted
- E75: limitation: off-heap: no gap series for S12; run dla gap S12 first (node.external is used when present)
- E75: limitation: threshold is provisional (not calibrated)
- E76: partial coverage (2 of 3 examined)
- E76: limitation: F5: growth evidence is from S1, not S13; not subtracted
- E76: limitation: off-heap: no gap series for S13; run dla gap S13 first (node.external is used when present)
- E76: limitation: threshold is provisional (not calibrated)
- E77: partial coverage (2 of 3 examined)
- E77: limitation: F5: growth evidence is from S1, not S14; not subtracted
- E77: limitation: off-heap: no gap series for S14; run dla gap S14 first (node.external is used when present)
- E77: limitation: threshold is provisional (not calibrated)
- E78: partial coverage (2 of 3 examined)
- E78: limitation: F5: growth evidence is from S1, not S15; not subtracted
- E78: limitation: off-heap: no gap series for S15; run dla gap S15 first (node.external is used when present)
- E78: limitation: threshold is provisional (not calibrated)
- E79: partial coverage (2 of 3 examined)
- E79: limitation: F5: growth evidence is from S1, not S16; not subtracted
- E79: limitation: off-heap: no gap series for S16; run dla gap S16 first (node.external is used when present)
- E79: limitation: threshold is provisional (not calibrated)
- E80: partial coverage (2 of 3 examined)
- E80: limitation: F5: growth evidence is from S1, not S17; not subtracted
- E80: limitation: off-heap: no gap series for S17; run dla gap S17 first (node.external is used when present)
- E80: limitation: threshold is provisional (not calibrated)

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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (e42033c209ab) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (81da0f869f69) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (81da0f869f69) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (3db4a155bb4e) | alloc/A1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S2.json (91e3396c8ca5) | trend/T3.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (91e3396c8ca5) | trend/T3.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S5.json (dc7155a0b8d5) | alloc/A2.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S7.json (18d3028fb4e1) | alloc/A3.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S9.json (a8c9eed6126b) | alloc/A4.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S11.json (1b3f1b4d1284) | alloc/A5.json |
| E19 | growth-series | test | dla 0.1.0 | artifact series/S4.json (37b20cbcf73f) | trend/T8.json |
| E20 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (37b20cbcf73f) | trend/T8.json |
| E21 | growth-series | test | dla 0.1.0 | artifact series/S6.json (42e55b9106dc) | trend/T9.json |
| E22 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (42e55b9106dc) | trend/T9.json |
| E23 | growth-series | test | dla 0.1.0 | artifact series/S8.json (dae26a09fbe5) | trend/T10.json |
| E24 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (dae26a09fbe5) | trend/T10.json |
| E25 | growth-series | test | dla 0.1.0 | artifact series/S10.json (0f8dd8f095b4) | trend/T11.json |
| E26 | unit-count-series | test | dla 0.1.0 | artifact series/S10.json (0f8dd8f095b4) | trend/T11.json |
| E27 | discriminator | test | dla 0.1.0 | artifact series/S13.json (9c0627dffea5) | alloc/A6.json |
| E28 | discriminator | test | dla 0.1.0 | artifact series/S15.json (4ae4b966c0ef) | alloc/A7.json |
| E29 | discriminator | test | dla 0.1.0 | artifact series/S17.json (dea11248485d) | alloc/A8.json |
| E30 | growth-series | test | dla 0.1.0 | artifact series/S1.json (81da0f869f69) | trend/T15.json |
| E31 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (81da0f869f69) | trend/T15.json |
| E32 | growth-series | test | dla 0.1.0 | artifact series/S2.json (91e3396c8ca5) | trend/T16.json |
| E33 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (91e3396c8ca5) | trend/T16.json |
| E34 | growth-series | test | dla 0.1.0 | artifact series/S3.json (3db4a155bb4e) | trend/T17.json |
| E35 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (3db4a155bb4e) | trend/T17.json |
| E36 | growth-series | test | dla 0.1.0 | artifact series/S4.json (37b20cbcf73f) | trend/T18.json |
| E37 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (37b20cbcf73f) | trend/T18.json |
| E38 | growth-series | test | dla 0.1.0 | artifact series/S5.json (dc7155a0b8d5) | trend/T19.json |
| E39 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (dc7155a0b8d5) | trend/T19.json |
| E40 | growth-series | test | dla 0.1.0 | artifact series/S6.json (42e55b9106dc) | trend/T20.json |
| E41 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (42e55b9106dc) | trend/T20.json |
| E42 | growth-series | test | dla 0.1.0 | artifact series/S7.json (18d3028fb4e1) | trend/T21.json |
| E43 | unit-count-series | test | dla 0.1.0 | artifact series/S7.json (18d3028fb4e1) | trend/T21.json |
| E44 | growth-series | test | dla 0.1.0 | artifact series/S8.json (dae26a09fbe5) | trend/T22.json |
| E45 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (dae26a09fbe5) | trend/T22.json |
| E46 | growth-series | test | dla 0.1.0 | artifact series/S9.json (a8c9eed6126b) | trend/T23.json |
| E47 | unit-count-series | test | dla 0.1.0 | artifact series/S9.json (a8c9eed6126b) | trend/T23.json |
| E48 | growth-series | test | dla 0.1.0 | artifact series/S10.json (0f8dd8f095b4) | trend/T24.json |
| E49 | unit-count-series | test | dla 0.1.0 | artifact series/S10.json (0f8dd8f095b4) | trend/T24.json |
| E50 | growth-series | test | dla 0.1.0 | artifact series/S11.json (1b3f1b4d1284) | trend/T25.json |
| E51 | unit-count-series | test | dla 0.1.0 | artifact series/S11.json (1b3f1b4d1284) | trend/T25.json |
| E52 | growth-series | test | dla 0.1.0 | artifact series/S12.json (edf94d25d634) | trend/T26.json |
| E53 | unit-count-series | test | dla 0.1.0 | artifact series/S12.json (edf94d25d634) | trend/T26.json |
| E54 | growth-series | test | dla 0.1.0 | artifact series/S13.json (9c0627dffea5) | trend/T27.json |
| E55 | unit-count-series | test | dla 0.1.0 | artifact series/S13.json (9c0627dffea5) | trend/T27.json |
| E56 | growth-series | test | dla 0.1.0 | artifact series/S14.json (376a45ca3736) | trend/T28.json |
| E57 | unit-count-series | test | dla 0.1.0 | artifact series/S14.json (376a45ca3736) | trend/T28.json |
| E58 | growth-series | test | dla 0.1.0 | artifact series/S15.json (4ae4b966c0ef) | trend/T29.json |
| E59 | unit-count-series | test | dla 0.1.0 | artifact series/S15.json (4ae4b966c0ef) | trend/T29.json |
| E60 | growth-series | test | dla 0.1.0 | artifact series/S16.json (f65cfdcde6d4) | trend/T30.json |
| E61 | unit-count-series | test | dla 0.1.0 | artifact series/S16.json (f65cfdcde6d4) | trend/T30.json |
| E62 | growth-series | test | dla 0.1.0 | artifact series/S17.json (dea11248485d) | trend/T31.json |
| E63 | unit-count-series | test | dla 0.1.0 | artifact series/S17.json (dea11248485d) | trend/T31.json |
| E64 | residual | test | dla 0.1.0 | artifact series/S1.json (81da0f869f69) | gap/residual-S1-E64.json |
| E65 | residual | test | dla 0.1.0 | artifact series/S2.json (91e3396c8ca5) | gap/residual-S2-E65.json |
| E66 | residual | test | dla 0.1.0 | artifact series/S3.json (3db4a155bb4e) | gap/residual-S3-E66.json |
| E67 | residual | test | dla 0.1.0 | artifact series/S4.json (37b20cbcf73f) | gap/residual-S4-E67.json |
| E68 | residual | test | dla 0.1.0 | artifact series/S5.json (dc7155a0b8d5) | gap/residual-S5-E68.json |
| E69 | residual | test | dla 0.1.0 | artifact series/S6.json (42e55b9106dc) | gap/residual-S6-E69.json |
| E70 | residual | test | dla 0.1.0 | artifact series/S7.json (18d3028fb4e1) | gap/residual-S7-E70.json |
| E71 | residual | test | dla 0.1.0 | artifact series/S8.json (dae26a09fbe5) | gap/residual-S8-E71.json |
| E72 | residual | test | dla 0.1.0 | artifact series/S9.json (a8c9eed6126b) | gap/residual-S9-E72.json |
| E73 | residual | test | dla 0.1.0 | artifact series/S10.json (0f8dd8f095b4) | gap/residual-S10-E73.json |
| E74 | residual | test | dla 0.1.0 | artifact series/S11.json (1b3f1b4d1284) | gap/residual-S11-E74.json |
| E75 | residual | test | dla 0.1.0 | artifact series/S12.json (edf94d25d634) | gap/residual-S12-E75.json |
| E76 | residual | test | dla 0.1.0 | artifact series/S13.json (9c0627dffea5) | gap/residual-S13-E76.json |
| E77 | residual | test | dla 0.1.0 | artifact series/S14.json (376a45ca3736) | gap/residual-S14-E77.json |
| E78 | residual | test | dla 0.1.0 | artifact series/S15.json (4ae4b966c0ef) | gap/residual-S15-E78.json |
| E79 | residual | test | dla 0.1.0 | artifact series/S16.json (f65cfdcde6d4) | gap/residual-S16-E79.json |
| E80 | residual | test | dla 0.1.0 | artifact series/S17.json (dea11248485d) | gap/residual-S17-E80.json |
