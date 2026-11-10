# DLA report

Run: `20261110-182046-866f`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 23 enumerated, 23 dispositioned; confirmed 0, strong 2, refuted 2, suspected 0, unresolved 19

Residual: heap 2681.478787878788 bytes/n unattributed of 2681.478787878788 total (threshold 670.369696969697) ABOVE THRESHOLD, see O1; heap 1978.3517045454544 bytes/n unattributed of 1978.3517045454544 total (threshold 494.5879261363637) ABOVE THRESHOLD, see O2; heap 799.8948863636363 bytes/n unattributed of 799.8948863636363 total (threshold 199.9737215909091) ABOVE THRESHOLD, see O2; heap 2680.034318181818 bytes/n unattributed of 2680.034318181818 total (threshold 670.0085795454545) ABOVE THRESHOLD, see O1; heap 2679.1959505772006 bytes/n unattributed of 2679.1959505772006 total (threshold 669.7989876443002) ABOVE THRESHOLD, see O1; heap 2679.7713636363637 bytes/n unattributed of 2679.7713636363637 total (threshold 669.9428409090909) ABOVE THRESHOLD, see O1; heap 2679.2693813131314 bytes/n unattributed of 2679.2693813131314 total (threshold 669.8173453282828) ABOVE THRESHOLD, see O1; heap 2679.984659090909 bytes/n unattributed of 2679.984659090909 total (threshold 669.9961647727273) ABOVE THRESHOLD, see O1; heap 2679.217523448773 bytes/n unattributed of 2679.217523448773 total (threshold 669.8043808621933) ABOVE THRESHOLD, see O1

Coverage: python 3520 iterations, 8 of 11 entry points exercised

## Findings

### F3: **UNCLASSIFIED**

- State: strong
- Label: UNCLASSIFIED (Module-global ingest.parser._newest heads a singly linked chain: Record.previous references the prior Record, so every parsed Record stays reachable and grows with rows ingested.)
- Growth: `probe:ingest.parser._newest` 15.97 count/n (steady, gate pass, E12)
- Evidence classes: context, discriminator, growth, intervention, static
- Strong: second class E38 intervention that removed the growth, independent of the growth record E12 (dla)
- Holder: static/global
- Owner: `ingest.parser._newest`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:3`
- Writers: `ingest/parser.py:29`
- Annotation: heuristic name match over the project's sources
- Annotation: module global _newest keeps the last Record; each Record.previous links to the prior one so the whole chain of every parsed record is retained
- Verified: no
- Sibling hypotheses: F3.n1
- Supporting evidence: E7, E10, E21, E12, E38
- Explanation (inferred): Static global ingest.parser._newest (parser.py:3) is reassigned at parser.py:29 per record and Record.previous links back, so the chain of all Records is never released; nothing removes it.
- Explanation structure: adds: ingest/parser.py:29; removal: never

### F6: **UNCLASSIFIED**

- State: strong
- Label: UNCLASSIFIED (Module-global registry._LISTENERS['checkpoint'] gets a new CheckpointWatcher per batch (pipeline.py:28) and nothing unsubscribes it; watchers (with marks dicts) accumulate per batch.)
- Growth: `probe:ingest.registry._LISTENERS` 7.974 count/n (steady, gate pass, E12)
- Evidence classes: context, discriminator, growth, intervention, static
- Strong: second class E39 intervention that removed the growth, independent of the growth record E12 (dla)
- Holder: registration-table
- Owner: `ingest.registry._LISTENERS`
- Region: heap
- Shape: steady
- Site: `ingest/registry.py:3`
- Writers: `ingest/registry.py:7`, `ingest/pipeline.py:28`
- Removers: `ingest/registry.py:24`
- Annotation: heuristic name match over the project's sources
- Annotation: every batch subscribes a CheckpointWatcher that is never unsubscribed; each watcher holds a marks dict sized to all known sensors
- Verified: no
- Sibling hypotheses: F6.n1
- Supporting evidence: E7, E10, E22, E12, E39
- Explanation (inferred): Registration table ingest.registry._LISTENERS (registry.py:3) gets a CheckpointWatcher added at pipeline.py:28 every batch; reset() (registry.py:24) clears it only when tests call it, never in production, so watchers accumulate.
- Explanation structure: adds: ingest/pipeline.py:28; removal: conditional at ingest/registry.py:24

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `ingest/parser.py:29`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F3 (same parser._newest chain, ingest/parser.py:29); tracked there
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `ingest/registry.py:3`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F6 (registry._LISTENERS fed at ingest/pipeline.py:28); tracked there
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `ingest.pipeline.Pipeline.per_sensor`
- Region: heap
- Site: `ingest/pipeline.py:15`
- Writers: `ingest/pipeline.py:25`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: per_sensor is keyed by sensor id (64 distinct in the load generator, overwrite not append) so likely bounded; no probe column of its own exists. Add a driver probe on Pipeline.per_sensor to refute.
- Supporting evidence: E7

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `ingest.pipeline.Pipeline.process_batch`
- Region: heap
- Site: `ingest/pipeline.py:28`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F6 (registry._LISTENERS fed at ingest/pipeline.py:28); tracked there
- Supporting evidence: E7

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `ingest.parser._newest`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:29`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F3 (same parser._newest chain, ingest/parser.py:29); tracked there
- Supporting evidence: E12

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `ingest.registry._LISTENERS`
- Region: heap
- Shape: steady
- Site: `ingest/pipeline.py:28`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F6 (registry._LISTENERS fed at ingest/pipeline.py:28); tracked there
- Supporting evidence: E12

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `ingest.registry._LISTENERS`
- Region: heap
- Shape: steady
- Site: `ingest/watchers.py:8`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F6 (registry._LISTENERS fed at ingest/pipeline.py:28); tracked there
- Supporting evidence: E20

### F10: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `ingest.parser._newest`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:24`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F3 (same parser._newest chain, ingest/parser.py:29); tracked there
- Supporting evidence: E20

### F11: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `ingest.parser._newest`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:25`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F3 (same parser._newest chain, ingest/parser.py:29); tracked there
- Supporting evidence: E20

### F12: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `ingest.parser._newest`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:16`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F3 (same parser._newest chain, ingest/parser.py:29); tracked there
- Supporting evidence: E20

### F13: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `ingest.parser._newest`
- Region: heap
- Shape: steady
- Site: `ingest/pipeline.py:27`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F3 (same parser._newest chain, ingest/parser.py:29); tracked there
- Supporting evidence: E20

### F14: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `ingest.parser._newest`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:29`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F3 (same parser._newest chain, ingest/parser.py:29); tracked there
- Supporting evidence: E17, E18

### F15: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `ingest.registry._LISTENERS`
- Region: heap
- Shape: steady
- Site: `ingest/watchers.py:8`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F6 (registry._LISTENERS fed at ingest/pipeline.py:28); tracked there
- Supporting evidence: E19, E20

### F16: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `ingest.parser._newest`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:29`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F3/F6 (same holder and site); tracked there
- Supporting evidence: E24, E26, E28, E30, E33

### F17: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `ingest.registry._LISTENERS`
- Region: heap
- Shape: steady
- Site: `ingest/pipeline.py:28`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F3/F6 (same holder and site); tracked there
- Supporting evidence: E24, E26, E28, E30, E33

### F18: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `ingest.parser._newest`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:29`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F3/F6 (same holder and site); tracked there
- Supporting evidence: E37, E35

### F19: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `ingest.registry._LISTENERS`
- Region: heap
- Shape: steady
- Site: `ingest/pipeline.py:28`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F3/F6 (same holder and site); tracked there
- Supporting evidence: E37, E35

### F20: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `ingest.registry._LISTENERS`
- Region: heap
- Shape: steady
- Site: `ingest/pipeline.py:28`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F3/F6: scratch intervention series; tracked there
- Supporting evidence: E49, E50, E51, E52

### F21: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `ingest.parser._newest`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:29`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F3/F6: scratch intervention series; tracked there
- Supporting evidence: E49, E50, E51, E52

## Ruled out

- **F3.n1** (UNCLASSIFIED): discriminator E21
  - Ruled out by: plateau+purge (leak) E21
  - Hypothesis: N-WARM: growth is a warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles of alloc-test
  - Contradicting evidence: E21
  - Originally supported by: E12
- **F6.n1** (UNCLASSIFIED): discriminator E22
  - Ruled out by: plateau+purge (leak) E22
  - Hypothesis: N-WARM: growth is a warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles of alloc-test
  - Contradicting evidence: E22
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

- E7: partial coverage
- E7: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E7: limitation: writers and removers are textual name matches
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
- E13: partial coverage
- E13: limitation: the agent lists the 300 types with the most instances, so a type outside the list at a capture has no figure there
- E13: limitation: bytes are shallow sys.getsizeof sizes: what an object points to is counted under its own type
- E13: limitation: objects are those the garbage collector tracks, counted after a full collection; the files are ordered as given
- E13: limitation: each histogram lists only the largest types; a type outside the list has no figure
- E14: limitation: the agent lists the 300 types with the most instances, so a type outside the list at a capture has no figure there
- E14: limitation: bytes are shallow sys.getsizeof sizes: what an object points to is counted under its own type
- E14: limitation: objects are those the garbage collector tracks, counted after a full collection; the files are ordered as given
- E14: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E15: limitation: tracemalloc sees allocations made after it started (interpreter start-up excluded) and only through Python's allocators; memory a C extension takes with malloc is not in it
- E15: limitation: each file lists the largest stacks by live bytes and, after the first, the stacks that grew most since the first file: a stack outside the list has no figure in that file
- E15: limitation: stacks are Python frames only, as deep as DLA_TRACEMALLOC_FRAMES (default 10); function names are the module of the frame
- E15: limitation: the files are ordered as given
- E16: limitation: tracemalloc sees allocations made after it started (interpreter start-up excluded) and only through Python's allocators; memory a C extension takes with malloc is not in it
- E16: limitation: each file lists the largest stacks by live bytes and, after the first, the stacks that grew most since the first file: a stack outside the list has no figure in that file
- E16: limitation: stacks are Python frames only, as deep as DLA_TRACEMALLOC_FRAMES (default 10); function names are the module of the frame
- E16: limitation: the files are ordered as given
- E16: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E17: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E17: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E18: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E19: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E20: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E21: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E21: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E21: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E21: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E21: limitation: points 1 to 9 come from S1; the extra points from S4
- E21: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E21: limitation: gate and plateau thresholds are provisional (not calibrated)
- E22: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E22: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E22: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E22: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E22: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E22: limitation: points 1 to 9 come from S1; the extra points from S6
- E22: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E22: limitation: gate and plateau thresholds are provisional (not calibrated)
- E23: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E25: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E25: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E25: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E25: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E25: limitation: points 1 to 9 come from S1; the extra points from S4
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E26: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E26: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E26: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E26: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E26: limitation: points 1 to 9 come from S1; the extra points from S4
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
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
- E31: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E31: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E31: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E31: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E31: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E31: limitation: points 1 to 9 come from S1; the extra points from S8
- E31: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E31: limitation: gate and plateau thresholds are provisional (not calibrated)
- E32: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E32: limitation: gate thresholds are provisional (not calibrated)
- E33: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E33: limitation: gate thresholds are provisional (not calibrated)
- E34: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E34: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E34: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E34: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E34: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E34: limitation: points 1 to 9 come from S1; the extra points from S8
- E34: limitation: gate thresholds are provisional (not calibrated)
- E35: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E35: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E35: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E35: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E35: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E35: limitation: points 1 to 9 come from S1; the extra points from S8
- E35: limitation: gate thresholds are provisional (not calibrated)
- E36: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E36: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E36: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E36: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E36: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E36: limitation: points 1 to 9 come from S1; the extra points from S8
- E36: limitation: gate thresholds are provisional (not calibrated)
- E37: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E37: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E37: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E37: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E37: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E37: limitation: points 1 to 9 come from S1; the extra points from S8
- E37: limitation: gate thresholds are provisional (not calibrated)
- E38: limitation: tested in a scratch copy; the project was not changed
- E38: limitation: gate thresholds are provisional (not calibrated); the 0.2 and 0.5 slope fractions are placeholders
- E38: limitation: exercised is null: the harness records entry points, not lines, so whether the edited line ran is not checked
- E38: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E38: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E39: limitation: tested in a scratch copy; the project was not changed
- E39: limitation: gate thresholds are provisional (not calibrated); the 0.2 and 0.5 slope fractions are placeholders
- E39: limitation: exercised is null: the harness records entry points, not lines, so whether the edited line ran is not checked
- E39: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E39: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E40: partial coverage (1 of 3 examined)
- E40: limitation: F3: growth measured as a count; bytes not attributed
- E40: limitation: F6: growth measured as a count; bytes not attributed
- E40: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E40: limitation: threshold is provisional (not calibrated)
- E41: partial coverage (1 of 3 examined)
- E41: limitation: F3: growth evidence is from S1, not S4; not subtracted
- E41: limitation: F6: growth evidence is from S1, not S4; not subtracted
- E41: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E41: limitation: threshold is provisional (not calibrated)
- E42: partial coverage (1 of 3 examined)
- E42: limitation: F3: growth evidence is from S1, not S5; not subtracted
- E42: limitation: F6: growth evidence is from S1, not S5; not subtracted
- E42: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E42: limitation: threshold is provisional (not calibrated)
- E43: partial coverage (1 of 3 examined)
- E43: limitation: F3: growth evidence is from S1, not S6; not subtracted
- E43: limitation: F6: growth evidence is from S1, not S6; not subtracted
- E43: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E43: limitation: threshold is provisional (not calibrated)
- E44: partial coverage (1 of 3 examined)
- E44: limitation: F3: growth evidence is from S1, not S7; not subtracted
- E44: limitation: F6: growth evidence is from S1, not S7; not subtracted
- E44: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E44: limitation: threshold is provisional (not calibrated)
- E45: partial coverage (1 of 3 examined)
- E45: limitation: F3: growth evidence is from S1, not S8; not subtracted
- E45: limitation: F6: growth evidence is from S1, not S8; not subtracted
- E45: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E45: limitation: threshold is provisional (not calibrated)
- E46: partial coverage (1 of 3 examined)
- E46: limitation: F3: growth evidence is from S1, not S9; not subtracted
- E46: limitation: F6: growth evidence is from S1, not S9; not subtracted
- E46: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E46: limitation: threshold is provisional (not calibrated)
- E47: partial coverage (1 of 3 examined)
- E47: limitation: F3: growth evidence is from S1, not S10; not subtracted
- E47: limitation: F6: growth evidence is from S1, not S10; not subtracted
- E47: limitation: off-heap: no gap series for S10; run dla gap S10 first (node.external is used when present)
- E47: limitation: threshold is provisional (not calibrated)
- E48: partial coverage (1 of 3 examined)
- E48: limitation: F3: growth evidence is from S1, not S11; not subtracted
- E48: limitation: F6: growth evidence is from S1, not S11; not subtracted
- E48: limitation: off-heap: no gap series for S11; run dla gap S11 first (node.external is used when present)
- E48: limitation: threshold is provisional (not calibrated)
- E49: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E49: limitation: gate thresholds are provisional (not calibrated)
- E50: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E50: limitation: gate thresholds are provisional (not calibrated)
- E51: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E51: limitation: gate thresholds are provisional (not calibrated)
- E52: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E52: limitation: gate thresholds are provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | — |
| E2 | run-config | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | probe/probe.json |
| E4 | static-hit | static | ruff 0.16.9 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | sweep/python-ruff.json |
| E5 | static-hit | static | pylint 4.1.3 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | sweep/python-pylint.json |
| E6 | static-hit | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | sweep/python-dla.json |
| E7 | census-holder | static | dla-census 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | census/python.json |
| E8 | workload | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | workload/workloads.json |
| E9 | workload | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | harness/H1/harness.json |
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (e56b64f2c594) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (350666f48e64) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (350666f48e64) | trend/T1.json |
| E13 | histogram-diff | live | py-types 0.1.0 | artifact histograms/H1.json (a37f66fdd70b) | histograms/H1.json |
| E14 | growth-series | live | py-types 0.1.0 | artifact series/S2.json (b3066361e4d1) | series/S2.json |
| E15 | alloc-stack | live | py-tracemalloc 0.1.0 | artifact allocs/A1.json (38fb32f91ec9) | allocs/A1.json |
| E16 | growth-series | live | py-tracemalloc 0.1.0 | artifact series/S3.json (b550990da6c7) | series/S3.json |
| E17 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (b3066361e4d1) | trend/T2.json |
| E18 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (b3066361e4d1) | trend/T2.json |
| E19 | growth-series | dump | dla 0.1.0 | artifact series/S3.json (b550990da6c7) | trend/T3.json |
| E20 | unit-count-series | dump | dla 0.1.0 | artifact series/S3.json (b550990da6c7) | trend/T3.json |
| E21 | discriminator | test | dla 0.1.0 | artifact series/S5.json (3950d54335f4) | alloc/A1.json |
| E22 | discriminator | test | dla 0.1.0 | artifact series/S7.json (ca72336e65c2) | alloc/A2.json |
| E23 | growth-series | test | dla 0.1.0 | artifact series/S4.json (59ac65914dbf) | trend/T6.json |
| E24 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (59ac65914dbf) | trend/T6.json |
| E25 | growth-series | test | dla 0.1.0 | artifact series/S5.json (3950d54335f4) | trend/T7.json |
| E26 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (3950d54335f4) | trend/T7.json |
| E27 | growth-series | test | dla 0.1.0 | artifact series/S6.json (5750dc5a1b1c) | trend/T8.json |
| E28 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (5750dc5a1b1c) | trend/T8.json |
| E29 | growth-series | test | dla 0.1.0 | artifact series/S7.json (ca72336e65c2) | trend/T9.json |
| E30 | unit-count-series | test | dla 0.1.0 | artifact series/S7.json (ca72336e65c2) | trend/T9.json |
| E31 | discriminator | test | dla 0.1.0 | artifact series/S9.json (7f5e6b55e686) | alloc/A3.json |
| E32 | growth-series | test | dla 0.1.0 | artifact series/S8.json (b9465dfdd8b8) | trend/T11.json |
| E33 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (b9465dfdd8b8) | trend/T11.json |
| E34 | growth-series | test | dla 0.1.0 | artifact series/S9.json (7f5e6b55e686) | trend/T12.json |
| E35 | unit-count-series | test | dla 0.1.0 | artifact series/S9.json (7f5e6b55e686) | trend/T12.json |
| E36 | growth-series | test | dla 0.1.0 | artifact series/S9.json (7f5e6b55e686) | trend/T13.json |
| E37 | unit-count-series | test | dla 0.1.0 | artifact series/S9.json (7f5e6b55e686) | trend/T13.json |
| E38 | intervention | test | dla 0.1.0 | artifact series/S10.json (010a5b11179f) | intervene/I1 |
| E39 | intervention | test | dla 0.1.0 | artifact series/S11.json (080c5fc15b93) | intervene/I2 |
| E40 | residual | test | dla 0.1.0 | artifact series/S1.json (350666f48e64) | gap/residual-S1-E40.json |
| E41 | residual | test | dla 0.1.0 | artifact series/S4.json (59ac65914dbf) | gap/residual-S4-E41.json |
| E42 | residual | test | dla 0.1.0 | artifact series/S5.json (3950d54335f4) | gap/residual-S5-E42.json |
| E43 | residual | test | dla 0.1.0 | artifact series/S6.json (5750dc5a1b1c) | gap/residual-S6-E43.json |
| E44 | residual | test | dla 0.1.0 | artifact series/S7.json (ca72336e65c2) | gap/residual-S7-E44.json |
| E45 | residual | test | dla 0.1.0 | artifact series/S8.json (b9465dfdd8b8) | gap/residual-S8-E45.json |
| E46 | residual | test | dla 0.1.0 | artifact series/S9.json (7f5e6b55e686) | gap/residual-S9-E46.json |
| E47 | residual | test | dla 0.1.0 | artifact series/S10.json (010a5b11179f) | gap/residual-S10-E47.json |
| E48 | residual | test | dla 0.1.0 | artifact series/S11.json (080c5fc15b93) | gap/residual-S11-E48.json |
| E49 | growth-series | test | dla 0.1.0 | artifact series/S10.json (010a5b11179f) | trend/T16.json |
| E50 | unit-count-series | test | dla 0.1.0 | artifact series/S10.json (010a5b11179f) | trend/T16.json |
| E51 | growth-series | test | dla 0.1.0 | artifact series/S11.json (080c5fc15b93) | trend/T17.json |
| E52 | unit-count-series | test | dla 0.1.0 | artifact series/S11.json (080c5fc15b93) | trend/T17.json |
