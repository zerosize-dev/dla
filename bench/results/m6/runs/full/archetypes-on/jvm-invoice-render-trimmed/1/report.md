# DLA report

Run: `20261110-204446-2221`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 8 enumerated, 8 dispositioned; confirmed 0, strong 0, refuted 7, suspected 1, unresolved 0

Residual: heap 0 bytes/n unattributed of -0.8519948519948519 total (threshold 1.5020627027025693); heap 0 bytes/n unattributed of -0.017417417417417418 total (threshold 0.23534605405405404); off-heap 0 bytes/n unattributed of -974.645045045045 total (threshold 8342.676064864863); non-heap 36.51675675675675 bytes/n unattributed of 36.51675675675675 total (threshold 11.80882313513524) ABOVE THRESHOLD, see O1; non-heap 13.342964782964785 bytes/n unattributed of 13.342964782964785 total (threshold 23.025280699754305)

Coverage: jvm 8880 iterations, 14 of 14 entry points exercised

## Findings

### F6: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: discriminator, static
- Holder: thread-local
- Owner: `billing.RenderContext.SEGMENTS`
- Region: heap
- Site: `src/billing/RenderContext.java:8`
- Writers: `src/billing/RenderContext.java:13`
- Removers: `src/billing/RenderContext.java:28`
- Annotation: heuristic name match over the project's sources
- Annotation: per-worker ThreadLocal list; render() appends then truncates to mark
- Reason: not_exercised
- Remedy: RenderContext.render has no try/finally around mark/truncate: if render throws between mark and truncate (e.g. null lines or customer data, format error) the worker thread's ThreadLocal list keeps the segments forever. Driver never throws; replay production failing invoices or add a throwing invoice to the harness to confirm
- Supporting evidence: E6, E13

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (non-leak) E13
  - Contradicting evidence: E13
  - Originally supported by: E6
- **F2** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (non-leak) E13
  - Contradicting evidence: E13
  - Originally supported by: E6
- **F3** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (non-leak) E13
  - Contradicting evidence: E13
  - Originally supported by: E6
- **F4** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (non-leak) E13
  - Contradicting evidence: E13
  - Originally supported by: E6
- **F5** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (non-leak) E13
  - Contradicting evidence: E13
  - Originally supported by: E6
- **F7** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (non-leak) E13
  - Contradicting evidence: E13
  - Originally supported by: E6
- **F8** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (non-leak) E13
  - Contradicting evidence: E13
  - Originally supported by: E15

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
- E12: limitation: gap.unexplained is negative at 9 point(s): the accounted figure exceeds os.rss (reserved but untouched memory)
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: points 1 to 9 come from S1; the extra points from S3
- E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E13: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: partial coverage (3 of 3 examined)
- E16: limitation: threshold is provisional (not calibrated)
- E17: partial coverage (2 of 3 examined)
- E17: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E17: limitation: O1 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E17: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo da12a68e42b7ad806ec1746a9d7bc1189ef42f58 | — |
| E2 | run-config | static | dla 0.1.0 | repo da12a68e42b7ad806ec1746a9d7bc1189ef42f58 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo da12a68e42b7ad806ec1746a9d7bc1189ef42f58 | probe/probe.json |
| E4 | static-hit | static | spotbugs unknown | repo da12a68e42b7ad806ec1746a9d7bc1189ef42f58 | sweep/jvm-spotbugs.json |
| E5 | static-hit | static | dla 0.1.0 | repo da12a68e42b7ad806ec1746a9d7bc1189ef42f58 | sweep/jvm-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo da12a68e42b7ad806ec1746a9d7bc1189ef42f58 | census/jvm.json |
| E7 | workload | static | dla 0.1.0 | repo da12a68e42b7ad806ec1746a9d7bc1189ef42f58 | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo da12a68e42b7ad806ec1746a9d7bc1189ef42f58 | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (de709fe24134) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (305597046535) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (305597046535) | trend/T1.json |
| E12 | non-heap | test | dla 0.1.0 | artifact series/S2.json (8f7ef29837c9) | trend/T2.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S4.json (4de97ff8d66c) | alloc/A1.json |
| E14 | growth-series | test | dla 0.1.0 | artifact series/S3.json (6de05924704d) | trend/T4.json |
| E15 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (6de05924704d) | trend/T4.json |
| E16 | residual | test | dla 0.1.0 | artifact series/S1.json (305597046535) | gap/residual-S1-E16.json |
| E17 | residual | test | dla 0.1.0 | artifact series/S4.json (4de97ff8d66c) | gap/residual-S4-E17.json |
