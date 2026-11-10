# DLA report

Run: `20261110-172047-bc67`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 8 enumerated, 8 dispositioned; confirmed 0, strong 0, refuted 4, suspected 0, unresolved 4

Residual: heap 0.9837837837837836 bytes/n unattributed of 0.9837837837837836 total (threshold 1.1231124324324324); heap 1.2833976833976837 bytes/n unattributed of 1.2833976833976837 total (threshold 0.7811249420849039) ABOVE THRESHOLD, see O1; heap 22.518438438438437 bytes/n unattributed of 22.518438438438437 total (threshold 37.94301976216217); heap 0.02432432432432433 bytes/n unattributed of 0.02432432432432433 total (threshold 3.780629999999999); heap 0.7897039897039897 bytes/n unattributed of 0.7897039897039897 total (threshold 2.9239848648648734); heap 0.27555555555555555 bytes/n unattributed of 0.27555555555555555 total (threshold 0.38062940540541423); heap 0.07401247401247402 bytes/n unattributed of 0.07401247401247402 total (threshold 0.18461092723494635); heap 2.1633633633633633 bytes/n unattributed of 2.1633633633633633 total (threshold 1.7833178378377934) ABOVE THRESHOLD, see O1; heap 40.05321321321321 bytes/n unattributed of 40.05321321321321 total (threshold 38.70994692612614) ABOVE THRESHOLD, see O1; non-heap 44.76396396396397 bytes/n unattributed of 44.76396396396397 total (threshold 44.84206702702702); non-heap 0 bytes/n unattributed of -0.33513513513513504 total (threshold 25.491561081081077); non-heap 216.71663663663665 bytes/n unattributed of 216.71663663663665 total (threshold 263.9397715027028); non-heap 0 bytes/n unattributed of -11.34054054054054 total (threshold 106.02250054054052); non-heap 18.954954954954957 bytes/n unattributed of 18.954954954954957 total (threshold 83.42843978378377); non-heap 13.287087087087087 bytes/n unattributed of 13.287087087087087 total (threshold 18.283873513513424); non-heap 2.2822022022022024 bytes/n unattributed of 2.2822022022022024 total (threshold 11.956671831831857); non-heap 10.942342342342345 bytes/n unattributed of 10.942342342342345 total (threshold 36.003710270270275); non-heap 249.96390456390455 bytes/n unattributed of 249.96390456390455 total (threshold 203.57734569646564) ABOVE THRESHOLD, see O2

Coverage: jvm 8880 iterations, 15 of 15 entry points exercised

## Findings

### F5: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: byte[]/String/Class/Object[] histogram counts rise within the short harness window (consistent with the bounded OrderCache filling to its 1000 cap, whose plateau was shown by alloc-test E13); a longer replay beyond the fill is needed to see them flatten
- Supporting evidence: E18

### F6: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: byte[]/String/Class/Object[] histogram counts rise within the short harness window (consistent with the bounded OrderCache filling to its 1000 cap, whose plateau was shown by alloc-test E13); a longer replay beyond the fill is needed to see them flatten
- Supporting evidence: E22

### F7: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: Class-histogram counts rise within the short window (JVM warm-up / bounded cache fill); longer replay needed to see plateau
- Supporting evidence: E31

### F8: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: Class-histogram counts rise within the short window (JVM warm-up / bounded cache fill); longer replay needed to see plateau
- Supporting evidence: E40

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (non-leak) E14
  - Contradicting evidence: E14
  - Originally supported by: E6
- **F2** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (non-leak) E15
  - Contradicting evidence: E15
  - Originally supported by: E6
- **F3** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (non-leak) E16
  - Contradicting evidence: E16
  - Originally supported by: E6
- **F4** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (non-leak) E13
  - Contradicting evidence: E13
  - Originally supported by: E6

## Blind spots and unknowns

### Coverage gaps

- jvm/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- jvm/codeql: missing (codeql is not on PATH or in the tools directory). Install hint: dla install codeql
- jvm/errorprone: missing (Error Prone jars not found in the tools directory (or DLA_ERRORPRONE_HOME)). Install hint: dla install errorprone
- jvm/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)
- jvm/sonar: missing (sonar-scanner is not installed and Sonar rules need a SonarQube or SonarCloud server). Install hint: install sonar-scanner, run a SonarQube or SonarCloud server and set SONAR_HOST_URL and SONAR_TOKEN (dla install sonar prints the steps)
- jvm/valgrind: blocked (Valgrind is not run on the JVM: the virtual machine's own allocations drown the project's; use the allocator shim, jcmd NMT or `dla attach bpftrace`)

### Runtime blind spots

None recorded.

### Entry points not exercisable

None recorded.

### Evidence limitations and unknowns

- E4: partial coverage
- E4: limitation: no auxiliary classpath: library calls are not modelled
- E6: partial coverage
- E6: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E6: limitation: writers and removers are textual name matches
- E7: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E7: limitation: workloads were discovered by reading files; none was run
- E8: limitation: harness written; nothing was run (dla harness run measures it)
- E8: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E8: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E9: partial coverage (9 of 9 examined)
- E9: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E9: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E10: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E10: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E11: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: partial coverage (2 of 3 examined)
- E12: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E12: limitation: threshold is provisional (not calibrated)
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: points 1 to 9 come from S1; the extra points from S2
- E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E13: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: points 1 to 9 come from S1; the extra points from S4
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: points 1 to 9 come from S1; the extra points from S6
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: points 1 to 9 come from S1; the extra points from S8
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: partial coverage (2 of 3 examined)
- E19: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E19: limitation: threshold is provisional (not calibrated)
- E20: partial coverage (2 of 3 examined)
- E20: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E20: limitation: threshold is provisional (not calibrated)
- E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: points 1 to 9 come from S1; the extra points from S2
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: points 1 to 9 come from S1; the extra points from S2
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: partial coverage (2 of 3 examined)
- E23: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E23: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E23: limitation: threshold is provisional (not calibrated)
- E24: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E24: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E25: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: partial coverage (2 of 3 examined)
- E26: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E26: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E26: limitation: threshold is provisional (not calibrated)
- E27: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E27: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E27: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E27: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E27: limitation: points 1 to 9 come from S1; the extra points from S4
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E28: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E28: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E28: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E28: limitation: points 1 to 9 come from S1; the extra points from S4
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: partial coverage (2 of 3 examined)
- E29: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E29: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E29: limitation: threshold is provisional (not calibrated)
- E30: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E30: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E30: limitation: gate thresholds are provisional (not calibrated)
- E31: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E31: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E31: limitation: gate thresholds are provisional (not calibrated)
- E32: partial coverage (2 of 3 examined)
- E32: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E32: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E32: limitation: threshold is provisional (not calibrated)
- E33: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E33: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E33: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E33: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E33: limitation: points 1 to 9 come from S1; the extra points from S6
- E33: limitation: gate thresholds are provisional (not calibrated)
- E34: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E34: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E34: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E34: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E34: limitation: points 1 to 9 come from S1; the extra points from S6
- E34: limitation: gate thresholds are provisional (not calibrated)
- E35: partial coverage (2 of 3 examined)
- E35: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E35: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E35: limitation: threshold is provisional (not calibrated)
- E36: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E36: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E36: limitation: gate thresholds are provisional (not calibrated)
- E37: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E37: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E37: limitation: gate thresholds are provisional (not calibrated)
- E38: partial coverage (2 of 3 examined)
- E38: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E38: limitation: threshold is provisional (not calibrated)
- E39: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E39: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E39: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E39: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E39: limitation: points 1 to 9 come from S1; the extra points from S8
- E39: limitation: gate thresholds are provisional (not calibrated)
- E40: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E40: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E40: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E40: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E40: limitation: points 1 to 9 come from S1; the extra points from S8
- E40: limitation: gate thresholds are provisional (not calibrated)
- E41: partial coverage (2 of 3 examined)
- E41: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E41: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | — |
| E2 | run-config | static | dla 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | probe/probe.json |
| E4 | static-hit | static | spotbugs unknown | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | sweep/jvm-spotbugs.json |
| E5 | static-hit | static | dla 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | sweep/jvm-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | census/jvm.json |
| E7 | workload | static | dla 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (30df8b6a26f2) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (fec9fc1fe435) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (fec9fc1fe435) | trend/T1.json |
| E12 | residual | test | dla 0.1.0 | artifact series/S1.json (fec9fc1fe435) | gap/residual-S1-E12.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S3.json (2a4729eea094) | alloc/A1.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S5.json (36632e494330) | alloc/A2.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S7.json (68c2607ca140) | alloc/A3.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S9.json (28d235fd41f7) | alloc/A4.json |
| E17 | growth-series | test | dla 0.1.0 | artifact series/S2.json (8133b567d570) | trend/T6.json |
| E18 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (8133b567d570) | trend/T6.json |
| E19 | residual | test | dla 0.1.0 | artifact series/S3.json (2a4729eea094) | gap/residual-S3-E19.json |
| E20 | residual | test | dla 0.1.0 | artifact series/S2.json (8133b567d570) | gap/residual-S2-E20.json |
| E21 | growth-series | test | dla 0.1.0 | artifact series/S3.json (2a4729eea094) | trend/T7.json |
| E22 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (2a4729eea094) | trend/T7.json |
| E23 | residual | test | dla 0.1.0 | artifact series/S3.json (2a4729eea094) | gap/residual-S3-E23.json |
| E24 | growth-series | test | dla 0.1.0 | artifact series/S4.json (8bb5c2b1f712) | trend/T8.json |
| E25 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (8bb5c2b1f712) | trend/T8.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S4.json (8bb5c2b1f712) | gap/residual-S4-E26.json |
| E27 | growth-series | test | dla 0.1.0 | artifact series/S5.json (36632e494330) | trend/T9.json |
| E28 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (36632e494330) | trend/T9.json |
| E29 | residual | test | dla 0.1.0 | artifact series/S5.json (36632e494330) | gap/residual-S5-E29.json |
| E30 | growth-series | test | dla 0.1.0 | artifact series/S6.json (b104dc4eb88d) | trend/T10.json |
| E31 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (b104dc4eb88d) | trend/T10.json |
| E32 | residual | test | dla 0.1.0 | artifact series/S6.json (b104dc4eb88d) | gap/residual-S6-E32.json |
| E33 | growth-series | test | dla 0.1.0 | artifact series/S7.json (68c2607ca140) | trend/T11.json |
| E34 | unit-count-series | test | dla 0.1.0 | artifact series/S7.json (68c2607ca140) | trend/T11.json |
| E35 | residual | test | dla 0.1.0 | artifact series/S7.json (68c2607ca140) | gap/residual-S7-E35.json |
| E36 | growth-series | test | dla 0.1.0 | artifact series/S8.json (12fc01dfb542) | trend/T12.json |
| E37 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (12fc01dfb542) | trend/T12.json |
| E38 | residual | test | dla 0.1.0 | artifact series/S8.json (12fc01dfb542) | gap/residual-S8-E38.json |
| E39 | growth-series | test | dla 0.1.0 | artifact series/S9.json (28d235fd41f7) | trend/T13.json |
| E40 | unit-count-series | test | dla 0.1.0 | artifact series/S9.json (28d235fd41f7) | trend/T13.json |
| E41 | residual | test | dla 0.1.0 | artifact series/S9.json (28d235fd41f7) | gap/residual-S9-E41.json |
