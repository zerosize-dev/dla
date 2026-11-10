# DLA report

Run: `20261110-190657-197d`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 5 enumerated, 5 dispositioned; confirmed 1, strong 0, refuted 1, suspected 3, unresolved 0

Residual: heap 5268.498106060606 bytes/n unattributed of 5268.498106060606 total (threshold 1317.1245265151515) ABOVE THRESHOLD, see O1; heap 5262.361363636363 bytes/n unattributed of 5262.361363636363 total (threshold 1315.5903409090909) ABOVE THRESHOLD, see O1; heap 5260.929331779332 bytes/n unattributed of 5260.929331779332 total (threshold 1315.232332944833) ABOVE THRESHOLD, see O1

Coverage: rust 1760 iterations, 0 of 10 entry points exercised

## Findings

### F3: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Rc reference cycle: Section.parent holds a strong Rc to the parent while parent.children holds the child; parse() trees are never freed after the response)
- Growth: `native.live_blocks@src/section.rs:16` 40.54 count/n (steady, gate pass, E10)
- Evidence classes: context, discriminator, growth, static
- Holder: cycle-only
- Owner: `src.section.Section.parent`
- Region: heap
- Shape: steady
- Site: `src/section.rs:10`
- Writers: `src/section.rs:28`
- Annotation: append() stores a strong Rc to the parent in child.parent while parent.children holds the child: Rc cycle, never freed when the parse result drops
- Verified: no
- Sibling hypotheses: F3.n1
- Supporting evidence: E5, E8, E10, E11
- Explanation (inferred): Section.parent (src/section.rs:10) is a cycle-only Rc holder; append at section.rs:28 links child to parent while children holds the child, and the cycle is never broken, so the tree is never freed.
- Explanation structure: adds: src/section.rs:28; removal: never

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Site: `src/section.rs:10`
- Reason: not_exercised
- Remedy: Same Rc cycle as F3 (Section.parent <-> Section.children); growth is attributed and confirmed on F3. Fix class: make the parent link a Weak
- Supporting evidence: E4

### F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Site: `src/section.rs:11`
- Reason: not_exercised
- Remedy: Same Rc cycle as F3 (Section.parent <-> Section.children); growth is attributed and confirmed on F3. Fix class: make the parent link a Weak
- Supporting evidence: E4

### F4: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: cycle-only
- Owner: `src.section.Section.children`
- Region: heap
- Site: `src/section.rs:11`
- Writers: `src/section.rs:28`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Same Rc cycle as F3 (Section.parent <-> Section.children); growth is attributed and confirmed on F3. Fix class: make the parent link a Weak
- Supporting evidence: E5, E4

## Ruled out

- **F3.n1** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (leak) E11
  - Hypothesis: N-WARM: growth is warm-up fill that plateaus
  - Refuted if: growth continues over extra cycles and after purge
  - Contradicting evidence: E11
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
- E14: partial coverage (1 of 3 examined)
- E14: limitation: F3: growth measured as a count; bytes not attributed
- E14: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E14: limitation: threshold is provisional (not calibrated)
- E15: partial coverage (1 of 3 examined)
- E15: limitation: F3: growth evidence is from S1, not S2; not subtracted
- E15: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E15: limitation: threshold is provisional (not calibrated)
- E16: partial coverage (1 of 3 examined)
- E16: limitation: F3: growth evidence is from S1, not S3; not subtracted
- E16: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E16: limitation: threshold is provisional (not calibrated)

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
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (56b96b7f6ebe) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (8138a9695604) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (8138a9695604) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (0eaf531d2fcc) | alloc/A1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S2.json (944beb7b2584) | trend/T3.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (944beb7b2584) | trend/T3.json |
| E14 | residual | test | dla 0.1.0 | artifact series/S1.json (8138a9695604) | gap/residual-S1-E14.json |
| E15 | residual | test | dla 0.1.0 | artifact series/S2.json (944beb7b2584) | gap/residual-S2-E15.json |
| E16 | residual | test | dla 0.1.0 | artifact series/S3.json (0eaf531d2fcc) | gap/residual-S3-E16.json |
