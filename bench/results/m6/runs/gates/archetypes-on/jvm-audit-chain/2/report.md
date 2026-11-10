# DLA report

Run: `20261110-165541-0fff`

Status: INCOMPLETE (11 open)

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 18 enumerated, 14 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 12

Residual: not measured

Coverage: jvm 8880 iterations, 19 of 19 entry points exercised

## Open items

- **F2**: finding F2 is candidate and has no disposition. Fix: `dla ledger resolve F2 --as unresolved --reason time_budget --remedy "<text>"`
- **F3**: finding F3 is candidate and has no disposition. Fix: `dla ledger resolve F3 --as unresolved --reason time_budget --remedy "<text>"`
- **F4**: finding F4 is candidate and has no disposition. Fix: `dla ledger resolve F4 --as unresolved --reason time_budget --remedy "<text>"`
- **F5**: finding F5 is candidate and has no disposition. Fix: `dla ledger resolve F5 --as unresolved --reason time_budget --remedy "<text>"`
- **T1:jvm.heap_used**: column jvm.heap_used of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:jvm.heap_used.pre_gc**: column jvm.heap_used.pre_gc of T1 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:jvm.heap_used**: column jvm.heap_used of T3 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:jvm.heap_committed**: column jvm.heap_committed of T4 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:jvm.heap_used**: column jvm.heap_used of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:jvm.heap_committed.pre_gc**: column jvm.heap_committed.pre_gc of T4 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:jvm.heap_used.pre_gc**: column jvm.heap_used.pre_gc of T4 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`

## Findings

### F6: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Static field RequestContext.latest heads a singly linked chain: every RequestContext keeps a strong 'previous' reference to the prior request's context (and its 384-byte body), so no context is ever collectable. Not a collection, so L-COLLECT does not fit; the chain is only needed to log the previous path and delta in Gateway.auditLine.)
- Growth: `probe:gateway.RequestContext.latest` 3 count/n (steady, gate pass, E11)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `gateway.RequestContext.latest`
- Region: heap
- Shape: steady
- Site: `src/gateway/RequestContext.java:5`
- Writers: `src/gateway/RequestContext.java:28`
- Annotation: heuristic name match over the project's sources
- Verified: no
- Sibling hypotheses: F6.n1
- Supporting evidence: E6, E9, E11
- Explanation (observed): RequestContext.latest (RequestContext.java:5) is a static head; open() at RequestContext.java:28 links each new context to the prior one via previous, and the link is never cleared, so every request and its body stays reachable.
- Explanation structure: adds: src/gateway/RequestContext.java:28; removal: never; evidence: E11

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `src/gateway/RequestContext.java:28`
- Reason: not_exercised
- Remedy: Same growth as confirmed F6 (RequestContext.latest chain); duplicate/derived column, covered by F6's fix
- Supporting evidence: E5

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `gateway.RequestContext.body`
- Region: heap
- Site: `src/gateway/RequestContext.java:11`
- Writers: `src/gateway/RequestContext.java:20`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Same growth as confirmed F6 (RequestContext.latest chain); duplicate/derived column, covered by F6's fix
- Supporting evidence: E6

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `gateway.RequestContext.previous`
- Region: heap
- Site: `src/gateway/RequestContext.java:13`
- Writers: `src/gateway/RequestContext.java:22`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Same growth as confirmed F6 (RequestContext.latest chain); duplicate/derived column, covered by F6's fix
- Supporting evidence: E6

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Same growth as confirmed F6 (RequestContext.latest chain); duplicate/derived column, covered by F6's fix
- Supporting evidence: E11

### F10: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Same growth as confirmed F6 (RequestContext.latest chain); duplicate/derived column, covered by F6's fix
- Supporting evidence: E11

### F11: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Same growth as confirmed F6 (RequestContext.latest chain); duplicate/derived column, covered by F6's fix
- Supporting evidence: E11

### F12: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Same growth as confirmed F6 (RequestContext.latest chain); duplicate/derived column, covered by F6's fix
- Supporting evidence: E14

### F13: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Same growth as confirmed F6 (RequestContext.latest chain); duplicate/derived column, covered by F6's fix
- Supporting evidence: E14

### F14: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Same growth as confirmed F6 (RequestContext.latest chain); duplicate/derived column, covered by F6's fix
- Supporting evidence: E14

### F15: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Same growth as confirmed F6 (RequestContext.latest chain); duplicate/derived column, covered by F6's fix
- Supporting evidence: E16

### F16: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Same growth as confirmed F6 (RequestContext.latest chain); duplicate/derived column, covered by F6's fix
- Supporting evidence: E16

### F17: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Same growth as confirmed F6 (RequestContext.latest chain); duplicate/derived column, covered by F6's fix
- Supporting evidence: E16

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

## Ruled out

- **F6.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM: growth is a warm-up/cache fill that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E12
  - Originally supported by: E11

## Blind spots and unknowns

### Coverage gaps

- jvm/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- jvm/codeql: missing (codeql is not on PATH or in the tools directory). Install hint: dla install codeql
- jvm/errorprone: missing (Error Prone jars not found in the tools directory (or DLA_ERRORPRONE_HOME)). Install hint: dla install errorprone
- jvm/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)
- jvm/sonar: missing (sonar-scanner is not installed and Sonar rules need a SonarQube or SonarCloud server). Install hint: install sonar-scanner, run a SonarQube or SonarCloud server and set SONAR_HOST_URL and SONAR_TOKEN (dla install sonar prints the steps)
- jvm/valgrind: blocked (Valgrind is not run on the JVM: the virtual machine's own allocations drown the project's; use the allocator shim, jcmd NMT or `dla attach bpftrace`)

### Runtime blind spots

None recorded.

### Entry points not exercisable

None recorded.

### Evidence limitations and unknowns

- E4: partial coverage
- E4: limitation: no auxiliary classpath: library calls are not modelled
- E6: partial coverage
- E6: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E6: limitation: writers and removers are textual name matches
- E7: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E7: limitation: workloads were discovered by reading files; none was run
- E8: limitation: harness written; nothing was run (dla harness run measures it)
- E8: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E8: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E9: partial coverage (9 of 9 examined)
- E9: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E9: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E10: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E10: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E11: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: points 1 to 9 come from S1; the extra points from S2
- E12: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E12: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E12: limitation: gate and plateau thresholds are provisional (not calibrated)
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: points 1 to 9 come from S1; the extra points from S2
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: points 1 to 9 come from S1; the extra points from S2
- E16: limitation: gate thresholds are provisional (not calibrated)

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
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | — |
| E2 | run-config | static | dla 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | probe/probe.json |
| E4 | static-hit | static | spotbugs unknown | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | sweep/jvm-spotbugs.json |
| E5 | static-hit | static | dla 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | sweep/jvm-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | census/jvm.json |
| E7 | workload | static | dla 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (3e3614c8d56c) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (ebd1eb40bde8) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (ebd1eb40bde8) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (0aa2f3a0acc6) | alloc/A1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S2.json (dc6384c7d9a7) | trend/T3.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (dc6384c7d9a7) | trend/T3.json |
| E15 | growth-series | test | dla 0.1.0 | artifact series/S3.json (0aa2f3a0acc6) | trend/T4.json |
| E16 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (0aa2f3a0acc6) | trend/T4.json |
