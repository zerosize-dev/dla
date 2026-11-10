# DLA report

Run: `20261110-170537-bb03`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 13 enumerated, 13 dispositioned; confirmed 1, strong 0, refuted 1, suspected 2, unresolved 9

Residual: heap 13533.95303030303 bytes/n unattributed of 13533.95303030303 total (threshold 3383.4882575757574) ABOVE THRESHOLD, see O1; heap 13486.103030303031 bytes/n unattributed of 13486.103030303031 total (threshold 3371.525757575758) ABOVE THRESHOLD, see O3; heap 13488.430909090908 bytes/n unattributed of 13488.430909090908 total (threshold 3372.107727272727) ABOVE THRESHOLD, see O3; non-heap 6969.561038961039 bytes/n unattributed of 6969.561038961039 total (threshold 1742.3902597402598) ABOVE THRESHOLD, see O2; non-heap 9153.483333333334 bytes/n unattributed of 9153.483333333334 total (threshold 2288.3708333333334) ABOVE THRESHOLD, see O4; non-heap 8168.927272727273 bytes/n unattributed of 8168.927272727273 total (threshold 2042.2318181818184) ABOVE THRESHOLD, see O4

Coverage: jvm 3520 iterations, 19 of 19 entry points exercised

## Findings

### F3: **L-REGISTER**

- State: confirmed
- Label: L-REGISTER (evidence: E11 unit-count-series)
- Growth: `probe:reports.EventBus.LISTENERS` 1 count/n (steady, gate pass, E11)
- Evidence classes: context, discriminator, growth, static
- Holder: registration-table
- Owner: `reports.EventBus.LISTENERS`
- Region: heap
- Shape: steady
- Site: `src/reports/EventBus.java:8`
- Writers: `src/reports/EventBus.java:13`, `ext/ledger/LedgerExtension.java:22`
- Removers: `src/reports/EventBus.java:17`, `src/reports/ExtensionManager.java:33`
- Annotation: heuristic name match over the project's sources
- Annotation: reload() drops loader/current without calling stop(), so unsubscribe at LedgerExtension.java:27 never runs
- Verified: no
- Sibling hypotheses: F3.n1
- Supporting evidence: E6, E9, E11, E12
- Explanation (inferred): Static registration-table EventBus.java:8 (LISTENERS) gets a Tally listener per start() at LedgerExtension.java:22; stop() unsubscribes, but reload() at src/reports/ExtensionManager.java:33 drops the loader unless stop() is called, which it never is, so each old URLClassLoader stays reachable.
- Explanation structure: adds: ext/ledger/LedgerExtension.java:22; removal: conditional at src/reports/ExtensionManager.java:33

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `ledger.LedgerExtension.ROUNDING`
- Region: heap
- Site: `ext/ledger/LedgerExtension.java:11`
- Reason: not_exercised
- Remedy: Census/harness-internal candidate; no growth tied of its own; per-loader copies are consequences of F3
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `ledger.LedgerExtension.start`
- Region: heap
- Site: `ext/ledger/LedgerExtension.java:22`
- Reason: not_exercised
- Remedy: Census/harness-internal candidate; no growth tied of its own; per-loader copies are consequences of F3
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: retention
- Holder: static/global
- Owner: `dla.harness.DlaDump.FIELDS`
- Region: heap
- Reason: not_exercised
- Remedy: DLA harness-internal static (DlaDump), not project code; instrument artifact, ignore
- Supporting evidence: E16

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: retention
- Holder: static/global
- Owner: `dla.harness.DlaDump.SHALLOW`
- Region: heap
- Reason: not_exercised
- Remedy: DLA harness-internal static (DlaDump), not project code; instrument artifact, ignore
- Supporting evidence: E16

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `reports.EventBus.LISTENERS`
- Region: heap
- Shape: steady
- Site: `src/reports/EventBus.java:8`
- Reason: not_exercised
- Remedy: Same growth as F3 (leaked URLClassLoaders held via EventBus.LISTENERS); duplicate registered to cover trend columns; fixing F3 removes it
- Supporting evidence: E28

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `reports.EventBus.LISTENERS`
- Region: heap
- Shape: steady
- Site: `src/reports/EventBus.java:8`
- Reason: not_exercised
- Remedy: Same growth as F3 (leaked URLClassLoaders held via EventBus.LISTENERS); duplicate registered to cover trend columns; fixing F3 removes it
- Supporting evidence: E30

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `reports.EventBus.LISTENERS`
- Region: heap
- Shape: steady
- Site: `src/reports/EventBus.java:8`
- Reason: not_exercised
- Remedy: Same growth as F3 (leaked URLClassLoaders held via EventBus.LISTENERS); duplicate registered to cover trend columns; fixing F3 removes it
- Supporting evidence: E43

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `reports.EventBus.LISTENERS`
- Region: heap
- Shape: steady
- Site: `src/reports/EventBus.java:8`
- Reason: not_exercised
- Remedy: Same growth as F3 (leaked URLClassLoaders held via EventBus.LISTENERS); duplicate registered to cover trend columns; fixing F3 removes it
- Supporting evidence: E45

### F10: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `reports.EventBus.LISTENERS`
- Region: heap
- Shape: steady
- Site: `src/reports/EventBus.java:8`
- Reason: not_exercised
- Remedy: Same growth as F3 (leaked URLClassLoaders held via EventBus.LISTENERS); duplicate registered to cover trend columns; fixing F3 removes it
- Supporting evidence: E42

### F11: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `reports.EventBus.LISTENERS`
- Region: heap
- Shape: steady
- Site: `src/reports/EventBus.java:8`
- Reason: not_exercised
- Remedy: Same growth as F3 (leaked URLClassLoaders held via EventBus.LISTENERS); duplicate registered to cover trend columns; fixing F3 removes it
- Supporting evidence: E44

### F12: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `reports.EventBus.LISTENERS`
- Region: heap
- Shape: steady
- Site: `src/reports/EventBus.java:8`
- Reason: not_exercised
- Remedy: Same growth as F3 (leaked URLClassLoaders held via EventBus.LISTENERS); duplicate registered to cover trend columns; fixing F3 removes it
- Supporting evidence: E46

## Ruled out

- **F3.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM: listener count is a warm-up fill that plateaus
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
- E13: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E14: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E15: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E16: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E17: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E17: limitation: series: per-point; one process per point at n = 440, 1760, 2640, 3960 (harness captures; checked: HPROF version, identifier size)
- E18: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E19: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E20: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E21: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E22: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E23: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E24: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E25: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E25: limitation: series: per-point; one process per point at n = 440, 1760, 2640, 3960 (harness captures; checked: HPROF version, identifier size)
- E26: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E27: limitation: coverage capped at 4000 classes
- E27: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E27: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: coverage capped at 4000 classes
- E28: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E28: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E29: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E29: limitation: coverage capped at 4000 classes
- E29: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E29: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E29: limitation: points 1 to 9 come from S1; the extra points from S2
- E29: limitation: gate thresholds are provisional (not calibrated)
- E30: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E30: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E30: limitation: coverage capped at 4000 classes
- E30: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E30: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E30: limitation: points 1 to 9 come from S1; the extra points from S2
- E30: limitation: gate thresholds are provisional (not calibrated)
- E31: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap dump
- E31: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E31: limitation: holders absent from a dump's 200 largest have no figure at that point
- E31: limitation: gate thresholds are provisional (not calibrated)
- E32: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap dump
- E32: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E32: limitation: holders absent from a dump's 200 largest have no figure at that point
- E32: limitation: gate thresholds are provisional (not calibrated)
- E33: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap dump
- E33: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E33: limitation: holders absent from a dump's 200 largest have no figure at that point
- E33: limitation: gate thresholds are provisional (not calibrated)
- E34: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap dump
- E34: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E34: limitation: holders absent from a dump's 200 largest have no figure at that point
- E34: limitation: gate thresholds are provisional (not calibrated)
- E35: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E35: limitation: the files carry no time, so they are ordered as given
- E36: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E36: limitation: the files carry no time, so they are ordered as given
- E36: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E37: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E37: limitation: the files carry no time, so they are ordered as given
- E38: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E38: limitation: the files carry no time, so they are ordered as given
- E38: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E39: limitation: loaders and classes are whole-runtime counts: they never tie to a finding
- E40: limitation: loaders and classes are whole-runtime counts: they never tie to a finding
- E40: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E41: partial coverage (2 of 3 examined)
- E41: limitation: F3: growth evidence is from S1, not S3; not subtracted
- E41: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E41: limitation: threshold is provisional (not calibrated)
- E42: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E42: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E42: limitation: gate thresholds are provisional (not calibrated)
- E43: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E43: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E43: limitation: gate thresholds are provisional (not calibrated)
- E44: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E44: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E44: limitation: gate thresholds are provisional (not calibrated)
- E45: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E45: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E45: limitation: gate thresholds are provisional (not calibrated)
- E46: limitation: x is the snapshot number in capture order; t_ms is the time since the first capture where the inputs record one
- E46: limitation: each point comes from a separate snapshot of one process; a figure absent from a snapshot has no value at that point
- E46: limitation: gate thresholds are provisional (not calibrated)
- E47: limitation: tested in a scratch copy; the project was not changed
- E47: limitation: gate thresholds are provisional (not calibrated); the 0.2 and 0.5 slope fractions are placeholders
- E47: limitation: exercised is null: the harness records entry points, not lines, so whether the edited line ran is not checked
- E47: limitation: every measured process of mode native exited with a non-zero code (p1-native: 1, p2-native: 1, p3-native: 1, p4-native: 1): the harness itself failed, so no series was written
- E48: partial coverage (2 of 3 examined)
- E48: limitation: F3: growth measured as a count; bytes not attributed
- E48: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E48: limitation: threshold is provisional (not calibrated)
- E49: partial coverage (2 of 3 examined)
- E49: limitation: F3: growth evidence is from S1, not S2; not subtracted
- E49: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E49: limitation: threshold is provisional (not calibrated)
- E50: partial coverage (2 of 3 examined)
- E50: limitation: F3: growth evidence is from S1, not S3; not subtracted
- E50: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E50: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | — |
| E2 | run-config | static | dla 0.1.0 | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | probe/probe.json |
| E4 | static-hit | static | spotbugs unknown | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | sweep/jvm-spotbugs.json |
| E5 | static-hit | static | dla 0.1.0 | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | sweep/jvm-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | census/jvm.json |
| E7 | workload | static | dla 0.1.0 | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (594f96825f89) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (32427ed50e81) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (32427ed50e81) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (3ca042b8a480) | alloc/A1.json |
| E13 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-170537-bb03/harness/H1/out/p1-native/capture/106/heap-1.hprof (3bb198468148) | summaries/heap-1-3bb19846.json |
| E14 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-170537-bb03/harness/H1/out/p4-native/capture/467/heap-1.hprof (643e87ae2165) | summaries/heap-1-643e87ae.json |
| E15 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-170537-bb03/harness/H1/out/p6-native/capture/709/heap-1.hprof (f5b6988f7b7d) | summaries/heap-1-f5b6988f.json |
| E16 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-170537-bb03/harness/H1/out/p9-native/capture/1072/heap-1.hprof (0f2d791ade0a) | summaries/heap-1-0f2d791a.json |
| E17 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S4.json (0224932a8697) | series/S4.json |
| E18 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S4.json (0224932a8697) | series/S4.json |
| E19 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S4.json (0224932a8697) | series/S4.json |
| E20 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S4.json (0224932a8697) | series/S4.json |
| E21 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-170537-bb03/harness/H1/out/p1-native/capture/106/heap-1.hprof (3bb198468148) | summaries/heap-1-3bb19846.json |
| E22 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-170537-bb03/harness/H1/out/p4-native/capture/467/heap-1.hprof (643e87ae2165) | summaries/heap-1-643e87ae.json |
| E23 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-170537-bb03/harness/H1/out/p6-native/capture/709/heap-1.hprof (f5b6988f7b7d) | summaries/heap-1-f5b6988f.json |
| E24 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-170537-bb03/harness/H1/out/p9-native/capture/1072/heap-1.hprof (0f2d791ade0a) | summaries/heap-1-0f2d791a.json |
| E25 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S5.json (96ae9477c74a) | series/S5.json |
| E26 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S5.json (96ae9477c74a) | series/S5.json |
| E27 | growth-series | test | dla 0.1.0 | artifact series/S2.json (915701321962) | trend/T3.json |
| E28 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (915701321962) | trend/T3.json |
| E29 | growth-series | test | dla 0.1.0 | artifact series/S3.json (3ca042b8a480) | trend/T4.json |
| E30 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (3ca042b8a480) | trend/T4.json |
| E31 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (0224932a8697) | trend/T5.json |
| E32 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (0224932a8697) | trend/T5.json |
| E33 | growth-series | dump | dla 0.1.0 | artifact series/S5.json (96ae9477c74a) | trend/T6.json |
| E34 | unit-count-series | dump | dla 0.1.0 | artifact series/S5.json (96ae9477c74a) | trend/T6.json |
| E35 | histogram-diff | live | jcmd-class-histogram 0.1.0 | artifact histograms/H1.json (9c64221a549b) | histograms/H1.json |
| E36 | growth-series | live | jcmd-class-histogram 0.1.0 | artifact series/S6.json (978b4a3a1c58) | series/S6.json |
| E37 | histogram-diff | live | jcmd-class-histogram 0.1.0 | artifact histograms/H2.json (c50bb1b5de3f) | histograms/H2.json |
| E38 | growth-series | live | jcmd-class-histogram 0.1.0 | artifact series/S7.json (4ed7a3344343) | series/S7.json |
| E39 | non-heap | live | jvm-classloader-stats 0.1.0 | artifact nonheap/N1.json (76df9e1bc54a) | nonheap/N1.json |
| E40 | growth-series | live | jvm-classloader-stats 0.1.0 | artifact series/S8.json (3d5ff11e426d) | series/S8.json |
| E41 | residual | test | dla 0.1.0 | artifact series/S3.json (3ca042b8a480) | gap/residual-S3-E41.json |
| E42 | growth-series | dump | dla 0.1.0 | artifact series/S6.json (978b4a3a1c58) | trend/T7.json |
| E43 | unit-count-series | dump | dla 0.1.0 | artifact series/S6.json (978b4a3a1c58) | trend/T7.json |
| E44 | growth-series | dump | dla 0.1.0 | artifact series/S7.json (4ed7a3344343) | trend/T8.json |
| E45 | unit-count-series | dump | dla 0.1.0 | artifact series/S7.json (4ed7a3344343) | trend/T8.json |
| E46 | unit-count-series | dump | dla 0.1.0 | artifact series/S8.json (3d5ff11e426d) | trend/T9.json |
| E47 | intervention | test | dla 0.1.0 | artifact intervene/I1/patch.diff (2625627da0c3) | intervene/I1 |
| E48 | residual | test | dla 0.1.0 | artifact series/S1.json (32427ed50e81) | gap/residual-S1-E48.json |
| E49 | residual | test | dla 0.1.0 | artifact series/S2.json (915701321962) | gap/residual-S2-E49.json |
| E50 | residual | test | dla 0.1.0 | artifact series/S3.json (3ca042b8a480) | gap/residual-S3-E50.json |
