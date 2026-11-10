# DLA report

Run: `20261110-164031-33f8`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 1 enumerated, 1 dispositioned; confirmed 0, strong 0, refuted 1, suspected 0, unresolved 0

Residual: heap 0 bytes/n unattributed of 0 total (threshold 0)

Coverage: rust 1760 iterations, 0 of 10 entry points exercised

## Findings

No findings to report.

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (non-leak) E12
  - Contradicting evidence: E12
  - Originally supported by: E6

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

- E6: partial coverage
- E6: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E6: limitation: writers and removers are textual name matches
- E7: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E7: limitation: workloads were discovered by reading files; none was run
- E8: limitation: harness written; nothing was run (dla harness run measures it)
- E8: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E9: partial coverage (9 of 9 examined)
- E9: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: exercised entry points are not measured for rust: the allocator shim sees allocations, not calls
- E10: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: points 1 to 9 come from S1; the extra points from S2
- E12: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E12: limitation: gate and plateau thresholds are provisional (not calibrated)
- E13: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: partial coverage (1 of 3 examined)
- E15: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E15: limitation: threshold is provisional (not calibrated)

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
| E4 | probe | static | dla-probe 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | probe/probe.json |
| E5 | static-hit | static | dla 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | sweep/rust-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | census/rust.json |
| E7 | workload | static | dla 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (f963f64557e4) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (9b9c67016ebe) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (9b9c67016ebe) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (82a453acfc35) | alloc/A1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S2.json (3016764c90d8) | trend/T3.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (3016764c90d8) | trend/T3.json |
| E15 | residual | test | dla 0.1.0 | artifact series/S3.json (82a453acfc35) | gap/residual-S3-E15.json |
