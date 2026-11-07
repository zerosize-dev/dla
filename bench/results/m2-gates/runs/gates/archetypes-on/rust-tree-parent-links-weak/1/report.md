# DLA report

Run: `20261107-194347-4455`

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
- Annotation: heuristic name match over the project's sources
- Annotation: children Vec owns Rc children downward; parent link is Weak (section.rs:27), so no cycle; whole tree dropped when Parsed goes out of scope in Service::handle. Peak RSS flat: 12984KB@1k, 13036KB@20k, 13024KB@80k docs.
- Reason: tool_missing
- Remedy: Installed dla lacks harness/sample/trend/gap/test/confirm commands; run them to formally refute. Manual check: peak RSS flat at ~13MB for 1k/20k/80k docs.
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
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | outline-service Rust binary |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E2 | from task statement / runner consent / run init |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E2 | from task statement / runner consent / run init |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E2 | from task statement / runner consent / run init |
| `intake.build` | Does the project build locally? | answered | E2 | from task statement / runner consent / run init |
| `intake.workload` | What can exercise the suspected path? | answered | E2 | from task statement / runner consent / run init |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | from task statement / runner consent / run init |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E2 | from task statement / runner consent / run init |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | — |
| E2 | run-config | static | dla 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | probe/probe.json |
| E4 | static-hit | static | dla 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | sweep/dla.json |
| E5 | census-holder | static | dla-census 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | census/rust.json |
| E6 | workload | static | dla 0.1.0 | repo 8d5de014b949d83409b892376c40e19035c6f18d | workload/workloads.json |
