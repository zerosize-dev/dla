# DLA report

Run: `20261110-194436-70fc`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 4 enumerated, 4 dispositioned; confirmed 0, strong 0, refuted 3, suspected 0, unresolved 1

Residual: heap 13.600454545454546 bytes/n unattributed of 13.600454545454546 total (threshold 20.420023090909293); heap 23.87272727272727 bytes/n unattributed of 23.87272727272727 total (threshold 13.38961090909091) ABOVE THRESHOLD, see O2; heap 16.03818181818182 bytes/n unattributed of 16.03818181818182 total (threshold 8.355394472727209) ABOVE THRESHOLD, see O2; heap 16.461363636363636 bytes/n unattributed of 16.461363636363636 total (threshold 4.115340909090909) ABOVE THRESHOLD, see O1; heap 15.931818181818182 bytes/n unattributed of 15.931818181818182 total (threshold 3.9829545454545454) ABOVE THRESHOLD, see O1; heap 15.77909090909091 bytes/n unattributed of 15.77909090909091 total (threshold 10.996752272727274) ABOVE THRESHOLD, see O1; heap 9.933838383838385 bytes/n unattributed of 9.933838383838385 total (threshold 12.552904636363635); heap 18.25121212121212 bytes/n unattributed of 18.25121212121212 total (threshold 4.56280303030303) ABOVE THRESHOLD, see O1; heap 13.50939393939394 bytes/n unattributed of 13.50939393939394 total (threshold 6.105885927272821) ABOVE THRESHOLD, see O1; non-heap 142.11818181818182 bytes/n unattributed of 142.11818181818182 total (threshold 335.19082909090906); non-heap 28.07272727272727 bytes/n unattributed of 28.07272727272727 total (threshold 22.68378) ABOVE THRESHOLD, see O3; non-heap 38.347575757575754 bytes/n unattributed of 38.347575757575754 total (threshold 61.379684927273175); non-heap 0 bytes/n unattributed of -17.153030303030302 total (threshold 108.8011786363632); non-heap 43.806277056277054 bytes/n unattributed of 43.806277056277054 total (threshold 96.89994409090856); non-heap 29.79090909090909 bytes/n unattributed of 29.79090909090909 total (threshold 90.44341363636364); non-heap 29.79090909090909 bytes/n unattributed of 29.79090909090909 total (threshold 65.99322163636364); non-heap 0 bytes/n unattributed of -3.170779220779221 total (threshold 48.854214350648775); non-heap 37.002272727272725 bytes/n unattributed of 37.002272727272725 total (threshold 75.69380604545454)

Coverage: jvm 3520 iterations, 19 of 19 entry points exercised

## Findings

### F4: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: JDK-internal java.lang.invoke.MethodType.internTable grew across short harness points (warm-up); re-measure with a longer reload series to confirm it plateaus
- Supporting evidence: E26

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E25
  - Ruled out by: plateau+purge (non-leak) E25
  - Contradicting evidence: E25
  - Originally supported by: E6
- **F2** (UNCLASSIFIED): discriminator E24
  - Ruled out by: plateau+purge (non-leak) E24
  - Contradicting evidence: E24
  - Originally supported by: E6
- **F3** (UNCLASSIFIED): discriminator E23
  - Ruled out by: plateau+purge (non-leak) E23
  - Contradicting evidence: E23
  - Originally supported by: E6

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
- E9: limitation: --capture alloc is not written by jvm harnesses
- E9: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E9: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E9: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E10: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E10: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E10: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E11: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E13: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E14: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E15: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E16: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E16: limitation: series: per-point; one process per point at n = 440, 1760, 2640, 3960 (harness captures; checked: HPROF version, identifier size)
- E17: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E18: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E19: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E20: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E21: limitation: loaders and classes are whole-runtime counts: they never tie to a finding
- E22: limitation: loaders and classes are whole-runtime counts: they never tie to a finding
- E22: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E23: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E23: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E23: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E23: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E23: limitation: points 1 to 9 come from S1; the extra points from S4
- E23: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E23: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E23: limitation: gate and plateau thresholds are provisional (not calibrated)
- E24: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E24: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E24: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E24: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E24: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E24: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E24: limitation: points 1 to 9 come from S1; the extra points from S6
- E24: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E24: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E24: limitation: gate and plateau thresholds are provisional (not calibrated)
- E25: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E25: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E25: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E25: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E25: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E25: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E25: limitation: points 1 to 9 come from S1; the extra points from S8
- E25: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E25: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E25: limitation: gate and plateau thresholds are provisional (not calibrated)
- E26: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap dump
- E26: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E26: limitation: holders absent from a dump's 200 largest have no figure at that point
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap dump
- E27: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E27: limitation: holders absent from a dump's 200 largest have no figure at that point
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: x is the snapshot number in capture order; t_ms is the time since the first capture where the inputs record one
- E28: limitation: each point comes from a separate snapshot of one process; a figure absent from a snapshot has no value at that point
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E29: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E29: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E29: limitation: gate thresholds are provisional (not calibrated)
- E30: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E30: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E30: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E30: limitation: gate thresholds are provisional (not calibrated)
- E31: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E31: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E31: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E31: limitation: gate thresholds are provisional (not calibrated)
- E32: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E32: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E32: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E32: limitation: gate thresholds are provisional (not calibrated)
- E33: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E33: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E33: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E33: limitation: gate thresholds are provisional (not calibrated)
- E34: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E34: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E34: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E34: limitation: gate thresholds are provisional (not calibrated)
- E35: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E35: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E35: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E35: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E35: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E35: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E35: limitation: points 1 to 9 come from S1; the extra points from S10
- E35: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E35: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E35: limitation: gate and plateau thresholds are provisional (not calibrated)
- E36: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E36: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E36: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E36: limitation: gate thresholds are provisional (not calibrated)
- E37: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E37: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E37: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E37: limitation: gate thresholds are provisional (not calibrated)
- E38: partial coverage (2 of 3 examined)
- E38: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E38: limitation: threshold is provisional (not calibrated)
- E39: partial coverage (2 of 3 examined)
- E39: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E39: limitation: threshold is provisional (not calibrated)
- E40: partial coverage (2 of 3 examined)
- E40: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E40: limitation: threshold is provisional (not calibrated)
- E41: partial coverage (2 of 3 examined)
- E41: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E41: limitation: threshold is provisional (not calibrated)
- E42: partial coverage (2 of 3 examined)
- E42: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E42: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E42: limitation: threshold is provisional (not calibrated)
- E43: partial coverage (2 of 3 examined)
- E43: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E43: limitation: threshold is provisional (not calibrated)
- E44: partial coverage (2 of 3 examined)
- E44: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E44: limitation: threshold is provisional (not calibrated)
- E45: partial coverage (2 of 3 examined)
- E45: limitation: off-heap: no gap series for S11; run dla gap S11 first (node.external is used when present)
- E45: limitation: threshold is provisional (not calibrated)
- E46: partial coverage (2 of 3 examined)
- E46: limitation: off-heap: no gap series for S10; run dla gap S10 first (node.external is used when present)
- E46: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | — |
| E2 | run-config | static | dla 0.1.0 | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | probe/probe.json |
| E4 | static-hit | static | spotbugs unknown | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | sweep/jvm-spotbugs.json |
| E5 | static-hit | static | dla 0.1.0 | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | sweep/jvm-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | census/jvm.json |
| E7 | workload | static | dla 0.1.0 | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (68185f8764d5) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (b53a92327130) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (b53a92327130) | trend/T1.json |
| E12 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-194436-70fc/harness/H1/out/p1-native/capture/109/heap-1.hprof (59d675cc4624) | summaries/heap-1-59d675cc.json |
| E13 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-194436-70fc/harness/H1/out/p4-native/capture/478/heap-1.hprof (38b3243238ec) | summaries/heap-1-38b32432.json |
| E14 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-194436-70fc/harness/H1/out/p6-native/capture/727/heap-1.hprof (02e1e3a77262) | summaries/heap-1-02e1e3a7.json |
| E15 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-194436-70fc/harness/H1/out/p9-native/capture/1096/heap-1.hprof (3e0f13958e47) | summaries/heap-1-3e0f1395.json |
| E16 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S2.json (73dc08768d41) | series/S2.json |
| E17 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S2.json (73dc08768d41) | series/S2.json |
| E18 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S2.json (73dc08768d41) | series/S2.json |
| E19 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S2.json (73dc08768d41) | series/S2.json |
| E20 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S2.json (73dc08768d41) | series/S2.json |
| E21 | non-heap | live | jvm-classloader-stats 0.1.0 | artifact nonheap/N1.json (8a89a82659f4) | nonheap/N1.json |
| E22 | growth-series | live | jvm-classloader-stats 0.1.0 | artifact series/S3.json (aa94401636ee) | series/S3.json |
| E23 | discriminator | test | dla 0.1.0 | artifact series/S5.json (92c72d55e9f9) | alloc/A1.json |
| E24 | discriminator | test | dla 0.1.0 | artifact series/S7.json (897edf5fa552) | alloc/A2.json |
| E25 | discriminator | test | dla 0.1.0 | artifact series/S9.json (a06067c6054a) | alloc/A3.json |
| E26 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (73dc08768d41) | trend/T5.json |
| E27 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (73dc08768d41) | trend/T5.json |
| E28 | unit-count-series | dump | dla 0.1.0 | artifact series/S3.json (aa94401636ee) | trend/T6.json |
| E29 | growth-series | test | dla 0.1.0 | artifact series/S4.json (7b8d489fce62) | trend/T7.json |
| E30 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (7b8d489fce62) | trend/T7.json |
| E31 | growth-series | test | dla 0.1.0 | artifact series/S6.json (970387ef2c83) | trend/T8.json |
| E32 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (970387ef2c83) | trend/T8.json |
| E33 | growth-series | test | dla 0.1.0 | artifact series/S8.json (c814367b66de) | trend/T9.json |
| E34 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (c814367b66de) | trend/T9.json |
| E35 | discriminator | test | dla 0.1.0 | artifact series/S11.json (66f4f51f0579) | alloc/A4.json |
| E36 | growth-series | test | dla 0.1.0 | artifact series/S10.json (00eaaec1aef1) | trend/T11.json |
| E37 | unit-count-series | test | dla 0.1.0 | artifact series/S10.json (00eaaec1aef1) | trend/T11.json |
| E38 | residual | test | dla 0.1.0 | artifact series/S1.json (b53a92327130) | gap/residual-S1-E38.json |
| E39 | residual | test | dla 0.1.0 | artifact series/S4.json (7b8d489fce62) | gap/residual-S4-E39.json |
| E40 | residual | test | dla 0.1.0 | artifact series/S5.json (92c72d55e9f9) | gap/residual-S5-E40.json |
| E41 | residual | test | dla 0.1.0 | artifact series/S6.json (970387ef2c83) | gap/residual-S6-E41.json |
| E42 | residual | test | dla 0.1.0 | artifact series/S7.json (897edf5fa552) | gap/residual-S7-E42.json |
| E43 | residual | test | dla 0.1.0 | artifact series/S8.json (c814367b66de) | gap/residual-S8-E43.json |
| E44 | residual | test | dla 0.1.0 | artifact series/S9.json (a06067c6054a) | gap/residual-S9-E44.json |
| E45 | residual | test | dla 0.1.0 | artifact series/S11.json (66f4f51f0579) | gap/residual-S11-E45.json |
| E46 | residual | test | dla 0.1.0 | artifact series/S10.json (00eaaec1aef1) | gap/residual-S10-E46.json |
