# DLA report

Run: `20261108-194322-4804`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 3 enumerated, 3 dispositioned; confirmed 0, strong 0, refuted 2, suspected 0, unresolved 1

Residual: not measured

Coverage: go 10560 iterations, 0 of 19 entry points exercised

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `internal.notify.render.Renderer.layouts`
- Region: heap
- Site: `internal/notify/render.go:10`
- Annotation: Renderer.layouts is written only in NewRenderer (no writer after construction). go.heap_live drift is ~2 B/request (~20KB over 11000 requests), os.rss plateaus, goroutines flat at 3, threads 6-7; not a holder of the reported growth.
- Reason: not_exercised
- Remedy: Heap drift of ~2 B/request is runtime noise, not a holder; the reported goroutine/thread growth was not reproduced by the sequential replay. Capture a goroutine profile (pprof debug=2) or dla sample from the live pod to see which created-by site grows; the sink/real delivery or concurrent/error paths in production are not covered here.
- Supporting evidence: E8

## Ruled out

- **F2** (UNCLASSIFIED): discriminator E18
  - Ruled out by: plateau+purge (non-leak) E18
  - Contradicting evidence: E18
  - Originally supported by: E8
- **F3** (UNCLASSIFIED): discriminator E19
  - Ruled out by: plateau+purge (non-leak) E19
  - Contradicting evidence: E19
  - Originally supported by: E8

## Blind spots and unknowns

### Coverage gaps

None recorded.

### Runtime blind spots

None recorded.

### Entry points not exercisable

- go: `internal/notify/hub.go::Hub.Opened`: no_harness
- go: `internal/workload/workload.go::NewRNG`: no_harness

### Evidence limitations and unknowns

- E8: partial coverage
- E8: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E8: limitation: writers and removers are textual name matches
- E9: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E9: limitation: workloads were discovered by reading files; none was run
- E10: limitation: harness written; nothing was run (dla harness run measures it)
- E10: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E10: limitation: goleak not available offline: the goleak check is skipped
- E11: partial coverage (9 of 9 examined)
- E11: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E11: limitation: no Go coverage profile was written to cov/
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: goleak not available offline: the goleak check is skipped
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E17: limitation: goleak not available offline: the goleak check is skipped
- E17: limitation: gate and plateau thresholds are provisional (not calibrated)
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E18: limitation: goleak not available offline: the goleak check is skipped
- E18: limitation: gate and plateau thresholds are provisional (not calibrated)
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E19: limitation: goleak not available offline: the goleak check is skipped
- E19: limitation: gate and plateau thresholds are provisional (not calibrated)
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E20: limitation: goleak not available offline: the goleak check is skipped
- E20: limitation: gate and plateau thresholds are provisional (not calibrated)
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: gate thresholds are provisional (not calibrated)

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
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 23911e0074f1c23bd2fa28d182500e99d68a2f3b | — |
| E2 | run-config | static | dla 0.1.0 | repo 23911e0074f1c23bd2fa28d182500e99d68a2f3b | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 23911e0074f1c23bd2fa28d182500e99d68a2f3b | probe/probe.json |
| E4 | static-hit | static | staticcheck 2025.1.1 | repo 23911e0074f1c23bd2fa28d182500e99d68a2f3b | sweep/go-staticcheck.json |
| E5 | static-hit | static | golangci-lint 2.5.0 | repo 23911e0074f1c23bd2fa28d182500e99d68a2f3b | sweep/go-golangci-lint.json |
| E6 | static-hit | static | govet 1.24.7 | repo 23911e0074f1c23bd2fa28d182500e99d68a2f3b | sweep/go-govet.json |
| E7 | static-hit | static | dla 0.1.0 | repo 23911e0074f1c23bd2fa28d182500e99d68a2f3b | sweep/go-dla.json |
| E8 | census-holder | static | dla-census 0.1.0 | repo 23911e0074f1c23bd2fa28d182500e99d68a2f3b | census/go.json |
| E9 | workload | static | dla 0.1.0 | repo 23911e0074f1c23bd2fa28d182500e99d68a2f3b | workload/workloads.json |
| E10 | workload | static | dla 0.1.0 | repo 23911e0074f1c23bd2fa28d182500e99d68a2f3b | harness/H1/harness.json |
| E11 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (188d38cc41d3) | harness/H1/run-S1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S1.json (315c4466064e) | trend/T1.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (315c4466064e) | trend/T1.json |
| E14 | growth-series | test | dla 0.1.0 | artifact series/S2.json (99d90b28934f) | trend/T2.json |
| E15 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (99d90b28934f) | trend/T2.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S3.json (379bf32e5172) | alloc/A1.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S4.json (f1b9e1a2bfb9) | alloc/A2.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S5.json (f28f3984b8ae) | alloc/A3.json |
| E19 | discriminator | test | dla 0.1.0 | artifact series/S6.json (8f340445073c) | alloc/A4.json |
| E20 | discriminator | test | dla 0.1.0 | artifact series/S7.json (26ee2e9aac89) | alloc/A5.json |
| E21 | growth-series | test | dla 0.1.0 | artifact series/S7.json (26ee2e9aac89) | trend/T8.json |
| E22 | unit-count-series | test | dla 0.1.0 | artifact series/S7.json (26ee2e9aac89) | trend/T8.json |
