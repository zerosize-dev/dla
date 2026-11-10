# DLA report

Run: `20261110-174631-170d`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 6 enumerated, 6 dispositioned; confirmed 0, strong 0, refuted 3, suspected 0, unresolved 3

Residual: heap 1.8051948051948052 bytes/n unattributed of 1.8051948051948052 total (threshold 0.4512987012987013) ABOVE THRESHOLD, see O1; heap 0.6000000000000001 bytes/n unattributed of 0.6000000000000001 total (threshold 1.65204); heap 0.9764069264069264 bytes/n unattributed of 0.9764069264069264 total (threshold 0.540796000000005) ABOVE THRESHOLD, see O3; non-heap 0 bytes/n unattributed of 0 total (threshold 0); non-heap 0 bytes/n unattributed of 0 total (threshold 0); non-heap 0 bytes/n unattributed of 0 total (threshold 0)

Coverage: go 3520 iterations, 16 of 19 entry points exercised

## Findings

### F1.n1: Hypothesis N-WARM: bounded structure, fixed at construction (UNCLASSIFIED)

- State: unresolved(not_exercised)
- Evidence classes: discriminator
- Sibling hypothesis of: F1
- Hypothesis: N-WARM: bounded structure, fixed at construction
- Refuted if: growth continues over the extra cycles
- Reason: not_exercised
- Remedy: non-leak sibling supported by alloc-test plateau; leak side refuted. sub-byte heap slope (<2 B/request) with plateau in alloc-test; the replay does not reproduce the reported growth. Remedy: capture pprof heap/goroutine and thread counts from the real pod across batches.
- Supporting evidence: E19

### F2.n1: Hypothesis N-WARM: worker goroutines are bounded; each Subscription is closed by defer in Handler.Serve (UNCLASSIFIED)

- State: unresolved(not_exercised)
- Evidence classes: discriminator
- Sibling hypothesis of: F2
- Hypothesis: N-WARM: worker goroutines are bounded; each Subscription is closed by defer in Handler.Serve
- Refuted if: growth continues over the extra cycles
- Reason: not_exercised
- Remedy: non-leak sibling supported by alloc-test plateau; leak side refuted. sub-byte heap slope (<2 B/request) with plateau in alloc-test; the replay does not reproduce the reported growth. Remedy: capture pprof heap/goroutine and thread counts from the real pod across batches.
- Supporting evidence: E18

### F3.n1: Hypothesis N-WARM: bounded structure, fixed at construction (UNCLASSIFIED)

- State: unresolved(not_exercised)
- Evidence classes: discriminator
- Sibling hypothesis of: F3
- Hypothesis: N-WARM: bounded structure, fixed at construction
- Refuted if: growth continues over the extra cycles
- Reason: not_exercised
- Remedy: non-leak sibling supported by alloc-test plateau; leak side refuted. sub-byte heap slope (<2 B/request) with plateau in alloc-test; the replay does not reproduce the reported growth. Remedy: capture pprof heap/goroutine and thread counts from the real pod across batches.
- Supporting evidence: E20

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E19
  - Ruled out by: plateau+purge (non-leak) E19
  - Contradicting evidence: E19
  - Originally supported by: E8
- **F2** (UNCLASSIFIED): discriminator E18
  - Ruled out by: plateau+purge (non-leak) E18
  - Contradicting evidence: E18
  - Originally supported by: E8
- **F3** (UNCLASSIFIED): discriminator E20
  - Ruled out by: plateau+purge (non-leak) E20
  - Contradicting evidence: E20
  - Originally supported by: E8

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
- E14: partial coverage (2 of 3 examined)
- E14: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E14: limitation: threshold is provisional (not calibrated)
- E15: limitation: goroutines are grouped by the `created by` frame: a group ties to a finding only when that go statement is the finding's site or a recorded writer
- E15: limitation: goroutines without a creator (main, runtime) are not grouped; wait reasons and totals never tie
- E16: limitation: goroutines are grouped by the `created by` frame: a group ties to a finding only when that go statement is the finding's site or a recorded writer
- E16: limitation: goroutines without a creator (main, runtime) are not grouped; wait reasons and totals never tie
- E16: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E17: limitation: x is the snapshot number in capture order; t_ms is the time since the first capture where the inputs record one
- E17: limitation: each point comes from a separate snapshot of one process; a figure absent from a snapshot has no value at that point
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: points 1 to 9 come from S1; the extra points from S3
- E18: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E18: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E18: limitation: gate and plateau thresholds are provisional (not calibrated)
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: points 1 to 9 come from S1; the extra points from S5
- E19: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E19: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E19: limitation: gate and plateau thresholds are provisional (not calibrated)
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: points 1 to 9 come from S1; the extra points from S7
- E20: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E20: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E20: limitation: gate and plateau thresholds are provisional (not calibrated)
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: partial coverage (2 of 3 examined)
- E27: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E27: limitation: threshold is provisional (not calibrated)
- E28: partial coverage (2 of 3 examined)
- E28: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E28: limitation: O2 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E28: limitation: threshold is provisional (not calibrated)
- E29: partial coverage (2 of 3 examined)
- E29: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E29: limitation: threshold is provisional (not calibrated)
- E30: partial coverage (2 of 3 examined)
- E30: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E30: limitation: O3 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O3 --as unresolved --reason time_budget --remedy "<text>"
- E30: limitation: threshold is provisional (not calibrated)

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
| E11 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (a026cc499020) | harness/H1/run-S1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S1.json (be9179046e5d) | trend/T1.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (be9179046e5d) | trend/T1.json |
| E14 | residual | test | dla 0.1.0 | artifact series/S1.json (be9179046e5d) | gap/residual-S1-E14.json |
| E15 | non-heap | live | go-goroutine 0.1.0 | artifact nonheap/N1.json (c3841a190fd8) | nonheap/N1.json |
| E16 | growth-series | live | go-goroutine 0.1.0 | artifact series/S2.json (8129e0c8ecd2) | series/S2.json |
| E17 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (8129e0c8ecd2) | trend/T2.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S4.json (902e3a67e514) | alloc/A1.json |
| E19 | discriminator | test | dla 0.1.0 | artifact series/S6.json (11a3c48851bf) | alloc/A2.json |
| E20 | discriminator | test | dla 0.1.0 | artifact series/S8.json (82c1c2530935) | alloc/A3.json |
| E21 | growth-series | test | dla 0.1.0 | artifact series/S3.json (c52eeaaa225d) | trend/T6.json |
| E22 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (c52eeaaa225d) | trend/T6.json |
| E23 | growth-series | test | dla 0.1.0 | artifact series/S5.json (6b17aaed09f2) | trend/T7.json |
| E24 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (6b17aaed09f2) | trend/T7.json |
| E25 | growth-series | test | dla 0.1.0 | artifact series/S7.json (31ed305b1e9d) | trend/T8.json |
| E26 | unit-count-series | test | dla 0.1.0 | artifact series/S7.json (31ed305b1e9d) | trend/T8.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S4.json (902e3a67e514) | gap/residual-S4-E27.json |
| E28 | residual | test | dla 0.1.0 | artifact series/S3.json (c52eeaaa225d) | gap/residual-S3-E28.json |
| E29 | residual | test | dla 0.1.0 | artifact series/S4.json (902e3a67e514) | gap/residual-S4-E29.json |
| E30 | residual | test | dla 0.1.0 | artifact series/S3.json (c52eeaaa225d) | gap/residual-S3-E30.json |
