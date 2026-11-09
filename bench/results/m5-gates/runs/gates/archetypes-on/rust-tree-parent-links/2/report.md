# DLA report

Run: `20261109-202509-aaeb`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 8 enumerated, 8 dispositioned; confirmed 2, strong 0, refuted 2, suspected 0, unresolved 4

Residual: not measured

Coverage: rust 1760 iterations, 0 of 10 entry points exercised

## Findings

### F3: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `native.live_blocks@src/section.rs:16` 40.54 count/n (steady, gate pass, E10)
- Evidence classes: context, discriminator, growth, static
- Holder: cycle-only
- Owner: `src.section.Section.parent`
- Region: heap
- Shape: steady
- Site: `src/section.rs:10`
- Writers: `src/section.rs:24`
- Annotation: Section.parent holds a strong Rc to the parent while parent.children holds Rc to child: reference cycle, so every parsed document tree (Parsed.root) is never freed after the response. Missing pairing: parent back-link should be non-owning (Weak) or be cleared on drop.
- Sibling hypotheses: F3.n1
- Supporting evidence: E5, E8, E10, E11

### F4: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `native.live_blocks@src/section.rs:16` 40.54 count/n (steady, gate pass, E10)
- Evidence classes: context, discriminator, growth, static
- Holder: cycle-only
- Owner: `src.section.Section.children`
- Region: heap
- Shape: steady
- Site: `src/section.rs:11`
- Writers: `src/section.rs:28`
- Annotation: heuristic name match over the project's sources
- Annotation: children side of the Section parent<->children Rc cycle (same leak as F3)
- Sibling hypotheses: F4.n1
- Supporting evidence: E5, E8, E10, E12

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: context, static
- Site: `src/section.rs:10`
- Sibling hypotheses: F1.n1
- Suggested archetype (unconfirmed): L-CYCLE
- Reason: not_exercised
- Remedy: Detector hit duplicates F3/F4 (same Rc cycle at section.rs struct fields); no separate holder. Fix class: make parent link Weak.
- Supporting evidence: E4, E8

### F1.n1: Hypothesis N-WARM: warm-up retention that plateaus (UNCLASSIFIED)

- State: unresolved(not_exercised)
- Evidence classes: growth
- Sibling hypothesis of: F1
- Hypothesis: N-WARM: warm-up retention that plateaus
- Refuted if: growth continues over extra cycles
- Reason: not_exercised
- Remedy: sibling of duplicate detector hit
- Supporting evidence: E10

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: context, static
- Site: `src/section.rs:11`
- Sibling hypotheses: F2.n1
- Suggested archetype (unconfirmed): L-CYCLE
- Reason: not_exercised
- Remedy: Detector hit duplicates F3/F4 (same Rc cycle at section.rs struct fields); no separate holder. Fix class: make parent link Weak.
- Supporting evidence: E4, E8

### F2.n1: Hypothesis N-WARM: warm-up retention that plateaus (UNCLASSIFIED)

- State: unresolved(not_exercised)
- Evidence classes: growth
- Sibling hypothesis of: F2
- Hypothesis: N-WARM: warm-up retention that plateaus
- Refuted if: growth continues over extra cycles
- Reason: not_exercised
- Remedy: sibling of duplicate detector hit
- Supporting evidence: E10

## Ruled out

- **F3.n1** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (leak) E11
  - Hypothesis: N-WARM: one-time warm-up/allocator retention that plateaus
  - Refuted if: growth continues over the extra cycles after purge
  - Contradicting evidence: E11
  - Originally supported by: E10
- **F4.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM: warm-up retention that plateaus
  - Refuted if: growth continues over extra cycles
  - Contradicting evidence: E12
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
- E12: partial coverage (9 of 32 examined)
- E12: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: points 1 to 9 come from S1; the extra points from S4
- E12: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E12: limitation: gate and plateau thresholds are provisional (not calibrated)
- E13: partial coverage (9 of 32 examined)
- E13: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: points 1 to 9 come from S1; the extra points from S6
- E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- E14: partial coverage (9 of 32 examined)
- E14: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: points 1 to 9 come from S1; the extra points from S8
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | — |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | — |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | — |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E3 | — |
| `intake.build` | Does the project build locally? | answered | E3 | — |
| `intake.workload` | What can exercise the suspected path? | answered | E6 | — |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
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
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (d2a834455a11) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (c2419e979751) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (c2419e979751) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (0acf99375c86) | alloc/A1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S5.json (085841caea09) | alloc/A2.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S7.json (2485461a3d7d) | alloc/A3.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S9.json (400d3655d333) | alloc/A4.json |
