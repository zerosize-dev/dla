# DLA report

Run: `20261108-124747-5b0c`

Status: COMPLETE

Findings: 8 enumerated, 8 dispositioned; confirmed 4, strong 0, refuted 4, suspected 0, unresolved 0

Residual: not measured

Coverage: python 3520 iterations, 10 of 10 entry points exercised

## Findings

### F1.n1: Hypothesis N-INTENT bounded intended retention (F1) (UNCLASSIFIED)

- State: confirmed
- Explained by N-INTENT bounded intended retention (F1) (discriminator E17)
- Evidence classes: context, discriminator, growth
- Sibling hypothesis of: F1
- Hypothesis: N-INTENT bounded intended retention (F1)
- Refuted if: holder size grows past its bound
- Supporting evidence: E12, E10, E17

### F2.n1: Hypothesis N-INTENT bounded intended retention (F2) (UNCLASSIFIED)

- State: confirmed
- Explained by N-INTENT bounded intended retention (F2) (discriminator E15)
- Evidence classes: context, discriminator, growth
- Sibling hypothesis of: F2
- Hypothesis: N-INTENT bounded intended retention (F2)
- Refuted if: holder size grows past its bound
- Supporting evidence: E12, E10, E15

### F3.n1: Hypothesis N-INTENT bounded intended retention (F3) (UNCLASSIFIED)

- State: confirmed
- Explained by N-INTENT bounded intended retention (F3) (discriminator E16)
- Evidence classes: context, discriminator, growth
- Sibling hypothesis of: F3
- Hypothesis: N-INTENT bounded intended retention (F3)
- Refuted if: holder size grows past its bound
- Supporting evidence: E12, E10, E16

### F4.n1: Hypothesis N-WARM bounded lru_cache(128) fills in the first ~128 requests then plateaus (UNCLASSIFIED)

- State: confirmed
- Explained by N-WARM bounded lru_cache(128) fills in the first ~128 requests then plateaus (discriminator E14)
- Evidence classes: context, discriminator, growth
- Sibling hypothesis of: F4
- Hypothesis: N-WARM bounded lru_cache(128) fills in the first ~128 requests then plateaus
- Refuted if: traced memory keeps rising after the cache size column is pinned at 128, with extra points
- Supporting evidence: E12, E10, E14

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E17
  - Ruled out by: plateau+purge (non-leak) E17
  - Contradicting evidence: E17
  - Originally supported by: E5, E10
- **F2** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (non-leak) E15
  - Contradicting evidence: E15
  - Originally supported by: E8, E10
- **F3** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (non-leak) E16
  - Contradicting evidence: E16
  - Originally supported by: E8, E10
- **F4** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (non-leak) E14
  - Contradicting evidence: E14
  - Originally supported by: E12, E10

## Blind spots and unknowns

### Coverage gaps

- jvm/codeql: missing (install failed: /tmp/dla-bench/m3-gates/gates/archetypes-on/python-report-memo-bounded/1/config/plugins/data/dla/tools/codeql: Read-only file system (os error 30)). Install hint: dla install codeql
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

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | reports package (python), ReportService |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | whole repo |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | RSS climbs with requests; daily restart |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | python |
| `intake.build` | Does the project build locally? | answered | E1 | pure python, no build |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | python3 -m reports.load --iterations N |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 60m |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 193963928cbca22594b461b4b8d3fe954c299cf0 | — |
| E2 | run-config | static | dla 0.1.0 | repo 193963928cbca22594b461b4b8d3fe954c299cf0 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 193963928cbca22594b461b4b8d3fe954c299cf0 | probe/probe.json |
| E4 | probe | static | dla-probe 0.1.0 | repo 193963928cbca22594b461b4b8d3fe954c299cf0 | probe/probe.json |
| E5 | static-hit | static | ruff 0.16.9 | repo 193963928cbca22594b461b4b8d3fe954c299cf0 | sweep/ruff.json |
| E6 | static-hit | static | pylint 4.1.3 | repo 193963928cbca22594b461b4b8d3fe954c299cf0 | sweep/pylint.json |
| E7 | static-hit | static | dla 0.1.0 | repo 193963928cbca22594b461b4b8d3fe954c299cf0 | sweep/dla.json |
| E8 | census-holder | static | dla-census 0.1.0 | repo 193963928cbca22594b461b4b8d3fe954c299cf0 | census/python.json |
| E9 | workload | static | dla 0.1.0 | repo 193963928cbca22594b461b4b8d3fe954c299cf0 | workload/workloads.json |
| E10 | workload | static | dla 0.1.0 | repo 193963928cbca22594b461b4b8d3fe954c299cf0 | harness/H1/harness.json |
| E11 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (59a65c0ef2e0) | harness/H1/run-S1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S1.json (8b30aea80777) | trend/T1.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (8b30aea80777) | trend/T1.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S3.json (fa21a2fa13ac) | alloc/A1.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S5.json (e64c15b2f9fb) | alloc/A2.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S7.json (527b6121fde3) | alloc/A3.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S9.json (3c3fc8269a26) | alloc/A4.json |
