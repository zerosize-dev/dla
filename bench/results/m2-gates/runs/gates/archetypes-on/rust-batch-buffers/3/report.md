# DLA report

Run: `20261107-195219-006c`

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
- Annotation: Vec::clear keeps capacity; batch 11 (60000 samples) leaves the buffer at high-water size. Scratch-copy run: RSS 3.3MB at seq10, 11.9MB after seq11 flush, flat to seq30; with shrink_to_fit after clear 6.3MB.
- Reason: tool_missing
- Remedy: Installed dla lacks gate/harness/sample/trend/ledger confirm; classification not machine-confirmed. Likely pool high-water mark (N-HWM), not a leak: bounded, no growth after batch 11. Fix class: release capacity after a burst (shrink_to_fit / reallocate).
- Supporting evidence: E5

### F2: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.rollup.Rollup.stats`
- Region: heap
- Site: `src/rollup.rs:18`
- Writers: `src/rollup.rs:20`
- Removers: `src/rollup.rs:58`
- Annotation: heuristic name match over the project's sources
- Annotation: HashMap::clear keeps bucket table; up to ~60k series in batch 11 retained at capacity. Same scratch measurement, shrink_to_fit on both containers drops retained RSS by ~5.6MB; stats alone not isolated.
- Reason: tool_missing
- Remedy: Same as F1; share of retained memory vs F1 not isolated. Fix class: shrink map capacity after burst.
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
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | rollup-agent Rust binary in project root |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | src/*.rs, 5 files |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | RSS from few MB to >10MB shortly after start, stays high when quiet |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | rust |
| `intake.build` | Does the project build locally? | answered | E1 | cargo, no deps |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | binary takes the flag iterations N with built-in feed |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 raw values |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 60m |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 77a18cd734c529473e4421f614485d13cba99d33 | — |
| E2 | run-config | static | dla 0.1.0 | repo 77a18cd734c529473e4421f614485d13cba99d33 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 77a18cd734c529473e4421f614485d13cba99d33 | probe/probe.json |
| E4 | static-hit | static | dla 0.1.0 | repo 77a18cd734c529473e4421f614485d13cba99d33 | sweep/dla.json |
| E5 | census-holder | static | dla-census 0.1.0 | repo 77a18cd734c529473e4421f614485d13cba99d33 | census/rust.json |
| E6 | workload | static | dla 0.1.0 | repo 77a18cd734c529473e4421f614485d13cba99d33 | workload/workloads.json |
