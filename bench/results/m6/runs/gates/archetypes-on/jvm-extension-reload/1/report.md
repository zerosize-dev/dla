# DLA report

Run: `20261110-163144-564f`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 8 enumerated, 8 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 6

Residual: heap 13509.262878787878 bytes/n unattributed of 13509.262878787878 total (threshold 3377.3157196969696) ABOVE THRESHOLD, see O1; heap 13369.817424242425 bytes/n unattributed of 13369.817424242425 total (threshold 3342.454356060606) ABOVE THRESHOLD, see O3; heap 13467.060991735538 bytes/n unattributed of 13467.060991735538 total (threshold 3366.7652479338844) ABOVE THRESHOLD, see O1; non-heap 6855.876363636364 bytes/n unattributed of 6855.876363636364 total (threshold 1713.969090909091) ABOVE THRESHOLD, see O2; non-heap 9178.880086580088 bytes/n unattributed of 9178.880086580088 total (threshold 2294.720021645022) ABOVE THRESHOLD, see O4; non-heap 8198.810101010102 bytes/n unattributed of 8198.810101010102 total (threshold 2049.7025252525254) ABOVE THRESHOLD, see O2

Coverage: jvm 3520 iterations, 19 of 19 entry points exercised

## Findings

### F3: **L-UNLOAD**

- State: confirmed
- Label: L-UNLOAD (evidence: E11 unit-count-series, E15 retention-path)
- Growth: `probe:reports.EventBus.LISTENERS` 1 count/n (steady, gate pass, E11)
- Evidence classes: context, discriminator, growth, static
- Holder: static/global
- Owner: `reports.EventBus.LISTENERS`
- Region: heap
- Shape: steady
- Site: `src/reports/EventBus.java:8`
- Writers: `src/reports/EventBus.java:13`, `ext/ledger/LedgerExtension.java:22`
- Removers: `src/reports/EventBus.java:17`, `ext/ledger/LedgerExtension.java:27`
- Annotation: heuristic name match over the project's sources
- Annotation: start() subscribes the Tally; stop() unsubscribes it, but ExtensionManager.reload() (src/reports/ExtensionManager.java:30-33) never calls stop() on the old extension
- Verified: no
- Sibling hypotheses: F3.n1
- Supporting evidence: E6, E9, E11, E12
- Explanation (observed): EventBus.LISTENERS (static registration list, EventBus.java:8) gains a Tally from LedgerExtension.java:22 per reload; removal at LedgerExtension.java:27 runs only when stop() is called, and ExtensionManager.reload never calls it, so old loaders stay pinned.
- Explanation structure: adds: ext/ledger/LedgerExtension.java:22; removal: conditional at ext/ledger/LedgerExtension.java:27; evidence: E19

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `ledger.LedgerExtension.ROUNDING`
- Region: heap
- Site: `ext/ledger/LedgerExtension.java:11`
- Reason: not_exercised
- Remedy: ROUNDING is a per-loader constant table (1 KB); only retained because F3 pins the loader. No independent growth.
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `ledger.LedgerExtension.start`
- Region: heap
- Site: `ext/ledger/LedgerExtension.java:22`
- Reason: not_exercised
- Remedy: Subscription site of F3's leak; same mechanism, tracked under F3.
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `reports.ExtensionManager.loader`
- Region: non-heap
- Shape: steady
- Site: `src/reports/ExtensionManager.java:23`
- Reason: not_exercised
- Remedy: Duplicate view of F3 (class-count growth from loaders pinned by EventBus.LISTENERS); no separate test, fix F3 and re-measure
- Supporting evidence: E11

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: retention
- Holder: static/global
- Owner: `dla.harness.DlaDump.FIELDS`
- Region: heap
- Reason: not_exercised
- Remedy: DLA harness instrumentation field, not project code.
- Supporting evidence: E18

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: retention
- Holder: static/global
- Owner: `dla.harness.DlaDump.SHALLOW`
- Region: heap
- Reason: not_exercised
- Remedy: DLA harness instrumentation field, not project code.
- Supporting evidence: E18

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `reports.EventBus.LISTENERS`
- Region: non-heap
- Shape: steady
- Site: `src/reports/EventBus.java:8`
- Reason: not_exercised
- Remedy: Duplicate view of F3 (class-count growth from loaders pinned by EventBus.LISTENERS); no separate test, fix F3 and re-measure
- Supporting evidence: E14, E23

## Ruled out

- **F3.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM: listener count is a warm-up fill that plateaus
  - Refuted if: listener count keeps growing over extra reload cycles and after purge
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
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: coverage capped at 4000 classes
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: points 1 to 9 come from S1; the extra points from S2
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: coverage capped at 4000 classes
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: points 1 to 9 come from S1; the extra points from S2
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E16: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E17: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E18: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E19: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E19: limitation: series: per-point; one process per point at n = 440, 1760, 2640, 3960 (harness captures; checked: HPROF version, identifier size)
- E20: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E21: limitation: loaders and classes are whole-runtime counts: they never tie to a finding
- E22: limitation: coverage capped at 4000 classes
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: coverage capped at 4000 classes
- E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E23: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E23: limitation: gate thresholds are provisional (not calibrated)
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
- E27: limitation: F3: growth evidence is from S1, not S3; not subtracted
- E27: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E27: limitation: threshold is provisional (not calibrated)
- E28: partial coverage (2 of 3 examined)
- E28: limitation: F3: growth evidence is from S1, not S2; not subtracted
- E28: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (fb0b4eb9ad68) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (98c63e6f0c35) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (98c63e6f0c35) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (d94385a0259b) | alloc/A1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S3.json (d94385a0259b) | trend/T3.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (d94385a0259b) | trend/T3.json |
| E15 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-163144-564f/harness/H1/out/p1-native/capture/105/heap-1.hprof (c3a8ebdb64c7) | summaries/heap-1-c3a8ebdb.json |
| E16 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-163144-564f/harness/H1/out/p4-native/capture/466/heap-1.hprof (e07b20318f13) | summaries/heap-1-e07b2031.json |
| E17 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-163144-564f/harness/H1/out/p6-native/capture/707/heap-1.hprof (a69a0766ede7) | summaries/heap-1-a69a0766.json |
| E18 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-163144-564f/harness/H1/out/p9-native/capture/1068/heap-1.hprof (0d27629a21da) | summaries/heap-1-0d27629a.json |
| E19 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S4.json (147ac45ce4ba) | series/S4.json |
| E20 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S4.json (147ac45ce4ba) | series/S4.json |
| E21 | non-heap | live | jvm-classloader-stats 0.1.0 | artifact nonheap/N1.json (a8ed6aea35c1) | nonheap/N1.json |
| E22 | growth-series | test | dla 0.1.0 | artifact series/S2.json (d8a4e1db3f73) | trend/T4.json |
| E23 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (d8a4e1db3f73) | trend/T4.json |
| E24 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (147ac45ce4ba) | trend/T5.json |
| E25 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (147ac45ce4ba) | trend/T5.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S1.json (98c63e6f0c35) | gap/residual-S1-E26.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S3.json (d94385a0259b) | gap/residual-S3-E27.json |
| E28 | residual | test | dla 0.1.0 | artifact series/S2.json (d8a4e1db3f73) | gap/residual-S2-E28.json |
