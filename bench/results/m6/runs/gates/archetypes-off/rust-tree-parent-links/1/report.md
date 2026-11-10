# DLA report

Run: `20261110-183152-85c0`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 8 enumerated, 8 dispositioned; confirmed 4, strong 0, refuted 4, suspected 0, unresolved 0

Residual: heap 5268.498106060606 bytes/n unattributed of 5268.498106060606 total (threshold 1317.1245265151515) ABOVE THRESHOLD, see O1; heap 0 bytes/n unattributed of 5262.361363636363 total (threshold 1315.5903409090909); heap 5260.929331779332 bytes/n unattributed of 5260.929331779332 total (threshold 1315.232332944833) ABOVE THRESHOLD, see O2; heap 5262.361363636363 bytes/n unattributed of 5262.361363636363 total (threshold 1315.5903409090909) ABOVE THRESHOLD, see O2; heap 5260.929331779332 bytes/n unattributed of 5260.929331779332 total (threshold 1315.232332944833) ABOVE THRESHOLD, see O2; heap 5262.361363636363 bytes/n unattributed of 5262.361363636363 total (threshold 1315.5903409090909) ABOVE THRESHOLD, see O2; heap 5260.929331779332 bytes/n unattributed of 5260.929331779332 total (threshold 1315.232332944833) ABOVE THRESHOLD, see O2; heap 5262.361363636363 bytes/n unattributed of 5262.361363636363 total (threshold 1315.5903409090909) ABOVE THRESHOLD, see O2; heap 5260.929331779332 bytes/n unattributed of 5260.929331779332 total (threshold 1315.232332944833) ABOVE THRESHOLD, see O3

Coverage: rust 1760 iterations, 0 of 10 entry points exercised

## Findings

### F1: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Rc reference cycle between Section.parent and Section.children; no Weak back-link, trees never freed)
- Growth: `native.live_bytes@src/section.rs:16` 4213 bytes/n (steady, gate pass, E12)
- Evidence classes: context, discriminator, growth, static
- Holder: cycle-only
- Region: heap
- Shape: steady
- Site: `src/section.rs:10`
- Writers: `src/section.rs:27`
- Verified: no
- Sibling hypotheses: F1.n1
- Supporting evidence: E4, E8, E12, E13, E14
- Explanation (inferred): cycle-only holder: Section.parent (src/section.rs:10) holds a strong Rc to the parent; append at src/section.rs:27 stores it, closing a cycle with children, and nothing removes it, so each document tree is never freed.
- Explanation structure: adds: src/section.rs:27; removal: never

### F2: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Rc reference cycle between Section.parent and Section.children; no Weak back-link, trees never freed)
- Growth: `native.live_bytes@src/section.rs:16` 4213 bytes/n (steady, gate pass, E12)
- Evidence classes: context, discriminator, growth, static
- Holder: cycle-only
- Region: heap
- Shape: steady
- Site: `src/section.rs:11`
- Writers: `src/section.rs:28`
- Verified: no
- Sibling hypotheses: F2.n1
- Supporting evidence: E4, E8, E12, E13, E15
- Explanation (inferred): cycle-only holder: Section.children (src/section.rs:11) holds Rc to each child; append at src/section.rs:28 pushes it, the child's parent link closes a cycle, and nothing removes it, so each document tree is never freed.
- Explanation structure: adds: src/section.rs:28; removal: never

### F3: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Rc reference cycle: Section.children holds Rc to child and child.parent holds Rc to parent (set in Section::append); no Weak, so every parsed document tree is never freed after the request)
- Growth: `native.live_blocks@src/section.rs:16` 40.54 count/n (steady, gate pass, E10)
- Evidence classes: context, discriminator, growth, static
- Holder: cycle-only
- Owner: `src.section.Section.parent`
- Region: heap
- Shape: steady
- Site: `src/section.rs:10`
- Writers: `src/section.rs:27`
- Annotation: parent back-link set in append
- Verified: no
- Sibling hypotheses: F3.n1
- Supporting evidence: E5, E8, E10, E11
- Explanation (inferred): Section.parent (src/section.rs:10) holds a strong Rc to the parent while parent.children holds an Rc to the child; append at src/section.rs:27 adds the back-link and nothing removes it (never downgraded to Weak), so each parsed tree stays alive after the request.
- Explanation structure: adds: src/section.rs:27; removal: never

### F4: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Rc reference cycle between Section.parent and Section.children; no Weak back-link, trees never freed)
- Growth: `native.live_bytes@src/section.rs:16` 4213 bytes/n (steady, gate pass, E12)
- Evidence classes: context, discriminator, growth, static
- Holder: cycle-only
- Owner: `src.section.Section.children`
- Region: heap
- Shape: steady
- Site: `src/section.rs:11`
- Writers: `src/section.rs:28`
- Annotation: heuristic name match over the project's sources
- Annotation: children push
- Verified: no
- Sibling hypotheses: F4.n1
- Supporting evidence: E5, E8, E12, E13, E16
- Explanation (inferred): Section.children (src/section.rs:11) holds Rc to each child; append at src/section.rs:28 pushes it, the child's parent link closes a cycle, and nothing removes it, so each document tree is never freed.
- Explanation structure: adds: src/section.rs:28; removal: never

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (leak) E14
  - Hypothesis: N-WARM: warm-up plateau
  - Refuted if: growth continues over extra cycles
  - Contradicting evidence: E14
  - Originally supported by: E13
- **F2.n1** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (leak) E15
  - Hypothesis: N-WARM: warm-up plateau
  - Refuted if: growth continues over extra cycles
  - Contradicting evidence: E15
  - Originally supported by: E13
- **F3.n1** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (leak) E11
  - Hypothesis: N-WARM: growth is warm-up and plateaus
  - Refuted if: growth continues over extra cycles and after purge
  - Contradicting evidence: E11
  - Originally supported by: E10
- **F4.n1** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (leak) E16
  - Hypothesis: N-WARM: warm-up plateau
  - Refuted if: growth continues over extra cycles
  - Contradicting evidence: E16
  - Originally supported by: E13

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
- E11: partial coverage (9 of 26 examined)
- E11: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: points 1 to 9 come from S1; the extra points from S2
- E11: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E11: limitation: gate and plateau thresholds are provisional (not calibrated)
- E12: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: partial coverage (9 of 32 examined)
- E14: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: points 1 to 9 come from S1; the extra points from S4
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: partial coverage (9 of 32 examined)
- E15: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: points 1 to 9 come from S1; the extra points from S6
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: partial coverage (9 of 32 examined)
- E16: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: points 1 to 9 come from S1; the extra points from S8
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: partial coverage (1 of 3 examined)
- E23: limitation: F1: growth evidence is from S2, not S1; not subtracted
- E23: limitation: F2: growth evidence is from S2, not S1; not subtracted
- E23: limitation: F3: growth measured as a count; bytes not attributed
- E23: limitation: F4: growth evidence is from S2, not S1; not subtracted
- E23: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E23: limitation: threshold is provisional (not calibrated)
- E24: partial coverage (1 of 3 examined)
- E24: limitation: F3: growth evidence is from S1, not S2; not subtracted
- E24: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E24: limitation: threshold is provisional (not calibrated)
- E25: partial coverage (1 of 3 examined)
- E25: limitation: F1: growth evidence is from S2, not S3; not subtracted
- E25: limitation: F2: growth evidence is from S2, not S3; not subtracted
- E25: limitation: F3: growth evidence is from S1, not S3; not subtracted
- E25: limitation: F4: growth evidence is from S2, not S3; not subtracted
- E25: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E25: limitation: threshold is provisional (not calibrated)
- E26: partial coverage (1 of 3 examined)
- E26: limitation: F1: growth evidence is from S2, not S4; not subtracted
- E26: limitation: F2: growth evidence is from S2, not S4; not subtracted
- E26: limitation: F3: growth evidence is from S1, not S4; not subtracted
- E26: limitation: F4: growth evidence is from S2, not S4; not subtracted
- E26: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E26: limitation: threshold is provisional (not calibrated)
- E27: partial coverage (1 of 3 examined)
- E27: limitation: F1: growth evidence is from S2, not S5; not subtracted
- E27: limitation: F2: growth evidence is from S2, not S5; not subtracted
- E27: limitation: F3: growth evidence is from S1, not S5; not subtracted
- E27: limitation: F4: growth evidence is from S2, not S5; not subtracted
- E27: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E27: limitation: threshold is provisional (not calibrated)
- E28: partial coverage (1 of 3 examined)
- E28: limitation: F1: growth evidence is from S2, not S6; not subtracted
- E28: limitation: F2: growth evidence is from S2, not S6; not subtracted
- E28: limitation: F3: growth evidence is from S1, not S6; not subtracted
- E28: limitation: F4: growth evidence is from S2, not S6; not subtracted
- E28: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E28: limitation: threshold is provisional (not calibrated)
- E29: partial coverage (1 of 3 examined)
- E29: limitation: F1: growth evidence is from S2, not S7; not subtracted
- E29: limitation: F2: growth evidence is from S2, not S7; not subtracted
- E29: limitation: F3: growth evidence is from S1, not S7; not subtracted
- E29: limitation: F4: growth evidence is from S2, not S7; not subtracted
- E29: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E29: limitation: threshold is provisional (not calibrated)
- E30: partial coverage (1 of 3 examined)
- E30: limitation: F1: growth evidence is from S2, not S8; not subtracted
- E30: limitation: F2: growth evidence is from S2, not S8; not subtracted
- E30: limitation: F3: growth evidence is from S1, not S8; not subtracted
- E30: limitation: F4: growth evidence is from S2, not S8; not subtracted
- E30: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E30: limitation: threshold is provisional (not calibrated)
- E31: partial coverage (1 of 3 examined)
- E31: limitation: F1: growth evidence is from S2, not S9; not subtracted
- E31: limitation: F2: growth evidence is from S2, not S9; not subtracted
- E31: limitation: F3: growth evidence is from S1, not S9; not subtracted
- E31: limitation: F4: growth evidence is from S2, not S9; not subtracted
- E31: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E31: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | — |
| E2 | run-config | static | dla 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | probe/probe.json |
| E4 | static-hit | static | dla 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | sweep/rust-dla.json |
| E5 | census-holder | static | dla-census 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | census/rust.json |
| E6 | workload | static | dla 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | workload/workloads.json |
| E7 | workload | static | dla 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | harness/H1/harness.json |
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (d5fe38e0b172) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (daa74c63ab4c) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (daa74c63ab4c) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (f5a612830387) | alloc/A1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S2.json (f20d658d31db) | trend/T3.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (f20d658d31db) | trend/T3.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S5.json (50040ec53ebe) | alloc/A2.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S7.json (81f0147044d4) | alloc/A3.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S9.json (58b43130f4b6) | alloc/A4.json |
| E17 | growth-series | test | dla 0.1.0 | artifact series/S4.json (eee2c87cf8f6) | trend/T7.json |
| E18 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (eee2c87cf8f6) | trend/T7.json |
| E19 | growth-series | test | dla 0.1.0 | artifact series/S6.json (bd936690ddd2) | trend/T8.json |
| E20 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (bd936690ddd2) | trend/T8.json |
| E21 | growth-series | test | dla 0.1.0 | artifact series/S8.json (c3610f3d85b1) | trend/T9.json |
| E22 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (c3610f3d85b1) | trend/T9.json |
| E23 | residual | test | dla 0.1.0 | artifact series/S1.json (daa74c63ab4c) | gap/residual-S1-E23.json |
| E24 | residual | test | dla 0.1.0 | artifact series/S2.json (f20d658d31db) | gap/residual-S2-E24.json |
| E25 | residual | test | dla 0.1.0 | artifact series/S3.json (f5a612830387) | gap/residual-S3-E25.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S4.json (eee2c87cf8f6) | gap/residual-S4-E26.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S5.json (50040ec53ebe) | gap/residual-S5-E27.json |
| E28 | residual | test | dla 0.1.0 | artifact series/S6.json (bd936690ddd2) | gap/residual-S6-E28.json |
| E29 | residual | test | dla 0.1.0 | artifact series/S7.json (81f0147044d4) | gap/residual-S7-E29.json |
| E30 | residual | test | dla 0.1.0 | artifact series/S8.json (c3610f3d85b1) | gap/residual-S8-E30.json |
| E31 | residual | test | dla 0.1.0 | artifact series/S9.json (58b43130f4b6) | gap/residual-S9-E31.json |
