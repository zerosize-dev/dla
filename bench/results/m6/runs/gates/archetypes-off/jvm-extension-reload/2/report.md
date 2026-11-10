# DLA report

Run: `20261110-190346-8ffa`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 8 enumerated, 8 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 6

Residual: heap 13850.201515151515 bytes/iteration unattributed of 13850.201515151515 total (threshold 3462.550378787879) ABOVE THRESHOLD, see O1; heap 13835.63484848485 bytes/iteration unattributed of 13835.63484848485 total (threshold 3458.9087121212124) ABOVE THRESHOLD, see O1; heap 13821.317694805197 bytes/iteration unattributed of 13821.317694805197 total (threshold 3455.3294237012988) ABOVE THRESHOLD, see O1; heap 13819.334545454543 bytes/iteration unattributed of 13819.334545454543 total (threshold 3454.8336363636363) ABOVE THRESHOLD, see O1; heap 13841.680086580089 bytes/iteration unattributed of 13841.680086580089 total (threshold 3460.4200216450217) ABOVE THRESHOLD, see O1; non-heap 6452.120454545455 bytes/iteration unattributed of 6452.120454545455 total (threshold 1613.0301136363637) ABOVE THRESHOLD, see O2; non-heap 6443.868181818181 bytes/iteration unattributed of 6443.868181818181 total (threshold 1610.967045454545) ABOVE THRESHOLD, see O2; non-heap 6395.494214876033 bytes/iteration unattributed of 6395.494214876033 total (threshold 1598.8735537190082) ABOVE THRESHOLD, see O2; non-heap 6470.721212121212 bytes/iteration unattributed of 6470.721212121212 total (threshold 1617.680303030303) ABOVE THRESHOLD, see O2; non-heap 6457.836969696969 bytes/iteration unattributed of 6457.836969696969 total (threshold 1614.4592424242423) ABOVE THRESHOLD, see O2

Coverage: jvm 3520 iterations, 0 of 0 entry points exercised

## Findings

### F3: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Static EventBus.LISTENERS holds each reloaded extension's Tally listener; ExtensionManager.reload() discards the old extension without stop(), so unsubscribe never runs and each listener pins its URLClassLoader, classes and statics (metaspace + heap).)
- Growth: `probe:reports.EventBus.LISTENERS` 1 count/iteration (steady, gate pass, E12)
- Evidence classes: context, growth, static
- Holder: registration-table
- Owner: `reports.EventBus.LISTENERS`
- Region: heap
- Shape: steady
- Site: `src/reports/EventBus.java:8`
- Writers: `src/reports/EventBus.java:13`, `src/reports/ExtensionManager.java:20`
- Removers: `src/reports/EventBus.java:17`
- Annotation: heuristic name match over the project's sources
- Annotation: reload() drops references to old loader/extension without calling stop(), so the Tally registered in start() is never unsubscribed
- Verified: no
- Sibling hypotheses: F3.n1
- Supporting evidence: E6, E8, E12
- Explanation (inferred): EventBus.LISTENERS (static registration table, EventBus.java:8) gets a Tally per start() via subscribe at EventBus.java:13; unsubscribe at EventBus.java:17 runs only when stop() is called, and only shutdown() calls it, never reload(), so old listeners and loaders stay reachable.
- Explanation structure: adds: src/reports/EventBus.java:13; removal: conditional at src/reports/EventBus.java:17

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `ledger.LedgerExtension.ROUNDING`
- Region: heap
- Site: `ext/ledger/LedgerExtension.java:11`
- Reason: not_exercised
- Remedy: ROUNDING long[] is a per-loader static copy (3520 long[] growth in histogram); freed once F3 releases the loaders
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `ledger.LedgerExtension.start`
- Region: heap
- Site: `ext/ledger/LedgerExtension.java:22`
- Writers: `ext/ledger/LedgerExtension.java:22`
- Removers: `ext/ledger/LedgerExtension.java:27`
- Reason: not_exercised
- Remedy: start() is the subscribe call site of F3; the missing unsubscribe is in ExtensionManager.reload(), see F3
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: discriminator
- Holder: registration-table
- Owner: `reports.EventBus.LISTENERS`
- Region: heap
- Shape: steady
- Site: `src/reports/ExtensionManager.java:27`
- Reason: not_exercised
- Remedy: Duplicate of F3: JDK class-count growth is a symptom of loaders pinned by EventBus.LISTENERS; fix F3 (stop() before reload) and re-measure
- Supporting evidence: E13

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `reports.EventBus.LISTENERS`
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Duplicate of F3: JDK class-count growth is a symptom of loaders pinned by EventBus.LISTENERS; fix F3 (stop() before reload) and re-measure
- Supporting evidence: E21, E23

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `reports.EventBus.LISTENERS`
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Duplicate of F3: JDK class-count growth is a symptom of loaders pinned by EventBus.LISTENERS; fix F3 (stop() before reload) and re-measure
- Supporting evidence: E24, E25, E26, E27

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `reports.EventBus.LISTENERS`
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Duplicate of F3: JDK class-count growth is a symptom of loaders pinned by EventBus.LISTENERS; fix F3 (stop() before reload) and re-measure
- Supporting evidence: E29

## Ruled out

- **F3.n1** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (leak) E13
  - Hypothesis: N-WARM: listeners fill once then plateau
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E13
  - Originally supported by: E10

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
- E7: limitation: harness written; nothing was run (dla harness run measures it)
- E7: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E7: limitation: junit-platform-console-standalone not used: the plain driver main runs the loop (JUnit variant is generated only when the jar is on the classpath)
- E8: partial coverage (9 of 9 examined)
- E8: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E8: limitation: no entry points are listed for this runtime: run dla workload discover before the harness to see which ones ran
- E9: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E9: limitation: gate thresholds are provisional (not calibrated)
- E10: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E13: limitation: junit-platform-console-standalone not used: the plain driver main runs the loop (JUnit variant is generated only when the jar is on the classpath)
- E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- E14: partial coverage (9 of 9 examined)
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: no entry points are listed for this runtime: run dla workload discover before the harness to see which ones ran
- E15: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E15: limitation: the files carry no time, so they are ordered as given
- E16: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E16: limitation: the files carry no time, so they are ordered as given
- E16: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E17: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E17: limitation: the files carry no time, so they are ordered as given
- E18: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E18: limitation: the files carry no time, so they are ordered as given
- E18: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E19: limitation: loaders and classes are whole-runtime counts: they never tie to a finding
- E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E24: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E25: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E26: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E27: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E29: limitation: gate thresholds are provisional (not calibrated)
- E30: partial coverage (2 of 3 examined)
- E30: limitation: F3: growth evidence is from S2, not S1; not subtracted
- E30: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E30: limitation: threshold is provisional (not calibrated)
- E31: partial coverage (2 of 3 examined)
- E31: limitation: F3: growth measured as a count; bytes not attributed
- E31: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E31: limitation: threshold is provisional (not calibrated)
- E32: partial coverage (2 of 3 examined)
- E32: limitation: F3: growth evidence is from S2, not S3; not subtracted
- E32: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E32: limitation: threshold is provisional (not calibrated)
- E33: partial coverage (2 of 3 examined)
- E33: limitation: F3: growth evidence is from S2, not S4; not subtracted
- E33: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E33: limitation: threshold is provisional (not calibrated)
- E34: partial coverage (2 of 3 examined)
- E34: limitation: F3: growth evidence is from S2, not S5; not subtracted
- E34: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E34: limitation: threshold is provisional (not calibrated)

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
| E7 | workload | static | dla 0.1.0 | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | harness/H1/harness.json |
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (670de28ed972) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (b9a395786642) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (b9a395786642) | trend/T1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S2.json (493ab76261ea) | trend/T2.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (493ab76261ea) | trend/T2.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S3.json (c503cf0e8dce) | alloc/A1.json |
| E14 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (44581f253019) | harness/H1/run-S4.json |
| E15 | histogram-diff | live | jcmd-class-histogram 0.1.0 | artifact histograms/H1.json (6eebeabe27e2) | histograms/H1.json |
| E16 | growth-series | live | jcmd-class-histogram 0.1.0 | artifact series/S6.json (c6f2d14bda65) | series/S6.json |
| E17 | histogram-diff | live | jcmd-class-histogram 0.1.0 | artifact histograms/H2.json (0b0201e49bbb) | histograms/H2.json |
| E18 | growth-series | live | jcmd-class-histogram 0.1.0 | artifact series/S7.json (943c10455d81) | series/S7.json |
| E19 | non-heap | live | jvm-classloader-stats 0.1.0 | artifact nonheap/N1.json (0b483f687415) | nonheap/N1.json |
| E20 | growth-series | test | dla 0.1.0 | artifact series/S4.json (6f21b419f8ba) | trend/T4.json |
| E21 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (6f21b419f8ba) | trend/T4.json |
| E22 | growth-series | test | dla 0.1.0 | artifact series/S5.json (0ff99cea8b30) | trend/T5.json |
| E23 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (0ff99cea8b30) | trend/T5.json |
| E24 | growth-series | dump | dla 0.1.0 | artifact series/S6.json (c6f2d14bda65) | trend/T6.json |
| E25 | unit-count-series | dump | dla 0.1.0 | artifact series/S6.json (c6f2d14bda65) | trend/T6.json |
| E26 | growth-series | dump | dla 0.1.0 | artifact series/S7.json (943c10455d81) | trend/T7.json |
| E27 | unit-count-series | dump | dla 0.1.0 | artifact series/S7.json (943c10455d81) | trend/T7.json |
| E28 | growth-series | test | dla 0.1.0 | artifact series/S3.json (c503cf0e8dce) | trend/T8.json |
| E29 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (c503cf0e8dce) | trend/T8.json |
| E30 | residual | test | dla 0.1.0 | artifact series/S1.json (b9a395786642) | gap/residual-S1-E30.json |
| E31 | residual | test | dla 0.1.0 | artifact series/S2.json (493ab76261ea) | gap/residual-S2-E31.json |
| E32 | residual | test | dla 0.1.0 | artifact series/S3.json (c503cf0e8dce) | gap/residual-S3-E32.json |
| E33 | residual | test | dla 0.1.0 | artifact series/S4.json (6f21b419f8ba) | gap/residual-S4-E33.json |
| E34 | residual | test | dla 0.1.0 | artifact series/S5.json (0ff99cea8b30) | gap/residual-S5-E34.json |
