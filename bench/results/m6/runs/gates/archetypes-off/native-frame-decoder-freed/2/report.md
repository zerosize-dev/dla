# DLA report

Run: `20261110-185825-34cc`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 4 enumerated, 4 dispositioned; confirmed 0, strong 0, refuted 4, suspected 0, unresolved 0

Residual: not measured

Coverage: native 3520 iterations, 0 of 10 entry points exercised

## Findings

No findings to report.

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (non-leak) E14
  - Contradicting evidence: E14
  - Originally supported by: E5
- **F2** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (non-leak) E15
  - Contradicting evidence: E15
  - Originally supported by: E5
- **F3** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (non-leak) E11
  - Contradicting evidence: E11
  - Originally supported by: E5
- **F4** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (non-leak) E16
  - Contradicting evidence: E16
  - Originally supported by: E5

## Blind spots and unknowns

### Coverage gaps

- native/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- native/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)

### Runtime blind spots

- native: the allocator shim does not record which entry points ran

### Entry points not exercisable

- native: `src/decoder.c::frame_status_name`: no_harness

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
- E12: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: points 1 to 9 come from S1; the extra points from S4
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: points 1 to 9 come from S1; the extra points from S6
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: points 1 to 9 come from S1; the extra points from S8
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: gate thresholds are provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo 88516bb787e57991e562d9cccf930c89c6f91f3f | — |
| E2 | run-config | static | dla 0.1.0 | repo 88516bb787e57991e562d9cccf930c89c6f91f3f | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 88516bb787e57991e562d9cccf930c89c6f91f3f | probe/probe.json |
| E4 | static-hit | static | dla 0.1.0 | repo 88516bb787e57991e562d9cccf930c89c6f91f3f | sweep/native-dla.json |
| E5 | census-holder | static | dla-census 0.1.0 | repo 88516bb787e57991e562d9cccf930c89c6f91f3f | census/native.json |
| E6 | workload | static | dla 0.1.0 | repo 88516bb787e57991e562d9cccf930c89c6f91f3f | workload/workloads.json |
| E7 | workload | static | dla 0.1.0 | repo 88516bb787e57991e562d9cccf930c89c6f91f3f | harness/H1/harness.json |
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (7bb59cc71e59) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (393bd481a2f4) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (393bd481a2f4) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (a315e86cfc19) | alloc/A1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S2.json (220c0e09363a) | trend/T3.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (220c0e09363a) | trend/T3.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S5.json (d5218750f583) | alloc/A2.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S7.json (5570ead3d567) | alloc/A3.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S9.json (41a0a5363ed1) | alloc/A4.json |
| E17 | growth-series | test | dla 0.1.0 | artifact series/S4.json (c0002dadcded) | trend/T7.json |
| E18 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (c0002dadcded) | trend/T7.json |
| E19 | growth-series | test | dla 0.1.0 | artifact series/S6.json (551340d77a9c) | trend/T8.json |
| E20 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (551340d77a9c) | trend/T8.json |
| E21 | growth-series | test | dla 0.1.0 | artifact series/S8.json (3370267b638d) | trend/T9.json |
| E22 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (3370267b638d) | trend/T9.json |
