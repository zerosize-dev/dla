# DLA report

Run: `20261108-131342-7840`

Status: COMPLETE

Findings: 2 enumerated, 2 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 0

Residual: not measured

Coverage: python 3520 iterations, 13 of 13 entry points exercised

## Findings

### F1.n1: Hypothesis N-HEADROOM: Node<->Tree cycles are garbage awaiting cyclic GC deferred by gen0 threshold 40000; memory is bounded, not retained (UNCLASSIFIED)

- State: confirmed
- Explained by N-HEADROOM (discriminator E14)
- Evidence classes: context, discriminator
- Sibling hypothesis of: F1
- Hypothesis: N-HEADROOM: Node<->Tree cycles are garbage awaiting cyclic GC deferred by gen0 threshold 40000; memory is bounded, not retained
- Refuted if: after gc.collect() live object count/memory still grows linearly with documents
- Supporting evidence: E11, E14

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (non-leak) E14
  - Contradicting evidence: E14
  - Originally supported by: E8

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

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E3 | outlines converter (python package) |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E3 | outlines/ package |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | memory graph rises while processing document queue; RSS alerts |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E3 | python |
| `intake.build` | Does the project build locally? | answered | E4 | python build ok |
| `intake.workload` | What can exercise the suspected path? | answered | E9 | python:main:outlines/load.py |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 raw |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 60m |

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
| E11 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (908a13edf011) | harness/H1/run-S1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S1.json (0adad059329a) | trend/T1.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (0adad059329a) | trend/T1.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S3.json (2d249beae0cc) | alloc/A1.json |
