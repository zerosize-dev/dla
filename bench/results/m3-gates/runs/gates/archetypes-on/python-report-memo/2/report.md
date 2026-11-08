# DLA report

Run: `20261108-130331-dbcc`

Status: COMPLETE

Findings: 8 enumerated, 8 dispositioned; confirmed 4, strong 0, refuted 4, suspected 0, unresolved 0

Residual: heap 70.82900432900442 bytes/n unattributed of 2360.5851731601733 total (threshold 590.1462932900433)

Coverage: python 3520 iterations, 10 of 10 entry points exercised

## Findings

### F1: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:reports.builder.ReportBuilder.category_totals.bytes` 2290 bytes/n (steady, gate pass, E12)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `reports.builder.ReportBuilder.category_totals`
- Region: heap
- Shape: steady
- Site: `reports/builder.py:14`
- Annotation: same site as F4 (builder.py:14), detector hit on the same lru_cache-on-method defect
- Sibling hypotheses: F1.n1
- Suggested archetype (unconfirmed): L-COLLECT
- Supporting evidence: E5, E11, E12

### F2: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:reports.builder.ReportBuilder.category_totals.bytes` 2290 bytes/n (steady, gate pass, E12)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `reports.builder.ReportBuilder.category_totals`
- Region: heap
- Shape: steady
- Site: `reports/builder.py:14`
- Annotation: same site as F4 (builder.py:14), detector hit on the same lru_cache-on-method defect
- Sibling hypotheses: F2.n1
- Suggested archetype (unconfirmed): L-COLLECT
- Supporting evidence: E6, E11, E12

### F3: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:reports.builder.ReportBuilder.category_totals.bytes` 2290 bytes/n (steady, gate pass, E12)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `reports.builder.ReportBuilder.category_totals`
- Region: heap
- Shape: steady
- Site: `reports/builder.py:14`
- Annotation: same site as F4 (builder.py:14), detector hit on the same lru_cache-on-method defect
- Sibling hypotheses: F3.n1
- Suggested archetype (unconfirmed): L-COLLECT
- Supporting evidence: E7, E11, E12

### F4: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:reports.builder.ReportBuilder.category_totals.bytes` 2290 bytes/n (steady, gate pass, E12)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `reports.builder.ReportBuilder.category_totals`
- Region: heap
- Shape: steady
- Site: `reports/builder.py:14`
- Writers: `reports/builder.py:14`
- Annotation: lru_cache(maxsize=None) on method; key includes self; ReportBuilder created per request in service.py:14; no cache_clear anywhere
- Sibling hypotheses: F4.n1
- Supporting evidence: E8, E11, E12

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (leak) E15
  - Hypothesis: N-WARM: bounded warm-up cache
  - Refuted if: cache size keeps growing across extra cycles after purge
  - Contradicting evidence: E15
  - Originally supported by: E13
- **F2.n1** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (leak) E16
  - Hypothesis: N-WARM: bounded warm-up cache
  - Refuted if: cache size keeps growing across extra cycles after purge
  - Contradicting evidence: E16
  - Originally supported by: E13
- **F3.n1** (UNCLASSIFIED): discriminator E17
  - Ruled out by: plateau+purge (leak) E17
  - Hypothesis: N-WARM: bounded warm-up cache
  - Refuted if: cache size keeps growing across extra cycles after purge
  - Contradicting evidence: E17
  - Originally supported by: E13
- **F4.n1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (leak) E14
  - Hypothesis: N-WARM: bounded warm-up cache that plateaus
  - Refuted if: cache size keeps growing across extra cycles after purge
  - Contradicting evidence: E14
  - Originally supported by: E13

## Blind spots and unknowns

### Coverage gaps

- jvm/codeql: missing (install failed: /tmp/dla-bench/m3-gates/gates/archetypes-on/python-report-memo/2/config/plugins/data/dla/tools/codeql: Read-only file system (os error 30)). Install hint: dla install codeql
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
- E11: limitation: command mode has one key mode (native): the requested fresh/repeated modes only exist in driver mode
- E11: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E12: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E14: limitation: points 1 to 9 come from S1; the extra points from S2
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E15: limitation: points 1 to 9 come from S1; the extra points from S4
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E16: limitation: points 1 to 9 come from S1; the extra points from S6
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E17: limitation: points 1 to 9 come from S1; the extra points from S8
- E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E17: limitation: gate and plateau thresholds are provisional (not calibrated)
- E18: partial coverage (1 of 3 examined)
- E18: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E18: limitation: threshold is provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | Python report service in reports/ |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | whole repo, python only |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | RSS climbs with requests; daily restart |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E3 | python |
| `intake.build` | Does the project build locally? | answered | E4 | build ok |
| `intake.workload` | What can exercise the suspected path? | answered | E3 | python3 -m reports.load --iterations N |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 raw values |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 3600s |

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
| E11 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (3c926f3038e2) | harness/H1/run-S1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S1.json (a2fde79c7929) | trend/T1.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (a2fde79c7929) | trend/T1.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S3.json (d4ecf04cea2c) | alloc/A1.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S5.json (df7a1119086f) | alloc/A2.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S7.json (6706790bdbaa) | alloc/A3.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S9.json (73e6849d6114) | alloc/A4.json |
| E18 | residual | test | dla 0.1.0 | artifact series/S1.json (a2fde79c7929) | gap/residual-S1-E18.json |
