# DLA report

Run: `20261109-200126-002e`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 8 enumerated, 8 dispositioned; confirmed 4, strong 0, refuted 4, suspected 0, unresolved 0

Residual: not measured

Coverage: python 3520 iterations, 10 of 10 entry points exercised

## Findings

### F1: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:reports.builder.ReportBuilder.category_totals.bytes` 2290 bytes/n (steady, gate pass, E11)
- Evidence classes: context, discriminator, growth, static
- Holder: static/global
- Region: heap
- Shape: steady
- Site: `reports/builder.py:14`
- Annotation: Same site and mechanism as F4: lru_cache on instance method keyed by per-request ReportBuilder (detector duplicate).
- Sibling hypotheses: F1.n1
- Suggested archetype (unconfirmed): L-COLLECT
- Supporting evidence: E4, E10, E11, E16

### F2: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:reports.builder.ReportBuilder.category_totals.bytes` 2290 bytes/n (steady, gate pass, E11)
- Evidence classes: context, discriminator, growth, static
- Holder: static/global
- Region: heap
- Shape: steady
- Site: `reports/builder.py:14`
- Annotation: Same site and mechanism as F4: lru_cache on instance method keyed by per-request ReportBuilder (detector duplicate).
- Sibling hypotheses: F2.n1
- Suggested archetype (unconfirmed): L-COLLECT
- Supporting evidence: E5, E10, E11, E17

### F3: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:reports.builder.ReportBuilder.category_totals.bytes` 2290 bytes/n (steady, gate pass, E11)
- Evidence classes: context, discriminator, growth, static
- Holder: static/global
- Region: heap
- Shape: steady
- Site: `reports/builder.py:14`
- Annotation: Same site and mechanism as F4: lru_cache on instance method keyed by per-request ReportBuilder (detector duplicate).
- Sibling hypotheses: F3.n1
- Suggested archetype (unconfirmed): L-COLLECT
- Supporting evidence: E6, E10, E11, E18

### F4: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:reports.builder.ReportBuilder.category_totals.bytes` 2290 bytes/n (steady, gate pass, E11)
- Evidence classes: context, discriminator, growth, static
- Holder: static/global
- Owner: `reports.builder.ReportBuilder.category_totals`
- Region: heap
- Shape: steady
- Site: `reports/builder.py:14`
- Writers: `reports/builder.py:14`
- Annotation: functools.lru_cache(maxsize=None) on instance method category_totals: cache key includes self, and ReportService.handle creates a new ReportBuilder per request, so every call misses and adds an entry; the cache (module-level, unbounded) also pins every builder (store, column buffers) and the tuple results. Missing pairing: no eviction/bound, and cache lifetime (class-level) is longer than the per-request builder it is keyed on.
- Sibling hypotheses: F4.n1
- Supporting evidence: E7, E10, E11, E14, E15

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (leak) E16
  - Hypothesis: N-WARM: cache fills to bounded working set then plateaus
  - Refuted if: cache size keeps growing over extra cycles
  - Contradicting evidence: E16
  - Originally supported by: E12
- **F2.n1** (UNCLASSIFIED): discriminator E17
  - Ruled out by: plateau+purge (leak) E17
  - Hypothesis: N-WARM: cache fills to bounded working set then plateaus
  - Refuted if: cache size keeps growing over extra cycles
  - Contradicting evidence: E17
  - Originally supported by: E12
- **F3.n1** (UNCLASSIFIED): discriminator E18
  - Ruled out by: plateau+purge (leak) E18
  - Hypothesis: N-WARM: cache fills to bounded working set then plateaus
  - Refuted if: cache size keeps growing over extra cycles
  - Contradicting evidence: E18
  - Originally supported by: E12
- **F4.n1** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (leak) E15
  - Hypothesis: N-WARM: cache fills to a bounded working set then plateaus
  - Refuted if: cache size and bytes keep growing over extra cycles after purge
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
- E16: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E16: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E16: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E16: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E16: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E16: limitation: points 1 to 9 come from S1; the extra points from S5
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E17: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E17: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E17: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E17: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E17: limitation: points 1 to 9 come from S1; the extra points from S7
- E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E17: limitation: gate and plateau thresholds are provisional (not calibrated)
- E18: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E18: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E18: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E18: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E18: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E18: limitation: points 1 to 9 come from S1; the extra points from S9
- E18: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E18: limitation: gate and plateau thresholds are provisional (not calibrated)

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
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (c66057ee1187) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (02c257c8e653) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (02c257c8e653) | trend/T1.json |
| E13 | alloc-stack | live | py-tracemalloc 0.1.0 | artifact allocs/A1.json (ab6f2098e8e7) | allocs/A1.json |
| E14 | growth-series | live | py-tracemalloc 0.1.0 | artifact series/S2.json (f59141a90ddd) | series/S2.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S4.json (fe2d19b40af6) | alloc/A1.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S6.json (2ad0488976be) | alloc/A2.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S8.json (5bb785c59ba0) | alloc/A3.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S10.json (fa7b53a26f06) | alloc/A4.json |
