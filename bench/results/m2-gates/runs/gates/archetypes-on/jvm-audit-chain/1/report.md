# DLA report

Run: `20261107-193916-750e`

Status: COMPLETE

Findings: 8 enumerated, 8 dispositioned; confirmed 0, strong 0, refuted 0, suspected 4, unresolved 4

Residual: not measured

## Findings

### F2: **UNCLASSIFIED**

- State: suspected_static_only(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `gateway.Gateway.routes`
- Region: heap
- Site: `src/gateway/Gateway.java:9`
- Writers: `src/gateway/Gateway.java:14`, `src/gateway/Gateway.java:15`, `src/gateway/Gateway.java:16`
- Annotation: heuristic name match over the project's sources
- Annotation: bounded: fixed 64 accounts / history capped at 20; routes fixed; auditSink is a lambda with counters only. No per-request growth observed
- Reason: tool_missing
- Remedy: static review only; bounded container, N/A for growth
- Supporting evidence: E7

### F3: **UNCLASSIFIED**

- State: suspected_static_only(tool_missing)
- Evidence classes: static
- Holder: closure/continuation
- Owner: `gateway.Gateway.auditSink`
- Region: heap
- Site: `src/gateway/Gateway.java:10`
- Writers: `src/gateway/Gateway.java:17`
- Annotation: heuristic name match over the project's sources
- Annotation: bounded: fixed 64 accounts / history capped at 20; routes fixed; auditSink is a lambda with counters only. No per-request growth observed
- Reason: tool_missing
- Remedy: static review only; bounded container, N/A for growth
- Supporting evidence: E7

### F4: **UNCLASSIFIED**

- State: suspected_static_only(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `gateway.Ledger.balances`
- Region: heap
- Site: `src/gateway/Ledger.java:12`
- Writers: `src/gateway/Ledger.java:18`, `src/gateway/Ledger.java:36`, `src/gateway/Ledger.java:37`
- Annotation: heuristic name match over the project's sources
- Annotation: bounded: fixed 64 accounts / history capped at 20; routes fixed; auditSink is a lambda with counters only. No per-request growth observed
- Reason: tool_missing
- Remedy: static review only; bounded container, N/A for growth
- Supporting evidence: E7

### F5: **UNCLASSIFIED**

- State: suspected_static_only(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `gateway.Ledger.history`
- Region: heap
- Site: `src/gateway/Ledger.java:13`
- Writers: `src/gateway/Ledger.java:19`
- Annotation: heuristic name match over the project's sources
- Annotation: bounded: fixed 64 accounts / history capped at 20; routes fixed; auditSink is a lambda with counters only. No per-request growth observed
- Reason: tool_missing
- Remedy: static review only; bounded container, N/A for growth
- Supporting evidence: E7

### F1: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Site: `src/gateway/RequestContext.java:28`
- Writers: `src/gateway/RequestContext.java:29`
- Annotation: open() links each new context to 'latest' via previous; chain never truncated
- Reason: tool_missing
- Remedy: installed dla lacks gate/harness/sample/trend/ledger test|confirm; run 'dla ledger test/confirm F1' after upgrading, with heap-graph summarize of RequestContext.latest
- Supporting evidence: E6

### F6: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: static/global
- Owner: `gateway.RequestContext.latest`
- Region: heap
- Site: `src/gateway/RequestContext.java:5`
- Writers: `src/gateway/RequestContext.java:28`, `src/gateway/RequestContext.java:29`
- Annotation: heuristic name match over the project's sources
- Annotation: static latest is a GC root; reassigned every request, no remover
- Reason: tool_missing
- Remedy: installed dla lacks gate/harness/sample/trend/ledger test|confirm; run 'dla ledger test/confirm F6' after upgrading, with heap-graph summarize of RequestContext.latest
- Supporting evidence: E7

### F7: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `gateway.RequestContext.body`
- Region: heap
- Site: `src/gateway/RequestContext.java:11`
- Writers: `src/gateway/RequestContext.java:20`
- Annotation: heuristic name match over the project's sources
- Annotation: retained transitively through the F8 chain; per-request body byte[] of ~384B
- Reason: tool_missing
- Remedy: installed dla lacks gate/harness/sample/trend/ledger test|confirm; run 'dla ledger test/confirm F7' after upgrading, with heap-graph summarize of RequestContext.latest
- Supporting evidence: E7

### F8: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `gateway.RequestContext.previous`
- Region: heap
- Site: `src/gateway/RequestContext.java:13`
- Writers: `src/gateway/RequestContext.java:22`, `src/gateway/RequestContext.java:28`
- Annotation: heuristic name match over the project's sources
- Annotation: previous pointer keeps every older context (and its 384-byte body) reachable from latest; no remover. Only use is previous.receivedAtNanos/path in the next request
- Reason: tool_missing
- Remedy: installed dla lacks gate/harness/sample/trend/ledger test|confirm; run 'dla ledger test/confirm F8' after upgrading, with heap-graph summarize of RequestContext.latest
- Supporting evidence: E7

## Ruled out

None.

## Blind spots and unknowns

### Coverage gaps

- jvm/codeql: missing (codeql is not on PATH or in the tools directory). Install hint: dla install codeql
- jvm/errorprone: missing (Error Prone jars not found in the tools directory (or DLA_ERRORPRONE_HOME)). Install hint: dla install errorprone
- jvm/sonar: missing (sonar-scanner is not installed and Sonar rules need a SonarQube or SonarCloud server). Install hint: install sonar-scanner, run a SonarQube or SonarCloud server and set SONAR_HOST_URL and SONAR_TOKEN (dla install sonar prints the steps)
- python/codeql: missing (install failed: /tmp/dla-bench/m2-gates/gates/archetypes-on/jvm-audit-chain/1/config/plugins/data/dla/tools/codeql: Read-only file system (os error 30)). Install hint: dla install codeql

### Runtime blind spots

None recorded.

### Entry points not exercisable

- jvm: `src/gateway/Handlers.java::Handlers.balance`: no_harness
- jvm: `src/gateway/Handlers.java::Handlers.statement`: no_harness
- jvm: `src/gateway/Handlers.java::Handlers.transfer`: no_harness
- jvm: `src/gateway/Ledger.java::Ledger.balance`: no_harness
- jvm: `src/gateway/Ledger.java::Ledger.statement`: no_harness
- jvm: `src/gateway/Ledger.java::Ledger.transfer`: no_harness
- jvm: `src/gateway/RequestContext.java::RequestContext.microsSincePrevious`: no_harness
- jvm: `src/gateway/RequestContext.java::RequestContext.previousPath`: no_harness
- jvm: `src/gateway/RequestContext.java::RequestContext.sequence`: no_harness
- jvm: `src/gateway/Response.java::Response.ok`: no_harness

### Evidence limitations and unknowns

- E5: partial coverage
- E5: limitation: no auxiliary classpath: library calls are not modelled
- E7: partial coverage
- E7: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E7: limitation: writers and removers are textual name matches
- E8: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E8: limitation: workloads were discovered by reading files; none was run
- E9: partial coverage
- E9: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E9: limitation: writers and removers are textual name matches

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | Java API gateway in src/gateway |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | whole repo, jvm only |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | heap grows linearly with requests served; restart only fix |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E4 | jvm only |
| `intake.build` | Does the project build locally? | answered | E4 | build.sh ok |
| `intake.workload` | What can exercise the suspected path? | answered | E8 | build.sh run [count] |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 raw values |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 60m |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | — |
| E2 | run-config | static | dla 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | probe/probe.json |
| E4 | probe | static | dla-probe 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | probe/probe.json |
| E5 | static-hit | static | spotbugs unknown | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | sweep/spotbugs.json |
| E6 | static-hit | static | dla 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | sweep/dla.json |
| E7 | census-holder | static | dla-census 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | census/jvm.json |
| E8 | workload | static | dla 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | workload/workloads.json |
| E9 | census-holder | static | dla-census 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | census/jvm.json |
