# DLA report

Run: `20261110-195333-2269`

Status: INCOMPLETE (6 open)

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 8 enumerated, 8 dispositioned; confirmed 0, strong 0, refuted 4, suspected 0, unresolved 4

Residual: heap 0.0936936936936937 bytes/n unattributed of 0.0936936936936937 total (threshold 182.5189362162162); heap 0.04954954954954955 bytes/n unattributed of 0.04954954954954955 total (threshold 0.34718027027027026); heap 0 bytes/n unattributed of -0.01096096096096096 total (threshold 0.2680701081081081); heap 1.1474474474474476 bytes/n unattributed of 1.1474474474474476 total (threshold 0.6475164864863976) ABOVE THRESHOLD, see O3; heap 0.9427027027027026 bytes/n unattributed of 0.9427027027027026 total (threshold 0.889666854054029) ABOVE THRESHOLD, see O3; heap 0.5603603603603604 bytes/n unattributed of 0.5603603603603604 total (threshold 1.7253113513513512); heap 1.042042042042042 bytes/n unattributed of 1.042042042042042 total (threshold 1.0128873873874082) ABOVE THRESHOLD, see O2; heap 1.6366366366366365 bytes/n unattributed of 1.6366366366366365 total (threshold 1.2162281081081523) ABOVE THRESHOLD, see O2; heap 1.1505105105105105 bytes/n unattributed of 1.1505105105105105 total (threshold 1.6336560144143852); heap 0 bytes/n unattributed of -0.2601801801801802 total (threshold 2.4289452972972705); heap 0.6144144144144144 bytes/n unattributed of 0.6144144144144144 total (threshold 1.3669304864864864); non-heap 30.05945945945946 bytes/n unattributed of 30.05945945945946 total (threshold 1055.077692972973); non-heap 0 bytes/n unattributed of -1.3423423423423424 total (threshold 8.409032432432433); non-heap 0 bytes/n unattributed of -2.9570141570141573 total (threshold 9.783251891891863); non-heap 4.957528957528957 bytes/n unattributed of 4.957528957528957 total (threshold 24.245293899613973); non-heap 17.453453453453452 bytes/n unattributed of 17.453453453453452 total (threshold 26.2515478198199); non-heap 10.35984555984556 bytes/n unattributed of 10.35984555984556 total (threshold 40.72873111969096); non-heap 29.197897897897896 bytes/n unattributed of 29.197897897897896 total (threshold 33.29959670270257); non-heap 11.964264264264266 bytes/n unattributed of 11.964264264264266 total (threshold 30.30695810810802); non-heap 21.993993993993993 bytes/n unattributed of 21.993993993993993 total (threshold 38.53264043243255); non-heap 0 bytes/n unattributed of -13.54054054054054 total (threshold 81.0896335135135); non-heap 14.59009009009009 bytes/n unattributed of 14.59009009009009 total (threshold 51.23798816216216)

Coverage: jvm 8880 iterations, 15 of 15 entry points exercised

## Open items

- **T18:jvm.class:java.util.concurrent.ConcurrentHashMap$Node.count**: column jvm.class:java.util.concurrent.ConcurrentHashMap$Node.count of T18 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E52 # column jvm.class:java.util.concurrent.ConcurrentHashMap$Node.count`
- **T18:jvm.class:jdk.internal.util.WeakReferenceKey.count**: column jvm.class:jdk.internal.util.WeakReferenceKey.count of T18 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E52 # column jvm.class:jdk.internal.util.WeakReferenceKey.count`
- **T19:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T19 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E55 # column jvm.class:java.lang.String.count`
- **T19:jvm.class:jdk.internal.util.WeakReferenceKey.count**: column jvm.class:jdk.internal.util.WeakReferenceKey.count of T19 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E55 # column jvm.class:jdk.internal.util.WeakReferenceKey.count`
- **T20:jvm.class:byte[].count**: column jvm.class:byte[].count of T20 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E60 # column jvm.class:byte[].count`
- **T20:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T20 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E60 # column jvm.class:java.lang.String.count`

## Findings

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: container-field
- Owner: `shop.OrderCache.ORDERS_BY_ID`
- Region: heap
- Reason: not_exercised
- Remedy: String/byte[] class-count drift in a short run; not tied to a holder and the cache plateaus at 1000; rerun longer with driver heap dumps
- Supporting evidence: E34

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: container-field
- Owner: `shop.OrderCache.ORDERS_BY_ID`
- Region: heap
- Reason: not_exercised
- Remedy: String/byte[] class-count drift in a short run; not tied to a holder and the cache plateaus at 1000; rerun longer with driver heap dumps
- Supporting evidence: E37

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: container-field
- Owner: `shop.OrderCache.ORDERS_BY_ID`
- Region: heap
- Reason: not_exercised
- Remedy: String/byte[] class-count drift in a short run; not tied to a holder and the cache plateaus at 1000; rerun longer with driver heap dumps
- Supporting evidence: E43

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: container-field
- Owner: `shop.OrderCache.ORDERS_BY_ID`
- Region: heap
- Site: `src/shop/OrderCache.java:25`
- Reason: not_exercised
- Remedy: String/byte[] class-count drift in a short run; not tied to a holder and the cache plateaus at 1000; rerun longer with driver heap dumps
- Supporting evidence: E34

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E20
  - Ruled out by: plateau+purge (non-leak) E20
  - Contradicting evidence: E20
  - Originally supported by: E6
- **F2** (UNCLASSIFIED): discriminator E21
  - Ruled out by: plateau+purge (non-leak) E21
  - Contradicting evidence: E21
  - Originally supported by: E6
- **F3** (UNCLASSIFIED): discriminator E22
  - Ruled out by: plateau+purge (non-leak) E22
  - Contradicting evidence: E22
  - Originally supported by: E6
- **F4** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (non-leak) E12
  - Contradicting evidence: E12
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
- E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: points 1 to 9 come from S1; the extra points from S2
- E12: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E12: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E12: limitation: gate and plateau thresholds are provisional (not calibrated)
- E13: partial coverage (2 of 3 examined)
- E13: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E13: limitation: threshold is provisional (not calibrated)
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: partial coverage (2 of 3 examined)
- E16: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E16: limitation: threshold is provisional (not calibrated)
- E17: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: points 1 to 9 come from S1; the extra points from S4
- E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E17: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E17: limitation: gate and plateau thresholds are provisional (not calibrated)
- E18: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: points 1 to 9 come from S1; the extra points from S6
- E20: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E20: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E20: limitation: gate and plateau thresholds are provisional (not calibrated)
- E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: points 1 to 9 come from S1; the extra points from S8
- E21: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E21: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E21: limitation: gate and plateau thresholds are provisional (not calibrated)
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: points 1 to 9 come from S1; the extra points from S10
- E22: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E22: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E22: limitation: gate and plateau thresholds are provisional (not calibrated)
- E23: partial coverage (2 of 3 examined)
- E23: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E23: limitation: threshold is provisional (not calibrated)
- E24: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E24: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E25: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: partial coverage (2 of 3 examined)
- E26: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E26: limitation: threshold is provisional (not calibrated)
- E27: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E27: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E28: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: partial coverage (2 of 3 examined)
- E29: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E29: limitation: threshold is provisional (not calibrated)
- E30: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E30: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E30: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E30: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E30: limitation: points 1 to 9 come from S1; the extra points from S2
- E30: limitation: gate thresholds are provisional (not calibrated)
- E31: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E31: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E31: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E31: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E31: limitation: points 1 to 9 come from S1; the extra points from S2
- E31: limitation: gate thresholds are provisional (not calibrated)
- E32: partial coverage (2 of 3 examined)
- E32: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E32: limitation: threshold is provisional (not calibrated)
- E33: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E33: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E33: limitation: gate thresholds are provisional (not calibrated)
- E34: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E34: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E34: limitation: gate thresholds are provisional (not calibrated)
- E35: partial coverage (2 of 3 examined)
- E35: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E35: limitation: O2 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E35: limitation: threshold is provisional (not calibrated)
- E36: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E36: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E36: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E36: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E36: limitation: points 1 to 9 come from S1; the extra points from S4
- E36: limitation: gate thresholds are provisional (not calibrated)
- E37: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E37: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E37: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E37: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E37: limitation: points 1 to 9 come from S1; the extra points from S4
- E37: limitation: gate thresholds are provisional (not calibrated)
- E38: partial coverage (2 of 3 examined)
- E38: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E38: limitation: threshold is provisional (not calibrated)
- E39: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E39: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E39: limitation: gate thresholds are provisional (not calibrated)
- E40: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E40: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E40: limitation: gate thresholds are provisional (not calibrated)
- E41: partial coverage (2 of 3 examined)
- E41: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E41: limitation: threshold is provisional (not calibrated)
- E42: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E42: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E42: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E42: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E42: limitation: points 1 to 9 come from S1; the extra points from S6
- E42: limitation: gate thresholds are provisional (not calibrated)
- E43: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E43: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E43: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E43: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E43: limitation: points 1 to 9 come from S1; the extra points from S6
- E43: limitation: gate thresholds are provisional (not calibrated)
- E44: partial coverage (2 of 3 examined)
- E44: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E44: limitation: O2 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E44: limitation: threshold is provisional (not calibrated)
- E45: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E45: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E45: limitation: gate thresholds are provisional (not calibrated)
- E46: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E46: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E46: limitation: gate thresholds are provisional (not calibrated)
- E47: partial coverage (2 of 3 examined)
- E47: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E47: limitation: O2 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E47: limitation: threshold is provisional (not calibrated)
- E48: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E48: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E48: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E48: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E48: limitation: points 1 to 9 come from S1; the extra points from S8
- E48: limitation: gate thresholds are provisional (not calibrated)
- E49: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E49: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E49: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E49: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E49: limitation: points 1 to 9 come from S1; the extra points from S8
- E49: limitation: gate thresholds are provisional (not calibrated)
- E50: partial coverage (2 of 3 examined)
- E50: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E50: limitation: O2 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E50: limitation: threshold is provisional (not calibrated)
- E51: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E51: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E51: limitation: gate thresholds are provisional (not calibrated)
- E52: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E52: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E52: limitation: gate thresholds are provisional (not calibrated)
- E53: partial coverage (2 of 3 examined)
- E53: limitation: off-heap: no gap series for S10; run dla gap S10 first (node.external is used when present)
- E53: limitation: O2 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E53: limitation: threshold is provisional (not calibrated)
- E54: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E54: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E54: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E54: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E54: limitation: points 1 to 9 come from S1; the extra points from S10
- E54: limitation: gate thresholds are provisional (not calibrated)
- E55: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E55: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E55: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E55: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E55: limitation: points 1 to 9 come from S1; the extra points from S10
- E55: limitation: gate thresholds are provisional (not calibrated)
- E56: partial coverage (2 of 3 examined)
- E56: limitation: off-heap: no gap series for S11; run dla gap S11 first (node.external is used when present)
- E56: limitation: O2 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E56: limitation: threshold is provisional (not calibrated)
- E57: partial coverage (2 of 3 examined)
- E57: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E57: limitation: threshold is provisional (not calibrated)
- E58: partial coverage (2 of 3 examined)
- E58: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E58: limitation: threshold is provisional (not calibrated)
- E59: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E59: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E59: limitation: gate thresholds are provisional (not calibrated)
- E60: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E60: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E60: limitation: gate thresholds are provisional (not calibrated)

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
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (f227a2d6938b) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (04a63b95b3f0) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (04a63b95b3f0) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (8c29d226d277) | alloc/A1.json |
| E13 | residual | test | dla 0.1.0 | artifact series/S1.json (04a63b95b3f0) | gap/residual-S1-E13.json |
| E14 | growth-series | test | dla 0.1.0 | artifact series/S2.json (41bacebc8cbb) | trend/T3.json |
| E15 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (41bacebc8cbb) | trend/T3.json |
| E16 | residual | test | dla 0.1.0 | artifact series/S3.json (8c29d226d277) | gap/residual-S3-E16.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S5.json (e9ad874f760b) | alloc/A2.json |
| E18 | growth-series | test | dla 0.1.0 | artifact series/S4.json (7d26522cfaf3) | trend/T5.json |
| E19 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (7d26522cfaf3) | trend/T5.json |
| E20 | discriminator | test | dla 0.1.0 | artifact series/S7.json (86c9fe069959) | alloc/A3.json |
| E21 | discriminator | test | dla 0.1.0 | artifact series/S9.json (62a618c57865) | alloc/A4.json |
| E22 | discriminator | test | dla 0.1.0 | artifact series/S11.json (5f06eef7f3e1) | alloc/A5.json |
| E23 | residual | test | dla 0.1.0 | artifact series/S4.json (7d26522cfaf3) | gap/residual-S4-E23.json |
| E24 | growth-series | test | dla 0.1.0 | artifact series/S1.json (04a63b95b3f0) | trend/T9.json |
| E25 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (04a63b95b3f0) | trend/T9.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S1.json (04a63b95b3f0) | gap/residual-S1-E26.json |
| E27 | growth-series | test | dla 0.1.0 | artifact series/S2.json (41bacebc8cbb) | trend/T10.json |
| E28 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (41bacebc8cbb) | trend/T10.json |
| E29 | residual | test | dla 0.1.0 | artifact series/S2.json (41bacebc8cbb) | gap/residual-S2-E29.json |
| E30 | growth-series | test | dla 0.1.0 | artifact series/S3.json (8c29d226d277) | trend/T11.json |
| E31 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (8c29d226d277) | trend/T11.json |
| E32 | residual | test | dla 0.1.0 | artifact series/S3.json (8c29d226d277) | gap/residual-S3-E32.json |
| E33 | growth-series | test | dla 0.1.0 | artifact series/S4.json (7d26522cfaf3) | trend/T12.json |
| E34 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (7d26522cfaf3) | trend/T12.json |
| E35 | residual | test | dla 0.1.0 | artifact series/S4.json (7d26522cfaf3) | gap/residual-S4-E35.json |
| E36 | growth-series | test | dla 0.1.0 | artifact series/S5.json (e9ad874f760b) | trend/T13.json |
| E37 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (e9ad874f760b) | trend/T13.json |
| E38 | residual | test | dla 0.1.0 | artifact series/S5.json (e9ad874f760b) | gap/residual-S5-E38.json |
| E39 | growth-series | test | dla 0.1.0 | artifact series/S6.json (56a8266dd083) | trend/T14.json |
| E40 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (56a8266dd083) | trend/T14.json |
| E41 | residual | test | dla 0.1.0 | artifact series/S6.json (56a8266dd083) | gap/residual-S6-E41.json |
| E42 | growth-series | test | dla 0.1.0 | artifact series/S7.json (86c9fe069959) | trend/T15.json |
| E43 | unit-count-series | test | dla 0.1.0 | artifact series/S7.json (86c9fe069959) | trend/T15.json |
| E44 | residual | test | dla 0.1.0 | artifact series/S7.json (86c9fe069959) | gap/residual-S7-E44.json |
| E45 | growth-series | test | dla 0.1.0 | artifact series/S8.json (14d4ba3df621) | trend/T16.json |
| E46 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (14d4ba3df621) | trend/T16.json |
| E47 | residual | test | dla 0.1.0 | artifact series/S8.json (14d4ba3df621) | gap/residual-S8-E47.json |
| E48 | growth-series | test | dla 0.1.0 | artifact series/S9.json (62a618c57865) | trend/T17.json |
| E49 | unit-count-series | test | dla 0.1.0 | artifact series/S9.json (62a618c57865) | trend/T17.json |
| E50 | residual | test | dla 0.1.0 | artifact series/S9.json (62a618c57865) | gap/residual-S9-E50.json |
| E51 | growth-series | test | dla 0.1.0 | artifact series/S10.json (b1c16603ac72) | trend/T18.json |
| E52 | unit-count-series | test | dla 0.1.0 | artifact series/S10.json (b1c16603ac72) | trend/T18.json |
| E53 | residual | test | dla 0.1.0 | artifact series/S10.json (b1c16603ac72) | gap/residual-S10-E53.json |
| E54 | growth-series | test | dla 0.1.0 | artifact series/S11.json (5f06eef7f3e1) | trend/T19.json |
| E55 | unit-count-series | test | dla 0.1.0 | artifact series/S11.json (5f06eef7f3e1) | trend/T19.json |
| E56 | residual | test | dla 0.1.0 | artifact series/S11.json (5f06eef7f3e1) | gap/residual-S11-E56.json |
| E57 | residual | test | dla 0.1.0 | artifact series/S2.json (41bacebc8cbb) | gap/residual-S2-E57.json |
| E58 | residual | test | dla 0.1.0 | artifact series/S3.json (8c29d226d277) | gap/residual-S3-E58.json |
| E59 | growth-series | test | dla 0.1.0 | artifact series/S6.json (56a8266dd083) | trend/T20.json |
| E60 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (56a8266dd083) | trend/T20.json |
