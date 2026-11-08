# DLA report

Run: `20261108-123248-442d`

Status: COMPLETE

Findings: 4 enumerated, 4 dispositioned; confirmed 2, strong 0, refuted 2, suspected 0, unresolved 0

Residual: not measured

Coverage: python 3520 iterations, 13 of 13 entry points exercised

## Findings

### F1.n1: Hypothesis N-HEADROOM: per-document Tree/Node cycles freed by cyclic GC, deferred by gen0 threshold 40000 (UNCLASSIFIED)

- State: confirmed
- Explained by N-HEADROOM (discriminator E15)
- Evidence classes: context, discriminator
- Sibling hypothesis of: F1
- Hypothesis: N-HEADROOM: per-document Tree/Node cycles freed by cyclic GC, deferred by gen0 threshold 40000
- Refuted if: growth continues over the extra cycles
- Supporting evidence: E15, E11, E14

### F2.n1: Hypothesis N-HEADROOM: traced growth is cyclic garbage awaiting collection because gc gen0 threshold is 40000 (tuning.py:11) (UNCLASSIFIED)

- State: confirmed
- Explained by N-HEADROOM (discriminator E14)
- Evidence classes: context, discriminator, growth
- Sibling hypothesis of: F2
- Hypothesis: N-HEADROOM: traced growth is cyclic garbage awaiting collection because gc gen0 threshold is 40000 (tuning.py:11)
- Refuted if: after gc.collect() traced bytes stay elevated and keep growing with iterations
- Supporting evidence: E12, E11, E14

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (non-leak) E15
  - Contradicting evidence: E15
  - Originally supported by: E8, E10
- **F2** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (non-leak) E14
  - Contradicting evidence: E14
  - Originally supported by: E12, E10

## Blind spots and unknowns

### Coverage gaps

- jvm/codeql: missing (install failed: /tmp/dla-bench/m3-gates/gates/archetypes-on/python-graph-churn/1/config/plugins/data/dla/tools/codeql: Read-only file system (os error 30)). Install hint: dla install codeql
- python/codeql: missing (install failed: /tmp/dla-bench/m3-gates/gates/archetypes-on/python-graph-churn/1/config/plugins/data/dla/tools/codeql: Read-only file system (os error 30)). Install hint: dla install codeql

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

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | python package 'outlines' (outline->TOC converter); symptom: memory graph rising during document queue; load via python3 -m outlines.load; tests/ present; consent+budget from runner (E2, 3600s) |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | python package 'outlines' (outline->TOC converter); symptom: memory graph rising during document queue; load via python3 -m outlines.load; tests/ present; consent+budget from runner (E2, 3600s) |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | python package 'outlines' (outline->TOC converter); symptom: memory graph rising during document queue; load via python3 -m outlines.load; tests/ present; consent+budget from runner (E2, 3600s) |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | python package 'outlines' (outline->TOC converter); symptom: memory graph rising during document queue; load via python3 -m outlines.load; tests/ present; consent+budget from runner (E2, 3600s) |
| `intake.build` | Does the project build locally? | answered | E1 | python package 'outlines' (outline->TOC converter); symptom: memory graph rising during document queue; load via python3 -m outlines.load; tests/ present; consent+budget from runner (E2, 3600s) |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | python package 'outlines' (outline->TOC converter); symptom: memory graph rising during document queue; load via python3 -m outlines.load; tests/ present; consent+budget from runner (E2, 3600s) |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E1 | python package 'outlines' (outline->TOC converter); symptom: memory graph rising during document queue; load via python3 -m outlines.load; tests/ present; consent+budget from runner (E2, 3600s) |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | python package 'outlines' (outline->TOC converter); symptom: memory graph rising during document queue; load via python3 -m outlines.load; tests/ present; consent+budget from runner (E2, 3600s) |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | — |
| E2 | run-config | static | dla 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | — |
| E3 | probe | static | dla-probe 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | probe/probe.json |
| E4 | probe | static | dla-probe 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | probe/probe.json |
| E5 | static-hit | static | ruff 0.16.9 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | sweep/ruff.json |
| E6 | static-hit | static | pylint 4.1.3 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | sweep/pylint.json |
| E7 | static-hit | static | dla 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | sweep/dla.json |
| E8 | census-holder | static | dla-census 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | census/python.json |
| E9 | workload | static | dla 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | workload/workloads.json |
| E10 | workload | static | dla 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | harness/H1/harness.json |
| E11 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (3eee47fb9e8c) | harness/H1/run-S1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S1.json (6005498ae523) | trend/T1.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (6005498ae523) | trend/T1.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S3.json (db18c0836556) | alloc/A1.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S5.json (de7f5ee301de) | alloc/A2.json |
