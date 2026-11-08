# DLA report

Run: `20261108-201840-b9cc`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 6 enumerated, 6 dispositioned; confirmed 0, strong 0, refuted 5, suspected 0, unresolved 1

Residual: not measured

Coverage: go 7040 iterations, 9 of 11 entry points exercised

## Findings

### F3.n1: Hypothesis N-HWM: pool free list sits at a high-water mark set by the largest batch (UNCLASSIFIED)

- State: unresolved(not_exercised)
- Evidence classes: discriminator
- Sibling hypothesis of: F3
- Hypothesis: N-HWM: pool free list sits at a high-water mark set by the largest batch
- Refuted if: go.heap_live keeps growing across extra cycles
- Reason: not_exercised
- Remedy: N-HWM sibling is the supported explanation (E20 favors non-leak) and cannot be refuted; no further measurement needed. Action: cap pool size/buffer capacity on Put.
- Supporting evidence: E20

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E21
  - Ruled out by: plateau+purge (non-leak) E21
  - Contradicting evidence: E21
  - Originally supported by: E8
- **F2** (UNCLASSIFIED): discriminator E22
  - Ruled out by: plateau+purge (non-leak) E22
  - Contradicting evidence: E22
  - Originally supported by: E8
- **F3** (UNCLASSIFIED): discriminator E20
  - Ruled out by: plateau+purge (non-leak) E20
  - Contradicting evidence: E20
  - Originally supported by: E8
- **F4** (UNCLASSIFIED): discriminator E23
  - Ruled out by: plateau+purge (non-leak) E23
  - Contradicting evidence: E23
  - Originally supported by: E8
- **F5** (UNCLASSIFIED): discriminator E24
  - Ruled out by: plateau+purge (non-leak) E24
  - Contradicting evidence: E24
  - Originally supported by: E8

## Blind spots and unknowns

### Coverage gaps

None recorded.

### Runtime blind spots

None recorded.

### Entry points not exercisable

None recorded.

### Evidence limitations and unknowns

- E8: partial coverage
- E8: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E8: limitation: writers and removers are textual name matches
- E9: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E9: limitation: workloads were discovered by reading files; none was run
- E10: limitation: harness written; nothing was run (dla harness run measures it)
- E10: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E10: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E11: partial coverage (9 of 9 examined)
- E11: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: points 1 to 9 come from S1; the extra points from S2
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: points 1 to 9 come from S1; the extra points from S4
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: harness written; nothing was run (dla harness run measures it)
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: goleak not available offline: the goleak check is skipped
- E17: partial coverage (9 of 9 examined)
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: no Go coverage profile was written to cov/
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E20: limitation: goleak not available offline: the goleak check is skipped
- E20: limitation: gate and plateau thresholds are provisional (not calibrated)
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E21: limitation: goleak not available offline: the goleak check is skipped
- E21: limitation: gate and plateau thresholds are provisional (not calibrated)
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E22: limitation: goleak not available offline: the goleak check is skipped
- E22: limitation: gate and plateau thresholds are provisional (not calibrated)
- E23: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E23: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E23: limitation: goleak not available offline: the goleak check is skipped
- E23: limitation: gate and plateau thresholds are provisional (not calibrated)
- E24: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E24: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E24: limitation: goleak not available offline: the goleak check is skipped
- E24: limitation: gate and plateau thresholds are provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo dd879aa91b6b7202ad9d226ebe261d07e9c68353 | — |
| E2 | run-config | static | dla 0.1.0 | repo dd879aa91b6b7202ad9d226ebe261d07e9c68353 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo dd879aa91b6b7202ad9d226ebe261d07e9c68353 | probe/probe.json |
| E4 | static-hit | static | staticcheck 2025.1.1 | repo dd879aa91b6b7202ad9d226ebe261d07e9c68353 | sweep/go-staticcheck.json |
| E5 | static-hit | static | golangci-lint 2.5.0 | repo dd879aa91b6b7202ad9d226ebe261d07e9c68353 | sweep/go-golangci-lint.json |
| E6 | static-hit | static | govet 1.24.7 | repo dd879aa91b6b7202ad9d226ebe261d07e9c68353 | sweep/go-govet.json |
| E7 | static-hit | static | dla 0.1.0 | repo dd879aa91b6b7202ad9d226ebe261d07e9c68353 | sweep/go-dla.json |
| E8 | census-holder | static | dla-census 0.1.0 | repo dd879aa91b6b7202ad9d226ebe261d07e9c68353 | census/go.json |
| E9 | workload | static | dla 0.1.0 | repo dd879aa91b6b7202ad9d226ebe261d07e9c68353 | workload/workloads.json |
| E10 | workload | static | dla 0.1.0 | repo dd879aa91b6b7202ad9d226ebe261d07e9c68353 | harness/H1/harness.json |
| E11 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (7a1a4d95ba6f) | harness/H1/run-S1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S1.json (d62cb207f6af) | trend/T1.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (d62cb207f6af) | trend/T1.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S3.json (54b4de774f6a) | alloc/A1.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S5.json (cd519f31661e) | alloc/A2.json |
| E16 | workload | static | dla 0.1.0 | repo dd879aa91b6b7202ad9d226ebe261d07e9c68353 | harness/H2/harness.json |
| E17 | workload | test | dla 0.1.0 | artifact harness/H2/harness.json (0e7b5a2a8a28) | harness/H2/run-S6.json |
| E18 | growth-series | test | dla 0.1.0 | artifact series/S6.json (8d466eb375b2) | trend/T4.json |
| E19 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (8d466eb375b2) | trend/T4.json |
| E20 | discriminator | test | dla 0.1.0 | artifact series/S8.json (8efaa72ef4eb) | alloc/A3.json |
| E21 | discriminator | test | dla 0.1.0 | artifact series/S9.json (7d51c2b93cec) | alloc/A4.json |
| E22 | discriminator | test | dla 0.1.0 | artifact series/S10.json (cdc3776a1da9) | alloc/A5.json |
| E23 | discriminator | test | dla 0.1.0 | artifact series/S11.json (16f475a9b4c8) | alloc/A6.json |
| E24 | discriminator | test | dla 0.1.0 | artifact series/S12.json (537e45d95600) | alloc/A7.json |
