# DLA report

Run: `20261109-204141-dbbd`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 4 enumerated, 4 dispositioned; confirmed 1, strong 0, refuted 1, suspected 2, unresolved 0

Residual: not measured

Coverage: go 3520 iterations, 15 of 19 entry points exercised

## Findings

### F2: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `group:goroutine/created:example.com/notifyd/internal/notify.newSubscription` 1173 count/dump (steady, gate pass, E15)
- Evidence classes: context, growth, static
- Holder: blocked-stack
- Owner: `internal.notify.subscription.newSubscription -> s.run()`
- Region: non-heap
- Shape: steady
- Site: `internal/notify/subscription.go:33`
- Writers: `internal/notify/handler.go:17`
- Annotation: Handler.Serve opens a Subscription (newSubscription starts the Subscription.run goroutine, 4KB scratch buffer) and never calls sub.Close(); nothing else removes it, so each request leaves one goroutine blocked in select plus its buffer. Missing pairing: Hub.Open -> Subscription.Close (also skipped on Push/Flush error returns).
- Sibling hypotheses: F2.n1
- Supporting evidence: E8, E11, E15, E17, E12

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `internal.notify.render.Renderer.layouts`
- Region: heap
- Site: `internal/notify/render.go:10`
- Reason: not_exercised
- Remedy: No probe or grown column tied to this holder (read-only config populated once at startup); add a driver-mode probe if doubt remains
- Supporting evidence: E8

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `internal.workload.workload.topics`
- Region: heap
- Site: `internal/workload/workload.go:19`
- Reason: not_exercised
- Remedy: No probe or grown column tied to this holder (read-only config populated once at startup); add a driver-mode probe if doubt remains
- Supporting evidence: E8

## Ruled out

- **F2.n1** (UNCLASSIFIED): discriminator E18
  - Ruled out by: plateau+purge (leak) E18
  - Hypothesis: N-WARM: growth is a warm-up cache that plateaus
  - Refuted if: goroutine count keeps growing over extra cycles
  - Contradicting evidence: E18
  - Originally supported by: E13

## Blind spots and unknowns

### Coverage gaps

- go/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- go/goroutineleak: blocked (the goroutineleak profile needs Go 1.27 or newer (go1.24.7 here) and GOTOOLCHAIN=local forbids switching to go1.27.0)
- go/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)

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
- E11: limitation: goroutineleak: blocked (the goroutineleak profile needs Go 1.27 or newer (go1.24.7 here) and GOTOOLCHAIN=local forbids switching to go1.27.0)
- E11: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: goroutines are grouped by the `created by` frame: a group ties to a finding only when that go statement is the finding's site or a recorded writer
- E14: limitation: goroutines without a creator (main, runtime) are not grouped; wait reasons and totals never tie
- E15: limitation: goroutines are grouped by the `created by` frame: a group ties to a finding only when that go statement is the finding's site or a recorded writer
- E15: limitation: goroutines without a creator (main, runtime) are not grouped; wait reasons and totals never tie
- E15: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E16: limitation: a Go heap profile is a sample (runtime.MemProfileRate): sizes are estimates, and objects smaller than the rate can be missed
- E16: limitation: in-use figures are as of the last completed collection: write the profile after runtime.GC()
- E16: limitation: a profile carries no holder: a stack ties to a finding only through its first project frame being the finding's site or writer
- E16: limitation: the profile is sampled: counts and bytes are estimates
- E17: limitation: a Go heap profile is a sample (runtime.MemProfileRate): sizes are estimates, and objects smaller than the rate can be missed
- E17: limitation: in-use figures are as of the last completed collection: write the profile after runtime.GC()
- E17: limitation: a profile carries no holder: a stack ties to a finding only through its first project frame being the finding's site or writer
- E17: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: points 1 to 9 come from S1; the extra points from S4
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
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | — |
| E2 | run-config | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | probe/probe.json |
| E4 | static-hit | static | staticcheck 2025.1.1 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | sweep/go-staticcheck.json |
| E5 | static-hit | static | golangci-lint 2.5.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | sweep/go-golangci-lint.json |
| E6 | static-hit | static | govet 1.24.7 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | sweep/go-govet.json |
| E7 | static-hit | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | sweep/go-dla.json |
| E8 | census-holder | static | dla-census 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | census/go.json |
| E9 | workload | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | workload/workloads.json |
| E10 | workload | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | harness/H1/harness.json |
| E11 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (519e4f90ff61) | harness/H1/run-S1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S1.json (868570ce0927) | trend/T1.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (868570ce0927) | trend/T1.json |
| E14 | non-heap | live | go-goroutine 0.1.0 | artifact nonheap/N1.json (d688811fbec2) | nonheap/N1.json |
| E15 | growth-series | live | go-goroutine 0.1.0 | artifact series/S2.json (82e90f80e31b) | series/S2.json |
| E16 | alloc-stack | live | go-pprof-heap 0.1.0 | artifact allocs/A1.json (dccf8d345f9b) | allocs/A1.json |
| E17 | growth-series | live | go-pprof-heap 0.1.0 | artifact series/S3.json (f60fb5b8b3a9) | series/S3.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S5.json (d15ca16f8160) | alloc/A1.json |
