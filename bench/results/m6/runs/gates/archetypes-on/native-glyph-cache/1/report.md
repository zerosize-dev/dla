# DLA report

Run: `20261110-162840-428f`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 13 enumerated, 13 dispositioned; confirmed 0, strong 0, refuted 13, suspected 0, unresolved 0

Residual: heap 0 bytes/n unattributed of 0 total (threshold 0); heap 0 bytes/n unattributed of 0 total (threshold 0); off-heap 7.868398268398268 bytes/n unattributed of 7.868398268398268 total (threshold 2.288540259739907) ABOVE THRESHOLD, see O2

Coverage: native 3520 iterations, 0 of 7 entry points exercised

## Findings

No findings to report.

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (non-leak) E15
  - Contradicting evidence: E15
  - Originally supported by: E5
- **F2** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (non-leak) E16
  - Contradicting evidence: E16
  - Originally supported by: E5
- **F3** (UNCLASSIFIED): discriminator E17
  - Ruled out by: plateau+purge (non-leak) E17
  - Contradicting evidence: E17
  - Originally supported by: E5
- **F4** (UNCLASSIFIED): discriminator E18
  - Ruled out by: plateau+purge (non-leak) E18
  - Contradicting evidence: E18
  - Originally supported by: E5
- **F5** (UNCLASSIFIED): discriminator E19
  - Ruled out by: plateau+purge (non-leak) E19
  - Contradicting evidence: E19
  - Originally supported by: E5
- **F6** (UNCLASSIFIED): discriminator E20
  - Ruled out by: plateau+purge (non-leak) E20
  - Contradicting evidence: E20
  - Originally supported by: E5
- **F7** (UNCLASSIFIED): discriminator E21
  - Ruled out by: plateau+purge (non-leak) E21
  - Contradicting evidence: E21
  - Originally supported by: E5
- **F8** (UNCLASSIFIED): discriminator E22
  - Ruled out by: plateau+purge (non-leak) E22
  - Contradicting evidence: E22
  - Originally supported by: E5
- **F9** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (non-leak) E12
  - Contradicting evidence: E12
  - Originally supported by: E5
- **F10** (UNCLASSIFIED): discriminator E23
  - Ruled out by: plateau+purge (non-leak) E23
  - Contradicting evidence: E23
  - Originally supported by: E5
- **F11** (UNCLASSIFIED): discriminator E24
  - Ruled out by: plateau+purge (non-leak) E24
  - Contradicting evidence: E24
  - Originally supported by: E5
- **F12** (UNCLASSIFIED): discriminator E25
  - Ruled out by: plateau+purge (non-leak) E25
  - Contradicting evidence: E25
  - Originally supported by: E5
- **F13** (UNCLASSIFIED): discriminator E26
  - Ruled out by: plateau+purge (non-leak) E26
  - Contradicting evidence: E26
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
- E11: partial coverage (1 of 3 examined)
- E11: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E11: limitation: threshold is provisional (not calibrated)
- E12: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: points 1 to 9 come from S1; the extra points from S2
- E12: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E12: limitation: gate and plateau thresholds are provisional (not calibrated)
- E13: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: points 1 to 9 come from S1; the extra points from S4
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: points 1 to 9 come from S1; the extra points from S6
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: points 1 to 9 come from S1; the extra points from S8
- E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E17: limitation: gate and plateau thresholds are provisional (not calibrated)
- E18: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: points 1 to 9 come from S1; the extra points from S10
- E18: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E18: limitation: gate and plateau thresholds are provisional (not calibrated)
- E19: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: points 1 to 9 come from S1; the extra points from S12
- E19: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E19: limitation: gate and plateau thresholds are provisional (not calibrated)
- E20: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: points 1 to 9 come from S1; the extra points from S14
- E20: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E20: limitation: gate and plateau thresholds are provisional (not calibrated)
- E21: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: points 1 to 9 come from S1; the extra points from S16
- E21: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E21: limitation: gate and plateau thresholds are provisional (not calibrated)
- E22: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: points 1 to 9 come from S1; the extra points from S18
- E22: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E22: limitation: gate and plateau thresholds are provisional (not calibrated)
- E23: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: points 1 to 9 come from S1; the extra points from S20
- E23: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E23: limitation: gate and plateau thresholds are provisional (not calibrated)
- E24: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: points 1 to 9 come from S1; the extra points from S22
- E24: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E24: limitation: gate and plateau thresholds are provisional (not calibrated)
- E25: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E25: limitation: points 1 to 9 come from S1; the extra points from S24
- E25: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E25: limitation: gate and plateau thresholds are provisional (not calibrated)
- E26: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E26: limitation: points 1 to 9 come from S1; the extra points from S26
- E26: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E26: limitation: gate and plateau thresholds are provisional (not calibrated)
- E27: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E29: limitation: gate thresholds are provisional (not calibrated)
- E30: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E30: limitation: gate thresholds are provisional (not calibrated)
- E31: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E31: limitation: gate thresholds are provisional (not calibrated)
- E32: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E32: limitation: gate thresholds are provisional (not calibrated)
- E33: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E33: limitation: gate thresholds are provisional (not calibrated)
- E34: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E34: limitation: gate thresholds are provisional (not calibrated)
- E35: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E35: limitation: gate thresholds are provisional (not calibrated)
- E36: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E36: limitation: gate thresholds are provisional (not calibrated)
- E37: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E37: limitation: gate thresholds are provisional (not calibrated)
- E38: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E38: limitation: gate thresholds are provisional (not calibrated)
- E39: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E39: limitation: gate thresholds are provisional (not calibrated)
- E40: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E40: limitation: gate thresholds are provisional (not calibrated)
- E41: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E41: limitation: gate thresholds are provisional (not calibrated)
- E42: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E42: limitation: gate thresholds are provisional (not calibrated)
- E43: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E43: limitation: gate thresholds are provisional (not calibrated)
- E44: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E44: limitation: gate thresholds are provisional (not calibrated)
- E45: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E45: limitation: gate thresholds are provisional (not calibrated)
- E46: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E46: limitation: gate thresholds are provisional (not calibrated)
- E47: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E47: limitation: gate thresholds are provisional (not calibrated)
- E48: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E48: limitation: gate thresholds are provisional (not calibrated)
- E49: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E49: limitation: gate thresholds are provisional (not calibrated)
- E50: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E50: limitation: gate thresholds are provisional (not calibrated)
- E51: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E51: limitation: gate thresholds are provisional (not calibrated)
- E52: partial coverage (2 of 3 examined)
- E52: limitation: threshold is provisional (not calibrated)
- E53: partial coverage (2 of 3 examined)
- E53: limitation: threshold is provisional (not calibrated)

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
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (599adbad794d) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (502fc2bc96db) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (502fc2bc96db) | trend/T1.json |
| E11 | residual | test | dla 0.1.0 | artifact series/S1.json (502fc2bc96db) | gap/residual-S1-E11.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (177084a43817) | alloc/A1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S2.json (cf25bd9e5ba3) | trend/T3.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (cf25bd9e5ba3) | trend/T3.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S5.json (3d553657d341) | alloc/A2.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S7.json (f7db44614a5f) | alloc/A3.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S9.json (e464433a23c7) | alloc/A4.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S11.json (5741c559a727) | alloc/A5.json |
| E19 | discriminator | test | dla 0.1.0 | artifact series/S13.json (b7957bda894d) | alloc/A6.json |
| E20 | discriminator | test | dla 0.1.0 | artifact series/S15.json (bc924f1e17c2) | alloc/A7.json |
| E21 | discriminator | test | dla 0.1.0 | artifact series/S17.json (1aeb1ac8114a) | alloc/A8.json |
| E22 | discriminator | test | dla 0.1.0 | artifact series/S19.json (bfc946c3b35b) | alloc/A9.json |
| E23 | discriminator | test | dla 0.1.0 | artifact series/S21.json (15033335a51a) | alloc/A10.json |
| E24 | discriminator | test | dla 0.1.0 | artifact series/S23.json (54090356b69c) | alloc/A11.json |
| E25 | discriminator | test | dla 0.1.0 | artifact series/S25.json (fb59b69d66aa) | alloc/A12.json |
| E26 | discriminator | test | dla 0.1.0 | artifact series/S27.json (a90925269f28) | alloc/A13.json |
| E27 | growth-series | test | dla 0.1.0 | artifact series/S4.json (b4f2be811a00) | trend/T16.json |
| E28 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (b4f2be811a00) | trend/T16.json |
| E29 | growth-series | test | dla 0.1.0 | artifact series/S6.json (c825e42ab19a) | trend/T17.json |
| E30 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (c825e42ab19a) | trend/T17.json |
| E31 | growth-series | test | dla 0.1.0 | artifact series/S8.json (cef48943cceb) | trend/T18.json |
| E32 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (cef48943cceb) | trend/T18.json |
| E33 | growth-series | test | dla 0.1.0 | artifact series/S10.json (fe5216d3c8b2) | trend/T19.json |
| E34 | unit-count-series | test | dla 0.1.0 | artifact series/S10.json (fe5216d3c8b2) | trend/T19.json |
| E35 | growth-series | test | dla 0.1.0 | artifact series/S12.json (6533edd2616d) | trend/T20.json |
| E36 | unit-count-series | test | dla 0.1.0 | artifact series/S12.json (6533edd2616d) | trend/T20.json |
| E37 | growth-series | test | dla 0.1.0 | artifact series/S14.json (0a8f795be388) | trend/T21.json |
| E38 | unit-count-series | test | dla 0.1.0 | artifact series/S14.json (0a8f795be388) | trend/T21.json |
| E39 | growth-series | test | dla 0.1.0 | artifact series/S16.json (e2c759ef5087) | trend/T22.json |
| E40 | unit-count-series | test | dla 0.1.0 | artifact series/S16.json (e2c759ef5087) | trend/T22.json |
| E41 | growth-series | test | dla 0.1.0 | artifact series/S18.json (3e0c8cbdd261) | trend/T23.json |
| E42 | unit-count-series | test | dla 0.1.0 | artifact series/S18.json (3e0c8cbdd261) | trend/T23.json |
| E43 | growth-series | test | dla 0.1.0 | artifact series/S20.json (4508a38f398c) | trend/T24.json |
| E44 | unit-count-series | test | dla 0.1.0 | artifact series/S20.json (4508a38f398c) | trend/T24.json |
| E45 | growth-series | test | dla 0.1.0 | artifact series/S22.json (05543f1c79a0) | trend/T25.json |
| E46 | unit-count-series | test | dla 0.1.0 | artifact series/S22.json (05543f1c79a0) | trend/T25.json |
| E47 | growth-series | test | dla 0.1.0 | artifact series/S24.json (b7462c4082ee) | trend/T26.json |
| E48 | unit-count-series | test | dla 0.1.0 | artifact series/S24.json (b7462c4082ee) | trend/T26.json |
| E49 | growth-series | test | dla 0.1.0 | artifact series/S26.json (f33e1cd6ce06) | trend/T27.json |
| E50 | unit-count-series | test | dla 0.1.0 | artifact series/S26.json (f33e1cd6ce06) | trend/T27.json |
| E51 | non-heap | test | dla 0.1.0 | artifact series/S28.json (c72da0ac619b) | trend/T28.json |
| E52 | residual | test | dla 0.1.0 | artifact series/S14.json (0a8f795be388) | gap/residual-S14-E52.json |
| E53 | residual | test | dla 0.1.0 | artifact series/S14.json (0a8f795be388) | gap/residual-S14-E53.json |
