# DLA report

Run: `20261109-202939-47b9`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 5 enumerated, 5 dispositioned; confirmed 1, strong 0, refuted 1, suspected 3, unresolved 0

Residual: not measured

Coverage: python 3520 iterations, 10 of 10 entry points exercised

## Findings

### F4: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:reports.builder.ReportBuilder.category_totals.bytes` 2290 bytes/n (steady, gate pass, E11)
- Evidence classes: context, discriminator, growth, static
- Holder: static/global
- Owner: `reports.builder.ReportBuilder.category_totals`
- Region: heap
- Shape: steady
- Site: `reports/builder.py:14`
- Annotation: functools.lru_cache(maxsize=None) on instance method ReportBuilder.category_totals (reports/builder.py:14): unbounded cache keyed on (self, account, month). The cache's global wrapper strongly holds every per-request ReportBuilder (service.handle creates a new one each request) plus its column buffers, and every (account, month) result tuple; entries never evicted. Missing pairing: bounded/per-instance cache or cache clear. Unlabelled (UNCLASSIFIED).
- Sibling hypotheses: F4.n1
- Supporting evidence: E7, E10, E11, E14, E15

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: context, static
- Site: `reports/builder.py:14`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: Static detector hit at reports/builder.py:14, the same site as confirmed F4; the dynamic evidence is recorded under F4 and no separate harness probe was bound to this finding.
- Supporting evidence: E4, E10

### F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: context, static
- Site: `reports/builder.py:14`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: Static detector hit at reports/builder.py:14, the same site as confirmed F4; the dynamic evidence is recorded under F4 and no separate harness probe was bound to this finding.
- Supporting evidence: E5, E10

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: context, static
- Site: `reports/builder.py:14`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: Static detector hit at reports/builder.py:14, the same site as confirmed F4; the dynamic evidence is recorded under F4 and no separate harness probe was bound to this finding.
- Supporting evidence: E6, E10

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
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (bd9cc024e397) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (a63700bd1a6b) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (a63700bd1a6b) | trend/T1.json |
| E13 | alloc-stack | live | py-tracemalloc 0.1.0 | artifact allocs/A1.json (693a6ea05fa0) | allocs/A1.json |
| E14 | growth-series | live | py-tracemalloc 0.1.0 | artifact series/S2.json (61cd12546c7c) | series/S2.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S4.json (881facb9d99c) | alloc/A1.json |
