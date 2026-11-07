# DLA report

Run: `20261107-194048-d9ee`

Status: COMPLETE

Findings: 1 enumerated, 1 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 1

Residual: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: static/global
- Owner: `billing.InFlightCharges.AWAITING`
- Region: heap
- Site: `src/main/java/billing/InFlightCharges.java:8`
- Writers: `src/main/java/billing/InFlightCharges.java:13`, `src/main/java/billing/InFlightCharges.java:12`, `src/main/java/billing/ChargeService.java:25`
- Removers: `src/main/java/billing/InFlightCharges.java:17`, `src/main/java/billing/InFlightCharges.java:16`, `src/main/java/billing/ChargeListener.java:20`
- Annotation: heuristic name match over the project's sources
- Annotation: Static ConcurrentHashMap AWAITING. register() on every charge; complete() only called from ChargeListener.onPosted. onRejected (line 26) and onExpired (line 31) never remove the entry, so every rejected/expired charge keeps its PendingCharge (with ChargeRequest incl. customer email) forever. Static analysis only; not measured: ledger client lib internal.billing.ledger is not available locally and no harness command exists in this dla release.
- Reason: tool_missing
- Remedy: Install a dla release with harness/sample/trend/ledger test+confirm; run the Main load driver with a ledger stub that rejects/expires some charges and check InFlightCharges.count() growth; add N-WARM/N-SCALE siblings. Repair class: release the map entry on every terminal callback (onRejected, onExpired), not just onPosted.
- Supporting evidence: E5

## Ruled out

None.

## Blind spots and unknowns

### Coverage gaps

- jvm/codeql: missing (codeql is not on PATH or in the tools directory). Install hint: dla install codeql
- jvm/errorprone: missing (Error Prone jars not found in the tools directory (or DLA_ERRORPRONE_HOME)). Install hint: dla install errorprone
- jvm/sonar: missing (sonar-scanner is not installed and Sonar rules need a SonarQube or SonarCloud server). Install hint: install sonar-scanner, run a SonarQube or SonarCloud server and set SONAR_HOST_URL and SONAR_TOKEN (dla install sonar prints the steps)
- jvm/spotbugs: blocked (no class files: run dla probe --build)

### Runtime blind spots

None recorded.

### Entry points not exercisable

- jvm: `src/main/java/billing/ChargeListener.java::ChargeListener.onExpired`: no_harness
- jvm: `src/main/java/billing/ChargeListener.java::ChargeListener.onPosted`: no_harness
- jvm: `src/main/java/billing/ChargeListener.java::ChargeListener.onRejected`: no_harness
- jvm: `src/main/java/billing/InFlightCharges.java::InFlightCharges.complete`: no_harness
- jvm: `src/main/java/billing/InFlightCharges.java::InFlightCharges.oldestAgeMillis`: no_harness
- jvm: `src/main/java/billing/PendingCharge.java::PendingCharge.ageMillis`: no_harness

### Evidence limitations and unknowns

- E5: partial coverage
- E5: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E5: limitation: writers and removers are textual name matches
- E6: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E6: limitation: workloads were discovered by reading files; none was run

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | see repo and task statement: charge gateway (billing, JVM/Java, Maven), heap climbs during day, only restart frees it |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | see repo and task statement: charge gateway (billing, JVM/Java, Maven), heap climbs during day, only restart frees it |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | see repo and task statement: charge gateway (billing, JVM/Java, Maven), heap climbs during day, only restart frees it |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | see repo and task statement: charge gateway (billing, JVM/Java, Maven), heap climbs during day, only restart frees it |
| `intake.build` | Does the project build locally? | answered | E1 | see repo and task statement: charge gateway (billing, JVM/Java, Maven), heap climbs during day, only restart frees it |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | see repo and task statement: charge gateway (billing, JVM/Java, Maven), heap climbs during day, only restart frees it |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4, raw values |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | see repo and task statement: charge gateway (billing, JVM/Java, Maven), heap climbs during day, only restart frees it |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 4667c4afdfe44f885669eee8ce8966a58a34a8ca | — |
| E2 | run-config | static | dla 0.1.0 | repo 4667c4afdfe44f885669eee8ce8966a58a34a8ca | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 4667c4afdfe44f885669eee8ce8966a58a34a8ca | probe/probe.json |
| E4 | static-hit | static | dla 0.1.0 | repo 4667c4afdfe44f885669eee8ce8966a58a34a8ca | sweep/dla.json |
| E5 | census-holder | static | dla-census 0.1.0 | repo 4667c4afdfe44f885669eee8ce8966a58a34a8ca | census/jvm.json |
| E6 | workload | static | dla 0.1.0 | repo 4667c4afdfe44f885669eee8ce8966a58a34a8ca | workload/workloads.json |
