# DLA report

Run: `20261107-195243-b715`

Status: COMPLETE

Findings: 2 enumerated, 2 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 2

Residual: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(build_fails)
- Evidence classes: static
- Holder: static/global
- Owner: `billing.InFlightCharges.AWAITING`
- Region: heap
- Site: `src/main/java/billing/InFlightCharges.java:8`
- Writers: `src/main/java/billing/InFlightCharges.java:13`, `src/main/java/billing/ChargeService.java:23`
- Removers: `src/main/java/billing/InFlightCharges.java:17`, `src/main/java/billing/ChargeListener.java:20`
- Annotation: heuristic name match over the project's sources
- Annotation: Only onPosted calls InFlightCharges.complete; onRejected (l.25) and onExpired (l.30) never remove the entry
- Reason: build_fails
- Remedy: Build needs private repo maven.billing.internal.example for internal.billing:ledger-client; run Main with a stub/real ledger that issues rejections/expiries, sample heap histogram for PendingCharge/ChargeRequest, then confirm. Repair class: remove-entry in onRejected/onExpired (release-exceptional).
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: unresolved(build_fails)
- Evidence classes: static
- Site: `src/main/java/billing/InFlightCharges.java:8`
- Hypothesis: N-WARM
- Refuted if: in-flight count plateaus/returns to 0 after ledger.drain() when all verdicts are onPosted
- Reason: build_fails
- Remedy: Run workload where all verdicts are onPosted and check whether inFlight() returns to 0 after drain; needs a buildable project.
- Supporting evidence: E6

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

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | charge gateway JVM service (billing.*) |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | src/main/java/billing |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | heap climbs during day, drops only on restart |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | jvm (maven, java 21) |
| `intake.build` | Does the project build locally? | answered | E1 | maven; depends on private internal.billing:ledger-client |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | billing.Main load driver |
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
