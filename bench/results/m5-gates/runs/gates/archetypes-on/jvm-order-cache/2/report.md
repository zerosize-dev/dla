# DLA report

Run: `20261109-203157-36e9`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 6 enumerated, 6 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 4

Residual: not measured

Coverage: jvm 8880 iterations, 14 of 14 entry points exercised

## Findings

### F5: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:shop.OrderCache.ORDERS_BY_ID.bytes` 490.2 bytes/n (steady, gate pass, E11)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `shop.OrderCache.ORDERS_BY_ID`
- Region: heap
- Shape: steady
- Site: `src/shop/OrderCache.java:8`
- Writers: `src/shop/OrderCache.java:17`
- Removers: `src/shop/OrderCache.java:26`, `src/shop/OrderCache.java:25`
- Annotation: heuristic name match over the project's sources
- Annotation: Static HashMap ORDERS_BY_ID is put() on every new order and never evicted; the only remover invalidateAll() is test-only. No bound/TTL, so every distinct Order (lines, notes strings) stays reachable for the process lifetime. Missing pairing: eviction/bounded size. No taxonomy label available: UNCLASSIFIED.
- Sibling hypotheses: F5.n1
- Supporting evidence: E7, E10, E11, E18

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `src/shop/OrderCache.java:8`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: duplicate detector hit of F5 (same site OrderCache.java:8)
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.Catalog.PRODUCTS`
- Region: heap
- Site: `src/shop/Catalog.java:9`
- Reason: not_exercised
- Remedy: census lead; no growth column tied to it (constant/immutable holder); no action unless trend shows growth
- Supporting evidence: E7

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.Main.CUSTOMERS`
- Region: heap
- Site: `src/shop/Main.java:5`
- Reason: not_exercised
- Remedy: census lead; no growth column tied to it (constant/immutable holder); no action unless trend shows growth
- Supporting evidence: E7

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.Order.lines`
- Region: heap
- Site: `src/shop/Order.java:6`
- Reason: not_exercised
- Remedy: census lead; no growth column tied to it (constant/immutable holder); no action unless trend shows growth
- Supporting evidence: E7

## Ruled out

- **F5.n1** (UNCLASSIFIED): discriminator E19
  - Ruled out by: dump-plateau (leak) E19
  - Hypothesis: N-WARM: bounded warm-up cache fill
  - Refuted if: growth continues over the extra cycles / dump-plateau favors leak
  - Contradicting evidence: E19
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
- E10: limitation: /proc/43/status lacks os.rss, os.rss_anon, os.rss_file, os.rss_shmem, os.swap, os.vm_data, os.vm_hwm (kernel thread or old kernel)
- E10: limitation: os.pss, os.pss_anon, os.private_dirty: /proc/43/smaps_rollup unreadable (No such process (os error 3))
- E10: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E10: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E11: limitation: /proc/43/status lacks os.rss, os.rss_anon, os.rss_file, os.rss_shmem, os.swap, os.vm_data, os.vm_hwm (kernel thread or old kernel)
- E11: limitation: os.pss, os.pss_anon, os.private_dirty: /proc/43/smaps_rollup unreadable (No such process (os error 3))
- E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E11: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: /proc/43/status lacks os.rss, os.rss_anon, os.rss_file, os.rss_shmem, os.swap, os.vm_data, os.vm_hwm (kernel thread or old kernel)
- E12: limitation: os.pss, os.pss_anon, os.private_dirty: /proc/43/smaps_rollup unreadable (No such process (os error 3))
- E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: /proc/43/status lacks os.rss, os.rss_anon, os.rss_file, os.rss_shmem, os.swap, os.vm_data, os.vm_hwm (kernel thread or old kernel)
- E13: limitation: os.pss, os.pss_anon, os.private_dirty: /proc/43/smaps_rollup unreadable (No such process (os error 3))
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: points 1 to 9 come from S1; the extra points from S2
- E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E13: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- E14: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E15: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E16: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E17: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E18: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E18: limitation: series: per-point; one process per point at n = 1110, 4440, 6660, 9990 (harness captures; checked: HPROF version, identifier size)
- E19: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)

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
| E4 | probe | static | dla-probe 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | probe/probe.json |
| E5 | static-hit | static | spotbugs unknown | repo 658c6e857134364322cd5318f94779646f61fc7a | sweep/jvm-spotbugs.json |
| E6 | static-hit | static | dla 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | sweep/jvm-dla.json |
| E7 | census-holder | static | dla-census 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | census/jvm.json |
| E8 | workload | static | dla 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | workload/workloads.json |
| E9 | workload | static | dla 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | harness/H1/harness.json |
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (8da66095a8aa) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (69cccde85771) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (69cccde85771) | trend/T1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S3.json (d5a709d7bdc7) | alloc/A1.json |
| E14 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261109-203157-36e9/harness/H1/out/p1-native/capture/82/heap-1.hprof (814d7b795d8e) | summaries/heap-1-814d7b79.json |
| E15 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261109-203157-36e9/harness/H1/out/p4-native/capture/373/heap-1.hprof (2fa8a941b490) | summaries/heap-1-2fa8a941.json |
| E16 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261109-203157-36e9/harness/H1/out/p6-native/capture/567/heap-1.hprof (b5ac90f94cda) | summaries/heap-1-b5ac90f9.json |
| E17 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261109-203157-36e9/harness/H1/out/p9-native/capture/858/heap-1.hprof (48115b6d047f) | summaries/heap-1-48115b6d.json |
| E18 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S4.json (19e08be265f6) | series/S4.json |
| E19 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S4.json (19e08be265f6) | series/S4.json |
