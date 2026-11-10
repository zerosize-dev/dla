# DLA report

Run: `20261110-160749-eb7b`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 5 enumerated, 5 dispositioned; confirmed 0, strong 0, refuted 5, suspected 0, unresolved 0

Residual: heap 1.596666666666667 bytes/iteration unattributed of 1.596666666666667 total (threshold 0.7252224545453873) ABOVE THRESHOLD, see O5; heap 5.290909090909091 bytes/iteration unattributed of 5.290909090909091 total (threshold 1.3227272727272728) ABOVE THRESHOLD, see O5; heap 0.15333333333333332 bytes/iteration unattributed of 0.15333333333333332 total (threshold 0.8865049454545505); heap 0.9635092180546726 bytes/iteration unattributed of 0.9635092180546726 total (threshold 1.593083389701237); heap 0.34090909090909094 bytes/iteration unattributed of 0.34090909090909094 total (threshold 0.8740600909090909); heap 2.7933566433566437 bytes/iteration unattributed of 2.7933566433566437 total (threshold 2.2204267762237584) ABOVE THRESHOLD, see O5; non-heap 0 bytes/iteration unattributed of 0 total (threshold 0); non-heap 0 bytes/iteration unattributed of 0 total (threshold 0); non-heap 0 bytes/iteration unattributed of 0 total (threshold 0); non-heap 0 bytes/iteration unattributed of 0 total (threshold 0); non-heap 0 bytes/iteration unattributed of 0 total (threshold 0); non-heap 0 bytes/iteration unattributed of 0 total (threshold 0)

Coverage: go 7040 iterations, 0 of 0 entry points exercised

## Findings

No findings to report.

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (non-leak) E16
  - Contradicting evidence: E16
  - Originally supported by: E8
- **F2** (UNCLASSIFIED): discriminator E17
  - Ruled out by: plateau+purge (non-leak) E17
  - Contradicting evidence: E17
  - Originally supported by: E8
- **F3** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (non-leak) E15
  - Contradicting evidence: E15
  - Originally supported by: E8
- **F4** (UNCLASSIFIED): discriminator E18
  - Ruled out by: plateau+purge (non-leak) E18
  - Contradicting evidence: E18
  - Originally supported by: E8
- **F5** (UNCLASSIFIED): discriminator E19
  - Ruled out by: plateau+purge (non-leak) E19
  - Contradicting evidence: E19
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
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: gate thresholds are provisional (not calibrated)
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
- E20: partial coverage (2 of 3 examined)
- E20: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E20: limitation: threshold is provisional (not calibrated)
- E21: partial coverage (2 of 3 examined)
- E21: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E21: limitation: threshold is provisional (not calibrated)
- E22: partial coverage (2 of 3 examined)
- E22: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E22: limitation: threshold is provisional (not calibrated)
- E23: partial coverage (2 of 3 examined)
- E23: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E23: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E23: limitation: threshold is provisional (not calibrated)
- E24: partial coverage (2 of 3 examined)
- E24: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E24: limitation: threshold is provisional (not calibrated)
- E25: partial coverage (2 of 3 examined)
- E25: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E25: limitation: threshold is provisional (not calibrated)
- E26: partial coverage (2 of 3 examined)
- E26: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E26: limitation: O2 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E26: limitation: threshold is provisional (not calibrated)
- E27: partial coverage (2 of 3 examined)
- E27: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E27: limitation: threshold is provisional (not calibrated)
- E28: partial coverage (2 of 3 examined)
- E28: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E28: limitation: threshold is provisional (not calibrated)
- E29: partial coverage (2 of 3 examined)
- E29: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E29: limitation: threshold is provisional (not calibrated)
- E30: partial coverage (2 of 3 examined)
- E30: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E30: limitation: O3 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O3 --as unresolved --reason time_budget --remedy "<text>"
- E30: limitation: threshold is provisional (not calibrated)
- E31: partial coverage (2 of 3 examined)
- E31: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E31: limitation: O3 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O3 --as unresolved --reason time_budget --remedy "<text>"
- E31: limitation: threshold is provisional (not calibrated)
- E32: partial coverage (2 of 3 examined)
- E32: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E32: limitation: O3 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O3 --as unresolved --reason time_budget --remedy "<text>"
- E32: limitation: threshold is provisional (not calibrated)
- E33: partial coverage (2 of 3 examined)
- E33: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E33: limitation: threshold is provisional (not calibrated)
- E34: partial coverage (2 of 3 examined)
- E34: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E34: limitation: threshold is provisional (not calibrated)
- E35: partial coverage (2 of 3 examined)
- E35: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E35: limitation: threshold is provisional (not calibrated)
- E36: partial coverage (2 of 3 examined)
- E36: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E36: limitation: O4 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O4 --as unresolved --reason time_budget --remedy "<text>"
- E36: limitation: threshold is provisional (not calibrated)
- E37: partial coverage (2 of 3 examined)
- E37: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E37: limitation: O4 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O4 --as unresolved --reason time_budget --remedy "<text>"
- E37: limitation: threshold is provisional (not calibrated)
- E38: partial coverage (2 of 3 examined)
- E38: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E38: limitation: O4 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O4 --as unresolved --reason time_budget --remedy "<text>"
- E38: limitation: threshold is provisional (not calibrated)
- E39: partial coverage (2 of 3 examined)
- E39: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E39: limitation: threshold is provisional (not calibrated)
- E40: partial coverage (2 of 3 examined)
- E40: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E40: limitation: threshold is provisional (not calibrated)
- E41: partial coverage (2 of 3 examined)
- E41: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E41: limitation: threshold is provisional (not calibrated)
- E42: partial coverage (2 of 3 examined)
- E42: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E42: limitation: O5 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O5 --as unresolved --reason time_budget --remedy "<text>"
- E42: limitation: threshold is provisional (not calibrated)
- E43: partial coverage (2 of 3 examined)
- E43: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E43: limitation: O5 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O5 --as unresolved --reason time_budget --remedy "<text>"
- E43: limitation: threshold is provisional (not calibrated)
- E44: partial coverage (2 of 3 examined)
- E44: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E44: limitation: O5 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O5 --as unresolved --reason time_budget --remedy "<text>"
- E44: limitation: threshold is provisional (not calibrated)
- E45: partial coverage (2 of 3 examined)
- E45: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E45: limitation: threshold is provisional (not calibrated)

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
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (99b615e92798) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (bfcfc83cef5c) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (bfcfc83cef5c) | trend/T1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S2.json (b98e6d70f3d0) | trend/T2.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (b98e6d70f3d0) | trend/T2.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S3.json (9c76f8ba7487) | alloc/A1.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S4.json (3d501ce82a30) | alloc/A2.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S5.json (3132d69bab5b) | alloc/A3.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S6.json (4a492f514359) | alloc/A4.json |
| E19 | discriminator | test | dla 0.1.0 | artifact series/S7.json (10d2e283017f) | alloc/A5.json |
| E20 | residual | test | dla 0.1.0 | artifact series/S3.json (9c76f8ba7487) | gap/residual-S3-E20.json |
| E21 | residual | test | dla 0.1.0 | artifact series/S1.json (bfcfc83cef5c) | gap/residual-S1-E21.json |
| E22 | residual | test | dla 0.1.0 | artifact series/S2.json (b98e6d70f3d0) | gap/residual-S2-E22.json |
| E23 | residual | test | dla 0.1.0 | artifact series/S3.json (9c76f8ba7487) | gap/residual-S3-E23.json |
| E24 | residual | test | dla 0.1.0 | artifact series/S1.json (bfcfc83cef5c) | gap/residual-S1-E24.json |
| E25 | residual | test | dla 0.1.0 | artifact series/S2.json (b98e6d70f3d0) | gap/residual-S2-E25.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S4.json (3d501ce82a30) | gap/residual-S4-E26.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S6.json (4a492f514359) | gap/residual-S6-E27.json |
| E28 | residual | test | dla 0.1.0 | artifact series/S1.json (bfcfc83cef5c) | gap/residual-S1-E28.json |
| E29 | residual | test | dla 0.1.0 | artifact series/S2.json (b98e6d70f3d0) | gap/residual-S2-E29.json |
| E30 | residual | test | dla 0.1.0 | artifact series/S3.json (9c76f8ba7487) | gap/residual-S3-E30.json |
| E31 | residual | test | dla 0.1.0 | artifact series/S4.json (3d501ce82a30) | gap/residual-S4-E31.json |
| E32 | residual | test | dla 0.1.0 | artifact series/S5.json (3132d69bab5b) | gap/residual-S5-E32.json |
| E33 | residual | test | dla 0.1.0 | artifact series/S6.json (4a492f514359) | gap/residual-S6-E33.json |
| E34 | residual | test | dla 0.1.0 | artifact series/S1.json (bfcfc83cef5c) | gap/residual-S1-E34.json |
| E35 | residual | test | dla 0.1.0 | artifact series/S2.json (b98e6d70f3d0) | gap/residual-S2-E35.json |
| E36 | residual | test | dla 0.1.0 | artifact series/S3.json (9c76f8ba7487) | gap/residual-S3-E36.json |
| E37 | residual | test | dla 0.1.0 | artifact series/S4.json (3d501ce82a30) | gap/residual-S4-E37.json |
| E38 | residual | test | dla 0.1.0 | artifact series/S5.json (3132d69bab5b) | gap/residual-S5-E38.json |
| E39 | residual | test | dla 0.1.0 | artifact series/S6.json (4a492f514359) | gap/residual-S6-E39.json |
| E40 | residual | test | dla 0.1.0 | artifact series/S1.json (bfcfc83cef5c) | gap/residual-S1-E40.json |
| E41 | residual | test | dla 0.1.0 | artifact series/S2.json (b98e6d70f3d0) | gap/residual-S2-E41.json |
| E42 | residual | test | dla 0.1.0 | artifact series/S3.json (9c76f8ba7487) | gap/residual-S3-E42.json |
| E43 | residual | test | dla 0.1.0 | artifact series/S4.json (3d501ce82a30) | gap/residual-S4-E43.json |
| E44 | residual | test | dla 0.1.0 | artifact series/S5.json (3132d69bab5b) | gap/residual-S5-E44.json |
| E45 | residual | test | dla 0.1.0 | artifact series/S6.json (4a492f514359) | gap/residual-S6-E45.json |
