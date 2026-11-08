# DLA report

Run: `20261108-200926-f32a`

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
- Annotation: Same Rc strong cycle as F3: parent back-link (Section.parent, section.rs:10/28) and children Vec (:11) both hold strong Rc, so dropped document trees are never freed. Repair class: make the back-link non-owning (Weak).
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
- Annotation: Same Rc strong cycle as F3: parent back-link (Section.parent, section.rs:10/28) and children Vec (:11) both hold strong Rc, so dropped document trees are never freed. Repair class: make the back-link non-owning (Weak).
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
- Writers: `src/section.rs:28`
- Annotation: Rc<Section> strong reference cycle: Section::append stores a strong Rc to the parent in child.parent (section.rs:28 / field at :10) while parent.children (:11) holds strong Rc to the child. Parsed tree (parser.rs root Rc) is dropped after each request but strong counts never reach zero, so every Section node (title String, RefCell, Vec) of every document is leaked. Missing pairing: the back-link should be non-owning (Weak) or severed on teardown. Also toc::render's breadcrumb walks parent links for level>=3. Archetype unlabelled (UNCLASSIFIED: no label commands in this release).
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
- Annotation: Same Rc strong cycle as F3: parent back-link (Section.parent, section.rs:10/28) and children Vec (:11) both hold strong Rc, so dropped document trees are never freed. Repair class: make the back-link non-owning (Weak).
- Sibling hypotheses: F4.n1
- Supporting evidence: E5, E8, E10

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM: growth is a warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles of alloc-test
  - Contradicting evidence: E12
  - Originally supported by: E10
- **F2.n1** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (leak) E13
  - Hypothesis: N-WARM: growth is a warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles of alloc-test
  - Contradicting evidence: E13
  - Originally supported by: E10
- **F3.n1** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (leak) E11
  - Hypothesis: N-WARM: growth is a warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles of alloc-test
  - Contradicting evidence: E11
  - Originally supported by: E10
- **F4.n1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (leak) E14
  - Hypothesis: N-WARM: growth is a warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles of alloc-test
  - Contradicting evidence: E14
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
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (cffd1671d45c) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (531fc74058ec) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (531fc74058ec) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (46f40465e2af) | alloc/A1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S5.json (f45fc3b08fa3) | alloc/A2.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S7.json (dc8eab5e9cc4) | alloc/A3.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S9.json (55d8b3cf4b3f) | alloc/A4.json |
