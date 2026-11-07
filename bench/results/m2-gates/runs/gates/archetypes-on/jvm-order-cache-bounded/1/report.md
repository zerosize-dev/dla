# DLA report

Run: `20261107-194421-3549`

Status: COMPLETE

Findings: 5 enumerated, 5 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 5

Residual: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.Catalog.PRODUCTS`
- Region: heap
- Site: `src/shop/Catalog.java:9`
- Reason: tool_missing
- Remedy: Constant/immutable holder, no writers; manual review shows it cannot grow. Needs dla release with ledger test/refute to record a refutation.
- Supporting evidence: E7

### F2: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.Main.CUSTOMERS`
- Region: heap
- Site: `src/shop/Main.java:5`
- Reason: tool_missing
- Remedy: Constant/immutable holder, no writers; manual review shows it cannot grow. Needs dla release with ledger test/refute to record a refutation.
- Supporting evidence: E7

### F3: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.Order.lines`
- Region: heap
- Site: `src/shop/Order.java:6`
- Reason: tool_missing
- Remedy: Constant/immutable holder, no writers; manual review shows it cannot grow. Needs dla release with ledger test/refute to record a refutation.
- Supporting evidence: E7

### F4: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.OrderCache.ORDERS_BY_ID`
- Region: heap
- Site: `src/shop/OrderCache.java:10`
- Writers: `src/shop/OrderCache.java:24`
- Removers: `src/shop/OrderCache.java:33`, `src/shop/OrderCache.java:15`
- Annotation: heuristic name match over the project's sources
- Annotation: removeEldestEntry evicts when size()>1000 (access-ordered LRU); put at :24 is bounded. Measured: 1000 entries, ~2 MB flat from 20k to 400k requests (also under -Xmx64m).
- Sibling hypotheses: F4.n1
- Reason: tool_missing
- Remedy: Manual run shows bounded cache and flat heap; installed dla lacks gate/harness/sample/trend/ledger test|refute|confirm, so refutation cannot be recorded. Install a dla release with those commands and rerun.
- Supporting evidence: E7

### F4.n1: Hypothesis N-INTENT (UNCLASSIFIED)

- State: unresolved(tool_missing)
- Evidence classes: static
- Sibling hypothesis of: F4
- Hypothesis: N-INTENT
- Refuted if: heap in use after System.gc() keeps rising across checkpoints with cache size pinned at 1000
- Reason: tool_missing
- Remedy: Manual run shows bounded cache and flat heap; installed dla lacks gate/harness/sample/trend/ledger test|refute|confirm, so refutation cannot be recorded. Install a dla release with those commands and rerun.
- Supporting evidence: E7

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
- jvm: `src/shop/OrderCache.java::OrderCache.removeEldestEntry`: no_harness

### Evidence limitations and unknowns

- E5: partial coverage
- E5: limitation: no auxiliary classpath: library calls are not modelled
- E7: partial coverage
- E7: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E7: limitation: writers and removers are textual name matches
- E8: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E8: limitation: workloads were discovered by reading files; none was run

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | Order service JVM (src/shop), OrderCache static LRU map |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | Whole repo, heap growth in replay |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | Live set grows after each batch; pod restarted daily |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | jvm (javac sources) |
| `intake.build` | Does the project build locally? | answered | E1 | build.sh javac + unit tests |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | Main replay driver via build.sh run N |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | Runner consent tiers 1-4, raw values (E2) |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 60m |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | — |
| E2 | run-config | static | dla 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | probe/probe.json |
| E4 | probe | static | dla-probe 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | probe/probe.json |
| E5 | static-hit | static | spotbugs unknown | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | sweep/spotbugs.json |
| E6 | static-hit | static | dla 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | sweep/dla.json |
| E7 | census-holder | static | dla-census 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | census/jvm.json |
| E8 | workload | static | dla 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | workload/workloads.json |
