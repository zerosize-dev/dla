# DLA report

Run: `20261110-192851-a030`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 15 enumerated, 15 dispositioned; confirmed 0, strong 1, refuted 1, suspected 7, unresolved 6

Residual: heap 509.79981981981985 bytes/n unattributed of 509.79981981981985 total (threshold 127.44995495495496) ABOVE THRESHOLD, see O1; heap 496.29099099099096 bytes/n unattributed of 496.29099099099096 total (threshold 124.07274774774774) ABOVE THRESHOLD, see O1; heap 506.41165165165165 bytes/n unattributed of 506.41165165165165 total (threshold 126.60291291291291) ABOVE THRESHOLD, see O1; non-heap 14.131891891891891 bytes/n unattributed of 14.131891891891891 total (threshold 76.68304864864918); non-heap 0 bytes/n unattributed of -6.914414414414415 total (threshold 43.15510864864864); non-heap 0 bytes/n unattributed of -12.585379665379666 total (threshold 31.708589448648674)

Coverage: jvm 8880 iterations, 19 of 19 entry points exercised

## Findings

### F6: **UNCLASSIFIED**

- State: strong
- Label: UNCLASSIFIED (static RequestContext.latest heads an unbounded linked list via previous; every request context plus body byte[] retained, never unlinked (should drop previous link / not keep history))
- Growth: `probe:gateway.RequestContext.latest` 3 count/n (steady, gate pass, E11)
- Evidence classes: context, growth, retention, static
- Strong: second class E13 retention-path that names the holder, independent of the growth record E11 (dla)
- Holder: static/global
- Owner: `gateway.RequestContext.latest`
- Region: heap
- Shape: steady
- Site: `src/gateway/RequestContext.java:5`
- Writers: `src/gateway/RequestContext.java:28`, `src/gateway/RequestContext.java:29`
- Annotation: heuristic name match over the project's sources
- Annotation: latest static set on each open; previous links chain
- Annotation: open() links each new context to the previous one via previous field and stores it in static latest; nothing ever clears previous
- Verified: no
- Sibling hypotheses: F6.n1
- Supporting evidence: E6, E9, E11, E13, E16
- Explanation (observed): RequestContext.java:5 static/global holder latest is set in open() at src/gateway/RequestContext.java:28 and each context keeps previous, so every request context and body stays reachable; removal never happens, nothing unlinks them.
- Explanation structure: adds: src/gateway/RequestContext.java:28; removal: never; evidence: E17

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Site: `src/gateway/RequestContext.java:28`
- Reason: not_exercised
- Remedy: Same chain as F6 (RequestContext.latest/previous/body); no separate growth isolated; fix F6 and re-measure
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
- Remedy: Bounded by design (fixed routes/accounts/20-entry history, audit sink counters); no growth in its probe; no separate harness probe
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
- Remedy: Bounded by design (fixed routes/accounts/20-entry history, audit sink counters); no growth in its probe; no separate harness probe
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
- Remedy: Bounded by design (fixed routes/accounts/20-entry history, audit sink counters); no growth in its probe; no separate harness probe
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
- Remedy: Bounded by design (fixed routes/accounts/20-entry history, audit sink counters); no growth in its probe; no separate harness probe
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
- Remedy: Same chain as F6 (RequestContext.latest/previous/body); no separate growth isolated; fix F6 and re-measure
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
- Remedy: Same chain as F6 (RequestContext.latest/previous/body); no separate growth isolated; fix F6 and re-measure
- Supporting evidence: E6

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Class-count growth (byte[]/String/RequestContext) is the per-request objects retained by F6; re-measure after fixing F6
- Supporting evidence: E22

### F10: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Class-count growth (byte[]/String/RequestContext) is the per-request objects retained by F6; re-measure after fixing F6
- Supporting evidence: E22

### F11: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Class-count growth (byte[]/String/RequestContext) is the per-request objects retained by F6; re-measure after fixing F6
- Supporting evidence: E22

### F12: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Class-count growth (byte[]/String/RequestContext) is the per-request objects retained by F6; re-measure after fixing F6
- Supporting evidence: E20

### F13: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Class-count growth (byte[]/String/RequestContext) is the per-request objects retained by F6; re-measure after fixing F6
- Supporting evidence: E20

### F14: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Class-count growth (byte[]/String/RequestContext) is the per-request objects retained by F6; re-measure after fixing F6
- Supporting evidence: E20

## Ruled out

- **F6.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM: one-time warm-up fill that plateaus
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
- E13: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E14: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E15: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E16: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E17: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E17: limitation: series: per-point; one process per point at n = 1110, 4440, 6660, 9990 (harness captures; checked: HPROF version, identifier size)
- E18: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: points 1 to 9 come from S1; the extra points from S2
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: points 1 to 9 come from S1; the extra points from S2
- E22: limitation: gate thresholds are provisional (not calibrated)
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
- E26: limitation: F6: growth evidence is from S1, not S2; not subtracted
- E26: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E26: limitation: threshold is provisional (not calibrated)
- E27: partial coverage (2 of 3 examined)
- E27: limitation: F6: growth evidence is from S1, not S3; not subtracted
- E27: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (a118f52d1c28) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (96bc123084eb) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (96bc123084eb) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (d3bd8f07470f) | alloc/A1.json |
| E13 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-192851-a030/harness/H1/out/p1-native/capture/82/heap-1.hprof (c5111808bf74) | summaries/heap-1-c5111808.json |
| E14 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-192851-a030/harness/H1/out/p4-native/capture/373/heap-1.hprof (dc12a7e877fa) | summaries/heap-1-dc12a7e8.json |
| E15 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-192851-a030/harness/H1/out/p6-native/capture/567/heap-1.hprof (bdc4b2a12561) | summaries/heap-1-bdc4b2a1.json |
| E16 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-192851-a030/harness/H1/out/p9-native/capture/858/heap-1.hprof (2251d19a5686) | summaries/heap-1-2251d19a.json |
| E17 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S4.json (84abc5142fc1) | series/S4.json |
| E18 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S4.json (84abc5142fc1) | series/S4.json |
| E19 | growth-series | test | dla 0.1.0 | artifact series/S2.json (3a465c7ff5e9) | trend/T3.json |
| E20 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (3a465c7ff5e9) | trend/T3.json |
| E21 | growth-series | test | dla 0.1.0 | artifact series/S3.json (d3bd8f07470f) | trend/T4.json |
| E22 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (d3bd8f07470f) | trend/T4.json |
| E23 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (84abc5142fc1) | trend/T5.json |
| E24 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (84abc5142fc1) | trend/T5.json |
| E25 | residual | test | dla 0.1.0 | artifact series/S1.json (96bc123084eb) | gap/residual-S1-E25.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S2.json (3a465c7ff5e9) | gap/residual-S2-E26.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S3.json (d3bd8f07470f) | gap/residual-S3-E27.json |
