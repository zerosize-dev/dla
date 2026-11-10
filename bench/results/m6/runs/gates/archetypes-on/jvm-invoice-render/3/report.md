# DLA report

Run: `20261110-180029-f601`

Status: INCOMPLETE (54 open)

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 11 enumerated, 5 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 3

Residual: not measured

Coverage: jvm 8880 iterations, 13 of 13 entry points exercised

## Open items

- **F1**: finding F1 is candidate and has no disposition. Fix: `dla ledger resolve F1 --as unresolved --reason time_budget --remedy "<text>"`
- **F2**: finding F2 is candidate and has no disposition. Fix: `dla ledger resolve F2 --as unresolved --reason time_budget --remedy "<text>"`
- **F3**: finding F3 is candidate and has no disposition. Fix: `dla ledger resolve F3 --as unresolved --reason time_budget --remedy "<text>"`
- **F4**: finding F4 is candidate and has no disposition. Fix: `dla ledger resolve F4 --as unresolved --reason time_budget --remedy "<text>"`
- **F5**: finding F5 is candidate and has no disposition. Fix: `dla ledger resolve F5 --as unresolved --reason time_budget --remedy "<text>"`
- **F7**: finding F7 is candidate and has no disposition. Fix: `dla ledger resolve F7 --as unresolved --reason time_budget --remedy "<text>"`
- **T1:jvm.heap_committed**: column jvm.heap_committed of T1 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:jvm.heap_used**: column jvm.heap_used of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:jvm.heap_committed.pre_gc**: column jvm.heap_committed.pre_gc of T1 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:jvm.heap_used.pre_gc**: column jvm.heap_used.pre_gc of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:jvm.heap_committed**: column jvm.heap_committed of T3 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:jvm.heap_used**: column jvm.heap_used of T3 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:jvm.heap_used.pre_gc**: column jvm.heap_used.pre_gc of T3 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:os.rss**: column os.rss of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:os.rss_anon**: column os.rss_anon of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:os.vm_hwm**: column os.vm_hwm of T4 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:jvm.heap_committed**: column jvm.heap_committed of T4 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:jvm.heap_used**: column jvm.heap_used of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:jvm.heap_committed.pre_gc**: column jvm.heap_committed.pre_gc of T4 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:jvm.heap_used.pre_gc**: column jvm.heap_used.pre_gc of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.rss**: column os.rss of T5 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.rss_anon**: column os.rss_anon of T5 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.vm_data**: column os.vm_data of T5 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.vm_hwm**: column os.vm_hwm of T5 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:jvm.classes_loaded**: column jvm.classes_loaded of T5 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:jvm.heap_committed**: column jvm.heap_committed of T5 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:jvm.heap_used**: column jvm.heap_used of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:jvm.heap_committed.pre_gc**: column jvm.heap_committed.pre_gc of T5 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:jvm.heap_used.pre_gc**: column jvm.heap_used.pre_gc of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:jvm.class:byte[].count**: column jvm.class:byte[].count of T5 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E28 # column jvm.class:byte[].count`
- **T5:jvm.class:java.lang.Class.count**: column jvm.class:java.lang.Class.count of T5 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E28 # column jvm.class:java.lang.Class.count`
- **T5:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T5 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E28 # column jvm.class:java.lang.String.count`
- **T6:holder:static:dla.harness.DlaDump.<resolved_references>**: column holder:static:dla.harness.DlaDump.<resolved_references> of T6 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E29 # column holder:static:dla.harness.DlaDump.<resolved_references>`
- **T6:holder:static:dla.harness.DlaDump.<resolved_references>.count**: column holder:static:dla.harness.DlaDump.<resolved_references>.count of T6 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E30 # column holder:static:dla.harness.DlaDump.<resolved_references>.count`
- **T7:type:byte[]**: column type:byte[] of T7 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E31 # column type:byte[]`
- **T7:type:byte[].count**: column type:byte[].count of T7 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E32 # column type:byte[].count`
- **T7:type:java.lang.String**: column type:java.lang.String of T7 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E31 # column type:java.lang.String`
- **T7:type:java.lang.String.count**: column type:java.lang.String.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E32 # column type:java.lang.String.count`
- **T7:type:java.lang.Object[]**: column type:java.lang.Object[] of T7 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E31 # column type:java.lang.Object[]`
- **T7:type:java.lang.Object[].count**: column type:java.lang.Object[].count of T7 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E32 # column type:java.lang.Object[].count`
- **T7:type:java.lang.invoke.LambdaForm$Name**: column type:java.lang.invoke.LambdaForm$Name of T7 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E31 # column type:java.lang.invoke.LambdaForm$Name`
- **T7:type:java.lang.invoke.LambdaForm$Name.count**: column type:java.lang.invoke.LambdaForm$Name.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E32 # column type:java.lang.invoke.LambdaForm$Name.count`
- **T7:type:java.lang.invoke.LambdaForm$Name[]**: column type:java.lang.invoke.LambdaForm$Name[] of T7 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E31 # column type:java.lang.invoke.LambdaForm$Name[]`
- **T7:type:java.lang.invoke.LambdaForm$Name[].count**: column type:java.lang.invoke.LambdaForm$Name[].count of T7 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E32 # column type:java.lang.invoke.LambdaForm$Name[].count`
- **T7:type:java.lang.invoke.LambdaForm**: column type:java.lang.invoke.LambdaForm of T7 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E31 # column type:java.lang.invoke.LambdaForm`
- **T7:type:java.lang.invoke.LambdaForm.count**: column type:java.lang.invoke.LambdaForm.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E32 # column type:java.lang.invoke.LambdaForm.count`
- **T7:type:java.lang.invoke.LambdaFormEditor$Transform**: column type:java.lang.invoke.LambdaFormEditor$Transform of T7 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E31 # column type:java.lang.invoke.LambdaFormEditor$Transform`
- **T7:type:java.lang.invoke.LambdaFormEditor$Transform.count**: column type:java.lang.invoke.LambdaFormEditor$Transform.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E32 # column type:java.lang.invoke.LambdaFormEditor$Transform.count`
- **T7:type:java.lang.invoke.LambdaForm$NamedFunction**: column type:java.lang.invoke.LambdaForm$NamedFunction of T7 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E31 # column type:java.lang.invoke.LambdaForm$NamedFunction`
- **T7:type:java.lang.invoke.LambdaForm$NamedFunction.count**: column type:java.lang.invoke.LambdaForm$NamedFunction.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E32 # column type:java.lang.invoke.LambdaForm$NamedFunction.count`
- **T7:type:java.lang.invoke.ResolvedMethodName**: column type:java.lang.invoke.ResolvedMethodName of T7 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E31 # column type:java.lang.invoke.ResolvedMethodName`
- **T7:type:java.lang.invoke.ResolvedMethodName.count**: column type:java.lang.invoke.ResolvedMethodName.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E32 # column type:java.lang.invoke.ResolvedMethodName.count`
- **T7:type:java.lang.Class**: column type:java.lang.Class of T7 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E31 # column type:java.lang.Class`
- **T7:type:java.lang.Class.count**: column type:java.lang.Class.count of T7 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E32 # column type:java.lang.Class.count`

## Findings

### F6: **L-TLS**

- State: confirmed
- Label: L-TLS (evidence: E10 growth-series, E18 retention-path)
- Growth: `probe:billing.RenderContext.SEGMENTS` 12.53 count/n (steady, gate pass, E11)
- Evidence classes: context, growth, static
- Holder: thread-local
- Owner: `billing.RenderContext.SEGMENTS`
- Region: heap
- Shape: steady
- Site: `src/billing/RenderContext.java:8`
- Writers: `src/billing/RenderContext.java:13`
- Annotation: heuristic name match over the project's sources
- Verified: no
- Sibling hypotheses: F6.n1
- Supporting evidence: E6, E9, E11
- Explanation (observed): RenderContext.SEGMENTS (RenderContext.java:8) is a ThreadLocal list on the 4 long-lived pool threads; append() at RenderContext.java:13 adds segments per invoice and the list is never cleared, truncated or removed (textFrom only reads a subList), so it grows without bound.
- Explanation structure: adds: src/billing/RenderContext.java:13; removal: never; evidence: E18

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: thread-local
- Owner: `billing.RenderContext.SEGMENTS`
- Region: heap
- Shape: steady
- Site: `src/billing/RenderContext.java:13`
- Reason: not_exercised
- Remedy: Duplicate growth-series entry for the same holder as F6 (RenderContext.SEGMENTS); covered by F6
- Supporting evidence: E11

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: thread-local
- Owner: `billing.RenderContext.SEGMENTS`
- Region: heap
- Shape: steady
- Site: `src/billing/RenderContext.java:13`
- Reason: not_exercised
- Remedy: Duplicate growth-series entry for the same holder as F6 (RenderContext.SEGMENTS); covered by F6
- Supporting evidence: E14

### F10: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: thread-local
- Owner: `billing.RenderContext.SEGMENTS`
- Region: heap
- Shape: steady
- Site: `src/billing/RenderContext.java:13`
- Reason: not_exercised
- Remedy: Duplicate growth-series entry for the same holder as F6 (RenderContext.SEGMENTS); covered by F6
- Supporting evidence: E16

### F1: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: container-field
- Owner: `billing.Invoice.lines`
- Region: heap
- Site: `src/billing/Invoice.java:6`
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: static/global
- Owner: `billing.InvoiceFeed.SKUS`
- Region: heap
- Site: `src/billing/InvoiceFeed.java:9`
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: static/global
- Owner: `billing.InvoiceFeed.NAMES`
- Region: heap
- Site: `src/billing/InvoiceFeed.java:10`
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: blocked-stack
- Owner: `billing.InvoiceService.pool`
- Region: non-heap
- Site: `src/billing/InvoiceService.java:17`
- Supporting evidence: E6

### F5: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: blocked-stack
- Owner: `billing.InvoiceService$WorkerFactory.newThread`
- Region: non-heap
- Site: `src/billing/InvoiceService.java:58`
- Supporting evidence: E6

### F7: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: static/global
- Owner: `billing.TaxTable.RATES`
- Region: heap
- Site: `src/billing/TaxTable.java:7`
- Supporting evidence: E6

## Ruled out

- **F6.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E12
  - Originally supported by: E11

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
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: points 1 to 9 come from S1; the extra points from S2
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: points 1 to 9 come from S1; the extra points from S2
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: partial coverage (9 of 9 examined)
- E17: limitation: command-mode heap dumps come from separate processes: dla summarize heap-graph compares dumps of one process only, so use a driver (dla harness gen --driver) for a dump series
- E17: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E19: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E20: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E21: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E22: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E22: limitation: series: per-point; one process per point at n = 1110, 4440, 6660, 9990 (harness captures; checked: HPROF version, identifier size)
- E23: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E24: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E24: limitation: the files carry no time, so they are ordered as given
- E24: limitation: two captures cannot tell a warm-up fill from a leak: no monotone type is reported and no growth column is written
- E25: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E25: limitation: the files carry no time, so they are ordered as given
- E26: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E26: limitation: the files carry no time, so they are ordered as given
- E26: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E27: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E27: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E28: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: limitation: series: per-point; x is the iteration count n of each point (1110, 4440, 6660, 9990); every point is a separate process and heap dump
- E29: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E29: limitation: holders absent from a dump's 200 largest have no figure at that point
- E29: limitation: gate thresholds are provisional (not calibrated)
- E30: limitation: series: per-point; x is the iteration count n of each point (1110, 4440, 6660, 9990); every point is a separate process and heap dump
- E30: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E30: limitation: holders absent from a dump's 200 largest have no figure at that point
- E30: limitation: gate thresholds are provisional (not calibrated)
- E31: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E31: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E31: limitation: gate thresholds are provisional (not calibrated)
- E32: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E32: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E32: limitation: gate thresholds are provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | — |
| E2 | run-config | static | dla 0.1.0 | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | probe/probe.json |
| E4 | static-hit | static | spotbugs unknown | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | sweep/jvm-spotbugs.json |
| E5 | static-hit | static | dla 0.1.0 | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | sweep/jvm-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | census/jvm.json |
| E7 | workload | static | dla 0.1.0 | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (32f6cd3550a4) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (2b62965c9bcb) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (2b62965c9bcb) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (35167537a685) | alloc/A1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S2.json (48488ca451c3) | trend/T3.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (48488ca451c3) | trend/T3.json |
| E15 | growth-series | test | dla 0.1.0 | artifact series/S3.json (35167537a685) | trend/T4.json |
| E16 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (35167537a685) | trend/T4.json |
| E17 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (111ab9a6885b) | harness/H1/run-S4.json |
| E18 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-180029-f601/harness/H1/out/p1-native/capture/86/heap-1.hprof (53febb08a644) | summaries/heap-1-53febb08.json |
| E19 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-180029-f601/harness/H1/out/p4-native/capture/404/heap-1.hprof (b5702db1ebf8) | summaries/heap-1-b5702db1.json |
| E20 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-180029-f601/harness/H1/out/p6-native/capture/616/heap-1.hprof (fb87b46a8da3) | summaries/heap-1-fb87b46a.json |
| E21 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-180029-f601/harness/H1/out/p9-native/capture/934/heap-1.hprof (6f1ddd2bca34) | summaries/heap-1-6f1ddd2b.json |
| E22 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S5.json (9d0394c83f25) | series/S5.json |
| E23 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S5.json (9d0394c83f25) | series/S5.json |
| E24 | histogram-diff | live | jcmd-class-histogram 0.1.0 | artifact histograms/H1.json (3404bef8efe1) | histograms/H1.json |
| E25 | histogram-diff | live | jcmd-class-histogram 0.1.0 | artifact histograms/H2.json (9d4382cd785e) | histograms/H2.json |
| E26 | growth-series | live | jcmd-class-histogram 0.1.0 | artifact series/S6.json (ede357fc1e53) | series/S6.json |
| E27 | growth-series | test | dla 0.1.0 | artifact series/S4.json (40dc69682b8d) | trend/T5.json |
| E28 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (40dc69682b8d) | trend/T5.json |
| E29 | growth-series | dump | dla 0.1.0 | artifact series/S5.json (9d0394c83f25) | trend/T6.json |
| E30 | unit-count-series | dump | dla 0.1.0 | artifact series/S5.json (9d0394c83f25) | trend/T6.json |
| E31 | growth-series | dump | dla 0.1.0 | artifact series/S6.json (ede357fc1e53) | trend/T7.json |
| E32 | unit-count-series | dump | dla 0.1.0 | artifact series/S6.json (ede357fc1e53) | trend/T7.json |
