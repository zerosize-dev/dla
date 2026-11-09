# DLA report

Run: `20261109-202028-6f55`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 4 enumerated, 4 dispositioned; confirmed 1, strong 0, refuted 1, suspected 2, unresolved 0

Residual: not measured

Coverage: go 7040 iterations, 0 of 0 entry points exercised

## Findings

### F2: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `group:goroutine/created:example.com/notifyd/internal/notify.newSubscription` 1173 count/dump (steady, gate pass, E14)
- Evidence classes: context, discriminator, growth, static
- Holder: blocked-stack
- Owner: `internal.notify.subscription.newSubscription -> s.run()`
- Region: non-heap
- Shape: steady
- Site: `internal/notify/subscription.go:33`
- Writers: `internal/notify/handler.go:15`
- Annotation: Handler.Serve opens a Subscription via Hub.Open (starts goroutine s.run at subscription.go:33) and never calls sub.Close(); the run loop only exits on done, so each request leaks one goroutine plus its 4KiB scratch buffer, channels and stack. Missing pairing: Close on Serve return (defer sub.Close()). Workload code: workload.Run -> Serve per request.
- Sibling hypotheses: F2.n1
- Supporting evidence: E8, E10, E14, E12, E17

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `internal.notify.render.Renderer.layouts`
- Region: heap
- Site: `internal/notify/render.go:10`
- Reason: not_exercised
- Remedy: Source shows no writer after construction (fixed 4-entry read-only table); no probe measured it. Add a driver probe on the variable if you want it measured.
- Supporting evidence: E8

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `internal.workload.workload.topics`
- Region: heap
- Site: `internal/workload/workload.go:19`
- Reason: not_exercised
- Remedy: Source shows no writer after construction (fixed 4-entry read-only table); no probe measured it. Add a driver probe on the variable if you want it measured.
- Supporting evidence: E8

## Ruled out

- **F2.n1** (UNCLASSIFIED): discriminator E17
  - Ruled out by: plateau+purge (leak) E17
  - Hypothesis: N-WARM: goroutine count is a warm-up/pool plateau
  - Refuted if: goroutine count keeps growing across extra cycles after purge
  - Contradicting evidence: E17
  - Originally supported by: E14

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
- E9: limitation: harness written; nothing was run (dla harness run measures it)
- E9: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E9: limitation: goleak not available offline: the goleak check is skipped
- E10: partial coverage (9 of 9 examined)
- E10: limitation: goroutineleak: blocked (the goroutineleak profile needs Go 1.27 or newer (go1.24.7 here) and GOTOOLCHAIN=local forbids switching to go1.27.0)
- E10: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E10: limitation: the goroutineleak profile is not available in go1.24.7
- E10: limitation: no entry points are listed for this runtime: run dla workload discover before the harness to see which ones ran
- E11: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E11: limitation: the goroutineleak profile is not available in go1.24.7
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: the goroutineleak profile is not available in go1.24.7
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: goroutines are grouped by the `created by` frame: a group ties to a finding only when that go statement is the finding's site or a recorded writer
- E13: limitation: goroutines without a creator (main, runtime) are not grouped; wait reasons and totals never tie
- E14: limitation: goroutines are grouped by the `created by` frame: a group ties to a finding only when that go statement is the finding's site or a recorded writer
- E14: limitation: goroutines without a creator (main, runtime) are not grouped; wait reasons and totals never tie
- E14: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E15: limitation: a Go heap profile is a sample (runtime.MemProfileRate): sizes are estimates, and objects smaller than the rate can be missed
- E15: limitation: in-use figures are as of the last completed collection: write the profile after runtime.GC()
- E15: limitation: a profile carries no holder: a stack ties to a finding only through its first project frame being the finding's site or writer
- E15: limitation: the profile is sampled: counts and bytes are estimates
- E16: limitation: a Go heap profile is a sample (runtime.MemProfileRate): sizes are estimates, and objects smaller than the rate can be missed
- E16: limitation: in-use figures are as of the last completed collection: write the profile after runtime.GC()
- E16: limitation: a profile carries no holder: a stack ties to a finding only through its first project frame being the finding's site or writer
- E16: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E17: limitation: goleak not available offline: the goleak check is skipped
- E17: limitation: gate and plateau thresholds are provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | — |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | — |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | — |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E3 | — |
| `intake.build` | Does the project build locally? | answered | E3 | — |
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
| E9 | workload | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | harness/H1/harness.json |
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (53245bcdc232) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (8ea6ba780ee8) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (8ea6ba780ee8) | trend/T1.json |
| E13 | non-heap | live | go-goroutine 0.1.0 | artifact nonheap/N1.json (af80e0688f8f) | nonheap/N1.json |
| E14 | growth-series | live | go-goroutine 0.1.0 | artifact series/S3.json (9b002808b6a1) | series/S3.json |
| E15 | alloc-stack | live | go-pprof-heap 0.1.0 | artifact allocs/A1.json (b8c435ced96d) | allocs/A1.json |
| E16 | growth-series | live | go-pprof-heap 0.1.0 | artifact series/S4.json (6d2622a54ea7) | series/S4.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S5.json (198889992a93) | alloc/A1.json |
