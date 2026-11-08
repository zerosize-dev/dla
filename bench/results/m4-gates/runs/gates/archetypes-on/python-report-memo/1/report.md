# DLA report

Run: `20261108-194458-8995`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 8 enumerated, 8 dispositioned; confirmed 4, strong 0, refuted 4, suspected 0, unresolved 0

Residual: not measured

Coverage: python 3520 iterations, 10 of 10 entry points exercised

## Findings

### F1: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:reports.builder.ReportBuilder.category_totals` 36.73 count/n (steady, gate pass, E12)
- Evidence classes: context, growth, static
- Holder: static/global
- Region: heap
- Shape: steady
- Site: `reports/builder.py:14`
- Annotation: Same site and mechanism as F4 (detector hit on the lru_cache-on-method at builder.py:14); see F4.
- Sibling hypotheses: F1.n1
- Suggested archetype (unconfirmed): L-COLLECT
- Supporting evidence: E4, E10, E12

### F2: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:reports.builder.ReportBuilder.category_totals` 36.73 count/n (steady, gate pass, E12)
- Evidence classes: context, growth, static
- Holder: static/global
- Region: heap
- Shape: steady
- Site: `reports/builder.py:14`
- Annotation: Same site and mechanism as F4 (detector hit on the lru_cache-on-method at builder.py:14); see F4.
- Sibling hypotheses: F2.n1
- Suggested archetype (unconfirmed): L-COLLECT
- Supporting evidence: E5, E10, E12

### F3: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:reports.builder.ReportBuilder.category_totals` 36.73 count/n (steady, gate pass, E12)
- Evidence classes: context, growth, static
- Holder: static/global
- Region: heap
- Shape: steady
- Site: `reports/builder.py:14`
- Annotation: Same site and mechanism as F4 (detector hit on the lru_cache-on-method at builder.py:14); see F4.
- Sibling hypotheses: F3.n1
- Suggested archetype (unconfirmed): L-COLLECT
- Supporting evidence: E6, E10, E12

### F4: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:reports.builder.ReportBuilder.category_totals` 36.73 count/n (steady, gate pass, E12)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `reports.builder.ReportBuilder.category_totals`
- Region: heap
- Shape: steady
- Site: `reports/builder.py:14`
- Writers: `reports/builder.py:14`
- Annotation: functools.lru_cache(maxsize=None) on instance method category_totals: cache key includes self, and ReportService.handle (reports/service.py:15) creates a new ReportBuilder per request, so every request adds an entry that pins the builder (store, column buffers) and the totals tuple; no entry is ever evicted or cleared. Missing pairing: a bounded/per-instance cache or a cache keyed on (account, month) outside the instance. Class of repair: don't memoize bound methods with an unbounded process-wide cache.
- Sibling hypotheses: F4.n1
- Supporting evidence: E7, E10, E12

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (leak) E14
  - Hypothesis: N-WARM: bounded warm-up cache; refuted if growth continues
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E14
  - Originally supported by: E12
- **F2.n1** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (leak) E15
  - Hypothesis: N-WARM: bounded warm-up cache; refuted if growth continues
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E15
  - Originally supported by: E12
- **F3.n1** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (leak) E16
  - Hypothesis: N-WARM: bounded warm-up cache; refuted if growth continues
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E16
  - Originally supported by: E12
- **F4.n1** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (leak) E13
  - Hypothesis: N-WARM: bounded warm-up cache that plateaus; refuted if currsize keeps growing over the extra cycles
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E13
  - Originally supported by: E12

## Blind spots and unknowns

### Coverage gaps

- python/codeql: missing (codeql is not on PATH or in the tools directory). Install hint: dla install codeql

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
- E10: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E11: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E13: limitation: points 1 to 9 come from S1; the extra points from S2
- E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- E14: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E14: limitation: points 1 to 9 come from S1; the extra points from S4
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E15: limitation: points 1 to 9 come from S1; the extra points from S6
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E16: limitation: points 1 to 9 come from S1; the extra points from S8
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)

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
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (92a3ddaf9789) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (8e59ef31fe32) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (8e59ef31fe32) | trend/T1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S3.json (2b7730571fc7) | alloc/A1.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S5.json (00d4510a97cf) | alloc/A2.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S7.json (98617c5a25d5) | alloc/A3.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S9.json (865490ebcd58) | alloc/A4.json |
