# DLA report

Run: `20261110-181531-7b2b`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 7 enumerated, 7 dispositioned; confirmed 1, strong 0, refuted 1, suspected 2, unresolved 3

Residual: heap 5441.203636363636 bytes/iteration unattributed of 5441.203636363636 total (threshold 1360.300909090909) ABOVE THRESHOLD, see O1; heap 5441.815151515151 bytes/iteration unattributed of 5441.815151515151 total (threshold 1360.4537878787878) ABOVE THRESHOLD, see O1; heap 5441.364848484848 bytes/iteration unattributed of 5441.364848484848 total (threshold 1360.341212121212) ABOVE THRESHOLD, see O1; non-heap 2283.830303030303 bytes/iteration unattributed of 2283.830303030303 total (threshold 570.9575757575758) ABOVE THRESHOLD, see O2; non-heap 2250.938181818182 bytes/iteration unattributed of 2250.938181818182 total (threshold 562.7345454545455) ABOVE THRESHOLD, see O2; non-heap 2134.8848484848486 bytes/iteration unattributed of 2134.8848484848486 total (threshold 533.7212121212121) ABOVE THRESHOLD, see O2

Coverage: go 7040 iterations, 0 of 0 entry points exercised

## Findings

### F2: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Per-request Subscription worker goroutine (subscription.go:33) is never stopped: Handler.Serve calls Hub.Open and never Subscription.Close, so each request leaves a blocked goroutine plus its 4KB scratch buffer and channels)
- Growth: `group:goroutine/created:example.com/notifyd/internal/notify.newSubscription` 1173 count/dump (steady, gate pass, E14)
- Evidence classes: context, growth, static
- Holder: blocked-stack
- Owner: `internal.notify.subscription.newSubscription -> s.run()`
- Region: non-heap
- Shape: steady
- Site: `internal/notify/subscription.go:33`
- Writers: `internal/notify/handler.go:18`
- Annotation: Handler.Serve opens a Subscription per request and never calls Close; the run goroutine and 4KB scratch stay
- Verified: no
- Sibling hypotheses: F2.n1
- Supporting evidence: E8, E10, E14, E16, E12
- Explanation (inferred): Subscription goroutine at subscription.go:33 is a blocked-stack holder: Handler.Serve at handler.go:18 opens one per request and never calls Close, so it stays blocked in select forever.
- Explanation structure: adds: internal/notify/handler.go:18; removal: never

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `internal.notify.render.Renderer.layouts`
- Region: heap
- Site: `internal/notify/render.go:10`
- Reason: not_exercised
- Remedy: Renderer.layouts is built once from a fixed 4-entry map and no growth column tied to it; confirm read-only in render.go
- Supporting evidence: E8

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `internal.workload.workload.topics`
- Region: heap
- Site: `internal/workload/workload.go:19`
- Reason: not_exercised
- Remedy: topics is a fixed package slice; probe P1 showed no growth; no writer found
- Supporting evidence: E8

### F4: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: Same growth as confirmed F2 (leaked Subscription goroutines from Handler.Serve, handler.go:18 / subscription.go:33): a symptom column of F2, not an independent holder. Re-measure after Serve closes its subscription.
- Supporting evidence: E20

### F5: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: Same growth as confirmed F2 (leaked Subscription goroutines from Handler.Serve, handler.go:18 / subscription.go:33): a symptom column of F2, not an independent holder. Re-measure after Serve closes its subscription.
- Supporting evidence: E21

### F6: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: Same growth as confirmed F2 (leaked Subscription goroutines from Handler.Serve, handler.go:18 / subscription.go:33): a symptom column of F2, not an independent holder. Re-measure after Serve closes its subscription.
- Supporting evidence: E22

## Ruled out

- **F2.n1** (UNCLASSIFIED): discriminator E17
  - Ruled out by: plateau+purge (leak) E17
  - Hypothesis: N-WARM: goroutine count is warm-up fill that plateaus
  - Refuted if: goroutines keep growing over extra cycles with repeated keys
  - Contradicting evidence: E17
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
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: the goroutineleak profile is not available in go1.24.7
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: the goroutineleak profile is not available in go1.24.7
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: x is the snapshot number in capture order; t_ms is the time since the first capture where the inputs record one
- E20: limitation: each point comes from a separate snapshot of one process; a figure absent from a snapshot has no value at that point
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E21: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E22: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: partial coverage (2 of 3 examined)
- E25: limitation: F2: growth measured as a count; bytes not attributed
- E25: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E25: limitation: threshold is provisional (not calibrated)
- E26: partial coverage (2 of 3 examined)
- E26: limitation: F2: growth evidence is from S3, not S2; not subtracted
- E26: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E26: limitation: threshold is provisional (not calibrated)
- E27: partial coverage (2 of 3 examined)
- E27: limitation: F2: growth evidence is from S3, not S5; not subtracted
- E27: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
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
| E9 | workload | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | harness/H1/harness.json |
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (bb3324edae35) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (ba623fb0a974) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (ba623fb0a974) | trend/T1.json |
| E13 | non-heap | live | go-goroutine 0.1.0 | artifact nonheap/N1.json (090a15ae3a20) | nonheap/N1.json |
| E14 | growth-series | live | go-goroutine 0.1.0 | artifact series/S3.json (efddc7fb3646) | series/S3.json |
| E15 | alloc-stack | live | go-pprof-heap 0.1.0 | artifact allocs/A1.json (793d6da68e38) | allocs/A1.json |
| E16 | growth-series | live | go-pprof-heap 0.1.0 | artifact series/S4.json (53135e5badb9) | series/S4.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S5.json (570f1ed3d72c) | alloc/A1.json |
| E18 | growth-series | test | dla 0.1.0 | artifact series/S2.json (ec564f7cd144) | trend/T3.json |
| E19 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (ec564f7cd144) | trend/T3.json |
| E20 | unit-count-series | dump | dla 0.1.0 | artifact series/S3.json (efddc7fb3646) | trend/T4.json |
| E21 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (53135e5badb9) | trend/T5.json |
| E22 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (53135e5badb9) | trend/T5.json |
| E23 | growth-series | test | dla 0.1.0 | artifact series/S5.json (570f1ed3d72c) | trend/T6.json |
| E24 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (570f1ed3d72c) | trend/T6.json |
| E25 | residual | test | dla 0.1.0 | artifact series/S1.json (ba623fb0a974) | gap/residual-S1-E25.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S2.json (ec564f7cd144) | gap/residual-S2-E26.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S5.json (570f1ed3d72c) | gap/residual-S5-E27.json |
