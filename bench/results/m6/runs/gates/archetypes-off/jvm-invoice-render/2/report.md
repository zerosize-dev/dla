# DLA report

Run: `20261110-191255-bafe`

Status: INCOMPLETE (9 open)

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 11 enumerated, 11 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 9

Residual: heap 1264.9045045045043 bytes/n unattributed of 1264.9045045045043 total (threshold 316.22612612612613) ABOVE THRESHOLD, see O3; heap 1257.5615615615616 bytes/n unattributed of 1257.5615615615616 total (threshold 314.3903903903904) ABOVE THRESHOLD, see O1; heap 1262.6066066066066 bytes/n unattributed of 1262.6066066066066 total (threshold 315.65165165165166) ABOVE THRESHOLD, see O1; non-heap 5.612912912912913 bytes/n unattributed of 5.612912912912913 total (threshold 54.15621054054045); non-heap 17.685885885885888 bytes/n unattributed of 17.685885885885888 total (threshold 7.0584735135136905) ABOVE THRESHOLD, see O2; non-heap 18.593720993720993 bytes/n unattributed of 18.593720993720993 total (threshold 9.156939112203036) ABOVE THRESHOLD, see O2

Coverage: jvm 8880 iterations, 13 of 13 entry points exercised

## Open items

- **O3**: open item O3: unexplained growth: 1265 bytes/n in heap (S1); add a hypothesis (dla ledger add --origin residual --evidence E29 ...) or record it unresolved. Fix: `dla ledger resolve O3 --as unresolved --reason time_budget --remedy "<text>"`
- **T1:os.rss**: column os.rss of T1 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:os.rss_anon**: column os.rss_anon of T1 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:os.vm_data**: column os.vm_data of T1 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:jvm.heap_committed**: column jvm.heap_committed of T1 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:jvm.heap_used**: column jvm.heap_used of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:jvm.heap_committed.pre_gc**: column jvm.heap_committed.pre_gc of T1 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:jvm.heap_used.pre_gc**: column jvm.heap_used.pre_gc of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **heap**: residual growth in heap (1264.905 bytes/n, threshold 316.226) has no resolved open item. Fix: `dla gap --residual then resolve the open item it raises`

## Findings

### F6: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (ThreadLocal<ArrayList<String>> RenderContext.SEGMENTS on the 4 long-lived pool workers: render() appends ~10 segments per invoice, never clears or removes the list (mark/textFrom only slice it), so each worker's list grows without bound and retains every segment string)
- Growth: `probe:billing.RenderContext.SEGMENTS` 12.53 count/n (steady, gate pass, E12)
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
- Supporting evidence: E7, E10, E12
- Explanation (observed): Site src/billing/RenderContext.java:8 (SEGMENTS). Adder src/billing/RenderContext.java:13: each render appends ~10 strings to a per-worker ThreadLocal list that is never cleared or removed; workers live for the service lifetime, so the lists retain every rendered segment. Class of repair: clear/trim the list (or remove the ThreadLocal) at the end of each render.
- Explanation structure: adds: src/billing/RenderContext.java:13; removal: never; evidence: E20

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `billing.Invoice.lines`
- Region: heap
- Site: `src/billing/Invoice.java:6`
- Reason: not_exercised
- Remedy: Census holder with no growth column tied in the harness run; not implicated, only F6 grew
- Supporting evidence: E7

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `billing.InvoiceFeed.SKUS`
- Region: heap
- Site: `src/billing/InvoiceFeed.java:9`
- Reason: not_exercised
- Remedy: Census holder with no growth column tied in the harness run; not implicated, only F6 grew
- Supporting evidence: E7

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `billing.InvoiceFeed.NAMES`
- Region: heap
- Site: `src/billing/InvoiceFeed.java:10`
- Reason: not_exercised
- Remedy: Census holder with no growth column tied in the harness run; not implicated, only F6 grew
- Supporting evidence: E7

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `billing.InvoiceService.pool`
- Region: non-heap
- Site: `src/billing/InvoiceService.java:17`
- Reason: not_exercised
- Remedy: Census holder with no growth column tied in the harness run; not implicated, only F6 grew
- Supporting evidence: E7

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `billing.InvoiceService$WorkerFactory.newThread`
- Region: non-heap
- Site: `src/billing/InvoiceService.java:58`
- Reason: not_exercised
- Remedy: Census holder with no growth column tied in the harness run; not implicated, only F6 grew
- Supporting evidence: E7

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `billing.TaxTable.RATES`
- Region: heap
- Site: `src/billing/TaxTable.java:7`
- Reason: not_exercised
- Remedy: Immutable Map.of constant; no growth column tied
- Supporting evidence: E7

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Growth-series row for the same RenderContext.SEGMENTS growth (byte[]/String counts, mirrors of F6); not tested separately, covered by confirmed F6
- Supporting evidence: E12

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Growth-series row for the same RenderContext.SEGMENTS growth (byte[]/String counts, mirrors of F6); not tested separately, covered by confirmed F6
- Supporting evidence: E15

### F10: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Growth-series row for the same RenderContext.SEGMENTS growth (byte[]/String counts, mirrors of F6); not tested separately, covered by confirmed F6
- Supporting evidence: E23

## Ruled out

- **F6.n1** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (leak) E13
  - Hypothesis: N-WARM: SEGMENTS growth is a warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E13
  - Originally supported by: E12

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

- E5: partial coverage
- E5: limitation: no auxiliary classpath: library calls are not modelled
- E7: partial coverage
- E7: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E7: limitation: writers and removers are textual name matches
- E8: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E8: limitation: workloads were discovered by reading files; none was run
- E9: limitation: harness written; nothing was run (dla harness run measures it)
- E9: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E9: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E10: partial coverage (9 of 9 examined)
- E10: limitation: command-mode heap dumps come from separate processes: dla summarize heap-graph compares dumps of one process only, so use a driver (dla harness gen --driver) for a dump series
- E10: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E10: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E11: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: gate thresholds are provisional (not calibrated)
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
- E14: limitation: points 1 to 9 come from S1; the extra points from S2
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: points 1 to 9 come from S1; the extra points from S2
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E17: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E18: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E19: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E20: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E20: limitation: series: per-point; one process per point at n = 1110, 4440, 6660, 9990 (harness captures; checked: HPROF version, identifier size)
- E21: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E23: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: series: per-point; x is the iteration count n of each point (1110, 4440, 6660, 9990); every point is a separate process and heap dump
- E24: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E24: limitation: holders absent from a dump's 200 largest have no figure at that point
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: series: per-point; x is the iteration count n of each point (1110, 4440, 6660, 9990); every point is a separate process and heap dump
- E25: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E25: limitation: holders absent from a dump's 200 largest have no figure at that point
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: partial coverage (2 of 3 examined)
- E26: limitation: F6: growth measured as a count; bytes not attributed
- E26: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E26: limitation: threshold is provisional (not calibrated)
- E27: partial coverage (2 of 3 examined)
- E27: limitation: F6: growth evidence is from S1, not S2; not subtracted
- E27: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E27: limitation: threshold is provisional (not calibrated)
- E28: partial coverage (2 of 3 examined)
- E28: limitation: F6: growth evidence is from S1, not S3; not subtracted
- E28: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E28: limitation: threshold is provisional (not calibrated)
- E29: partial coverage (2 of 3 examined)
- E29: limitation: F6: growth measured as a count; bytes not attributed
- E29: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E29: limitation: threshold is provisional (not calibrated)

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
| E4 | probe | static | dla-probe 0.1.0 | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | probe/probe.json |
| E5 | static-hit | static | spotbugs unknown | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | sweep/jvm-spotbugs.json |
| E6 | static-hit | static | dla 0.1.0 | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | sweep/jvm-dla.json |
| E7 | census-holder | static | dla-census 0.1.0 | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | census/jvm.json |
| E8 | workload | static | dla 0.1.0 | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | workload/workloads.json |
| E9 | workload | static | dla 0.1.0 | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | harness/H1/harness.json |
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (8a957977050b) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (7a02323addc6) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (7a02323addc6) | trend/T1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S3.json (5808b284a3f4) | alloc/A1.json |
| E14 | growth-series | test | dla 0.1.0 | artifact series/S3.json (5808b284a3f4) | trend/T3.json |
| E15 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (5808b284a3f4) | trend/T3.json |
| E16 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-191255-bafe/harness/H1/out/p1-native/capture/86/heap-1.hprof (69332182d83e) | summaries/heap-1-69332182.json |
| E17 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-191255-bafe/harness/H1/out/p4-native/capture/403/heap-1.hprof (4a074bb7dccc) | summaries/heap-1-4a074bb7.json |
| E18 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-191255-bafe/harness/H1/out/p6-native/capture/615/heap-1.hprof (550acca7cd62) | summaries/heap-1-550acca7.json |
| E19 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-191255-bafe/harness/H1/out/p9-native/capture/933/heap-1.hprof (3f056e202c5b) | summaries/heap-1-3f056e20.json |
| E20 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S4.json (3d95b99f5b65) | series/S4.json |
| E21 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S4.json (3d95b99f5b65) | series/S4.json |
| E22 | growth-series | test | dla 0.1.0 | artifact series/S2.json (c2e04411fed4) | trend/T4.json |
| E23 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (c2e04411fed4) | trend/T4.json |
| E24 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (3d95b99f5b65) | trend/T5.json |
| E25 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (3d95b99f5b65) | trend/T5.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S1.json (7a02323addc6) | gap/residual-S1-E26.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S2.json (c2e04411fed4) | gap/residual-S2-E27.json |
| E28 | residual | test | dla 0.1.0 | artifact series/S3.json (5808b284a3f4) | gap/residual-S3-E28.json |
| E29 | residual | test | dla 0.1.0 | artifact series/S1.json (7a02323addc6) | gap/residual-S1-E29.json |
