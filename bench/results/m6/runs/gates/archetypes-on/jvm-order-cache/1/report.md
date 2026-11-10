# DLA report

Run: `20261110-164201-43e4`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 14 enumerated, 14 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 12

Residual: heap 568.9173745173746 bytes/n unattributed of 568.9173745173746 total (threshold 142.22934362934365) ABOVE THRESHOLD, see O1; heap 515.1196053196053 bytes/n unattributed of 515.1196053196053 total (threshold 128.77990132990132) ABOVE THRESHOLD, see O1; heap 514.3211068211068 bytes/n unattributed of 514.3211068211068 total (threshold 128.5802767052767) ABOVE THRESHOLD, see O1; non-heap 303.83526383526385 bytes/n unattributed of 303.83526383526385 total (threshold 686.9555545945946); non-heap 0 bytes/n unattributed of -6.465379665379666 total (threshold 18.75092386100382); non-heap 3.583783783783784 bytes/n unattributed of 3.583783783783784 total (threshold 27.178595783783784)

Coverage: jvm 8880 iterations, 14 of 14 entry points exercised

## Findings

### F5: **L-COLLECT**

- State: confirmed
- Label: L-COLLECT (evidence: E16 growth-series, E12 retention-path)
- Growth: `probe:shop.OrderCache.ORDERS_BY_ID` 6.75 count/n (steady, gate pass, E11)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `shop.OrderCache.ORDERS_BY_ID`
- Region: heap
- Shape: steady
- Site: `src/shop/OrderCache.java:8`
- Writers: `src/shop/OrderCache.java:17`
- Removers: `src/shop/OrderCache.java:26`
- Annotation: heuristic name match over the project's sources
- Annotation: only remover is invalidateAll, used by tests
- Verified: no
- Sibling hypotheses: F5.n1
- Supporting evidence: E6, E9, E11, E16, E19
- Explanation (observed): Static HashMap OrderCache.ORDERS_BY_ID (src/shop/OrderCache.java:8) gains an entry per new order id from store() at src/shop/OrderCache.java:17 via OrderService.place; entries are removed only when invalidateAll at src/shop/OrderCache.java:26 runs, which is test-only, so production never evicts.
- Explanation structure: adds: src/shop/OrderCache.java:17; removal: conditional at src/shop/OrderCache.java:26; evidence: E16

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `src/shop/OrderCache.java:8`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: Duplicate of confirmed F5 (same holder shop.OrderCache.ORDERS_BY_ID); its growth columns are consequences of F5, no separate leak
- Supporting evidence: E5

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.Catalog.PRODUCTS`
- Region: heap
- Site: `src/shop/Catalog.java:9`
- Reason: not_exercised
- Remedy: Immutable/fixed-size holder (constant list or per-instance record field); probes show no growth, so no leak evidence; no action needed
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.Main.CUSTOMERS`
- Region: heap
- Site: `src/shop/Main.java:5`
- Reason: not_exercised
- Remedy: Immutable/fixed-size holder (constant list or per-instance record field); probes show no growth, so no leak evidence; no action needed
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.Order.lines`
- Region: heap
- Site: `src/shop/Order.java:6`
- Reason: not_exercised
- Remedy: Immutable/fixed-size holder (constant list or per-instance record field); probes show no growth, so no leak evidence; no action needed
- Supporting evidence: E6

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `shop.OrderCache.ORDERS_BY_ID`
- Region: heap
- Shape: steady
- Site: `src/shop/OrderCache.java:8`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F5 (same holder shop.OrderCache.ORDERS_BY_ID); its growth columns are consequences of F5, no separate leak
- Supporting evidence: E11

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `shop.OrderCache.ORDERS_BY_ID`
- Region: heap
- Shape: steady
- Site: `src/shop/OrderCache.java:8`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F5 (same holder shop.OrderCache.ORDERS_BY_ID); its growth columns are consequences of F5, no separate leak
- Supporting evidence: E20

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `shop.OrderCache.ORDERS_BY_ID`
- Region: heap
- Shape: steady
- Site: `src/shop/OrderCache.java:8`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F5 (same holder shop.OrderCache.ORDERS_BY_ID); its growth columns are consequences of F5, no separate leak
- Supporting evidence: E22

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `shop.OrderCache.ORDERS_BY_ID`
- Region: heap
- Shape: steady
- Site: `src/shop/OrderCache.java:8`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F5 (same holder shop.OrderCache.ORDERS_BY_ID); its growth columns are consequences of F5, no separate leak
- Supporting evidence: E21

### F10: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `shop.OrderCache.ORDERS_BY_ID`
- Region: heap
- Shape: steady
- Site: `src/shop/OrderCache.java:8`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F5 (same holder shop.OrderCache.ORDERS_BY_ID); its growth columns are consequences of F5, no separate leak
- Supporting evidence: E23

### F11: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `shop.OrderCache.ORDERS_BY_ID`
- Region: heap
- Shape: steady
- Site: `src/shop/OrderCache.java:8`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F5 (same holder shop.OrderCache.ORDERS_BY_ID); its growth columns are consequences of F5, no separate leak
- Supporting evidence: E24

### F12: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `shop.OrderCache.ORDERS_BY_ID`
- Region: heap
- Shape: steady
- Site: `src/shop/OrderCache.java:8`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F5 (same holder shop.OrderCache.ORDERS_BY_ID); its growth columns are consequences of F5, no separate leak
- Supporting evidence: E28

### F13: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `shop.OrderCache.ORDERS_BY_ID`
- Region: heap
- Shape: steady
- Site: `src/shop/OrderCache.java:8`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F5 (same holder shop.OrderCache.ORDERS_BY_ID); its growth columns are consequences of F5, no separate leak
- Supporting evidence: E30

## Ruled out

- **F5.n1** (UNCLASSIFIED): discriminator E26
  - Ruled out by: plateau+purge (leak) E26
  - Hypothesis: N-WARM: cache fills during warm-up then plateaus
  - Refuted if: growth continues over the extra alloc-test cycles and after purge
  - Contradicting evidence: E26
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
- E18: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E18: limitation: the files carry no time, so they are ordered as given
- E19: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E19: limitation: the files carry no time, so they are ordered as given
- E19: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E20: limitation: series: per-point; x is the iteration count n of each point (1110, 4440, 6660, 9990); every point is a separate process and heap dump
- E20: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E20: limitation: holders absent from a dump's 200 largest have no figure at that point
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: series: per-point; x is the iteration count n of each point (1110, 4440, 6660, 9990); every point is a separate process and heap dump
- E21: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E21: limitation: holders absent from a dump's 200 largest have no figure at that point
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E22: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E23: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: partial coverage (5 of 32 examined)
- E24: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E24: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: partial coverage (1 of 32 examined)
- E25: limitation: series: per-point; x is the iteration count n of each point (1110, 4440, 6660, 9990); every point is a separate process and heap dump
- E25: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E25: limitation: holders absent from a dump's 200 largest have no figure at that point
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E26: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E26: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E26: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E26: limitation: points 1 to 9 come from S1; the extra points from S4
- E26: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E26: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E26: limitation: gate and plateau thresholds are provisional (not calibrated)
- E27: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E27: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E28: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E29: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E29: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E29: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E29: limitation: points 1 to 9 come from S1; the extra points from S4
- E29: limitation: gate thresholds are provisional (not calibrated)
- E30: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E30: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E30: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E30: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E30: limitation: points 1 to 9 come from S1; the extra points from S4
- E30: limitation: gate thresholds are provisional (not calibrated)
- E31: partial coverage (2 of 3 examined)
- E31: limitation: F5: growth measured as a count; bytes not attributed
- E31: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E31: limitation: threshold is provisional (not calibrated)
- E32: partial coverage (2 of 3 examined)
- E32: limitation: F5: growth evidence is from S1, not S4; not subtracted
- E32: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E32: limitation: threshold is provisional (not calibrated)
- E33: partial coverage (2 of 3 examined)
- E33: limitation: F5: growth evidence is from S1, not S5; not subtracted
- E33: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E33: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | — |
| E2 | run-config | static | dla 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | probe/probe.json |
| E4 | static-hit | static | spotbugs unknown | repo 658c6e857134364322cd5318f94779646f61fc7a | sweep/jvm-spotbugs.json |
| E5 | static-hit | static | dla 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | sweep/jvm-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | census/jvm.json |
| E7 | workload | static | dla 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (918b4a36e301) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (57a1ad78de71) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (57a1ad78de71) | trend/T1.json |
| E12 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-164201-43e4/harness/H1/out/p1-native/capture/87/heap-1.hprof (1a314ed95b97) | summaries/heap-1-1a314ed9.json |
| E13 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-164201-43e4/harness/H1/out/p4-native/capture/380/heap-1.hprof (08c22a4f4894) | summaries/heap-1-08c22a4f.json |
| E14 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-164201-43e4/harness/H1/out/p6-native/capture/576/heap-1.hprof (acf55c88172d) | summaries/heap-1-acf55c88.json |
| E15 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-164201-43e4/harness/H1/out/p9-native/capture/869/heap-1.hprof (52eab08c7b7a) | summaries/heap-1-52eab08c.json |
| E16 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S2.json (1398263dd3eb) | series/S2.json |
| E17 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S2.json (1398263dd3eb) | series/S2.json |
| E18 | histogram-diff | live | jcmd-class-histogram 0.1.0 | artifact histograms/H1.json (6f9b914d94fa) | histograms/H1.json |
| E19 | growth-series | live | jcmd-class-histogram 0.1.0 | artifact series/S3.json (2a9ea9f62b4b) | series/S3.json |
| E20 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (1398263dd3eb) | trend/T2.json |
| E21 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (1398263dd3eb) | trend/T2.json |
| E22 | growth-series | dump | dla 0.1.0 | artifact series/S3.json (2a9ea9f62b4b) | trend/T3.json |
| E23 | unit-count-series | dump | dla 0.1.0 | artifact series/S3.json (2a9ea9f62b4b) | trend/T3.json |
| E24 | growth-series | dump | dla 0.1.0 | artifact series/S3.json (2a9ea9f62b4b) | trend/T4.json |
| E25 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (1398263dd3eb) | trend/T5.json |
| E26 | discriminator | test | dla 0.1.0 | artifact series/S5.json (549c8775f16c) | alloc/A1.json |
| E27 | growth-series | test | dla 0.1.0 | artifact series/S4.json (645f0a34d4b4) | trend/T7.json |
| E28 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (645f0a34d4b4) | trend/T7.json |
| E29 | growth-series | test | dla 0.1.0 | artifact series/S5.json (549c8775f16c) | trend/T8.json |
| E30 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (549c8775f16c) | trend/T8.json |
| E31 | residual | test | dla 0.1.0 | artifact series/S1.json (57a1ad78de71) | gap/residual-S1-E31.json |
| E32 | residual | test | dla 0.1.0 | artifact series/S4.json (645f0a34d4b4) | gap/residual-S4-E32.json |
| E33 | residual | test | dla 0.1.0 | artifact series/S5.json (549c8775f16c) | gap/residual-S5-E33.json |
