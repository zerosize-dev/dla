# DLA report

Run: `20261110-193601-f01a`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 6 enumerated, 6 dispositioned; confirmed 3, strong 0, refuted 3, suspected 0, unresolved 0

Residual: heap 0.14675324675324675 bytes/n unattributed of 0.14675324675324675 total (threshold 1.4465664935064992); heap 0.7845454545454545 bytes/n unattributed of 0.7845454545454545 total (threshold 0.8209560545454525); heap 1.6545454545454543 bytes/n unattributed of 1.6545454545454543 total (threshold 1.507630909090909) ABOVE THRESHOLD, see O3; heap 1.5236363636363637 bytes/n unattributed of 1.5236363636363637 total (threshold 2.134944); heap 1.1294372294372297 bytes/n unattributed of 1.1294372294372297 total (threshold 2.355381038961017); heap 0.8909090909090909 bytes/n unattributed of 0.8909090909090909 total (threshold 0.8248647272727272) ABOVE THRESHOLD, see O1; non-heap 0 bytes/n unattributed of 0 total (threshold 23.659985454545453); non-heap 0 bytes/n unattributed of 0 total (threshold 0); non-heap 0 bytes/n unattributed of 0 total (threshold 0); non-heap 0 bytes/n unattributed of 0 total (threshold 0); non-heap 0 bytes/n unattributed of 0 total (threshold 47.319970909090905); non-heap 0 bytes/n unattributed of 0 total (threshold 0)

Coverage: go 3520 iterations, 16 of 19 entry points exercised

## Findings

### F1.n1: Hypothesis N-INTENT bounded by design (fixed layouts/topics/worker closed per request) (UNCLASSIFIED)

- State: confirmed
- Explained by N-INTENT bounded by design (fixed layouts/topics/worker closed per request) (discriminator E22)
- Evidence classes: context, discriminator, growth
- Sibling hypothesis of: F1
- Hypothesis: N-INTENT bounded by design (fixed layouts/topics/worker closed per request)
- Refuted if: holder or goroutine count grows across traffic batches
- Supporting evidence: E12, E11, E22

### F2.n1: Hypothesis N-INTENT bounded by design (fixed layouts/topics/worker closed per request) (UNCLASSIFIED)

- State: confirmed
- Explained by N-INTENT bounded by design (fixed layouts/topics/worker closed per request) (discriminator E21)
- Evidence classes: context, discriminator, growth
- Sibling hypothesis of: F2
- Hypothesis: N-INTENT bounded by design (fixed layouts/topics/worker closed per request)
- Refuted if: holder or goroutine count grows across traffic batches
- Supporting evidence: E12, E11, E21

### F3.n1: Hypothesis N-INTENT bounded by design (fixed layouts/topics/worker closed per request) (UNCLASSIFIED)

- State: confirmed
- Explained by N-INTENT bounded by design (fixed layouts/topics/worker closed per request) (discriminator E23)
- Evidence classes: context, discriminator, growth
- Sibling hypothesis of: F3
- Hypothesis: N-INTENT bounded by design (fixed layouts/topics/worker closed per request)
- Refuted if: holder or goroutine count grows across traffic batches
- Supporting evidence: E12, E11, E23

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E22
  - Ruled out by: plateau+purge (non-leak) E22
  - Contradicting evidence: E22
  - Originally supported by: E8, E11, E12, E15
- **F2** (UNCLASSIFIED): discriminator E21
  - Ruled out by: plateau+purge (non-leak) E21
  - Contradicting evidence: E21
  - Originally supported by: E8, E11, E12, E15
- **F3** (UNCLASSIFIED): discriminator E23
  - Ruled out by: plateau+purge (non-leak) E23
  - Contradicting evidence: E23
  - Originally supported by: E8, E11, E12, E15

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
- E16: limitation: a Go heap profile is a sample (runtime.MemProfileRate): sizes are estimates, and objects smaller than the rate can be missed
- E16: limitation: in-use figures are as of the last completed collection: write the profile after runtime.GC()
- E16: limitation: a profile carries no holder: a stack ties to a finding only through its first project frame being the finding's site or writer
- E16: limitation: the profile is sampled: counts and bytes are estimates
- E17: limitation: a Go heap profile is a sample (runtime.MemProfileRate): sizes are estimates, and objects smaller than the rate can be missed
- E17: limitation: in-use figures are as of the last completed collection: write the profile after runtime.GC()
- E17: limitation: a profile carries no holder: a stack ties to a finding only through its first project frame being the finding's site or writer
- E17: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E18: limitation: x is the snapshot number in capture order; t_ms is the time since the first capture where the inputs record one
- E18: limitation: each point comes from a separate snapshot of one process; a figure absent from a snapshot has no value at that point
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E19: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E20: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: points 1 to 9 come from S1; the extra points from S4
- E21: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E21: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E21: limitation: gate and plateau thresholds are provisional (not calibrated)
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: points 1 to 9 come from S1; the extra points from S6
- E22: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E22: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E22: limitation: gate and plateau thresholds are provisional (not calibrated)
- E23: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E23: limitation: points 1 to 9 come from S1; the extra points from S8
- E23: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E23: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E23: limitation: gate and plateau thresholds are provisional (not calibrated)
- E24: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E29: limitation: gate thresholds are provisional (not calibrated)
- E30: partial coverage (2 of 3 examined)
- E30: limitation: F1.n1: growth evidence is from S1, not S5; not subtracted
- E30: limitation: F2.n1: growth evidence is from S1, not S5; not subtracted
- E30: limitation: F3.n1: growth evidence is from S1, not S5; not subtracted
- E30: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E30: limitation: threshold is provisional (not calibrated)
- E31: partial coverage (2 of 3 examined)
- E31: limitation: F1.n1: growth evidence is from S1, not S7; not subtracted
- E31: limitation: F2.n1: growth evidence is from S1, not S7; not subtracted
- E31: limitation: F3.n1: growth evidence is from S1, not S7; not subtracted
- E31: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E31: limitation: threshold is provisional (not calibrated)
- E32: partial coverage (2 of 3 examined)
- E32: limitation: F1.n1: growth evidence is from S1, not S9; not subtracted
- E32: limitation: F2.n1: growth evidence is from S1, not S9; not subtracted
- E32: limitation: F3.n1: growth evidence is from S1, not S9; not subtracted
- E32: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E32: limitation: threshold is provisional (not calibrated)
- E33: partial coverage (2 of 3 examined)
- E33: limitation: F1.n1: growth evidence is from S1, not S6; not subtracted
- E33: limitation: F2.n1: growth evidence is from S1, not S6; not subtracted
- E33: limitation: F3.n1: growth evidence is from S1, not S6; not subtracted
- E33: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E33: limitation: threshold is provisional (not calibrated)
- E34: partial coverage (2 of 3 examined)
- E34: limitation: F1.n1: growth evidence is from S1, not S8; not subtracted
- E34: limitation: F2.n1: growth evidence is from S1, not S8; not subtracted
- E34: limitation: F3.n1: growth evidence is from S1, not S8; not subtracted
- E34: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E34: limitation: O2 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E34: limitation: threshold is provisional (not calibrated)
- E35: partial coverage (2 of 3 examined)
- E35: limitation: F1.n1: growth evidence is from S1, not S4; not subtracted
- E35: limitation: F2.n1: growth evidence is from S1, not S4; not subtracted
- E35: limitation: F3.n1: growth evidence is from S1, not S4; not subtracted
- E35: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E35: limitation: O2 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E35: limitation: threshold is provisional (not calibrated)
- E36: partial coverage (2 of 3 examined)
- E36: limitation: F1.n1: growth evidence is from S1, not S6; not subtracted
- E36: limitation: F2.n1: growth evidence is from S1, not S6; not subtracted
- E36: limitation: F3.n1: growth evidence is from S1, not S6; not subtracted
- E36: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E36: limitation: threshold is provisional (not calibrated)

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
| E11 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (f2f660b68364) | harness/H1/run-S1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S1.json (8c061597ea26) | trend/T1.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (8c061597ea26) | trend/T1.json |
| E14 | non-heap | live | go-goroutine 0.1.0 | artifact nonheap/N1.json (37a7a4be6dde) | nonheap/N1.json |
| E15 | growth-series | live | go-goroutine 0.1.0 | artifact series/S2.json (8129e0c8ecd2) | series/S2.json |
| E16 | alloc-stack | live | go-pprof-heap 0.1.0 | artifact allocs/A1.json (b7d50c49cd5a) | allocs/A1.json |
| E17 | growth-series | live | go-pprof-heap 0.1.0 | artifact series/S3.json (bc787ac31b7b) | series/S3.json |
| E18 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (8129e0c8ecd2) | trend/T2.json |
| E19 | growth-series | dump | dla 0.1.0 | artifact series/S3.json (bc787ac31b7b) | trend/T3.json |
| E20 | unit-count-series | dump | dla 0.1.0 | artifact series/S3.json (bc787ac31b7b) | trend/T3.json |
| E21 | discriminator | test | dla 0.1.0 | artifact series/S5.json (6ceaeb86d8ce) | alloc/A1.json |
| E22 | discriminator | test | dla 0.1.0 | artifact series/S7.json (f256e6fbb6bb) | alloc/A2.json |
| E23 | discriminator | test | dla 0.1.0 | artifact series/S9.json (aaa1f1350ba3) | alloc/A3.json |
| E24 | growth-series | test | dla 0.1.0 | artifact series/S4.json (441e707cac51) | trend/T7.json |
| E25 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (441e707cac51) | trend/T7.json |
| E26 | growth-series | test | dla 0.1.0 | artifact series/S6.json (5d349336d097) | trend/T8.json |
| E27 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (5d349336d097) | trend/T8.json |
| E28 | growth-series | test | dla 0.1.0 | artifact series/S8.json (0666bb5bd9f7) | trend/T9.json |
| E29 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (0666bb5bd9f7) | trend/T9.json |
| E30 | residual | test | dla 0.1.0 | artifact series/S5.json (6ceaeb86d8ce) | gap/residual-S5-E30.json |
| E31 | residual | test | dla 0.1.0 | artifact series/S7.json (f256e6fbb6bb) | gap/residual-S7-E31.json |
| E32 | residual | test | dla 0.1.0 | artifact series/S9.json (aaa1f1350ba3) | gap/residual-S9-E32.json |
| E33 | residual | test | dla 0.1.0 | artifact series/S6.json (5d349336d097) | gap/residual-S6-E33.json |
| E34 | residual | test | dla 0.1.0 | artifact series/S8.json (0666bb5bd9f7) | gap/residual-S8-E34.json |
| E35 | residual | test | dla 0.1.0 | artifact series/S4.json (441e707cac51) | gap/residual-S4-E35.json |
| E36 | residual | test | dla 0.1.0 | artifact series/S6.json (5d349336d097) | gap/residual-S6-E36.json |
