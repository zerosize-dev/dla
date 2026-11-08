# DLA report

Run: `20261108-201000-7e69`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 5 enumerated, 5 dispositioned; confirmed 0, strong 0, refuted 2, suspected 0, unresolved 3

Residual: not measured

Coverage: go 14080 iterations, 16 of 19 entry points exercised

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: discriminator, static
- Holder: container-field
- Owner: `internal.notify.render.Renderer.layouts`
- Region: heap
- Site: `internal/notify/render.go:10`
- Reason: not_exercised
- Remedy: alloc-test heap_live stayed inconclusive (E16,E18); Renderer.layouts is copied once in NewRenderer and never written afterwards, so no growth path exists in source. Rerun with --extra 64 or a longer replay with a heap profile if heap growth is still seen.
- Supporting evidence: E8, E18

### F2.n1: Hypothesis N-WARM: goroutine count is warm-up only (UNCLASSIFIED)

- State: unresolved(not_exercised)
- Evidence classes: discriminator
- Sibling hypothesis of: F2
- Hypothesis: N-WARM: goroutine count is warm-up only
- Refuted if: growth continues over the extra cycles
- Reason: not_exercised
- Remedy: Non-leak sibling; supported by plateau discriminator E15. No action needed unless production shows growth the replay does not reproduce.
- Supporting evidence: E15

### F3.n1: Hypothesis N-INTENT: immutable topic list (UNCLASSIFIED)

- State: unresolved(not_exercised)
- Evidence classes: discriminator
- Sibling hypothesis of: F3
- Hypothesis: N-INTENT: immutable topic list
- Refuted if: growth continues over the extra cycles
- Reason: not_exercised
- Remedy: Non-leak sibling; supported by plateau discriminator E17. No action needed unless production shows growth the replay does not reproduce.
- Supporting evidence: E17

## Ruled out

- **F2** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (non-leak) E15
  - Contradicting evidence: E15
  - Originally supported by: E8
- **F3** (UNCLASSIFIED): discriminator E17
  - Ruled out by: plateau+purge (non-leak) E17
  - Contradicting evidence: E17
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
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: points 1 to 9 come from S1; the extra points from S6
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: points 1 to 9 come from S1; the extra points from S8
- E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E17: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E17: limitation: gate and plateau thresholds are provisional (not calibrated)
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: points 1 to 9 come from S1; the extra points from S10
- E18: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E18: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E18: limitation: gate and plateau thresholds are provisional (not calibrated)

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
| E11 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (2c8026360ddd) | harness/H1/run-S1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S1.json (f4302041a14a) | trend/T1.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (f4302041a14a) | trend/T1.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S3.json (4ba04d18f237) | alloc/A1.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S5.json (96335ea98e78) | alloc/A2.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S7.json (32ba7bfcb89b) | alloc/A3.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S9.json (804618b0eeb5) | alloc/A4.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S11.json (e28ac068b731) | alloc/A5.json |
