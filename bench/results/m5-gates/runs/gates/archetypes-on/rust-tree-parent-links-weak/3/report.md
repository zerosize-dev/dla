# DLA report

Run: `20261109-205202-7add`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 1 enumerated, 1 dispositioned; confirmed 0, strong 0, refuted 1, suspected 0, unresolved 0

Residual: not measured

Coverage: rust 1760 iterations, 0 of 10 entry points exercised

## Findings

No findings to report.

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (non-leak) E12
  - Contradicting evidence: E12
  - Originally supported by: E5

## Blind spots and unknowns

### Coverage gaps

- rust/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- rust/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)
- rust/rapx: blocked (cannot copy the project: Permission denied (os error 13))

### Runtime blind spots

- rust: the allocator shim does not record which entry points ran

### Entry points not exercisable

- rust: `src/parser.rs::ParseError.fmt`: no_harness

### Evidence limitations and unknowns

- E5: partial coverage
- E5: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E5: limitation: writers and removers are textual name matches
- E6: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E6: limitation: workloads were discovered by reading files; none was run
- E7: limitation: harness written; nothing was run (dla harness run measures it)
- E7: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E8: partial coverage (9 of 9 examined)
- E8: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: exercised entry points are not measured for rust: the allocator shim sees allocations, not calls
- E9: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: gate thresholds are provisional (not calibrated)
- E10: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: points 1 to 9 come from S1; the extra points from S2
- E11: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E11: limitation: gate and plateau thresholds are provisional (not calibrated)
- E12: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: points 1 to 9 come from S1; the extra points from S4
- E12: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E12: limitation: gate and plateau thresholds are provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | — |
| E2 | run-config | static | dla 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | probe/probe.json |
| E4 | static-hit | static | dla 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | sweep/rust-dla.json |
| E5 | census-holder | static | dla-census 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | census/rust.json |
| E6 | workload | static | dla 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | workload/workloads.json |
| E7 | workload | static | dla 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | harness/H1/harness.json |
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (a7742008b70f) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (c50315f7ae58) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (c50315f7ae58) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (b4bbfc2b472b) | alloc/A1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S5.json (1088357b70fe) | alloc/A2.json |
