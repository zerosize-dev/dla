# DLA report

Run: `20261110-175002-4b5b`

Status: INCOMPLETE (48 open)

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 8 enumerated, 8 dispositioned; confirmed 2, strong 0, refuted 2, suspected 2, unresolved 2

Residual: heap 13511.187272727271 bytes/n unattributed of 13511.187272727271 total (threshold 3377.7968181818183) ABOVE THRESHOLD, see O1; heap 13454.012337662338 bytes/n unattributed of 13454.012337662338 total (threshold 3363.5030844155845) ABOVE THRESHOLD, see O1; heap 13499.680735930737 bytes/n unattributed of 13499.680735930737 total (threshold 3374.9201839826837) ABOVE THRESHOLD, see O1; heap 13483.486363636364 bytes/n unattributed of 13483.486363636364 total (threshold 3370.87159090909) ABOVE THRESHOLD, see O1; non-heap 6913.723376623377 bytes/n unattributed of 6913.723376623377 total (threshold 1728.4308441558442) ABOVE THRESHOLD, see O2; non-heap 9189.41181818182 bytes/n unattributed of 9189.41181818182 total (threshold 2297.352954545455) ABOVE THRESHOLD, see O2; non-heap 8156.648051948052 bytes/n unattributed of 8156.648051948052 total (threshold 2039.162012987013) ABOVE THRESHOLD, see O2; non-heap 8163.257644628099 bytes/n unattributed of 8163.257644628099 total (threshold 2040.8144111570248) ABOVE THRESHOLD, see O2

Coverage: jvm 3520 iterations, 19 of 19 entry points exercised

## Open items

- **T7:jvm.class:byte[].count**: column jvm.class:byte[].count of T7 (steady) is not flat and is tied to no finding. Fix: `dla trend S7`
- **T7:jvm.class:java.lang.Class$ReflectionData.count**: column jvm.class:java.lang.Class$ReflectionData.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla trend S7`
- **T7:jvm.class:java.lang.Class.count**: column jvm.class:java.lang.Class.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla trend S7`
- **T7:jvm.class:java.lang.Object.count**: column jvm.class:java.lang.Object.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla trend S7`
- **T7:jvm.class:java.lang.Object[].count**: column jvm.class:java.lang.Object[].count of T7 (steady) is not flat and is tied to no finding. Fix: `dla trend S7`
- **T7:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla trend S7`
- **T7:jvm.class:java.lang.invoke.MemberName.count**: column jvm.class:java.lang.invoke.MemberName.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla trend S7`
- **T7:jvm.class:java.lang.ref.SoftReference.count**: column jvm.class:java.lang.ref.SoftReference.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla trend S7`
- **T7:jvm.class:java.lang.reflect.Field.count**: column jvm.class:java.lang.reflect.Field.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla trend S7`
- **T7:jvm.class:java.lang.reflect.Field[].count**: column jvm.class:java.lang.reflect.Field[].count of T7 (steady) is not flat and is tied to no finding. Fix: `dla trend S7`
- **T7:jvm.class:java.security.CodeSource.count**: column jvm.class:java.security.CodeSource.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla trend S7`
- **T7:jvm.class:java.security.Principal[].count**: column jvm.class:java.security.Principal[].count of T7 (steady) is not flat and is tied to no finding. Fix: `dla trend S7`
- **T7:jvm.class:java.security.ProtectionDomain$Key.count**: column jvm.class:java.security.ProtectionDomain$Key.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla trend S7`
- **T7:jvm.class:java.security.ProtectionDomain.count**: column jvm.class:java.security.ProtectionDomain.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla trend S7`
- **T7:jvm.class:java.util.ArrayList.count**: column jvm.class:java.util.ArrayList.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla trend S7`
- **T7:jvm.class:java.util.HashMap$Node.count**: column jvm.class:java.util.HashMap$Node.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla trend S7`
- **T7:jvm.class:java.util.HashMap.count**: column jvm.class:java.util.HashMap.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla trend S7`
- **T7:jvm.class:java.util.concurrent.ConcurrentHashMap$Node.count**: column jvm.class:java.util.concurrent.ConcurrentHashMap$Node.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla trend S7`
- **T7:jvm.class:java.util.concurrent.ConcurrentHashMap$Node[].count**: column jvm.class:java.util.concurrent.ConcurrentHashMap$Node[].count of T7 (steady) is not flat and is tied to no finding. Fix: `dla trend S7`
- **T7:jvm.class:java.util.concurrent.ConcurrentHashMap.count**: column jvm.class:java.util.concurrent.ConcurrentHashMap.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla trend S7`
- **T8:os.rss**: column os.rss of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:os.rss_anon**: column os.rss_anon of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:os.vm_hwm**: column os.vm_hwm of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:jvm.classes_loaded**: column jvm.classes_loaded of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:jvm.heap_used**: column jvm.heap_used of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:jvm.nonheap_committed**: column jvm.nonheap_committed of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:jvm.nonheap_used**: column jvm.nonheap_used of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:jvm.heap_used.pre_gc**: column jvm.heap_used.pre_gc of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:jvm.class:byte[].count**: column jvm.class:byte[].count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E33 # column jvm.class:byte[].count`
- **T8:jvm.class:java.lang.Class$ReflectionData.count**: column jvm.class:java.lang.Class$ReflectionData.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E33 # column jvm.class:java.lang.Class$ReflectionData.count`
- **T8:jvm.class:java.lang.Class.count**: column jvm.class:java.lang.Class.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E33 # column jvm.class:java.lang.Class.count`
- **T8:jvm.class:java.lang.Object.count**: column jvm.class:java.lang.Object.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E33 # column jvm.class:java.lang.Object.count`
- **T8:jvm.class:java.lang.Object[].count**: column jvm.class:java.lang.Object[].count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E33 # column jvm.class:java.lang.Object[].count`
- **T8:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E33 # column jvm.class:java.lang.String.count`
- **T8:jvm.class:java.lang.invoke.MemberName.count**: column jvm.class:java.lang.invoke.MemberName.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E33 # column jvm.class:java.lang.invoke.MemberName.count`
- **T8:jvm.class:java.lang.ref.SoftReference.count**: column jvm.class:java.lang.ref.SoftReference.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E33 # column jvm.class:java.lang.ref.SoftReference.count`
- **T8:jvm.class:java.lang.reflect.Field.count**: column jvm.class:java.lang.reflect.Field.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E33 # column jvm.class:java.lang.reflect.Field.count`
- **T8:jvm.class:java.lang.reflect.Field[].count**: column jvm.class:java.lang.reflect.Field[].count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E33 # column jvm.class:java.lang.reflect.Field[].count`
- **T8:jvm.class:java.security.CodeSource.count**: column jvm.class:java.security.CodeSource.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E33 # column jvm.class:java.security.CodeSource.count`
- **T8:jvm.class:java.security.Principal[].count**: column jvm.class:java.security.Principal[].count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E33 # column jvm.class:java.security.Principal[].count`
- **T8:jvm.class:java.security.ProtectionDomain$Key.count**: column jvm.class:java.security.ProtectionDomain$Key.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E33 # column jvm.class:java.security.ProtectionDomain$Key.count`
- **T8:jvm.class:java.security.ProtectionDomain.count**: column jvm.class:java.security.ProtectionDomain.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E33 # column jvm.class:java.security.ProtectionDomain.count`
- **T8:jvm.class:java.util.ArrayList.count**: column jvm.class:java.util.ArrayList.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E33 # column jvm.class:java.util.ArrayList.count`
- **T8:jvm.class:java.util.HashMap$Node.count**: column jvm.class:java.util.HashMap$Node.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E33 # column jvm.class:java.util.HashMap$Node.count`
- **T8:jvm.class:java.util.HashMap.count**: column jvm.class:java.util.HashMap.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E33 # column jvm.class:java.util.HashMap.count`
- **T8:jvm.class:java.util.concurrent.ConcurrentHashMap$Node.count**: column jvm.class:java.util.concurrent.ConcurrentHashMap$Node.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E33 # column jvm.class:java.util.concurrent.ConcurrentHashMap$Node.count`
- **T8:jvm.class:java.util.concurrent.ConcurrentHashMap$Node[].count**: column jvm.class:java.util.concurrent.ConcurrentHashMap$Node[].count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E33 # column jvm.class:java.util.concurrent.ConcurrentHashMap$Node[].count`
- **T8:jvm.class:java.util.concurrent.ConcurrentHashMap.count**: column jvm.class:java.util.concurrent.ConcurrentHashMap.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E33 # column jvm.class:java.util.concurrent.ConcurrentHashMap.count`

## Findings

### F3: **L-REGISTER**

- State: confirmed
- Label: L-REGISTER (evidence: E11 unit-count-series)
- Growth: `probe:reports.EventBus.LISTENERS` 1 count/n (steady, gate pass, E11)
- Evidence classes: context, discriminator, growth, static
- Holder: registration-table
- Owner: `reports.EventBus.LISTENERS`
- Region: heap
- Shape: steady
- Site: `src/reports/EventBus.java:8`
- Writers: `src/reports/EventBus.java:13`, `ext/ledger/LedgerExtension.java:22`
- Removers: `src/reports/EventBus.java:17`, `ext/ledger/LedgerExtension.java:27`
- Annotation: heuristic name match over the project's sources
- Annotation: EventBus.LISTENERS holds Tally per reload; ExtensionManager.reload() drops loader/current without calling current.stop()
- Verified: no
- Sibling hypotheses: F3.n1
- Supporting evidence: E6, E9, E11, E14
- Explanation (inferred): EventBus.LISTENERS (src/reports/EventBus.java:8) holds each LedgerExtension Tally subscribed in start() at ext/ledger/LedgerExtension.java:22; reload() drops the old extension without calling stop() (ext/ledger/LedgerExtension.java:27), so removal is never reached.
- Explanation structure: adds: ext/ledger/LedgerExtension.java:22; removal: unreachable at ext/ledger/LedgerExtension.java:27

### F6: **L-REGISTER**

- State: confirmed
- Label: L-REGISTER (evidence: E18 unit-count-series)
- Growth: `probe:reports.EventBus.LISTENERS` 1 count/n (steady, gate pass, E18)
- Evidence classes: context, discriminator, growth
- Holder: registration-table
- Owner: `reports.EventBus.LISTENERS`
- Region: heap
- Shape: steady
- Site: `src/reports/EventBus.java:8`
- Writers: `ext/ledger/LedgerExtension.java:22`
- Removers: `ext/ledger/LedgerExtension.java:27`
- Annotation: Class/byte[]/long[] growth is the per-reload URLClassLoader and ROUNDING table retained through the leaked Tally
- Verified: no
- Sibling hypotheses: F6.n1
- Supporting evidence: E15, E16, E18, E20, E9, E31
- Explanation (inferred): EventBus.LISTENERS (src/reports/EventBus.java:8) holds each LedgerExtension Tally subscribed in start() at ext/ledger/LedgerExtension.java:22; reload() drops the old extension without calling stop() (ext/ledger/LedgerExtension.java:27), so removal is never reached.
- Explanation structure: adds: ext/ledger/LedgerExtension.java:22; removal: unreachable at ext/ledger/LedgerExtension.java:27

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `ledger.LedgerExtension.ROUNDING`
- Region: heap
- Site: `ext/ledger/LedgerExtension.java:11`
- Reason: not_exercised
- Remedy: ROUNDING is a per-classloader static table; it is retained only via F3 and is not a separate leak
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `ledger.LedgerExtension.start`
- Region: heap
- Site: `ext/ledger/LedgerExtension.java:22`
- Reason: not_exercised
- Remedy: start() subscription is the writer side of F3, not a separate leak
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: retention
- Holder: static/global
- Owner: `dla.harness.DlaDump.FIELDS`
- Region: heap
- Reason: not_exercised
- Remedy: dla.harness.DlaDump.FIELDS is the DLA instrument itself (N-OBSERVER), not project code
- Supporting evidence: E24

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: retention
- Holder: static/global
- Owner: `dla.harness.DlaDump.SHALLOW`
- Region: heap
- Reason: not_exercised
- Remedy: dla.harness.DlaDump.SHALLOW is the DLA instrument itself (N-OBSERVER), not project code
- Supporting evidence: E24

## Ruled out

- **F3.n1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (leak) E14
  - Hypothesis: N-WARM: listener count is a warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E14
  - Originally supported by: E11
- **F6.n1** (UNCLASSIFIED): discriminator E31
  - Ruled out by: plateau+purge (leak) E31
  - Hypothesis: N-WARM: class growth is a warm-up fill
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E31
  - Originally supported by: E18

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
- E9: limitation: command-mode heap dumps come from separate processes: dla summarize heap-graph compares dumps of one process only, so use a driver (dla harness gen --driver) for a dump series
- E9: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E9: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E9: limitation: coverage capped at 4000 classes
- E10: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E10: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E10: limitation: coverage capped at 4000 classes
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E11: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E11: limitation: coverage capped at 4000 classes
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E12: limitation: the files carry no time, so they are ordered as given
- E13: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E13: limitation: the files carry no time, so they are ordered as given
- E13: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: coverage capped at 4000 classes
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: points 1 to 9 come from S1; the extra points from S3
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E15: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E16: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: coverage capped at 4000 classes
- E17: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: coverage capped at 4000 classes
- E18: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: coverage capped at 4000 classes
- E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: points 1 to 9 come from S1; the extra points from S3
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: coverage capped at 4000 classes
- E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: points 1 to 9 come from S1; the extra points from S3
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E22: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E23: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E24: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E25: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E25: limitation: series: per-point; one process per point at n = 440, 1760, 2640, 3960 (harness captures; checked: HPROF version, identifier size)
- E26: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E27: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E28: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E29: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap dump
- E29: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E29: limitation: holders absent from a dump's 200 largest have no figure at that point
- E29: limitation: gate thresholds are provisional (not calibrated)
- E30: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap dump
- E30: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E30: limitation: holders absent from a dump's 200 largest have no figure at that point
- E30: limitation: gate thresholds are provisional (not calibrated)
- E31: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E31: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E31: limitation: coverage capped at 4000 classes
- E31: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E31: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E31: limitation: points 1 to 9 come from S1; the extra points from S6
- E31: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E31: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E31: limitation: gate and plateau thresholds are provisional (not calibrated)
- E32: limitation: coverage capped at 4000 classes
- E32: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E32: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E32: limitation: gate thresholds are provisional (not calibrated)
- E33: limitation: coverage capped at 4000 classes
- E33: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E33: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E33: limitation: gate thresholds are provisional (not calibrated)
- E34: partial coverage (2 of 3 examined)
- E34: limitation: F3: growth measured as a count; bytes not attributed
- E34: limitation: F6: growth evidence is from S2, not S1; not subtracted
- E34: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E34: limitation: threshold is provisional (not calibrated)
- E35: partial coverage (2 of 3 examined)
- E35: limitation: F3: growth evidence is from S1, not S3; not subtracted
- E35: limitation: F6: growth measured as a count; bytes not attributed
- E35: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E35: limitation: threshold is provisional (not calibrated)
- E36: partial coverage (2 of 3 examined)
- E36: limitation: F3: growth evidence is from S1, not S4; not subtracted
- E36: limitation: F6: growth measured as a count; bytes not attributed
- E36: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E36: limitation: threshold is provisional (not calibrated)
- E37: partial coverage (2 of 3 examined)
- E37: limitation: F3: growth evidence is from S1, not S7; not subtracted
- E37: limitation: F6: growth evidence is from S2, not S7; not subtracted
- E37: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E37: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | — |
| E2 | run-config | static | dla 0.1.0 | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | probe/probe.json |
| E4 | static-hit | static | spotbugs unknown | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | sweep/jvm-spotbugs.json |
| E5 | static-hit | static | dla 0.1.0 | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | sweep/jvm-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | census/jvm.json |
| E7 | workload | static | dla 0.1.0 | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (868033a85dec) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (bdec61306a48) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (bdec61306a48) | trend/T1.json |
| E12 | histogram-diff | live | jcmd-class-histogram 0.1.0 | artifact histograms/H1.json (2efc58851b78) | histograms/H1.json |
| E13 | growth-series | live | jcmd-class-histogram 0.1.0 | artifact series/S2.json (1978908b9abe) | series/S2.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S4.json (8d32f88ee9b2) | alloc/A1.json |
| E15 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (1978908b9abe) | trend/T3.json |
| E16 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (1978908b9abe) | trend/T3.json |
| E17 | growth-series | test | dla 0.1.0 | artifact series/S3.json (699d42b6cfe7) | trend/T4.json |
| E18 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (699d42b6cfe7) | trend/T4.json |
| E19 | growth-series | test | dla 0.1.0 | artifact series/S4.json (8d32f88ee9b2) | trend/T5.json |
| E20 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (8d32f88ee9b2) | trend/T5.json |
| E21 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-175002-4b5b/harness/H1/out/p1-native/capture/105/heap-1.hprof (04194933fa5b) | summaries/heap-1-04194933.json |
| E22 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-175002-4b5b/harness/H1/out/p4-native/capture/466/heap-1.hprof (21ed2748050b) | summaries/heap-1-21ed2748.json |
| E23 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-175002-4b5b/harness/H1/out/p6-native/capture/707/heap-1.hprof (6551b3fcbc34) | summaries/heap-1-6551b3fc.json |
| E24 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-175002-4b5b/harness/H1/out/p9-native/capture/1069/heap-1.hprof (101b3212c4e9) | summaries/heap-1-101b3212.json |
| E25 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S5.json (79c9983302dd) | series/S5.json |
| E26 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S5.json (79c9983302dd) | series/S5.json |
| E27 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S5.json (79c9983302dd) | series/S5.json |
| E28 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S5.json (79c9983302dd) | series/S5.json |
| E29 | growth-series | dump | dla 0.1.0 | artifact series/S5.json (79c9983302dd) | trend/T6.json |
| E30 | unit-count-series | dump | dla 0.1.0 | artifact series/S5.json (79c9983302dd) | trend/T6.json |
| E31 | discriminator | test | dla 0.1.0 | artifact series/S7.json (e53282625152) | alloc/A2.json |
| E32 | growth-series | test | dla 0.1.0 | artifact series/S6.json (622cd6796e0d) | trend/T8.json |
| E33 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (622cd6796e0d) | trend/T8.json |
| E34 | residual | test | dla 0.1.0 | artifact series/S1.json (bdec61306a48) | gap/residual-S1-E34.json |
| E35 | residual | test | dla 0.1.0 | artifact series/S3.json (699d42b6cfe7) | gap/residual-S3-E35.json |
| E36 | residual | test | dla 0.1.0 | artifact series/S4.json (8d32f88ee9b2) | gap/residual-S4-E36.json |
| E37 | residual | test | dla 0.1.0 | artifact series/S7.json (e53282625152) | gap/residual-S7-E37.json |
