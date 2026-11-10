# DLA report

Run: `20261110-174307-c629`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 10 enumerated, 10 dispositioned; confirmed 2, strong 0, refuted 2, suspected 0, unresolved 6

Residual: heap 2681.669696969697 bytes/n unattributed of 2681.669696969697 total (threshold 670.4174242424242) ABOVE THRESHOLD, see O1; heap 2680.0316477272727 bytes/n unattributed of 2680.0316477272727 total (threshold 670.0079119318182) ABOVE THRESHOLD, see O1; heap 2679.2007485569984 bytes/n unattributed of 2679.2007485569984 total (threshold 669.8001871392496) ABOVE THRESHOLD, see O1; heap 2680.0383522727275 bytes/n unattributed of 2680.0383522727275 total (threshold 670.0095880681819) ABOVE THRESHOLD, see O1; heap 2679.142768595041 bytes/n unattributed of 2679.142768595041 total (threshold 669.7856921487603) ABOVE THRESHOLD, see O1

Coverage: python 3520 iterations, 8 of 11 entry points exercised

## Findings

### F3: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Module global ingest.parser._newest holds the latest Record; each Record.previous points to the prior one, so the whole history of Records is a singly linked chain retained from the global (never trimmed). Not a collection or listener.)
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
- Sibling hypotheses: F3.n1
- Supporting evidence: E7, E10, E12
- Explanation (observed): Holder ingest.parser._newest (parser.py:3) is reassigned at parser.py:29 to each new Record, and Record.previous (set at parser.py:28) links it to the prior one; the chain is never cut, so every parsed Record stays reachable from the global.
- Explanation structure: adds: ingest/parser.py:29; removal: never; evidence: E12

### F6: **L-REGISTER**

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
- Sibling hypotheses: F6.n1
- Supporting evidence: E7, E10, E12
- Explanation (observed): Pipeline.process_batch (pipeline.py:26) subscribes a new CheckpointWatcher per batch into registry._LISTENERS (registry.py:3, appended at registry.py:7); the only remover, reset() at registry.py:24, is never reached in production (tests only), so every watcher and its marks dict stays.
- Explanation structure: adds: ingest/registry.py:7; removal: unreachable at ingest/registry.py:24; evidence: E12

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `ingest/parser.py:29`
- Reason: not_exercised
- Remedy: Duplicate detector hit of F3 (parser.py:29 chain head); covered by confirmed F3
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `ingest/registry.py:3`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: Duplicate detector hit of F6 (same holder registry._LISTENERS, registry.py:3); covered by confirmed F6
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
- Remedy: Census candidate not shown to grow in the harness; no growth tied
- Supporting evidence: E7

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `ingest.pipeline.Pipeline.process_batch`
- Region: heap
- Site: `ingest/pipeline.py:28`
- Reason: not_exercised
- Remedy: Census candidate not shown to grow in the harness; no growth tied
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
- Remedy: Duplicate growth-series entry of F3 (Record chain from parser._newest); covered by confirmed F3
- Supporting evidence: E17, E18, E20, E22, E24, E26

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `ingest.registry._LISTENERS`
- Region: heap
- Shape: steady
- Site: `ingest/pipeline.py:26`
- Reason: not_exercised
- Remedy: Duplicate growth-series entry of F6 (CheckpointWatcher objects in _LISTENERS); covered by confirmed F6
- Supporting evidence: E17, E18, E20, E22, E24, E26

## Ruled out

- **F3.n1** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (leak) E15
  - Hypothesis: N-WARM: growth is a warm-up fill that plateaus
  - Refuted if: growth continues over extra cycles / does not plateau or purge
  - Contradicting evidence: E15
  - Originally supported by: E12
- **F6.n1** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (leak) E16
  - Hypothesis: N-WARM: growth is a warm-up fill that plateaus
  - Refuted if: growth continues over extra cycles / does not plateau or purge
  - Contradicting evidence: E16
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
- E15: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E15: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E15: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E15: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E15: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E15: limitation: points 1 to 9 come from S1; the extra points from S3
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E16: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E16: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E16: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E16: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E16: limitation: points 1 to 9 come from S1; the extra points from S5
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E17: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E18: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E23: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E23: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E23: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E23: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E23: limitation: points 1 to 9 come from S1; the extra points from S3
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E24: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E24: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E24: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E24: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E24: limitation: points 1 to 9 come from S1; the extra points from S3
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E25: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E25: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E25: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E25: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E25: limitation: points 1 to 9 come from S1; the extra points from S5
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E26: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E26: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E26: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E26: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E26: limitation: points 1 to 9 come from S1; the extra points from S5
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: partial coverage (1 of 3 examined)
- E27: limitation: F3: growth measured as a count; bytes not attributed
- E27: limitation: F6: growth measured as a count; bytes not attributed
- E27: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E27: limitation: threshold is provisional (not calibrated)
- E28: partial coverage (1 of 3 examined)
- E28: limitation: F3: growth evidence is from S1, not S3; not subtracted
- E28: limitation: F6: growth evidence is from S1, not S3; not subtracted
- E28: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E28: limitation: threshold is provisional (not calibrated)
- E29: partial coverage (1 of 3 examined)
- E29: limitation: F3: growth evidence is from S1, not S4; not subtracted
- E29: limitation: F6: growth evidence is from S1, not S4; not subtracted
- E29: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E29: limitation: threshold is provisional (not calibrated)
- E30: partial coverage (1 of 3 examined)
- E30: limitation: F3: growth evidence is from S1, not S5; not subtracted
- E30: limitation: F6: growth evidence is from S1, not S5; not subtracted
- E30: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E30: limitation: threshold is provisional (not calibrated)
- E31: partial coverage (1 of 3 examined)
- E31: limitation: F3: growth evidence is from S1, not S6; not subtracted
- E31: limitation: F6: growth evidence is from S1, not S6; not subtracted
- E31: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E31: limitation: threshold is provisional (not calibrated)

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
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (c9a25326649d) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (a818f03c2dbf) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (a818f03c2dbf) | trend/T1.json |
| E13 | histogram-diff | live | py-types 0.1.0 | artifact histograms/H1.json (15be4ed96837) | histograms/H1.json |
| E14 | growth-series | live | py-types 0.1.0 | artifact series/S2.json (b3066361e4d1) | series/S2.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S4.json (3df9158789a0) | alloc/A1.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S6.json (7a43c4592105) | alloc/A2.json |
| E17 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (b3066361e4d1) | trend/T4.json |
| E18 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (b3066361e4d1) | trend/T4.json |
| E19 | growth-series | test | dla 0.1.0 | artifact series/S3.json (847b46c4cbf1) | trend/T5.json |
| E20 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (847b46c4cbf1) | trend/T5.json |
| E21 | growth-series | test | dla 0.1.0 | artifact series/S5.json (7a25d4d213a5) | trend/T6.json |
| E22 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (7a25d4d213a5) | trend/T6.json |
| E23 | growth-series | test | dla 0.1.0 | artifact series/S4.json (3df9158789a0) | trend/T7.json |
| E24 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (3df9158789a0) | trend/T7.json |
| E25 | growth-series | test | dla 0.1.0 | artifact series/S6.json (7a43c4592105) | trend/T8.json |
| E26 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (7a43c4592105) | trend/T8.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S1.json (a818f03c2dbf) | gap/residual-S1-E27.json |
| E28 | residual | test | dla 0.1.0 | artifact series/S3.json (847b46c4cbf1) | gap/residual-S3-E28.json |
| E29 | residual | test | dla 0.1.0 | artifact series/S4.json (3df9158789a0) | gap/residual-S4-E29.json |
| E30 | residual | test | dla 0.1.0 | artifact series/S5.json (7a25d4d213a5) | gap/residual-S5-E30.json |
| E31 | residual | test | dla 0.1.0 | artifact series/S6.json (7a43c4592105) | gap/residual-S6-E31.json |
