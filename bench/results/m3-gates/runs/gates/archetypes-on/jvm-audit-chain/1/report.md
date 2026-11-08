# DLA report

Run: `20261108-122938-9de6`

Status: COMPLETE

Findings: 9 enumerated, 9 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 7

Residual: not measured

Coverage: jvm 8880 iterations, 19 of 19 entry points exercised

## Findings

### F6: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:gateway.RequestContext.latest` 3 count/n (steady, gate pass, E13)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `gateway.RequestContext.latest`
- Region: heap
- Shape: steady
- Site: `src/gateway/RequestContext.java:5`
- Writers: `src/gateway/RequestContext.java:28`
- Annotation: heuristic name match over the project's sources
- Annotation: static latest never cleared; each context links previous (line 22), chaining all requests incl. body byte[]; no remover
- Sibling hypotheses: F6.n1
- Supporting evidence: E7, E11, E13

### F1: **UNCLASSIFIED**

- State: unresolved(no_harness)
- Evidence classes: static
- Holder: static/global
- Owner: `gateway.RequestContext.latest`
- Region: heap
- Site: `src/gateway/RequestContext.java:28`
- Annotation: Same chain as F6 (detector jvm-chain-head-growth)
- Reason: no_harness
- Remedy: Same chain as confirmed F6 (detector hit at same site); no separate evidence needed.
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: unresolved(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `gateway.Gateway.routes`
- Region: heap
- Site: `src/gateway/Gateway.java:9`
- Writers: `src/gateway/Gateway.java:14`, `src/gateway/Gateway.java:15`, `src/gateway/Gateway.java:16`
- Annotation: heuristic name match over the project's sources
- Reason: no_harness
- Remedy: Source shows bounded/fixed-key holder; alloc-test column was whole-heap jvm.heap_used, confounded by F6. Re-test with a per-holder probe after F6 is fixed.
- Supporting evidence: E7

### F3: **UNCLASSIFIED**

- State: unresolved(no_harness)
- Evidence classes: static
- Holder: closure/continuation
- Owner: `gateway.Gateway.auditSink`
- Region: heap
- Site: `src/gateway/Gateway.java:10`
- Writers: `src/gateway/Gateway.java:17`
- Annotation: heuristic name match over the project's sources
- Reason: no_harness
- Remedy: Source shows bounded/fixed-key holder; alloc-test column was whole-heap jvm.heap_used, confounded by F6. Re-test with a per-holder probe after F6 is fixed.
- Supporting evidence: E7

### F4: **UNCLASSIFIED**

- State: unresolved(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `gateway.Ledger.balances`
- Region: heap
- Site: `src/gateway/Ledger.java:12`
- Writers: `src/gateway/Ledger.java:18`, `src/gateway/Ledger.java:36`, `src/gateway/Ledger.java:37`
- Annotation: heuristic name match over the project's sources
- Reason: no_harness
- Remedy: Source shows bounded/fixed-key holder; alloc-test column was whole-heap jvm.heap_used, confounded by F6. Re-test with a per-holder probe after F6 is fixed.
- Supporting evidence: E7

### F5: **UNCLASSIFIED**

- State: unresolved(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `gateway.Ledger.history`
- Region: heap
- Site: `src/gateway/Ledger.java:13`
- Writers: `src/gateway/Ledger.java:19`, `src/gateway/RequestContext.java:28`
- Annotation: heuristic name match over the project's sources
- Annotation: latest static never cleared; previous link chains every request (line 22/28); no remover
- Annotation: CORRECTION: previous annotation on F5 was misapplied; F5 (Ledger.history) is bounded to 20 entries/account (Ledger.java:53-55 removeFirst). Chain annotation belongs to RequestContext findings.
- Reason: no_harness
- Remedy: Source shows bounded/fixed-key holder; alloc-test column was whole-heap jvm.heap_used, confounded by F6. Re-test with a per-holder probe after F6 is fixed.
- Supporting evidence: E7

### F7: **UNCLASSIFIED**

- State: unresolved(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `gateway.RequestContext.body`
- Region: heap
- Site: `src/gateway/RequestContext.java:11`
- Writers: `src/gateway/RequestContext.java:20`
- Annotation: heuristic name match over the project's sources
- Reason: no_harness
- Remedy: Field of the chain confirmed as F6; body bytes growth (probe ...latest.bytes ~497 B/request) is part of F6.
- Supporting evidence: E7

### F8: **UNCLASSIFIED**

- State: unresolved(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `gateway.RequestContext.previous`
- Region: heap
- Site: `src/gateway/RequestContext.java:13`
- Writers: `src/gateway/RequestContext.java:22`
- Annotation: heuristic name match over the project's sources
- Reason: no_harness
- Remedy: The 'previous' link is the mechanism of F6; not a separate holder.
- Supporting evidence: E7

## Ruled out

- **F6.n1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (leak) E14
  - Hypothesis: N-WARM: growth is a warm-up cache that plateaus
  - Refuted if: growth continues linearly over extra cycles after purge
  - Contradicting evidence: E14
  - Originally supported by: E13

## Blind spots and unknowns

### Coverage gaps

- jvm/codeql: missing (codeql is not on PATH or in the tools directory). Install hint: dla install codeql
- jvm/errorprone: missing (Error Prone jars not found in the tools directory (or DLA_ERRORPRONE_HOME)). Install hint: dla install errorprone
- jvm/sonar: missing (sonar-scanner is not installed and Sonar rules need a SonarQube or SonarCloud server). Install hint: install sonar-scanner, run a SonarQube or SonarCloud server and set SONAR_HOST_URL and SONAR_TOKEN (dla install sonar prints the steps)

### Runtime blind spots

None recorded.

### Entry points not exercisable

None recorded.

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
- E10: limitation: harness written; nothing was run (dla harness run measures it)
- E10: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E10: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E11: partial coverage (9 of 9 examined)
- E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E11: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: points 1 to 9 come from S1; the extra points from S2
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: points 1 to 9 come from S1; the extra points from S4
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: points 1 to 9 come from S1; the extra points from S6
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: points 1 to 9 come from S1; the extra points from S8
- E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E17: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E17: limitation: gate and plateau thresholds are provisional (not calibrated)
- E18: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: points 1 to 9 come from S1; the extra points from S10
- E18: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E18: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E18: limitation: gate and plateau thresholds are provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | Unattended run: from task prompt/repo (gateway, JVM, request-count-proportional growth); consent E2 |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | Unattended run: from task prompt/repo (gateway, JVM, request-count-proportional growth); consent E2 |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | Unattended run: from task prompt/repo (gateway, JVM, request-count-proportional growth); consent E2 |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | Unattended run: from task prompt/repo (gateway, JVM, request-count-proportional growth); consent E2 |
| `intake.build` | Does the project build locally? | answered | E1 | Unattended run: from task prompt/repo (gateway, JVM, request-count-proportional growth); consent E2 |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | Unattended run: from task prompt/repo (gateway, JVM, request-count-proportional growth); consent E2 |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E1 | Unattended run: from task prompt/repo (gateway, JVM, request-count-proportional growth); consent E2 |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | Unattended run: from task prompt/repo (gateway, JVM, request-count-proportional growth); consent E2 |

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
| E10 | workload | static | dla 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | harness/H1/harness.json |
| E11 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (ea015f43023a) | harness/H1/run-S1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S1.json (e83f0d02f41c) | trend/T1.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (e83f0d02f41c) | trend/T1.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S3.json (67b417fc6eef) | alloc/A1.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S5.json (83267794436a) | alloc/A2.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S7.json (cc94b294525a) | alloc/A3.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S9.json (364befbbbda7) | alloc/A4.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S11.json (fa7d783de38b) | alloc/A5.json |
