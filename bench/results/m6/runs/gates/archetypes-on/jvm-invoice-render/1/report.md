# DLA report

Run: `20261110-164042-e3bd`

Status: INCOMPLETE (1 open)

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 11 enumerated, 11 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 9

Residual: heap 1277.1898326898329 bytes/n unattributed of 1277.1898326898329 total (threshold 319.29745817245816) ABOVE THRESHOLD, see O1; heap 1258.0326726726726 bytes/n unattributed of 1258.0326726726726 total (threshold 314.50816816816814) ABOVE THRESHOLD, see O1; heap 1259.313063063063 bytes/n unattributed of 1259.313063063063 total (threshold 314.82826576576576) ABOVE THRESHOLD, see O1; non-heap 0 bytes/n unattributed of -5.164135564135564 total (threshold 25.17307057915025); non-heap 0 bytes/n unattributed of -7.696954096954096 total (threshold 24.28214764478752); non-heap 0 bytes/n unattributed of -9.165165165165163 total (threshold 10.079186738738574)

Coverage: jvm 8880 iterations, 13 of 13 entry points exercised

## Open items

- **F6**: finding F6 is confirmed and has no stored explanation. Fix: `dla ledger explain F6 --tag inferred --adds FILE:LINE --removal never --text "<site, holder, who adds, why nothing removes>"`

## Findings

### F6: **L-TLS**

- State: confirmed
- Label: L-TLS (evidence: E16 growth-series, E12 retention-path)
- Growth: `holder:thread-local:billing.RenderContext.SEGMENTS` 3717115 bytes/n (steady, gate pass, E16)
- Evidence classes: context, growth, static
- Holder: thread-local
- Owner: `billing.RenderContext.SEGMENTS`
- Region: heap
- Shape: steady
- Site: `src/billing/RenderContext.java:8`
- Writers: `src/billing/RenderContext.java:13`
- Removers: `src/billing/RenderContext.java:99`
- Annotation: heuristic name match over the project's sources
- Annotation: CORRECTION: remover src/billing/RenderContext.java:99 was recorded by mistake by the investigator; that line does not exist. RenderContext has no remove/clear code; true removal is never.
- Verified: no
- Sibling hypotheses: F6.n1
- Supporting evidence: E6, E9, E16, E11

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `billing.Invoice.lines`
- Region: heap
- Site: `src/billing/Invoice.java:6`
- Reason: not_exercised
- Remedy: Census lead only; no growth column tied to it in the harness (immutable constants, per-call data, or fixed 4-thread pool). Add a probe if suspected.
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `billing.InvoiceFeed.SKUS`
- Region: heap
- Site: `src/billing/InvoiceFeed.java:9`
- Reason: not_exercised
- Remedy: Census lead only; no growth column tied to it in the harness (immutable constants, per-call data, or fixed 4-thread pool). Add a probe if suspected.
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `billing.InvoiceFeed.NAMES`
- Region: heap
- Site: `src/billing/InvoiceFeed.java:10`
- Reason: not_exercised
- Remedy: Census lead only; no growth column tied to it in the harness (immutable constants, per-call data, or fixed 4-thread pool). Add a probe if suspected.
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `billing.InvoiceService.pool`
- Region: non-heap
- Site: `src/billing/InvoiceService.java:17`
- Reason: not_exercised
- Remedy: Census lead only; no growth column tied to it in the harness (immutable constants, per-call data, or fixed 4-thread pool). Add a probe if suspected.
- Supporting evidence: E6

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `billing.InvoiceService$WorkerFactory.newThread`
- Region: non-heap
- Site: `src/billing/InvoiceService.java:58`
- Reason: not_exercised
- Remedy: Census lead only; no growth column tied to it in the harness (immutable constants, per-call data, or fixed 4-thread pool). Add a probe if suspected.
- Supporting evidence: E6

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `billing.TaxTable.RATES`
- Region: heap
- Site: `src/billing/TaxTable.java:7`
- Reason: not_exercised
- Remedy: Census lead only; no growth column tied to it in the harness (immutable constants, per-call data, or fixed 4-thread pool). Add a probe if suspected.
- Supporting evidence: E6

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Duplicate view of F6 growth (String/byte[] counts are the segment strings held by RenderContext.SEGMENTS); no separate holder; re-check after F6 fix
- Supporting evidence: E11

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Duplicate view of F6 growth (String/byte[] counts are the segment strings held by RenderContext.SEGMENTS); no separate holder; re-check after F6 fix
- Supporting evidence: E22

### F10: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Duplicate view of F6 growth (String/byte[] counts are the segment strings held by RenderContext.SEGMENTS); no separate holder; re-check after F6 fix
- Supporting evidence: E24

## Ruled out

- **F6.n1** (UNCLASSIFIED): discriminator E18
  - Ruled out by: plateau+purge (leak) E18
  - Hypothesis: N-WARM: warm-up fill of per-thread buffer that plateaus
  - Refuted if: growth continues over the extra cycles and dump-plateau favors leak
  - Contradicting evidence: E18
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
- E9: limitation: command-mode heap dumps come from separate processes: dla summarize heap-graph compares dumps of one process only, so use a driver (dla harness gen --driver) for a dump series
- E9: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E9: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E10: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E10: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E11: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E13: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E14: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E15: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E16: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E16: limitation: series: per-point; one process per point at n = 1110, 4440, 6660, 9990 (harness captures; checked: HPROF version, identifier size)
- E17: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E18: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: points 1 to 9 come from S1; the extra points from S3
- E18: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E18: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E18: limitation: gate and plateau thresholds are provisional (not calibrated)
- E19: limitation: series: per-point; x is the iteration count n of each point (1110, 4440, 6660, 9990); every point is a separate process and heap dump
- E19: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E19: limitation: holders absent from a dump's 200 largest have no figure at that point
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: series: per-point; x is the iteration count n of each point (1110, 4440, 6660, 9990); every point is a separate process and heap dump
- E20: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E20: limitation: holders absent from a dump's 200 largest have no figure at that point
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E23: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E23: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E23: limitation: points 1 to 9 come from S1; the extra points from S3
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E24: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E24: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E24: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E24: limitation: points 1 to 9 come from S1; the extra points from S3
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: partial coverage (2 of 3 examined)
- E25: limitation: F6: growth measured as a count; bytes not attributed
- E25: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E25: limitation: threshold is provisional (not calibrated)
- E26: partial coverage (2 of 3 examined)
- E26: limitation: F6: growth evidence is from S2, not S3; not subtracted
- E26: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E26: limitation: threshold is provisional (not calibrated)
- E27: partial coverage (2 of 3 examined)
- E27: limitation: F6: growth evidence is from S2, not S4; not subtracted
- E27: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E27: limitation: threshold is provisional (not calibrated)

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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (a2e4bba96918) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (ab4685075834) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (ab4685075834) | trend/T1.json |
| E12 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-164042-e3bd/harness/H1/out/p1-native/capture/138/heap-1.hprof (8bcba40f0c8b) | summaries/heap-1-8bcba40f.json |
| E13 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-164042-e3bd/harness/H1/out/p4-native/capture/455/heap-1.hprof (7b205007e8c6) | summaries/heap-1-7b205007.json |
| E14 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-164042-e3bd/harness/H1/out/p6-native/capture/667/heap-1.hprof (ee917450d9e3) | summaries/heap-1-ee917450.json |
| E15 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-164042-e3bd/harness/H1/out/p9-native/capture/985/heap-1.hprof (01968b7494bf) | summaries/heap-1-01968b74.json |
| E16 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S2.json (2d21a638f6c1) | series/S2.json |
| E17 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S2.json (2d21a638f6c1) | series/S2.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S4.json (56380e6e3b81) | alloc/A1.json |
| E19 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (2d21a638f6c1) | trend/T3.json |
| E20 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (2d21a638f6c1) | trend/T3.json |
| E21 | growth-series | test | dla 0.1.0 | artifact series/S3.json (1e39172ae7b6) | trend/T4.json |
| E22 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (1e39172ae7b6) | trend/T4.json |
| E23 | growth-series | test | dla 0.1.0 | artifact series/S4.json (56380e6e3b81) | trend/T5.json |
| E24 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (56380e6e3b81) | trend/T5.json |
| E25 | residual | test | dla 0.1.0 | artifact series/S1.json (ab4685075834) | gap/residual-S1-E25.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S3.json (1e39172ae7b6) | gap/residual-S3-E26.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S4.json (56380e6e3b81) | gap/residual-S4-E27.json |
