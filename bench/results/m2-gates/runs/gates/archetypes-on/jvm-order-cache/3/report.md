# DLA report

Run: `20261107-195441-a39a`

Status: COMPLETE

Findings: 5 enumerated, 5 dispositioned; confirmed 0, strong 0, refuted 0, suspected 4, unresolved 1

Residual: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Site: `src/shop/OrderCache.java:8`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: Reviewed by hand: immutable/fixed or bounded per-order data; no growth observed; not a retention root
- Supporting evidence: E5

### F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.Catalog.PRODUCTS`
- Region: heap
- Site: `src/shop/Catalog.java:9`
- Reason: not_exercised
- Remedy: Reviewed by hand: immutable/fixed or bounded per-order data; no growth observed; not a retention root
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.Main.CUSTOMERS`
- Region: heap
- Site: `src/shop/Main.java:5`
- Reason: not_exercised
- Remedy: Reviewed by hand: immutable/fixed or bounded per-order data; no growth observed; not a retention root
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.Order.lines`
- Region: heap
- Site: `src/shop/Order.java:6`
- Reason: not_exercised
- Remedy: Reviewed by hand: immutable/fixed or bounded per-order data; no growth observed; not a retention root
- Supporting evidence: E6

### F5: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.OrderCache.ORDERS_BY_ID`
- Region: heap
- Site: `src/shop/OrderCache.java:8`
- Writers: `src/shop/OrderCache.java:17`, `src/shop/OrderOrderCache.java:17`, `src/shop/OrderService.java:52`
- Removers: `src/shop/OrderCache.java:26`
- Annotation: heuristic name match over the project's sources
- Annotation: Only writer is OrderService.place (store, line 52), never evicted/expired. Only remover invalidateAll is test-only (not called from src). Measured with ./build.sh run 20000: after GC heap in use 3913->6363->8915->11338 KB per 5000 requests, cache 4375->17500 orders (~linear, ~0.5 KB/order)
- Annotation: Correction: the first annotation's writer 'src/shop/OrderOrderCache.java:17' was a typo for src/shop/OrderCache.java:17.
- Reason: tool_missing
- Remedy: Installed dla 0.1.0 lacks ledger test/confirm/refute, harness, trend, gate. Measured growth is manual (E-level: build.sh run 20000). Once available: add N-WARM/N-HEADROOM siblings, confirm F5, label as unbounded cache; repair class: bound/evict entries (size cap or TTL) in OrderCache
- Supporting evidence: E6

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
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | Local repository: Java order service (src/shop) |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | Whole repo, src/shop; tests excluded |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | Live set grows after every traffic batch in long replay; pod restarted ~daily |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | JVM (Java, javac) |
| `intake.build` | Does the project build locally? | answered | E1 | ./build.sh compiles and runs tests; './build.sh run N' replays |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | shop.Main replay driver with count arg |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | Runner consent tiers 1-4, raw values |
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
