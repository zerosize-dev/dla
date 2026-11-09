# DLA report

Run: `20261109-203558-e63f`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 5 enumerated, 5 dispositioned; confirmed 0, strong 0, refuted 5, suspected 0, unresolved 0

Residual: not measured

Coverage: go 7040 iterations, 0 of 0 entry points exercised

## Findings

No findings to report.

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E17
  - Ruled out by: plateau+purge (non-leak) E17
  - Contradicting evidence: E17
  - Originally supported by: E8
- **F2** (UNCLASSIFIED): discriminator E18
  - Ruled out by: plateau+purge (non-leak) E18
  - Contradicting evidence: E18
  - Originally supported by: E8
- **F3** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (non-leak) E16
  - Contradicting evidence: E16
  - Originally supported by: E8
- **F4** (UNCLASSIFIED): discriminator E19
  - Ruled out by: plateau+purge (non-leak) E19
  - Contradicting evidence: E19
  - Originally supported by: E8
- **F5** (UNCLASSIFIED): discriminator E20
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

None recorded.

### Evidence limitations and unknowns

- E8: partial coverage
- E8: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E8: limitation: writers and removers are textual name matches
- E9: limitation: harness written; nothing was run (dla harness run measures it)
- E9: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E9: limitation: goleak not available offline: the goleak check is skipped
- E10: partial coverage (9 of 9 examined)
- E10: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E10: limitation: no entry points are listed for this runtime: run dla workload discover before the harness to see which ones ran
- E11: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: a Go heap profile is a sample (runtime.MemProfileRate): sizes are estimates, and objects smaller than the rate can be missed
- E13: limitation: in-use figures are as of the last completed collection: write the profile after runtime.GC()
- E13: limitation: a profile carries no holder: a stack ties to a finding only through its first project frame being the finding's site or writer
- E13: limitation: the profile is sampled: counts and bytes are estimates
- E14: limitation: a Go heap profile is a sample (runtime.MemProfileRate): sizes are estimates, and objects smaller than the rate can be missed
- E14: limitation: in-use figures are as of the last completed collection: write the profile after runtime.GC()
- E14: limitation: a profile carries no holder: a stack ties to a finding only through its first project frame being the finding's site or writer
- E14: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: goleak not available offline: the goleak check is skipped
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: goleak not available offline: the goleak check is skipped
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E17: limitation: goleak not available offline: the goleak check is skipped
- E17: limitation: gate and plateau thresholds are provisional (not calibrated)
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E18: limitation: goleak not available offline: the goleak check is skipped
- E18: limitation: gate and plateau thresholds are provisional (not calibrated)
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E19: limitation: goleak not available offline: the goleak check is skipped
- E19: limitation: gate and plateau thresholds are provisional (not calibrated)
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E20: limitation: goleak not available offline: the goleak check is skipped
- E20: limitation: gate and plateau thresholds are provisional (not calibrated)

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
| E9 | workload | static | dla 0.1.0 | repo dd879aa91b6b7202ad9d226ebe261d07e9c68353 | harness/H1/harness.json |
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (336a0c2f7b03) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (6ba5adf3c9b5) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (6ba5adf3c9b5) | trend/T1.json |
| E13 | alloc-stack | live | go-pprof-heap 0.1.0 | artifact allocs/A1.json (20619f06df99) | allocs/A1.json |
| E14 | growth-series | live | go-pprof-heap 0.1.0 | artifact series/S3.json (03a3e8821310) | series/S3.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S4.json (276873864dba) | alloc/A1.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S5.json (a4084d8d80f7) | alloc/A2.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S6.json (0b6a0fc6083d) | alloc/A3.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S7.json (ba351c274013) | alloc/A4.json |
| E19 | discriminator | test | dla 0.1.0 | artifact series/S8.json (20a7882f7125) | alloc/A5.json |
| E20 | discriminator | test | dla 0.1.0 | artifact series/S9.json (69d6d21f7726) | alloc/A6.json |
