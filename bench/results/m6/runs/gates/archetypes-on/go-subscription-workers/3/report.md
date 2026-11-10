# DLA report

Run: `20261110-173506-95fe`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 5 enumerated, 5 dispositioned; confirmed 1, strong 0, refuted 1, suspected 2, unresolved 1

Residual: heap 5440.859545454545 bytes/n unattributed of 5440.859545454545 total (threshold 1360.2148863636362) ABOVE THRESHOLD, see O5; heap 5439.378787878788 bytes/n unattributed of 5439.378787878788 total (threshold 1359.844696969697) ABOVE THRESHOLD, see O5; heap 5441.980681818182 bytes/n unattributed of 5441.980681818182 total (threshold 1360.4951704545456) ABOVE THRESHOLD, see O5; non-heap 2203.151515151515 bytes/n unattributed of 2203.151515151515 total (threshold 550.7878787878788) ABOVE THRESHOLD, see O6; non-heap 2110.060606060606 bytes/n unattributed of 2110.060606060606 total (threshold 527.5151515151515) ABOVE THRESHOLD, see O6; non-heap 2122.4727272727273 bytes/n unattributed of 2122.4727272727273 total (threshold 530.6181818181818) ABOVE THRESHOLD, see O6

Coverage: go 3520 iterations, 15 of 19 entry points exercised

## Findings

### F2: **L-WORKER**

- State: confirmed
- Label: L-WORKER (evidence: E13 unit-count-series)
- Growth: `group:goroutine/created:example.com/notifyd/internal/notify.newSubscription` 1173 count/dump (steady, gate pass, E15)
- Evidence classes: context, discriminator, growth, static
- Holder: blocked-stack
- Owner: `internal.notify.subscription.newSubscription -> s.run()`
- Region: non-heap
- Shape: steady
- Site: `internal/notify/subscription.go:33`
- Writers: `internal/notify/handler.go:15`
- Removers: `internal/notify/subscription.go:86`
- Annotation: Handler.Serve opens a Subscription per request (hub.Open starts a goroutine) and never calls sub.Close; no remover exists
- Annotation: Subscription.Close exists (stops worker) but Handler.Serve never calls it
- Verified: no
- Sibling hypotheses: F2.n1
- Supporting evidence: E8, E11, E15, E16, E12
- Explanation (inferred): Goroutine started at internal/notify/subscription.go:33 (blocked-stack holder) per request: internal/notify/handler.go:15 calls hub.Open; Close at internal/notify/subscription.go:86 is never reached from Serve, so each worker parks forever.
- Explanation structure: adds: internal/notify/handler.go:15; removal: unreachable at internal/notify/subscription.go:86

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `internal.notify.render.Renderer.layouts`
- Region: heap
- Site: `internal/notify/render.go:10`
- Reason: not_exercised
- Remedy: layouts map is read-only after construction; no growth column tied to it
- Supporting evidence: E8

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `internal.workload.workload.topics`
- Region: heap
- Site: `internal/workload/workload.go:19`
- Reason: not_exercised
- Remedy: topics is a fixed 4-element slice; no growth observed
- Supporting evidence: E8

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: wait:select group is the same goroutines as F2 (created at subscription.go:33); no separate site
- Supporting evidence: E17

## Ruled out

- **F2.n1** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (leak) E16
  - Hypothesis: N-WARM: goroutine count is a warm-up/high-water plateau, not unbounded
  - Refuted if: growth continues over the extra cycles after purge
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

- E8: partial coverage
- E8: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E8: limitation: writers and removers are textual name matches
- E9: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E9: limitation: workloads were discovered by reading files; none was run
- E10: limitation: harness written; nothing was run (dla harness run measures it)
- E10: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E10: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E11: partial coverage (9 of 9 examined)
- E11: limitation: the goroutineleak profile comes from a separate pass with GOTOOLCHAIN=go1.27.0 (the project builds with go1.24.7); that pass measures nothing else
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
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: points 1 to 9 come from S1; the extra points from S3
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: x is the snapshot number in capture order; t_ms is the time since the first capture where the inputs record one
- E17: limitation: each point comes from a separate snapshot of one process; a figure absent from a snapshot has no value at that point
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: partial coverage (2 of 3 examined)
- E20: limitation: F2: no passing column is tied to it; nothing attributed
- E20: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E20: limitation: threshold is provisional (not calibrated)
- E21: partial coverage (2 of 3 examined)
- E21: limitation: F2: growth evidence is from S2, not S3; not subtracted
- E21: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E21: limitation: threshold is provisional (not calibrated)
- E22: partial coverage (2 of 3 examined)
- E22: limitation: F2: growth evidence is from S2, not S4; not subtracted
- E22: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E22: limitation: threshold is provisional (not calibrated)
- E23: partial coverage (2 of 3 examined)
- E23: limitation: F2: no passing column is tied to it; nothing attributed
- E23: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E23: limitation: threshold is provisional (not calibrated)
- E24: partial coverage (2 of 3 examined)
- E24: limitation: F2: growth evidence is from S2, not S3; not subtracted
- E24: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E24: limitation: threshold is provisional (not calibrated)
- E25: partial coverage (2 of 3 examined)
- E25: limitation: F2: growth evidence is from S2, not S4; not subtracted
- E25: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E25: limitation: threshold is provisional (not calibrated)
- E26: partial coverage (2 of 3 examined)
- E26: limitation: F2: no passing column is tied to it; nothing attributed
- E26: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E26: limitation: threshold is provisional (not calibrated)
- E27: partial coverage (2 of 3 examined)
- E27: limitation: F2: growth evidence is from S2, not S3; not subtracted
- E27: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E27: limitation: threshold is provisional (not calibrated)

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
| E4 | static-hit | static | staticcheck 2025.1.1 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | sweep/go-staticcheck.json |
| E5 | static-hit | static | golangci-lint 2.5.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | sweep/go-golangci-lint.json |
| E6 | static-hit | static | govet 1.24.7 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | sweep/go-govet.json |
| E7 | static-hit | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | sweep/go-dla.json |
| E8 | census-holder | static | dla-census 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | census/go.json |
| E9 | workload | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | workload/workloads.json |
| E10 | workload | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | harness/H1/harness.json |
| E11 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (a4ab2f90abc2) | harness/H1/run-S1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S1.json (d568640bb74b) | trend/T1.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (d568640bb74b) | trend/T1.json |
| E14 | non-heap | live | go-goroutine 0.1.0 | artifact nonheap/N1.json (c85f5f4e4fc4) | nonheap/N1.json |
| E15 | growth-series | live | go-goroutine 0.1.0 | artifact series/S2.json (82e90f80e31b) | series/S2.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S4.json (e33d958d0f09) | alloc/A1.json |
| E17 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (82e90f80e31b) | trend/T3.json |
| E18 | growth-series | test | dla 0.1.0 | artifact series/S3.json (6e2bc80ad452) | trend/T4.json |
| E19 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (6e2bc80ad452) | trend/T4.json |
| E20 | residual | test | dla 0.1.0 | artifact series/S1.json (d568640bb74b) | gap/residual-S1-E20.json |
| E21 | residual | test | dla 0.1.0 | artifact series/S3.json (6e2bc80ad452) | gap/residual-S3-E21.json |
| E22 | residual | test | dla 0.1.0 | artifact series/S4.json (e33d958d0f09) | gap/residual-S4-E22.json |
| E23 | residual | test | dla 0.1.0 | artifact series/S1.json (d568640bb74b) | gap/residual-S1-E23.json |
| E24 | residual | test | dla 0.1.0 | artifact series/S3.json (6e2bc80ad452) | gap/residual-S3-E24.json |
| E25 | residual | test | dla 0.1.0 | artifact series/S4.json (e33d958d0f09) | gap/residual-S4-E25.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S1.json (d568640bb74b) | gap/residual-S1-E26.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S3.json (6e2bc80ad452) | gap/residual-S3-E27.json |
