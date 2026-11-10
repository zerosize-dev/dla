# DLA report

Run: `20261110-184019-bbe6`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 7 enumerated, 7 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 5

Residual: heap 508.3675675675675 bytes/n unattributed of 508.3675675675675 total (threshold 127.09189189189188) ABOVE THRESHOLD, see O3; heap 565.1259259259259 bytes/n unattributed of 565.1259259259259 total (threshold 141.28148148148148) ABOVE THRESHOLD, see O3; heap 515.0844272844272 bytes/n unattributed of 515.0844272844272 total (threshold 128.7711068211068) ABOVE THRESHOLD, see O3; heap 555.0342342342342 bytes/n unattributed of 555.0342342342342 total (threshold 138.75855855855855) ABOVE THRESHOLD, see O3; non-heap 4.7329729729729735 bytes/n unattributed of 4.7329729729729735 total (threshold 3.724703351351377) ABOVE THRESHOLD, see O4; non-heap 343.11855855855856 bytes/n unattributed of 343.11855855855856 total (threshold 272.299780127764) ABOVE THRESHOLD, see O4; non-heap 0 bytes/n unattributed of -4.77981981981982 total (threshold 18.21067848648665); non-heap 263.25017325017325 bytes/n unattributed of 263.25017325017325 total (threshold 196.97402348440735) ABOVE THRESHOLD, see O4

Coverage: jvm 8880 iterations, 14 of 14 entry points exercised

## Findings

### F5: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Static HashMap shop.OrderCache.ORDERS_BY_ID (OrderCache.java:8) is written by store() on every new order id (OrderService.place) and never evicted; invalidateAll() is test-only. Live set grows with the number of distinct order ids: unbounded cache.)
- Growth: `type:shop.Order` 124320 bytes/dump (steady, gate pass, E18)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `shop.OrderCache.ORDERS_BY_ID`
- Region: heap
- Shape: steady
- Site: `src/shop/OrderCache.java:8`
- Writers: `src/shop/OrderCache.java:17`
- Removers: `src/shop/OrderCache.java:26`
- Annotation: heuristic name match over the project's sources
- Verified: no
- Sibling hypotheses: F5.n1
- Supporting evidence: E5, E8, E18
- Explanation (observed): OrderCache.java:8 static HashMap ORDERS_BY_ID is the holder; store() at OrderCache.java:17 adds every new order from OrderService.place; the only remover, invalidateAll at OrderCache.java:26, is never reached in production (tests only).
- Explanation structure: adds: src/shop/OrderCache.java:17; removal: unreachable at src/shop/OrderCache.java:26; evidence: E15

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `src/shop/OrderCache.java:8`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F5 (same holder shop.OrderCache.ORDERS_BY_ID, same site OrderCache.java:8); not tested separately
- Supporting evidence: E4

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.Catalog.PRODUCTS`
- Region: heap
- Site: `src/shop/Catalog.java:9`
- Reason: not_exercised
- Remedy: Catalog.PRODUCTS is a fixed-size static table; no growth column tied to it in any series
- Supporting evidence: E5

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.Main.CUSTOMERS`
- Region: heap
- Site: `src/shop/Main.java:5`
- Reason: not_exercised
- Remedy: Main.CUSTOMERS is a fixed 6-element constant array; no growth tied
- Supporting evidence: E5

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.Order.lines`
- Region: heap
- Site: `src/shop/Order.java:6`
- Reason: not_exercised
- Remedy: Order.lines is retained only through F5's cache entries; not an independent holder
- Supporting evidence: E5

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `shop.OrderCache.ORDERS_BY_ID`
- Region: heap
- Shape: steady
- Site: `src/shop/OrderCache.java:8`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F5 (same holder shop.OrderCache.ORDERS_BY_ID, same site OrderCache.java:8); not tested separately
- Supporting evidence: E23, E24, E26, E28, E30

## Ruled out

- **F5.n1** (UNCLASSIFIED): discriminator E20
  - Ruled out by: plateau+purge (leak) E20
  - Hypothesis: N-WARM
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E20
  - Originally supported by: E10

## Blind spots and unknowns

### Coverage gaps

- jvm/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- jvm/codeql: missing (codeql is not on PATH or in the tools directory). Install hint: dla install codeql
- jvm/errorprone: missing (Error Prone jars not found in the tools directory (or DLA_ERRORPRONE_HOME)). Install hint: dla install errorprone
- jvm/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)
- jvm/sonar: missing (sonar-scanner is not installed and Sonar rules need a SonarQube or SonarCloud server). Install hint: install sonar-scanner, run a SonarQube or SonarCloud server and set SONAR_HOST_URL and SONAR_TOKEN (dla install sonar prints the steps)
- jvm/spotbugs: blocked (no class files: run dla probe --build)
- jvm/valgrind: blocked (Valgrind is not run on the JVM: the virtual machine's own allocations drown the project's; use the allocator shim, jcmd NMT or `dla attach bpftrace`)

### Runtime blind spots

None recorded.

### Entry points not exercisable

None recorded.

### Evidence limitations and unknowns

- E5: partial coverage
- E5: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E5: limitation: writers and removers are textual name matches
- E6: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E6: limitation: workloads were discovered by reading files; none was run
- E7: limitation: harness written; nothing was run (dla harness run measures it)
- E7: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E7: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E8: partial coverage (9 of 9 examined)
- E8: limitation: command-mode heap dumps come from separate processes: dla summarize heap-graph compares dumps of one process only, so use a driver (dla harness gen --driver) for a dump series
- E8: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E8: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E9: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E9: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E9: limitation: gate thresholds are provisional (not calibrated)
- E10: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E10: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E12: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E13: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E14: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E15: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E15: limitation: series: per-point; one process per point at n = 1110, 4440, 6660, 9990 (harness captures; checked: HPROF version, identifier size)
- E16: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E17: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E17: limitation: the files carry no time, so they are ordered as given
- E18: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E18: limitation: the files carry no time, so they are ordered as given
- E18: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E19: partial coverage (13 of 13 examined)
- E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: points 1 to 9 come from S1; the extra points from S5
- E20: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E20: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E20: limitation: gate and plateau thresholds are provisional (not calibrated)
- E21: limitation: series: per-point; x is the iteration count n of each point (1110, 4440, 6660, 9990); every point is a separate process and heap dump
- E21: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E21: limitation: holders absent from a dump's 200 largest have no figure at that point
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: series: per-point; x is the iteration count n of each point (1110, 4440, 6660, 9990); every point is a separate process and heap dump
- E22: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E22: limitation: holders absent from a dump's 200 largest have no figure at that point
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E23: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E24: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E25: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E26: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E26: limitation: gate thresholds are provisional (not calibrated)
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
- E29: limitation: points 1 to 9 come from S1; the extra points from S5
- E29: limitation: gate thresholds are provisional (not calibrated)
- E30: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E30: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E30: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E30: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E30: limitation: points 1 to 9 come from S1; the extra points from S5
- E30: limitation: gate thresholds are provisional (not calibrated)
- E31: partial coverage (2 of 3 examined)
- E31: limitation: F5: growth evidence is from S3, not S1; not subtracted
- E31: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E31: limitation: threshold is provisional (not calibrated)
- E32: partial coverage (2 of 3 examined)
- E32: limitation: F5: growth evidence is from S3, not S4; not subtracted
- E32: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E32: limitation: threshold is provisional (not calibrated)
- E33: partial coverage (2 of 3 examined)
- E33: limitation: F5: growth evidence is from S3, not S5; not subtracted
- E33: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E33: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E33: limitation: threshold is provisional (not calibrated)
- E34: partial coverage (2 of 3 examined)
- E34: limitation: F5: growth evidence is from S3, not S6; not subtracted
- E34: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E34: limitation: threshold is provisional (not calibrated)
- E35: partial coverage (2 of 3 examined)
- E35: limitation: F5: growth evidence is from S3, not S1; not subtracted
- E35: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E35: limitation: threshold is provisional (not calibrated)
- E36: partial coverage (2 of 3 examined)
- E36: limitation: F5: growth evidence is from S3, not S4; not subtracted
- E36: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E36: limitation: threshold is provisional (not calibrated)
- E37: partial coverage (2 of 3 examined)
- E37: limitation: F5: growth evidence is from S3, not S5; not subtracted
- E37: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E37: limitation: O4 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O4 --as unresolved --reason time_budget --remedy "<text>"
- E37: limitation: threshold is provisional (not calibrated)
- E38: partial coverage (2 of 3 examined)
- E38: limitation: F5: growth evidence is from S3, not S6; not subtracted
- E38: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E38: limitation: threshold is provisional (not calibrated)

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
| E4 | static-hit | static | dla 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | sweep/jvm-dla.json |
| E5 | census-holder | static | dla-census 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | census/jvm.json |
| E6 | workload | static | dla 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | workload/workloads.json |
| E7 | workload | static | dla 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | harness/H1/harness.json |
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (8ae2b01ae4e5) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (d3f482e44df1) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (d3f482e44df1) | trend/T1.json |
| E11 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-184019-bbe6/harness/H1/out/p1-native/capture/134/heap-1.hprof (b8b1c6aacb3f) | summaries/heap-1-b8b1c6aa.json |
| E12 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-184019-bbe6/harness/H1/out/p4-native/capture/425/heap-1.hprof (c61808dc3d03) | summaries/heap-1-c61808dc.json |
| E13 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-184019-bbe6/harness/H1/out/p6-native/capture/619/heap-1.hprof (ac1ff518a9c8) | summaries/heap-1-ac1ff518.json |
| E14 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-184019-bbe6/harness/H1/out/p9-native/capture/910/heap-1.hprof (55e9edfd720e) | summaries/heap-1-55e9edfd.json |
| E15 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S2.json (4c292968b854) | series/S2.json |
| E16 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S2.json (4c292968b854) | series/S2.json |
| E17 | histogram-diff | live | jcmd-class-histogram 0.1.0 | artifact histograms/H1.json (f991e768dd70) | histograms/H1.json |
| E18 | growth-series | live | jcmd-class-histogram 0.1.0 | artifact series/S3.json (bc97521f56e0) | series/S3.json |
| E19 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (8ae2b01ae4e5) | harness/H1/run-S4.json |
| E20 | discriminator | test | dla 0.1.0 | artifact series/S6.json (88854d061b46) | alloc/A1.json |
| E21 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (4c292968b854) | trend/T3.json |
| E22 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (4c292968b854) | trend/T3.json |
| E23 | growth-series | dump | dla 0.1.0 | artifact series/S3.json (bc97521f56e0) | trend/T4.json |
| E24 | unit-count-series | dump | dla 0.1.0 | artifact series/S3.json (bc97521f56e0) | trend/T4.json |
| E25 | growth-series | test | dla 0.1.0 | artifact series/S4.json (5c936963dac6) | trend/T5.json |
| E26 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (5c936963dac6) | trend/T5.json |
| E27 | growth-series | test | dla 0.1.0 | artifact series/S5.json (7c9c34ceb1d4) | trend/T6.json |
| E28 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (7c9c34ceb1d4) | trend/T6.json |
| E29 | growth-series | test | dla 0.1.0 | artifact series/S6.json (88854d061b46) | trend/T7.json |
| E30 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (88854d061b46) | trend/T7.json |
| E31 | residual | test | dla 0.1.0 | artifact series/S1.json (d3f482e44df1) | gap/residual-S1-E31.json |
| E32 | residual | test | dla 0.1.0 | artifact series/S4.json (5c936963dac6) | gap/residual-S4-E32.json |
| E33 | residual | test | dla 0.1.0 | artifact series/S5.json (7c9c34ceb1d4) | gap/residual-S5-E33.json |
| E34 | residual | test | dla 0.1.0 | artifact series/S6.json (88854d061b46) | gap/residual-S6-E34.json |
| E35 | residual | test | dla 0.1.0 | artifact series/S1.json (d3f482e44df1) | gap/residual-S1-E35.json |
| E36 | residual | test | dla 0.1.0 | artifact series/S4.json (5c936963dac6) | gap/residual-S4-E36.json |
| E37 | residual | test | dla 0.1.0 | artifact series/S5.json (7c9c34ceb1d4) | gap/residual-S5-E37.json |
| E38 | residual | test | dla 0.1.0 | artifact series/S6.json (88854d061b46) | gap/residual-S6-E38.json |
