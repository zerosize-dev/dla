# DLA report

Run: `20261107-194826-e6ba`

Status: COMPLETE

Findings: 4 enumerated, 4 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 4

Residual: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Site: `src/section.rs:10`
- Suggested archetype (unconfirmed): L-CYCLE
- Reason: tool_missing
- Remedy: Installed dla lacks harness/sample/trend/gate/confirm/summarize. Manual measure: maxrss 13016/27996/104828 KB at 1000/4000/16000 docs (linear ~6KB/doc). Repair class: make Section.parent a Weak<Section> (or break links on drop) so the cycle is released.
- Supporting evidence: E4

### F2: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Site: `src/section.rs:11`
- Suggested archetype (unconfirmed): L-CYCLE
- Reason: tool_missing
- Remedy: Installed dla lacks harness/sample/trend/gate/confirm/summarize. Manual measure: maxrss 13016/27996/104828 KB at 1000/4000/16000 docs (linear ~6KB/doc). Repair class: make Section.parent a Weak<Section> (or break links on drop) so the cycle is released.
- Supporting evidence: E4

### F3: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: cycle-only
- Owner: `src.section.Section.parent`
- Region: heap
- Site: `src/section.rs:10`
- Writers: `src/section.rs:27`
- Annotation: Section::append stores a strong Rc to the parent in child.parent; parent.children holds strong Rc to child => Rc cycle. No remover/Drop/Weak anywhere; parser.rs:53 calls append for every heading, toc.rs:22 only reads. Root Rc dropped at end of Service::handle but cycle keeps every Section alive.
- Reason: tool_missing
- Remedy: Installed dla lacks harness/sample/trend/gate/confirm/summarize. Manual measure: maxrss 13016/27996/104828 KB at 1000/4000/16000 docs (linear ~6KB/doc). Repair class: make Section.parent a Weak<Section> (or break links on drop) so the cycle is released.
- Supporting evidence: E5

### F4: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: cycle-only
- Owner: `src.section.Section.children`
- Region: heap
- Site: `src/section.rs:11`
- Writers: `src/section.rs:28`
- Annotation: heuristic name match over the project's sources
- Annotation: children Vec holds strong Rc; other half of cycle with F3. Never cleared.
- Reason: tool_missing
- Remedy: Installed dla lacks harness/sample/trend/gate/confirm/summarize. Manual measure: maxrss 13016/27996/104828 KB at 1000/4000/16000 docs (linear ~6KB/doc). Repair class: make Section.parent a Weak<Section> (or break links on drop) so the cycle is released.
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
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | outline-service Rust binary in project/ |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | src/*.rs, 6 files, no deps |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | RSS grows with documents processed; pod restarted every few days |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | rust only |
| `intake.build` | Does the project build locally? | answered | E1 | cargo build |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | binary --iterations N with deterministic documents |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4, raw values |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 60m |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | — |
| E2 | run-config | static | dla 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | probe/probe.json |
| E4 | static-hit | static | dla 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | sweep/dla.json |
| E5 | census-holder | static | dla-census 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | census/rust.json |
| E6 | workload | static | dla 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | workload/workloads.json |
