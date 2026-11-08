# DLA report

Run: `20261108-125320-3ced`

Status: COMPLETE

Findings: 4 enumerated, 4 dispositioned; confirmed 0, strong 0, refuted 2, suspected 0, unresolved 2

Residual: not measured

Coverage: python 3520 iterations, 13 of 13 entry points exercised

## Findings

### F1.n1: Hypothesis N-HEADROOM: cyclic garbage deferred by raised gc threshold, not retained (UNCLASSIFIED)

- State: unresolved(env_unavailable)
- Evidence classes: discriminator
- Sibling hypothesis of: F1
- Hypothesis: N-HEADROOM: cyclic garbage deferred by raised gc threshold, not retained
- Refuted if: retention persists after gc.collect() purge
- Reason: env_unavailable
- Remedy: Supported non-leak hypothesis (N-HEADROOM); no leak-favoring discriminator exists to refute it. Nothing further needed unless the production workload differs from the synthetic one.
- Supporting evidence: E14

### F2.n1: Hypothesis N-HEADROOM: gc gen0 threshold 40000 defers collection of cyclic Tree/Node garbage; bounded plateau (UNCLASSIFIED)

- State: unresolved(env_unavailable)
- Evidence classes: growth
- Sibling hypothesis of: F2
- Hypothesis: N-HEADROOM: gc gen0 threshold 40000 defers collection of cyclic Tree/Node garbage; bounded plateau
- Refuted if: after gc.collect() purge, py.traced returns to baseline = headroom (non-leak); stays high = real retention
- Reason: env_unavailable
- Remedy: Supported non-leak hypothesis (N-HEADROOM); no leak-favoring discriminator exists to refute it. Nothing further needed unless the production workload differs from the synthetic one.
- Supporting evidence: E12

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (non-leak) E15
  - Contradicting evidence: E15
  - Originally supported by: E8, E11
- **F2** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (non-leak) E14
  - Contradicting evidence: E14
  - Originally supported by: E12, E11

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
- E15: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E15: limitation: points 1 to 9 come from S1; the extra points from S4
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | outline converter, python package outlines/ |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | whole repo |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | memory graph trends up over document queue; RSS alerts |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | python |
| `intake.build` | Does the project build locally? | answered | E1 | pure python, no build |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | python3 -m outlines.load --iterations N |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 raw |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 3600s |

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
| E11 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (59e6b34094c5) | harness/H1/run-S1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S1.json (da192b4ca41c) | trend/T1.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (da192b4ca41c) | trend/T1.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S3.json (dd941763fb4d) | alloc/A1.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S5.json (c4b70af78c51) | alloc/A2.json |
