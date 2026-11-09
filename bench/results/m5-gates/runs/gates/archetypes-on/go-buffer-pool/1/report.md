# DLA report

Run: `20261109-194257-5ff3`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 5 enumerated, 5 dispositioned; confirmed 0, strong 0, refuted 4, suspected 0, unresolved 1

Residual: not measured

Coverage: go 3520 iterations, 9 of 11 entry points exercised

## Findings

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: discriminator, static
- Holder: static/global
- Owner: `internal.workload.workload.skus`
- Region: heap
- Site: `internal/workload/workload.go:27`
- Reason: not_exercised
- Remedy: F4 is the read-only package var skus (no writer). alloc-test's 'leak' verdict rests on ~10KB heap_live noise with no passing trend column tied to it; no action needed unless a larger run shows growth
- Supporting evidence: E8, E20

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E17
  - Ruled out by: plateau+purge (non-leak) E17
  - Contradicting evidence: E17
  - Originally supported by: E8
- **F2** (UNCLASSIFIED): discriminator E18
  - Ruled out by: plateau+purge (non-leak) E18
  - Contradicting evidence: E18
  - Originally supported by: E8
- **F3** (UNCLASSIFIED): discriminator E19
  - Ruled out by: plateau+purge (non-leak) E19
  - Contradicting evidence: E19
  - Originally supported by: E8
- **F5** (UNCLASSIFIED): discriminator E21
  - Ruled out by: plateau+purge (non-leak) E21
  - Contradicting evidence: E21
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
- E14: limitation: a Go heap profile is a sample (runtime.MemProfileRate): sizes are estimates, and objects smaller than the rate can be missed
- E14: limitation: in-use figures are as of the last completed collection: write the profile after runtime.GC()
- E14: limitation: a profile carries no holder: a stack ties to a finding only through its first project frame being the finding's site or writer
- E14: limitation: the profile is sampled: counts and bytes are estimates
- E15: limitation: a Go heap profile is a sample (runtime.MemProfileRate): sizes are estimates, and objects smaller than the rate can be missed
- E15: limitation: in-use figures are as of the last completed collection: write the profile after runtime.GC()
- E15: limitation: a profile carries no holder: a stack ties to a finding only through its first project frame being the finding's site or writer
- E15: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: points 1 to 9 come from S1; the extra points from S3
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: points 1 to 9 come from S1; the extra points from S5
- E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E17: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E17: limitation: gate and plateau thresholds are provisional (not calibrated)
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: points 1 to 9 come from S1; the extra points from S7
- E18: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E18: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E18: limitation: gate and plateau thresholds are provisional (not calibrated)
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: points 1 to 9 come from S1; the extra points from S9
- E19: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E19: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E19: limitation: gate and plateau thresholds are provisional (not calibrated)
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: points 1 to 9 come from S1; the extra points from S11
- E20: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E20: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E20: limitation: gate and plateau thresholds are provisional (not calibrated)
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: points 1 to 9 come from S1; the extra points from S13
- E21: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E21: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E21: limitation: gate and plateau thresholds are provisional (not calibrated)

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
| E11 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (4697d6ccb956) | harness/H1/run-S1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S1.json (effcd189773e) | trend/T1.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (effcd189773e) | trend/T1.json |
| E14 | alloc-stack | live | go-pprof-heap 0.1.0 | artifact allocs/A1.json (5fa105aaa77b) | allocs/A1.json |
| E15 | growth-series | live | go-pprof-heap 0.1.0 | artifact series/S2.json (9884105fa4ff) | series/S2.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S4.json (1a4f65f1058f) | alloc/A1.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S6.json (6bbb4c4512df) | alloc/A2.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S8.json (36d24883120a) | alloc/A3.json |
| E19 | discriminator | test | dla 0.1.0 | artifact series/S10.json (0b347bbab574) | alloc/A4.json |
| E20 | discriminator | test | dla 0.1.0 | artifact series/S12.json (3bd1af7d504e) | alloc/A5.json |
| E21 | discriminator | test | dla 0.1.0 | artifact series/S14.json (d7395d7c732f) | alloc/A6.json |
