# DLA report

Run: `20261110-180923-76c0`

Status: INCOMPLETE (3 open)

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 5 enumerated, 5 dispositioned; confirmed 0, strong 0, refuted 5, suspected 0, unresolved 0

Residual: heap 4.599134199134199 bytes/n unattributed of 4.599134199134199 total (threshold 3.04992) ABOVE THRESHOLD, see O2; heap 2.408471074380165 bytes/iteration unattributed of 2.408471074380165 total (threshold 2.327325552216447) ABOVE THRESHOLD, see O2; non-heap 0 bytes/n unattributed of 0 total (threshold 0); non-heap 0 bytes/iteration unattributed of 0 total (threshold 0)

Coverage: go 5280 iterations, 9 of 11 entry points exercised

## Open items

- **T8:os.vm_hwm**: column os.vm_hwm of T8 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:go.released**: column go.released of T8 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:go.heap_objects.pre_gc**: column go.heap_objects.pre_gc of T8 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`

## Findings

No findings to report.

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E21
  - Ruled out by: plateau+purge (non-leak) E21
  - Contradicting evidence: E21
  - Originally supported by: E8
- **F2** (UNCLASSIFIED): discriminator E22
  - Ruled out by: plateau+purge (non-leak) E22
  - Contradicting evidence: E22
  - Originally supported by: E8
- **F3** (UNCLASSIFIED): discriminator E23
  - Ruled out by: plateau+purge (non-leak) E23
  - Contradicting evidence: E23
  - Originally supported by: E8
- **F4** (UNCLASSIFIED): discriminator E24
  - Ruled out by: plateau+purge (non-leak) E24
  - Contradicting evidence: E24
  - Originally supported by: E8
- **F5** (UNCLASSIFIED): discriminator E25
  - Ruled out by: plateau+purge (non-leak) E25
  - Contradicting evidence: E25
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
- E11: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: harness written; nothing was run (dla harness run measures it)
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: goleak not available offline: the goleak check is skipped
- E15: partial coverage (9 of 9 examined)
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: no Go coverage profile was written to cov/
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: a Go heap profile is a sample (runtime.MemProfileRate): sizes are estimates, and objects smaller than the rate can be missed
- E18: limitation: in-use figures are as of the last completed collection: write the profile after runtime.GC()
- E18: limitation: a profile carries no holder: a stack ties to a finding only through its first project frame being the finding's site or writer
- E18: limitation: the profile is sampled: counts and bytes are estimates
- E19: limitation: a Go heap profile is a sample (runtime.MemProfileRate): sizes are estimates, and objects smaller than the rate can be missed
- E19: limitation: in-use figures are as of the last completed collection: write the profile after runtime.GC()
- E19: limitation: a profile carries no holder: a stack ties to a finding only through its first project frame being the finding's site or writer
- E19: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E20: limitation: goleak not available offline: the goleak check is skipped
- E20: limitation: gate and plateau thresholds are provisional (not calibrated)
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E21: limitation: goleak not available offline: the goleak check is skipped
- E21: limitation: gate and plateau thresholds are provisional (not calibrated)
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E22: limitation: goleak not available offline: the goleak check is skipped
- E22: limitation: gate and plateau thresholds are provisional (not calibrated)
- E23: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E23: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E23: limitation: goleak not available offline: the goleak check is skipped
- E23: limitation: gate and plateau thresholds are provisional (not calibrated)
- E24: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E24: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E24: limitation: goleak not available offline: the goleak check is skipped
- E24: limitation: gate and plateau thresholds are provisional (not calibrated)
- E25: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E25: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E25: limitation: goleak not available offline: the goleak check is skipped
- E25: limitation: gate and plateau thresholds are provisional (not calibrated)
- E26: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E26: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E27: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: partial coverage (2 of 3 examined)
- E28: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E28: limitation: threshold is provisional (not calibrated)
- E29: partial coverage (2 of 3 examined)
- E29: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E29: limitation: threshold is provisional (not calibrated)
- E30: partial coverage (2 of 3 examined)
- E30: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E30: limitation: threshold is provisional (not calibrated)
- E31: partial coverage (2 of 3 examined)
- E31: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E31: limitation: threshold is provisional (not calibrated)

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
| E11 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (015cfd888764) | harness/H1/run-S1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S1.json (176fc3f6c4c8) | trend/T1.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (176fc3f6c4c8) | trend/T1.json |
| E14 | workload | static | dla 0.1.0 | repo dd879aa91b6b7202ad9d226ebe261d07e9c68353 | harness/H2/harness.json |
| E15 | workload | test | dla 0.1.0 | artifact harness/H2/harness.json (b5816d595809) | harness/H2/run-S2.json |
| E16 | growth-series | test | dla 0.1.0 | artifact series/S2.json (76d3ba3b81d8) | trend/T2.json |
| E17 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (76d3ba3b81d8) | trend/T2.json |
| E18 | alloc-stack | live | go-pprof-heap 0.1.0 | artifact allocs/A1.json (2c9865aebf37) | allocs/A1.json |
| E19 | growth-series | live | go-pprof-heap 0.1.0 | artifact series/S3.json (b3eba3c39bce) | series/S3.json |
| E20 | discriminator | test | dla 0.1.0 | artifact series/S4.json (3834e0e8ecde) | alloc/A1.json |
| E21 | discriminator | test | dla 0.1.0 | artifact series/S5.json (4796c9e9b853) | alloc/A2.json |
| E22 | discriminator | test | dla 0.1.0 | artifact series/S6.json (ce80033d8151) | alloc/A3.json |
| E23 | discriminator | test | dla 0.1.0 | artifact series/S7.json (c3558d863547) | alloc/A4.json |
| E24 | discriminator | test | dla 0.1.0 | artifact series/S8.json (ee39877605e1) | alloc/A5.json |
| E25 | discriminator | test | dla 0.1.0 | artifact series/S9.json (7c779ee548d0) | alloc/A6.json |
| E26 | growth-series | dump | dla 0.1.0 | artifact series/S3.json (b3eba3c39bce) | trend/T9.json |
| E27 | unit-count-series | dump | dla 0.1.0 | artifact series/S3.json (b3eba3c39bce) | trend/T9.json |
| E28 | residual | test | dla 0.1.0 | artifact series/S1.json (176fc3f6c4c8) | gap/residual-S1-E28.json |
| E29 | residual | test | dla 0.1.0 | artifact series/S7.json (c3558d863547) | gap/residual-S7-E29.json |
| E30 | residual | test | dla 0.1.0 | artifact series/S1.json (176fc3f6c4c8) | gap/residual-S1-E30.json |
| E31 | residual | test | dla 0.1.0 | artifact series/S7.json (c3558d863547) | gap/residual-S7-E31.json |
