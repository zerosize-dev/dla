# DLA report

Run: `20261107-194630-a576`

Status: COMPLETE

Findings: 2 enumerated, 2 dispositioned; confirmed 0, strong 0, refuted 0, suspected 2, unresolved 0

Residual: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: suspected_static_only(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.rollup.Rollup.samples`
- Region: heap
- Site: `src/rollup.rs:17`
- Writers: `src/rollup.rs:28`
- Removers: `src/rollup.rs:57`
- Annotation: heuristic name match over the project's sources
- Annotation: Vec::clear keeps capacity; 60k-sample burst (feed.rs:166) leaves ~65536*40B buffer allocated
- Reason: tool_missing
- Remedy: installed dla lacks gate/harness/sample/trend/confirm; manual /proc RSS run showed one-time step at batch 11 then flat (high-water mark, N-HWM), not unbounded growth
- Supporting evidence: E5

### F2: **UNCLASSIFIED**

- State: suspected_static_only(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.rollup.Rollup.stats`
- Region: heap
- Site: `src/rollup.rs:18`
- Writers: `src/rollup.rs:34`
- Removers: `src/rollup.rs:58`
- Annotation: heuristic name match over the project's sources
- Annotation: HashMap::clear keeps bucket table; burst with up to 60k distinct series keeps table sized for it
- Reason: tool_missing
- Remedy: installed dla lacks gate/harness/sample/trend/confirm; manual /proc RSS run showed one-time step at batch 11 then flat (high-water mark, N-HWM), not unbounded growth
- Supporting evidence: E5

## Ruled out

None.

## Blind spots and unknowns

### Coverage gaps

- rust/rapx: blocked (cannot copy the project: Permission denied (os error 13))

### Runtime blind spots

None recorded.

### Entry points not exercisable

- rust: `src/sample.rs::SampleError.fmt`: no_harness

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
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | rollup-agent Rust crate, src/ |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | whole crate |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | RSS few MB -> >10MB soon after start, stays high when quiet |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | rust |
| `intake.build` | Does the project build locally? | answered | E1 | cargo, no deps |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | main --iterations N with deterministic feed |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 3600s |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 77a18cd734c529473e4421f614485d13cba99d33 | — |
| E2 | run-config | static | dla 0.1.0 | repo 77a18cd734c529473e4421f614485d13cba99d33 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 77a18cd734c529473e4421f614485d13cba99d33 | probe/probe.json |
| E4 | static-hit | static | dla 0.1.0 | repo 77a18cd734c529473e4421f614485d13cba99d33 | sweep/dla.json |
| E5 | census-holder | static | dla-census 0.1.0 | repo 77a18cd734c529473e4421f614485d13cba99d33 | census/rust.json |
| E6 | workload | static | dla 0.1.0 | repo 77a18cd734c529473e4421f614485d13cba99d33 | workload/workloads.json |
