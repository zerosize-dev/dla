# DLA report

Run: `20261107-195020-2588`

Status: COMPLETE

Findings: 4 enumerated, 4 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 4

Residual: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.Catalog.PRODUCTS`
- Region: heap
- Site: `src/shop/Catalog.java:9`
- Reason: not_exercised
- Remedy: Static/read-only holder (immutable List/array or per-order field); no growth series observed; no refute command in this dla release
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.Main.CUSTOMERS`
- Region: heap
- Site: `src/shop/Main.java:5`
- Reason: not_exercised
- Remedy: Static/read-only holder (immutable List/array or per-order field); no growth series observed; no refute command in this dla release
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.Order.lines`
- Region: heap
- Site: `src/shop/Order.java:6`
- Reason: not_exercised
- Remedy: Static/read-only holder (immutable List/array or per-order field); no growth series observed; no refute command in this dla release
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.OrderCache.ORDERS_BY_ID`
- Region: heap
- Site: `src/shop/OrderCache.java:10`
- Writers: `src/shop/OrderCache.java:24`
- Removers: `src/shop/OrderCache.java:33`, `src/shop/OrderCache.java:17`
- Annotation: heuristic name match over the project's sources
- Annotation: LinkedHashMap access-order with removeEldestEntry capping at 1000; measured flat at 1000 entries / ~2023KB over 40000 requests (3 consecutive checkpoints identical)
- Reason: tool_missing
- Remedy: Installed dla lacks harness/sample/trend/ledger refute; measured by hand: bounded at 1000, heap flat ~2023KB. Re-run with fuller dla release
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
- jvm: `src/shop/OrderCache.java::OrderCache.removeEldestEntry`: no_harness

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
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | Local repo: Java order service (src/shop), load driver Main |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | Whole repo, 7 source files |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | Live set grows after each replay batch; pod restarted ~daily |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | jvm |
| `intake.build` | Does the project build locally? | answered | E1 | ./build.sh (javac, plain) |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | ./build.sh run N replays requests via Main |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 raw values |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 60m |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | — |
| E2 | run-config | static | dla 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | probe/probe.json |
| E4 | static-hit | static | spotbugs unknown | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | sweep/spotbugs.json |
| E5 | static-hit | static | dla 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | sweep/dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | census/jvm.json |
| E7 | workload | static | dla 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | workload/workloads.json |
