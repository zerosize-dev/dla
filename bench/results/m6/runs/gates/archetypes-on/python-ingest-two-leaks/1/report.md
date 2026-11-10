# DLA report

Run: `20261110-162530-1fda`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 18 enumerated, 18 dispositioned; confirmed 2, strong 0, refuted 2, suspected 0, unresolved 14

Residual: heap 2678.960284090909 bytes/n unattributed of 2678.960284090909 total (threshold 669.7400710227273) ABOVE THRESHOLD, see O2; heap 2680.0467045454543 bytes/n unattributed of 2680.0467045454543 total (threshold 670.0116761363636) ABOVE THRESHOLD, see O1; heap 2679.6097377622377 bytes/n unattributed of 2679.6097377622377 total (threshold 669.9024344405594) ABOVE THRESHOLD, see O1; heap 2679.8029545454547 bytes/n unattributed of 2679.8029545454547 total (threshold 669.9507386363637) ABOVE THRESHOLD, see O1; heap 2679.590839160839 bytes/n unattributed of 2679.590839160839 total (threshold 669.8977097902098) ABOVE THRESHOLD, see O1

Coverage: python 3520 iterations, 8 of 11 entry points exercised

## Findings

### F1: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Module global parser._newest holds the latest Record; each Record.previous points to the prior one (parser.py:27), forming an unbounded singly linked chain of every Record ever parsed, so no Record is freed. Not a collection: growth is by back-reference chain from a single global head.)
- Growth: `probe:ingest.parser._newest` 15.97 count/n (steady, gate pass, E12)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `ingest.parser._newest`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:3`
- Writers: `ingest/parser.py:29`
- Annotation: heuristic name match over the project's sources
- Verified: no
- Sibling hypotheses: F1.n1
- Supporting evidence: E4, E10, E12
- Explanation (inferred): ingest/parser.py:3 _newest module global is reassigned per record (parser.py:29) while record.previous (parser.py:27) links to the prior Record; nothing ever clears previous, so every Record stays reachable and is never removed.
- Explanation structure: adds: ingest/parser.py:29; removal: never

### F4: **L-REGISTER**

- State: confirmed
- Label: L-REGISTER (evidence: E12 unit-count-series)
- Growth: `probe:ingest.registry._LISTENERS` 7.974 count/n (steady, gate pass, E12)
- Evidence classes: context, growth, static
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
- Supporting evidence: E4, E10, E12
- Explanation (observed): ingest/registry.py:3 _LISTENERS holds a CheckpointWatcher added per batch by subscribe (registry.py:7, called pipeline.py:28); it is removed only when reset() (registry.py:24, tests only) runs, so none is freed in production.
- Explanation structure: adds: ingest/registry.py:7; removal: conditional at ingest/registry.py:24; evidence: E12

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `ingest.pipeline.Pipeline.per_sensor`
- Region: heap
- Site: `ingest/pipeline.py:15`
- Writers: `ingest/pipeline.py:25`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Pipeline.per_sensor is keyed by 64 sensors so bounded by design; no growth column; not separately exercised
- Supporting evidence: E4

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `ingest.pipeline.Pipeline.process_batch`
- Region: heap
- Site: `ingest/pipeline.py:28`
- Reason: not_exercised
- Remedy: Adding site of confirmed F4 (pipeline.py:28); covered by F4
- Supporting evidence: E4

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `ingest/parser.py:29`
- Reason: not_exercised
- Remedy: Duplicate detector hit of confirmed F1 (same parser.py chain); covered by F1
- Supporting evidence: E7

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `ingest/registry.py:3`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: Duplicate detector hit of confirmed F4 (_LISTENERS); covered by F4
- Supporting evidence: E7

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `ingest.models.Record`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:29`
- Reason: not_exercised
- Remedy: Duplicate growth-series row for F1 Record count; covered by F1
- Supporting evidence: E20

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `ingest.watchers.CheckpointWatcher`
- Region: heap
- Shape: steady
- Site: `ingest/pipeline.py:28`
- Reason: not_exercised
- Remedy: Duplicate growth-series row for F4 CheckpointWatcher count; covered by F4
- Supporting evidence: E20

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `ingest.models.Record`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:29`
- Reason: not_exercised
- Remedy: Duplicate growth-series row for F1 Record count; covered by F1
- Supporting evidence: E16

### F10: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `ingest.watchers.CheckpointWatcher`
- Region: heap
- Shape: steady
- Site: `ingest/pipeline.py:28`
- Reason: not_exercised
- Remedy: Duplicate growth-series row for F4 CheckpointWatcher count; covered by F4
- Supporting evidence: E16

### F11: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `ingest.models.Record`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:29`
- Reason: not_exercised
- Remedy: Duplicate growth-series row for F1 Record count; covered by F1
- Supporting evidence: E18

### F12: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `ingest.watchers.CheckpointWatcher`
- Region: heap
- Shape: steady
- Site: `ingest/pipeline.py:28`
- Reason: not_exercised
- Remedy: Duplicate growth-series row for F4 CheckpointWatcher count; covered by F4
- Supporting evidence: E18

### F13: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `ingest.models.Record`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:29`
- Reason: not_exercised
- Remedy: Duplicate growth-series row for F1 Record count; covered by F1
- Supporting evidence: E22

### F14: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `ingest.watchers.CheckpointWatcher`
- Region: heap
- Shape: steady
- Site: `ingest/pipeline.py:28`
- Reason: not_exercised
- Remedy: Duplicate growth-series row for F4 CheckpointWatcher count; covered by F4
- Supporting evidence: E22

### F15: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `ingest.models.Record`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:29`
- Reason: not_exercised
- Remedy: Duplicate growth-series row for F1 Record count; covered by F1
- Supporting evidence: E24

### F16: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `ingest.watchers.CheckpointWatcher`
- Region: heap
- Shape: steady
- Site: `ingest/pipeline.py:28`
- Reason: not_exercised
- Remedy: Duplicate growth-series row for F4 CheckpointWatcher count; covered by F4
- Supporting evidence: E24

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (leak) E13
  - Hypothesis: N-WARM
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E13
  - Originally supported by: E12
- **F4.n1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (leak) E14
  - Hypothesis: N-WARM
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
- E17: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E19: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E19: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E19: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E19: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E19: limitation: points 1 to 9 come from S1; the extra points from S2
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E20: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E20: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E20: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E20: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E20: limitation: points 1 to 9 come from S1; the extra points from S2
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
- E23: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E23: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E23: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E23: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E23: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E23: limitation: points 1 to 9 come from S1; the extra points from S2
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E24: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E24: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E24: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E24: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E24: limitation: points 1 to 9 come from S1; the extra points from S2
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: partial coverage
- E25: limitation: the agent lists the 300 types with the most instances, so a type outside the list at a capture has no figure there
- E25: limitation: bytes are shallow sys.getsizeof sizes: what an object points to is counted under its own type
- E25: limitation: objects are those the garbage collector tracks, counted after a full collection; the files are ordered as given
- E25: limitation: each histogram lists only the largest types; a type outside the list has no figure
- E25: limitation: two captures cannot tell a warm-up fill from a leak: no monotone type is reported and no growth column is written
- E26: limitation: tracemalloc sees allocations made after it started (interpreter start-up excluded) and only through Python's allocators; memory a C extension takes with malloc is not in it
- E26: limitation: each file lists the largest stacks by live bytes and, after the first, the stacks that grew most since the first file: a stack outside the list has no figure in that file
- E26: limitation: stacks are Python frames only, as deep as DLA_TRACEMALLOC_FRAMES (default 10); function names are the module of the frame
- E26: limitation: the files are ordered as given
- E27: partial coverage (1 of 3 examined)
- E27: limitation: F1: growth measured as a count; bytes not attributed
- E27: limitation: F4: growth measured as a count; bytes not attributed
- E27: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E27: limitation: threshold is provisional (not calibrated)
- E28: partial coverage (1 of 3 examined)
- E28: limitation: F1: growth evidence is from S1, not S2; not subtracted
- E28: limitation: F4: growth evidence is from S1, not S2; not subtracted
- E28: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E28: limitation: threshold is provisional (not calibrated)
- E29: partial coverage (1 of 3 examined)
- E29: limitation: F1: growth evidence is from S1, not S3; not subtracted
- E29: limitation: F4: growth evidence is from S1, not S3; not subtracted
- E29: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E29: limitation: threshold is provisional (not calibrated)
- E30: partial coverage (1 of 3 examined)
- E30: limitation: F1: growth evidence is from S1, not S4; not subtracted
- E30: limitation: F4: growth evidence is from S1, not S4; not subtracted
- E30: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E30: limitation: threshold is provisional (not calibrated)
- E31: partial coverage (1 of 3 examined)
- E31: limitation: F1: growth evidence is from S1, not S5; not subtracted
- E31: limitation: F4: growth evidence is from S1, not S5; not subtracted
- E31: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E31: limitation: threshold is provisional (not calibrated)
- E32: partial coverage (1 of 3 examined)
- E32: limitation: F1: growth measured as a count; bytes not attributed
- E32: limitation: F4: growth measured as a count; bytes not attributed
- E32: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E32: limitation: threshold is provisional (not calibrated)

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
| E4 | census-holder | static | dla-census 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | census/python.json |
| E5 | static-hit | static | ruff 0.16.9 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | sweep/python-ruff.json |
| E6 | static-hit | static | pylint 4.1.3 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | sweep/python-pylint.json |
| E7 | static-hit | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | sweep/python-dla.json |
| E8 | workload | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | workload/workloads.json |
| E9 | workload | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | harness/H1/harness.json |
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (43d1a8d7a2c2) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (f237234087aa) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (f237234087aa) | trend/T1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S3.json (9927e79efa71) | alloc/A1.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S5.json (d6114795cf80) | alloc/A2.json |
| E15 | growth-series | test | dla 0.1.0 | artifact series/S2.json (aa5a819e67d7) | trend/T4.json |
| E16 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (aa5a819e67d7) | trend/T4.json |
| E17 | growth-series | test | dla 0.1.0 | artifact series/S4.json (c07c61114cb9) | trend/T5.json |
| E18 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (c07c61114cb9) | trend/T5.json |
| E19 | growth-series | test | dla 0.1.0 | artifact series/S3.json (9927e79efa71) | trend/T6.json |
| E20 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (9927e79efa71) | trend/T6.json |
| E21 | growth-series | test | dla 0.1.0 | artifact series/S5.json (d6114795cf80) | trend/T7.json |
| E22 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (d6114795cf80) | trend/T7.json |
| E23 | growth-series | test | dla 0.1.0 | artifact series/S3.json (9927e79efa71) | trend/T8.json |
| E24 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (9927e79efa71) | trend/T8.json |
| E25 | histogram-diff | live | py-types 0.1.0 | artifact histograms/H1.json (44a7a749575c) | histograms/H1.json |
| E26 | alloc-stack | live | py-tracemalloc 0.1.0 | artifact allocs/A1.json (e26a7c40cb36) | allocs/A1.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S1.json (f237234087aa) | gap/residual-S1-E27.json |
| E28 | residual | test | dla 0.1.0 | artifact series/S2.json (aa5a819e67d7) | gap/residual-S2-E28.json |
| E29 | residual | test | dla 0.1.0 | artifact series/S3.json (9927e79efa71) | gap/residual-S3-E29.json |
| E30 | residual | test | dla 0.1.0 | artifact series/S4.json (c07c61114cb9) | gap/residual-S4-E30.json |
| E31 | residual | test | dla 0.1.0 | artifact series/S5.json (d6114795cf80) | gap/residual-S5-E31.json |
| E32 | residual | test | dla 0.1.0 | artifact series/S1.json (f237234087aa) | gap/residual-S1-E32.json |
