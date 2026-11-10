# DLA report

Run: `20261110-162047-8e54`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 5 enumerated, 5 dispositioned; confirmed 1, strong 0, refuted 1, suspected 2, unresolved 1

Residual: heap 5441.236363636363 bytes/iteration unattributed of 5441.236363636363 total (threshold 1360.3090909090909) ABOVE THRESHOLD, see O1; heap 5441.740692640693 bytes/iteration unattributed of 5441.740692640693 total (threshold 1360.4351731601732) ABOVE THRESHOLD, see O3; heap 5441.332727272727 bytes/iteration unattributed of 5441.332727272727 total (threshold 1360.333181818182) ABOVE THRESHOLD, see O3; non-heap 2252.8 bytes/iteration unattributed of 2252.8 total (threshold 563.2) ABOVE THRESHOLD, see O2; non-heap 2259.0060606060606 bytes/iteration unattributed of 2259.0060606060606 total (threshold 564.7515151515152) ABOVE THRESHOLD, see O4; non-heap 2128.6787878787877 bytes/iteration unattributed of 2128.6787878787877 total (threshold 532.1696969696969) ABOVE THRESHOLD, see O4

Coverage: go 7040 iterations, 0 of 0 entry points exercised

## Findings

### F2: **L-WORKER**

- State: confirmed
- Label: L-WORKER (evidence: E12 unit-count-series)
- Growth: `group:goroutine/created:example.com/notifyd/internal/notify.newSubscription` 1173 count/dump (steady, gate pass, E14)
- Evidence classes: context, growth, static
- Holder: blocked-stack
- Owner: `internal.notify.subscription.newSubscription -> s.run()`
- Region: non-heap
- Shape: steady
- Site: `internal/notify/subscription.go:33`
- Writers: `internal/notify/subscription.go:33`, `internal/notify/hub.go:36`
- Annotation: Handler.Serve opens a Subscription per request (Hub.Open -> newSubscription -> go s.run()) and never calls Close; run() blocks in select until done is closed
- Verified: no
- Sibling hypotheses: F2.n1
- Supporting evidence: E8, E10, E14, E11
- Explanation (inferred): Subscription.run goroutine (blocked-stack, subscription.go:33) is started per request by Hub.Open in Handler.Serve, which never calls sub.Close, so done is never closed and each goroutine and its 4KB scratch stay forever.
- Explanation structure: adds: internal/notify/subscription.go:33; removal: never

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `internal.notify.render.Renderer.layouts`
- Region: heap
- Site: `internal/notify/render.go:10`
- Reason: not_exercised
- Remedy: Renderer.layouts is copied at construction and never written afterwards; no growth column at it. Add a driver probe on it if doubt remains.
- Supporting evidence: E8

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `internal.workload.workload.topics`
- Region: heap
- Site: `internal/workload/workload.go:19`
- Reason: not_exercised
- Remedy: workload.topics is a fixed 4-element slice never appended to; no probe body was filled. Probe len(topics) in the driver to close.
- Supporting evidence: E8

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: Same goroutines as confirmed F2, grouped by wait state (select); no separate holder. Closes with the F2 fix.
- Supporting evidence: E18

## Ruled out

- **F2.n1** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (leak) E15
  - Hypothesis: N-WARM: goroutine count is a warm-up/pool high-water mark
  - Refuted if: goroutine count plateaus over extra cycles or after purge
  - Contradicting evidence: E15
  - Originally supported by: E12

## Blind spots and unknowns

### Coverage gaps

- go/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
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
- E10: limitation: the goroutineleak profile comes from a separate pass with GOTOOLCHAIN=go1.27.0 (the project builds with go1.24.7); that pass measures nothing else
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
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: goleak not available offline: the goleak check is skipped
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: the goroutineleak profile is not available in go1.24.7
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: the goroutineleak profile is not available in go1.24.7
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: x is the snapshot number in capture order; t_ms is the time since the first capture where the inputs record one
- E18: limitation: each point comes from a separate snapshot of one process; a figure absent from a snapshot has no value at that point
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: partial coverage (2 of 3 examined)
- E19: limitation: F2: no passing column is tied to it; nothing attributed
- E19: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E19: limitation: threshold is provisional (not calibrated)
- E20: partial coverage (2 of 3 examined)
- E20: limitation: F2: growth evidence is from S3, not S2; not subtracted
- E20: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E20: limitation: threshold is provisional (not calibrated)
- E21: partial coverage (2 of 3 examined)
- E21: limitation: F2: growth evidence is from S3, not S4; not subtracted
- E21: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E21: limitation: threshold is provisional (not calibrated)

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
| E9 | workload | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | harness/H1/harness.json |
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (fe3e5775a236) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (26ab9bd72611) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (26ab9bd72611) | trend/T1.json |
| E13 | non-heap | live | go-goroutine 0.1.0 | artifact nonheap/N1.json (39d26a4a5ff5) | nonheap/N1.json |
| E14 | growth-series | live | go-goroutine 0.1.0 | artifact series/S3.json (efddc7fb3646) | series/S3.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S4.json (75288c925aaa) | alloc/A1.json |
| E16 | growth-series | test | dla 0.1.0 | artifact series/S2.json (0263c01dd15c) | trend/T3.json |
| E17 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (0263c01dd15c) | trend/T3.json |
| E18 | unit-count-series | dump | dla 0.1.0 | artifact series/S3.json (efddc7fb3646) | trend/T4.json |
| E19 | residual | test | dla 0.1.0 | artifact series/S1.json (26ab9bd72611) | gap/residual-S1-E19.json |
| E20 | residual | test | dla 0.1.0 | artifact series/S2.json (0263c01dd15c) | gap/residual-S2-E20.json |
| E21 | residual | test | dla 0.1.0 | artifact series/S4.json (75288c925aaa) | gap/residual-S4-E21.json |
