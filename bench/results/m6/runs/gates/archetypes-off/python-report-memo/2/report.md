# DLA report

Run: `20261110-190236-8562`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 7 enumerated, 7 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 5

Residual: heap 2372.2613636363635 bytes/n unattributed of 2372.2613636363635 total (threshold 593.0653409090909) ABOVE THRESHOLD, see O1; heap 2324.653409090909 bytes/n unattributed of 2324.653409090909 total (threshold 581.1633522727273) ABOVE THRESHOLD, see O1; heap 2344.0205492424243 bytes/n unattributed of 2344.0205492424243 total (threshold 586.0051373106061) ABOVE THRESHOLD, see O1

Coverage: python 3520 iterations, 10 of 10 entry points exercised

## Findings

### F4: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (functools.lru_cache(maxsize=None) on ReportBuilder.category_totals: the cache is a process-global keyed on (self, account, month); ReportService.handle builds a new ReportBuilder per request, so every request adds an entry that pins that builder (with its column buffers and store ref) and its result tuple; never evicted)
- Growth: `probe:reports.builder.ReportBuilder.category_totals` 36.73 count/n (steady, gate pass, E12)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `reports.builder.ReportBuilder.category_totals`
- Region: heap
- Shape: steady
- Site: `reports/builder.py:14`
- Writers: `reports/builder.py:14`
- Annotation: lru_cache(maxsize=None) on method; key includes self, new builder per request
- Verified: no
- Sibling hypotheses: F4.n1
- Supporting evidence: E7, E10, E12
- Explanation (inferred): reports/builder.py:14 lru_cache(maxsize=None) on category_totals is a static/global cache keyed on self; ReportService.handle makes a new ReportBuilder per request so each call adds an entry pinning that builder, and nothing ever removes entries (removal never happens).
- Explanation structure: adds: reports/builder.py:14; removal: never

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `reports/builder.py:14`
- Reason: not_exercised
- Remedy: Not separately exercised: same site/mechanism as confirmed F4 (lru_cache on method, reports/builder.py:14); fix F4 and re-run harness.
- Supporting evidence: E4

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `reports/builder.py:14`
- Reason: not_exercised
- Remedy: Not separately exercised: same site/mechanism as confirmed F4 (lru_cache on method, reports/builder.py:14); fix F4 and re-run harness.
- Supporting evidence: E5

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `reports/builder.py:14`
- Reason: not_exercised
- Remedy: Not separately exercised: same site/mechanism as confirmed F4 (lru_cache on method, reports/builder.py:14); fix F4 and re-run harness.
- Supporting evidence: E6

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `reports.builder.ReportBuilder.category_totals`
- Region: heap
- Shape: steady
- Site: `reports/builder.py:14`
- Reason: not_exercised
- Remedy: Not separately exercised: same site/mechanism as confirmed F4 (lru_cache on method, reports/builder.py:14); fix F4 and re-run harness.
- Supporting evidence: E12, E15, E17

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `reports.builder.ReportBuilder.category_totals`
- Region: heap
- Shape: steady
- Site: `reports/builder.py:14`
- Reason: not_exercised
- Remedy: Not separately exercised: same site/mechanism as confirmed F4 (lru_cache on method, reports/builder.py:14); fix F4 and re-run harness.
- Supporting evidence: E22, E23, E24, E25

## Ruled out

- **F4.n1** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (leak) E13
  - Hypothesis: N-WARM: bounded warm-up cache fill (account x month space is finite)
  - Refuted if: growth continues over extra cycles / does not plateau after purge
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
- E20: partial coverage
- E20: limitation: the agent lists the 300 types with the most instances, so a type outside the list at a capture has no figure there
- E20: limitation: bytes are shallow sys.getsizeof sizes: what an object points to is counted under its own type
- E20: limitation: objects are those the garbage collector tracks, counted after a full collection; the files are ordered as given
- E20: limitation: each histogram lists only the largest types; a type outside the list has no figure
- E21: limitation: the agent lists the 300 types with the most instances, so a type outside the list at a capture has no figure there
- E21: limitation: bytes are shallow sys.getsizeof sizes: what an object points to is counted under its own type
- E21: limitation: objects are those the garbage collector tracks, counted after a full collection; the files are ordered as given
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
- E26: partial coverage (1 of 3 examined)
- E26: limitation: F4: growth measured as a count; bytes not attributed
- E26: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E26: limitation: threshold is provisional (not calibrated)
- E27: partial coverage (1 of 3 examined)
- E27: limitation: F4: growth evidence is from S1, not S2; not subtracted
- E27: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E27: limitation: threshold is provisional (not calibrated)
- E28: partial coverage (1 of 3 examined)
- E28: limitation: F4: growth evidence is from S1, not S3; not subtracted
- E28: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E28: limitation: threshold is provisional (not calibrated)

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
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (c7cafbe100bd) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (0dae0971fb50) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (0dae0971fb50) | trend/T1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S3.json (9ce42bf33433) | alloc/A1.json |
| E14 | growth-series | test | dla 0.1.0 | artifact series/S2.json (b2c7abdb029f) | trend/T3.json |
| E15 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (b2c7abdb029f) | trend/T3.json |
| E16 | growth-series | test | dla 0.1.0 | artifact series/S3.json (9ce42bf33433) | trend/T4.json |
| E17 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (9ce42bf33433) | trend/T4.json |
| E18 | alloc-stack | live | py-tracemalloc 0.1.0 | artifact allocs/A1.json (7a933dc98296) | allocs/A1.json |
| E19 | growth-series | live | py-tracemalloc 0.1.0 | artifact series/S4.json (a4bd9924444e) | series/S4.json |
| E20 | histogram-diff | live | py-types 0.1.0 | artifact histograms/H1.json (80bf42d9ad0f) | histograms/H1.json |
| E21 | growth-series | live | py-types 0.1.0 | artifact series/S5.json (e6514ec71b59) | series/S5.json |
| E22 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (a4bd9924444e) | trend/T5.json |
| E23 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (a4bd9924444e) | trend/T5.json |
| E24 | growth-series | dump | dla 0.1.0 | artifact series/S5.json (e6514ec71b59) | trend/T6.json |
| E25 | unit-count-series | dump | dla 0.1.0 | artifact series/S5.json (e6514ec71b59) | trend/T6.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S1.json (0dae0971fb50) | gap/residual-S1-E26.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S2.json (b2c7abdb029f) | gap/residual-S2-E27.json |
| E28 | residual | test | dla 0.1.0 | artifact series/S3.json (9ce42bf33433) | gap/residual-S3-E28.json |
