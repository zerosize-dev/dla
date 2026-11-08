# DLA report

Run: `20261108-131855-6d1e`

Status: COMPLETE

Findings: 1 enumerated, 1 dispositioned; confirmed 0, strong 0, refuted 0, suspected 1, unresolved 0

Residual: not measured

Coverage: jvm no iterations, 0 of 15 entry points exercised

## Findings

### F1: **UNCLASSIFIED**

- State: suspected_static_only(build_fails)
- Evidence classes: context, static
- Holder: static/global
- Owner: `billing.InFlightCharges.AWAITING`
- Region: heap
- Site: `src/main/java/billing/InFlightCharges.java:8`
- Writers: `src/main/java/billing/InFlightCharges.java:13`, `src/main/java/billing/ChargeService.java:23`
- Removers: `src/main/java/billing/InFlightCharges.java:17`, `src/main/java/billing/ChargeListener.java:20`
- Annotation: heuristic name match over the project's sources
- Annotation: AWAITING map: register on every charge; complete() only called from onPosted. onRejected/onExpired never remove -> entries (with PendingCharge+request PII) retained forever
- Reason: build_fails
- Remedy: Make internal.billing:ledger-client 3.2.1 resolvable (access to maven.billing.internal.example or a stub), then dla harness run H1 and alloc-test; fix class: remove AWAITING entry on onRejected/onExpired as well
- Supporting evidence: E6, E8

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

- jvm: `src/main/java/billing/ChargeListener.java::ChargeListener.onExpired`: build_fails
- jvm: `src/main/java/billing/ChargeListener.java::ChargeListener.onPosted`: build_fails
- jvm: `src/main/java/billing/ChargeListener.java::ChargeListener.onRejected`: build_fails
- jvm: `src/main/java/billing/ChargeService.java::ChargeService.charge`: build_fails
- jvm: `src/main/java/billing/ChargeService.java::ChargeService.inFlight`: build_fails
- jvm: `src/main/java/billing/CustomerNotifier.java::CustomerNotifier.declined`: build_fails
- jvm: `src/main/java/billing/CustomerNotifier.java::CustomerNotifier.delayed`: build_fails
- jvm: `src/main/java/billing/CustomerNotifier.java::CustomerNotifier.paid`: build_fails
- jvm: `src/main/java/billing/CustomerNotifier.java::CustomerNotifier.paidCount`: build_fails
- jvm: `src/main/java/billing/CustomerNotifier.java::CustomerNotifier.summary`: build_fails
- jvm: `src/main/java/billing/InFlightCharges.java::InFlightCharges.complete`: build_fails
- jvm: `src/main/java/billing/InFlightCharges.java::InFlightCharges.count`: build_fails
- jvm: `src/main/java/billing/InFlightCharges.java::InFlightCharges.oldestAgeMillis`: build_fails
- jvm: `src/main/java/billing/InFlightCharges.java::InFlightCharges.register`: build_fails
- jvm: `src/main/java/billing/PendingCharge.java::PendingCharge.ageMillis`: build_fails

### Evidence limitations and unknowns

- E6: partial coverage
- E6: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E6: limitation: writers and removers are textual name matches
- E7: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E7: limitation: workloads were discovered by reading files; none was run
- E8: limitation: harness written; nothing was run (dla harness run measures it)
- E8: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E8: limitation: note: discovery marked the workload not exercisable (build_fails): the harness build is likely to fail

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | Repository charge-gateway (JVM, Maven); symptom is heap growth in running service |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | Whole repo, src/main/java/billing |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | Heap climbs over working day, reclaimed only on restart |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E3 | jvm only |
| `intake.build` | Does the project build locally? | answered | E4 | mvn build fails: private dependency internal.billing:ledger-client unresolvable |
| `intake.workload` | What can exercise the suspected path? | answered | E3 | Main load driver; junit test; needs ledger service |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 raw values |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 60m |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 4667c4afdfe44f885669eee8ce8966a58a34a8ca | — |
| E2 | run-config | static | dla 0.1.0 | repo 4667c4afdfe44f885669eee8ce8966a58a34a8ca | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 4667c4afdfe44f885669eee8ce8966a58a34a8ca | probe/probe.json |
| E4 | probe | static | dla-probe 0.1.0 | repo 4667c4afdfe44f885669eee8ce8966a58a34a8ca | probe/probe.json |
| E5 | static-hit | static | dla 0.1.0 | repo 4667c4afdfe44f885669eee8ce8966a58a34a8ca | sweep/dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo 4667c4afdfe44f885669eee8ce8966a58a34a8ca | census/jvm.json |
| E7 | workload | static | dla 0.1.0 | repo 4667c4afdfe44f885669eee8ce8966a58a34a8ca | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo 4667c4afdfe44f885669eee8ce8966a58a34a8ca | harness/H1/harness.json |
