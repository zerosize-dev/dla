# DLA report

Run: `20261110-172037-2b28`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 9 enumerated, 9 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 7

Residual: heap 507.603432003432 bytes/n unattributed of 507.603432003432 total (threshold 126.900858000858) ABOVE THRESHOLD, see O1; heap 515.2046332046332 bytes/n unattributed of 515.2046332046332 total (threshold 128.8011583011583) ABOVE THRESHOLD, see O1; heap 510.2445645645646 bytes/n unattributed of 510.2445645645646 total (threshold 127.56114114114116) ABOVE THRESHOLD, see O1; non-heap 4.433033033033033 bytes/n unattributed of 4.433033033033033 total (threshold 74.06798648648666); non-heap 6.4246246246246255 bytes/n unattributed of 6.4246246246246255 total (threshold 31.667343783783696); non-heap 5.938138138138138 bytes/n unattributed of 5.938138138138138 total (threshold 19.969419891891768)

Coverage: jvm 8880 iterations, 14 of 14 entry points exercised

## Findings

### F5: **L-COLLECT**

- State: confirmed
- Label: L-COLLECT (evidence: E11 unit-count-series, E6 census-holder)
- Growth: `probe:shop.OrderCache.ORDERS_BY_ID` 6.75 count/n (steady, gate pass, E11)
- Evidence classes: context, discriminator, growth, static
- Holder: static/global
- Owner: `shop.OrderCache.ORDERS_BY_ID`
- Region: heap
- Shape: steady
- Site: `src/shop/OrderCache.java:8`
- Writers: `src/shop/OrderCache.java:17`, `src/shop/OrderService.java:23`
- Removers: `src/shop/OrderCache.java:26`
- Annotation: heuristic name match over the project's sources
- Annotation: store() on every non-cached order; only remover invalidateAll() is called from tests only
- Verified: no
- Sibling hypotheses: F5.n1
- Supporting evidence: E6, E9, E11, E12
- Explanation (inferred): Static HashMap ORDERS_BY_ID (OrderCache.java:8, static/global holder) gains an Order per distinct order id via store() at OrderCache.java:17, called from OrderService.place; the only remover invalidateAll (OrderCache.java:26) runs only when tests call it, never in service, so entries stay.
- Explanation structure: adds: src/shop/OrderCache.java:17; removal: conditional at src/shop/OrderCache.java:26

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `src/shop/OrderCache.java:8`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: Duplicate of confirmed F5 (same holder ORDERS_BY_ID, same site OrderCache.java:8); covered by F5's confirmation, no separate test
- Supporting evidence: E5

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.Catalog.PRODUCTS`
- Region: heap
- Site: `src/shop/Catalog.java:9`
- Reason: not_exercised
- Remedy: Fixed-size/immutable holder (no growth column tied; census lead only); no separate test needed
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.Main.CUSTOMERS`
- Region: heap
- Site: `src/shop/Main.java:5`
- Reason: not_exercised
- Remedy: Fixed-size/immutable holder (no growth column tied; census lead only); no separate test needed
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.Order.lines`
- Region: heap
- Site: `src/shop/Order.java:6`
- Reason: not_exercised
- Remedy: Fixed-size/immutable holder (no growth column tied; census lead only); no separate test needed
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
- Remedy: Duplicate of confirmed F5 (same holder ORDERS_BY_ID, same site OrderCache.java:8); covered by F5's confirmation, no separate test
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
- Remedy: Duplicate of confirmed F5 (same holder ORDERS_BY_ID, same site OrderCache.java:8); covered by F5's confirmation, no separate test
- Supporting evidence: E14

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `shop.OrderCache.ORDERS_BY_ID`
- Region: heap
- Shape: steady
- Site: `src/shop/OrderCache.java:8`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F5 (same holder ORDERS_BY_ID, same site OrderCache.java:8); covered by F5's confirmation, no separate test
- Supporting evidence: E16

## Ruled out

- **F5.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM: cache fills to a working-set ceiling then plateaus
  - Refuted if: size/bytes keep growing across extra cycles and after purge
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
- E17: partial coverage (2 of 3 examined)
- E17: limitation: F5: growth measured as a count; bytes not attributed
- E17: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E17: limitation: threshold is provisional (not calibrated)
- E18: partial coverage (2 of 3 examined)
- E18: limitation: F5: growth evidence is from S1, not S2; not subtracted
- E18: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E18: limitation: threshold is provisional (not calibrated)
- E19: partial coverage (2 of 3 examined)
- E19: limitation: F5: growth evidence is from S1, not S3; not subtracted
- E19: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E19: limitation: threshold is provisional (not calibrated)

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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (9e7cb15d55d8) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (04a1dc7ef16d) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (04a1dc7ef16d) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (ed32a4516e7f) | alloc/A1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S2.json (bd9cf28ab167) | trend/T3.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (bd9cf28ab167) | trend/T3.json |
| E15 | growth-series | test | dla 0.1.0 | artifact series/S3.json (ed32a4516e7f) | trend/T4.json |
| E16 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (ed32a4516e7f) | trend/T4.json |
| E17 | residual | test | dla 0.1.0 | artifact series/S1.json (04a1dc7ef16d) | gap/residual-S1-E17.json |
| E18 | residual | test | dla 0.1.0 | artifact series/S2.json (bd9cf28ab167) | gap/residual-S2-E18.json |
| E19 | residual | test | dla 0.1.0 | artifact series/S3.json (ed32a4516e7f) | gap/residual-S3-E19.json |
