# DLA report

Run: `20261108-130151-b47b`

Status: COMPLETE

Findings: 6 enumerated, 6 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 4

Residual: not measured

Coverage: jvm 8880 iterations, 14 of 14 entry points exercised

## Findings

### F5: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:shop.OrderCache.ORDERS_BY_ID` 6.75 count/n (steady, gate pass, E12)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `shop.OrderCache.ORDERS_BY_ID`
- Region: heap
- Shape: steady
- Site: `src/shop/OrderCache.java:8`
- Writers: `src/shop/OrderCache.java:17`, `src/shop/OrderService.java:23`
- Removers: `src/shop/OrderCache.java:26`
- Annotation: heuristic name match over the project's sources
- Annotation: put() on every new order id; no remover except test-only invalidateAll (line 25), never called from Main/OrderService
- Sibling hypotheses: F5.n1
- Supporting evidence: E7, E10, E12

### F1: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: discriminator, static
- Holder: static/global
- Site: `src/shop/OrderCache.java:8`
- Writers: `src/shop/OrderCache.java:17`
- Annotation: same site as F5
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: tool_missing
- Remedy: No growth column tied to it; F1 duplicates F5 (same site); F2/F3 immutable constants, F4 retained only via F5. Re-test after fixing F5.
- Supporting evidence: E6, E13

### F2: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: discriminator, static
- Holder: static/global
- Owner: `shop.Catalog.PRODUCTS`
- Region: heap
- Site: `src/shop/Catalog.java:9`
- Annotation: immutable fixed 12-item list; no writer
- Reason: tool_missing
- Remedy: No growth column tied to it; F1 duplicates F5 (same site); F2/F3 immutable constants, F4 retained only via F5. Re-test after fixing F5.
- Supporting evidence: E7, E13

### F3: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: discriminator, static
- Holder: static/global
- Owner: `shop.Main.CUSTOMERS`
- Region: heap
- Site: `src/shop/Main.java:5`
- Annotation: immutable constant array
- Reason: tool_missing
- Remedy: No growth column tied to it; F1 duplicates F5 (same site); F2/F3 immutable constants, F4 retained only via F5. Re-test after fixing F5.
- Supporting evidence: E7, E13

### F4: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: discriminator, static
- Holder: container-field
- Owner: `shop.Order.lines`
- Region: heap
- Site: `src/shop/Order.java:6`
- Annotation: per-Order copyOf list, bounded by Order lifetime; retained via F5
- Reason: tool_missing
- Remedy: No growth column tied to it; F1 duplicates F5 (same site); F2/F3 immutable constants, F4 retained only via F5. Re-test after fixing F5.
- Supporting evidence: E7, E13

## Ruled out

- **F5.n1** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (leak) E13
  - Hypothesis: N-WARM cache fills to a bounded working set then plateaus
  - Refuted if: map size keeps growing across extra cycles and after purge
  - Contradicting evidence: E13
  - Originally supported by: E12

## Blind spots and unknowns

### Coverage gaps

- jvm/codeql: missing (codeql is not on PATH or in the tools directory). Install hint: dla install codeql
- jvm/errorprone: missing (Error Prone jars not found in the tools directory (or DLA_ERRORPRONE_HOME)). Install hint: dla install errorprone
- jvm/sonar: missing (sonar-scanner is not installed and Sonar rules need a SonarQube or SonarCloud server). Install hint: install sonar-scanner, run a SonarQube or SonarCloud server and set SONAR_HOST_URL and SONAR_TOKEN (dla install sonar prints the steps)

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

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | Local repository: Java order service (src/shop) |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | Whole repo, src/shop and test |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | Live set grows after every traffic batch during replay; pod restarted ~daily |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E3 | jvm only |
| `intake.build` | Does the project build locally? | answered | E3 | build.sh; verified by probe --build |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | Replay traffic via Main; discover next |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | Runner consent tiers 1-4, raw values |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 3600s |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | — |
| E2 | run-config | static | dla 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | probe/probe.json |
| E4 | probe | static | dla-probe 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | probe/probe.json |
| E5 | static-hit | static | spotbugs unknown | repo 658c6e857134364322cd5318f94779646f61fc7a | sweep/spotbugs.json |
| E6 | static-hit | static | dla 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | sweep/dla.json |
| E7 | census-holder | static | dla-census 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | census/jvm.json |
| E8 | workload | static | dla 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | workload/workloads.json |
| E9 | workload | static | dla 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | harness/H1/harness.json |
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (a1bb19919a42) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (3c5762f30a96) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (3c5762f30a96) | trend/T1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S3.json (95bc816fe062) | alloc/A1.json |
