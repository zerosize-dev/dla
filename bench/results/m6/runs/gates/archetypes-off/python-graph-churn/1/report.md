# DLA report

Run: `20261110-181228-0d67`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 2 enumerated, 2 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 0

Residual: heap 1.675 bytes/n unattributed of 1.675 total (threshold 1.221700909090909) ABOVE THRESHOLD, see O1; heap 1.072267316017316 bytes/n unattributed of 1.072267316017316 total (threshold 0.268066829004329) ABOVE THRESHOLD, see O1; heap 1.0924999999999998 bytes/n unattributed of 1.0924999999999998 total (threshold 0.27312499999999995) ABOVE THRESHOLD, see O1

Coverage: python 3520 iterations, 13 of 13 entry points exercised

## Findings

### F1.n1: Hypothesis N-HEADROOM: cyclic Node/Tree garbage (parent<->children, node.tree) awaits the cyclic GC, whose young threshold tuning.py:11 raised to 40000 (UNCLASSIFIED)

- State: confirmed
- Explained by N-HEADROOM (discriminator E14)
- Evidence classes: context, discriminator
- Sibling hypothesis of: F1
- Hypothesis: N-HEADROOM: cyclic Node/Tree garbage (parent<->children, node.tree) awaits the cyclic GC, whose young threshold tuning.py:11 raised to 40000
- Refuted if: growth continues over the extra cycles
- Supporting evidence: E14, E10

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (non-leak) E14
  - Contradicting evidence: E14
  - Originally supported by: E7

## Blind spots and unknowns

### Coverage gaps

- python/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- python/codeql: missing (codeql is not on PATH or in the tools directory). Install hint: dla install codeql
- python/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)

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
- E13: partial coverage (1 of 3 examined)
- E13: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E13: limitation: threshold is provisional (not calibrated)
- E14: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E14: limitation: points 1 to 9 come from S1; the extra points from S2
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E15: limitation: points 1 to 9 come from S1; the extra points from S2
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E16: limitation: points 1 to 9 come from S1; the extra points from S2
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: partial coverage (1 of 3 examined)
- E19: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E19: limitation: threshold is provisional (not calibrated)
- E20: partial coverage (1 of 3 examined)
- E20: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E20: limitation: threshold is provisional (not calibrated)
- E21: partial coverage (1 of 3 examined)
- E21: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E21: limitation: threshold is provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | — |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | — |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | — |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E3 | — |
| `intake.build` | Does the project build locally? | answered | E3 | — |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | — |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | — |
| E2 | run-config | static | dla 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | — |
| E3 | probe | static | dla-probe 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | probe/probe.json |
| E4 | static-hit | static | ruff 0.16.9 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | sweep/python-ruff.json |
| E5 | static-hit | static | pylint 4.1.3 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | sweep/python-pylint.json |
| E6 | static-hit | static | dla 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | sweep/python-dla.json |
| E7 | census-holder | static | dla-census 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | census/python.json |
| E8 | workload | static | dla 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | workload/workloads.json |
| E9 | workload | static | dla 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | harness/H1/harness.json |
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (45f57a5e5df3) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (16e2fe622f90) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (16e2fe622f90) | trend/T1.json |
| E13 | residual | test | dla 0.1.0 | artifact series/S1.json (16e2fe622f90) | gap/residual-S1-E13.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S3.json (599657f504f6) | alloc/A1.json |
| E15 | growth-series | test | dla 0.1.0 | artifact series/S3.json (599657f504f6) | trend/T3.json |
| E16 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (599657f504f6) | trend/T3.json |
| E17 | growth-series | test | dla 0.1.0 | artifact series/S2.json (9bbb30ac1353) | trend/T4.json |
| E18 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (9bbb30ac1353) | trend/T4.json |
| E19 | residual | test | dla 0.1.0 | artifact series/S1.json (16e2fe622f90) | gap/residual-S1-E19.json |
| E20 | residual | test | dla 0.1.0 | artifact series/S2.json (9bbb30ac1353) | gap/residual-S2-E20.json |
| E21 | residual | test | dla 0.1.0 | artifact series/S3.json (599657f504f6) | gap/residual-S3-E21.json |
