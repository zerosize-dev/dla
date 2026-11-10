# DLA report

Run: `20261110-193858-9466`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 11 enumerated, 11 dispositioned; confirmed 3, strong 0, refuted 3, suspected 0, unresolved 5

Residual: heap 13545.717662337662 bytes/n unattributed of 13545.717662337662 total (threshold 3386.429415584415) ABOVE THRESHOLD, see O1; heap 13546.520454545454 bytes/n unattributed of 13546.520454545454 total (threshold 3386.6301136363636) ABOVE THRESHOLD, see O1; heap 13499.06063936064 bytes/n unattributed of 13499.06063936064 total (threshold 3374.76515984016) ABOVE THRESHOLD, see O1; heap 13536.521212121212 bytes/n unattributed of 13536.521212121212 total (threshold 3384.1303030303034) ABOVE THRESHOLD, see O1; heap 13512.488636363636 bytes/n unattributed of 13512.488636363636 total (threshold 3378.122159090909) ABOVE THRESHOLD, see O1; heap 13425.478181818182 bytes/n unattributed of 13425.478181818182 total (threshold 3356.3695454545455) ABOVE THRESHOLD, see O1; heap 13473.520104895106 bytes/n unattributed of 13473.520104895106 total (threshold 3368.3800262237764) ABOVE THRESHOLD, see O1; non-heap 6928.914285714285 bytes/n unattributed of 6928.914285714285 total (threshold 1732.2285714285713) ABOVE THRESHOLD, see O2; non-heap 9181.192467532468 bytes/n unattributed of 9181.192467532468 total (threshold 2295.298116883117) ABOVE THRESHOLD, see O2; non-heap 8125.897922077922 bytes/n unattributed of 8125.897922077922 total (threshold 2031.4744805194805) ABOVE THRESHOLD, see O2; non-heap 9058.307792207792 bytes/n unattributed of 9058.307792207792 total (threshold 2264.576948051948) ABOVE THRESHOLD, see O2; non-heap 8094.892045454546 bytes/n unattributed of 8094.892045454546 total (threshold 2023.7230113636365) ABOVE THRESHOLD, see O2; non-heap 9216.57196969697 bytes/n unattributed of 9216.57196969697 total (threshold 2304.1429924242425) ABOVE THRESHOLD, see O2; non-heap 8125.612121212122 bytes/n unattributed of 8125.612121212122 total (threshold 2031.4030303030304) ABOVE THRESHOLD, see O2

Coverage: jvm 3520 iterations, 19 of 19 entry points exercised

## Findings

### F3: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (static EventBus.LISTENERS keeps listener of each reloaded extension; reload() never calls stop(), so each URLClassLoader and its classes (metaspace) stay reachable)
- Growth: `probe:reports.EventBus.LISTENERS` 1 count/n (steady, gate pass, E11)
- Evidence classes: context, growth, static
- Holder: registration-table
- Owner: `reports.EventBus.LISTENERS`
- Region: heap
- Shape: steady
- Site: `src/reports/EventBus.java:8`
- Writers: `src/reports/EventBus.java:13`, `ext/ledger/LedgerExtension.java:22`
- Removers: `src/reports/EventBus.java:17`
- Annotation: heuristic name match over the project's sources
- Annotation: ExtensionManager.reload() drops loader/current without calling current.stop(), so unsubscribe never runs; static LISTENERS keeps each Tally, its loader and classes
- Verified: no
- Sibling hypotheses: F3.n1
- Supporting evidence: E6, E9, E11
- Explanation (observed): Static list EventBus.LISTENERS (src/reports/EventBus.java:8) gains a Tally at EventBus.java:13 on each start(); ExtensionManager.reload() drops the loader without calling stop(), so unsubscribe (EventBus.java:17) is skipped unless stop() runs, and each Tally pins its URLClassLoader and classes.
- Explanation structure: adds: src/reports/EventBus.java:13, ext/ledger/LedgerExtension.java:22; removal: conditional at src/reports/EventBus.java:17; evidence: E31

### F4: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (static EventBus.LISTENERS keeps listener of each reloaded extension; reload() never calls stop(), so each URLClassLoader and its classes (metaspace) stay reachable)
- Growth: `probe:reports.EventBus.LISTENERS` 1 count/n (steady, gate pass, E11)
- Evidence classes: context, growth
- Holder: registration-table
- Owner: `reports.EventBus.LISTENERS`
- Region: heap
- Shape: steady
- Locus: app
- Site: `src/reports/ExtensionManager.java:27`
- Writers: `ext/ledger/LedgerExtension.java:22`
- Removers: `src/reports/EventBus.java:17`
- Annotation: same root cause as F3: class counts are the retained-loader manifestation
- Verified: no
- Sibling hypotheses: F4.n1
- Supporting evidence: E11, E14, E16, E9
- Explanation (inferred): ExtensionManager.reload() at src/reports/ExtensionManager.java:27 discards the old extension; the Tally added at ext/ledger/LedgerExtension.java:22 stays in EventBus.LISTENERS, pinning each URLClassLoader so class counts rise per reload; remover EventBus.java:17 is skipped unless stop() is called.
- Explanation structure: adds: ext/ledger/LedgerExtension.java:22, src/reports/EventBus.java:13; removal: conditional at src/reports/EventBus.java:17

### F5: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (static EventBus.LISTENERS keeps listener of each reloaded extension; reload() never calls stop(), so each URLClassLoader and its classes (metaspace) stay reachable)
- Growth: `probe:reports.EventBus.LISTENERS` 1 count/n (steady, gate pass, E19)
- Evidence classes: context, growth
- Holder: registration-table
- Owner: `reports.EventBus.LISTENERS`
- Region: heap
- Shape: steady
- Locus: app
- Site: `src/reports/ExtensionManager.java:27`
- Writers: `ext/ledger/LedgerExtension.java:22`
- Removers: `src/reports/EventBus.java:17`
- Annotation: same root cause as F3: class counts are the retained-loader manifestation
- Verified: no
- Sibling hypotheses: F5.n1
- Supporting evidence: E19, E21, E9
- Explanation (inferred): ExtensionManager.reload() at src/reports/ExtensionManager.java:27 discards the old extension; the Tally added at ext/ledger/LedgerExtension.java:22 stays in EventBus.LISTENERS, pinning each URLClassLoader so class counts rise per reload; remover EventBus.java:17 is skipped unless stop() is called.
- Explanation structure: adds: ext/ledger/LedgerExtension.java:22, src/reports/EventBus.java:13; removal: conditional at src/reports/EventBus.java:17

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `ledger.LedgerExtension.ROUNDING`
- Region: heap
- Site: `ext/ledger/LedgerExtension.java:11`
- Reason: not_exercised
- Remedy: ROUNDING is a per-loader copy retained only because F3 pins the loader; not an independent holder. Fix F3 first
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `ledger.LedgerExtension.start`
- Region: heap
- Site: `ext/ledger/LedgerExtension.java:22`
- Reason: not_exercised
- Remedy: start() is the adding site for F3 (LedgerExtension.java:22); not an independent holder
- Supporting evidence: E6

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `reports.EventBus.LISTENERS`
- Region: heap
- Shape: steady
- Locus: app
- Site: `src/reports/ExtensionManager.java:27`
- Reason: not_exercised
- Remedy: Class-count columns of alloc-test repeat runs duplicate F3/F4/F5 (same retained loaders); fix F3 by unsubscribing on reload, then rerun harness to see them flatten
- Supporting evidence: E24, E26

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: retention
- Holder: static/global
- Owner: `dla.harness.DlaDump.FIELDS`
- Region: heap
- Reason: not_exercised
- Remedy: Harness instrumentation (dla.harness.DlaDump) field, not project code; ignore
- Supporting evidence: E30

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: retention
- Holder: static/global
- Owner: `dla.harness.DlaDump.SHALLOW`
- Region: heap
- Reason: not_exercised
- Remedy: Harness instrumentation (dla.harness.DlaDump) field, not project code; ignore
- Supporting evidence: E30

## Ruled out

- **F3.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM: listener list fills during warm-up then plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E12
  - Originally supported by: E11
- **F4.n1** (UNCLASSIFIED): discriminator E17
  - Ruled out by: plateau+purge (leak) E17
  - Hypothesis: N-WARM: class/metaspace growth is a warm-up that plateaus
  - Refuted if: class counts keep growing over the extra cycles
  - Contradicting evidence: E17
  - Originally supported by: E16
- **F5.n1** (UNCLASSIFIED): discriminator E22
  - Ruled out by: plateau+purge (leak) E22
  - Hypothesis: N-WARM: class growth plateaus after warm-up
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E22
  - Originally supported by: E21

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
- E17: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: coverage capped at 4000 classes
- E17: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: points 1 to 9 come from S1; the extra points from S4
- E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E17: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E17: limitation: gate and plateau thresholds are provisional (not calibrated)
- E18: limitation: coverage capped at 4000 classes
- E18: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: coverage capped at 4000 classes
- E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: coverage capped at 4000 classes
- E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: points 1 to 9 come from S1; the extra points from S4
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: coverage capped at 4000 classes
- E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: points 1 to 9 come from S1; the extra points from S4
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: coverage capped at 4000 classes
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: points 1 to 9 come from S1; the extra points from S6
- E22: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E22: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E22: limitation: gate and plateau thresholds are provisional (not calibrated)
- E23: limitation: coverage capped at 4000 classes
- E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E23: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: coverage capped at 4000 classes
- E24: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E24: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E25: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E25: limitation: coverage capped at 4000 classes
- E25: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E25: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E25: limitation: points 1 to 9 come from S1; the extra points from S6
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E26: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E26: limitation: coverage capped at 4000 classes
- E26: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E26: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E26: limitation: points 1 to 9 come from S1; the extra points from S6
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E28: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E29: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E30: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E31: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E31: limitation: series: per-point; one process per point at n = 440, 1760, 2640, 3960 (harness captures; checked: HPROF version, identifier size)
- E32: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E33: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap dump
- E33: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E33: limitation: holders absent from a dump's 200 largest have no figure at that point
- E33: limitation: gate thresholds are provisional (not calibrated)
- E34: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap dump
- E34: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E34: limitation: holders absent from a dump's 200 largest have no figure at that point
- E34: limitation: gate thresholds are provisional (not calibrated)
- E35: partial coverage (2 of 3 examined)
- E35: limitation: F3: growth measured as a count; bytes not attributed
- E35: limitation: F4: growth measured as a count; bytes not attributed
- E35: limitation: F5: growth evidence is from S4, not S1; not subtracted
- E35: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E35: limitation: threshold is provisional (not calibrated)
- E36: partial coverage (2 of 3 examined)
- E36: limitation: F3: growth evidence is from S1, not S2; not subtracted
- E36: limitation: F4: growth measured as a count; bytes not attributed
- E36: limitation: F5: growth evidence is from S4, not S2; not subtracted
- E36: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E36: limitation: threshold is provisional (not calibrated)
- E37: partial coverage (2 of 3 examined)
- E37: limitation: F3: growth evidence is from S1, not S3; not subtracted
- E37: limitation: F4: growth measured as a count; bytes not attributed
- E37: limitation: F5: growth evidence is from S4, not S3; not subtracted
- E37: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E37: limitation: threshold is provisional (not calibrated)
- E38: partial coverage (2 of 3 examined)
- E38: limitation: F3: growth evidence is from S1, not S4; not subtracted
- E38: limitation: F4: growth evidence is from S1, not S4; not subtracted
- E38: limitation: F5: growth measured as a count; bytes not attributed
- E38: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E38: limitation: threshold is provisional (not calibrated)
- E39: partial coverage (2 of 3 examined)
- E39: limitation: F3: growth evidence is from S1, not S5; not subtracted
- E39: limitation: F4: growth evidence is from S1, not S5; not subtracted
- E39: limitation: F5: growth measured as a count; bytes not attributed
- E39: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E39: limitation: threshold is provisional (not calibrated)
- E40: partial coverage (2 of 3 examined)
- E40: limitation: F3: growth evidence is from S1, not S6; not subtracted
- E40: limitation: F4: growth evidence is from S1, not S6; not subtracted
- E40: limitation: F5: growth evidence is from S4, not S6; not subtracted
- E40: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E40: limitation: threshold is provisional (not calibrated)
- E41: partial coverage (2 of 3 examined)
- E41: limitation: F3: growth evidence is from S1, not S7; not subtracted
- E41: limitation: F4: growth evidence is from S1, not S7; not subtracted
- E41: limitation: F5: growth evidence is from S4, not S7; not subtracted
- E41: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E41: limitation: threshold is provisional (not calibrated)

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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (ea8a4d837005) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (656044bcc52e) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (656044bcc52e) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (a41e8eac3629) | alloc/A1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S2.json (b6daa1b52c36) | trend/T3.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (b6daa1b52c36) | trend/T3.json |
| E15 | growth-series | test | dla 0.1.0 | artifact series/S3.json (a41e8eac3629) | trend/T4.json |
| E16 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (a41e8eac3629) | trend/T4.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S5.json (380ec2c422f3) | alloc/A2.json |
| E18 | growth-series | test | dla 0.1.0 | artifact series/S4.json (4ed2f7a2259b) | trend/T6.json |
| E19 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (4ed2f7a2259b) | trend/T6.json |
| E20 | growth-series | test | dla 0.1.0 | artifact series/S5.json (380ec2c422f3) | trend/T7.json |
| E21 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (380ec2c422f3) | trend/T7.json |
| E22 | discriminator | test | dla 0.1.0 | artifact series/S7.json (8b18163f08c7) | alloc/A3.json |
| E23 | growth-series | test | dla 0.1.0 | artifact series/S6.json (50ab94a5f035) | trend/T9.json |
| E24 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (50ab94a5f035) | trend/T9.json |
| E25 | growth-series | test | dla 0.1.0 | artifact series/S7.json (8b18163f08c7) | trend/T10.json |
| E26 | unit-count-series | test | dla 0.1.0 | artifact series/S7.json (8b18163f08c7) | trend/T10.json |
| E27 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-193858-9466/harness/H1/out/p1-native/capture/105/heap-1.hprof (c009450aa500) | summaries/heap-1-c009450a.json |
| E28 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-193858-9466/harness/H1/out/p4-native/capture/466/heap-1.hprof (0831f8458955) | summaries/heap-1-0831f845.json |
| E29 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-193858-9466/harness/H1/out/p6-native/capture/707/heap-1.hprof (e00b4db173b5) | summaries/heap-1-e00b4db1.json |
| E30 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-193858-9466/harness/H1/out/p9-native/capture/1069/heap-1.hprof (5d6948583c60) | summaries/heap-1-5d694858.json |
| E31 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S8.json (a68375b19166) | series/S8.json |
| E32 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S8.json (a68375b19166) | series/S8.json |
| E33 | growth-series | dump | dla 0.1.0 | artifact series/S8.json (a68375b19166) | trend/T11.json |
| E34 | unit-count-series | dump | dla 0.1.0 | artifact series/S8.json (a68375b19166) | trend/T11.json |
| E35 | residual | test | dla 0.1.0 | artifact series/S1.json (656044bcc52e) | gap/residual-S1-E35.json |
| E36 | residual | test | dla 0.1.0 | artifact series/S2.json (b6daa1b52c36) | gap/residual-S2-E36.json |
| E37 | residual | test | dla 0.1.0 | artifact series/S3.json (a41e8eac3629) | gap/residual-S3-E37.json |
| E38 | residual | test | dla 0.1.0 | artifact series/S4.json (4ed2f7a2259b) | gap/residual-S4-E38.json |
| E39 | residual | test | dla 0.1.0 | artifact series/S5.json (380ec2c422f3) | gap/residual-S5-E39.json |
| E40 | residual | test | dla 0.1.0 | artifact series/S6.json (50ab94a5f035) | gap/residual-S6-E40.json |
| E41 | residual | test | dla 0.1.0 | artifact series/S7.json (8b18163f08c7) | gap/residual-S7-E41.json |
