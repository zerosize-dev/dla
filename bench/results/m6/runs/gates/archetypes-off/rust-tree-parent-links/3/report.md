# DLA report

Run: `20261110-194347-0685`

Status: INCOMPLETE (28 open)

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 8 enumerated, 8 dispositioned; confirmed 4, strong 0, refuted 4, suspected 0, unresolved 0

Residual: heap 5268.498106060606 bytes/n unattributed of 5268.498106060606 total (threshold 1317.1245265151515) ABOVE THRESHOLD, see O1; heap 5262.361363636363 bytes/n unattributed of 5262.361363636363 total (threshold 1315.5903409090909) ABOVE THRESHOLD, see O1; heap 0 bytes/n unattributed of 5262.361363636363 total (threshold 1315.5903409090909); heap 0 bytes/n unattributed of 5262.361363636363 total (threshold 1315.5903409090909); heap 0 bytes/n unattributed of 5262.361363636363 total (threshold 1315.5903409090909)

Coverage: rust 1760 iterations, 0 of 10 entry points exercised

## Open items

- **T2:os.rss**: column os.rss of T2 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T2:os.rss_anon**: column os.rss_anon of T2 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T2:os.vm_data**: column os.vm_data of T2 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T2:os.vm_hwm**: column os.vm_hwm of T2 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T2:native.live_blocks**: column native.live_blocks of T2 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T2:native.live_bytes**: column native.live_bytes of T2 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T2:native.peak_bytes**: column native.peak_bytes of T2 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:os.rss**: column os.rss of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:os.rss_anon**: column os.rss_anon of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:os.vm_data**: column os.vm_data of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:os.vm_hwm**: column os.vm_hwm of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:native.live_blocks**: column native.live_blocks of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:native.live_bytes**: column native.live_bytes of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:native.peak_bytes**: column native.peak_bytes of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.rss**: column os.rss of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.rss_anon**: column os.rss_anon of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.vm_data**: column os.vm_data of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.vm_hwm**: column os.vm_hwm of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:native.live_blocks**: column native.live_blocks of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:native.live_bytes**: column native.live_bytes of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:native.peak_bytes**: column native.peak_bytes of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:os.rss**: column os.rss of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:os.rss_anon**: column os.rss_anon of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:os.vm_data**: column os.vm_data of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:os.vm_hwm**: column os.vm_hwm of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:native.live_blocks**: column native.live_blocks of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:native.live_bytes**: column native.live_bytes of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:native.peak_bytes**: column native.peak_bytes of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`

## Findings

### F1: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Same Rc cycle as F3: Section.children strong Rc to child and child.parent strong Rc back, set in Section::append; per-request tree never freed)
- Growth: `native.live_bytes@src/section.rs:16` 4213 bytes/n (steady, gate pass, E19)
- Evidence classes: context, discriminator, growth, static
- Holder: cycle-only
- Region: heap
- Shape: steady
- Site: `src/section.rs:10`
- Writers: `src/section.rs:28`
- Annotation: append builds the strong parent<->children cycle; no Weak, no remover
- Verified: no
- Sibling hypotheses: F1.n1
- Supporting evidence: E4, E8, E15, E19
- Explanation (inferred): Site src/section.rs:10, a cycle-only holder (Rc strong cycle between Section.parent and Section.children): append at section.rs:28 adds both links, and nothing removes or downgrades them, so removal never happens and the tree leaks.
- Explanation structure: adds: src/section.rs:28; removal: never

### F2: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Same Rc cycle as F3: Section.children strong Rc to child and child.parent strong Rc back, set in Section::append; per-request tree never freed)
- Growth: `native.live_bytes@src/section.rs:16` 4213 bytes/n (steady, gate pass, E21)
- Evidence classes: context, discriminator, growth, static
- Holder: cycle-only
- Region: heap
- Shape: steady
- Site: `src/section.rs:11`
- Writers: `src/section.rs:28`
- Annotation: append builds the strong parent<->children cycle; no Weak, no remover
- Verified: no
- Sibling hypotheses: F2.n1
- Supporting evidence: E4, E8, E16, E21
- Explanation (inferred): Site src/section.rs:11, a cycle-only holder (Rc strong cycle between Section.parent and Section.children): append at section.rs:28 adds both links, and nothing removes or downgrades them, so removal never happens and the tree leaks.
- Explanation structure: adds: src/section.rs:28; removal: never

### F3: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Rc reference cycle: Section.parent holds a strong Rc to the parent while the parent's children Vec holds Rc to the child; parser::parse builds this per request so every document tree is never freed (strong count never reaches 0))
- Growth: `native.live_blocks@src/section.rs:16` 40.54 count/n (steady, gate pass, E10)
- Evidence classes: context, discriminator, growth, static
- Holder: cycle-only
- Owner: `src.section.Section.parent`
- Region: heap
- Shape: steady
- Site: `src/section.rs:10`
- Writers: `src/section.rs:28`, `src/section.rs:29`
- Annotation: append() sets child.parent = Rc::clone(parent) and pushes child into parent.children: strong cycle; no Weak, no remover
- Verified: no
- Sibling hypotheses: F3.n1
- Supporting evidence: E5, E8, E10, E11
- Explanation (inferred): Section.parent (src/section.rs:10) holds a strong Rc to the parent while children holds Rc to child; append at section.rs:28 creates a cycle per heading and nothing removes or downgrades the link, so removal never happens and each document tree stays allocated.
- Explanation structure: adds: src/section.rs:28; removal: never

### F4: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Same Rc cycle as F3: Section.children strong Rc to child and child.parent strong Rc back, set in Section::append; per-request tree never freed)
- Growth: `native.live_bytes@src/section.rs:16` 4213 bytes/n (steady, gate pass, E17)
- Evidence classes: context, discriminator, growth, static
- Holder: cycle-only
- Owner: `src.section.Section.children`
- Region: heap
- Shape: steady
- Site: `src/section.rs:11`
- Writers: `src/section.rs:28`
- Annotation: heuristic name match over the project's sources
- Annotation: append builds the strong parent<->children cycle; no Weak, no remover
- Verified: no
- Sibling hypotheses: F4.n1
- Supporting evidence: E5, E8, E14, E17
- Explanation (inferred): Site src/section.rs:11, owner Section.children, a cycle-only holder: the parent's children Vec holds a strong Rc to each child, added by append at section.rs:28, while child.parent points back; nothing removes it, removal never happens.
- Explanation structure: adds: src/section.rs:28; removal: never

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (leak) E15
  - Hypothesis: N-WARM: growth is warm-up that plateaus
  - Refuted if: growth continues over extra cycles
  - Contradicting evidence: E15
  - Originally supported by: E10
- **F2.n1** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (leak) E16
  - Hypothesis: N-WARM: growth is warm-up that plateaus
  - Refuted if: growth continues over extra cycles
  - Contradicting evidence: E16
  - Originally supported by: E10
- **F3.n1** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (leak) E11
  - Hypothesis: N-WARM: growth is warm-up that plateaus
  - Refuted if: growth continues over extra cycles and does not plateau after trim
  - Contradicting evidence: E11
  - Originally supported by: E10
- **F4.n1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (leak) E14
  - Hypothesis: N-WARM: growth is warm-up that plateaus
  - Refuted if: growth continues over extra cycles
  - Contradicting evidence: E14
  - Originally supported by: E10

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
- E14: partial coverage (9 of 26 examined)
- E14: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: points 1 to 9 come from S1; the extra points from S4
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: partial coverage (9 of 26 examined)
- E15: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: points 1 to 9 come from S1; the extra points from S6
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: partial coverage (9 of 26 examined)
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
- E23: limitation: F1: growth evidence is from S6, not S1; not subtracted
- E23: limitation: F2: growth evidence is from S8, not S1; not subtracted
- E23: limitation: F3: growth measured as a count; bytes not attributed
- E23: limitation: F4: growth evidence is from S4, not S1; not subtracted
- E23: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E23: limitation: threshold is provisional (not calibrated)
- E24: partial coverage (1 of 3 examined)
- E24: limitation: F1: growth evidence is from S6, not S2; not subtracted
- E24: limitation: F2: growth evidence is from S8, not S2; not subtracted
- E24: limitation: F3: growth evidence is from S1, not S2; not subtracted
- E24: limitation: F4: growth evidence is from S4, not S2; not subtracted
- E24: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E24: limitation: threshold is provisional (not calibrated)
- E25: partial coverage (1 of 3 examined)
- E25: limitation: F1: growth evidence is from S6, not S4; not subtracted
- E25: limitation: F2: growth evidence is from S8, not S4; not subtracted
- E25: limitation: F3: growth evidence is from S1, not S4; not subtracted
- E25: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E25: limitation: threshold is provisional (not calibrated)
- E26: partial coverage (1 of 3 examined)
- E26: limitation: F2: growth evidence is from S8, not S6; not subtracted
- E26: limitation: F3: growth evidence is from S1, not S6; not subtracted
- E26: limitation: F4: growth evidence is from S4, not S6; not subtracted
- E26: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E26: limitation: threshold is provisional (not calibrated)
- E27: partial coverage (1 of 3 examined)
- E27: limitation: F1: growth evidence is from S6, not S8; not subtracted
- E27: limitation: F3: growth evidence is from S1, not S8; not subtracted
- E27: limitation: F4: growth evidence is from S4, not S8; not subtracted
- E27: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E27: limitation: threshold is provisional (not calibrated)

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
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (5605a285baa6) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (476595746871) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (476595746871) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (290c3eb8df0e) | alloc/A1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S2.json (676033a4ba1c) | trend/T3.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (676033a4ba1c) | trend/T3.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S5.json (0d592524dd45) | alloc/A2.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S7.json (1e0dc20b9ff4) | alloc/A3.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S9.json (cc8152707dba) | alloc/A4.json |
| E17 | growth-series | test | dla 0.1.0 | artifact series/S4.json (3a3ac9e3de6d) | trend/T7.json |
| E18 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (3a3ac9e3de6d) | trend/T7.json |
| E19 | growth-series | test | dla 0.1.0 | artifact series/S6.json (57b46cb8a868) | trend/T8.json |
| E20 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (57b46cb8a868) | trend/T8.json |
| E21 | growth-series | test | dla 0.1.0 | artifact series/S8.json (5eddb91ec19e) | trend/T9.json |
| E22 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (5eddb91ec19e) | trend/T9.json |
| E23 | residual | test | dla 0.1.0 | artifact series/S1.json (476595746871) | gap/residual-S1-E23.json |
| E24 | residual | test | dla 0.1.0 | artifact series/S2.json (676033a4ba1c) | gap/residual-S2-E24.json |
| E25 | residual | test | dla 0.1.0 | artifact series/S4.json (3a3ac9e3de6d) | gap/residual-S4-E25.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S6.json (57b46cb8a868) | gap/residual-S6-E26.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S8.json (5eddb91ec19e) | gap/residual-S8-E27.json |
