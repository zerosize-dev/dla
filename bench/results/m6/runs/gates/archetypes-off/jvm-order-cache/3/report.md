# DLA report

Run: `20261110-195258-9e68`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 10 enumerated, 10 dispositioned; confirmed 0, strong 1, refuted 1, suspected 3, unresolved 5

Residual: heap 596.6976404976406 bytes/n unattributed of 596.6976404976406 total (threshold 149.17441012441014) ABOVE THRESHOLD, see O1; heap 514.494036894037 bytes/n unattributed of 514.494036894037 total (threshold 128.62350922350925) ABOVE THRESHOLD, see O1; heap 517.8497297297297 bytes/n unattributed of 517.8497297297297 total (threshold 129.46243243243242) ABOVE THRESHOLD, see O1; non-heap 522.4649506649507 bytes/n unattributed of 522.4649506649507 total (threshold 415.80336123552104) ABOVE THRESHOLD, see O2; non-heap 0 bytes/n unattributed of -29.78978978978979 total (threshold 34.93593297297315); non-heap 8.196996996996997 bytes/n unattributed of 8.196996996996997 total (threshold 22.12484425225221)

Coverage: jvm 8880 iterations, 14 of 14 entry points exercised

## Findings

### F5: **UNCLASSIFIED**

- State: strong
- Label: UNCLASSIFIED (Static HashMap OrderCache.ORDERS_BY_ID keyed by order id; every new order is stored and nothing evicts it (invalidateAll only used by tests), so the live set grows with distinct orders)
- Growth: `probe:shop.OrderCache.ORDERS_BY_ID` 6.75 count/n (steady, gate pass, E11)
- Evidence classes: context, discriminator, growth, retention, static
- Strong: second class E20 retention-path that names the holder, independent of the growth record E11 (dla)
- Holder: static/global
- Owner: `shop.OrderCache.ORDERS_BY_ID`
- Region: heap
- Shape: steady
- Site: `src/shop/OrderCache.java:8`
- Writers: `src/shop/OrderCache.java:17`
- Removers: `src/shop/OrderCache.java:26`
- Annotation: heuristic name match over the project's sources
- Annotation: put on every new order id; only remover is invalidateAll, used by tests only
- Verified: no
- Sibling hypotheses: F5.n1
- Supporting evidence: E6, E9, E11, E12, E20, E21
- Explanation (observed): Static map OrderCache.ORDERS_BY_ID (OrderCache.java:8) gets every new order via store() at src/shop/OrderCache.java:17 from OrderService.place; the only remover invalidateAll at src/shop/OrderCache.java:26 is never reached in production (tests only).
- Explanation structure: adds: src/shop/OrderCache.java:17; removal: unreachable at src/shop/OrderCache.java:26; evidence: E21

### F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.Catalog.PRODUCTS`
- Region: heap
- Site: `src/shop/Catalog.java:9`
- Reason: not_exercised
- Remedy: immutable constant (List.of / String[]), no writers; no growth tied
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.Main.CUSTOMERS`
- Region: heap
- Site: `src/shop/Main.java:5`
- Reason: not_exercised
- Remedy: immutable constant (List.of / String[]), no writers; no growth tied
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.Order.lines`
- Region: heap
- Site: `src/shop/Order.java:6`
- Reason: not_exercised
- Remedy: Order.lines is retained through F5 (OrderCache); no independent holder
- Supporting evidence: E6

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `src/shop/OrderCache.java:8`
- Reason: not_exercised
- Remedy: duplicate of F5 (same site OrderCache.java:8); covered by F5
- Supporting evidence: E5

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: object-count growth of Order/Line/String/byte[]/HashMap$Node is the payload retained by F5 (type-histogram-diff E29 tied to F5); no separate holder
- Supporting evidence: E14

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: object-count growth of Order/Line/String/byte[]/HashMap$Node is the payload retained by F5 (type-histogram-diff E29 tied to F5); no separate holder
- Supporting evidence: E16

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: type growth (Order, Line, String, byte[]) is the payload retained by F5 (heap-graph E21 names OrderCache.ORDERS_BY_ID); no separate holder
- Supporting evidence: E30

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: type growth (Order, Line, String, byte[]) is the payload retained by F5 (heap-graph E21 names OrderCache.ORDERS_BY_ID); no separate holder
- Supporting evidence: E31

## Ruled out

- **F5.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM: cache fills then plateaus
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
- E25: limitation: F5: growth measured as a count; bytes not attributed
- E25: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E25: limitation: threshold is provisional (not calibrated)
- E26: partial coverage (2 of 3 examined)
- E26: limitation: F5: growth evidence is from S1, not S2; not subtracted
- E26: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E26: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E26: limitation: threshold is provisional (not calibrated)
- E27: partial coverage (2 of 3 examined)
- E27: limitation: F5: growth evidence is from S1, not S3; not subtracted
- E27: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E27: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E27: limitation: threshold is provisional (not calibrated)
- E28: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E28: limitation: the files carry no time, so they are ordered as given
- E29: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E29: limitation: the files carry no time, so they are ordered as given
- E29: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E30: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E30: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E30: limitation: gate thresholds are provisional (not calibrated)
- E31: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E31: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E31: limitation: gate thresholds are provisional (not calibrated)

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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (01a64ccf9a4a) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (f6d6b6111f8a) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (f6d6b6111f8a) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (e1e8f6e3763b) | alloc/A1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S2.json (5611976fdb7d) | trend/T3.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (5611976fdb7d) | trend/T3.json |
| E15 | growth-series | test | dla 0.1.0 | artifact series/S3.json (e1e8f6e3763b) | trend/T4.json |
| E16 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (e1e8f6e3763b) | trend/T4.json |
| E17 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-195258-9e68/harness/H1/out/p1-native/capture/82/heap-1.hprof (851f5b5c6967) | summaries/heap-1-851f5b5c.json |
| E18 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-195258-9e68/harness/H1/out/p4-native/capture/373/heap-1.hprof (b500c5fd7633) | summaries/heap-1-b500c5fd.json |
| E19 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-195258-9e68/harness/H1/out/p6-native/capture/567/heap-1.hprof (8a985017c841) | summaries/heap-1-8a985017.json |
| E20 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-195258-9e68/harness/H1/out/p9-native/capture/858/heap-1.hprof (df51635fe8da) | summaries/heap-1-df51635f.json |
| E21 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S4.json (88b8362c381b) | series/S4.json |
| E22 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S4.json (88b8362c381b) | series/S4.json |
| E23 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (88b8362c381b) | trend/T5.json |
| E24 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (88b8362c381b) | trend/T5.json |
| E25 | residual | test | dla 0.1.0 | artifact series/S1.json (f6d6b6111f8a) | gap/residual-S1-E25.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S2.json (5611976fdb7d) | gap/residual-S2-E26.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S3.json (e1e8f6e3763b) | gap/residual-S3-E27.json |
| E28 | histogram-diff | live | jcmd-class-histogram 0.1.0 | artifact histograms/H1.json (4bd5aeee68fd) | histograms/H1.json |
| E29 | growth-series | live | jcmd-class-histogram 0.1.0 | artifact series/S5.json (51152e534d65) | series/S5.json |
| E30 | growth-series | dump | dla 0.1.0 | artifact series/S5.json (51152e534d65) | trend/T6.json |
| E31 | unit-count-series | dump | dla 0.1.0 | artifact series/S5.json (51152e534d65) | trend/T6.json |
