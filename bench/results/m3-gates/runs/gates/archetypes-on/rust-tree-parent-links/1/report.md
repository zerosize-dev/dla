# DLA report

Run: `20261108-123932-76ff`

Status: COMPLETE

Findings: 5 enumerated, 5 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 3

Residual: not measured

Coverage: rust 1760 iterations, 0 of 10 entry points exercised

## Findings

### F2: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `native.live_blocks@src/section.rs:16` 40.54 count/n (steady, gate pass, E11)
- Evidence classes: context, growth, static
- Holder: cycle-only
- Owner: `src.section.Section`
- Region: heap
- Shape: steady
- Site: `src/section.rs:11`
- Writers: `src/section.rs:28`
- Annotation: children Vec pushes; parent->child strong, child->parent strong => cycle; no remover
- Sibling hypotheses: F2.n1
- Suggested archetype (unconfirmed): L-CYCLE
- Supporting evidence: E5, E9, E11

### F1: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth, static
- Site: `src/section.rs:10`
- Writers: `src/section.rs:27`
- Annotation: parent back-link Rc set in append; never cleared, no Drop impl
- Suggested archetype (unconfirmed): L-CYCLE
- Reason: time_budget
- Remedy: Same Rc parent<->children cycle as confirmed F2 (other half of the pair); not separately tested. Make Section.parent a Weak<Section> (or clear links on drop), then re-run harness H1 to confirm live_blocks plateaus.
- Supporting evidence: E5, E11

### F3: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth, static
- Holder: cycle-only
- Owner: `src.section.Section.parent`
- Region: heap
- Site: `src/section.rs:10`
- Writers: `src/section.rs:27`
- Annotation: see list
- Annotation: parent field: strong Rc to parent set at append, never cleared; cycle with F4 children
- Reason: time_budget
- Remedy: Same Rc parent<->children cycle as confirmed F2 (other half of the pair); not separately tested. Make Section.parent a Weak<Section> (or clear links on drop), then re-run harness H1 to confirm live_blocks plateaus.
- Supporting evidence: E6, E11

### F4: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth, static
- Holder: cycle-only
- Owner: `src.section.Section.children`
- Region: heap
- Site: `src/section.rs:11`
- Writers: `src/section.rs:28`
- Annotation: heuristic name match over the project's sources
- Annotation: x
- Reason: time_budget
- Remedy: Same Rc parent<->children cycle as confirmed F2 (other half of the pair); not separately tested. Make Section.parent a Weak<Section> (or clear links on drop), then re-run harness H1 to confirm live_blocks plateaus.
- Supporting evidence: E6, E11

## Ruled out

- **F2.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-ALLOC allocator retention / warm-up: freed memory not returned, growth would plateau after purge
  - Refuted if: live_blocks (allocator-level live count) keeps growing per document across extra points and after purge
  - Contradicting evidence: E12
  - Originally supported by: E11

## Blind spots and unknowns

### Coverage gaps

- rust/rapx: blocked (cannot copy the project: Permission denied (os error 13))

### Runtime blind spots

- rust: the allocator shim does not record which entry points ran

### Entry points not exercisable

- rust: `src/parser.rs::ParseError.fmt`: no_harness

### Evidence limitations and unknowns

- E6: partial coverage
- E6: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E6: limitation: writers and removers are textual name matches
- E7: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E7: limitation: workloads were discovered by reading files; none was run
- E8: limitation: harness written; nothing was run (dla harness run measures it)
- E8: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E9: partial coverage (9 of 9 examined)
- E9: limitation: 544 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: exercised entry points are not measured for rust: the allocator shim sees allocations, not calls
- E10: limitation: 544 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: 544 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: partial coverage (9 of 18 examined)
- E12: limitation: 544 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: points 1 to 9 come from S1; the extra points from S2
- E12: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E12: limitation: gate and plateau thresholds are provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | outline-service rust crate |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | from task text and repo: single rust crate, RSS grows per document, harness via --iterations |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | from task text and repo: single rust crate, RSS grows per document, harness via --iterations |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | from task text and repo: single rust crate, RSS grows per document, harness via --iterations |
| `intake.build` | Does the project build locally? | answered | E1 | from task text and repo: single rust crate, RSS grows per document, harness via --iterations |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | from task text and repo: single rust crate, RSS grows per document, harness via --iterations |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 raw values |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 60m |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | — |
| E2 | run-config | static | dla 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | probe/probe.json |
| E4 | probe | static | dla-probe 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | probe/probe.json |
| E5 | static-hit | static | dla 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | sweep/dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | census/rust.json |
| E7 | workload | static | dla 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (d6f83953cf52) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (4fcde8e6e477) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (4fcde8e6e477) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (d0d859ee79b2) | alloc/A1.json |
