# DLA report

Run: `20261107-194838-9b91`

Status: COMPLETE

Findings: 6 enumerated, 6 dispositioned; confirmed 0, strong 0, refuted 0, suspected 4, unresolved 2

Residual: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: suspected_static_only(tool_missing)
- Evidence classes: static
- Site: `src/shop/OrderCache.java:8`
- Annotation: Same site as F5 (OrderCache.java:8, detector jvm-unbounded-collection); duplicate of F5, see F5 annotation for evidence
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: tool_missing
- Remedy: duplicate of F5; install a dla release with ledger test/confirm, harness, sample, trend, gap, alloc-test, summarize, bundle and gate commands, then re-run
- Supporting evidence: E5

### F2: **UNCLASSIFIED**

- State: suspected_static_only(tool_missing)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.Catalog.PRODUCTS`
- Region: heap
- Site: `src/shop/Catalog.java:9`
- Annotation: Catalog.PRODUCTS is a fixed List.of of 12 items, constant size, not growing
- Reason: tool_missing
- Remedy: constant-size or dependent holder (see annotation); no growth series; install a dla release with ledger test/confirm, harness, sample, trend, gap, alloc-test, summarize, bundle and gate commands, then re-run
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: suspected_static_only(tool_missing)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.Main.CUSTOMERS`
- Region: heap
- Site: `src/shop/Main.java:5`
- Annotation: Main.CUSTOMERS is a fixed 6-element array, constant size, not growing
- Reason: tool_missing
- Remedy: constant-size or dependent holder (see annotation); no growth series; install a dla release with ledger test/confirm, harness, sample, trend, gap, alloc-test, summarize, bundle and gate commands, then re-run
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: suspected_static_only(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.Order.lines`
- Region: heap
- Site: `src/shop/Order.java:6`
- Writers: `src/shop/OrderService.java:28`
- Annotation: Order.lines is per-Order list; retained only because the Order is held by OrderCache (F5); not an independent holder
- Reason: tool_missing
- Remedy: constant-size or dependent holder (see annotation); no growth series; install a dla release with ledger test/confirm, harness, sample, trend, gap, alloc-test, summarize, bundle and gate commands, then re-run
- Supporting evidence: E6

### F5: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.OrderCache.ORDERS_BY_ID`
- Region: heap
- Site: `src/shop/OrderCache.java:8`
- Writers: `src/shop/OrderCache.java:17`, `src/shop/OrderService.java:28`
- Removers: `src/shop/OrderCache.java:26`
- Annotation: heuristic name match over the project's sources
- Annotation: Only writer on the request path is OrderService.place (store, line 28). The sole remover, invalidateAll, is called only from tests; no TTL, size bound or eviction in production. Measured with ./build.sh run 40000: post-System.gc heap 6414/11277/16251/21098 KB at 10k/20k/30k/40k requests, cache size 8750/17500/26250/35000 (matches placed count); ~0.55 KB retained per cached Order (Order, Line list, id and notes strings).
- Sibling hypotheses: F5.n1
- Reason: tool_missing
- Remedy: Manually measured growth (see annotation) but ledger confirm/test, trend, summarize (heap-graph) missing: install a dla release with ledger test/confirm, harness, sample, trend, gap, alloc-test, summarize, bundle and gate commands, then re-run
- Supporting evidence: E6

### F5.n1: Hypothesis N-WARM (UNCLASSIFIED)

- State: unresolved(tool_missing)
- Evidence classes: context
- Sibling hypothesis of: F5
- Hypothesis: N-WARM
- Refuted if: heap after forced GC plateaus when cache size stops rising / repeats of the same ids
- Reason: tool_missing
- Remedy: N-WARM not formally refuted; cache grows linearly to 35000 entries with no plateau across 4 checkpoints; install a dla release with ledger test/confirm, harness, sample, trend, gap, alloc-test, summarize, bundle and gate commands, then re-run
- Supporting evidence: E3

## Ruled out

None.

## Blind spots and unknowns

### Coverage gaps

- jvm/codeql: missing (codeql is not on PATH or in the tools directory). Install hint: dla install codeql
- jvm/errorprone: missing (Error Prone jars not found in the tools directory (or DLA_ERRORPRONE_HOME)). Install hint: dla install errorprone
- jvm/sonar: missing (sonar-scanner is not installed and Sonar rules need a SonarQube or SonarCloud server). Install hint: install sonar-scanner, run a SonarQube or SonarCloud server and set SONAR_HOST_URL and SONAR_TOKEN (dla install sonar prints the steps)

### Runtime blind spots

None recorded.

### Entry points not exercisable

- jvm: `src/shop/Catalog.java::Catalog.at`: no_harness

### Evidence limitations and unknowns

- E4: partial coverage
- E4: limitation: no auxiliary classpath: library calls are not modelled
- E6: partial coverage
- E6: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E6: limitation: writers and removers are textual name matches
- E7: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E7: limitation: workloads were discovered by reading files; none was run

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | order service JVM (src/shop), OrderCache static map suspected |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | whole repo: src/shop/*.java, test/shop |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | live set grows after each batch during replay; pod restarted ~daily |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | jvm (java, javac via build.sh) |
| `intake.build` | Does the project build locally? | answered | E1 | ./build.sh [run N] |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | shop.Main replay driver, count arg |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4, raw values |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 3600s |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | — |
| E2 | run-config | static | dla 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | probe/probe.json |
| E4 | static-hit | static | spotbugs unknown | repo 658c6e857134364322cd5318f94779646f61fc7a | sweep/spotbugs.json |
| E5 | static-hit | static | dla 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | sweep/dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | census/jvm.json |
| E7 | workload | static | dla 0.1.0 | repo 658c6e857134364322cd5318f94779646f61fc7a | workload/workloads.json |
