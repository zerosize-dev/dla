# DLA report

Run: `20261110-183211-6aab`

Status: INCOMPLETE (20 open)

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 3 enumerated, 3 dispositioned; confirmed 0, strong 0, refuted 1, suspected 2, unresolved 0

Residual: heap 16.46515151515152 bytes/n unattributed of 16.46515151515152 total (threshold 6.32463681818193) ABOVE THRESHOLD, see O1; heap 16.163636363636364 bytes/n unattributed of 16.163636363636364 total (threshold 4.459352727272727) ABOVE THRESHOLD, see O1; heap 17.166666666666664 bytes/n unattributed of 17.166666666666664 total (threshold 4.291666666666666) ABOVE THRESHOLD, see O1; heap 19.89393939393939 bytes/n unattributed of 19.89393939393939 total (threshold 4.973484848484848) ABOVE THRESHOLD, see O1; heap 18.82479338842975 bytes/n unattributed of 18.82479338842975 total (threshold 4.7061983471074385) ABOVE THRESHOLD, see O1; heap 16.054545454545455 bytes/n unattributed of 16.054545454545455 total (threshold 4.182087272727273) ABOVE THRESHOLD, see O1; heap 16.027272727272727 bytes/n unattributed of 16.027272727272727 total (threshold 27.348193636363632); heap 16.373376623376622 bytes/n unattributed of 16.373376623376622 total (threshold 4.107957272727295) ABOVE THRESHOLD, see O1; heap 24.272727272727273 bytes/n unattributed of 24.272727272727273 total (threshold 6.068181818181818) ABOVE THRESHOLD, see O1; heap 19.695779220779222 bytes/n unattributed of 19.695779220779222 total (threshold 4.923944805194806) ABOVE THRESHOLD, see O1; non-heap 144.05930735930735 bytes/n unattributed of 144.05930735930735 total (threshold 85.44424597402572) ABOVE THRESHOLD, see O2; non-heap 20.513636363636365 bytes/n unattributed of 20.513636363636365 total (threshold 16.47707727272727) ABOVE THRESHOLD, see O2; non-heap 72.91931818181817 bytes/n unattributed of 72.91931818181817 total (threshold 44.90761704545454) ABOVE THRESHOLD, see O2; non-heap 16.420454545454547 bytes/n unattributed of 16.420454545454547 total (threshold 110.76827113636362); non-heap 60.0020979020979 bytes/n unattributed of 60.0020979020979 total (threshold 53.08008667132833) ABOVE THRESHOLD, see O2; non-heap 146.88484848484848 bytes/n unattributed of 146.88484848484848 total (threshold 66.45898909090864) ABOVE THRESHOLD, see O2; non-heap 0 bytes/n unattributed of -32.29333333333334 total (threshold 111.54889854545418); non-heap 51.04275724275725 bytes/n unattributed of 51.04275724275725 total (threshold 60.17119510489509); non-heap 0 bytes/n unattributed of -23.03030303030303 total (threshold 28.082754545454993); non-heap 54.94891774891775 bytes/n unattributed of 54.94891774891775 total (threshold 54.39229381818143) ABOVE THRESHOLD, see O2

Coverage: jvm 3520 iterations, 19 of 19 entry points exercised

## Open items

- **T2:jvm.class:byte[].count**: column jvm.class:byte[].count of T2 (stair) is not flat and is tied to no finding. Fix: `dla trend S3`
- **T2:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T2 (stair) is not flat and is tied to no finding. Fix: `dla trend S3`
- **T4:jvm.class:java.lang.Class.count**: column jvm.class:java.lang.Class.count of T4 (stair) is not flat and is tied to no finding. Fix: `dla trend S7`
- **T5:jvm.class:int[].count**: column jvm.class:int[].count of T5 (stair) is not flat and is tied to no finding. Fix: `dla trend S9`
- **T5:jvm.class:java.lang.Class.count**: column jvm.class:java.lang.Class.count of T5 (stair) is not flat and is tied to no finding. Fix: `dla trend S9`
- **T6:jvm.class:java.lang.Class.count**: column jvm.class:java.lang.Class.count of T6 (stair) is not flat and is tied to no finding. Fix: `dla trend S11`
- **T8:jvm.class:byte[].count**: column jvm.class:byte[].count of T8 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E25 # column jvm.class:byte[].count`
- **T8:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T8 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E25 # column jvm.class:java.lang.String.count`
- **T9:jvm.class:byte[].count**: column jvm.class:byte[].count of T9 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E27 # column jvm.class:byte[].count`
- **T9:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T9 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E27 # column jvm.class:java.lang.String.count`
- **T11:type:jdk.internal.vm.FillerElement[]**: column type:jdk.internal.vm.FillerElement[] of T11 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E30 # column type:jdk.internal.vm.FillerElement[]`
- **T11:type:java.util.concurrent.ConcurrentHashMap$Node[]**: column type:java.util.concurrent.ConcurrentHashMap$Node[] of T11 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E30 # column type:java.util.concurrent.ConcurrentHashMap$Node[]`
- **T11:type:byte[]**: column type:byte[] of T11 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E30 # column type:byte[]`
- **T11:type:byte[].count**: column type:byte[].count of T11 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E31 # column type:byte[].count`
- **T11:type:java.lang.Class**: column type:java.lang.Class of T11 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E30 # column type:java.lang.Class`
- **T11:type:java.lang.Class.count**: column type:java.lang.Class.count of T11 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E31 # column type:java.lang.Class.count`
- **T11:type:java.lang.String**: column type:java.lang.String of T11 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E30 # column type:java.lang.String`
- **T11:type:java.lang.String.count**: column type:java.lang.String.count of T11 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E31 # column type:java.lang.String.count`
- **T11:type:int[]**: column type:int[] of T11 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E30 # column type:int[]`
- **T11:type:int[].count**: column type:int[].count of T11 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E31 # column type:int[].count`

## Findings

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: discriminator, static
- Holder: static/global
- Owner: `ledger.LedgerExtension.ROUNDING`
- Region: heap
- Site: `ext/ledger/LedgerExtension.java:11`
- Reason: not_exercised
- Remedy: ROUNDING is a fixed 1024-entry static final table, rebuilt per loader and collected with it; no holder-tied growth observed. Only whole-heap drift (~15 B/reload) was seen
- Supporting evidence: E6, E21

### F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: discriminator, static
- Holder: registration-table
- Owner: `ledger.LedgerExtension.start`
- Region: heap
- Site: `ext/ledger/LedgerExtension.java:22`
- Reason: not_exercised
- Remedy: subscribe is paired with unsubscribe in stop(); EventBus listener probe stayed at 1 across 7480 reloads; no tied growth
- Supporting evidence: E6, E19

## Ruled out

- **F3** (UNCLASSIFIED): discriminator E19
  - Ruled out by: plateau+purge (non-leak) E19
  - Contradicting evidence: E19
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
- E9: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E9: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E9: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E10: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E10: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E10: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E11: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: points 1 to 9 come from S1; the extra points from S2
- E12: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E12: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E12: limitation: gate and plateau thresholds are provisional (not calibrated)
- E13: partial coverage (2 of 3 examined)
- E13: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E13: limitation: threshold is provisional (not calibrated)
- E14: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: partial coverage (9 of 9 examined)
- E16: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E17: limitation: the files carry no time, so they are ordered as given
- E18: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E18: limitation: the files carry no time, so they are ordered as given
- E18: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E19: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: points 1 to 9 come from S4; the extra points from S6
- E19: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E19: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E19: limitation: gate and plateau thresholds are provisional (not calibrated)
- E20: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: points 1 to 9 come from S4; the extra points from S8
- E20: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E20: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E20: limitation: gate and plateau thresholds are provisional (not calibrated)
- E21: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: points 1 to 9 come from S4; the extra points from S10
- E21: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E21: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E21: limitation: gate and plateau thresholds are provisional (not calibrated)
- E22: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E23: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E24: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E24: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E25: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E25: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E26: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E26: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E27: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E27: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E28: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E28: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E29: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E29: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E29: limitation: gate thresholds are provisional (not calibrated)
- E30: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E30: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E30: limitation: gate thresholds are provisional (not calibrated)
- E31: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E31: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E31: limitation: gate thresholds are provisional (not calibrated)
- E32: partial coverage (2 of 3 examined)
- E32: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E32: limitation: threshold is provisional (not calibrated)
- E33: partial coverage (2 of 3 examined)
- E33: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E33: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E33: limitation: threshold is provisional (not calibrated)
- E34: partial coverage (2 of 3 examined)
- E34: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E34: limitation: threshold is provisional (not calibrated)
- E35: partial coverage (2 of 3 examined)
- E35: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E35: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E35: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E35: limitation: threshold is provisional (not calibrated)
- E36: partial coverage (2 of 3 examined)
- E36: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E36: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E36: limitation: threshold is provisional (not calibrated)
- E37: partial coverage (2 of 3 examined)
- E37: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E37: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E37: limitation: threshold is provisional (not calibrated)
- E38: partial coverage (2 of 3 examined)
- E38: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E38: limitation: threshold is provisional (not calibrated)
- E39: partial coverage (2 of 3 examined)
- E39: limitation: off-heap: no gap series for S10; run dla gap S10 first (node.external is used when present)
- E39: limitation: threshold is provisional (not calibrated)
- E40: partial coverage (2 of 3 examined)
- E40: limitation: off-heap: no gap series for S11; run dla gap S11 first (node.external is used when present)
- E40: limitation: threshold is provisional (not calibrated)
- E41: partial coverage (2 of 3 examined)
- E41: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E41: limitation: threshold is provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | target: repo in cwd, jvm, reload leak symptom |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | scope: repo in cwd, jvm, reload leak symptom |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | symptom: repo in cwd, jvm, reload leak symptom |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | runtimes: repo in cwd, jvm, reload leak symptom |
| `intake.build` | Does the project build locally? | answered | E1 | build: repo in cwd, jvm, reload leak symptom |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | workload: repo in cwd, jvm, reload leak symptom |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | budget: repo in cwd, jvm, reload leak symptom |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | — |
| E2 | run-config | static | dla 0.1.0 | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | probe/probe.json |
| E4 | static-hit | static | spotbugs unknown | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | sweep/jvm-spotbugs.json |
| E5 | static-hit | static | dla 0.1.0 | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | sweep/jvm-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | census/jvm.json |
| E7 | workload | static | dla 0.1.0 | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (a442f90d46c5) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (be60e7bc4d87) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (be60e7bc4d87) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (cbcea2e53d43) | alloc/A1.json |
| E13 | residual | test | dla 0.1.0 | artifact series/S3.json (cbcea2e53d43) | gap/residual-S3-E13.json |
| E14 | growth-series | test | dla 0.1.0 | artifact series/S2.json (a1fed103676c) | trend/T3.json |
| E15 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (a1fed103676c) | trend/T3.json |
| E16 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (d6f963b07442) | harness/H1/run-S4.json |
| E17 | histogram-diff | live | jcmd-class-histogram 0.1.0 | artifact histograms/H1.json (2df8c3fa24ea) | histograms/H1.json |
| E18 | growth-series | live | jcmd-class-histogram 0.1.0 | artifact series/S5.json (8b46defee5d3) | series/S5.json |
| E19 | discriminator | test | dla 0.1.0 | artifact series/S7.json (207a246d6e26) | alloc/A2.json |
| E20 | discriminator | test | dla 0.1.0 | artifact series/S9.json (d0801ed8ee86) | alloc/A3.json |
| E21 | discriminator | test | dla 0.1.0 | artifact series/S11.json (c62100687a09) | alloc/A4.json |
| E22 | growth-series | test | dla 0.1.0 | artifact series/S4.json (f29dfd342f51) | trend/T7.json |
| E23 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (f29dfd342f51) | trend/T7.json |
| E24 | growth-series | test | dla 0.1.0 | artifact series/S6.json (e43b26055402) | trend/T8.json |
| E25 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (e43b26055402) | trend/T8.json |
| E26 | growth-series | test | dla 0.1.0 | artifact series/S8.json (33a0eabb373c) | trend/T9.json |
| E27 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (33a0eabb373c) | trend/T9.json |
| E28 | growth-series | test | dla 0.1.0 | artifact series/S10.json (42640692a740) | trend/T10.json |
| E29 | unit-count-series | test | dla 0.1.0 | artifact series/S10.json (42640692a740) | trend/T10.json |
| E30 | growth-series | dump | dla 0.1.0 | artifact series/S5.json (8b46defee5d3) | trend/T11.json |
| E31 | unit-count-series | dump | dla 0.1.0 | artifact series/S5.json (8b46defee5d3) | trend/T11.json |
| E32 | residual | test | dla 0.1.0 | artifact series/S1.json (be60e7bc4d87) | gap/residual-S1-E32.json |
| E33 | residual | test | dla 0.1.0 | artifact series/S2.json (a1fed103676c) | gap/residual-S2-E33.json |
| E34 | residual | test | dla 0.1.0 | artifact series/S4.json (f29dfd342f51) | gap/residual-S4-E34.json |
| E35 | residual | test | dla 0.1.0 | artifact series/S6.json (e43b26055402) | gap/residual-S6-E35.json |
| E36 | residual | test | dla 0.1.0 | artifact series/S7.json (207a246d6e26) | gap/residual-S7-E36.json |
| E37 | residual | test | dla 0.1.0 | artifact series/S8.json (33a0eabb373c) | gap/residual-S8-E37.json |
| E38 | residual | test | dla 0.1.0 | artifact series/S9.json (d0801ed8ee86) | gap/residual-S9-E38.json |
| E39 | residual | test | dla 0.1.0 | artifact series/S10.json (42640692a740) | gap/residual-S10-E39.json |
| E40 | residual | test | dla 0.1.0 | artifact series/S11.json (c62100687a09) | gap/residual-S11-E40.json |
| E41 | residual | test | dla 0.1.0 | artifact series/S1.json (be60e7bc4d87) | gap/residual-S1-E41.json |
