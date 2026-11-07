# DLA report

Run: `20261107-195106-9ded`

Status: INCOMPLETE (8 open)

Findings: 9 enumerated, 1 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 1

Residual: not measured

## Open items

- **F1.n1**: finding F1.n1 is candidate and has no disposition. Fix: `dla ledger resolve F1.n1 --as unresolved --reason time_budget --remedy "<text>"`
- **F2**: finding F2 is candidate and has no disposition. Fix: `dla ledger resolve F2 --as unresolved --reason time_budget --remedy "<text>"`
- **F3**: finding F3 is candidate and has no disposition. Fix: `dla ledger resolve F3 --as unresolved --reason time_budget --remedy "<text>"`
- **F4**: finding F4 is candidate and has no disposition. Fix: `dla ledger resolve F4 --as unresolved --reason time_budget --remedy "<text>"`
- **F5**: finding F5 is candidate and has no disposition. Fix: `dla ledger resolve F5 --as unresolved --reason time_budget --remedy "<text>"`
- **F6**: finding F6 is candidate and has no disposition. Fix: `dla ledger resolve F6 --as unresolved --reason time_budget --remedy "<text>"`
- **F7**: finding F7 is candidate and has no disposition. Fix: `dla ledger resolve F7 --as unresolved --reason time_budget --remedy "<text>"`
- **F8**: finding F8 is candidate and has no disposition. Fix: `dla ledger resolve F8 --as unresolved --reason time_budget --remedy "<text>"`

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Site: `src/gateway/RequestContext.java:28`
- Writers: `src/gateway/RequestContext.java:28`
- Annotation: chain head growth; only previous.receivedAtNanos and previous.path are needed
- Sibling hypotheses: F1.n1
- Reason: tool_missing
- Remedy: Install a dla release with ledger test/confirm, harness, sample, trend and gate, then confirm F1 (RequestContext.java:28) with a slope-gated run and a refuted N-WARM sibling
- Supporting evidence: E5

### F1.n1: Hypothesis N-WARM (UNCLASSIFIED)

- State: candidate
- Evidence classes: static
- Sibling hypothesis of: F1
- Hypothesis: N-WARM
- Refuted if: post-GC heap plateaus as request count rises
- Supporting evidence: E5

### F2: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: container-field
- Owner: `gateway.Gateway.routes`
- Region: heap
- Site: `src/gateway/Gateway.java:9`
- Writers: `src/gateway/Gateway.java:14`, `src/gateway/Gateway.java:15`, `src/gateway/Gateway.java:16`
- Annotation: heuristic name match over the project's sources
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: closure/continuation
- Owner: `gateway.Gateway.auditSink`
- Region: heap
- Site: `src/gateway/Gateway.java:10`
- Writers: `src/gateway/Gateway.java:17`
- Annotation: heuristic name match over the project's sources
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: container-field
- Owner: `gateway.Ledger.balances`
- Region: heap
- Site: `src/gateway/Ledger.java:12`
- Writers: `src/gateway/Ledger.java:18`, `src/gateway/Ledger.java:36`, `src/gateway/Ledger.java:37`
- Annotation: heuristic name match over the project's sources
- Supporting evidence: E6

### F5: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: container-field
- Owner: `gateway.Ledger.history`
- Region: heap
- Site: `src/gateway/Ledger.java:13`
- Writers: `src/gateway/Ledger.java:19`
- Annotation: heuristic name match over the project's sources
- Supporting evidence: E6

### F6: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: static/global
- Owner: `gateway.RequestContext.latest`
- Region: heap
- Site: `src/gateway/RequestContext.java:5`
- Writers: `src/gateway/RequestContext.java:28`
- Annotation: heuristic name match over the project's sources
- Annotation: static latest overwritten per request; never cleared
- Supporting evidence: E6

### F7: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: container-field
- Owner: `gateway.RequestContext.body`
- Region: heap
- Site: `src/gateway/RequestContext.java:11`
- Writers: `src/gateway/RequestContext.java:20`
- Annotation: heuristic name match over the project's sources
- Supporting evidence: E6

### F8: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: container-field
- Owner: `gateway.RequestContext.previous`
- Region: heap
- Site: `src/gateway/RequestContext.java:13`
- Writers: `src/gateway/RequestContext.java:22`, `src/gateway/RequestContext.java:20`
- Annotation: heuristic name match over the project's sources
- Annotation: each context keeps previous; chain from latest reaches every context ever created; no remover
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
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | Java gateway in src/gateway; symptom: heap grows with requests; consent by runner; budget 60m |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | Java gateway in src/gateway; symptom: heap grows with requests; consent by runner; budget 60m |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | Java gateway in src/gateway; symptom: heap grows with requests; consent by runner; budget 60m |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | Java gateway in src/gateway; symptom: heap grows with requests; consent by runner; budget 60m |
| `intake.build` | Does the project build locally? | answered | E1 | Java gateway in src/gateway; symptom: heap grows with requests; consent by runner; budget 60m |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | Java gateway in src/gateway; symptom: heap grows with requests; consent by runner; budget 60m |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | Java gateway in src/gateway; symptom: heap grows with requests; consent by runner; budget 60m |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | Java gateway in src/gateway; symptom: heap grows with requests; consent by runner; budget 60m |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | — |
| E2 | run-config | static | dla 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | probe/probe.json |
| E4 | static-hit | static | spotbugs unknown | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | sweep/spotbugs.json |
| E5 | static-hit | static | dla 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | sweep/dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | census/jvm.json |
| E7 | workload | static | dla 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | workload/workloads.json |
