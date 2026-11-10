# DLA report

Run: `20261110-191511-24f4`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 7 enumerated, 7 dispositioned; confirmed 0, strong 1, refuted 1, suspected 0, unresolved 5

Residual: heap 508.5278078078078 bytes/n unattributed of 508.5278078078078 total (threshold 127.13195195195196) ABOVE THRESHOLD, see O2; heap 508.86054054054057 bytes/n unattributed of 508.86054054054057 total (threshold 127.21513513513514) ABOVE THRESHOLD, see O1; heap 507.3594294294294 bytes/n unattributed of 507.3594294294294 total (threshold 126.83985735735736) ABOVE THRESHOLD, see O1; non-heap 0 bytes/n unattributed of -1.9627627627627628 total (threshold 32.812590270270356); non-heap 4.143543543543544 bytes/n unattributed of 4.143543543543544 total (threshold 17.668508648648828); non-heap 1.2737594737594735 bytes/n unattributed of 1.2737594737594735 total (threshold 6.562988720720709)

Coverage: jvm 8880 iterations, 14 of 14 entry points exercised

## Findings

### F5: **UNCLASSIFIED**

- State: strong
- Label: UNCLASSIFIED (Static HashMap OrderCache.ORDERS_BY_ID is written by store() for every new order and never evicted or bounded; Order objects (with lines and notes strings) are retained for the process lifetime)
- Growth: `probe:shop.OrderCache.ORDERS_BY_ID` 6.75 count/n (steady, gate pass, E11)
- Evidence classes: context, growth, retention, static
- Strong: second class E24 retention-path that names the holder, independent of the growth record E11 (dla)
- Holder: static/global
- Owner: `shop.OrderCache.ORDERS_BY_ID`
- Region: heap
- Shape: steady
- Site: `src/shop/OrderCache.java:8`
- Writers: `src/shop/OrderCache.java:17`
- Removers: `src/shop/OrderCache.java:26`
- Annotation: heuristic name match over the project's sources
- Annotation: ORDERS_BY_ID static HashMap, store() puts every order; no eviction; only invalidateAll (tests) clears
- Annotation: no remover in production code; invalidateAll (line 29) is test-only
- Verified: no
- Sibling hypotheses: F5.n1
- Supporting evidence: E6, E9, E11, E13, E24
- Explanation (observed): Static map ORDERS_BY_ID declared at OrderCache.java:8 (static/global holder) receives every new Order via store() at OrderCache.java:17; entries are removed only if invalidateAll at OrderCache.java:26 runs, which is test-only, so production never evicts.
- Explanation structure: adds: src/shop/OrderCache.java:17; removal: conditional at src/shop/OrderCache.java:26; evidence: E25

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `src/shop/OrderCache.java:8`
- Reason: not_exercised
- Remedy: duplicate of F5 (same map, detector hit); see F5
- Supporting evidence: E5

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.Catalog.PRODUCTS`
- Region: heap
- Site: `src/shop/Catalog.java:9`
- Reason: not_exercised
- Remedy: no growth tied; fixed-size/immutable holder (census lead only); F4 Order.lines is per-order content retained via F5
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.Main.CUSTOMERS`
- Region: heap
- Site: `src/shop/Main.java:5`
- Reason: not_exercised
- Remedy: no growth tied; fixed-size/immutable holder (census lead only); F4 Order.lines is per-order content retained via F5
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.Order.lines`
- Region: heap
- Site: `src/shop/Order.java:6`
- Reason: not_exercised
- Remedy: no growth tied; fixed-size/immutable holder (census lead only); F4 Order.lines is per-order content retained via F5
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
- Remedy: growth-series duplicate: these type/count columns are the contents of orders held by F5; fixing F5 removes them
- Supporting evidence: E15, E16, E18, E20

## Ruled out

- **F5.n1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (leak) E14
  - Hypothesis: N-WARM: orders cache is a warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E14
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
- E12: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E12: limitation: the files carry no time, so they are ordered as given
- E13: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E13: limitation: the files carry no time, so they are ordered as given
- E13: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
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
- E17: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: points 1 to 9 come from S1; the extra points from S3
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: points 1 to 9 come from S1; the extra points from S3
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E22: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E23: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E24: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E25: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E25: limitation: series: per-point; one process per point at n = 1110, 4440, 6660, 9990 (harness captures; checked: HPROF version, identifier size)
- E26: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E27: limitation: tested in a scratch copy; the project was not changed
- E27: limitation: gate thresholds are provisional (not calibrated); the 0.2 and 0.5 slope fractions are placeholders
- E27: limitation: exercised is null: the harness records entry points, not lines, so whether the edited line ran is not checked
- E27: limitation: every measured process of mode native exited with a non-zero code (p1-native: 1, p2-native: 1, p3-native: 1, p4-native: 1): the harness itself failed, so no series was written
- E28: limitation: series: per-point; x is the iteration count n of each point (1110, 4440, 6660, 9990); every point is a separate process and heap dump
- E28: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E28: limitation: holders absent from a dump's 200 largest have no figure at that point
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: limitation: series: per-point; x is the iteration count n of each point (1110, 4440, 6660, 9990); every point is a separate process and heap dump
- E29: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E29: limitation: holders absent from a dump's 200 largest have no figure at that point
- E29: limitation: gate thresholds are provisional (not calibrated)
- E30: partial coverage (2 of 3 examined)
- E30: limitation: F5: growth measured as a count; bytes not attributed
- E30: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E30: limitation: threshold is provisional (not calibrated)
- E31: partial coverage (2 of 3 examined)
- E31: limitation: F5: growth evidence is from S1, not S3; not subtracted
- E31: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E31: limitation: threshold is provisional (not calibrated)
- E32: partial coverage (2 of 3 examined)
- E32: limitation: F5: growth evidence is from S1, not S4; not subtracted
- E32: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E32: limitation: threshold is provisional (not calibrated)
- E33: partial coverage (2 of 3 examined)
- E33: limitation: F5: growth measured as a count; bytes not attributed
- E33: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (4f68c04de1fb) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (6e15ab50b9f5) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (6e15ab50b9f5) | trend/T1.json |
| E12 | histogram-diff | live | jcmd-class-histogram 0.1.0 | artifact histograms/H1.json (93807074c808) | histograms/H1.json |
| E13 | growth-series | live | jcmd-class-histogram 0.1.0 | artifact series/S2.json (027112b27679) | series/S2.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S4.json (815f171983d2) | alloc/A1.json |
| E15 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (027112b27679) | trend/T3.json |
| E16 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (027112b27679) | trend/T3.json |
| E17 | growth-series | test | dla 0.1.0 | artifact series/S3.json (69bf79577565) | trend/T4.json |
| E18 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (69bf79577565) | trend/T4.json |
| E19 | growth-series | test | dla 0.1.0 | artifact series/S4.json (815f171983d2) | trend/T5.json |
| E20 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (815f171983d2) | trend/T5.json |
| E21 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-191511-24f4/harness/H1/out/p1-native/capture/82/heap-1.hprof (6d5a82d0463d) | summaries/heap-1-6d5a82d0.json |
| E22 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-191511-24f4/harness/H1/out/p4-native/capture/374/heap-1.hprof (a6ee1df606ec) | summaries/heap-1-a6ee1df6.json |
| E23 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-191511-24f4/harness/H1/out/p6-native/capture/568/heap-1.hprof (bd53145a0c4e) | summaries/heap-1-bd53145a.json |
| E24 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-191511-24f4/harness/H1/out/p9-native/capture/859/heap-1.hprof (0f481ad14008) | summaries/heap-1-0f481ad1.json |
| E25 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S5.json (a0b86c68303b) | series/S5.json |
| E26 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S5.json (a0b86c68303b) | series/S5.json |
| E27 | intervention | test | dla 0.1.0 | artifact intervene/I1/patch.diff (afaaa47a34da) | intervene/I1 |
| E28 | growth-series | dump | dla 0.1.0 | artifact series/S5.json (a0b86c68303b) | trend/T6.json |
| E29 | unit-count-series | dump | dla 0.1.0 | artifact series/S5.json (a0b86c68303b) | trend/T6.json |
| E30 | residual | test | dla 0.1.0 | artifact series/S1.json (6e15ab50b9f5) | gap/residual-S1-E30.json |
| E31 | residual | test | dla 0.1.0 | artifact series/S3.json (69bf79577565) | gap/residual-S3-E31.json |
| E32 | residual | test | dla 0.1.0 | artifact series/S4.json (815f171983d2) | gap/residual-S4-E32.json |
| E33 | residual | test | dla 0.1.0 | artifact series/S1.json (6e15ab50b9f5) | gap/residual-S1-E33.json |
