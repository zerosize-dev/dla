# DLA report

Run: `20261110-185956-41a0`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 3 enumerated, 3 dispositioned; confirmed 0, strong 0, refuted 3, suspected 0, unresolved 0

Residual: heap 1.0357575757575757 bytes/iteration unattributed of 1.0357575757575757 total (threshold 0.4270658181818028) ABOVE THRESHOLD, see O3; heap 1.123896103896104 bytes/iteration unattributed of 1.123896103896104 total (threshold 0.663621662337672) ABOVE THRESHOLD, see O3; heap 2.2685314685314686 bytes/iteration unattributed of 2.2685314685314686 total (threshold 1.1991434685314648) ABOVE THRESHOLD, see O3; non-heap 0 bytes/iteration unattributed of 0 total (threshold 0); non-heap 0 bytes/iteration unattributed of 0 total (threshold 0); non-heap 15.958441558441558 bytes/iteration unattributed of 15.958441558441558 total (threshold 7.09799563636365) ABOVE THRESHOLD, see O4

Coverage: go 7040 iterations, 0 of 19 entry points exercised

## Findings

No findings to report.

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (non-leak) E16
  - Contradicting evidence: E16
  - Originally supported by: E8
- **F2** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (non-leak) E16
  - Contradicting evidence: E16
  - Originally supported by: E8
- **F3** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (non-leak) E16
  - Contradicting evidence: E16
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
- E17: partial coverage (2 of 3 examined)
- E17: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E17: limitation: threshold is provisional (not calibrated)
- E18: partial coverage (2 of 3 examined)
- E18: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E18: limitation: threshold is provisional (not calibrated)
- E19: partial coverage (2 of 3 examined)
- E19: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E19: limitation: threshold is provisional (not calibrated)
- E20: partial coverage (2 of 3 examined)
- E20: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E20: limitation: threshold is provisional (not calibrated)
- E21: partial coverage (2 of 3 examined)
- E21: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E21: limitation: threshold is provisional (not calibrated)
- E22: partial coverage (2 of 3 examined)
- E22: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E22: limitation: threshold is provisional (not calibrated)

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
| E11 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (90552a1e1b21) | harness/H1/run-S1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S1.json (debe0da66273) | trend/T1.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (debe0da66273) | trend/T1.json |
| E14 | growth-series | test | dla 0.1.0 | artifact series/S2.json (01f21feda600) | trend/T2.json |
| E15 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (01f21feda600) | trend/T2.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S3.json (7c61cc3c2cd8) | alloc/A1.json |
| E17 | residual | test | dla 0.1.0 | artifact series/S1.json (debe0da66273) | gap/residual-S1-E17.json |
| E18 | residual | test | dla 0.1.0 | artifact series/S2.json (01f21feda600) | gap/residual-S2-E18.json |
| E19 | residual | test | dla 0.1.0 | artifact series/S3.json (7c61cc3c2cd8) | gap/residual-S3-E19.json |
| E20 | residual | test | dla 0.1.0 | artifact series/S1.json (debe0da66273) | gap/residual-S1-E20.json |
| E21 | residual | test | dla 0.1.0 | artifact series/S2.json (01f21feda600) | gap/residual-S2-E21.json |
| E22 | residual | test | dla 0.1.0 | artifact series/S3.json (7c61cc3c2cd8) | gap/residual-S3-E22.json |
