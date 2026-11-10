# DLA report

Run: `20261110-164506-316a`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 7 enumerated, 7 dispositioned; confirmed 0, strong 0, refuted 5, suspected 0, unresolved 2

Residual: heap 0.2922077922077922 bytes/iteration unattributed of 0.2922077922077922 total (threshold 0.8196659999999775); heap 1.6864935064935065 bytes/iteration unattributed of 1.6864935064935065 total (threshold 2.94875111688316); heap 6.398484848484848 bytes/iteration unattributed of 6.398484848484848 total (threshold 1.599621212121212) ABOVE THRESHOLD, see O1; heap 0.7666666666666667 bytes/iteration unattributed of 0.7666666666666667 total (threshold 1.1582250909090648); heap 0.4834710743801653 bytes/iteration unattributed of 0.4834710743801653 total (threshold 1.6573262479339483); heap 0.3977272727272727 bytes/iteration unattributed of 0.3977272727272727 total (threshold 1.2635795454545455); heap 0.6745454545454546 bytes/iteration unattributed of 0.6745454545454546 total (threshold 1.579103781818229); non-heap 0 bytes/iteration unattributed of 0 total (threshold 0); non-heap 0 bytes/iteration unattributed of 0 total (threshold 0); non-heap 11.525541125541125 bytes/iteration unattributed of 11.525541125541125 total (threshold 9.01332779220776) ABOVE THRESHOLD, see O2; non-heap 0 bytes/iteration unattributed of 0 total (threshold 0); non-heap 0 bytes/iteration unattributed of 0 total (threshold 0); non-heap 0 bytes/iteration unattributed of 0 total (threshold 0); non-heap 0 bytes/iteration unattributed of 0 total (threshold 0)

Coverage: go 7040 iterations, 9 of 11 entry points exercised

## Findings

### F3.n1: Hypothesis N-HWM: BufferPool.free holds one buffer per invoice of the largest batch (<=400, grown capacity); bounded high-water mark (UNCLASSIFIED)

- State: unresolved(time_budget)
- Evidence classes: discriminator
- Sibling hypothesis of: F3
- Hypothesis: N-HWM: BufferPool.free holds one buffer per invoice of the largest batch (<=400, grown capacity); bounded high-water mark
- Refuted if: live heap keeps growing over extra cycles
- Reason: time_budget
- Remedy: N-HWM is supported by alloc-test E26 (plateau, heap_live ~1.5MB flat); no further test planned
- Supporting evidence: E26

### F6: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: MakeBatch stack in the heap profile is the in-flight batch (size random up to 400 invoices) alive at capture time, not a retained holder; confirm with more profiles
- Supporting evidence: E31, E32

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E27
  - Ruled out by: plateau+purge (non-leak) E27
  - Contradicting evidence: E27
  - Originally supported by: E8
- **F2** (UNCLASSIFIED): discriminator E28
  - Ruled out by: plateau+purge (non-leak) E28
  - Contradicting evidence: E28
  - Originally supported by: E8
- **F3** (UNCLASSIFIED): discriminator E26
  - Ruled out by: plateau+purge (non-leak) E26
  - Contradicting evidence: E26
  - Originally supported by: E8
- **F4** (UNCLASSIFIED): discriminator E29
  - Ruled out by: plateau+purge (non-leak) E29
  - Contradicting evidence: E29
  - Originally supported by: E8
- **F5** (UNCLASSIFIED): discriminator E30
  - Ruled out by: plateau+purge (non-leak) E30
  - Contradicting evidence: E30
  - Originally supported by: E8

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
- E14: limitation: a Go heap profile is a sample (runtime.MemProfileRate): sizes are estimates, and objects smaller than the rate can be missed
- E14: limitation: in-use figures are as of the last completed collection: write the profile after runtime.GC()
- E14: limitation: a profile carries no holder: a stack ties to a finding only through its first project frame being the finding's site or writer
- E14: limitation: the profile is sampled: counts and bytes are estimates
- E15: limitation: a Go heap profile is a sample (runtime.MemProfileRate): sizes are estimates, and objects smaller than the rate can be missed
- E15: limitation: in-use figures are as of the last completed collection: write the profile after runtime.GC()
- E15: limitation: a profile carries no holder: a stack ties to a finding only through its first project frame being the finding's site or writer
- E15: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E16: limitation: harness written; nothing was run (dla harness run measures it)
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: goleak not available offline: the goleak check is skipped
- E17: partial coverage (9 of 9 examined)
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: no Go coverage profile was written to cov/
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E22: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E23: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: a Go heap profile is a sample (runtime.MemProfileRate): sizes are estimates, and objects smaller than the rate can be missed
- E24: limitation: in-use figures are as of the last completed collection: write the profile after runtime.GC()
- E24: limitation: a profile carries no holder: a stack ties to a finding only through its first project frame being the finding's site or writer
- E24: limitation: the profile is sampled: counts and bytes are estimates
- E25: limitation: a Go heap profile is a sample (runtime.MemProfileRate): sizes are estimates, and objects smaller than the rate can be missed
- E25: limitation: in-use figures are as of the last completed collection: write the profile after runtime.GC()
- E25: limitation: a profile carries no holder: a stack ties to a finding only through its first project frame being the finding's site or writer
- E25: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E26: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E26: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E26: limitation: goleak not available offline: the goleak check is skipped
- E26: limitation: gate and plateau thresholds are provisional (not calibrated)
- E27: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E27: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E27: limitation: goleak not available offline: the goleak check is skipped
- E27: limitation: gate and plateau thresholds are provisional (not calibrated)
- E28: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E28: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E28: limitation: goleak not available offline: the goleak check is skipped
- E28: limitation: gate and plateau thresholds are provisional (not calibrated)
- E29: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E29: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E29: limitation: goleak not available offline: the goleak check is skipped
- E29: limitation: gate and plateau thresholds are provisional (not calibrated)
- E30: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E30: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E30: limitation: goleak not available offline: the goleak check is skipped
- E30: limitation: gate and plateau thresholds are provisional (not calibrated)
- E31: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E31: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E31: limitation: gate thresholds are provisional (not calibrated)
- E32: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E32: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E32: limitation: gate thresholds are provisional (not calibrated)
- E33: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E33: limitation: gate thresholds are provisional (not calibrated)
- E34: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E34: limitation: gate thresholds are provisional (not calibrated)
- E35: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E35: limitation: gate thresholds are provisional (not calibrated)
- E36: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E36: limitation: gate thresholds are provisional (not calibrated)
- E37: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E37: limitation: gate thresholds are provisional (not calibrated)
- E38: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E38: limitation: gate thresholds are provisional (not calibrated)
- E39: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E39: limitation: gate thresholds are provisional (not calibrated)
- E40: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E40: limitation: gate thresholds are provisional (not calibrated)
- E41: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E41: limitation: gate thresholds are provisional (not calibrated)
- E42: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E42: limitation: gate thresholds are provisional (not calibrated)
- E43: partial coverage (2 of 3 examined)
- E43: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E43: limitation: threshold is provisional (not calibrated)
- E44: partial coverage (2 of 3 examined)
- E44: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E44: limitation: threshold is provisional (not calibrated)
- E45: partial coverage (2 of 3 examined)
- E45: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E45: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E45: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E45: limitation: threshold is provisional (not calibrated)
- E46: partial coverage (2 of 3 examined)
- E46: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E46: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E46: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E46: limitation: threshold is provisional (not calibrated)
- E47: partial coverage (2 of 3 examined)
- E47: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E47: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E47: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E47: limitation: threshold is provisional (not calibrated)
- E48: partial coverage (2 of 3 examined)
- E48: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E48: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E48: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E48: limitation: threshold is provisional (not calibrated)
- E49: partial coverage (2 of 3 examined)
- E49: limitation: off-heap: no gap series for S10; run dla gap S10 first (node.external is used when present)
- E49: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E49: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E49: limitation: threshold is provisional (not calibrated)

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
| E11 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (c6327a317d3d) | harness/H1/run-S1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S1.json (f53a89613307) | trend/T1.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (f53a89613307) | trend/T1.json |
| E14 | alloc-stack | live | go-pprof-heap 0.1.0 | artifact allocs/A1.json (1c2fdfc6525a) | allocs/A1.json |
| E15 | growth-series | live | go-pprof-heap 0.1.0 | artifact series/S2.json (71ab044ffcee) | series/S2.json |
| E16 | workload | static | dla 0.1.0 | repo dd879aa91b6b7202ad9d226ebe261d07e9c68353 | harness/H2/harness.json |
| E17 | workload | test | dla 0.1.0 | artifact harness/H2/harness.json (e0b3c7d97dcd) | harness/H2/run-S3.json |
| E18 | growth-series | test | dla 0.1.0 | artifact series/S3.json (4c6d4fed947f) | trend/T2.json |
| E19 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (4c6d4fed947f) | trend/T2.json |
| E20 | growth-series | test | dla 0.1.0 | artifact series/S4.json (a857843e8bc9) | trend/T3.json |
| E21 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (a857843e8bc9) | trend/T3.json |
| E22 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (71ab044ffcee) | trend/T4.json |
| E23 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (71ab044ffcee) | trend/T4.json |
| E24 | alloc-stack | live | go-pprof-heap 0.1.0 | artifact allocs/A2.json (42061ea40694) | allocs/A2.json |
| E25 | growth-series | live | go-pprof-heap 0.1.0 | artifact series/S5.json (58f94f958120) | series/S5.json |
| E26 | discriminator | test | dla 0.1.0 | artifact series/S6.json (99fe991a0dc3) | alloc/A1.json |
| E27 | discriminator | test | dla 0.1.0 | artifact series/S7.json (0811fac6e829) | alloc/A2.json |
| E28 | discriminator | test | dla 0.1.0 | artifact series/S8.json (02d68a33d286) | alloc/A3.json |
| E29 | discriminator | test | dla 0.1.0 | artifact series/S9.json (479b706d5b73) | alloc/A4.json |
| E30 | discriminator | test | dla 0.1.0 | artifact series/S10.json (a13eef120176) | alloc/A5.json |
| E31 | growth-series | dump | dla 0.1.0 | artifact series/S5.json (58f94f958120) | trend/T10.json |
| E32 | unit-count-series | dump | dla 0.1.0 | artifact series/S5.json (58f94f958120) | trend/T10.json |
| E33 | growth-series | test | dla 0.1.0 | artifact series/S6.json (99fe991a0dc3) | trend/T11.json |
| E34 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (99fe991a0dc3) | trend/T11.json |
| E35 | growth-series | test | dla 0.1.0 | artifact series/S7.json (0811fac6e829) | trend/T12.json |
| E36 | unit-count-series | test | dla 0.1.0 | artifact series/S7.json (0811fac6e829) | trend/T12.json |
| E37 | growth-series | test | dla 0.1.0 | artifact series/S8.json (02d68a33d286) | trend/T13.json |
| E38 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (02d68a33d286) | trend/T13.json |
| E39 | growth-series | test | dla 0.1.0 | artifact series/S9.json (479b706d5b73) | trend/T14.json |
| E40 | unit-count-series | test | dla 0.1.0 | artifact series/S9.json (479b706d5b73) | trend/T14.json |
| E41 | growth-series | test | dla 0.1.0 | artifact series/S10.json (a13eef120176) | trend/T15.json |
| E42 | unit-count-series | test | dla 0.1.0 | artifact series/S10.json (a13eef120176) | trend/T15.json |
| E43 | residual | test | dla 0.1.0 | artifact series/S3.json (4c6d4fed947f) | gap/residual-S3-E43.json |
| E44 | residual | test | dla 0.1.0 | artifact series/S4.json (a857843e8bc9) | gap/residual-S4-E44.json |
| E45 | residual | test | dla 0.1.0 | artifact series/S6.json (99fe991a0dc3) | gap/residual-S6-E45.json |
| E46 | residual | test | dla 0.1.0 | artifact series/S7.json (0811fac6e829) | gap/residual-S7-E46.json |
| E47 | residual | test | dla 0.1.0 | artifact series/S8.json (02d68a33d286) | gap/residual-S8-E47.json |
| E48 | residual | test | dla 0.1.0 | artifact series/S9.json (479b706d5b73) | gap/residual-S9-E48.json |
| E49 | residual | test | dla 0.1.0 | artifact series/S10.json (a13eef120176) | gap/residual-S10-E49.json |
