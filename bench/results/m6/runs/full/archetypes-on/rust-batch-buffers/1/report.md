# DLA report

Run: `20261110-201554-7cab`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 2 enumerated, 2 dispositioned; confirmed 0, strong 0, refuted 2, suspected 0, unresolved 0

Residual: not measured

Coverage: rust 880 iterations, 0 of 10 entry points exercised

## Findings

No findings to report.

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (non-leak) E16
  - Contradicting evidence: E16
  - Originally supported by: E5
- **F2** (UNCLASSIFIED): discriminator E17
  - Ruled out by: plateau+purge (non-leak) E17
  - Contradicting evidence: E17
  - Originally supported by: E5

## Blind spots and unknowns

### Coverage gaps

- rust/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- rust/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)
- rust/rapx: blocked (cannot copy the project: Permission denied (os error 13))

### Runtime blind spots

- rust: the allocator shim does not record which entry points ran

### Entry points not exercisable

- rust: `src/sample.rs::SampleError.fmt`: no_harness

### Evidence limitations and unknowns

- E5: partial coverage
- E5: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E5: limitation: writers and removers are textual name matches
- E6: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E6: limitation: workloads were discovered by reading files; none was run
- E7: limitation: harness written; nothing was run (dla harness run measures it)
- E7: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E8: partial coverage (0 of 9 examined)
- E8: limitation: heaptrack slows the program down; points 1, 5, 9 of 9 ran under it and no series was written (dla harness run without --tool measures the series)
- E9: partial coverage (9 of 9 examined)
- E9: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: exercised entry points are not measured for rust: the allocator shim sees allocations, not calls
- E10: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: .dla/20261110-201554-7cab/harness/H1/out/p1-native/heaptrack.txt elides 1 group(s) of stacks ("and N from M other places"): their bytes are not read, so a growing site can be missing; print with -m0 -s 50
- E12: limitation: .dla/20261110-201554-7cab/harness/H1/out/p5-native/heaptrack.txt elides 1 group(s) of stacks ("and N from M other places"): their bytes are not read, so a growing site can be missing; print with -m0 -s 50
- E12: limitation: .dla/20261110-201554-7cab/harness/H1/out/p9-native/heaptrack.txt elides 1 group(s) of stacks ("and N from M other places"): their bytes are not read, so a growing site can be missing; print with -m0 -s 50
- E12: limitation: figures are the bytes heaptrack counts as leaked: still allocated when the program exits
- E12: limitation: heaptrack_print rounds sizes (3.02M): figures are accurate to about 1%; -F folded files carry exact bytes
- E13: limitation: .dla/20261110-201554-7cab/harness/H1/out/p1-native/heaptrack.txt elides 1 group(s) of stacks ("and N from M other places"): their bytes are not read, so a growing site can be missing; print with -m0 -s 50
- E13: limitation: .dla/20261110-201554-7cab/harness/H1/out/p5-native/heaptrack.txt elides 1 group(s) of stacks ("and N from M other places"): their bytes are not read, so a growing site can be missing; print with -m0 -s 50
- E13: limitation: .dla/20261110-201554-7cab/harness/H1/out/p9-native/heaptrack.txt elides 1 group(s) of stacks ("and N from M other places"): their bytes are not read, so a growing site can be missing; print with -m0 -s 50
- E13: limitation: figures are the bytes heaptrack counts as leaked: still allocated when the program exits
- E13: limitation: heaptrack_print rounds sizes (3.02M): figures are accurate to about 1%; -F folded files carry exact bytes
- E13: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E14: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E14: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E15: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: points 1 to 9 come from S1; the extra points from S3
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: points 1 to 9 come from S1; the extra points from S5
- E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E17: limitation: gate and plateau thresholds are provisional (not calibrated)
- E18: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: gate thresholds are provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo 77a18cd734c529473e4421f614485d13cba99d33 | — |
| E2 | run-config | static | dla 0.1.0 | repo 77a18cd734c529473e4421f614485d13cba99d33 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 77a18cd734c529473e4421f614485d13cba99d33 | probe/probe.json |
| E4 | static-hit | static | dla 0.1.0 | repo 77a18cd734c529473e4421f614485d13cba99d33 | sweep/rust-dla.json |
| E5 | census-holder | static | dla-census 0.1.0 | repo 77a18cd734c529473e4421f614485d13cba99d33 | census/rust.json |
| E6 | workload | static | dla 0.1.0 | repo 77a18cd734c529473e4421f614485d13cba99d33 | workload/workloads.json |
| E7 | workload | static | dla 0.1.0 | repo 77a18cd734c529473e4421f614485d13cba99d33 | harness/H1/harness.json |
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (27d9ca9b81fa) | harness/H1/run-heaptrack.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (27d9ca9b81fa) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (c35baf544976) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (c35baf544976) | trend/T1.json |
| E12 | alloc-stack | live | heaptrack 0.1.0 | artifact allocs/A1.json (ed72d92c9110) | allocs/A1.json |
| E13 | growth-series | live | heaptrack 0.1.0 | artifact series/S2.json (638f76b75360) | series/S2.json |
| E14 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (638f76b75360) | trend/T2.json |
| E15 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (638f76b75360) | trend/T2.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S4.json (c272ebed5a87) | alloc/A1.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S6.json (ffbfd2244691) | alloc/A2.json |
| E18 | growth-series | test | dla 0.1.0 | artifact series/S3.json (54118721bb92) | trend/T5.json |
| E19 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (54118721bb92) | trend/T5.json |
| E20 | growth-series | test | dla 0.1.0 | artifact series/S5.json (afbe6bf4e747) | trend/T6.json |
| E21 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (afbe6bf4e747) | trend/T6.json |
