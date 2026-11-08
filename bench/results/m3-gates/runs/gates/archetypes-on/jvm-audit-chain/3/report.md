# DLA report

Run: `20261108-131012-a9db`

Status: INCOMPLETE (4 open)

Findings: 13 enumerated, 9 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 7

Residual: not measured

Coverage: jvm 8880 iterations, 19 of 19 entry points exercised

## Open items

- **F2.n1**: finding F2.n1 is candidate and has no disposition. Fix: `dla ledger resolve F2.n1 --as unresolved --reason time_budget --remedy "<text>"`
- **F3.n1**: finding F3.n1 is candidate and has no disposition. Fix: `dla ledger resolve F3.n1 --as unresolved --reason time_budget --remedy "<text>"`
- **F4.n1**: finding F4.n1 is candidate and has no disposition. Fix: `dla ledger resolve F4.n1 --as unresolved --reason time_budget --remedy "<text>"`
- **F5.n1**: finding F5.n1 is candidate and has no disposition. Fix: `dla ledger resolve F5.n1 --as unresolved --reason time_budget --remedy "<text>"`

## Findings

### F6: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:gateway.RequestContext.latest` 3 count/n (steady, gate pass, E12)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `gateway.RequestContext.latest`
- Region: heap
- Shape: steady
- Site: `src/gateway/RequestContext.java:5`
- Writers: `src/gateway/RequestContext.java:28`
- Annotation: heuristic name match over the project's sources
- Annotation: static head of chain; never cleared
- Sibling hypotheses: F6.n1
- Supporting evidence: E7, E10, E12

### F1: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: static
- Holder: static/global
- Site: `src/gateway/RequestContext.java:28`
- Writers: `src/gateway/RequestContext.java:28`
- Annotation: latest assigned each request; new context links previous=latest (line 27), forming an unbounded singly-linked chain; no remover anywhere
- Reason: env_unavailable
- Remedy: same mechanism as confirmed F6 (RequestContext chain); separate confirmation not run for this duplicate
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: context, static
- Holder: container-field
- Owner: `gateway.Gateway.routes`
- Region: heap
- Site: `src/gateway/Gateway.java:9`
- Writers: `src/gateway/Gateway.java:14`, `src/gateway/Gateway.java:15`, `src/gateway/Gateway.java:16`
- Annotation: heuristic name match over the project's sources
- Sibling hypotheses: F2.n1
- Reason: env_unavailable
- Remedy: source shows bounded holder (fixed routes/accounts, history capped at 20); alloc-test only saw whole-heap growth caused by F6, so no per-holder evidence. Re-test after the F6 chain is fixed
- Supporting evidence: E7, E10

### F3: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: context, static
- Holder: closure/continuation
- Owner: `gateway.Gateway.auditSink`
- Region: heap
- Site: `src/gateway/Gateway.java:10`
- Writers: `src/gateway/Gateway.java:17`
- Annotation: heuristic name match over the project's sources
- Sibling hypotheses: F3.n1
- Reason: env_unavailable
- Remedy: source shows bounded holder (fixed routes/accounts, history capped at 20); alloc-test only saw whole-heap growth caused by F6, so no per-holder evidence. Re-test after the F6 chain is fixed
- Supporting evidence: E7, E10

### F4: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: context, static
- Holder: container-field
- Owner: `gateway.Ledger.balances`
- Region: heap
- Site: `src/gateway/Ledger.java:12`
- Writers: `src/gateway/Ledger.java:18`, `src/gateway/Ledger.java:36`, `src/gateway/Ledger.java:37`
- Annotation: heuristic name match over the project's sources
- Sibling hypotheses: F4.n1
- Reason: env_unavailable
- Remedy: source shows bounded holder (fixed routes/accounts, history capped at 20); alloc-test only saw whole-heap growth caused by F6, so no per-holder evidence. Re-test after the F6 chain is fixed
- Supporting evidence: E7, E10

### F5: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: context, static
- Holder: container-field
- Owner: `gateway.Ledger.history`
- Region: heap
- Site: `src/gateway/Ledger.java:13`
- Writers: `src/gateway/Ledger.java:19`
- Annotation: heuristic name match over the project's sources
- Sibling hypotheses: F5.n1
- Reason: env_unavailable
- Remedy: source shows bounded holder (fixed routes/accounts, history capped at 20); alloc-test only saw whole-heap growth caused by F6, so no per-holder evidence. Re-test after the F6 chain is fixed
- Supporting evidence: E7, E10

### F7: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: static
- Holder: container-field
- Owner: `gateway.RequestContext.body`
- Region: heap
- Site: `src/gateway/RequestContext.java:11`
- Writers: `src/gateway/RequestContext.java:20`
- Annotation: heuristic name match over the project's sources
- Annotation: 384-byte padded body retained via chain
- Reason: env_unavailable
- Remedy: same mechanism as confirmed F6 (RequestContext chain); separate confirmation not run for this duplicate
- Supporting evidence: E7

### F8: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: static
- Holder: container-field
- Owner: `gateway.RequestContext.previous`
- Region: heap
- Site: `src/gateway/RequestContext.java:13`
- Writers: `src/gateway/RequestContext.java:22`
- Annotation: heuristic name match over the project's sources
- Annotation: previous pointer keeps every older context (and its body) reachable; only needs previous.receivedAtNanos/path, used in Gateway.auditLine
- Reason: env_unavailable
- Remedy: same mechanism as confirmed F6 (RequestContext chain); separate confirmation not run for this duplicate
- Supporting evidence: E7

### F2.n1: Hypothesis N-INTENT: bounded by design (fixed accounts/routes/history cap) (UNCLASSIFIED)

- State: candidate
- Evidence classes: growth
- Sibling hypothesis of: F2
- Hypothesis: N-INTENT: bounded by design (fixed accounts/routes/history cap)
- Refuted if: container size grows with requests
- Supporting evidence: E12

### F3.n1: Hypothesis N-INTENT: bounded by design (fixed accounts/routes/history cap) (UNCLASSIFIED)

- State: candidate
- Evidence classes: growth
- Sibling hypothesis of: F3
- Hypothesis: N-INTENT: bounded by design (fixed accounts/routes/history cap)
- Refuted if: container size grows with requests
- Supporting evidence: E12

### F4.n1: Hypothesis N-INTENT: bounded by design (fixed accounts/routes/history cap) (UNCLASSIFIED)

- State: candidate
- Evidence classes: growth
- Sibling hypothesis of: F4
- Hypothesis: N-INTENT: bounded by design (fixed accounts/routes/history cap)
- Refuted if: container size grows with requests
- Supporting evidence: E12

### F5.n1: Hypothesis N-INTENT: bounded by design (fixed accounts/routes/history cap) (UNCLASSIFIED)

- State: candidate
- Evidence classes: growth
- Sibling hypothesis of: F5
- Hypothesis: N-INTENT: bounded by design (fixed accounts/routes/history cap)
- Refuted if: container size grows with requests
- Supporting evidence: E12

## Ruled out

- **F6.n1** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (leak) E13
  - Hypothesis: N-WARM: growth is warm-up/cache fill that plateaus
  - Refuted if: growth continues linearly over extra cycles after purge
  - Contradicting evidence: E13
  - Originally supported by: E12

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
- E9: limitation: harness written; nothing was run (dla harness run measures it)
- E9: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E9: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E10: partial coverage (9 of 9 examined)
- E10: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E10: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E11: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: points 1 to 9 come from S1; the extra points from S2
- E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E13: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- E14: partial coverage (1 of 24 examined)
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: gate thresholds are provisional (not calibrated)
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
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | local repo: Java API gateway in src/gateway |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | whole repo, single module |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | memory grows with requests served; only restart reduces |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | jvm (javac, build.sh) |
| `intake.build` | Does the project build locally? | answered | E1 | ./build.sh |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | gateway.Main <count> load driver |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4, raw values |
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
| E9 | workload | static | dla 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | harness/H1/harness.json |
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (ff87c63b23b8) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (4440069c6060) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (4440069c6060) | trend/T1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S3.json (9ef6cf471b8f) | alloc/A1.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (4440069c6060) | trend/T3.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S5.json (1d58816e4722) | alloc/A2.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S7.json (228779a2b227) | alloc/A3.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S9.json (29da4e5af876) | alloc/A4.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S11.json (5a5689a22041) | alloc/A5.json |
