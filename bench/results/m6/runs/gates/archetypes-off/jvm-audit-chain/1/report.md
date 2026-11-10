# DLA report

Run: `20261110-181728-de90`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 12 enumerated, 12 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 10

Residual: heap 511.63063063063066 bytes/n unattributed of 511.63063063063066 total (threshold 127.90765765765768) ABOVE THRESHOLD, see O1; heap 498.46486486486486 bytes/n unattributed of 498.46486486486486 total (threshold 124.61621621621622) ABOVE THRESHOLD, see O1; heap 504.97135135135136 bytes/n unattributed of 504.97135135135136 total (threshold 126.24283783783784) ABOVE THRESHOLD, see O1; non-heap 1.2813813813813812 bytes/n unattributed of 1.2813813813813812 total (threshold 66.28176054054018); non-heap 0 bytes/n unattributed of -4.384041184041184 total (threshold 25.20632617760633); non-heap 0 bytes/n unattributed of -6.957534807534808 total (threshold 29.04034553808383)

Coverage: jvm 8880 iterations, 19 of 19 entry points exercised

## Findings

### F6: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Static RequestContext.latest plus per-instance previous field forms an unbounded singly linked chain of every request context (with its 384-byte body); nothing ever nulls previous or latest)
- Growth: `probe:gateway.RequestContext.latest` 3 count/n (steady, gate pass, E11)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `gateway.RequestContext.latest`
- Region: heap
- Shape: steady
- Site: `src/gateway/RequestContext.java:5`
- Writers: `src/gateway/RequestContext.java:28`
- Annotation: heuristic name match over the project's sources
- Annotation: static latest points to newest context
- Verified: no
- Sibling hypotheses: F6.n1
- Supporting evidence: E6, E9, E11, E16
- Explanation (observed): Static field latest (RequestContext.java:5, static/global holder) is set to each new context at src/gateway/RequestContext.java:28, and its previous field links the prior one; previous is never cleared, so every request and body stays reachable.
- Explanation structure: adds: src/gateway/RequestContext.java:28; removal: never; evidence: E16

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth, static
- Site: `src/gateway/RequestContext.java:28`
- Reason: not_exercised
- Remedy: Same retained chain as confirmed F6 (RequestContext.latest/previous); no separate fix needed
- Supporting evidence: E5, E16

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `gateway.Gateway.routes`
- Region: heap
- Site: `src/gateway/Gateway.java:9`
- Writers: `src/gateway/Gateway.java:14`, `src/gateway/Gateway.java:15`, `src/gateway/Gateway.java:16`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: No growth tied to this holder in the harness; bounded by code (fixed keys / history capped at 20). Not a leak candidate
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: closure/continuation
- Owner: `gateway.Gateway.auditSink`
- Region: heap
- Site: `src/gateway/Gateway.java:10`
- Writers: `src/gateway/Gateway.java:17`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: No growth tied to this holder in the harness; bounded by code (fixed keys / history capped at 20). Not a leak candidate
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `gateway.Ledger.balances`
- Region: heap
- Site: `src/gateway/Ledger.java:12`
- Writers: `src/gateway/Ledger.java:18`, `src/gateway/Ledger.java:36`, `src/gateway/Ledger.java:37`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: No growth tied to this holder in the harness; bounded by code (fixed keys / history capped at 20). Not a leak candidate
- Supporting evidence: E6

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `gateway.Ledger.history`
- Region: heap
- Site: `src/gateway/Ledger.java:13`
- Writers: `src/gateway/Ledger.java:19`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: No growth tied to this holder in the harness; bounded by code (fixed keys / history capped at 20). Not a leak candidate
- Supporting evidence: E6

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth, static
- Holder: container-field
- Owner: `gateway.RequestContext.body`
- Region: heap
- Site: `src/gateway/RequestContext.java:11`
- Writers: `src/gateway/RequestContext.java:20`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Same retained chain as confirmed F6 (RequestContext.latest/previous); no separate fix needed
- Supporting evidence: E6, E16

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth, static
- Holder: container-field
- Owner: `gateway.RequestContext.previous`
- Region: heap
- Site: `src/gateway/RequestContext.java:13`
- Writers: `src/gateway/RequestContext.java:22`, `src/gateway/RequestContext.java:26`
- Annotation: heuristic name match over the project's sources
- Annotation: previous links each context to its predecessor; never cleared
- Reason: not_exercised
- Remedy: Same retained chain as confirmed F6 (RequestContext.latest/previous); no separate fix needed
- Supporting evidence: E6, E16

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Same retained chain as confirmed F6 (RequestContext.latest/previous); no separate fix needed
- Supporting evidence: E11, E16

### F10: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Same retained chain as confirmed F6 (RequestContext.latest/previous); no separate fix needed
- Supporting evidence: E22, E16

### F11: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Same retained chain as confirmed F6 (RequestContext.latest/previous); no separate fix needed
- Supporting evidence: E24, E16

## Ruled out

- **F6.n1** (UNCLASSIFIED): discriminator E18
  - Ruled out by: plateau+purge (leak) E18
  - Hypothesis: N-WARM: bounded warm-up fill
  - Refuted if: growth continues over extra cycles and dump-plateau favors leak
  - Contradicting evidence: E18
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
- E9: limitation: command-mode heap dumps come from separate processes: dla summarize heap-graph compares dumps of one process only, so use a driver (dla harness gen --driver) for a dump series
- E9: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E9: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E10: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E10: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E11: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E13: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E14: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E15: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E16: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E16: limitation: series: per-point; one process per point at n = 1110, 4440, 6660, 9990 (harness captures; checked: HPROF version, identifier size)
- E17: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E18: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: points 1 to 9 come from S1; the extra points from S3
- E18: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E18: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E18: limitation: gate and plateau thresholds are provisional (not calibrated)
- E19: limitation: series: per-point; x is the iteration count n of each point (1110, 4440, 6660, 9990); every point is a separate process and heap dump
- E19: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E19: limitation: holders absent from a dump's 200 largest have no figure at that point
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: series: per-point; x is the iteration count n of each point (1110, 4440, 6660, 9990); every point is a separate process and heap dump
- E20: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E20: limitation: holders absent from a dump's 200 largest have no figure at that point
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E23: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E23: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E23: limitation: points 1 to 9 come from S1; the extra points from S3
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E24: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E24: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E24: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E24: limitation: points 1 to 9 come from S1; the extra points from S3
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: partial coverage (2 of 3 examined)
- E25: limitation: F6: growth measured as a count; bytes not attributed
- E25: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E25: limitation: threshold is provisional (not calibrated)
- E26: partial coverage (2 of 3 examined)
- E26: limitation: F6: growth evidence is from S1, not S3; not subtracted
- E26: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E26: limitation: threshold is provisional (not calibrated)
- E27: partial coverage (2 of 3 examined)
- E27: limitation: F6: growth evidence is from S1, not S4; not subtracted
- E27: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E27: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | — |
| E2 | run-config | static | dla 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | probe/probe.json |
| E4 | static-hit | static | spotbugs unknown | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | sweep/jvm-spotbugs.json |
| E5 | static-hit | static | dla 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | sweep/jvm-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | census/jvm.json |
| E7 | workload | static | dla 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo fe3bb27e3ff142bfefee871fee10a5c2fbcefb77 | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (b40c23838c06) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (c10175d6d585) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (c10175d6d585) | trend/T1.json |
| E12 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-181728-de90/harness/H1/out/p1-native/capture/82/heap-1.hprof (f57f7c302322) | summaries/heap-1-f57f7c30.json |
| E13 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-181728-de90/harness/H1/out/p4-native/capture/373/heap-1.hprof (944ab5b242bb) | summaries/heap-1-944ab5b2.json |
| E14 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-181728-de90/harness/H1/out/p6-native/capture/567/heap-1.hprof (55ac3f78089e) | summaries/heap-1-55ac3f78.json |
| E15 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-181728-de90/harness/H1/out/p9-native/capture/859/heap-1.hprof (dec388848ed7) | summaries/heap-1-dec38884.json |
| E16 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S2.json (3cf880487c74) | series/S2.json |
| E17 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S2.json (3cf880487c74) | series/S2.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S4.json (69abe0516fc2) | alloc/A1.json |
| E19 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (3cf880487c74) | trend/T3.json |
| E20 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (3cf880487c74) | trend/T3.json |
| E21 | growth-series | test | dla 0.1.0 | artifact series/S3.json (82ff9831fa65) | trend/T4.json |
| E22 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (82ff9831fa65) | trend/T4.json |
| E23 | growth-series | test | dla 0.1.0 | artifact series/S4.json (69abe0516fc2) | trend/T5.json |
| E24 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (69abe0516fc2) | trend/T5.json |
| E25 | residual | test | dla 0.1.0 | artifact series/S1.json (c10175d6d585) | gap/residual-S1-E25.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S3.json (82ff9831fa65) | gap/residual-S3-E26.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S4.json (69abe0516fc2) | gap/residual-S4-E27.json |
