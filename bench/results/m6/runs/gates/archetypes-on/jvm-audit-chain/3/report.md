# DLA report

Run: `20261110-173800-f61c`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 16 enumerated, 16 dispositioned; confirmed 0, strong 1, refuted 1, suspected 7, unresolved 7

Residual: heap 514.9445302445303 bytes/n unattributed of 514.9445302445303 total (threshold 128.73613256113256) ABOVE THRESHOLD, see O1; heap 504.96024024024024 bytes/n unattributed of 504.96024024024024 total (threshold 126.24006006006006) ABOVE THRESHOLD, see O1; heap 509.2432432432432 bytes/n unattributed of 509.2432432432432 total (threshold 127.3108108108108) ABOVE THRESHOLD, see O1; non-heap 3.1339339339339336 bytes/n unattributed of 3.1339339339339336 total (threshold 54.24522378378395); non-heap 0 bytes/n unattributed of -6.962162162162162 total (threshold 7.443911351351352); non-heap 0 bytes/n unattributed of -5.13976833976834 total (threshold 11.756693621621553)

Coverage: jvm 8880 iterations, 19 of 19 entry points exercised

## Findings

### F6: **UNCLASSIFIED**

- State: strong
- Label: UNCLASSIFIED (Static-rooted singly linked chain: RequestContext.latest plus per-instance previous link retain every request context and its 384-byte body; not a collection or listener, nothing removes links)
- Growth: `probe:gateway.RequestContext.latest` 3 count/n (steady, gate pass, E11)
- Evidence classes: context, discriminator, growth, retention, static
- Strong: second class E17 retention-path that names the holder, independent of the growth record E11 (dla)
- Holder: static/global
- Owner: `gateway.RequestContext.latest`
- Region: heap
- Shape: steady
- Site: `src/gateway/RequestContext.java:5`
- Writers: `src/gateway/RequestContext.java:28`, `src/gateway/RequestContext.java:29`
- Annotation: heuristic name match over the project's sources
- Annotation: static latest pins newest RequestContext; each context's previous link keeps all earlier contexts (and their body byte[]) reachable; nothing ever nulls latest or previous
- Verified: no
- Sibling hypotheses: F6.n1
- Supporting evidence: E6, E9, E11, E12, E17, E18, E19, E20
- Explanation (inferred): Static field RequestContext.latest (RequestContext.java:5) is the root; open() at src/gateway/RequestContext.java:28 links each new context to the prior one via previous, so every request and its body stays reachable and is never cleared.
- Explanation structure: adds: src/gateway/RequestContext.java:28; removal: never

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Site: `src/gateway/RequestContext.java:28`
- Reason: not_exercised
- Remedy: Same mechanism as F6 (static-rooted RequestContext chain); fixing F6 resolves it
- Supporting evidence: E5

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
- Remedy: Census holder with bounded or constant size; no growth tied; plateau in harness
- Supporting evidence: E6

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
- Remedy: Census holder with bounded or constant size; no growth tied; plateau in harness
- Supporting evidence: E6

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
- Remedy: Census holder with bounded or constant size; no growth tied; plateau in harness
- Supporting evidence: E6

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
- Remedy: Census holder with bounded or constant size; no growth tied; plateau in harness
- Supporting evidence: E6

### F7: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `gateway.RequestContext.body`
- Region: heap
- Site: `src/gateway/RequestContext.java:11`
- Writers: `src/gateway/RequestContext.java:20`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Same mechanism as F6 (static-rooted RequestContext chain); fixing F6 resolves it
- Supporting evidence: E6

### F8: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `gateway.RequestContext.previous`
- Region: heap
- Site: `src/gateway/RequestContext.java:13`
- Writers: `src/gateway/RequestContext.java:22`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Same mechanism as F6 (static-rooted RequestContext chain); fixing F6 resolves it
- Supporting evidence: E6

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `gateway.RequestContext.latest`
- Region: heap
- Shape: steady
- Site: `src/gateway/RequestContext.java:28`
- Reason: not_exercised
- Remedy: Same mechanism as F6 (static-rooted RequestContext chain); fixing F6 resolves it
- Supporting evidence: E14

### F10: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `gateway.RequestContext.latest`
- Region: heap
- Shape: steady
- Site: `src/gateway/RequestContext.java:28`
- Reason: not_exercised
- Remedy: Same mechanism as F6 (static-rooted RequestContext chain); fixing F6 resolves it
- Supporting evidence: E14

### F11: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `gateway.RequestContext.latest`
- Region: heap
- Shape: steady
- Site: `src/gateway/RequestContext.java:28`
- Reason: not_exercised
- Remedy: Same mechanism as F6 (static-rooted RequestContext chain); fixing F6 resolves it
- Supporting evidence: E14

### F12: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `gateway.RequestContext.latest`
- Region: heap
- Shape: steady
- Site: `src/gateway/RequestContext.java:28`
- Reason: not_exercised
- Remedy: Same mechanism as F6 (static-rooted RequestContext chain); fixing F6 resolves it
- Supporting evidence: E16

### F13: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `gateway.RequestContext.latest`
- Region: heap
- Shape: steady
- Site: `src/gateway/RequestContext.java:28`
- Reason: not_exercised
- Remedy: Same mechanism as F6 (static-rooted RequestContext chain); fixing F6 resolves it
- Supporting evidence: E16

### F14: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `gateway.RequestContext.latest`
- Region: heap
- Shape: steady
- Site: `src/gateway/RequestContext.java:28`
- Reason: not_exercised
- Remedy: Same mechanism as F6 (static-rooted RequestContext chain); fixing F6 resolves it
- Supporting evidence: E16

### F15: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `gateway.RequestContext.latest`
- Region: heap
- Shape: steady
- Site: `src/gateway/RequestContext.java:28`
- Reason: not_exercised
- Remedy: Same mechanism as F6 (static-rooted RequestContext chain); fixing F6 resolves it
- Supporting evidence: E16

## Ruled out

- **F6.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM: one-time fill that plateaus
  - Refuted if: growth continues over the extra cycles and survives purge
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
- E9: limitation: command-mode heap dumps come from separate processes: dla summarize heap-graph compares dumps of one process only, so use a driver (dla harness gen --driver) for a dump series
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
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: points 1 to 9 come from S1; the extra points from S2
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: points 1 to 9 come from S1; the extra points from S2
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E18: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E19: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E20: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E21: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E21: limitation: series: per-point; one process per point at n = 1110, 4440, 6660, 9990 (harness captures; checked: HPROF version, identifier size)
- E22: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E23: limitation: series: per-point; x is the iteration count n of each point (1110, 4440, 6660, 9990); every point is a separate process and heap dump
- E23: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E23: limitation: holders absent from a dump's 200 largest have no figure at that point
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: series: per-point; x is the iteration count n of each point (1110, 4440, 6660, 9990); every point is a separate process and heap dump
- E24: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E24: limitation: holders absent from a dump's 200 largest have no figure at that point
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
- E27: limitation: F6: growth evidence is from S1, not S2; not subtracted
- E27: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (7211ae0ff11d) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (8c0796f1d3df) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (8c0796f1d3df) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (230cd7bcd285) | alloc/A1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S3.json (230cd7bcd285) | trend/T3.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (230cd7bcd285) | trend/T3.json |
| E15 | growth-series | test | dla 0.1.0 | artifact series/S2.json (1adbddba9da7) | trend/T4.json |
| E16 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (1adbddba9da7) | trend/T4.json |
| E17 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-173800-f61c/harness/H1/out/p1-native/capture/82/heap-1.hprof (b9b92f2dfbb1) | summaries/heap-1-b9b92f2d.json |
| E18 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-173800-f61c/harness/H1/out/p4-native/capture/374/heap-1.hprof (19ce9a945e6d) | summaries/heap-1-19ce9a94.json |
| E19 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-173800-f61c/harness/H1/out/p6-native/capture/571/heap-1.hprof (1271bb39446c) | summaries/heap-1-1271bb39.json |
| E20 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-173800-f61c/harness/H1/out/p9-native/capture/864/heap-1.hprof (80cfcd4cf2a9) | summaries/heap-1-80cfcd4c.json |
| E21 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S4.json (042f7630a0bd) | series/S4.json |
| E22 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S4.json (042f7630a0bd) | series/S4.json |
| E23 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (042f7630a0bd) | trend/T5.json |
| E24 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (042f7630a0bd) | trend/T5.json |
| E25 | residual | test | dla 0.1.0 | artifact series/S1.json (8c0796f1d3df) | gap/residual-S1-E25.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S3.json (230cd7bcd285) | gap/residual-S3-E26.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S2.json (1adbddba9da7) | gap/residual-S2-E27.json |
