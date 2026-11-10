# DLA report

Run: `20261110-162254-9188`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 13 enumerated, 13 dispositioned; confirmed 0, strong 1, refuted 1, suspected 0, unresolved 11

Residual: heap 510.2804804804805 bytes/n unattributed of 510.2804804804805 total (threshold 127.57012012012012) ABOVE THRESHOLD, see O1; heap 495.02882882882886 bytes/n unattributed of 495.02882882882886 total (threshold 123.75720720720722) ABOVE THRESHOLD, see O1; heap 503.5441441441441 bytes/n unattributed of 503.5441441441441 total (threshold 125.88603603603605) ABOVE THRESHOLD, see O1; non-heap 0 bytes/n unattributed of -13.615915915915917 total (threshold 26.028121081080897); non-heap 0 bytes/n unattributed of -10.351231231231232 total (threshold 52.00220454054051); non-heap 0 bytes/n unattributed of -24.562517062517063 total (threshold 29.00628580835385)

Coverage: jvm 8880 iterations, 19 of 19 entry points exercised

## Findings

### F6: **UNCLASSIFIED**

- State: strong
- Label: UNCLASSIFIED (Static RequestContext.latest heads a singly linked chain: each new context stores the previous one (and its body byte[]), so every served request stays reachable; no code ever clears previous or latest. Not a collection or listener.)
- Growth: `probe:gateway.RequestContext.latest` 3 count/n (steady, gate pass, E11)
- Evidence classes: context, discriminator, growth, retention, static
- Strong: second class E20 retention-path that names the holder, independent of the growth record E11 (dla)
- Holder: static/global
- Owner: `gateway.RequestContext.latest`
- Region: heap
- Shape: steady
- Site: `src/gateway/RequestContext.java:5`
- Writers: `src/gateway/RequestContext.java:28`
- Annotation: heuristic name match over the project's sources
- Annotation: static latest + previous chain keeps every RequestContext (and its body byte[]) reachable; nothing ever nulls previous or latest
- Verified: no
- Sibling hypotheses: F6.n1
- Supporting evidence: E6, E9, E11, E12, E20, E21
- Explanation (observed): Static field latest (RequestContext.java:5, static/global holder) is reassigned per request at RequestContext.java:28 and each context keeps a previous link plus its body, so the chain grows with every request and is never cut.
- Explanation structure: adds: src/gateway/RequestContext.java:28; removal: never; evidence: E21

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `src/gateway/RequestContext.java:28`
- Reason: not_exercised
- Remedy: Duplicate view of F6 (RequestContext.latest/previous chain, line 28 write); covered by F6 confirmation
- Supporting evidence: E5

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
- Remedy: No probe column grew for this holder in the harness; bounded by design (route table / fixed accounts / 20-entry history). Not tested further.
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
- Remedy: No probe column grew for this holder in the harness; bounded by design (route table / fixed accounts / 20-entry history). Not tested further.
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
- Remedy: No probe column grew for this holder in the harness; bounded by design (route table / fixed accounts / 20-entry history). Not tested further.
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
- Remedy: No probe column grew for this holder in the harness; bounded by design (route table / fixed accounts / 20-entry history). Not tested further.
- Supporting evidence: E6

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
- Remedy: Duplicate view of F6 (same RequestContext.latest/previous chain); covered by F6 confirmation
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
- Remedy: Duplicate view of F6 (RequestContext.latest/previous chain, line 28 write); covered by F6 confirmation
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
- Remedy: Duplicate view of F6 (same RequestContext.latest/previous chain); covered by F6 confirmation
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
- Remedy: Duplicate view of F6 (same RequestContext.latest/previous chain); covered by F6 confirmation
- Supporting evidence: E16

### F11: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `gateway.RequestContext.latest`
- Region: heap
- Shape: steady
- Site: `src/gateway/RequestContext.java:28`
- Reason: not_exercised
- Remedy: Duplicate view of F6 (same RequestContext.latest/previous chain); covered by F6 confirmation
- Supporting evidence: E28

### F12: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `gateway.RequestContext.latest`
- Region: heap
- Shape: steady
- Site: `src/gateway/RequestContext.java:28`
- Reason: not_exercised
- Remedy: Duplicate view of F6 (same RequestContext.latest/previous chain); covered by F6 confirmation
- Supporting evidence: E27

## Ruled out

- **F6.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM: warm-up fill that plateaus
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
- E17: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E18: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E19: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E20: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E21: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E21: limitation: series: per-point; one process per point at n = 1110, 4440, 6660, 9990 (harness captures; checked: HPROF version, identifier size)
- E22: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E23: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E23: limitation: the files carry no time, so they are ordered as given
- E24: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E24: limitation: the files carry no time, so they are ordered as given
- E24: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E25: limitation: series: per-point; x is the iteration count n of each point (1110, 4440, 6660, 9990); every point is a separate process and heap dump
- E25: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E25: limitation: holders absent from a dump's 200 largest have no figure at that point
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: series: per-point; x is the iteration count n of each point (1110, 4440, 6660, 9990); every point is a separate process and heap dump
- E26: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E26: limitation: holders absent from a dump's 200 largest have no figure at that point
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E27: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E28: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: partial coverage (2 of 3 examined)
- E29: limitation: F6: growth measured as a count; bytes not attributed
- E29: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E29: limitation: threshold is provisional (not calibrated)
- E30: partial coverage (2 of 3 examined)
- E30: limitation: F6: growth evidence is from S1, not S2; not subtracted
- E30: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E30: limitation: threshold is provisional (not calibrated)
- E31: partial coverage (2 of 3 examined)
- E31: limitation: F6: growth evidence is from S1, not S3; not subtracted
- E31: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E31: limitation: threshold is provisional (not calibrated)

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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (fd8c93ada347) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (e11ec3d19bb3) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (e11ec3d19bb3) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (9370cef1e865) | alloc/A1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S2.json (478131d4e4d0) | trend/T3.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (478131d4e4d0) | trend/T3.json |
| E15 | growth-series | test | dla 0.1.0 | artifact series/S3.json (9370cef1e865) | trend/T4.json |
| E16 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (9370cef1e865) | trend/T4.json |
| E17 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-162254-9188/harness/H1/out/p1-native/capture/82/heap-1.hprof (3ff539f15504) | summaries/heap-1-3ff539f1.json |
| E18 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-162254-9188/harness/H1/out/p4-native/capture/373/heap-1.hprof (e6231156b9c3) | summaries/heap-1-e6231156.json |
| E19 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-162254-9188/harness/H1/out/p6-native/capture/567/heap-1.hprof (5e50a0dccedd) | summaries/heap-1-5e50a0dc.json |
| E20 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-162254-9188/harness/H1/out/p9-native/capture/858/heap-1.hprof (ef97acd601e9) | summaries/heap-1-ef97acd6.json |
| E21 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S4.json (18907ad3393c) | series/S4.json |
| E22 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S4.json (18907ad3393c) | series/S4.json |
| E23 | histogram-diff | live | jcmd-class-histogram 0.1.0 | artifact histograms/H1.json (4056e2bd9181) | histograms/H1.json |
| E24 | growth-series | live | jcmd-class-histogram 0.1.0 | artifact series/S5.json (eddbebab6f7d) | series/S5.json |
| E25 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (18907ad3393c) | trend/T5.json |
| E26 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (18907ad3393c) | trend/T5.json |
| E27 | growth-series | dump | dla 0.1.0 | artifact series/S5.json (eddbebab6f7d) | trend/T6.json |
| E28 | unit-count-series | dump | dla 0.1.0 | artifact series/S5.json (eddbebab6f7d) | trend/T6.json |
| E29 | residual | test | dla 0.1.0 | artifact series/S1.json (e11ec3d19bb3) | gap/residual-S1-E29.json |
| E30 | residual | test | dla 0.1.0 | artifact series/S2.json (478131d4e4d0) | gap/residual-S2-E30.json |
| E31 | residual | test | dla 0.1.0 | artifact series/S3.json (9370cef1e865) | gap/residual-S3-E31.json |
