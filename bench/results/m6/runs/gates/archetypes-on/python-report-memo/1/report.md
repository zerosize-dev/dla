# DLA report

Run: `20261110-162933-a878`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 10 enumerated, 10 dispositioned; confirmed 0, strong 1, refuted 1, suspected 3, unresolved 5

Residual: heap 2371.664772727273 bytes/n unattributed of 2371.664772727273 total (threshold 592.9161931818182) ABOVE THRESHOLD, see O1; heap 2325.288636363636 bytes/n unattributed of 2325.288636363636 total (threshold 581.322159090909) ABOVE THRESHOLD, see O1; heap 2343.828456439394 bytes/n unattributed of 2343.828456439394 total (threshold 585.9571141098485) ABOVE THRESHOLD, see O1; heap 0.44130681818181816 bytes/n unattributed of 0.44130681818181816 total (threshold 0.11032670454545454) ABOVE THRESHOLD, see O2

Coverage: python 3520 iterations, 10 of 10 entry points exercised

## Findings

### F4: **L-COLLECT**

- State: strong
- Label: L-COLLECT (evidence: E12 unit-count-series, E7 census-holder)
- Growth: `probe:reports.builder.ReportBuilder.category_totals` 36.73 count/n (steady, gate pass, E12)
- Evidence classes: context, growth, intervention, static
- Strong: second class E20 intervention that removed the growth, independent of the growth record E12 (dla)
- Holder: static/global
- Owner: `reports.builder.ReportBuilder.category_totals`
- Region: heap
- Shape: steady
- Site: `reports/builder.py:14`
- Writers: `reports/builder.py:14`
- Annotation: lru_cache(maxsize=None) on instance method; key includes self; builder created per request
- Verified: no
- Sibling hypotheses: F4.n1
- Supporting evidence: E7, E10, E12, E20
- Explanation (inferred): Static/global lru_cache(maxsize=None) on ReportBuilder.category_totals (builder.py:14) keys on self; service.handle makes a new builder per request, so every builder and its column buffers is retained forever, never evicted.
- Explanation structure: adds: reports/builder.py:14; removal: never

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: growth, static
- Site: `reports/builder.py:14`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: Same site (builder.py:14) as confirmed F4; detector hit duplicate, covered by F4 evidence
- Supporting evidence: E4, E12

### F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: growth, static
- Site: `reports/builder.py:14`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: Same site (builder.py:14) as confirmed F4; detector hit duplicate, covered by F4 evidence
- Supporting evidence: E5, E12

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: growth, static
- Site: `reports/builder.py:14`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: Same site (builder.py:14) as confirmed F4; detector hit duplicate, covered by F4 evidence
- Supporting evidence: E6, E12

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Type-count growth (dict/list/ReportBuilder) is the retained builders of F4; confirm by a per-type probe if needed
- Supporting evidence: E17

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Type-count growth (dict/list/ReportBuilder) is the retained builders of F4; confirm by a per-type probe if needed
- Supporting evidence: E15

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Type-count growth (dict/list/ReportBuilder) is the retained builders of F4; confirm by a per-type probe if needed
- Supporting evidence: E17

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Allocation-stack growth in reports/builder.py and formatting.py is the retained builders' column buffers of F4; per-stack tie not established
- Supporting evidence: E21

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Allocation-stack growth in reports/builder.py and formatting.py is the retained builders' column buffers of F4; per-stack tie not established
- Supporting evidence: E22

## Ruled out

- **F4.n1** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (leak) E13
  - Hypothesis: N-WARM: bounded warm-up cache that plateaus once all (account,month) keys are seen
  - Refuted if: growth continues over the extra cycles / does not plateau after purge
  - Contradicting evidence: E13
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
- E14: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E16: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E16: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E16: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E16: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E16: limitation: points 1 to 9 come from S1; the extra points from S2
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E17: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E17: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E17: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E17: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E17: limitation: points 1 to 9 come from S1; the extra points from S2
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: tracemalloc sees allocations made after it started (interpreter start-up excluded) and only through Python's allocators; memory a C extension takes with malloc is not in it
- E18: limitation: each file lists the largest stacks by live bytes and, after the first, the stacks that grew most since the first file: a stack outside the list has no figure in that file
- E18: limitation: stacks are Python frames only, as deep as DLA_TRACEMALLOC_FRAMES (default 10); function names are the module of the frame
- E18: limitation: the files are ordered as given
- E19: limitation: tracemalloc sees allocations made after it started (interpreter start-up excluded) and only through Python's allocators; memory a C extension takes with malloc is not in it
- E19: limitation: each file lists the largest stacks by live bytes and, after the first, the stacks that grew most since the first file: a stack outside the list has no figure in that file
- E19: limitation: stacks are Python frames only, as deep as DLA_TRACEMALLOC_FRAMES (default 10); function names are the module of the frame
- E19: limitation: the files are ordered as given
- E19: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E20: limitation: tested in a scratch copy; the project was not changed
- E20: limitation: gate thresholds are provisional (not calibrated); the 0.2 and 0.5 slope fractions are placeholders
- E20: limitation: exercised is null: the harness records entry points, not lines, so whether the edited line ran is not checked
- E20: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E20: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E21: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E21: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E22: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: partial coverage (1 of 3 examined)
- E23: limitation: F4: growth measured as a count; bytes not attributed
- E23: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E23: limitation: threshold is provisional (not calibrated)
- E24: partial coverage (1 of 3 examined)
- E24: limitation: F4: growth evidence is from S1, not S2; not subtracted
- E24: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E24: limitation: threshold is provisional (not calibrated)
- E25: partial coverage (1 of 3 examined)
- E25: limitation: F4: growth evidence is from S1, not S3; not subtracted
- E25: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E25: limitation: threshold is provisional (not calibrated)
- E26: partial coverage (1 of 3 examined)
- E26: limitation: F4: growth evidence is from S1, not S5; not subtracted
- E26: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E26: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | — |
| E2 | run-config | static | dla 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | probe/probe.json |
| E4 | static-hit | static | ruff 0.16.9 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | sweep/python-ruff.json |
| E5 | static-hit | static | pylint 4.1.3 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | sweep/python-pylint.json |
| E6 | static-hit | static | dla 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | sweep/python-dla.json |
| E7 | census-holder | static | dla-census 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | census/python.json |
| E8 | workload | static | dla 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | workload/workloads.json |
| E9 | workload | static | dla 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | harness/H1/harness.json |
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (7b440b8a2485) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (15c5cad0787b) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (15c5cad0787b) | trend/T1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S3.json (fc237d0e37df) | alloc/A1.json |
| E14 | growth-series | test | dla 0.1.0 | artifact series/S2.json (0ad8746de7e4) | trend/T3.json |
| E15 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (0ad8746de7e4) | trend/T3.json |
| E16 | growth-series | test | dla 0.1.0 | artifact series/S3.json (fc237d0e37df) | trend/T4.json |
| E17 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (fc237d0e37df) | trend/T4.json |
| E18 | alloc-stack | live | py-tracemalloc 0.1.0 | artifact allocs/A1.json (5cd22ec28d41) | allocs/A1.json |
| E19 | growth-series | live | py-tracemalloc 0.1.0 | artifact series/S4.json (69378c788123) | series/S4.json |
| E20 | intervention | test | dla 0.1.0 | artifact series/S5.json (2ed5a7d08176) | intervene/I1 |
| E21 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (69378c788123) | trend/T6.json |
| E22 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (69378c788123) | trend/T6.json |
| E23 | residual | test | dla 0.1.0 | artifact series/S1.json (15c5cad0787b) | gap/residual-S1-E23.json |
| E24 | residual | test | dla 0.1.0 | artifact series/S2.json (0ad8746de7e4) | gap/residual-S2-E24.json |
| E25 | residual | test | dla 0.1.0 | artifact series/S3.json (fc237d0e37df) | gap/residual-S3-E25.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S5.json (2ed5a7d08176) | gap/residual-S5-E26.json |
