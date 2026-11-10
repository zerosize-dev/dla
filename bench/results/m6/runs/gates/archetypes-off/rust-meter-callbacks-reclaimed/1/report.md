# DLA report

Run: `20261110-182721-f8c8`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 2 enumerated, 2 dispositioned; confirmed 0, strong 0, refuted 2, suspected 0, unresolved 0

Residual: heap 0 bytes/n unattributed of 0 total (threshold 0)

Coverage: rust 1760 iterations, 0 of 14 entry points exercised

## Findings

No findings to report.

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (non-leak) E12
  - Contradicting evidence: E12
  - Originally supported by: E5
- **F2** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (non-leak) E11
  - Contradicting evidence: E11
  - Originally supported by: E5

## Blind spots and unknowns

### Coverage gaps

- rust/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- rust/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)
- rust/rapx: blocked (cannot copy the project: Permission denied (os error 13))

### Runtime blind spots

- rust: the allocator shim does not record which entry points ran

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
- E13: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: partial coverage (1 of 3 examined)
- E17: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E17: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo cfd0eb004e9f853ec14007ff5c3ff62e608d22f3 | — |
| E2 | run-config | static | dla 0.1.0 | repo cfd0eb004e9f853ec14007ff5c3ff62e608d22f3 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo cfd0eb004e9f853ec14007ff5c3ff62e608d22f3 | probe/probe.json |
| E4 | static-hit | static | dla 0.1.0 | repo cfd0eb004e9f853ec14007ff5c3ff62e608d22f3 | sweep/rust-dla.json |
| E5 | census-holder | static | dla-census 0.1.0 | repo cfd0eb004e9f853ec14007ff5c3ff62e608d22f3 | census/rust.json |
| E6 | workload | static | dla 0.1.0 | repo cfd0eb004e9f853ec14007ff5c3ff62e608d22f3 | workload/workloads.json |
| E7 | workload | static | dla 0.1.0 | repo cfd0eb004e9f853ec14007ff5c3ff62e608d22f3 | harness/H1/harness.json |
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (351a2f48cd04) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (c1e85bf7dccc) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (c1e85bf7dccc) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (c8728d9d5a2a) | alloc/A1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S5.json (e2ea89d5e9f7) | alloc/A2.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S2.json (922a9f9553a1) | trend/T4.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (922a9f9553a1) | trend/T4.json |
| E15 | growth-series | test | dla 0.1.0 | artifact series/S4.json (7710d1b8e81b) | trend/T5.json |
| E16 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (7710d1b8e81b) | trend/T5.json |
| E17 | residual | test | dla 0.1.0 | artifact series/S1.json (c1e85bf7dccc) | gap/residual-S1-E17.json |
