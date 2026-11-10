# DLA report

Run: `20261110-190146-47bc`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 13 enumerated, 13 dispositioned; confirmed 0, strong 0, refuted 13, suspected 0, unresolved 0

Residual: heap 0 bytes/n unattributed of 0 total (threshold 0)

Coverage: native 3520 iterations, 0 of 7 entry points exercised

## Findings

No findings to report.

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (non-leak) E11
  - Contradicting evidence: E11
  - Originally supported by: E5
- **F2** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (non-leak) E11
  - Contradicting evidence: E11
  - Originally supported by: E5
- **F3** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (non-leak) E11
  - Contradicting evidence: E11
  - Originally supported by: E5
- **F4** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (non-leak) E11
  - Contradicting evidence: E11
  - Originally supported by: E5
- **F5** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (non-leak) E11
  - Contradicting evidence: E11
  - Originally supported by: E5
- **F6** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (non-leak) E11
  - Contradicting evidence: E11
  - Originally supported by: E5
- **F7** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (non-leak) E11
  - Contradicting evidence: E11
  - Originally supported by: E5
- **F8** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (non-leak) E11
  - Contradicting evidence: E11
  - Originally supported by: E5
- **F9** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (non-leak) E11
  - Contradicting evidence: E11
  - Originally supported by: E5
- **F10** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (non-leak) E11
  - Contradicting evidence: E11
  - Originally supported by: E5
- **F11** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (non-leak) E11
  - Contradicting evidence: E11
  - Originally supported by: E5
- **F12** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (non-leak) E11
  - Contradicting evidence: E11
  - Originally supported by: E5
- **F13** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (non-leak) E11
  - Contradicting evidence: E11
  - Originally supported by: E5

## Blind spots and unknowns

### Coverage gaps

- native/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- native/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)

### Runtime blind spots

- native: the allocator shim does not record which entry points ran

### Entry points not exercisable

None recorded.

### Evidence limitations and unknowns

- E5: partial coverage
- E5: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E5: limitation: writers and removers are textual name matches
- E6: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E6: limitation: workloads were discovered by reading files; none was run
- E7: limitation: harness written; nothing was run (dla harness run measures it)
- E7: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E8: partial coverage (9 of 9 examined)
- E8: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: exercised entry points are not measured for native: the allocator shim sees allocations, not calls
- E9: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: gate thresholds are provisional (not calibrated)
- E10: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: points 1 to 9 come from S1; the extra points from S2
- E11: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E11: limitation: gate and plateau thresholds are provisional (not calibrated)
- E12: partial coverage (1 of 3 examined)
- E12: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E12: limitation: threshold is provisional (not calibrated)
- E13: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: points 1 to 9 come from S1; the extra points from S2
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: points 1 to 9 come from S1; the extra points from S2
- E16: limitation: gate thresholds are provisional (not calibrated)

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
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | — |
| E2 | run-config | static | dla 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | probe/probe.json |
| E4 | static-hit | static | dla 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | sweep/native-dla.json |
| E5 | census-holder | static | dla-census 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | census/native.json |
| E6 | workload | static | dla 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | workload/workloads.json |
| E7 | workload | static | dla 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | harness/H1/harness.json |
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (a5e905bc0736) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (a83aaf92ba49) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (a83aaf92ba49) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (a72d7a00aba4) | alloc/A1.json |
| E12 | residual | test | dla 0.1.0 | artifact series/S1.json (a83aaf92ba49) | gap/residual-S1-E12.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S2.json (4723a2eda687) | trend/T3.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (4723a2eda687) | trend/T3.json |
| E15 | growth-series | test | dla 0.1.0 | artifact series/S3.json (a72d7a00aba4) | trend/T4.json |
| E16 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (a72d7a00aba4) | trend/T4.json |
