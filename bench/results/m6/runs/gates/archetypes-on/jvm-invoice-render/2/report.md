# DLA report

Run: `20261110-171738-38bc`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 12 enumerated, 12 dispositioned; confirmed 1, strong 0, refuted 2, suspected 6, unresolved 3

Residual: heap 1275.521921921922 bytes/n unattributed of 1275.521921921922 total (threshold 318.8804804804805) ABOVE THRESHOLD, see O1; heap 1246.1383783783783 bytes/n unattributed of 1246.1383783783783 total (threshold 311.53459459459464) ABOVE THRESHOLD, see O1; heap 1259.1710210210208 bytes/n unattributed of 1259.1710210210208 total (threshold 314.7927552552552) ABOVE THRESHOLD, see O1; non-heap 10.18018018018018 bytes/n unattributed of 10.18018018018018 total (threshold 33.97958918918919); non-heap 28.55135135135135 bytes/n unattributed of 28.55135135135135 total (threshold 27.022245405405403) ABOVE THRESHOLD, see O2; non-heap 1.4700128700128698 bytes/n unattributed of 1.4700128700128698 total (threshold 17.65194627027009)

Coverage: jvm 8880 iterations, 13 of 13 entry points exercised

## Findings

### F6: **L-TLS**

- State: confirmed
- Label: L-TLS (evidence: E21 growth-series, E17 retention-path)
- Growth: `probe:billing.RenderContext.SEGMENTS` 12.53 count/n (steady, gate pass, E11)
- Evidence classes: context, discriminator, growth, static
- Holder: thread-local
- Owner: `billing.RenderContext.SEGMENTS`
- Region: heap
- Shape: steady
- Site: `src/billing/RenderContext.java:8`
- Writers: `src/billing/RenderContext.java:13`
- Removers: `src/billing/InvoiceRenderer.java:9`
- Annotation: heuristic name match over the project's sources
- Verified: no
- Sibling hypotheses: F6.n1, F6.n2
- Supporting evidence: E6, E9, E11, E12
- Explanation (observed): RenderContext.java:8 SEGMENTS thread-local on 4 pooled render threads gets each segment via append() at RenderContext.java:13; InvoiceRenderer.java:9 only mark()s an offset and reads a sublist, so entries go only when remove() or clear() is called, which never happens; lists grow per invoice.
- Explanation structure: adds: src/billing/RenderContext.java:13; removal: conditional at src/billing/InvoiceRenderer.java:9; evidence: E21

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `billing.Invoice.lines`
- Region: heap
- Site: `src/billing/Invoice.java:6`
- Reason: not_exercised
- Remedy: Census lead only: no tied column grew in the harness (fixed data / constant-size holder); not the cause of the growth, which F6 explains
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `billing.InvoiceFeed.SKUS`
- Region: heap
- Site: `src/billing/InvoiceFeed.java:9`
- Reason: not_exercised
- Remedy: Census lead only: no tied column grew in the harness (fixed data / constant-size holder); not the cause of the growth, which F6 explains
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `billing.InvoiceFeed.NAMES`
- Region: heap
- Site: `src/billing/InvoiceFeed.java:10`
- Reason: not_exercised
- Remedy: Census lead only: no tied column grew in the harness (fixed data / constant-size holder); not the cause of the growth, which F6 explains
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `billing.InvoiceService.pool`
- Region: non-heap
- Site: `src/billing/InvoiceService.java:17`
- Reason: not_exercised
- Remedy: Census lead only: no tied column grew in the harness (fixed data / constant-size holder); not the cause of the growth, which F6 explains
- Supporting evidence: E6

### F5: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `billing.InvoiceService$WorkerFactory.newThread`
- Region: non-heap
- Site: `src/billing/InvoiceService.java:58`
- Reason: not_exercised
- Remedy: Census lead only: no tied column grew in the harness (fixed data / constant-size holder); not the cause of the growth, which F6 explains
- Supporting evidence: E6

### F7: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `billing.TaxTable.RATES`
- Region: heap
- Site: `src/billing/TaxTable.java:7`
- Reason: not_exercised
- Remedy: Census lead only: no tied column grew in the harness (fixed data / constant-size holder); not the cause of the growth, which F6 explains
- Supporting evidence: E6

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Same growth as F6 (String/byte[] segments held by RenderContext.SEGMENTS); duplicate growth-series row, no separate holder
- Supporting evidence: E11

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Same growth as F6; duplicate growth-series row
- Supporting evidence: E16

### F10: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Same growth as F6; duplicate growth-series row
- Supporting evidence: E14

## Ruled out

- **F6.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM: growth is a warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E12
  - Originally supported by: E11
- **F6.n2** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-HWM: thread-local is cut back after each unit and holds only the largest unit
  - Refuted if: growth continues across extra cycles instead of plateauing
  - Contradicting evidence: E12
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
- E17: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E18: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E19: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E20: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E21: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E21: limitation: series: per-point; one process per point at n = 1110, 4440, 6660, 9990 (harness captures; checked: HPROF version, identifier size)
- E22: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E23: limitation: series: per-point; x is the iteration count n of each point (1110, 4440, 6660, 9990); every point is a separate process and heap dump
- E23: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E23: limitation: holders absent from a dump's 200 largest have no figure at that point
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: series: per-point; x is the iteration count n of each point (1110, 4440, 6660, 9990); every point is a separate process and heap dump
- E24: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E24: limitation: holders absent from a dump's 200 largest have no figure at that point
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: partial coverage (2 of 3 examined)
- E25: limitation: F6: growth measured as a count; bytes not attributed
- E25: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E25: limitation: threshold is provisional (not calibrated)
- E26: partial coverage (2 of 3 examined)
- E26: limitation: F6: growth evidence is from S1, not S2; not subtracted
- E26: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E26: limitation: threshold is provisional (not calibrated)
- E27: partial coverage (2 of 3 examined)
- E27: limitation: F6: growth evidence is from S1, not S3; not subtracted
- E27: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E27: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (391543302dd1) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (42e8d99f172d) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (42e8d99f172d) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (316af340a26c) | alloc/A1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S2.json (fd68a3708cc1) | trend/T3.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (fd68a3708cc1) | trend/T3.json |
| E15 | growth-series | test | dla 0.1.0 | artifact series/S3.json (316af340a26c) | trend/T4.json |
| E16 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (316af340a26c) | trend/T4.json |
| E17 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-171738-38bc/harness/H1/out/p1-native/capture/86/heap-1.hprof (332b8f5b2ca8) | summaries/heap-1-332b8f5b.json |
| E18 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-171738-38bc/harness/H1/out/p4-native/capture/403/heap-1.hprof (32beb57df2b6) | summaries/heap-1-32beb57d.json |
| E19 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-171738-38bc/harness/H1/out/p6-native/capture/615/heap-1.hprof (c335caae2ab6) | summaries/heap-1-c335caae.json |
| E20 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-171738-38bc/harness/H1/out/p9-native/capture/933/heap-1.hprof (d3b0c03396fb) | summaries/heap-1-d3b0c033.json |
| E21 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S4.json (8241c2524a8c) | series/S4.json |
| E22 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S4.json (8241c2524a8c) | series/S4.json |
| E23 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (8241c2524a8c) | trend/T5.json |
| E24 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (8241c2524a8c) | trend/T5.json |
| E25 | residual | test | dla 0.1.0 | artifact series/S1.json (42e8d99f172d) | gap/residual-S1-E25.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S2.json (fd68a3708cc1) | gap/residual-S2-E26.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S3.json (316af340a26c) | gap/residual-S3-E27.json |
