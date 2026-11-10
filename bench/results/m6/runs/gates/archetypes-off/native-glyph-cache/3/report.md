# DLA report

Run: `20261110-193608-bae1`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 15 enumerated, 15 dispositioned; confirmed 1, strong 0, refuted 14, suspected 0, unresolved 0

Residual: heap 0 bytes/n unattributed of 0 total (threshold 0); heap 0 bytes/n unattributed of 0 total (threshold 0); heap 0 bytes/n unattributed of 0 total (threshold 0); heap 0 bytes/n unattributed of 0 total (threshold 0); heap 0 bytes/n unattributed of 0 total (threshold 0); heap 0 bytes/n unattributed of 0 total (threshold 0); heap 0 bytes/n unattributed of 0 total (threshold 0); heap 0 bytes/n unattributed of 0 total (threshold 0); heap 0 bytes/n unattributed of 0 total (threshold 0); heap 0 bytes/n unattributed of 0 total (threshold 0); heap 0 bytes/n unattributed of 0 total (threshold 0); heap 0 bytes/n unattributed of 0 total (threshold 0); heap 0 bytes/n unattributed of 0 total (threshold 0); heap 0 bytes/n unattributed of 0 total (threshold 0); heap 0 bytes/n unattributed of 0 total (threshold 0)

Coverage: native 5280 iterations, 0 of 7 entry points exercised

## Findings

### F14.n1: Hypothesis N-WARM (UNCLASSIFIED)

- State: confirmed
- Explained by N-WARM (discriminator E11)
- Evidence classes: context, discriminator
- Sibling hypothesis of: F14
- Hypothesis: N-WARM
- Refuted if: growth continues over the extra cycles
- Supporting evidence: E11, E8

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
- **F10** (UNCLASSIFIED): discriminator E21
  - Ruled out by: plateau+purge (non-leak) E21
  - Contradicting evidence: E21
  - Originally supported by: E5
- **F11** (UNCLASSIFIED): discriminator E22
  - Ruled out by: plateau+purge (non-leak) E22
  - Contradicting evidence: E22
  - Originally supported by: E5
- **F12** (UNCLASSIFIED): discriminator E23
  - Ruled out by: plateau+purge (non-leak) E23
  - Contradicting evidence: E23
  - Originally supported by: E5
- **F13** (UNCLASSIFIED): discriminator E24
  - Ruled out by: plateau+purge (non-leak) E24
  - Contradicting evidence: E24
  - Originally supported by: E5
- **F14** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (non-leak) E11
  - Contradicting evidence: E11
  - Originally supported by: E9

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
- E8: partial coverage (12 of 12 examined)
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
- E24: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E24: limitation: gate and plateau thresholds are provisional (not calibrated)
- E25: partial coverage (1 of 3 examined)
- E25: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E25: limitation: threshold is provisional (not calibrated)
- E26: partial coverage (1 of 3 examined)
- E26: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E26: limitation: threshold is provisional (not calibrated)
- E27: partial coverage (1 of 3 examined)
- E27: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E27: limitation: threshold is provisional (not calibrated)
- E28: partial coverage (1 of 3 examined)
- E28: limitation: off-heap: no gap series for S10; run dla gap S10 first (node.external is used when present)
- E28: limitation: threshold is provisional (not calibrated)
- E29: partial coverage (1 of 3 examined)
- E29: limitation: off-heap: no gap series for S11; run dla gap S11 first (node.external is used when present)
- E29: limitation: threshold is provisional (not calibrated)
- E30: partial coverage (1 of 3 examined)
- E30: limitation: off-heap: no gap series for S12; run dla gap S12 first (node.external is used when present)
- E30: limitation: threshold is provisional (not calibrated)
- E31: partial coverage (1 of 3 examined)
- E31: limitation: off-heap: no gap series for S13; run dla gap S13 first (node.external is used when present)
- E31: limitation: threshold is provisional (not calibrated)
- E32: partial coverage (1 of 3 examined)
- E32: limitation: off-heap: no gap series for S14; run dla gap S14 first (node.external is used when present)
- E32: limitation: threshold is provisional (not calibrated)
- E33: partial coverage (1 of 3 examined)
- E33: limitation: off-heap: no gap series for S15; run dla gap S15 first (node.external is used when present)
- E33: limitation: threshold is provisional (not calibrated)
- E34: partial coverage (1 of 3 examined)
- E34: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E34: limitation: threshold is provisional (not calibrated)
- E35: partial coverage (1 of 3 examined)
- E35: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E35: limitation: threshold is provisional (not calibrated)
- E36: partial coverage (1 of 3 examined)
- E36: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E36: limitation: threshold is provisional (not calibrated)
- E37: partial coverage (1 of 3 examined)
- E37: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E37: limitation: threshold is provisional (not calibrated)
- E38: partial coverage (1 of 3 examined)
- E38: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E38: limitation: threshold is provisional (not calibrated)
- E39: partial coverage (1 of 3 examined)
- E39: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E39: limitation: threshold is provisional (not calibrated)
- E40: partial coverage (1 of 3 examined)
- E40: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E40: limitation: threshold is provisional (not calibrated)
- E41: partial coverage (1 of 3 examined)
- E41: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E41: limitation: threshold is provisional (not calibrated)

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
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (02e24862541d) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (3230fd99730b) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (3230fd99730b) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S2.json (c45c4439b6ad) | alloc/A1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (22d6115b9966) | alloc/A2.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S4.json (20f9e020a946) | alloc/A3.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S5.json (21b311e7241b) | alloc/A4.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S6.json (1320bb9bd3f1) | alloc/A5.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S7.json (5927ec0279d9) | alloc/A6.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S8.json (12da6487b52e) | alloc/A7.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S9.json (8c4e98817b1c) | alloc/A8.json |
| E19 | discriminator | test | dla 0.1.0 | artifact series/S10.json (44bc272e8a24) | alloc/A9.json |
| E20 | discriminator | test | dla 0.1.0 | artifact series/S11.json (5b48d2ae5cd4) | alloc/A10.json |
| E21 | discriminator | test | dla 0.1.0 | artifact series/S12.json (69a9263c6fd4) | alloc/A11.json |
| E22 | discriminator | test | dla 0.1.0 | artifact series/S13.json (b00f99c629e6) | alloc/A12.json |
| E23 | discriminator | test | dla 0.1.0 | artifact series/S14.json (3e00af196698) | alloc/A13.json |
| E24 | discriminator | test | dla 0.1.0 | artifact series/S15.json (4c9da901ff5f) | alloc/A14.json |
| E25 | residual | test | dla 0.1.0 | artifact series/S1.json (3230fd99730b) | gap/residual-S1-E25.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S2.json (c45c4439b6ad) | gap/residual-S2-E26.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S1.json (3230fd99730b) | gap/residual-S1-E27.json |
| E28 | residual | test | dla 0.1.0 | artifact series/S10.json (44bc272e8a24) | gap/residual-S10-E28.json |
| E29 | residual | test | dla 0.1.0 | artifact series/S11.json (5b48d2ae5cd4) | gap/residual-S11-E29.json |
| E30 | residual | test | dla 0.1.0 | artifact series/S12.json (69a9263c6fd4) | gap/residual-S12-E30.json |
| E31 | residual | test | dla 0.1.0 | artifact series/S13.json (b00f99c629e6) | gap/residual-S13-E31.json |
| E32 | residual | test | dla 0.1.0 | artifact series/S14.json (3e00af196698) | gap/residual-S14-E32.json |
| E33 | residual | test | dla 0.1.0 | artifact series/S15.json (4c9da901ff5f) | gap/residual-S15-E33.json |
| E34 | residual | test | dla 0.1.0 | artifact series/S2.json (c45c4439b6ad) | gap/residual-S2-E34.json |
| E35 | residual | test | dla 0.1.0 | artifact series/S3.json (22d6115b9966) | gap/residual-S3-E35.json |
| E36 | residual | test | dla 0.1.0 | artifact series/S4.json (20f9e020a946) | gap/residual-S4-E36.json |
| E37 | residual | test | dla 0.1.0 | artifact series/S5.json (21b311e7241b) | gap/residual-S5-E37.json |
| E38 | residual | test | dla 0.1.0 | artifact series/S6.json (1320bb9bd3f1) | gap/residual-S6-E38.json |
| E39 | residual | test | dla 0.1.0 | artifact series/S7.json (5927ec0279d9) | gap/residual-S7-E39.json |
| E40 | residual | test | dla 0.1.0 | artifact series/S8.json (12da6487b52e) | gap/residual-S8-E40.json |
| E41 | residual | test | dla 0.1.0 | artifact series/S9.json (8c4e98817b1c) | gap/residual-S9-E41.json |
