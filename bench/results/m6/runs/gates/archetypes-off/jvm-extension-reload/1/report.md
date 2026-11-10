# DLA report

Run: `20261110-182822-61d7`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 8 enumerated, 8 dispositioned; confirmed 1, strong 0, refuted 1, suspected 2, unresolved 4

Residual: heap 13521.43787878788 bytes/n unattributed of 13521.43787878788 total (threshold 3380.35946969697) ABOVE THRESHOLD, see O1; heap 13458.85909090909 bytes/n unattributed of 13458.85909090909 total (threshold 3364.7147727272727) ABOVE THRESHOLD, see O1; heap 13482.035064935066 bytes/n unattributed of 13482.035064935066 total (threshold 3370.5087662337664) ABOVE THRESHOLD, see O1; non-heap 6994.287272727273 bytes/n unattributed of 6994.287272727273 total (threshold 1748.5718181818183) ABOVE THRESHOLD, see O2; non-heap 9203.763636363636 bytes/n unattributed of 9203.763636363636 total (threshold 2300.94090909091) ABOVE THRESHOLD, see O2; non-heap 8084.36606060606 bytes/n unattributed of 8084.36606060606 total (threshold 2021.091515151515) ABOVE THRESHOLD, see O2

Coverage: jvm 3520 iterations, 19 of 19 entry points exercised

## Findings

### F3: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (EventBus.LISTENERS static list holds the Tally listener registered by each LedgerExtension instance; ExtensionManager.reload() drops loader/current without calling stop(), so unsubscribe never runs and each listener pins its URLClassLoader, classes and statics (metaspace + heap))
- Growth: `probe:reports.EventBus.LISTENERS` 1 count/n (steady, gate pass, E11)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `reports.EventBus.LISTENERS`
- Region: heap
- Shape: steady
- Site: `src/reports/EventBus.java:8`
- Writers: `src/reports/EventBus.java:13`
- Removers: `src/reports/EventBus.java:17`
- Annotation: heuristic name match over the project's sources
- Annotation: LISTENERS add in subscribe, remove in unsubscribe; reload() never calls stop()
- Verified: no
- Sibling hypotheses: F3.n1
- Supporting evidence: E6, E9, E11
- Explanation (observed): static EventBus.LISTENERS (EventBus.java:8) grows by one Tally per reload: LedgerExtension.start subscribes (EventBus.java:13); the remover unsubscribe (EventBus.java:17) runs only when stop is called, which reload never does, so each Tally pins its classloader.
- Explanation structure: adds: src/reports/EventBus.java:13; removal: conditional at src/reports/EventBus.java:17; evidence: E21

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `ledger.LedgerExtension.ROUNDING`
- Region: heap
- Site: `ext/ledger/LedgerExtension.java:11`
- Reason: not_exercised
- Remedy: static ROUNDING table is a fixed 1024-entry array per loader; its footprint scales with retained loaders (F3), not independently; no growth column of its own
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `ledger.LedgerExtension.start`
- Region: heap
- Site: `ext/ledger/LedgerExtension.java:22`
- Writers: `ext/ledger/LedgerExtension.java:22`
- Removers: `ext/ledger/LedgerExtension.java:27`
- Reason: not_exercised
- Remedy: LedgerExtension.start subscribe site is the writer of F3; same mechanism, tracked under F3
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Holder: registration-table
- Owner: `reports.ExtensionManager.loader`
- Region: non-heap
- Shape: steady
- Site: `src/reports/ExtensionManager.java:26`
- Annotation: x
- Reason: time_budget
- Remedy: JDK class-count and metaspace columns are the per-loader footprint of the listeners pinned by F3; re-run after fixing F3 (call stop() in reload) to confirm they go flat
- Supporting evidence: E11

### F5: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Holder: registration-table
- Owner: `reports.ExtensionManager.loader`
- Region: non-heap
- Shape: steady
- Site: `src/reports/ExtensionManager.java:26`
- Reason: time_budget
- Remedy: same as F4 (alloc-test series S2/S3); re-run after fixing F3
- Supporting evidence: E14, E16

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: retention
- Holder: static/global
- Owner: `dla.harness.DlaDump.FIELDS`
- Region: heap
- Reason: not_exercised
- Remedy: dla.harness.DlaDump.FIELDS is DLA's own instrument (N-OBSERVER), not project code
- Supporting evidence: E20

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: retention
- Holder: static/global
- Owner: `dla.harness.DlaDump.SHALLOW`
- Region: heap
- Reason: not_exercised
- Remedy: dla.harness.DlaDump.SHALLOW is DLA's own instrument (N-OBSERVER), not project code
- Supporting evidence: E20

## Ruled out

- **F3.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM: listener count is a warm-up fill that plateaus
  - Refuted if: listener count and loaded classes keep growing over extra cycles after purge
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
- E9: limitation: coverage capped at 4000 classes
- E10: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E10: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E10: limitation: coverage capped at 4000 classes
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E11: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E11: limitation: coverage capped at 4000 classes
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: coverage capped at 4000 classes
- E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: points 1 to 9 come from S1; the extra points from S2
- E12: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E12: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E12: limitation: gate and plateau thresholds are provisional (not calibrated)
- E13: limitation: coverage capped at 4000 classes
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: coverage capped at 4000 classes
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: coverage capped at 4000 classes
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: points 1 to 9 come from S1; the extra points from S2
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: coverage capped at 4000 classes
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: points 1 to 9 come from S1; the extra points from S2
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E18: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E19: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E20: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E21: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E21: limitation: series: per-point; one process per point at n = 440, 1760, 2640, 3960 (harness captures; checked: HPROF version, identifier size)
- E22: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E23: limitation: tested in a scratch copy; the project was not changed
- E23: limitation: gate thresholds are provisional (not calibrated); the 0.2 and 0.5 slope fractions are placeholders
- E23: limitation: exercised is null: the harness records entry points, not lines, so whether the edited line ran is not checked
- E23: limitation: every measured process of mode native exited with a non-zero code (p1-native: 1, p2-native: 1, p3-native: 1, p4-native: 1): the harness itself failed, so no series was written
- E24: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap dump
- E24: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E24: limitation: holders absent from a dump's 200 largest have no figure at that point
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap dump
- E25: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E25: limitation: holders absent from a dump's 200 largest have no figure at that point
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: partial coverage (2 of 3 examined)
- E26: limitation: F3: growth measured as a count; bytes not attributed
- E26: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E26: limitation: threshold is provisional (not calibrated)
- E27: partial coverage (2 of 3 examined)
- E27: limitation: F3: growth evidence is from S1, not S2; not subtracted
- E27: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E27: limitation: threshold is provisional (not calibrated)
- E28: partial coverage (2 of 3 examined)
- E28: limitation: F3: growth evidence is from S1, not S3; not subtracted
- E28: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E28: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | — |
| E2 | run-config | static | dla 0.1.0 | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | probe/probe.json |
| E4 | static-hit | static | spotbugs unknown | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | sweep/jvm-spotbugs.json |
| E5 | static-hit | static | dla 0.1.0 | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | sweep/jvm-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | census/jvm.json |
| E7 | workload | static | dla 0.1.0 | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (d9a80f9dbb94) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (9c6925ef37fd) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (9c6925ef37fd) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (f514e3242290) | alloc/A1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S2.json (cbf02ebb4414) | trend/T3.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (cbf02ebb4414) | trend/T3.json |
| E15 | growth-series | test | dla 0.1.0 | artifact series/S3.json (f514e3242290) | trend/T4.json |
| E16 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (f514e3242290) | trend/T4.json |
| E17 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-182822-61d7/harness/H1/out/p1-native/capture/117/heap-1.hprof (522d94d91078) | summaries/heap-1-522d94d9.json |
| E18 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-182822-61d7/harness/H1/out/p4-native/capture/478/heap-1.hprof (3162051d61ea) | summaries/heap-1-3162051d.json |
| E19 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-182822-61d7/harness/H1/out/p6-native/capture/719/heap-1.hprof (af1ab6643b3a) | summaries/heap-1-af1ab664.json |
| E20 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-182822-61d7/harness/H1/out/p9-native/capture/1081/heap-1.hprof (1e015e6ce186) | summaries/heap-1-1e015e6c.json |
| E21 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S4.json (c30a904ba416) | series/S4.json |
| E22 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S4.json (c30a904ba416) | series/S4.json |
| E23 | intervention | test | dla 0.1.0 | artifact intervene/I1/patch.diff (85e68bf6f129) | intervene/I1 |
| E24 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (c30a904ba416) | trend/T5.json |
| E25 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (c30a904ba416) | trend/T5.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S1.json (9c6925ef37fd) | gap/residual-S1-E26.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S2.json (cbf02ebb4414) | gap/residual-S2-E27.json |
| E28 | residual | test | dla 0.1.0 | artifact series/S3.json (f514e3242290) | gap/residual-S3-E28.json |
