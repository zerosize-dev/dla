# DLA report

Run: `20261108-203337-671e`

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
- Owner: `src.section.Section.parent`
- Region: heap
- Shape: steady
- Site: `src/section.rs:10`
- Annotation: Detector hit on same cycle as F3 (parent back-link)
- Sibling hypotheses: F1.n1
- Suggested archetype (unconfirmed): L-CYCLE
- Supporting evidence: E4, E8, E10

### F2: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `native.live_blocks@src/section.rs:16` 40.54 count/n (steady, gate pass, E10)
- Evidence classes: context, growth, static
- Holder: cycle-only
- Owner: `src.section.Section.children`
- Region: heap
- Shape: steady
- Site: `src/section.rs:11`
- Annotation: Detector hit on same cycle as F4
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
- Writers: `src/section.rs:28`, `src/section.rs:29`
- Annotation: Section.parent holds a strong Rc to the parent while the parent's children Vec holds strong Rc to the child (Section::append). Parser creates the tree per request; dropping Parsed.root leaves strong count >0 on every node, so the whole tree (titles, nodes) is never freed. No remover: the parent back-link is never cleared or Weak. Repair class: make the back-link a Weak reference.
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
- Writers: `src/section.rs:28`, `src/section.rs:29`
- Annotation: heuristic name match over the project's sources
- Annotation: Other half of the same Rc cycle: Section.children strong Rc to child, child.parent strong Rc back. Same root cause as F3.
- Sibling hypotheses: F4.n1
- Supporting evidence: E5, E8, E10

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (leak) E13
  - Hypothesis: N-WARM: warm-up fill that plateaus
  - Refuted if: live_blocks keeps growing after purge
  - Contradicting evidence: E13
  - Originally supported by: E10
- **F2.n1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (leak) E14
  - Hypothesis: N-WARM: warm-up fill that plateaus
  - Refuted if: live_blocks keeps growing after purge
  - Contradicting evidence: E14
  - Originally supported by: E10
- **F3.n1** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (leak) E11
  - Hypothesis: N-WARM: warm-up/cache fill that plateaus
  - Refuted if: live_blocks keeps growing linearly across extra cycles after purge
  - Contradicting evidence: E11
  - Originally supported by: E10
- **F4.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM: warm-up fill that plateaus
  - Refuted if: live_blocks keeps growing after purge
  - Contradicting evidence: E12
  - Originally supported by: E10

## Blind spots and unknowns

### Coverage gaps

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
- E11: partial coverage (9 of 18 examined)
- E11: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: points 1 to 9 come from S1; the extra points from S2
- E11: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E11: limitation: gate and plateau thresholds are provisional (not calibrated)
- E12: partial coverage (9 of 24 examined)
- E12: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: points 1 to 9 come from S1; the extra points from S4
- E12: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E12: limitation: gate and plateau thresholds are provisional (not calibrated)
- E13: partial coverage (9 of 24 examined)
- E13: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: points 1 to 9 come from S1; the extra points from S6
- E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- E14: partial coverage (9 of 24 examined)
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
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (5fc2ce889e9d) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (beb435915a5e) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (beb435915a5e) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (daa96f652239) | alloc/A1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S5.json (306e652d09ea) | alloc/A2.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S7.json (5dd08757fd69) | alloc/A3.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S9.json (0b719aa8fb59) | alloc/A4.json |
