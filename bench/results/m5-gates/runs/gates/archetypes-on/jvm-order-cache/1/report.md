# DLA report

Run: `20261109-200425-bc1d`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 6 enumerated, 6 dispositioned; confirmed 1, strong 0, refuted 1, suspected 4, unresolved 0

Residual: not measured

Coverage: jvm 8880 iterations, 14 of 14 entry points exercised

## Findings

### F5: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `holder:static:shop.OrderCache.ORDERS_BY_ID` 1487923 bytes/n (steady, gate pass, E16)
- Evidence classes: context, discriminator, growth, static
- Holder: static/global
- Owner: `shop.OrderCache.ORDERS_BY_ID`
- Region: heap
- Shape: steady
- Site: `src/shop/OrderCache.java:8`
- Writers: `src/shop/OrderCache.java:17`
- Removers: `src/shop/OrderCache.java:26`
- Annotation: heuristic name match over the project's sources
- Annotation: Static HashMap ORDERS_BY_ID in OrderCache: store() on every new order id, never evicted (invalidateAll only used by tests); each Order retains notes string and line list. Missing pairing: eviction/TTL/size bound. UNCLASSIFIED (no label commands).
- Sibling hypotheses: F5.n1
- Supporting evidence: E6, E9, E16, E11, E18

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Site: `src/shop/OrderCache.java:8`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: Same site/holder as confirmed F5 (OrderCache.java:8); duplicate detector hit, covered by F5
- Supporting evidence: E5

### F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.Catalog.PRODUCTS`
- Region: heap
- Site: `src/shop/Catalog.java:9`
- Reason: not_exercised
- Remedy: Immutable fixed-size data (no growth column tied in harness); no action
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.Main.CUSTOMERS`
- Region: heap
- Site: `src/shop/Main.java:5`
- Reason: not_exercised
- Remedy: Immutable fixed-size data (no growth column tied in harness); no action
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.Order.lines`
- Region: heap
- Site: `src/shop/Order.java:6`
- Reason: not_exercised
- Remedy: Immutable fixed-size data (no growth column tied in harness); no action
- Supporting evidence: E6

## Ruled out

- **F5.n1** (UNCLASSIFIED): discriminator E18
  - Ruled out by: plateau+purge (leak) E18
  - Hypothesis: N-WARM: bounded warm-up fill that plateaus
  - Refuted if: retained size keeps growing strictly across all dumps / extra cycles
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
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (60b3d430d16d) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (0a1c77908d0b) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (0a1c77908d0b) | trend/T1.json |
| E12 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261109-200425-bc1d/harness/H1/out/p1-native/capture/82/heap-1.hprof (cd28d1379c9e) | summaries/heap-1-cd28d137.json |
| E13 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261109-200425-bc1d/harness/H1/out/p4-native/capture/373/heap-1.hprof (64e3642dca41) | summaries/heap-1-64e3642d.json |
| E14 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261109-200425-bc1d/harness/H1/out/p6-native/capture/567/heap-1.hprof (ffd79a54ce8b) | summaries/heap-1-ffd79a54.json |
| E15 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261109-200425-bc1d/harness/H1/out/p9-native/capture/858/heap-1.hprof (f529fa705e82) | summaries/heap-1-f529fa70.json |
| E16 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S2.json (f25f67b75642) | series/S2.json |
| E17 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S2.json (f25f67b75642) | series/S2.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S4.json (6211d501dc3d) | alloc/A1.json |
