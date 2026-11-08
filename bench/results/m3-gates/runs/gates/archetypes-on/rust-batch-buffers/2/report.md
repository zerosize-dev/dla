# DLA report

Run: `20261108-125402-dc73`

Status: COMPLETE

Findings: 4 enumerated, 4 dispositioned; confirmed 2, strong 0, refuted 2, suspected 0, unresolved 0

Residual: not measured

Coverage: rust 880 iterations, 0 of 10 entry points exercised

## Findings

### F1.n1: Hypothesis N-HWM: Vec<Sample> keeps burst capacity (pool/high-water mark), not a leak (UNCLASSIFIED)

- State: confirmed
- Explained by N-HWM (discriminator E13)
- Evidence classes: context, discriminator
- Sibling hypothesis of: F1
- Hypothesis: N-HWM: Vec<Sample> keeps burst capacity (pool/high-water mark), not a leak
- Refuted if: live bytes stay near zero between batches and RSS drops after purge
- Supporting evidence: E9, E13

### F2.n1: Hypothesis N-HWM: HashMap keeps bucket capacity after clear() (UNCLASSIFIED)

- State: confirmed
- Explained by N-HWM (discriminator E14)
- Evidence classes: context, discriminator
- Sibling hypothesis of: F2
- Hypothesis: N-HWM: HashMap keeps bucket capacity after clear()
- Refuted if: growth with iterations
- Supporting evidence: E9, E14

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (non-leak) E13
  - Contradicting evidence: E13
  - Originally supported by: E6, E8
- **F2** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (non-leak) E14
  - Contradicting evidence: E14
  - Originally supported by: E6, E8

## Blind spots and unknowns

### Coverage gaps

- rust/rapx: blocked (cannot copy the project: Permission denied (os error 13))

### Runtime blind spots

- rust: the allocator shim does not record which entry points ran

### Entry points not exercisable

- rust: `src/sample.rs::SampleError.fmt`: no_harness

### Evidence limitations and unknowns

- E6: partial coverage
- E6: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E6: limitation: writers and removers are textual name matches
- E7: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E7: limitation: workloads were discovered by reading files; none was run
- E8: limitation: harness written; nothing was run (dla harness run measures it)
- E8: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E9: partial coverage (9 of 9 examined)
- E9: limitation: command mode has one key mode (native): the requested fresh/repeated modes only exist in driver mode
- E9: limitation: 544 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: exercised entry points are not measured for rust: the allocator shim sees allocations, not calls
- E10: limitation: 544 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: 544 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: 544 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: 544 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: points 1 to 9 come from S1; the extra points from S3
- E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- E14: limitation: 544 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: points 1 to 9 come from S1; the extra points from S5
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | rollup-agent (rust binary), src/ |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | whole crate |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | RSS from few MB to >10MB shortly after start, stays high when idle |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E3 | rust only |
| `intake.build` | Does the project build locally? | answered | E4 | cargo build ok |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | main --iterations N |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4, raw values |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 3600s |

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
| E8 | workload | static | dla 0.1.0 | repo 77a18cd734c529473e4421f614485d13cba99d33 | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (ed4994ce8493) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (3f249e56eef6) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (3f249e56eef6) | trend/T1.json |
| E12 | non-heap | test | dla 0.1.0 | artifact series/S2.json (58bd2e558c4c) | trend/T2.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S4.json (4467a8f29e94) | alloc/A1.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S6.json (e88a9f1a706f) | alloc/A2.json |
