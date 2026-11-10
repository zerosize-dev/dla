# DLA report

Run: `20261110-204415-62df`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 12 enumerated, 12 dispositioned; confirmed 1, strong 0, refuted 1, suspected 6, unresolved 4

Residual: heap 1280.4709395109394 bytes/n unattributed of 1280.4709395109394 total (threshold 320.11773487773485) ABOVE THRESHOLD, see O2; heap 1276.088888888889 bytes/n unattributed of 1276.088888888889 total (threshold 319.02222222222224) ABOVE THRESHOLD, see O2; heap 1266.7222255222257 bytes/n unattributed of 1266.7222255222257 total (threshold 316.68055638055637) ABOVE THRESHOLD, see O2; non-heap 36.76492492492493 bytes/n unattributed of 36.76492492492493 total (threshold 40.22062918918883); non-heap 0 bytes/n unattributed of -6.299528099528099 total (threshold 39.51089474903485); non-heap 4.198318318318318 bytes/n unattributed of 4.198318318318318 total (threshold 20.475854681081277)

Coverage: jvm 8880 iterations, 13 of 13 entry points exercised

## Findings

### F6: **L-TLS**

- State: confirmed
- Label: L-TLS (evidence: E17 growth-series, E13 retention-path)
- Growth: `probe:billing.RenderContext.SEGMENTS` 12.53 count/n (steady, gate pass, E11)
- Evidence classes: context, discriminator, growth, static
- Holder: thread-local
- Owner: `billing.RenderContext.SEGMENTS`
- Region: heap
- Shape: steady
- Site: `src/billing/RenderContext.java:8`
- Writers: `src/billing/RenderContext.java:13`
- Annotation: heuristic name match over the project's sources
- Annotation: RenderContext.mark/textFrom never trims or removes the per-thread list; pooled threads keep every segment
- Verified: DISAGREES (site: finding `src/billing/RenderContext.java:8` vs verifier `src/billing/RenderContext.java:13`; bundle ad855c651c09); finding mechanism: RenderContext.SEGMENTS (RenderContext.java:8) is a thread-local list on the 4 long-lived pool threads; append at RenderContext.java:13 adds every segment of every invoice and the list is never truncated, nothing removes entries or calls remove(), so it grows per invoice.; verifier mechanism: Static ThreadLocal<List<String>> per pooled render worker; append() adds segments for every invoice and textFrom(mark) only reads a sublist, never clears or truncates, so each worker's list grows unbounded across the fixed thread pool's lifetime.
- Sibling hypotheses: F6.n1
- Supporting evidence: E6, E9, E11, E12
- Explanation (observed): RenderContext.SEGMENTS (RenderContext.java:8) is a thread-local list on the 4 long-lived pool threads; append at RenderContext.java:13 adds every segment of every invoice and the list is never truncated, nothing removes entries or calls remove(), so it grows per invoice.
- Explanation structure: adds: src/billing/RenderContext.java:13; removal: never; evidence: E17

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `billing.Invoice.lines`
- Region: heap
- Site: `src/billing/Invoice.java:6`
- Reason: not_exercised
- Remedy: census lead only; no growth column tied to it (constant arrays/immutable map, fixed 4-thread pool, per-invoice record); F6 explains the growth
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `billing.InvoiceFeed.SKUS`
- Region: heap
- Site: `src/billing/InvoiceFeed.java:9`
- Reason: not_exercised
- Remedy: census lead only; no growth column tied to it (constant arrays/immutable map, fixed 4-thread pool, per-invoice record); F6 explains the growth
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `billing.InvoiceFeed.NAMES`
- Region: heap
- Site: `src/billing/InvoiceFeed.java:10`
- Reason: not_exercised
- Remedy: census lead only; no growth column tied to it (constant arrays/immutable map, fixed 4-thread pool, per-invoice record); F6 explains the growth
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `billing.InvoiceService.pool`
- Region: non-heap
- Site: `src/billing/InvoiceService.java:17`
- Reason: not_exercised
- Remedy: census lead only; no growth column tied to it (constant arrays/immutable map, fixed 4-thread pool, per-invoice record); F6 explains the growth
- Supporting evidence: E6

### F5: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `billing.InvoiceService$WorkerFactory.newThread`
- Region: non-heap
- Site: `src/billing/InvoiceService.java:58`
- Reason: not_exercised
- Remedy: census lead only; no growth column tied to it (constant arrays/immutable map, fixed 4-thread pool, per-invoice record); F6 explains the growth
- Supporting evidence: E6

### F7: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `billing.TaxTable.RATES`
- Region: heap
- Site: `src/billing/TaxTable.java:7`
- Reason: not_exercised
- Remedy: census lead only; no growth column tied to it (constant arrays/immutable map, fixed 4-thread pool, per-invoice record); F6 explains the growth
- Supporting evidence: E6

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: duplicate symptom column of F6 (String/byte[] counts); no separate holder
- Supporting evidence: E28

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: thread-local
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: symptom columns (String/byte[] counts) of F6; covered by F6's confirmation, no separate holder
- Supporting evidence: E22

### F10: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: thread-local
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: symptom columns (String/byte[] counts) of F6; covered by F6's confirmation, no separate holder
- Supporting evidence: E25

### F11: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: thread-local
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: symptom columns (String/byte[] counts) of F6; covered by F6's confirmation, no separate holder
- Supporting evidence: E26

## Ruled out

- **F6.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM: warm-up fill of the per-thread buffer
  - Refuted if: growth continues over the extra cycles and survives a purge
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
- E13: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E14: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E15: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E16: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E17: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E17: limitation: series: per-point; one process per point at n = 1110, 4440, 6660, 9990 (harness captures; checked: HPROF version, identifier size)
- E18: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E19: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E19: limitation: the files carry no time, so they are ordered as given
- E20: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E20: limitation: the files carry no time, so they are ordered as given
- E20: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: series: per-point; x is the iteration count n of each point (1110, 4440, 6660, 9990); every point is a separate process and heap dump
- E23: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E23: limitation: holders absent from a dump's 200 largest have no figure at that point
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: series: per-point; x is the iteration count n of each point (1110, 4440, 6660, 9990); every point is a separate process and heap dump
- E24: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E24: limitation: holders absent from a dump's 200 largest have no figure at that point
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E25: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E26: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E27: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E27: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E27: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E27: limitation: points 1 to 9 come from S1; the extra points from S2
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E28: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E28: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E28: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E28: limitation: points 1 to 9 come from S1; the extra points from S2
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: partial coverage (2 of 3 examined)
- E29: limitation: F6: growth measured as a count; bytes not attributed
- E29: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E29: limitation: threshold is provisional (not calibrated)
- E30: partial coverage (2 of 3 examined)
- E30: limitation: F6: growth evidence is from S1, not S2; not subtracted
- E30: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E30: limitation: threshold is provisional (not calibrated)
- E31: partial coverage (2 of 3 examined)
- E31: limitation: F6: growth evidence is from S1, not S3; not subtracted
- E31: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E31: limitation: threshold is provisional (not calibrated)
- E32: partial coverage (2 of 3 examined)
- E32: limitation: F6: growth measured as a count; bytes not attributed
- E32: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E32: limitation: threshold is provisional (not calibrated)
- E33: partial coverage (2 of 3 examined)
- E33: limitation: F6: growth evidence is from S1, not S2; not subtracted
- E33: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E33: limitation: threshold is provisional (not calibrated)
- E34: partial coverage (2 of 3 examined)
- E34: limitation: F6: growth evidence is from S1, not S3; not subtracted
- E34: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E34: limitation: threshold is provisional (not calibrated)

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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (e09dcea695b3) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (67cdcf468d28) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (67cdcf468d28) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (4d3e5725859e) | alloc/A1.json |
| E13 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-204415-62df/harness/H1/out/p1-native/capture/86/heap-1.hprof (6b4b70fda08b) | summaries/heap-1-6b4b70fd.json |
| E14 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-204415-62df/harness/H1/out/p4-native/capture/403/heap-1.hprof (29194cf59497) | summaries/heap-1-29194cf5.json |
| E15 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-204415-62df/harness/H1/out/p6-native/capture/615/heap-1.hprof (884bf492a95c) | summaries/heap-1-884bf492.json |
| E16 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-204415-62df/harness/H1/out/p9-native/capture/933/heap-1.hprof (242341e1578b) | summaries/heap-1-242341e1.json |
| E17 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S4.json (da4319b36379) | series/S4.json |
| E18 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S4.json (da4319b36379) | series/S4.json |
| E19 | histogram-diff | live | jcmd-class-histogram 0.1.0 | artifact histograms/H1.json (16c100990b92) | histograms/H1.json |
| E20 | growth-series | live | jcmd-class-histogram 0.1.0 | artifact series/S5.json (de0e7676f8f6) | series/S5.json |
| E21 | growth-series | test | dla 0.1.0 | artifact series/S2.json (89041a845749) | trend/T3.json |
| E22 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (89041a845749) | trend/T3.json |
| E23 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (da4319b36379) | trend/T4.json |
| E24 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (da4319b36379) | trend/T4.json |
| E25 | growth-series | dump | dla 0.1.0 | artifact series/S5.json (de0e7676f8f6) | trend/T5.json |
| E26 | unit-count-series | dump | dla 0.1.0 | artifact series/S5.json (de0e7676f8f6) | trend/T5.json |
| E27 | growth-series | test | dla 0.1.0 | artifact series/S3.json (4d3e5725859e) | trend/T6.json |
| E28 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (4d3e5725859e) | trend/T6.json |
| E29 | residual | test | dla 0.1.0 | artifact series/S1.json (67cdcf468d28) | gap/residual-S1-E29.json |
| E30 | residual | test | dla 0.1.0 | artifact series/S2.json (89041a845749) | gap/residual-S2-E30.json |
| E31 | residual | test | dla 0.1.0 | artifact series/S3.json (4d3e5725859e) | gap/residual-S3-E31.json |
| E32 | residual | test | dla 0.1.0 | artifact series/S1.json (67cdcf468d28) | gap/residual-S1-E32.json |
| E33 | residual | test | dla 0.1.0 | artifact series/S2.json (89041a845749) | gap/residual-S2-E33.json |
| E34 | residual | test | dla 0.1.0 | artifact series/S3.json (4d3e5725859e) | gap/residual-S3-E34.json |
