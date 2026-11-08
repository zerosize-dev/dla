# DLA report

Run: `20261108-194313-15f7`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 5 enumerated, 5 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 3

Residual: not measured

Coverage: rust 1760 iterations, 0 of 10 entry points exercised

## Findings

### F3: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `native.live_bytes@src/section.rs:16` 4216 bytes/n (steady, gate pass, E9)
- Evidence classes: context, growth, static
- Holder: cycle-only
- Owner: `src.section.Section.parent`
- Region: heap
- Shape: steady
- Site: `src/section.rs:10`
- Annotation: Section.parent holds a strong Rc to the parent while the parent's children Vec holds Rc to the child (section.rs:28-29, set in Section::append). Reference-count cycle: after parse() returns and root is dropped, every Section keeps strong count >=1 via its neighbour, so no tree is ever freed. Per-document growth ~ whole tree. Missing pairing: the back-link should be non-owning (Weak) or be cleared on teardown.
- Sibling hypotheses: F3.n1
- Supporting evidence: E5, E8, E9

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `src/section.rs:10`
- Suggested archetype (unconfirmed): L-CYCLE
- Reason: not_exercised
- Remedy: Same Rc cycle as confirmed F3 (other half or detector duplicate of section.rs:10/11); fixing F3's back-link covers it; not separately confirmed
- Supporting evidence: E4

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `src/section.rs:11`
- Suggested archetype (unconfirmed): L-CYCLE
- Reason: not_exercised
- Remedy: Same Rc cycle as confirmed F3 (other half or detector duplicate of section.rs:10/11); fixing F3's back-link covers it; not separately confirmed
- Supporting evidence: E4

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: cycle-only
- Owner: `src.section.Section.children`
- Region: heap
- Site: `src/section.rs:11`
- Writers: `src/section.rs:28`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Same Rc cycle as confirmed F3 (other half or detector duplicate of section.rs:10/11); fixing F3's back-link covers it; not separately confirmed
- Supporting evidence: E5

## Ruled out

- **F3.n1** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (leak) E11
  - Hypothesis: N-WARM: warm-up/high-water fill that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E11
  - Originally supported by: E11

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
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (71ac8c695b61) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (8c269fa0a5cb) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (8c269fa0a5cb) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (c8419355151b) | alloc/A1.json |
