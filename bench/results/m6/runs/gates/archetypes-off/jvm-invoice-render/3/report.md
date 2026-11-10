# DLA report

Run: `20261110-195052-d2e0`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 11 enumerated, 11 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 9

Residual: heap 1273.0204204204206 bytes/n unattributed of 1273.0204204204206 total (threshold 318.25510510510514) ABOVE THRESHOLD, see O1; heap 1245.1846846846847 bytes/n unattributed of 1245.1846846846847 total (threshold 311.2961711711712) ABOVE THRESHOLD, see O1; heap 1258.6002574002573 bytes/n unattributed of 1258.6002574002573 total (threshold 314.6500643500643) ABOVE THRESHOLD, see O1; non-heap 0 bytes/n unattributed of -5.906906906906906 total (threshold 44.23051837837802); non-heap 36.98558558558558 bytes/n unattributed of 36.98558558558558 total (threshold 68.75485945945945); non-heap 10.505705705705704 bytes/n unattributed of 10.505705705705704 total (threshold 25.18051401801769)

Coverage: jvm 8880 iterations, 13 of 13 entry points exercised

## Findings

### F6: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (static ThreadLocal<List<String>> RenderContext.SEGMENTS on the 4 long-lived pooled render threads: every render appends ~N segments to the per-thread list and never truncates it after textFrom, so the list grows for the life of the thread)
- Growth: `probe:billing.RenderContext.SEGMENTS` 12.53 count/n (steady, gate pass, E11)
- Evidence classes: context, growth, static
- Holder: thread-local
- Owner: `billing.RenderContext.SEGMENTS`
- Region: heap
- Shape: steady
- Site: `src/billing/RenderContext.java:8`
- Writers: `src/billing/RenderContext.java:13`, `src/billing/RenderContext.java:14`
- Annotation: heuristic name match over the project's sources
- Annotation: append() adds to the per-thread list; no remove/clear/ThreadLocal.remove anywhere; mark/textFrom only read a sublist
- Verified: no
- Sibling hypotheses: F6.n1
- Supporting evidence: E6, E9, E11
- Explanation (inferred): ThreadLocal SEGMENTS at RenderContext.java:8 is a thread-local list on pooled threads; append() at RenderContext.java:14 adds segments on every render and the list is never cleared, so removal never happens and it grows forever.
- Explanation structure: adds: src/billing/RenderContext.java:14; removal: never

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `billing.Invoice.lines`
- Region: heap
- Site: `src/billing/Invoice.java:6`
- Reason: not_exercised
- Remedy: Census lead only; its probe column did not grow in the harness, so no evidence either way. Re-check if F6 is fixed and heap still climbs.
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `billing.InvoiceFeed.SKUS`
- Region: heap
- Site: `src/billing/InvoiceFeed.java:9`
- Reason: not_exercised
- Remedy: Census lead only; its probe column did not grow in the harness, so no evidence either way. Re-check if F6 is fixed and heap still climbs.
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `billing.InvoiceFeed.NAMES`
- Region: heap
- Site: `src/billing/InvoiceFeed.java:10`
- Reason: not_exercised
- Remedy: Census lead only; its probe column did not grow in the harness, so no evidence either way. Re-check if F6 is fixed and heap still climbs.
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `billing.InvoiceService.pool`
- Region: non-heap
- Site: `src/billing/InvoiceService.java:17`
- Reason: not_exercised
- Remedy: Census lead only; its probe column did not grow in the harness, so no evidence either way. Re-check if F6 is fixed and heap still climbs.
- Supporting evidence: E6

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `billing.InvoiceService$WorkerFactory.newThread`
- Region: non-heap
- Site: `src/billing/InvoiceService.java:58`
- Reason: not_exercised
- Remedy: Census lead only; its probe column did not grow in the harness, so no evidence either way. Re-check if F6 is fixed and heap still climbs.
- Supporting evidence: E6

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `billing.TaxTable.RATES`
- Region: heap
- Site: `src/billing/TaxTable.java:7`
- Reason: not_exercised
- Remedy: Census lead only; its probe column did not grow in the harness, so no evidence either way. Re-check if F6 is fixed and heap still climbs.
- Supporting evidence: E6

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Hypothesis: String/byte[] instance growth: the segment strings retained by RenderContext.SEGMENTS
- Reason: not_exercised
- Remedy: String/byte[] growth is the contents of F6's list; fixing F6 removes it
- Supporting evidence: E11

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Hypothesis: String/byte[] growth is the segment strings held by F6's RenderContext.SEGMENTS list
- Reason: not_exercised
- Remedy: Same growth as F6 (contents of RenderContext.SEGMENTS); fixing F6 removes it
- Supporting evidence: E14

### F10: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Hypothesis: String/byte[] growth is the segment strings held by F6's RenderContext.SEGMENTS list
- Reason: not_exercised
- Remedy: Same growth as F6 (contents of RenderContext.SEGMENTS); fixing F6 removes it
- Supporting evidence: E16

## Ruled out

- **F6.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM: segments are a warm-up fill that plateaus
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
- E17: partial coverage (2 of 3 examined)
- E17: limitation: F6: growth measured as a count; bytes not attributed
- E17: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E17: limitation: threshold is provisional (not calibrated)
- E18: partial coverage (2 of 3 examined)
- E18: limitation: F6: growth evidence is from S1, not S2; not subtracted
- E18: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E18: limitation: threshold is provisional (not calibrated)
- E19: partial coverage (2 of 3 examined)
- E19: limitation: F6: growth evidence is from S1, not S3; not subtracted
- E19: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E19: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | — |
| E2 | run-config | static | dla 0.1.0 | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | probe/probe.json |
| E4 | static-hit | static | spotbugs unknown | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | sweep/jvm-spotbugs.json |
| E5 | static-hit | static | dla 0.1.0 | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | sweep/jvm-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | census/jvm.json |
| E7 | workload | static | dla 0.1.0 | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo 83cf8992bb603d1f10b2edd6124a984dbb69ace1 | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (14bf06f9d36a) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (7c95b3d30a74) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (7c95b3d30a74) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (2c9995db9358) | alloc/A1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S2.json (3732dfedb274) | trend/T3.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (3732dfedb274) | trend/T3.json |
| E15 | growth-series | test | dla 0.1.0 | artifact series/S3.json (2c9995db9358) | trend/T4.json |
| E16 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (2c9995db9358) | trend/T4.json |
| E17 | residual | test | dla 0.1.0 | artifact series/S1.json (7c95b3d30a74) | gap/residual-S1-E17.json |
| E18 | residual | test | dla 0.1.0 | artifact series/S2.json (3732dfedb274) | gap/residual-S2-E18.json |
| E19 | residual | test | dla 0.1.0 | artifact series/S3.json (2c9995db9358) | gap/residual-S3-E19.json |
