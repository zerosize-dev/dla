# DLA report

Run: `20261109-195524-d6e5`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 8 enumerated, 8 dispositioned; confirmed 4, strong 0, refuted 4, suspected 0, unresolved 0

Residual: not measured

Coverage: rust 1760 iterations, 0 of 10 entry points exercised

## Findings

### F1: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `native.live_blocks@src/section.rs:16` 40.54 count/n (steady, gate pass, E10)
- Evidence classes: context, growth, static
- Holder: cycle-only
- Region: heap
- Shape: steady
- Site: `src/section.rs:10`
- Sibling hypotheses: F1.n1
- Suggested archetype (unconfirmed): L-CYCLE
- Supporting evidence: E4, E8, E10

### F2: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `native.live_blocks@src/section.rs:16` 40.54 count/n (steady, gate pass, E10)
- Evidence classes: context, growth, static
- Holder: cycle-only
- Region: heap
- Shape: steady
- Site: `src/section.rs:11`
- Sibling hypotheses: F2.n1
- Suggested archetype (unconfirmed): L-CYCLE
- Supporting evidence: E4, E8, E10

### F3: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `native.live_blocks@src/section.rs:16` 40.54 count/n (steady, gate pass, E10)
- Evidence classes: context, growth, static
- Holder: cycle-only
- Owner: `src.section.Section.parent`
- Region: heap
- Shape: steady
- Site: `src/section.rs:10`
- Annotation: Rc reference cycle: Section.parent (strong Rc, section.rs:10) and Section.children (strong Rc, section.rs:11) point at each other via Section::append; no Weak back-link and no teardown, so every parsed document tree is never freed after the response. Missing pairing: parent back-link should be non-owning (Weak).
- Sibling hypotheses: F3.n1
- Supporting evidence: E5, E8, E10

### F4: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `native.live_blocks@src/section.rs:16` 40.54 count/n (steady, gate pass, E10)
- Evidence classes: context, growth, static
- Holder: cycle-only
- Owner: `src.section.Section.children`
- Region: heap
- Shape: steady
- Site: `src/section.rs:11`
- Writers: `src/section.rs:28`
- Annotation: heuristic name match over the project's sources
- Annotation: Forward half of the same Rc cycle as F3: children holds strong Rc to child, child.parent holds strong Rc back.
- Sibling hypotheses: F4.n1
- Supporting evidence: E5, E8, E10

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (leak) E16
  - Hypothesis: N-WARM: warm-up fill that plateaus
  - Refuted if: growth continues after purge
  - Contradicting evidence: E16
  - Originally supported by: E10
- **F2.n1** (UNCLASSIFIED): discriminator E17
  - Ruled out by: plateau+purge (leak) E17
  - Hypothesis: N-WARM: warm-up fill that plateaus
  - Refuted if: growth continues after purge
  - Contradicting evidence: E17
  - Originally supported by: E10
- **F3.n1** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (leak) E11
  - Hypothesis: N-WARM: warm-up/cache fill that plateaus
  - Refuted if: live blocks keep growing after purge over extra cycles
  - Contradicting evidence: E11
  - Originally supported by: E10
- **F4.n1** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (leak) E15
  - Hypothesis: N-WARM: warm-up fill that plateaus
  - Refuted if: growth continues after purge
  - Contradicting evidence: E15
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
- E15: partial coverage (9 of 32 examined)
- E15: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: points 1 to 9 come from S1; the extra points from S10
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: partial coverage (9 of 32 examined)
- E16: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: points 1 to 9 come from S1; the extra points from S12
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: partial coverage (9 of 32 examined)
- E17: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: points 1 to 9 come from S1; the extra points from S14
- E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E17: limitation: gate and plateau thresholds are provisional (not calibrated)

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
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (e946835751bf) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (1bd221975991) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (1bd221975991) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (a0daedb2dd4d) | alloc/A1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S5.json (8526179094b3) | alloc/A2.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S7.json (f5c6a588256e) | alloc/A3.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S9.json (87fe07ceca72) | alloc/A4.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S11.json (88c019a26982) | alloc/A5.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S13.json (677f9011acd6) | alloc/A6.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S15.json (a1fff23293fe) | alloc/A7.json |
