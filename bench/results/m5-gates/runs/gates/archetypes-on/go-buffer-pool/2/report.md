# DLA report

Run: `20261109-201205-9aee`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 5 enumerated, 5 dispositioned; confirmed 0, strong 0, refuted 5, suspected 0, unresolved 0

Residual: not measured

Coverage: go 3520 iterations, 9 of 11 entry points exercised

## Findings

No findings to report.

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (non-leak) E16
  - Contradicting evidence: E16
  - Originally supported by: E8
- **F2** (UNCLASSIFIED): discriminator E23
  - Ruled out by: plateau+purge (non-leak) E23
  - Contradicting evidence: E23
  - Originally supported by: E8
- **F3** (UNCLASSIFIED): discriminator E18
  - Ruled out by: plateau+purge (non-leak) E18
  - Contradicting evidence: E18
  - Originally supported by: E8
- **F4** (UNCLASSIFIED): discriminator E19
  - Ruled out by: plateau+purge (non-leak) E19
  - Contradicting evidence: E19
  - Originally supported by: E8
- **F5** (UNCLASSIFIED): discriminator E20
  - Ruled out by: plateau+purge (non-leak) E20
  - Contradicting evidence: E20
  - Originally supported by: E8

## Blind spots and unknowns

### Coverage gaps

- go/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- go/goroutineleak: blocked (the goroutineleak profile needs Go 1.27 or newer (go1.24.7 here) and GOTOOLCHAIN=local forbids switching to go1.27.0)
- go/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)

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
- E11: limitation: goroutineleak: blocked (the goroutineleak profile needs Go 1.27 or newer (go1.24.7 here) and GOTOOLCHAIN=local forbids switching to go1.27.0)
- E11: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: gap.unexplained is negative at 4 point(s): the accounted figure exceeds os.rss (reserved but untouched memory)
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: points 1 to 9 come from S1; the extra points from S3
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: points 1 to 9 come from S1; the extra points from S5
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: points 1 to 9 come from S1; the extra points from S7
- E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E17: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E17: limitation: gate and plateau thresholds are provisional (not calibrated)
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: points 1 to 9 come from S1; the extra points from S9
- E18: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E18: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E18: limitation: gate and plateau thresholds are provisional (not calibrated)
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: points 1 to 9 come from S1; the extra points from S11
- E19: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E19: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E19: limitation: gate and plateau thresholds are provisional (not calibrated)
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: points 1 to 9 come from S1; the extra points from S13
- E20: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E20: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E20: limitation: gate and plateau thresholds are provisional (not calibrated)
- E21: limitation: a Go heap profile is a sample (runtime.MemProfileRate): sizes are estimates, and objects smaller than the rate can be missed
- E21: limitation: in-use figures are as of the last completed collection: write the profile after runtime.GC()
- E21: limitation: a profile carries no holder: a stack ties to a finding only through its first project frame being the finding's site or writer
- E21: limitation: the profile is sampled: counts and bytes are estimates
- E22: limitation: a Go heap profile is a sample (runtime.MemProfileRate): sizes are estimates, and objects smaller than the rate can be missed
- E22: limitation: in-use figures are as of the last completed collection: write the profile after runtime.GC()
- E22: limitation: a profile carries no holder: a stack ties to a finding only through its first project frame being the finding's site or writer
- E22: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E23: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E23: limitation: points 1 to 9 come from S1; the extra points from S16
- E23: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E23: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E23: limitation: gate and plateau thresholds are provisional (not calibrated)

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
| E11 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (798a674ada95) | harness/H1/run-S1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S1.json (bbdb2adaa225) | trend/T1.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (bbdb2adaa225) | trend/T1.json |
| E14 | non-heap | test | dla 0.1.0 | artifact series/S2.json (566fb2603203) | trend/T2.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S4.json (8e66caba296e) | alloc/A1.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S6.json (271143cc5be3) | alloc/A2.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S8.json (4fb4eccfde52) | alloc/A3.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S10.json (4dfa35b055da) | alloc/A4.json |
| E19 | discriminator | test | dla 0.1.0 | artifact series/S12.json (02c0786465c1) | alloc/A5.json |
| E20 | discriminator | test | dla 0.1.0 | artifact series/S14.json (232f3cf545b2) | alloc/A6.json |
| E21 | alloc-stack | live | go-pprof-heap 0.1.0 | artifact allocs/A1.json (125c0f409a01) | allocs/A1.json |
| E22 | growth-series | live | go-pprof-heap 0.1.0 | artifact series/S15.json (291beb240f4b) | series/S15.json |
| E23 | discriminator | test | dla 0.1.0 | artifact series/S17.json (a5260288efc0) | alloc/A7.json |
