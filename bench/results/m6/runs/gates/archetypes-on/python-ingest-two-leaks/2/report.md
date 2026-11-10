# DLA report

Run: `20261110-165802-e976`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 18 enumerated, 18 dispositioned; confirmed 2, strong 0, refuted 2, suspected 0, unresolved 14

Residual: heap 2681.604356060606 bytes/n unattributed of 2681.604356060606 total (threshold 670.4010890151515) ABOVE THRESHOLD, see O1; heap 2679.943693181818 bytes/n unattributed of 2679.943693181818 total (threshold 669.9859232954545) ABOVE THRESHOLD, see O1; heap 2679.313455988456 bytes/n unattributed of 2679.313455988456 total (threshold 669.828363997114) ABOVE THRESHOLD, see O1; heap 2679.9815909090908 bytes/n unattributed of 2679.9815909090908 total (threshold 669.9953977272727) ABOVE THRESHOLD, see O1; heap 2679.2618145743145 bytes/n unattributed of 2679.2618145743145 total (threshold 669.8154536435786) ABOVE THRESHOLD, see O1

Coverage: python 3520 iterations, 8 of 11 entry points exercised

## Findings

### F1: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Module global ingest.parser._newest keeps the last Record; every Record.previous points at the prior Record, so the whole chain of all parsed records is reachable and never freed (unbounded linked list through the previous field).)
- Growth: `probe:ingest.parser._newest` 15.97 count/n (steady, gate pass, E12)
- Evidence classes: context, discriminator, growth, static
- Holder: static/global
- Owner: `ingest.parser._newest`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:3`
- Writers: `ingest/parser.py:29`
- Annotation: heuristic name match over the project's sources
- Verified: no
- Sibling hypotheses: F1.n1
- Supporting evidence: E4, E10, E12, E17
- Explanation (observed): ingest/parser.py:3 holder _newest (module global) is reassigned at parser.py:29 and each Record keeps .previous (parser.py:26) to the prior record, so every record parsed stays reachable; the chain is never cut or removed.
- Explanation structure: adds: ingest/parser.py:29; removal: never; evidence: E12

### F4: **L-REGISTER**

- State: confirmed
- Label: L-REGISTER (evidence: E12 unit-count-series)
- Growth: `probe:ingest.registry._LISTENERS` 7.974 count/n (steady, gate pass, E12)
- Evidence classes: context, discriminator, growth, static
- Holder: registration-table
- Owner: `ingest.registry._LISTENERS`
- Region: heap
- Shape: steady
- Site: `ingest/registry.py:3`
- Writers: `ingest/registry.py:7`
- Removers: `ingest/registry.py:24`
- Annotation: heuristic name match over the project's sources
- Verified: no
- Sibling hypotheses: F4.n1
- Supporting evidence: E4, E10, E12, E18
- Explanation (observed): ingest/registry.py:3 holder _LISTENERS gets a CheckpointWatcher per batch from pipeline.py:28 via subscribe (registry.py:7); the only remover, reset() at registry.py:24, is never reached in the worker (tests only), so one watcher per batch is retained.
- Explanation structure: adds: ingest/registry.py:7; removal: unreachable at ingest/registry.py:24; evidence: E12

### F2: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: container-field
- Owner: `ingest.pipeline.Pipeline.per_sensor`
- Region: heap
- Site: `ingest/pipeline.py:15`
- Writers: `ingest/pipeline.py:25`
- Annotation: heuristic name match over the project's sources
- Reason: time_budget
- Remedy: Duplicate or symptom column of confirmed F1 (parser chain) / F4 (registry listeners); not separately tested to stay in budget. F2 per_sensor is bounded by 64 sensors.
- Supporting evidence: E4

### F3: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Holder: registration-table
- Owner: `ingest.pipeline.Pipeline.process_batch`
- Region: heap
- Site: `ingest/pipeline.py:28`
- Reason: time_budget
- Remedy: Duplicate or symptom column of confirmed F1 (parser chain) / F4 (registry listeners); not separately tested to stay in budget. F2 per_sensor is bounded by 64 sensors.
- Supporting evidence: E4

### F5: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Site: `ingest/parser.py:29`
- Reason: time_budget
- Remedy: Duplicate or symptom column of confirmed F1 (parser chain) / F4 (registry listeners); not separately tested to stay in budget. F2 per_sensor is bounded by 64 sensors.
- Supporting evidence: E7

### F6: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: static
- Site: `ingest/registry.py:3`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: time_budget
- Remedy: Duplicate or symptom column of confirmed F1 (parser chain) / F4 (registry listeners); not separately tested to stay in budget. F2 per_sensor is bounded by 64 sensors.
- Supporting evidence: E7

### F7: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: Duplicate or symptom column of confirmed F1 (parser chain) / F4 (registry listeners); not separately tested to stay in budget. F2 per_sensor is bounded by 64 sensors.
- Supporting evidence: E19

### F8: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: Duplicate or symptom column of confirmed F1 (parser chain) / F4 (registry listeners); not separately tested to stay in budget. F2 per_sensor is bounded by 64 sensors.
- Supporting evidence: E20

### F9: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: Duplicate or symptom column of confirmed F1 (parser chain) / F4 (registry listeners); not separately tested to stay in budget. F2 per_sensor is bounded by 64 sensors.
- Supporting evidence: E21

### F10: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: Duplicate or symptom column of confirmed F1 (parser chain) / F4 (registry listeners); not separately tested to stay in budget. F2 per_sensor is bounded by 64 sensors.
- Supporting evidence: E22

### F11: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: Duplicate or symptom column of confirmed F1 (parser chain) / F4 (registry listeners); not separately tested to stay in budget. F2 per_sensor is bounded by 64 sensors.
- Supporting evidence: E24

### F12: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: Duplicate or symptom column of confirmed F1 (parser chain) / F4 (registry listeners); not separately tested to stay in budget. F2 per_sensor is bounded by 64 sensors.
- Supporting evidence: E26

### F13: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: discriminator
- Reason: time_budget
- Remedy: Duplicate or symptom column of confirmed F1 (parser chain) / F4 (registry listeners); not separately tested to stay in budget. F2 per_sensor is bounded by 64 sensors.
- Supporting evidence: E17

### F14: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: discriminator
- Reason: time_budget
- Remedy: Duplicate or symptom column of confirmed F1 (parser chain) / F4 (registry listeners); not separately tested to stay in budget. F2 per_sensor is bounded by 64 sensors.
- Supporting evidence: E18

### F15: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: time_budget
- Remedy: Duplicate or symptom column of confirmed F1 (parser chain) / F4 (registry listeners); not separately tested to stay in budget. F2 per_sensor is bounded by 64 sensors.
- Supporting evidence: E28

### F16: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: time_budget
- Remedy: Duplicate or symptom column of confirmed F1 (parser chain) / F4 (registry listeners); not separately tested to stay in budget. F2 per_sensor is bounded by 64 sensors.
- Supporting evidence: E30

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E17
  - Ruled out by: plateau+purge (leak) E17
  - Hypothesis: N-WARM
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E17
  - Originally supported by: E12
- **F4.n1** (UNCLASSIFIED): discriminator E18
  - Ruled out by: plateau+purge (leak) E18
  - Hypothesis: N-WARM
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E18
  - Originally supported by: E12

## Blind spots and unknowns

### Coverage gaps

- python/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- python/codeql: missing (codeql is not on PATH or in the tools directory). Install hint: dla install codeql
- python/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)
- python/memray: missing (memray is not installed in the target's python interpreter, so --capture alloc has the tracemalloc report only). Install hint: pip install memray in the interpreter the workload runs under, then dla harness run <H> --capture alloc

### Runtime blind spots

None recorded.

### Entry points not exercisable

None recorded.

### Evidence limitations and unknowns

- E4: partial coverage
- E4: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E4: limitation: writers and removers are textual name matches
- E8: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E8: limitation: workloads were discovered by reading files; none was run
- E9: limitation: harness written; nothing was run (dla harness run measures it)
- E9: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E10: partial coverage (9 of 9 examined)
- E10: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E10: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E10: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E10: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E10: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E11: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E11: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E11: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E11: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E11: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E12: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E12: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E12: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E12: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: tracemalloc sees allocations made after it started (interpreter start-up excluded) and only through Python's allocators; memory a C extension takes with malloc is not in it
- E13: limitation: each file lists the largest stacks by live bytes and, after the first, the stacks that grew most since the first file: a stack outside the list has no figure in that file
- E13: limitation: stacks are Python frames only, as deep as DLA_TRACEMALLOC_FRAMES (default 10); function names are the module of the frame
- E13: limitation: the files are ordered as given
- E14: limitation: tracemalloc sees allocations made after it started (interpreter start-up excluded) and only through Python's allocators; memory a C extension takes with malloc is not in it
- E14: limitation: each file lists the largest stacks by live bytes and, after the first, the stacks that grew most since the first file: a stack outside the list has no figure in that file
- E14: limitation: stacks are Python frames only, as deep as DLA_TRACEMALLOC_FRAMES (default 10); function names are the module of the frame
- E14: limitation: the files are ordered as given
- E14: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E15: partial coverage
- E15: limitation: the agent lists the 300 types with the most instances, so a type outside the list at a capture has no figure there
- E15: limitation: bytes are shallow sys.getsizeof sizes: what an object points to is counted under its own type
- E15: limitation: objects are those the garbage collector tracks, counted after a full collection; the files are ordered as given
- E15: limitation: each histogram lists only the largest types; a type outside the list has no figure
- E16: limitation: the agent lists the 300 types with the most instances, so a type outside the list at a capture has no figure there
- E16: limitation: bytes are shallow sys.getsizeof sizes: what an object points to is counted under its own type
- E16: limitation: objects are those the garbage collector tracks, counted after a full collection; the files are ordered as given
- E16: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E17: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E17: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E17: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E17: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E17: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E17: limitation: points 1 to 9 come from S1; the extra points from S4
- E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E17: limitation: gate and plateau thresholds are provisional (not calibrated)
- E18: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E18: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E18: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E18: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E18: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E18: limitation: points 1 to 9 come from S1; the extra points from S6
- E18: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E18: limitation: gate and plateau thresholds are provisional (not calibrated)
- E19: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E19: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E20: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E21: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E22: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E27: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E27: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E27: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E27: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E27: limitation: points 1 to 9 come from S1; the extra points from S4
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E28: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E28: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E28: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E28: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E28: limitation: points 1 to 9 come from S1; the extra points from S4
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E29: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E29: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E29: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E29: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E29: limitation: points 1 to 9 come from S1; the extra points from S6
- E29: limitation: gate thresholds are provisional (not calibrated)
- E30: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E30: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E30: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E30: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E30: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E30: limitation: points 1 to 9 come from S1; the extra points from S6
- E30: limitation: gate thresholds are provisional (not calibrated)
- E31: partial coverage (1 of 3 examined)
- E31: limitation: F1: growth measured as a count; bytes not attributed
- E31: limitation: F4: growth measured as a count; bytes not attributed
- E31: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E31: limitation: threshold is provisional (not calibrated)
- E32: partial coverage (1 of 3 examined)
- E32: limitation: F1: growth evidence is from S1, not S4; not subtracted
- E32: limitation: F4: growth evidence is from S1, not S4; not subtracted
- E32: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E32: limitation: threshold is provisional (not calibrated)
- E33: partial coverage (1 of 3 examined)
- E33: limitation: F1: growth evidence is from S1, not S5; not subtracted
- E33: limitation: F4: growth evidence is from S1, not S5; not subtracted
- E33: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E33: limitation: threshold is provisional (not calibrated)
- E34: partial coverage (1 of 3 examined)
- E34: limitation: F1: growth evidence is from S1, not S6; not subtracted
- E34: limitation: F4: growth evidence is from S1, not S6; not subtracted
- E34: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E34: limitation: threshold is provisional (not calibrated)
- E35: partial coverage (1 of 3 examined)
- E35: limitation: F1: growth evidence is from S1, not S7; not subtracted
- E35: limitation: F4: growth evidence is from S1, not S7; not subtracted
- E35: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
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
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E1 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | — |
| E2 | run-config | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | probe/probe.json |
| E4 | census-holder | static | dla-census 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | census/python.json |
| E5 | static-hit | static | ruff 0.16.9 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | sweep/python-ruff.json |
| E6 | static-hit | static | pylint 4.1.3 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | sweep/python-pylint.json |
| E7 | static-hit | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | sweep/python-dla.json |
| E8 | workload | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | workload/workloads.json |
| E9 | workload | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | harness/H1/harness.json |
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (67087c50def0) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (ac61240c9467) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (ac61240c9467) | trend/T1.json |
| E13 | alloc-stack | live | py-tracemalloc 0.1.0 | artifact allocs/A1.json (2ed121e2426c) | allocs/A1.json |
| E14 | growth-series | live | py-tracemalloc 0.1.0 | artifact series/S2.json (a3e0794a4701) | series/S2.json |
| E15 | histogram-diff | live | py-types 0.1.0 | artifact histograms/H1.json (eef4d60e771f) | histograms/H1.json |
| E16 | growth-series | live | py-types 0.1.0 | artifact series/S3.json (c84c8a90119c) | series/S3.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S5.json (9828b8f4f6cf) | alloc/A1.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S7.json (16a37f43e3d1) | alloc/A2.json |
| E19 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (a3e0794a4701) | trend/T4.json |
| E20 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (a3e0794a4701) | trend/T4.json |
| E21 | growth-series | dump | dla 0.1.0 | artifact series/S3.json (c84c8a90119c) | trend/T5.json |
| E22 | unit-count-series | dump | dla 0.1.0 | artifact series/S3.json (c84c8a90119c) | trend/T5.json |
| E23 | growth-series | test | dla 0.1.0 | artifact series/S4.json (13a557e9ac39) | trend/T6.json |
| E24 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (13a557e9ac39) | trend/T6.json |
| E25 | growth-series | test | dla 0.1.0 | artifact series/S6.json (cb86e5af9729) | trend/T7.json |
| E26 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (cb86e5af9729) | trend/T7.json |
| E27 | growth-series | test | dla 0.1.0 | artifact series/S5.json (9828b8f4f6cf) | trend/T8.json |
| E28 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (9828b8f4f6cf) | trend/T8.json |
| E29 | growth-series | test | dla 0.1.0 | artifact series/S7.json (16a37f43e3d1) | trend/T9.json |
| E30 | unit-count-series | test | dla 0.1.0 | artifact series/S7.json (16a37f43e3d1) | trend/T9.json |
| E31 | residual | test | dla 0.1.0 | artifact series/S1.json (ac61240c9467) | gap/residual-S1-E31.json |
| E32 | residual | test | dla 0.1.0 | artifact series/S4.json (13a557e9ac39) | gap/residual-S4-E32.json |
| E33 | residual | test | dla 0.1.0 | artifact series/S5.json (9828b8f4f6cf) | gap/residual-S5-E33.json |
| E34 | residual | test | dla 0.1.0 | artifact series/S6.json (cb86e5af9729) | gap/residual-S6-E34.json |
| E35 | residual | test | dla 0.1.0 | artifact series/S7.json (16a37f43e3d1) | gap/residual-S7-E35.json |
