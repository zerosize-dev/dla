# DLA report

Run: `20261107-195608-d17d`

Status: COMPLETE

Findings: 1 enumerated, 1 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 1

Residual: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.section.Section.children`
- Region: heap
- Site: `src/section.rs:11`
- Writers: `src/section.rs:28`
- Removers: `src/section.rs:28`
- Annotation: heuristic name match over the project's sources
- Annotation: children Vec owned by parent via Rc; parent link is Weak (section.rs:27), so no Rc cycle; whole tree dropped when Parsed goes out of scope in Service::handle
- Reason: tool_missing
- Remedy: installed dla lacks harness/sample/trend/gap/gate; rerun with a release that has them. Manual RSS check showed flat peak RSS (~13MB) for 1k-400k documents
- Supporting evidence: E5

## Ruled out

None.

## Blind spots and unknowns

### Coverage gaps

- rust/rapx: blocked (cannot copy the project: Permission denied (os error 13))

### Runtime blind spots

None recorded.

### Entry points not exercisable

- rust: `src/parser.rs::ParseError.fmt`: no_harness

### Evidence limitations and unknowns

- E5: partial coverage
- E5: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E5: limitation: writers and removers are textual name matches
- E6: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E6: limitation: workloads were discovered by reading files; none was run

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | outline-service Rust crate in project root |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | whole crate src/ |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | RSS grows with documents processed (from task text) |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E3 | rust only |
| `intake.build` | Does the project build locally? | answered | E3 | cargo builds ok |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | main --iterations N synthetic documents |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 raw values |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 60m |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | — |
| E2 | run-config | static | dla 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | probe/probe.json |
| E4 | static-hit | static | dla 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | sweep/dla.json |
| E5 | census-holder | static | dla-census 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | census/rust.json |
| E6 | workload | static | dla 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | workload/workloads.json |
