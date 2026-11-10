# DLA report

Run: `20261110-182649-a917`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 9 enumerated, 9 dispositioned; confirmed 1, strong 0, refuted 1, suspected 3, unresolved 4

Residual: heap 2371.5373863636364 bytes/n unattributed of 2371.5373863636364 total (threshold 592.8843465909091) ABOVE THRESHOLD, see O1; heap 2323.787689393939 bytes/n unattributed of 2323.787689393939 total (threshold 580.9469223484848) ABOVE THRESHOLD, see O1; heap 2343.982575757575 bytes/n unattributed of 2343.982575757575 total (threshold 585.9956439393939) ABOVE THRESHOLD, see O1

Coverage: python 3520 iterations, 10 of 10 entry points exercised

## Findings

### F4: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (functools.lru_cache(maxsize=None) on the instance method ReportBuilder.category_totals: the cache is a process-lifetime static holder keyed on self, so every per-request ReportBuilder (and its column buffers) is pinned and never evicted)
- Growth: `probe:reports.builder.ReportBuilder.category_totals` 36.73 count/n (steady, gate pass, E12)
- Evidence classes: context, discriminator, growth, static
- Holder: static/global
- Owner: `reports.builder.ReportBuilder.category_totals`
- Region: heap
- Shape: steady
- Site: `reports/builder.py:14`
- Writers: `reports/builder.py:14`, `reports/builder.py:12`, `reports/builder.py:18`, `reports/builder.py:19`, `reports/builder.py:38`, `reports/formatting.py:16`
- Annotation: lru_cache on a method keys on self; every per-request ReportBuilder and its column buffers stay referenced
- Verified: no
- Sibling hypotheses: F4.n1
- Supporting evidence: E7, E10, E12, E15
- Explanation (inferred): reports/builder.py:14 lru_cache(maxsize=None) on category_totals is a static/global holder; each request's new ReportBuilder is cache key (self) and value, so it is retained and never evicted.
- Explanation structure: adds: reports/builder.py:14; removal: never

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Site: `reports/builder.py:14`
- Reason: not_exercised
- Remedy: Same site and mechanism as confirmed F4 (detector hit on the lru_cache method); no separate verdict needed
- Supporting evidence: E4

### F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Site: `reports/builder.py:14`
- Reason: not_exercised
- Remedy: Same site and mechanism as confirmed F4 (detector hit on the lru_cache method); no separate verdict needed
- Supporting evidence: E5, E4

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Site: `reports/builder.py:14`
- Reason: not_exercised
- Remedy: Same site and mechanism as confirmed F4 (detector hit on the lru_cache method); no separate verdict needed
- Supporting evidence: E6, E4

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `reports.builder.ReportBuilder.category_totals`
- Region: heap
- Shape: steady
- Site: `reports/builder.py:14`
- Reason: not_exercised
- Remedy: Growth-series bookkeeping duplicate of F4 (same site/holder); covered by F4's confirmation
- Supporting evidence: E16, E17

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `reports.builder.ReportBuilder.category_totals`
- Region: heap
- Shape: steady
- Site: `reports/builder.py:14`
- Reason: not_exercised
- Remedy: Growth-series bookkeeping duplicate of F4 (same site/holder); covered by F4's confirmation
- Supporting evidence: E19, E21

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `reports.builder.ReportBuilder.category_totals`
- Region: heap
- Shape: steady
- Site: `reports/builder.py:14`
- Reason: not_exercised
- Remedy: Growth-series bookkeeping duplicate of F4 (same site/holder); covered by F4's confirmation
- Supporting evidence: E24, E25

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `reports.builder.ReportBuilder.category_totals`
- Region: heap
- Shape: steady
- Site: `reports/builder.py:14`
- Reason: not_exercised
- Remedy: Growth-series bookkeeping duplicate of F4 (same site/holder); covered by F4's confirmation
- Supporting evidence: E28, E29

## Ruled out

- **F4.n1** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (leak) E15
  - Hypothesis: N-WARM: bounded warm-up cache that plateaus
  - Refuted if: growth continues over the extra cycles and survives purge
  - Contradicting evidence: E15
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
- E13: limitation: tracemalloc sees allocations made after it started (interpreter start-up excluded) and only through Python's allocators; memory a C extension takes with malloc is not in it
- E13: limitation: each file lists the largest stacks by live bytes and, after the first, the stacks that grew most since the first file: a stack outside the list has no figure in that file
- E13: limitation: stacks are Python frames only, as deep as DLA_TRACEMALLOC_FRAMES (default 10); function names are the module of the frame
- E13: limitation: the files are ordered as given
- E14: limitation: tracemalloc sees allocations made after it started (interpreter start-up excluded) and only through Python's allocators; memory a C extension takes with malloc is not in it
- E14: limitation: each file lists the largest stacks by live bytes and, after the first, the stacks that grew most since the first file: a stack outside the list has no figure in that file
- E14: limitation: stacks are Python frames only, as deep as DLA_TRACEMALLOC_FRAMES (default 10); function names are the module of the frame
- E14: limitation: the files are ordered as given
- E14: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E15: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E15: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E15: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E15: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E15: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E15: limitation: points 1 to 9 come from S1; the extra points from S3
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E16: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E17: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E20: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E20: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E20: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E20: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E20: limitation: points 1 to 9 come from S1; the extra points from S3
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E21: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E21: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E21: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E21: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E21: limitation: points 1 to 9 come from S1; the extra points from S3
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: tracemalloc sees allocations made after it started (interpreter start-up excluded) and only through Python's allocators; memory a C extension takes with malloc is not in it
- E22: limitation: each file lists the largest stacks by live bytes and, after the first, the stacks that grew most since the first file: a stack outside the list has no figure in that file
- E22: limitation: stacks are Python frames only, as deep as DLA_TRACEMALLOC_FRAMES (default 10); function names are the module of the frame
- E22: limitation: the files are ordered as given
- E23: limitation: tracemalloc sees allocations made after it started (interpreter start-up excluded) and only through Python's allocators; memory a C extension takes with malloc is not in it
- E23: limitation: each file lists the largest stacks by live bytes and, after the first, the stacks that grew most since the first file: a stack outside the list has no figure in that file
- E23: limitation: stacks are Python frames only, as deep as DLA_TRACEMALLOC_FRAMES (default 10); function names are the module of the frame
- E23: limitation: the files are ordered as given
- E23: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E24: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E24: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E25: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: partial coverage
- E26: limitation: the agent lists the 300 types with the most instances, so a type outside the list at a capture has no figure there
- E26: limitation: bytes are shallow sys.getsizeof sizes: what an object points to is counted under its own type
- E26: limitation: objects are those the garbage collector tracks, counted after a full collection; the files are ordered as given
- E26: limitation: each histogram lists only the largest types; a type outside the list has no figure
- E27: limitation: the agent lists the 300 types with the most instances, so a type outside the list at a capture has no figure there
- E27: limitation: bytes are shallow sys.getsizeof sizes: what an object points to is counted under its own type
- E27: limitation: objects are those the garbage collector tracks, counted after a full collection; the files are ordered as given
- E27: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E28: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E28: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E29: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E29: limitation: gate thresholds are provisional (not calibrated)
- E30: partial coverage (1 of 3 examined)
- E30: limitation: F4: growth measured as a count; bytes not attributed
- E30: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E30: limitation: threshold is provisional (not calibrated)
- E31: partial coverage (1 of 3 examined)
- E31: limitation: F4: growth evidence is from S1, not S3; not subtracted
- E31: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E31: limitation: threshold is provisional (not calibrated)
- E32: partial coverage (1 of 3 examined)
- E32: limitation: F4: growth evidence is from S1, not S4; not subtracted
- E32: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
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
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (fbdeabb65734) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (ff5cd382eb88) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (ff5cd382eb88) | trend/T1.json |
| E13 | alloc-stack | live | py-tracemalloc 0.1.0 | artifact allocs/A1.json (cef5f447d20f) | allocs/A1.json |
| E14 | growth-series | live | py-tracemalloc 0.1.0 | artifact series/S2.json (98a055eb6b0c) | series/S2.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S4.json (1c42ab3d3184) | alloc/A1.json |
| E16 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (98a055eb6b0c) | trend/T3.json |
| E17 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (98a055eb6b0c) | trend/T3.json |
| E18 | growth-series | test | dla 0.1.0 | artifact series/S3.json (ade913c1046c) | trend/T4.json |
| E19 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (ade913c1046c) | trend/T4.json |
| E20 | growth-series | test | dla 0.1.0 | artifact series/S4.json (1c42ab3d3184) | trend/T5.json |
| E21 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (1c42ab3d3184) | trend/T5.json |
| E22 | alloc-stack | live | py-tracemalloc 0.1.0 | artifact allocs/A2.json (38c361d8b240) | allocs/A2.json |
| E23 | growth-series | live | py-tracemalloc 0.1.0 | artifact series/S5.json (a1b320979517) | series/S5.json |
| E24 | growth-series | dump | dla 0.1.0 | artifact series/S5.json (a1b320979517) | trend/T6.json |
| E25 | unit-count-series | dump | dla 0.1.0 | artifact series/S5.json (a1b320979517) | trend/T6.json |
| E26 | histogram-diff | live | py-types 0.1.0 | artifact histograms/H1.json (e17609ed5565) | histograms/H1.json |
| E27 | growth-series | live | py-types 0.1.0 | artifact series/S6.json (c7161ce63843) | series/S6.json |
| E28 | growth-series | dump | dla 0.1.0 | artifact series/S6.json (c7161ce63843) | trend/T7.json |
| E29 | unit-count-series | dump | dla 0.1.0 | artifact series/S6.json (c7161ce63843) | trend/T7.json |
| E30 | residual | test | dla 0.1.0 | artifact series/S1.json (ff5cd382eb88) | gap/residual-S1-E30.json |
| E31 | residual | test | dla 0.1.0 | artifact series/S3.json (ade913c1046c) | gap/residual-S3-E31.json |
| E32 | residual | test | dla 0.1.0 | artifact series/S4.json (1c42ab3d3184) | gap/residual-S4-E32.json |
