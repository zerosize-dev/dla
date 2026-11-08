# DLA report

Run: `20261108-201221-bfeb`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 5 enumerated, 5 dispositioned; confirmed 1, strong 0, refuted 1, suspected 3, unresolved 0

Residual: not measured

Coverage: python 3520 iterations, 10 of 10 entry points exercised

## Findings

### F4: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:reports.builder.ReportBuilder.category_totals` 36.73 count/n (steady, gate pass, E12)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `reports.builder.ReportBuilder.category_totals`
- Region: heap
- Shape: steady
- Site: `reports/builder.py:14`
- Writers: `reports/builder.py:14`, `reports/service.py:12`
- Annotation: functools.lru_cache(maxsize=None) on instance method ReportBuilder.category_totals: cache is class-level, key includes self, and ReportService.handle builds a new ReportBuilder per request, so every call misses and the cache pins each builder (with its column buffers and store ref) plus the totals tuple forever. Missing pairing: bound/evict (or per-instance/no cache).
- Sibling hypotheses: F4.n1
- Supporting evidence: E7, E10, E12

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: growth, static
- Site: `reports/builder.py:14`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: Same site and mechanism as confirmed F4 (reports/builder.py:14); detector duplicate, no separate harness probe. Fixing F4 covers it.
- Supporting evidence: E4, E12

### F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: growth, static
- Site: `reports/builder.py:14`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: Same site and mechanism as confirmed F4 (reports/builder.py:14); detector duplicate, no separate harness probe. Fixing F4 covers it.
- Supporting evidence: E5, E12

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: growth, static
- Site: `reports/builder.py:14`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: Same site and mechanism as confirmed F4 (reports/builder.py:14); detector duplicate, no separate harness probe. Fixing F4 covers it.
- Supporting evidence: E6, E12

## Ruled out

- **F4.n1** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (leak) E13
  - Hypothesis: N-WARM: bounded warm-up cache that plateaus
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
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (3f71789e2e53) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (951d4d193d98) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (951d4d193d98) | trend/T1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S3.json (bbf2b940eb22) | alloc/A1.json |
