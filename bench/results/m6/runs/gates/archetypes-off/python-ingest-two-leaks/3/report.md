# DLA report

Run: `20261110-193151-cf67`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 16 enumerated, 16 dispositioned; confirmed 2, strong 0, refuted 2, suspected 0, unresolved 12

Residual: heap 2681.5041666666666 bytes/n unattributed of 2681.5041666666666 total (threshold 670.3760416666667) ABOVE THRESHOLD, see O2; heap 2679.2199585137087 bytes/n unattributed of 2679.2199585137087 total (threshold 669.8049896284272) ABOVE THRESHOLD, see O2; heap 2680.0131818181817 bytes/n unattributed of 2680.0131818181817 total (threshold 670.0032954545454) ABOVE THRESHOLD, see O2; heap 2679.232196969697 bytes/n unattributed of 2679.232196969697 total (threshold 669.8080492424242) ABOVE THRESHOLD, see O2; heap 2679.837897727273 bytes/n unattributed of 2679.837897727273 total (threshold 669.9594744318182) ABOVE THRESHOLD, see O2; heap 2679.195481601731 bytes/n unattributed of 2679.195481601731 total (threshold 669.7988704004329) ABOVE THRESHOLD, see O2; heap 2679.920681818182 bytes/n unattributed of 2679.920681818182 total (threshold 669.9801704545455) ABOVE THRESHOLD, see O2; heap 2679.2601551226553 bytes/n unattributed of 2679.2601551226553 total (threshold 669.8150387806638) ABOVE THRESHOLD, see O2; heap 2679.974886363636 bytes/n unattributed of 2679.974886363636 total (threshold 669.993721590909) ABOVE THRESHOLD, see O2

Coverage: python 3520 iterations, 8 of 11 entry points exercised

## Findings

### F3: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (module global ingest.parser._newest keeps the last Record; every Record.previous points to the one parsed before, so all records ever parsed stay reachable (unbounded linked chain))
- Growth: `probe:ingest.parser._newest` 15.97 count/n (steady, gate pass, E12)
- Evidence classes: context, discriminator, growth, static
- Holder: static/global
- Owner: `ingest.parser._newest`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:3`
- Writers: `ingest/parser.py:29`
- Annotation: heuristic name match over the project's sources
- Annotation: module global _newest rebinds each parse; each Record.previous links to the prior record, so the whole record chain stays reachable
- Verified: no
- Sibling hypotheses: F3.n1
- Supporting evidence: E7, E10, E12, E13
- Explanation (inferred): Static/global holder ingest.parser._newest (ingest/parser.py:3); parse_line (parser.py:29) rebinds it and sets record.previous, so each Record chains to its predecessor and is never released.
- Explanation structure: adds: ingest/parser.py:29; removal: never

### F6: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (registry._LISTENERS['checkpoint'] gets a CheckpointWatcher per batch (pipeline.py:28) and nothing unsubscribes; each watcher holds a marks dict of all known sensors)
- Growth: `probe:ingest.registry._LISTENERS` 7.974 count/n (steady, gate pass, E12)
- Evidence classes: context, discriminator, growth, static
- Holder: registration-table
- Owner: `ingest.registry._LISTENERS`
- Region: heap
- Shape: steady
- Site: `ingest/registry.py:3`
- Writers: `ingest/registry.py:7`, `ingest/pipeline.py:28`
- Removers: `ingest/registry.py:24`
- Annotation: heuristic name match over the project's sources
- Annotation: listeners subscribed per batch, no unsubscribe; reset() only used by tests
- Verified: no
- Sibling hypotheses: F6.n1
- Supporting evidence: E7, E10, E12, E14
- Explanation (inferred): Registration-table ingest.registry._LISTENERS (registry.py:3); subscribe (registry.py:7) appends one CheckpointWatcher per batch from pipeline.py:28; removal happens only when reset (registry.py:24, tests only) runs, so in production watchers accumulate.
- Explanation structure: adds: ingest/registry.py:7; removal: conditional at ingest/registry.py:24

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `ingest/parser.py:29`
- Reason: not_exercised
- Remedy: detector hit duplicates confirmed F3 (same site parser.py:29); fix F3
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `ingest/registry.py:3`
- Reason: not_exercised
- Remedy: detector hit duplicates confirmed F6 (registry.py:3); fix F6
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
- Remedy: per_sensor keyed by 64 sensor names, bounded; no probe column grew; no separate evidence
- Supporting evidence: E7

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `ingest.pipeline.Pipeline.process_batch`
- Region: heap
- Site: `ingest/pipeline.py:28`
- Writers: `ingest/pipeline.py:28`
- Reason: not_exercised
- Remedy: registration call site is the writer of F6; no separate growth
- Supporting evidence: E7

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: discriminator, growth
- Holder: static/global
- Owner: `ingest.parser._newest`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:29`
- Reason: not_exercised
- Remedy: duplicate of F3/F6, added only to account for type-histogram columns (Record, CheckpointWatcher counts)
- Supporting evidence: E12, E14, E16, E18, E20, E22

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: discriminator, growth
- Holder: registration-table
- Owner: `ingest.registry._LISTENERS`
- Region: heap
- Shape: steady
- Site: `ingest/registry.py:3`
- Reason: not_exercised
- Remedy: duplicate of F3/F6, added only to account for type-histogram columns (Record, CheckpointWatcher counts)
- Supporting evidence: E12, E14, E16, E18, E20, E22

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `ingest.parser._newest`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:29`
- Reason: not_exercised
- Remedy: duplicate of F3/F6, added only to account for type-histogram columns (Record, CheckpointWatcher counts)
- Supporting evidence: E25, E26

### F10: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `ingest.registry._LISTENERS`
- Region: heap
- Shape: steady
- Site: `ingest/registry.py:3`
- Reason: not_exercised
- Remedy: duplicate of F3/F6, added only to account for type-histogram columns (Record, CheckpointWatcher counts)
- Supporting evidence: E25, E26

### F11: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `ingest.parser._newest`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:29`
- Reason: not_exercised
- Remedy: duplicate of F3/F6, added only to account for type-count columns
- Supporting evidence: E30, E32

### F12: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `ingest.registry._LISTENERS`
- Region: heap
- Shape: steady
- Site: `ingest/registry.py:3`
- Reason: not_exercised
- Remedy: duplicate of F3/F6, added only to account for type-count columns
- Supporting evidence: E30, E32

### F13: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `ingest.parser._newest`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:29`
- Reason: not_exercised
- Remedy: duplicate of F3/F6, added only to account for type-count columns
- Supporting evidence: E52, E54

### F14: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `ingest.registry._LISTENERS`
- Region: heap
- Shape: steady
- Site: `ingest/registry.py:3`
- Reason: not_exercised
- Remedy: duplicate of F3/F6, added only to account for type-count columns
- Supporting evidence: E52, E54

## Ruled out

- **F3.n1** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (leak) E13
  - Hypothesis: N-WARM: bounded warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E13
  - Originally supported by: E12
- **F6.n1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (leak) E14
  - Hypothesis: N-WARM: bounded warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E14
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
- E13: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E13: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E13: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E13: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E13: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E13: limitation: points 1 to 9 come from S1; the extra points from S2
- E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- E14: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E14: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E14: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E14: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E14: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E14: limitation: points 1 to 9 come from S1; the extra points from S4
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E17: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E17: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E17: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E17: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E17: limitation: points 1 to 9 come from S1; the extra points from S2
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E18: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E18: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E18: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E18: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E18: limitation: points 1 to 9 come from S1; the extra points from S2
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E21: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E21: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E21: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E21: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E21: limitation: points 1 to 9 come from S1; the extra points from S4
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E22: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E22: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E22: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E22: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E22: limitation: points 1 to 9 come from S1; the extra points from S4
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: partial coverage
- E23: limitation: the agent lists the 300 types with the most instances, so a type outside the list at a capture has no figure there
- E23: limitation: bytes are shallow sys.getsizeof sizes: what an object points to is counted under its own type
- E23: limitation: objects are those the garbage collector tracks, counted after a full collection; the files are ordered as given
- E23: limitation: each histogram lists only the largest types; a type outside the list has no figure
- E24: limitation: the agent lists the 300 types with the most instances, so a type outside the list at a capture has no figure there
- E24: limitation: bytes are shallow sys.getsizeof sizes: what an object points to is counted under its own type
- E24: limitation: objects are those the garbage collector tracks, counted after a full collection; the files are ordered as given
- E24: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E25: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E25: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E26: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E27: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E27: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E27: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E27: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E27: limitation: points 1 to 9 come from S1; the extra points from S7
- E27: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E27: limitation: gate and plateau thresholds are provisional (not calibrated)
- E28: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E28: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E28: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E28: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E28: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E28: limitation: points 1 to 9 come from S1; the extra points from S9
- E28: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E28: limitation: gate and plateau thresholds are provisional (not calibrated)
- E29: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E29: limitation: gate thresholds are provisional (not calibrated)
- E30: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E30: limitation: gate thresholds are provisional (not calibrated)
- E31: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E31: limitation: gate thresholds are provisional (not calibrated)
- E32: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E32: limitation: gate thresholds are provisional (not calibrated)
- E33: partial coverage (1 of 3 examined)
- E33: limitation: F3: growth measured as a count; bytes not attributed
- E33: limitation: F6: growth measured as a count; bytes not attributed
- E33: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E33: limitation: threshold is provisional (not calibrated)
- E34: partial coverage (1 of 3 examined)
- E34: limitation: F3: growth evidence is from S1, not S2; not subtracted
- E34: limitation: F6: growth evidence is from S1, not S2; not subtracted
- E34: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E34: limitation: threshold is provisional (not calibrated)
- E35: partial coverage (1 of 3 examined)
- E35: limitation: F3: growth evidence is from S1, not S3; not subtracted
- E35: limitation: F6: growth evidence is from S1, not S3; not subtracted
- E35: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E35: limitation: threshold is provisional (not calibrated)
- E36: partial coverage (1 of 3 examined)
- E36: limitation: F3: growth evidence is from S1, not S4; not subtracted
- E36: limitation: F6: growth evidence is from S1, not S4; not subtracted
- E36: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E36: limitation: threshold is provisional (not calibrated)
- E37: partial coverage (1 of 3 examined)
- E37: limitation: F3: growth evidence is from S1, not S5; not subtracted
- E37: limitation: F6: growth evidence is from S1, not S5; not subtracted
- E37: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E37: limitation: threshold is provisional (not calibrated)
- E38: partial coverage (1 of 3 examined)
- E38: limitation: F3: growth evidence is from S1, not S7; not subtracted
- E38: limitation: F6: growth evidence is from S1, not S7; not subtracted
- E38: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E38: limitation: threshold is provisional (not calibrated)
- E39: partial coverage (1 of 3 examined)
- E39: limitation: F3: growth evidence is from S1, not S8; not subtracted
- E39: limitation: F6: growth evidence is from S1, not S8; not subtracted
- E39: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E39: limitation: threshold is provisional (not calibrated)
- E40: partial coverage (1 of 3 examined)
- E40: limitation: F3: growth evidence is from S1, not S9; not subtracted
- E40: limitation: F6: growth evidence is from S1, not S9; not subtracted
- E40: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E40: limitation: threshold is provisional (not calibrated)
- E41: partial coverage (1 of 3 examined)
- E41: limitation: F3: growth evidence is from S1, not S10; not subtracted
- E41: limitation: F6: growth evidence is from S1, not S10; not subtracted
- E41: limitation: off-heap: no gap series for S10; run dla gap S10 first (node.external is used when present)
- E41: limitation: threshold is provisional (not calibrated)
- E42: partial coverage (1 of 3 examined)
- E42: limitation: F3: growth measured as a count; bytes not attributed
- E42: limitation: F6: growth measured as a count; bytes not attributed
- E42: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E42: limitation: threshold is provisional (not calibrated)
- E43: partial coverage (1 of 3 examined)
- E43: limitation: F3: growth evidence is from S1, not S2; not subtracted
- E43: limitation: F6: growth evidence is from S1, not S2; not subtracted
- E43: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E43: limitation: threshold is provisional (not calibrated)
- E44: partial coverage (1 of 3 examined)
- E44: limitation: F3: growth evidence is from S1, not S3; not subtracted
- E44: limitation: F6: growth evidence is from S1, not S3; not subtracted
- E44: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E44: limitation: threshold is provisional (not calibrated)
- E45: partial coverage (1 of 3 examined)
- E45: limitation: F3: growth evidence is from S1, not S4; not subtracted
- E45: limitation: F6: growth evidence is from S1, not S4; not subtracted
- E45: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E45: limitation: threshold is provisional (not calibrated)
- E46: partial coverage (1 of 3 examined)
- E46: limitation: F3: growth evidence is from S1, not S5; not subtracted
- E46: limitation: F6: growth evidence is from S1, not S5; not subtracted
- E46: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E46: limitation: threshold is provisional (not calibrated)
- E47: partial coverage (1 of 3 examined)
- E47: limitation: F3: growth evidence is from S1, not S7; not subtracted
- E47: limitation: F6: growth evidence is from S1, not S7; not subtracted
- E47: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E47: limitation: threshold is provisional (not calibrated)
- E48: partial coverage (1 of 3 examined)
- E48: limitation: F3: growth evidence is from S1, not S8; not subtracted
- E48: limitation: F6: growth evidence is from S1, not S8; not subtracted
- E48: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E48: limitation: threshold is provisional (not calibrated)
- E49: partial coverage (1 of 3 examined)
- E49: limitation: F3: growth evidence is from S1, not S9; not subtracted
- E49: limitation: F6: growth evidence is from S1, not S9; not subtracted
- E49: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E49: limitation: threshold is provisional (not calibrated)
- E50: partial coverage (1 of 3 examined)
- E50: limitation: F3: growth evidence is from S1, not S10; not subtracted
- E50: limitation: F6: growth evidence is from S1, not S10; not subtracted
- E50: limitation: off-heap: no gap series for S10; run dla gap S10 first (node.external is used when present)
- E50: limitation: threshold is provisional (not calibrated)
- E51: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E51: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E51: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E51: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E51: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E51: limitation: points 1 to 9 come from S1; the extra points from S7
- E51: limitation: gate thresholds are provisional (not calibrated)
- E52: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E52: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E52: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E52: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E52: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E52: limitation: points 1 to 9 come from S1; the extra points from S7
- E52: limitation: gate thresholds are provisional (not calibrated)
- E53: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E53: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E53: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E53: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E53: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E53: limitation: points 1 to 9 come from S1; the extra points from S9
- E53: limitation: gate thresholds are provisional (not calibrated)
- E54: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E54: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E54: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E54: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E54: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E54: limitation: points 1 to 9 come from S1; the extra points from S9
- E54: limitation: gate thresholds are provisional (not calibrated)

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
| E4 | static-hit | static | ruff 0.16.9 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | sweep/python-ruff.json |
| E5 | static-hit | static | pylint 4.1.3 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | sweep/python-pylint.json |
| E6 | static-hit | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | sweep/python-dla.json |
| E7 | census-holder | static | dla-census 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | census/python.json |
| E8 | workload | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | workload/workloads.json |
| E9 | workload | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | harness/H1/harness.json |
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (c4272a7d50d8) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (2a66621343cf) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (2a66621343cf) | trend/T1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S3.json (dd3b61aa4dc8) | alloc/A1.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S5.json (4f193f0bbc28) | alloc/A2.json |
| E15 | growth-series | test | dla 0.1.0 | artifact series/S2.json (9078a1f6d771) | trend/T4.json |
| E16 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (9078a1f6d771) | trend/T4.json |
| E17 | growth-series | test | dla 0.1.0 | artifact series/S3.json (dd3b61aa4dc8) | trend/T5.json |
| E18 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (dd3b61aa4dc8) | trend/T5.json |
| E19 | growth-series | test | dla 0.1.0 | artifact series/S4.json (27c7d253f15c) | trend/T6.json |
| E20 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (27c7d253f15c) | trend/T6.json |
| E21 | growth-series | test | dla 0.1.0 | artifact series/S5.json (4f193f0bbc28) | trend/T7.json |
| E22 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (4f193f0bbc28) | trend/T7.json |
| E23 | histogram-diff | live | py-types 0.1.0 | artifact histograms/H1.json (6e405ae90fab) | histograms/H1.json |
| E24 | growth-series | live | py-types 0.1.0 | artifact series/S6.json (fcd175394635) | series/S6.json |
| E25 | growth-series | dump | dla 0.1.0 | artifact series/S6.json (fcd175394635) | trend/T8.json |
| E26 | unit-count-series | dump | dla 0.1.0 | artifact series/S6.json (fcd175394635) | trend/T8.json |
| E27 | discriminator | test | dla 0.1.0 | artifact series/S8.json (1b138a346088) | alloc/A3.json |
| E28 | discriminator | test | dla 0.1.0 | artifact series/S10.json (3ea44debc9b3) | alloc/A4.json |
| E29 | growth-series | test | dla 0.1.0 | artifact series/S7.json (88f32eab56b2) | trend/T11.json |
| E30 | unit-count-series | test | dla 0.1.0 | artifact series/S7.json (88f32eab56b2) | trend/T11.json |
| E31 | growth-series | test | dla 0.1.0 | artifact series/S9.json (593880121d1b) | trend/T12.json |
| E32 | unit-count-series | test | dla 0.1.0 | artifact series/S9.json (593880121d1b) | trend/T12.json |
| E33 | residual | test | dla 0.1.0 | artifact series/S1.json (2a66621343cf) | gap/residual-S1-E33.json |
| E34 | residual | test | dla 0.1.0 | artifact series/S2.json (9078a1f6d771) | gap/residual-S2-E34.json |
| E35 | residual | test | dla 0.1.0 | artifact series/S3.json (dd3b61aa4dc8) | gap/residual-S3-E35.json |
| E36 | residual | test | dla 0.1.0 | artifact series/S4.json (27c7d253f15c) | gap/residual-S4-E36.json |
| E37 | residual | test | dla 0.1.0 | artifact series/S5.json (4f193f0bbc28) | gap/residual-S5-E37.json |
| E38 | residual | test | dla 0.1.0 | artifact series/S7.json (88f32eab56b2) | gap/residual-S7-E38.json |
| E39 | residual | test | dla 0.1.0 | artifact series/S8.json (1b138a346088) | gap/residual-S8-E39.json |
| E40 | residual | test | dla 0.1.0 | artifact series/S9.json (593880121d1b) | gap/residual-S9-E40.json |
| E41 | residual | test | dla 0.1.0 | artifact series/S10.json (3ea44debc9b3) | gap/residual-S10-E41.json |
| E42 | residual | test | dla 0.1.0 | artifact series/S1.json (2a66621343cf) | gap/residual-S1-E42.json |
| E43 | residual | test | dla 0.1.0 | artifact series/S2.json (9078a1f6d771) | gap/residual-S2-E43.json |
| E44 | residual | test | dla 0.1.0 | artifact series/S3.json (dd3b61aa4dc8) | gap/residual-S3-E44.json |
| E45 | residual | test | dla 0.1.0 | artifact series/S4.json (27c7d253f15c) | gap/residual-S4-E45.json |
| E46 | residual | test | dla 0.1.0 | artifact series/S5.json (4f193f0bbc28) | gap/residual-S5-E46.json |
| E47 | residual | test | dla 0.1.0 | artifact series/S7.json (88f32eab56b2) | gap/residual-S7-E47.json |
| E48 | residual | test | dla 0.1.0 | artifact series/S8.json (1b138a346088) | gap/residual-S8-E48.json |
| E49 | residual | test | dla 0.1.0 | artifact series/S9.json (593880121d1b) | gap/residual-S9-E49.json |
| E50 | residual | test | dla 0.1.0 | artifact series/S10.json (3ea44debc9b3) | gap/residual-S10-E50.json |
| E51 | growth-series | test | dla 0.1.0 | artifact series/S8.json (1b138a346088) | trend/T13.json |
| E52 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (1b138a346088) | trend/T13.json |
| E53 | growth-series | test | dla 0.1.0 | artifact series/S10.json (3ea44debc9b3) | trend/T14.json |
| E54 | unit-count-series | test | dla 0.1.0 | artifact series/S10.json (3ea44debc9b3) | trend/T14.json |
