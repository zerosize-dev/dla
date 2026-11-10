# DLA report

Run: `20261110-161112-25ef`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 2 enumerated, 2 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 0

Residual: heap 2.009659090909091 bytes/n unattributed of 2.009659090909091 total (threshold 2.7939548863636365); heap 1.3549242424242425 bytes/n unattributed of 1.3549242424242425 total (threshold 0.3387310606060606) ABOVE THRESHOLD, see O1; heap 1.185284090909091 bytes/n unattributed of 1.185284090909091 total (threshold 0.29632102272727273) ABOVE THRESHOLD, see O1; heap 1.189556277056277 bytes/n unattributed of 1.189556277056277 total (threshold 0.2973890692640693) ABOVE THRESHOLD, see O1; heap 1.198737373737374 bytes/n unattributed of 1.198737373737374 total (threshold 0.29968434343434347) ABOVE THRESHOLD, see O1

Coverage: python 3520 iterations, 13 of 13 entry points exercised

## Findings

### F1.n1: Hypothesis N-HEADROOM: cyclic Node/Tree garbage waits for the collector because gc threshold is raised to 40000 (outlines/tuning.py:10); heap is flat after collection (UNCLASSIFIED)

- State: confirmed
- Explained by N-HEADROOM (discriminator E26)
- Evidence classes: context, discriminator
- Sibling hypothesis of: F1
- Hypothesis: N-HEADROOM: cyclic Node/Tree garbage waits for the collector because gc threshold is raised to 40000 (outlines/tuning.py:10); heap is flat after collection
- Refuted if: growth continues over the extra cycles
- Supporting evidence: E26, E9

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E26
  - Ruled out by: plateau+purge (non-leak) E26
  - Contradicting evidence: E26
  - Originally supported by: E7

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
- E14: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: partial coverage (1 of 3 examined)
- E16: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E16: limitation: threshold is provisional (not calibrated)
- E17: partial coverage (1 of 3 examined)
- E17: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E17: limitation: threshold is provisional (not calibrated)
- E18: partial coverage
- E18: limitation: the agent lists the 300 types with the most instances, so a type outside the list at a capture has no figure there
- E18: limitation: bytes are shallow sys.getsizeof sizes: what an object points to is counted under its own type
- E18: limitation: objects are those the garbage collector tracks, counted after a full collection; the files are ordered as given
- E18: limitation: each histogram lists only the largest types; a type outside the list has no figure
- E19: limitation: the agent lists the 300 types with the most instances, so a type outside the list at a capture has no figure there
- E19: limitation: bytes are shallow sys.getsizeof sizes: what an object points to is counted under its own type
- E19: limitation: objects are those the garbage collector tracks, counted after a full collection; the files are ordered as given
- E19: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E20: limitation: tracemalloc sees allocations made after it started (interpreter start-up excluded) and only through Python's allocators; memory a C extension takes with malloc is not in it
- E20: limitation: each file lists the largest stacks by live bytes and, after the first, the stacks that grew most since the first file: a stack outside the list has no figure in that file
- E20: limitation: stacks are Python frames only, as deep as DLA_TRACEMALLOC_FRAMES (default 10); function names are the module of the frame
- E20: limitation: the files are ordered as given
- E21: limitation: tracemalloc sees allocations made after it started (interpreter start-up excluded) and only through Python's allocators; memory a C extension takes with malloc is not in it
- E21: limitation: each file lists the largest stacks by live bytes and, after the first, the stacks that grew most since the first file: a stack outside the list has no figure in that file
- E21: limitation: stacks are Python frames only, as deep as DLA_TRACEMALLOC_FRAMES (default 10); function names are the module of the frame
- E21: limitation: the files are ordered as given
- E21: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E22: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E22: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E23: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E24: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E25: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E26: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E26: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E26: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E26: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E26: limitation: points 1 to 9 come from S1; the extra points from S6
- E26: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E26: limitation: gate and plateau thresholds are provisional (not calibrated)
- E27: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E27: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E27: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E27: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E27: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E27: limitation: points 1 to 9 come from S1; the extra points from S6
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E28: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E28: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E28: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E28: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E28: limitation: points 1 to 9 come from S1; the extra points from S6
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: partial coverage (1 of 3 examined)
- E29: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E29: limitation: threshold is provisional (not calibrated)
- E30: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E30: limitation: gate thresholds are provisional (not calibrated)
- E31: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E31: limitation: gate thresholds are provisional (not calibrated)
- E32: partial coverage (1 of 3 examined)
- E32: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E32: limitation: threshold is provisional (not calibrated)
- E33: partial coverage (1 of 3 examined)
- E33: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E33: limitation: threshold is provisional (not calibrated)
- E34: partial coverage (1 of 3 examined)
- E34: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E34: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | — |
| E2 | run-config | static | dla 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | — |
| E3 | probe | static | dla-probe 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | probe/probe.json |
| E4 | static-hit | static | ruff 0.16.9 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | sweep/python-ruff.json |
| E5 | static-hit | static | pylint 4.1.3 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | sweep/python-pylint.json |
| E6 | static-hit | static | dla 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | sweep/python-dla.json |
| E7 | census-holder | static | dla-census 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | census/python.json |
| E8 | workload | static | dla 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | workload/workloads.json |
| E9 | workload | static | dla 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | harness/H1/harness.json |
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (62738303ae40) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (63f248577d7a) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (63f248577d7a) | trend/T1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S3.json (07629fcdbcf7) | alloc/A1.json |
| E14 | growth-series | test | dla 0.1.0 | artifact series/S2.json (af4548d1379e) | trend/T3.json |
| E15 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (af4548d1379e) | trend/T3.json |
| E16 | residual | test | dla 0.1.0 | artifact series/S1.json (63f248577d7a) | gap/residual-S1-E16.json |
| E17 | residual | test | dla 0.1.0 | artifact series/S2.json (af4548d1379e) | gap/residual-S2-E17.json |
| E18 | histogram-diff | live | py-types 0.1.0 | artifact histograms/H1.json (b779685df906) | histograms/H1.json |
| E19 | growth-series | live | py-types 0.1.0 | artifact series/S4.json (df37c0e0db1a) | series/S4.json |
| E20 | alloc-stack | live | py-tracemalloc 0.1.0 | artifact allocs/A1.json (b813c264091d) | allocs/A1.json |
| E21 | growth-series | live | py-tracemalloc 0.1.0 | artifact series/S5.json (f119466bfb4d) | series/S5.json |
| E22 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (df37c0e0db1a) | trend/T4.json |
| E23 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (df37c0e0db1a) | trend/T4.json |
| E24 | growth-series | dump | dla 0.1.0 | artifact series/S5.json (f119466bfb4d) | trend/T5.json |
| E25 | unit-count-series | dump | dla 0.1.0 | artifact series/S5.json (f119466bfb4d) | trend/T5.json |
| E26 | discriminator | test | dla 0.1.0 | artifact series/S7.json (27d2b1862b75) | alloc/A2.json |
| E27 | growth-series | test | dla 0.1.0 | artifact series/S7.json (27d2b1862b75) | trend/T7.json |
| E28 | unit-count-series | test | dla 0.1.0 | artifact series/S7.json (27d2b1862b75) | trend/T7.json |
| E29 | residual | test | dla 0.1.0 | artifact series/S2.json (af4548d1379e) | gap/residual-S2-E29.json |
| E30 | growth-series | test | dla 0.1.0 | artifact series/S6.json (174edc104cb0) | trend/T8.json |
| E31 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (174edc104cb0) | trend/T8.json |
| E32 | residual | test | dla 0.1.0 | artifact series/S3.json (07629fcdbcf7) | gap/residual-S3-E32.json |
| E33 | residual | test | dla 0.1.0 | artifact series/S6.json (174edc104cb0) | gap/residual-S6-E33.json |
| E34 | residual | test | dla 0.1.0 | artifact series/S7.json (27d2b1862b75) | gap/residual-S7-E34.json |
