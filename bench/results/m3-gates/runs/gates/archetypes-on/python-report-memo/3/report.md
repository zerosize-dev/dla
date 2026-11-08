# DLA report

Run: `20261108-132511-3bd3`

Status: COMPLETE

Findings: 5 enumerated, 5 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 3

Residual: not measured

Coverage: python 3520 iterations, 10 of 10 entry points exercised

## Findings

### F4: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:reports.builder.ReportBuilder.category_totals` 36.73 count/n (steady, gate pass, E13)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `reports.builder.ReportBuilder.category_totals`
- Region: heap
- Shape: steady
- Site: `reports/builder.py:14`
- Writers: `reports/builder.py:14`
- Annotation: lru_cache(maxsize=None) on method; key includes self; a new ReportBuilder per request (service.py handle) so every builder, its store ref, column buffers and result tuple are pinned forever. No remover.
- Sibling hypotheses: F4.n1
- Supporting evidence: E8, E11, E13

### F1: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: growth, static
- Site: `reports/builder.py:14`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: tool_missing
- Remedy: same site as confirmed F4 (builder.py:14); detector duplicate, no separate harness column; label/explain/bundle commands absent in this release
- Supporting evidence: E5, E13

### F2: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: growth, static
- Site: `reports/builder.py:14`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: tool_missing
- Remedy: same site as confirmed F4 (builder.py:14); detector duplicate, no separate harness column; label/explain/bundle commands absent in this release
- Supporting evidence: E6, E13

### F3: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: growth, static
- Site: `reports/builder.py:14`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: tool_missing
- Remedy: same site as confirmed F4 (builder.py:14); detector duplicate, no separate harness column; label/explain/bundle commands absent in this release
- Supporting evidence: E7, E13

## Ruled out

- **F4.n1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (leak) E14
  - Hypothesis: N-WARM bounded warm-up cache of (account,month) keys
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E14
  - Originally supported by: E13

## Blind spots and unknowns

### Coverage gaps

- python/codeql: missing (codeql is not on PATH or in the tools directory). Install hint: dla install codeql

### Runtime blind spots

None recorded.

### Entry points not exercisable

None recorded.

### Evidence limitations and unknowns

- E8: partial coverage
- E8: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E8: limitation: writers and removers are textual name matches
- E9: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E9: limitation: workloads were discovered by reading files; none was run
- E10: limitation: harness written; nothing was run (dla harness run measures it)
- E10: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E11: partial coverage (9 of 9 examined)
- E11: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E12: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E14: limitation: points 1 to 9 come from S1; the extra points from S2
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | Python repo: report service (reports/) |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | whole repo, reports package |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | RSS climbs per few thousand requests; daily restart |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E3 | python only |
| `intake.build` | Does the project build locally? | answered | E4 | builds ok |
| `intake.workload` | What can exercise the suspected path? | answered | E9 | python:main:reports/load.py |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4, raw values |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 60m |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | — |
| E2 | run-config | static | dla 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | probe/probe.json |
| E4 | probe | static | dla-probe 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | probe/probe.json |
| E5 | static-hit | static | ruff 0.16.9 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | sweep/ruff.json |
| E6 | static-hit | static | pylint 4.1.3 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | sweep/pylint.json |
| E7 | static-hit | static | dla 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | sweep/dla.json |
| E8 | census-holder | static | dla-census 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | census/python.json |
| E9 | workload | static | dla 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | workload/workloads.json |
| E10 | workload | static | dla 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | harness/H1/harness.json |
| E11 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (2d0dc410eb58) | harness/H1/run-S1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S1.json (193797fb5e0a) | trend/T1.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (193797fb5e0a) | trend/T1.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S3.json (b962e8573e4a) | alloc/A1.json |
