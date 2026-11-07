# DLA report

Run: `20261107-194036-2e5f`

Status: COMPLETE

Findings: 2 enumerated, 2 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 2

Residual: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.rollup.Rollup.samples`
- Region: heap
- Site: `src/rollup.rs:17`
- Writers: `src/rollup.rs:28`
- Removers: `src/rollup.rs:57`
- Annotation: heuristic name match over the project's sources
- Annotation: samples Vec: clear() at flush drops len but keeps capacity; batch 11 (60,000 samples, feed.rs:15) sets capacity high-water mark. Manual measurement: RSS 3.3MB through seq 10, 11.95MB from seq 11, flat to seq 39. Bounded, not growing => high-water mark (N-HWM), not a leak.
- Reason: tool_missing
- Remedy: Installed dla lacks test/confirm/refute/gate/harness/alloc-test. Manual evidence says capacity high-water mark, not leak: after clear() use shrink_to_fit()/replace with Vec::new() when capacity greatly exceeds len, or process in bounded chunks.
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.rollup.Rollup.stats`
- Region: heap
- Site: `src/rollup.rs:18`
- Writers: `src/rollup.rs:35`
- Removers: `src/rollup.rs:58`
- Annotation: heuristic name match over the project's sources
- Annotation: stats HashMap: entry() at flush:35; clear() keeps buckets allocated; same HWM contributor (up to ~60k distinct series in batch 11). Bounded.
- Reason: tool_missing
- Remedy: Same as F1 for HashMap: shrink_to_fit() after clear() or recreate the map per batch.
- Supporting evidence: E6

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

- E6: partial coverage
- E6: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E6: limitation: writers and removers are textual name matches
- E7: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E7: limitation: workloads were discovered by reading files; none was run

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | from task args/repo (rust crate rollup-agent, unattended) |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | from task args/repo (rust crate rollup-agent, unattended) |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | from task args/repo (rust crate rollup-agent, unattended) |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | from task args/repo (rust crate rollup-agent, unattended) |
| `intake.build` | Does the project build locally? | answered | E1 | from task args/repo (rust crate rollup-agent, unattended) |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | from task args/repo (rust crate rollup-agent, unattended) |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | from task args/repo (rust crate rollup-agent, unattended) |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | from task args/repo (rust crate rollup-agent, unattended) |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 77a18cd734c529473e4421f614485d13cba99d33 | — |
| E2 | run-config | static | dla 0.1.0 | repo 77a18cd734c529473e4421f614485d13cba99d33 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 77a18cd734c529473e4421f614485d13cba99d33 | probe/probe.json |
| E4 | probe | static | dla-probe 0.1.0 | repo 77a18cd734c529473e4421f614485d13cba99d33 | probe/probe.json |
| E5 | static-hit | static | dla 0.1.0 | repo 77a18cd734c529473e4421f614485d13cba99d33 | sweep/dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo 77a18cd734c529473e4421f614485d13cba99d33 | census/rust.json |
| E7 | workload | static | dla 0.1.0 | repo 77a18cd734c529473e4421f614485d13cba99d33 | workload/workloads.json |
