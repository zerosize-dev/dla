# DLA report

Run: `20261108-200609-bc6f`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 9 enumerated, 9 dispositioned; confirmed 1, strong 0, refuted 1, suspected 7, unresolved 0

Residual: not measured

Coverage: jvm 8880 iterations, 19 of 19 entry points exercised

## Findings

### F1: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:gateway.RequestContext.latest` 3 count/n (steady, gate pass, E11)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `gateway.RequestContext.latest`
- Region: heap
- Shape: steady
- Site: `src/gateway/RequestContext.java:28`
- Annotation: static latest pins newest RequestContext; each holds previous (final field, line 13/22), forming an unbounded linked chain of every request incl. 384-byte body. Nothing nulls previous or latest. Only previousPath/microsSincePrevious need the prior request, but they retain whole object.
- Sibling hypotheses: F1.n1
- Supporting evidence: E6, E9, E11

### F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `gateway.Gateway.routes`
- Region: heap
- Site: `src/gateway/Gateway.java:9`
- Writers: `src/gateway/Gateway.java:14`, `src/gateway/Gateway.java:15`, `src/gateway/Gateway.java:16`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: No per-holder probe grew; region-level heap_used growth is explained by F1. Source review: bounded (fixed route/account keys; history capped at 20; auditSink is a counter lambda). Probe this holder individually with a driver-mode harness to refute formally.
- Supporting evidence: E7

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: closure/continuation
- Owner: `gateway.Gateway.auditSink`
- Region: heap
- Site: `src/gateway/Gateway.java:10`
- Writers: `src/gateway/Gateway.java:17`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: No per-holder probe grew; region-level heap_used growth is explained by F1. Source review: bounded (fixed route/account keys; history capped at 20; auditSink is a counter lambda). Probe this holder individually with a driver-mode harness to refute formally.
- Supporting evidence: E7

### F4: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `gateway.Ledger.balances`
- Region: heap
- Site: `src/gateway/Ledger.java:12`
- Writers: `src/gateway/Ledger.java:18`, `src/gateway/Ledger.java:36`, `src/gateway/Ledger.java:37`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: No per-holder probe grew; region-level heap_used growth is explained by F1. Source review: bounded (fixed route/account keys; history capped at 20; auditSink is a counter lambda). Probe this holder individually with a driver-mode harness to refute formally.
- Supporting evidence: E7

### F5: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `gateway.Ledger.history`
- Region: heap
- Site: `src/gateway/Ledger.java:13`
- Writers: `src/gateway/Ledger.java:19`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: No per-holder probe grew; region-level heap_used growth is explained by F1. Source review: bounded (fixed route/account keys; history capped at 20; auditSink is a counter lambda). Probe this holder individually with a driver-mode harness to refute formally.
- Supporting evidence: E7

### F6: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: growth, static
- Holder: static/global
- Owner: `gateway.RequestContext.latest`
- Region: heap
- Site: `src/gateway/RequestContext.java:5`
- Writers: `src/gateway/RequestContext.java:28`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Part of the same RequestContext chain as confirmed F1 (latest/body/previous); fixed by F1's repair, not an independent leak.
- Supporting evidence: E7, E11

### F7: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: growth, static
- Holder: container-field
- Owner: `gateway.RequestContext.body`
- Region: heap
- Site: `src/gateway/RequestContext.java:11`
- Writers: `src/gateway/RequestContext.java:20`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Part of the same RequestContext chain as confirmed F1 (latest/body/previous); fixed by F1's repair, not an independent leak.
- Supporting evidence: E7, E11

### F8: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: growth, static
- Holder: container-field
- Owner: `gateway.RequestContext.previous`
- Region: heap
- Site: `src/gateway/RequestContext.java:13`
- Writers: `src/gateway/RequestContext.java:22`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Part of the same RequestContext chain as confirmed F1 (latest/body/previous); fixed by F1's repair, not an independent leak.
- Supporting evidence: E7, E11

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM: growth is a warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles of alloc-test
  - Contradicting evidence: E12
  - Originally supported by: E11

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

- E4: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E4: limitation: workloads were discovered by reading files; none was run
- E5: partial coverage
- E5: limitation: no auxiliary classpath: library calls are not modelled
- E7: partial coverage
- E7: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E7: limitation: writers and removers are textual name matches
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
- E13: limitation: /proc/18/status lacks os.rss, os.rss_anon, os.rss_file, os.rss_shmem, os.swap, os.vm_data, os.vm_hwm (kernel thread or old kernel)
- E13: limitation: os.pss, os.pss_anon, os.private_dirty: /proc/18/smaps_rollup unreadable (No such process (os error 3))
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: points 1 to 9 come from S1; the extra points from S4
- E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E13: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: points 1 to 9 come from S1; the extra points from S6
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: points 1 to 9 come from S1; the extra points from S8
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: points 1 to 9 come from S1; the extra points from S10
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | — |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | — |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | — |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E3 | — |
| `intake.build` | Does the project build locally? | answered | E3 | — |
| `intake.workload` | What can exercise the suspected path? | answered | E3 | — |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | — |
| E2 | run-config | static | dla 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | probe/probe.json |
| E4 | workload | static | dla 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | workload/workloads.json |
| E5 | static-hit | static | spotbugs unknown | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | sweep/jvm-spotbugs.json |
| E6 | static-hit | static | dla 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | sweep/jvm-dla.json |
| E7 | census-holder | static | dla-census 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | census/jvm.json |
| E8 | workload | static | dla 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (0b3fc6fb9a97) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (6a5f8716221c) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (6a5f8716221c) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (b1466685cd21) | alloc/A1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S5.json (091922e14516) | alloc/A2.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S7.json (f18aa0724e60) | alloc/A3.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S9.json (31aafaf59fba) | alloc/A4.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S11.json (48fd2293f3ed) | alloc/A5.json |
