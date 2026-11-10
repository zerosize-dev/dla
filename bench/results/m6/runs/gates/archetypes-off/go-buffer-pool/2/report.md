# DLA report

Run: `20261110-184411-ee6f`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 7 enumerated, 7 dispositioned; confirmed 0, strong 0, refuted 6, suspected 0, unresolved 1

Residual: heap 5.718181818181819 bytes/n unattributed of 5.718181818181819 total (threshold 1.429545454545455) ABOVE THRESHOLD, see O1; heap 1.8464580873671783 bytes/iteration unattributed of 1.8464580873671783 total (threshold 1.739289347107503) ABOVE THRESHOLD, see O1; heap 1.956060606060606 bytes/iteration unattributed of 1.956060606060606 total (threshold 1.523708454545455) ABOVE THRESHOLD, see O1; heap 1.613986013986014 bytes/iteration unattributed of 1.613986013986014 total (threshold 1.8575008111888471); heap 6.922727272727273 bytes/iteration unattributed of 6.922727272727273 total (threshold 1.7306818181818182) ABOVE THRESHOLD, see O1; heap 3.2290043290043293 bytes/iteration unattributed of 3.2290043290043293 total (threshold 3.0083851948052107) ABOVE THRESHOLD, see O1; heap 0 bytes/iteration unattributed of -0.023636363636363636 total (threshold 0.9467074909091224); heap 0 bytes/n unattributed of -1.1545454545454543 total (threshold 2.8361945454545454); heap 2.0115702479338844 bytes/n unattributed of 2.0115702479338844 total (threshold 1.638701851239672) ABOVE THRESHOLD, see O1; heap 1.9681818181818185 bytes/iteration unattributed of 1.9681818181818185 total (threshold 1.8054024545454543) ABOVE THRESHOLD, see O1; non-heap 0 bytes/n unattributed of 0 total (threshold 0); non-heap 0 bytes/iteration unattributed of 0 total (threshold 0); non-heap 0 bytes/iteration unattributed of 0 total (threshold 0); non-heap 0 bytes/iteration unattributed of 0 total (threshold 0); non-heap 0 bytes/iteration unattributed of 0 total (threshold 0); non-heap 0 bytes/iteration unattributed of 0 total (threshold 0); non-heap 0 bytes/iteration unattributed of 0 total (threshold 0); non-heap 0 bytes/n unattributed of 0 total (threshold 0); non-heap 0 bytes/n unattributed of 0 total (threshold 0); non-heap 0 bytes/iteration unattributed of 0 total (threshold 0)

Coverage: go 7040 iterations, 9 of 11 entry points exercised

## Findings

### F3.n1: Hypothesis N-HWM: idle pool retains peak batch worth of buffers, bounded by maxBatch (400), plateaus (UNCLASSIFIED)

- State: unresolved(not_exercised)
- Evidence classes: discriminator
- Sibling hypothesis of: F3
- Hypothesis: N-HWM: idle pool retains peak batch worth of buffers, bounded by maxBatch (400), plateaus
- Refuted if: idle pool/heap_live keeps growing across extra cycles
- Reason: not_exercised
- Remedy: N-HWM is the supported explanation (alloc-test E24 plateau); no GoVar probe can reach the struct field BufferPool.free, so idle bytes were not measured directly
- Supporting evidence: E24

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E30
  - Ruled out by: plateau+purge (non-leak) E30
  - Contradicting evidence: E30
  - Originally supported by: E8
- **F2** (UNCLASSIFIED): discriminator E31
  - Ruled out by: plateau+purge (non-leak) E31
  - Contradicting evidence: E31
  - Originally supported by: E8
- **F3** (UNCLASSIFIED): discriminator E24
  - Ruled out by: plateau+purge (non-leak) E24
  - Contradicting evidence: E24
  - Originally supported by: E8
- **F4** (UNCLASSIFIED): discriminator E32
  - Ruled out by: plateau+purge (non-leak) E32
  - Contradicting evidence: E32
  - Originally supported by: E8
- **F5** (UNCLASSIFIED): discriminator E33
  - Ruled out by: plateau+purge (non-leak) E33
  - Contradicting evidence: E33
  - Originally supported by: E8
- **F6** (UNCLASSIFIED): discriminator E29
  - Ruled out by: plateau+purge (non-leak) E29
  - Contradicting evidence: E29
  - Originally supported by: E25, E26

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
- E14: limitation: harness written; nothing was run (dla harness run measures it)
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: goleak not available offline: the goleak check is skipped
- E15: limitation: a Go heap profile is a sample (runtime.MemProfileRate): sizes are estimates, and objects smaller than the rate can be missed
- E15: limitation: in-use figures are as of the last completed collection: write the profile after runtime.GC()
- E15: limitation: a profile carries no holder: a stack ties to a finding only through its first project frame being the finding's site or writer
- E15: limitation: the profile is sampled: counts and bytes are estimates
- E16: limitation: a Go heap profile is a sample (runtime.MemProfileRate): sizes are estimates, and objects smaller than the rate can be missed
- E16: limitation: in-use figures are as of the last completed collection: write the profile after runtime.GC()
- E16: limitation: a profile carries no holder: a stack ties to a finding only through its first project frame being the finding's site or writer
- E16: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
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
- E22: limitation: a Go heap profile is a sample (runtime.MemProfileRate): sizes are estimates, and objects smaller than the rate can be missed
- E22: limitation: in-use figures are as of the last completed collection: write the profile after runtime.GC()
- E22: limitation: a profile carries no holder: a stack ties to a finding only through its first project frame being the finding's site or writer
- E22: limitation: the profile is sampled: counts and bytes are estimates
- E23: limitation: a Go heap profile is a sample (runtime.MemProfileRate): sizes are estimates, and objects smaller than the rate can be missed
- E23: limitation: in-use figures are as of the last completed collection: write the profile after runtime.GC()
- E23: limitation: a profile carries no holder: a stack ties to a finding only through its first project frame being the finding's site or writer
- E23: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E24: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E24: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E24: limitation: goleak not available offline: the goleak check is skipped
- E24: limitation: gate and plateau thresholds are provisional (not calibrated)
- E25: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E25: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E26: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E27: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E28: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E29: limitation: points 1 to 9 come from S1; the extra points from S7
- E29: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E29: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E29: limitation: gate and plateau thresholds are provisional (not calibrated)
- E30: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E30: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E30: limitation: goleak not available offline: the goleak check is skipped
- E30: limitation: gate and plateau thresholds are provisional (not calibrated)
- E31: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E31: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E31: limitation: goleak not available offline: the goleak check is skipped
- E31: limitation: gate and plateau thresholds are provisional (not calibrated)
- E32: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E32: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E32: limitation: goleak not available offline: the goleak check is skipped
- E32: limitation: gate and plateau thresholds are provisional (not calibrated)
- E33: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E33: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E33: limitation: goleak not available offline: the goleak check is skipped
- E33: limitation: gate and plateau thresholds are provisional (not calibrated)
- E34: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E34: limitation: gate thresholds are provisional (not calibrated)
- E35: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E35: limitation: gate thresholds are provisional (not calibrated)
- E36: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E36: limitation: points 1 to 9 come from S1; the extra points from S7
- E36: limitation: gate thresholds are provisional (not calibrated)
- E37: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E37: limitation: points 1 to 9 come from S1; the extra points from S7
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
- E43: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E43: limitation: gate thresholds are provisional (not calibrated)
- E44: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E44: limitation: gate thresholds are provisional (not calibrated)
- E45: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E45: limitation: gate thresholds are provisional (not calibrated)
- E46: partial coverage (2 of 3 examined)
- E46: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E46: limitation: threshold is provisional (not calibrated)
- E47: partial coverage (2 of 3 examined)
- E47: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E47: limitation: threshold is provisional (not calibrated)
- E48: partial coverage (2 of 3 examined)
- E48: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E48: limitation: threshold is provisional (not calibrated)
- E49: partial coverage (2 of 3 examined)
- E49: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E49: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E49: limitation: threshold is provisional (not calibrated)
- E50: partial coverage (2 of 3 examined)
- E50: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E50: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E50: limitation: threshold is provisional (not calibrated)
- E51: partial coverage (2 of 3 examined)
- E51: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E51: limitation: threshold is provisional (not calibrated)
- E52: partial coverage (2 of 3 examined)
- E52: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E52: limitation: threshold is provisional (not calibrated)
- E53: partial coverage (2 of 3 examined)
- E53: limitation: off-heap: no gap series for S10; run dla gap S10 first (node.external is used when present)
- E53: limitation: threshold is provisional (not calibrated)
- E54: partial coverage (2 of 3 examined)
- E54: limitation: off-heap: no gap series for S11; run dla gap S11 first (node.external is used when present)
- E54: limitation: threshold is provisional (not calibrated)
- E55: partial coverage (2 of 3 examined)
- E55: limitation: off-heap: no gap series for S12; run dla gap S12 first (node.external is used when present)
- E55: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E55: limitation: threshold is provisional (not calibrated)

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
| E11 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (e4a9ef060e73) | harness/H1/run-S1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S1.json (774213608017) | trend/T1.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (774213608017) | trend/T1.json |
| E14 | workload | static | dla 0.1.0 | repo dd879aa91b6b7202ad9d226ebe261d07e9c68353 | harness/H2/harness.json |
| E15 | alloc-stack | live | go-pprof-heap 0.1.0 | artifact allocs/A1.json (89a7dc5e7082) | allocs/A1.json |
| E16 | growth-series | live | go-pprof-heap 0.1.0 | artifact series/S2.json (dae759211233) | series/S2.json |
| E17 | workload | test | dla 0.1.0 | artifact harness/H2/harness.json (34b1020e2b19) | harness/H2/run-S3.json |
| E18 | growth-series | test | dla 0.1.0 | artifact series/S3.json (39ef27ba99bb) | trend/T2.json |
| E19 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (39ef27ba99bb) | trend/T2.json |
| E20 | growth-series | test | dla 0.1.0 | artifact series/S4.json (bfd0a7c28235) | trend/T3.json |
| E21 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (bfd0a7c28235) | trend/T3.json |
| E22 | alloc-stack | live | go-pprof-heap 0.1.0 | artifact allocs/A2.json (f194f8b7d88d) | allocs/A2.json |
| E23 | growth-series | live | go-pprof-heap 0.1.0 | artifact series/S5.json (64b6d498d8a9) | series/S5.json |
| E24 | discriminator | test | dla 0.1.0 | artifact series/S6.json (ddbeab1b0c05) | alloc/A1.json |
| E25 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (dae759211233) | trend/T5.json |
| E26 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (dae759211233) | trend/T5.json |
| E27 | growth-series | dump | dla 0.1.0 | artifact series/S5.json (64b6d498d8a9) | trend/T6.json |
| E28 | unit-count-series | dump | dla 0.1.0 | artifact series/S5.json (64b6d498d8a9) | trend/T6.json |
| E29 | discriminator | test | dla 0.1.0 | artifact series/S8.json (5718ad8a1334) | alloc/A2.json |
| E30 | discriminator | test | dla 0.1.0 | artifact series/S9.json (798e785deefa) | alloc/A3.json |
| E31 | discriminator | test | dla 0.1.0 | artifact series/S10.json (03d7d198449b) | alloc/A4.json |
| E32 | discriminator | test | dla 0.1.0 | artifact series/S11.json (db290286b795) | alloc/A5.json |
| E33 | discriminator | test | dla 0.1.0 | artifact series/S12.json (28669759ea9b) | alloc/A6.json |
| E34 | growth-series | test | dla 0.1.0 | artifact series/S7.json (e5c4b216dfc1) | trend/T12.json |
| E35 | unit-count-series | test | dla 0.1.0 | artifact series/S7.json (e5c4b216dfc1) | trend/T12.json |
| E36 | growth-series | test | dla 0.1.0 | artifact series/S8.json (5718ad8a1334) | trend/T13.json |
| E37 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (5718ad8a1334) | trend/T13.json |
| E38 | growth-series | test | dla 0.1.0 | artifact series/S9.json (798e785deefa) | trend/T14.json |
| E39 | unit-count-series | test | dla 0.1.0 | artifact series/S9.json (798e785deefa) | trend/T14.json |
| E40 | growth-series | test | dla 0.1.0 | artifact series/S10.json (03d7d198449b) | trend/T15.json |
| E41 | unit-count-series | test | dla 0.1.0 | artifact series/S10.json (03d7d198449b) | trend/T15.json |
| E42 | growth-series | test | dla 0.1.0 | artifact series/S11.json (db290286b795) | trend/T16.json |
| E43 | unit-count-series | test | dla 0.1.0 | artifact series/S11.json (db290286b795) | trend/T16.json |
| E44 | growth-series | test | dla 0.1.0 | artifact series/S12.json (28669759ea9b) | trend/T17.json |
| E45 | unit-count-series | test | dla 0.1.0 | artifact series/S12.json (28669759ea9b) | trend/T17.json |
| E46 | residual | test | dla 0.1.0 | artifact series/S1.json (774213608017) | gap/residual-S1-E46.json |
| E47 | residual | test | dla 0.1.0 | artifact series/S3.json (39ef27ba99bb) | gap/residual-S3-E47.json |
| E48 | residual | test | dla 0.1.0 | artifact series/S4.json (bfd0a7c28235) | gap/residual-S4-E48.json |
| E49 | residual | test | dla 0.1.0 | artifact series/S6.json (ddbeab1b0c05) | gap/residual-S6-E49.json |
| E50 | residual | test | dla 0.1.0 | artifact series/S7.json (e5c4b216dfc1) | gap/residual-S7-E50.json |
| E51 | residual | test | dla 0.1.0 | artifact series/S8.json (5718ad8a1334) | gap/residual-S8-E51.json |
| E52 | residual | test | dla 0.1.0 | artifact series/S9.json (798e785deefa) | gap/residual-S9-E52.json |
| E53 | residual | test | dla 0.1.0 | artifact series/S10.json (03d7d198449b) | gap/residual-S10-E53.json |
| E54 | residual | test | dla 0.1.0 | artifact series/S11.json (db290286b795) | gap/residual-S11-E54.json |
| E55 | residual | test | dla 0.1.0 | artifact series/S12.json (28669759ea9b) | gap/residual-S12-E55.json |
