# DLA report

Run: `20261108-132645-0690`

Status: COMPLETE

Findings: 2 enumerated, 2 dispositioned; confirmed 0, strong 0, refuted 1, suspected 0, unresolved 1

Residual: not measured

Coverage: rust 3520 iterations, 0 of 10 entry points exercised

## Findings

### F1.n1: Hypothesis N-SCALE: per-request tree is freed; no growth with documents (UNCLASSIFIED)

- State: unresolved(tool_missing)
- Evidence classes: growth
- Sibling hypothesis of: F1
- Hypothesis: N-SCALE: per-request tree is freed; no growth with documents
- Refuted if: native.live_bytes or os.rss slope passes gate across iterations
- Reason: tool_missing
- Remedy: Non-leak sibling is supported by E13 (plateau, purge releases RSS); this dla release has no disposition for a supported non-leak sibling and no 'dla gate' command. Nothing further to test.
- Supporting evidence: E9

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (non-leak) E13
  - Contradicting evidence: E13
  - Originally supported by: E5, E7

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
- E8: limitation: 544 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: exercised entry points are not measured for rust: the allocator shim sees allocations, not calls
- E9: limitation: 544 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: gate thresholds are provisional (not calibrated)
- E10: limitation: 544 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: 544 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: 544 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: points 1 to 9 come from S1; the extra points from S3
- E12: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E12: limitation: gate and plateau thresholds are provisional (not calibrated)
- E13: limitation: 544 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: points 1 to 9 come from S1; the extra points from S5
- E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E13: limitation: gate and plateau thresholds are provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | from task args/repo |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | from task args/repo |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | from task args/repo |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | from task args/repo |
| `intake.build` | Does the project build locally? | answered | E3 | from task args/repo |
| `intake.workload` | What can exercise the suspected path? | answered | E6 | from task args/repo |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | from task args/repo |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | from task args/repo |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | — |
| E2 | run-config | static | dla 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | probe/probe.json |
| E4 | static-hit | static | dla 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | sweep/dla.json |
| E5 | census-holder | static | dla-census 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | census/rust.json |
| E6 | workload | static | dla 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | workload/workloads.json |
| E7 | workload | static | dla 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | harness/H1/harness.json |
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (fd421a01e3d6) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (d8f2f883900d) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (d8f2f883900d) | trend/T1.json |
| E11 | non-heap | test | dla 0.1.0 | artifact series/S2.json (98b2b3f2412e) | trend/T2.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S4.json (0faf9bb5b3aa) | alloc/A1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S6.json (c48ebda4026e) | alloc/A2.json |
