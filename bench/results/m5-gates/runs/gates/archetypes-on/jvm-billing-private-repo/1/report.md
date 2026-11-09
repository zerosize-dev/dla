# DLA report

Run: `20261109-195926-04a4`

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
- Writers: `src/main/java/billing/InFlightCharges.java:13`
- Removers: `src/main/java/billing/InFlightCharges.java:17`
- Annotation: heuristic name match over the project's sources
- Annotation: InFlightCharges.AWAITING gets an entry per charge (ChargeService.java:23 register) but the only removal on a path is ChargeListener.onPosted (ChargeListener.java:21). onRejected and onExpired never call InFlightCharges.complete, so each rejected/expired charge keeps its PendingCharge (with ChargeRequest incl. customer email) in the static map forever. Missing pairing: removal on every terminal ledger outcome. Not measured: build fails offline (private ledger-client dependency).
- Reason: build_fails
- Remedy: Provide access to internal.billing:ledger-client 3.2.1 (maven.billing.internal.example) or a stub, then run dla harness gen/run, trend, alloc-test on probe:billing.InFlightCharges.AWAITING, and confirm
- Supporting evidence: E5

## Ruled out

None.

## Blind spots and unknowns

### Coverage gaps

- jvm/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- jvm/codeql: missing (codeql is not on PATH or in the tools directory). Install hint: dla install codeql
- jvm/errorprone: missing (Error Prone jars not found in the tools directory (or DLA_ERRORPRONE_HOME)). Install hint: dla install errorprone
- jvm/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)
- jvm/sonar: missing (sonar-scanner is not installed and Sonar rules need a SonarQube or SonarCloud server). Install hint: install sonar-scanner, run a SonarQube or SonarCloud server and set SONAR_HOST_URL and SONAR_TOKEN (dla install sonar prints the steps)
- jvm/spotbugs: blocked (no class files: run dla probe --build)
- jvm/valgrind: blocked (Valgrind is not run on the JVM: the virtual machine's own allocations drown the project's; use the allocator shim, jcmd NMT or `dla attach bpftrace`)

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
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | — |
| `intake.build` | Does the project build locally? | answered | E1 | — |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | — |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E1 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 4667c4afdfe44f885669eee8ce8966a58a34a8ca | — |
| E2 | run-config | static | dla 0.1.0 | repo 4667c4afdfe44f885669eee8ce8966a58a34a8ca | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 4667c4afdfe44f885669eee8ce8966a58a34a8ca | probe/probe.json |
| E4 | static-hit | static | dla 0.1.0 | repo 4667c4afdfe44f885669eee8ce8966a58a34a8ca | sweep/jvm-dla.json |
| E5 | census-holder | static | dla-census 0.1.0 | repo 4667c4afdfe44f885669eee8ce8966a58a34a8ca | census/jvm.json |
