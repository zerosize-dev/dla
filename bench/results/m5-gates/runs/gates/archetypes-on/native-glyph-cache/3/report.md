# DLA report

Run: `20261109-204506-2e1e`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 13 enumerated, 13 dispositioned; confirmed 0, strong 0, refuted 13, suspected 0, unresolved 0

Residual: not measured

Coverage: native 7040 iterations, 0 of 7 entry points exercised

## Findings

No findings to report.

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (non-leak) E12
  - Contradicting evidence: E12
  - Originally supported by: E5
- **F2** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (non-leak) E13
  - Contradicting evidence: E13
  - Originally supported by: E5
- **F3** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (non-leak) E14
  - Contradicting evidence: E14
  - Originally supported by: E5
- **F4** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (non-leak) E15
  - Contradicting evidence: E15
  - Originally supported by: E5
- **F5** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (non-leak) E16
  - Contradicting evidence: E16
  - Originally supported by: E5
- **F6** (UNCLASSIFIED): discriminator E17
  - Ruled out by: plateau+purge (non-leak) E17
  - Contradicting evidence: E17
  - Originally supported by: E5
- **F7** (UNCLASSIFIED): discriminator E18
  - Ruled out by: plateau+purge (non-leak) E18
  - Contradicting evidence: E18
  - Originally supported by: E5
- **F8** (UNCLASSIFIED): discriminator E19
  - Ruled out by: plateau+purge (non-leak) E19
  - Contradicting evidence: E19
  - Originally supported by: E5
- **F9** (UNCLASSIFIED): discriminator E20
  - Ruled out by: plateau+purge (non-leak) E20
  - Contradicting evidence: E20
  - Originally supported by: E5
- **F10** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (non-leak) E11
  - Contradicting evidence: E11
  - Originally supported by: E5
- **F11** (UNCLASSIFIED): discriminator E21
  - Ruled out by: plateau+purge (non-leak) E21
  - Contradicting evidence: E21
  - Originally supported by: E5
- **F12** (UNCLASSIFIED): discriminator E22
  - Ruled out by: plateau+purge (non-leak) E22
  - Contradicting evidence: E22
  - Originally supported by: E5
- **F13** (UNCLASSIFIED): discriminator E23
  - Ruled out by: plateau+purge (non-leak) E23
  - Contradicting evidence: E23
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
- E8: partial coverage (11 of 11 examined)
- E8: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: exercised entry points are not measured for native: the allocator shim sees allocations, not calls
- E9: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: gate thresholds are provisional (not calibrated)
- E10: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E11: limitation: gate and plateau thresholds are provisional (not calibrated)
- E12: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E12: limitation: gate and plateau thresholds are provisional (not calibrated)
- E13: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- E14: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E17: limitation: gate and plateau thresholds are provisional (not calibrated)
- E18: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E18: limitation: gate and plateau thresholds are provisional (not calibrated)
- E19: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E19: limitation: gate and plateau thresholds are provisional (not calibrated)
- E20: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E20: limitation: gate and plateau thresholds are provisional (not calibrated)
- E21: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E21: limitation: gate and plateau thresholds are provisional (not calibrated)
- E22: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E22: limitation: gate and plateau thresholds are provisional (not calibrated)
- E23: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E23: limitation: gate and plateau thresholds are provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | — |
| E2 | run-config | static | dla 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | probe/probe.json |
| E4 | static-hit | static | dla 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | sweep/native-dla.json |
| E5 | census-holder | static | dla-census 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | census/native.json |
| E6 | workload | static | dla 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | workload/workloads.json |
| E7 | workload | static | dla 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | harness/H1/harness.json |
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (87d66beb343b) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (c41b84c40b31) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (c41b84c40b31) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S2.json (d861699c423b) | alloc/A1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (0060081c88d8) | alloc/A2.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S4.json (b9f4e981c7c9) | alloc/A3.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S5.json (011c396be817) | alloc/A4.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S6.json (8912f26ed5db) | alloc/A5.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S7.json (8a5b714fb2db) | alloc/A6.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S8.json (5544b2519887) | alloc/A7.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S9.json (708fb6a431dd) | alloc/A8.json |
| E19 | discriminator | test | dla 0.1.0 | artifact series/S10.json (370fe8243f35) | alloc/A9.json |
| E20 | discriminator | test | dla 0.1.0 | artifact series/S11.json (02fe8ab08fee) | alloc/A10.json |
| E21 | discriminator | test | dla 0.1.0 | artifact series/S12.json (993e93e14751) | alloc/A11.json |
| E22 | discriminator | test | dla 0.1.0 | artifact series/S13.json (c12287585739) | alloc/A12.json |
| E23 | discriminator | test | dla 0.1.0 | artifact series/S14.json (784b036772b2) | alloc/A13.json |
