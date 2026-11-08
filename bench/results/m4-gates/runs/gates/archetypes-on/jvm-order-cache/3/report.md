# DLA report

Run: `20261108-203732-9f61`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 6 enumerated, 6 dispositioned; confirmed 1, strong 0, refuted 3, suspected 0, unresolved 2

Residual: not measured

Coverage: jvm 8880 iterations, 14 of 14 entry points exercised

## Findings

### F5: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:shop.OrderCache.ORDERS_BY_ID` 6.75 count/n (steady, gate pass, E11)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `shop.OrderCache.ORDERS_BY_ID`
- Region: heap
- Shape: steady
- Site: `src/shop/OrderCache.java:8`
- Writers: `src/shop/OrderCache.java:17`, `src/shop/OrderService.java:25`
- Removers: `src/shop/OrderCache.java:26`
- Annotation: heuristic name match over the project's sources
- Annotation: OrderCache.ORDERS_BY_ID is a static HashMap with no eviction or size bound. OrderService.place stores every new order (key customer+requestId, unique per request) via OrderCache.store; the only remover, invalidateAll, is called solely from tests. Each Order retains its notes String and line list, so the live set grows with every distinct request. Missing pairing: a bound/eviction (size or TTL) for the cache.
- Sibling hypotheses: F5.n1
- Supporting evidence: E6, E9, E11

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `src/shop/OrderCache.java:8`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: Same site and holder as confirmed F5 (OrderCache.java:8); detector duplicate, no separate action. Fix F5.
- Supporting evidence: E5

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.Order.lines`
- Region: heap
- Site: `src/shop/Order.java:6`
- Reason: not_exercised
- Remedy: Order.lines is a per-order field retained only through F5's cache; not an independent holder. Fix F5.
- Supporting evidence: E6

## Ruled out

- **F2** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (non-leak) E13
  - Contradicting evidence: E13
  - Originally supported by: E6
- **F3** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (non-leak) E14
  - Contradicting evidence: E14
  - Originally supported by: E6
- **F5.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM: bounded warm-up fill of the order cache
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E12
  - Originally supported by: E11

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
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: points 1 to 9 come from S1; the extra points from S4
- E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E13: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: points 1 to 9 come from S1; the extra points from S6
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: points 1 to 9 come from S1; the extra points from S8
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: points 1 to 9 come from S1; the extra points from S10
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | — |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | — |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | — |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E3 | — |
| `intake.build` | Does the project build locally? | answered | E3 | — |
| `intake.workload` | What can exercise the suspected path? | answered | E3 | — |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E3 | — |

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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (af748fb46bee) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (8265bcf3619e) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (8265bcf3619e) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (1345b9770675) | alloc/A1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S5.json (88d657b6edc1) | alloc/A2.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S7.json (f3a100b45aae) | alloc/A3.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S9.json (eebf1e425672) | alloc/A4.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S11.json (df3f881281cd) | alloc/A5.json |
