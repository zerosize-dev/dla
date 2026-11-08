# DLA report

Run: `20261108-201127-31bf`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 1 enumerated, 1 dispositioned; confirmed 0, strong 0, refuted 0, suspected 1, unresolved 0

Residual: not measured

Coverage: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: suspected_static_only(build_fails)
- Evidence classes: static
- Holder: static/global
- Owner: `billing.InFlightCharges.AWAITING`
- Region: heap
- Site: `src/main/java/billing/InFlightCharges.java:8`
- Writers: `src/main/java/billing/InFlightCharges.java:13`, `src/main/java/billing/ChargeService.java:23`
- Removers: `src/main/java/billing/InFlightCharges.java:17`, `src/main/java/billing/ChargeListener.java:22`
- Annotation: heuristic name match over the project's sources
- Annotation: InFlightCharges.AWAITING (static ConcurrentHashMap) gets a PendingCharge per charge in ChargeService.charge; the only remove is in ChargeListener.onPosted. onRejected and onExpired never call InFlightCharges.complete, so declined/expired charges stay forever, each retaining PendingCharge -> ChargeRequest (email, invoice, account strings). Missing pairing: release the entry on every terminal outcome (rejected, expired), not just posted. Not exercised: build fails offline and ledger-client is a private artifact.
- Reason: build_fails
- Remedy: Make internal.billing:ledger-client:3.2.1 and Maven plugins resolvable, then run dla probe --build, dla harness gen/run, alloc-test and confirm; or supply 3+ heap dumps of the live gateway for the artifact path
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

None recorded.

### Evidence limitations and unknowns

- E5: partial coverage
- E5: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E5: limitation: writers and removers are textual name matches

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | — |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | — |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | — |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E3 | — |
| `intake.build` | Does the project build locally? | unresolved | — | build_fails: Provide network/mirror access to Maven Central and the private internal.billing:ledger-client:3.2.1 artifact |
| `intake.workload` | What can exercise the suspected path? | unresolved | — | no_harness: Workload Main needs the private ledger-client and a ledger host; build fails |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 4667c4afdfe44f885669eee8ce8966a58a34a8ca | — |
| E2 | run-config | static | dla 0.1.0 | repo 4667c4afdfe44f885669eee8ce8966a58a34a8ca | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 4667c4afdfe44f885669eee8ce8966a58a34a8ca | probe/probe.json |
| E4 | static-hit | static | dla 0.1.0 | repo 4667c4afdfe44f885669eee8ce8966a58a34a8ca | sweep/jvm-dla.json |
| E5 | census-holder | static | dla-census 0.1.0 | repo 4667c4afdfe44f885669eee8ce8966a58a34a8ca | census/jvm.json |
