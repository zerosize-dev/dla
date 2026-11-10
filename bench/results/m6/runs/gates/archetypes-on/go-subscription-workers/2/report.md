# DLA report

Run: `20261110-165455-cf01`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 5 enumerated, 5 dispositioned; confirmed 1, strong 0, refuted 1, suspected 2, unresolved 1

Residual: heap 5441.784848484849 bytes/iteration unattributed of 5441.784848484849 total (threshold 1360.4462121212125) ABOVE THRESHOLD, see O1; heap 5443.769090909091 bytes/iteration unattributed of 5443.769090909091 total (threshold 1360.942272727273) ABOVE THRESHOLD, see O1; heap 5441.318831168832 bytes/iteration unattributed of 5441.318831168832 total (threshold 1360.329707792208) ABOVE THRESHOLD, see O1; non-heap 2271.418181818182 bytes/iteration unattributed of 2271.418181818182 total (threshold 567.8545454545455) ABOVE THRESHOLD, see O2; non-heap 2295.001212121212 bytes/iteration unattributed of 2295.001212121212 total (threshold 573.750303030303) ABOVE THRESHOLD, see O2; non-heap 2126.610101010101 bytes/iteration unattributed of 2126.610101010101 total (threshold 531.6525252525253) ABOVE THRESHOLD, see O2

Coverage: go 7040 iterations, 0 of 19 entry points exercised

## Findings

### F2: **L-WORKER**

- State: confirmed
- Label: L-WORKER (evidence: E15 unit-count-series)
- Growth: `go.goroutines@internal/notify/subscription.go:33` 1 count/iteration (steady, gate pass, E15)
- Evidence classes: context, growth, static
- Holder: blocked-stack
- Owner: `internal.notify.subscription.newSubscription -> s.run()`
- Region: non-heap
- Shape: steady
- Site: `internal/notify/subscription.go:33`
- Writers: `internal/notify/handler.go:17`
- Verified: no
- Sibling hypotheses: F2.n1
- Supporting evidence: E4, E11, E15
- Explanation (observed): Handler.Serve (handler.go:17) calls Hub.Open, whose newSubscription starts a goroutine at subscription.go:33 parked in select in run(); Serve never calls Subscription.Close, so each request leaks one goroutine plus its 4KB scratch buffer.
- Explanation structure: adds: internal/notify/subscription.go:33; removal: never; evidence: E18

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `internal.notify.render.Renderer.layouts`
- Region: heap
- Site: `internal/notify/render.go:10`
- Reason: not_exercised
- Remedy: Renderer.layouts is read-only after construction; no growth observed
- Supporting evidence: E4

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `internal.workload.workload.topics`
- Region: heap
- Site: `internal/workload/workload.go:19`
- Reason: not_exercised
- Remedy: workload.topics is a fixed package var; no growth observed
- Supporting evidence: E4

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: Same goroutine growth as F2 seen as the select-wait group; fixing F2 (close the Subscription) removes it. Re-run harness after the fix.
- Supporting evidence: E20

## Ruled out

- **F2.n1** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (leak) E16
  - Hypothesis: N-WARM
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E16
  - Originally supported by: E13

## Blind spots and unknowns

### Coverage gaps

- go/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- go/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)

### Runtime blind spots

None recorded.

### Entry points not exercisable

- go: `internal/notify/hub.go::Hub.Opened`: no_harness
- go: `internal/workload/workload.go::NewRNG`: no_harness

### Evidence limitations and unknowns

- E4: partial coverage
- E4: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E4: limitation: writers and removers are textual name matches
- E9: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E9: limitation: workloads were discovered by reading files; none was run
- E10: limitation: harness written; nothing was run (dla harness run measures it)
- E10: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E10: limitation: goleak not available offline: the goleak check is skipped
- E11: partial coverage (9 of 9 examined)
- E11: limitation: the goroutineleak profile comes from a separate pass with GOTOOLCHAIN=go1.27.0 (the project builds with go1.24.7); that pass measures nothing else
- E11: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E11: limitation: the goroutineleak profile is not available in go1.24.7
- E11: limitation: no Go coverage profile was written to cov/
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: the goroutineleak profile is not available in go1.24.7
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: the goroutineleak profile is not available in go1.24.7
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: the goroutineleak profile is not available in go1.24.7
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: the goroutineleak profile is not available in go1.24.7
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: goleak not available offline: the goleak check is skipped
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: goroutines are grouped by the `created by` frame: a group ties to a finding only when that go statement is the finding's site or a recorded writer
- E17: limitation: goroutines without a creator (main, runtime) are not grouped; wait reasons and totals never tie
- E18: limitation: goroutines are grouped by the `created by` frame: a group ties to a finding only when that go statement is the finding's site or a recorded writer
- E18: limitation: goroutines without a creator (main, runtime) are not grouped; wait reasons and totals never tie
- E18: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E19: limitation: a Go heap profile is a sample (runtime.MemProfileRate): sizes are estimates, and objects smaller than the rate can be missed
- E19: limitation: in-use figures are as of the last completed collection: write the profile after runtime.GC()
- E19: limitation: a profile carries no holder: a stack ties to a finding only through its first project frame being the finding's site or writer
- E19: limitation: the profile is sampled: counts and bytes are estimates
- E20: limitation: x is the snapshot number in capture order; t_ms is the time since the first capture where the inputs record one
- E20: limitation: each point comes from a separate snapshot of one process; a figure absent from a snapshot has no value at that point
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: partial coverage (2 of 3 examined)
- E21: limitation: F2: growth evidence is from S2, not S1; not subtracted
- E21: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E21: limitation: threshold is provisional (not calibrated)
- E22: partial coverage (2 of 3 examined)
- E22: limitation: F2: growth measured as a count; bytes not attributed
- E22: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E22: limitation: threshold is provisional (not calibrated)
- E23: partial coverage (2 of 3 examined)
- E23: limitation: F2: growth evidence is from S2, not S3; not subtracted
- E23: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E23: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | — |
| E2 | run-config | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | probe/probe.json |
| E4 | census-holder | static | dla-census 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | census/go.json |
| E5 | static-hit | static | staticcheck 2025.1.1 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | sweep/go-staticcheck.json |
| E6 | static-hit | static | golangci-lint 2.5.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | sweep/go-golangci-lint.json |
| E7 | static-hit | static | govet 1.24.7 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | sweep/go-govet.json |
| E8 | static-hit | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | sweep/go-dla.json |
| E9 | workload | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | workload/workloads.json |
| E10 | workload | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | harness/H1/harness.json |
| E11 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (96a055705adc) | harness/H1/run-S1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S1.json (a7293f5fed5b) | trend/T1.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (a7293f5fed5b) | trend/T1.json |
| E14 | growth-series | test | dla 0.1.0 | artifact series/S2.json (7c2eb78e067b) | trend/T2.json |
| E15 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (7c2eb78e067b) | trend/T2.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S3.json (f8259e2017a8) | alloc/A1.json |
| E17 | non-heap | live | go-goroutine 0.1.0 | artifact nonheap/N1.json (f9cac3ec6eaf) | nonheap/N1.json |
| E18 | growth-series | live | go-goroutine 0.1.0 | artifact series/S4.json (f9156452847e) | series/S4.json |
| E19 | alloc-stack | live | go-pprof-heap 0.1.0 | artifact allocs/A1.json (3cdf5cff73c6) | allocs/A1.json |
| E20 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (f9156452847e) | trend/T4.json |
| E21 | residual | test | dla 0.1.0 | artifact series/S1.json (a7293f5fed5b) | gap/residual-S1-E21.json |
| E22 | residual | test | dla 0.1.0 | artifact series/S2.json (7c2eb78e067b) | gap/residual-S2-E22.json |
| E23 | residual | test | dla 0.1.0 | artifact series/S3.json (f8259e2017a8) | gap/residual-S3-E23.json |
