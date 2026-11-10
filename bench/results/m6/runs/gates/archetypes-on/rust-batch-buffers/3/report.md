# DLA report

Run: `20261110-173439-b942`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 4 enumerated, 4 dispositioned; confirmed 2, strong 0, refuted 2, suspected 0, unresolved 0

Residual: heap 0 bytes/n unattributed of 0 total (threshold 0); heap 0 bytes/n unattributed of 0 total (threshold 0); heap 0 bytes/n unattributed of 0 total (threshold 0); heap 0 bytes/n unattributed of 0 total (threshold 0)

Coverage: rust 880 iterations, 0 of 10 entry points exercised

## Findings

### F1.n1: Hypothesis N-HWM: Vec capacity retained after clear() from the 60,000-sample burst in batch 11; bounded high-water mark, not growth (UNCLASSIFIED)

- State: confirmed
- Explained by N-HWM (discriminator E11)
- Evidence classes: context, discriminator
- Sibling hypothesis of: F1
- Hypothesis: N-HWM: Vec capacity retained after clear() from the 60,000-sample burst in batch 11; bounded high-water mark, not growth
- Refuted if: growth continues over the extra cycles
- Supporting evidence: E11, E8

### F2.n1: Hypothesis N-HWM: HashMap bucket capacity retained after clear() from the 60,000-sample burst in batch 11; bounded high-water mark (UNCLASSIFIED)

- State: confirmed
- Explained by N-HWM (discriminator E14)
- Evidence classes: context, discriminator
- Sibling hypothesis of: F2
- Hypothesis: N-HWM: HashMap bucket capacity retained after clear() from the 60,000-sample burst in batch 11; bounded high-water mark
- Refuted if: growth continues over the extra cycles
- Supporting evidence: E14, E8

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (non-leak) E11
  - Contradicting evidence: E11
  - Originally supported by: E5
- **F2** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (non-leak) E14
  - Contradicting evidence: E14
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
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: points 1 to 9 come from S1; the extra points from S4
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: partial coverage (1 of 3 examined)
- E17: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E17: limitation: threshold is provisional (not calibrated)
- E18: partial coverage (1 of 3 examined)
- E18: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E18: limitation: threshold is provisional (not calibrated)
- E19: partial coverage (1 of 3 examined)
- E19: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E19: limitation: threshold is provisional (not calibrated)
- E20: partial coverage (1 of 3 examined)
- E20: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E20: limitation: threshold is provisional (not calibrated)

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
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (489221914878) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (935eebe530f3) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (935eebe530f3) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (eabcfb4dc503) | alloc/A1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S2.json (b4685c8762f5) | trend/T3.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (b4685c8762f5) | trend/T3.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S5.json (5a8e17a52b85) | alloc/A2.json |
| E15 | growth-series | test | dla 0.1.0 | artifact series/S4.json (6fba062d56eb) | trend/T5.json |
| E16 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (6fba062d56eb) | trend/T5.json |
| E17 | residual | test | dla 0.1.0 | artifact series/S1.json (935eebe530f3) | gap/residual-S1-E17.json |
| E18 | residual | test | dla 0.1.0 | artifact series/S3.json (eabcfb4dc503) | gap/residual-S3-E18.json |
| E19 | residual | test | dla 0.1.0 | artifact series/S4.json (6fba062d56eb) | gap/residual-S4-E19.json |
| E20 | residual | test | dla 0.1.0 | artifact series/S5.json (5a8e17a52b85) | gap/residual-S5-E20.json |
