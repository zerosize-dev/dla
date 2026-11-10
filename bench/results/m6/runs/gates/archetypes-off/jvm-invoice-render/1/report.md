# DLA report

Run: `20261110-183727-3c4f`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 10 enumerated, 10 dispositioned; confirmed 0, strong 1, refuted 1, suspected 6, unresolved 2

Residual: heap 1267.091891891892 bytes/n unattributed of 1267.091891891892 total (threshold 316.772972972973) ABOVE THRESHOLD, see O1; heap 1275.259459459459 bytes/n unattributed of 1275.259459459459 total (threshold 318.81486486486483) ABOVE THRESHOLD, see O1; heap 1268.558815958816 bytes/n unattributed of 1268.558815958816 total (threshold 317.139703989704) ABOVE THRESHOLD, see O1; non-heap 12.906306306306304 bytes/n unattributed of 12.906306306306304 total (threshold 38.71704); non-heap 0 bytes/n unattributed of -0.8465465465465465 total (threshold 57.32214351351386); non-heap 1.218018018018018 bytes/n unattributed of 1.218018018018018 total (threshold 21.956371027027025)

Coverage: jvm 8880 iterations, 13 of 13 entry points exercised

## Findings

### F6: **UNCLASSIFIED**

- State: strong
- Label: UNCLASSIFIED (ThreadLocal<List<String>> RenderContext.SEGMENTS on the 4 long-lived pool threads: render() appends segments and reads a subList from mark, but nothing ever truncates or removes the list, so every rendered invoice's text segments stay reachable from the worker threads)
- Growth: `probe:billing.RenderContext.SEGMENTS` 12.53 count/n (steady, gate pass, E11)
- Evidence classes: context, discriminator, growth, retention, static
- Strong: second class E22 retention-path that names the holder, independent of the growth record E11 (dla)
- Holder: thread-local
- Owner: `billing.RenderContext.SEGMENTS`
- Region: heap
- Shape: steady
- Site: `src/billing/RenderContext.java:8`
- Writers: `src/billing/RenderContext.java:13`
- Annotation: heuristic name match over the project's sources
- Annotation: per-thread ArrayList in ThreadLocal never cleared; render marks and slices but never truncates
- Annotation: append never paired with clear/remove/ThreadLocal.remove; mark only slices
- Verified: no
- Sibling hypotheses: F6.n1
- Supporting evidence: E6, E9, E11, E12, E22, E23
- Explanation (inferred): RenderContext.java:8 SEGMENTS is a thread-local holder on the pool workers; append at RenderContext.java:13 adds every invoice's segments and nothing ever clears or removes the list, so removal never happens.
- Explanation structure: adds: src/billing/RenderContext.java:13; removal: never

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `billing.Invoice.lines`
- Region: heap
- Site: `src/billing/Invoice.java:6`
- Reason: not_exercised
- Remedy: census lead (immutable constant, Invoice.lines or pool/thread holder); no probe grew for it and the confirmed growth is held by F6
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `billing.InvoiceFeed.SKUS`
- Region: heap
- Site: `src/billing/InvoiceFeed.java:9`
- Reason: not_exercised
- Remedy: census lead (immutable constant, Invoice.lines or pool/thread holder); no probe grew for it and the confirmed growth is held by F6
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `billing.InvoiceFeed.NAMES`
- Region: heap
- Site: `src/billing/InvoiceFeed.java:10`
- Reason: not_exercised
- Remedy: census lead (immutable constant, Invoice.lines or pool/thread holder); no probe grew for it and the confirmed growth is held by F6
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `billing.InvoiceService.pool`
- Region: non-heap
- Site: `src/billing/InvoiceService.java:17`
- Reason: not_exercised
- Remedy: census lead (immutable constant, Invoice.lines or pool/thread holder); no probe grew for it and the confirmed growth is held by F6
- Supporting evidence: E6

### F5: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `billing.InvoiceService$WorkerFactory.newThread`
- Region: non-heap
- Site: `src/billing/InvoiceService.java:58`
- Reason: not_exercised
- Remedy: census lead (immutable constant, Invoice.lines or pool/thread holder); no probe grew for it and the confirmed growth is held by F6
- Supporting evidence: E6

### F7: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `billing.TaxTable.RATES`
- Region: heap
- Site: `src/billing/TaxTable.java:7`
- Reason: not_exercised
- Remedy: census lead (immutable constant, Invoice.lines or pool/thread holder); no probe grew for it and the confirmed growth is held by F6
- Supporting evidence: E6

### F8: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: time_budget
- Remedy: duplicate growth lead of F6 (byte[]/String counts are the segments held by F6)
- Supporting evidence: E11

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: catch-all for byte[]/String count growth; these are the segment strings held by F6, covered by F6's confirmation; no separate holder
- Supporting evidence: E14, E16, E27, E28, E29, E30, E31, E32

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
- E17: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E17: limitation: the files carry no time, so they are ordered as given
- E18: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E18: limitation: the files carry no time, so they are ordered as given
- E18: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E19: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E20: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E21: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E22: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E23: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E23: limitation: series: per-point; one process per point at n = 1110, 4440, 6660, 9990 (harness captures; checked: HPROF version, identifier size)
- E24: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E25: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E25: limitation: the files carry no time, so they are ordered as given
- E26: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E26: limitation: the files carry no time, so they are ordered as given
- E26: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E27: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E27: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E28: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: limitation: series: per-point; x is the iteration count n of each point (1110, 4440, 6660, 9990); every point is a separate process and heap dump
- E29: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E29: limitation: holders absent from a dump's 200 largest have no figure at that point
- E29: limitation: gate thresholds are provisional (not calibrated)
- E30: limitation: series: per-point; x is the iteration count n of each point (1110, 4440, 6660, 9990); every point is a separate process and heap dump
- E30: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E30: limitation: holders absent from a dump's 200 largest have no figure at that point
- E30: limitation: gate thresholds are provisional (not calibrated)
- E31: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E31: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E31: limitation: gate thresholds are provisional (not calibrated)
- E32: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E32: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E32: limitation: gate thresholds are provisional (not calibrated)
- E33: limitation: tested in a scratch copy; the project was not changed
- E33: limitation: gate thresholds are provisional (not calibrated); the 0.2 and 0.5 slope fractions are placeholders
- E33: limitation: exercised is null: the harness records entry points, not lines, so whether the edited line ran is not checked
- E33: limitation: every measured process of mode native exited with a non-zero code (p1-native: 1, p2-native: 1, p3-native: 1, p4-native: 1): the harness itself failed, so no series was written
- E34: partial coverage (2 of 3 examined)
- E34: limitation: F6: growth measured as a count; bytes not attributed
- E34: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E34: limitation: threshold is provisional (not calibrated)
- E35: partial coverage (2 of 3 examined)
- E35: limitation: F6: growth evidence is from S1, not S2; not subtracted
- E35: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E35: limitation: threshold is provisional (not calibrated)
- E36: partial coverage (2 of 3 examined)
- E36: limitation: F6: growth evidence is from S1, not S3; not subtracted
- E36: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E36: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | — |
| E2 | run-config | static | dla 0.1.0 | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | probe/probe.json |
| E4 | static-hit | static | spotbugs unknown | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | sweep/jvm-spotbugs.json |
| E5 | static-hit | static | dla 0.1.0 | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | sweep/jvm-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | census/jvm.json |
| E7 | workload | static | dla 0.1.0 | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (266623ad5339) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (bc556627a636) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (bc556627a636) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (6b5bcf998b9e) | alloc/A1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S2.json (6108b71f4991) | trend/T3.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (6108b71f4991) | trend/T3.json |
| E15 | growth-series | test | dla 0.1.0 | artifact series/S3.json (6b5bcf998b9e) | trend/T4.json |
| E16 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (6b5bcf998b9e) | trend/T4.json |
| E17 | histogram-diff | live | jcmd-class-histogram 0.1.0 | artifact histograms/H1.json (b647dd044a1b) | histograms/H1.json |
| E18 | growth-series | live | jcmd-class-histogram 0.1.0 | artifact series/S4.json (70c0404cb4e0) | series/S4.json |
| E19 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-183727-3c4f/harness/H1/out/p1-native/capture/86/heap-1.hprof (e60e13976ca7) | summaries/heap-1-e60e1397.json |
| E20 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-183727-3c4f/harness/H1/out/p4-native/capture/403/heap-1.hprof (99ce17f3f545) | summaries/heap-1-99ce17f3.json |
| E21 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-183727-3c4f/harness/H1/out/p6-native/capture/615/heap-1.hprof (03eb44d6c748) | summaries/heap-1-03eb44d6.json |
| E22 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-183727-3c4f/harness/H1/out/p9-native/capture/933/heap-1.hprof (65ea6598d610) | summaries/heap-1-65ea6598.json |
| E23 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S5.json (27995c36a17c) | series/S5.json |
| E24 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S5.json (27995c36a17c) | series/S5.json |
| E25 | histogram-diff | live | jcmd-class-histogram 0.1.0 | artifact histograms/H2.json (5cc20af6251c) | histograms/H2.json |
| E26 | growth-series | live | jcmd-class-histogram 0.1.0 | artifact series/S6.json (4897651c2bd8) | series/S6.json |
| E27 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (70c0404cb4e0) | trend/T5.json |
| E28 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (70c0404cb4e0) | trend/T5.json |
| E29 | growth-series | dump | dla 0.1.0 | artifact series/S5.json (27995c36a17c) | trend/T6.json |
| E30 | unit-count-series | dump | dla 0.1.0 | artifact series/S5.json (27995c36a17c) | trend/T6.json |
| E31 | growth-series | dump | dla 0.1.0 | artifact series/S6.json (4897651c2bd8) | trend/T7.json |
| E32 | unit-count-series | dump | dla 0.1.0 | artifact series/S6.json (4897651c2bd8) | trend/T7.json |
| E33 | intervention | test | dla 0.1.0 | artifact intervene/I1/patch.diff (cbad9bdec5df) | intervene/I1 |
| E34 | residual | test | dla 0.1.0 | artifact series/S1.json (bc556627a636) | gap/residual-S1-E34.json |
| E35 | residual | test | dla 0.1.0 | artifact series/S2.json (6108b71f4991) | gap/residual-S2-E35.json |
| E36 | residual | test | dla 0.1.0 | artifact series/S3.json (6b5bcf998b9e) | gap/residual-S3-E36.json |
