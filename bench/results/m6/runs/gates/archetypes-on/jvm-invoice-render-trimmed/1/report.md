# DLA report

Run: `20261110-164118-f999`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 7 enumerated, 7 dispositioned; confirmed 0, strong 0, refuted 0, suspected 7, unresolved 0

Residual: heap 0 bytes/n unattributed of -1.6113256113256114 total (threshold 2.6374415444014487); non-heap 1.2228228228228228 bytes/n unattributed of 1.2228228228228228 total (threshold 21.884473513513154)

Coverage: jvm 8880 iterations, 14 of 14 entry points exercised

## Findings

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `billing.Invoice.lines`
- Region: heap
- Site: `src/billing/Invoice.java:6`
- Reason: not_exercised
- Remedy: No growth in harness H1 (9 points) nor in a 100k-invoice run (heap flat ~1.69MB); reproduce with production traffic/data, e.g. a render that throws, or take heap dumps from the live process
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `billing.InvoiceFeed.SKUS`
- Region: heap
- Site: `src/billing/InvoiceFeed.java:9`
- Reason: not_exercised
- Remedy: No growth in harness H1 (9 points) nor in a 100k-invoice run (heap flat ~1.69MB); reproduce with production traffic/data, e.g. a render that throws, or take heap dumps from the live process
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `billing.InvoiceFeed.NAMES`
- Region: heap
- Site: `src/billing/InvoiceFeed.java:10`
- Reason: not_exercised
- Remedy: No growth in harness H1 (9 points) nor in a 100k-invoice run (heap flat ~1.69MB); reproduce with production traffic/data, e.g. a render that throws, or take heap dumps from the live process
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `billing.InvoiceService.pool`
- Region: non-heap
- Site: `src/billing/InvoiceService.java:17`
- Reason: not_exercised
- Remedy: No growth in harness H1 (9 points) nor in a 100k-invoice run (heap flat ~1.69MB); reproduce with production traffic/data, e.g. a render that throws, or take heap dumps from the live process
- Supporting evidence: E6

### F5: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `billing.InvoiceService$WorkerFactory.newThread`
- Region: non-heap
- Site: `src/billing/InvoiceService.java:58`
- Reason: not_exercised
- Remedy: No growth in harness H1 (9 points) nor in a 100k-invoice run (heap flat ~1.69MB); reproduce with production traffic/data, e.g. a render that throws, or take heap dumps from the live process
- Supporting evidence: E6

### F6: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: thread-local
- Owner: `billing.RenderContext.SEGMENTS`
- Region: heap
- Site: `src/billing/RenderContext.java:8`
- Writers: `src/billing/RenderContext.java:13`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: No growth in harness H1 (9 points) nor in a 100k-invoice run (heap flat ~1.69MB); reproduce with production traffic/data, e.g. a render that throws, or take heap dumps from the live process
- Supporting evidence: E6

### F7: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `billing.TaxTable.RATES`
- Region: heap
- Site: `src/billing/TaxTable.java:7`
- Reason: not_exercised
- Remedy: No growth in harness H1 (9 points) nor in a 100k-invoice run (heap flat ~1.69MB); reproduce with production traffic/data, e.g. a render that throws, or take heap dumps from the live process
- Supporting evidence: E6

## Ruled out

None.

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
- E12: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E12: limitation: the files carry no time, so they are ordered as given
- E13: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E13: limitation: the files carry no time, so they are ordered as given
- E13: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E14: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E14: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E15: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: partial coverage (2 of 3 examined)
- E16: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E16: limitation: threshold is provisional (not calibrated)

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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (ecc13e9e79f6) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (f522846b50b5) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (f522846b50b5) | trend/T1.json |
| E12 | histogram-diff | live | jcmd-class-histogram 0.1.0 | artifact histograms/H1.json (11e82fd17fbc) | histograms/H1.json |
| E13 | growth-series | live | jcmd-class-histogram 0.1.0 | artifact series/S2.json (c7e7c38867b7) | series/S2.json |
| E14 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (c7e7c38867b7) | trend/T2.json |
| E15 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (c7e7c38867b7) | trend/T2.json |
| E16 | residual | test | dla 0.1.0 | artifact series/S1.json (f522846b50b5) | gap/residual-S1-E16.json |
