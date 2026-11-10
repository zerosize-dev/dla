# DLA report

Run: `20261110-192611-7bb0`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 5 enumerated, 5 dispositioned; confirmed 1, strong 0, refuted 1, suspected 2, unresolved 1

Residual: heap 5441.072727272727 bytes/iteration unattributed of 5441.072727272727 total (threshold 1360.2681818181818) ABOVE THRESHOLD, see O1; heap 5441.121212121212 bytes/iteration unattributed of 5441.121212121212 total (threshold 1360.280303030303) ABOVE THRESHOLD, see O3; heap 5441.8207792207795 bytes/iteration unattributed of 5441.8207792207795 total (threshold 1360.4551948051949) ABOVE THRESHOLD, see O3; heap 5441.3033216783215 bytes/iteration unattributed of 5441.3033216783215 total (threshold 1360.3258304195804) ABOVE THRESHOLD, see O5; heap 5442.310139860139 bytes/iteration unattributed of 5442.310139860139 total (threshold 1360.5775349650348) ABOVE THRESHOLD, see O7; non-heap 2254.12987012987 bytes/iteration unattributed of 2254.12987012987 total (threshold 563.5324675324675) ABOVE THRESHOLD, see O2; non-heap 2271.418181818182 bytes/iteration unattributed of 2271.418181818182 total (threshold 567.8545454545455) ABOVE THRESHOLD, see O4; non-heap 2128.235497835498 bytes/iteration unattributed of 2128.235497835498 total (threshold 532.0588744588745) ABOVE THRESHOLD, see O4; non-heap 2130.492867132867 bytes/iteration unattributed of 2130.492867132867 total (threshold 532.6232167832168) ABOVE THRESHOLD, see O6; non-heap 2143.159595959596 bytes/iteration unattributed of 2143.159595959596 total (threshold 535.789898989899) ABOVE THRESHOLD, see O8

Coverage: go 7040 iterations, 0 of 19 entry points exercised

## Findings

### F2: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Per-request Subscription worker goroutine (subscription.go:33) parked in select on events/flushes/done; Handler.Serve never calls Subscription.Close, so each request leaks one goroutine plus its 4KB scratch buffer and Subscription)
- Growth: `group:goroutine/created:example.com/notifyd/internal/notify.newSubscription` 1173 count/dump (steady, gate pass, E18)
- Evidence classes: context, growth, static
- Holder: blocked-stack
- Owner: `internal.notify.subscription.newSubscription -> s.run()`
- Region: non-heap
- Shape: steady
- Site: `internal/notify/subscription.go:33`
- Writers: `internal/notify/handler.go:17`
- Annotation: Handler.Serve opens a Subscription via Hub.Open (worker goroutine started at subscription.go:33) and never calls Close; the worker blocks forever in select
- Verified: no
- Sibling hypotheses: F2.n1
- Supporting evidence: E9, E12, E18, E13
- Explanation (inferred): Hub.Open at handler.go:17 starts a Subscription whose worker goroutine (subscription.go:33) blocks in select; Handler.Serve never calls Close, so the blocked-stack goroutine and its 4KB scratch buffer are never released.
- Explanation structure: adds: internal/notify/handler.go:17; removal: never

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: discriminator, static
- Holder: container-field
- Owner: `internal.notify.render.Renderer.layouts`
- Region: heap
- Site: `internal/notify/render.go:10`
- Reason: not_exercised
- Remedy: Written only at construction (NewRenderer / package literal), no writer after init; alloc-test favors leak only because F2's whole-process growth dominates. Re-run alloc-test after fixing F2 (closing the Subscription) to show it flat.
- Supporting evidence: E9, E23

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: discriminator, static
- Holder: static/global
- Owner: `internal.workload.workload.topics`
- Region: heap
- Site: `internal/workload/workload.go:19`
- Reason: not_exercised
- Remedy: Written only at construction (NewRenderer / package literal), no writer after init; alloc-test favors leak only because F2's whole-process growth dominates. Re-run alloc-test after fixing F2 (closing the Subscription) to show it flat.
- Supporting evidence: E9, E22

### F4: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: Duplicate view of F2: the goroutine wait:select group from the summarize series is the same per-request Subscription worker (subscription.go:33); covered by confirmed F2
- Supporting evidence: E20

## Ruled out

- **F2.n1** (UNCLASSIFIED): discriminator E19
  - Ruled out by: plateau+purge (leak) E19
  - Hypothesis: N-WARM: goroutine growth is a warm-up/pool high-water that plateaus
  - Refuted if: goroutine count keeps growing linearly over extra cycles and after purge
  - Contradicting evidence: E19
  - Originally supported by: E14

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

- E9: partial coverage
- E9: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E9: limitation: writers and removers are textual name matches
- E10: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E10: limitation: workloads were discovered by reading files; none was run
- E11: limitation: harness written; nothing was run (dla harness run measures it)
- E11: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E11: limitation: goleak not available offline: the goleak check is skipped
- E12: partial coverage (9 of 9 examined)
- E12: limitation: the goroutineleak profile comes from a separate pass with GOTOOLCHAIN=go1.27.0 (the project builds with go1.24.7); that pass measures nothing else
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: the goroutineleak profile is not available in go1.24.7
- E12: limitation: no Go coverage profile was written to cov/
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
- E16: limitation: the goroutineleak profile is not available in go1.24.7
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: goroutines are grouped by the `created by` frame: a group ties to a finding only when that go statement is the finding's site or a recorded writer
- E17: limitation: goroutines without a creator (main, runtime) are not grouped; wait reasons and totals never tie
- E18: limitation: goroutines are grouped by the `created by` frame: a group ties to a finding only when that go statement is the finding's site or a recorded writer
- E18: limitation: goroutines without a creator (main, runtime) are not grouped; wait reasons and totals never tie
- E18: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E19: limitation: goleak not available offline: the goleak check is skipped
- E19: limitation: gate and plateau thresholds are provisional (not calibrated)
- E20: limitation: x is the snapshot number in capture order; t_ms is the time since the first capture where the inputs record one
- E20: limitation: each point comes from a separate snapshot of one process; a figure absent from a snapshot has no value at that point
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: partial coverage (2 of 3 examined)
- E21: limitation: F2: no passing column is tied to it; nothing attributed
- E21: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E21: limitation: threshold is provisional (not calibrated)
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E22: limitation: goleak not available offline: the goleak check is skipped
- E22: limitation: gate and plateau thresholds are provisional (not calibrated)
- E23: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E23: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E23: limitation: goleak not available offline: the goleak check is skipped
- E23: limitation: gate and plateau thresholds are provisional (not calibrated)
- E24: partial coverage (2 of 3 examined)
- E24: limitation: F2: growth evidence is from S3, not S2; not subtracted
- E24: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E24: limitation: threshold is provisional (not calibrated)
- E25: partial coverage (2 of 3 examined)
- E25: limitation: F2: growth evidence is from S3, not S4; not subtracted
- E25: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E25: limitation: threshold is provisional (not calibrated)
- E26: partial coverage (2 of 3 examined)
- E26: limitation: F2: growth evidence is from S3, not S5; not subtracted
- E26: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E26: limitation: threshold is provisional (not calibrated)
- E27: partial coverage (2 of 3 examined)
- E27: limitation: F2: growth evidence is from S3, not S6; not subtracted
- E27: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
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
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | — |
| E2 | run-config | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | probe/probe.json |
| E4 | probe | static | dla-probe 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | probe/probe.json |
| E5 | static-hit | static | staticcheck 2025.1.1 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | sweep/go-staticcheck.json |
| E6 | static-hit | static | golangci-lint 2.5.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | sweep/go-golangci-lint.json |
| E7 | static-hit | static | govet 1.24.7 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | sweep/go-govet.json |
| E8 | static-hit | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | sweep/go-dla.json |
| E9 | census-holder | static | dla-census 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | census/go.json |
| E10 | workload | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | workload/workloads.json |
| E11 | workload | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | harness/H1/harness.json |
| E12 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (18dc0cfb789c) | harness/H1/run-S1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S1.json (c7a969b4647e) | trend/T1.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (c7a969b4647e) | trend/T1.json |
| E15 | growth-series | test | dla 0.1.0 | artifact series/S2.json (0fd57bfba94c) | trend/T2.json |
| E16 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (0fd57bfba94c) | trend/T2.json |
| E17 | non-heap | live | go-goroutine 0.1.0 | artifact nonheap/N1.json (4593df8f8746) | nonheap/N1.json |
| E18 | growth-series | live | go-goroutine 0.1.0 | artifact series/S3.json (efddc7fb3646) | series/S3.json |
| E19 | discriminator | test | dla 0.1.0 | artifact series/S4.json (cd61dd317e35) | alloc/A1.json |
| E20 | unit-count-series | dump | dla 0.1.0 | artifact series/S3.json (efddc7fb3646) | trend/T4.json |
| E21 | residual | test | dla 0.1.0 | artifact series/S1.json (c7a969b4647e) | gap/residual-S1-E21.json |
| E22 | discriminator | test | dla 0.1.0 | artifact series/S5.json (a20447d33956) | alloc/A2.json |
| E23 | discriminator | test | dla 0.1.0 | artifact series/S6.json (540df11d5c60) | alloc/A3.json |
| E24 | residual | test | dla 0.1.0 | artifact series/S2.json (0fd57bfba94c) | gap/residual-S2-E24.json |
| E25 | residual | test | dla 0.1.0 | artifact series/S4.json (cd61dd317e35) | gap/residual-S4-E25.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S5.json (a20447d33956) | gap/residual-S5-E26.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S6.json (540df11d5c60) | gap/residual-S6-E27.json |
