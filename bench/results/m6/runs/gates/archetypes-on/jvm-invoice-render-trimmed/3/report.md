# DLA report

Run: `20261110-180346-2750`

Status: INCOMPLETE (2 open)

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 7 enumerated, 7 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 7

Residual: not measured

Coverage: jvm 8880 iterations, 14 of 14 entry points exercised

## Open items

- **T1:os.rss**: column os.rss of T1 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:os.rss_anon**: column os.rss_anon of T1 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `billing.Invoice.lines`
- Region: heap
- Site: `src/billing/Invoice.java:6`
- Reason: not_exercised
- Remedy: Bundled load driver (200k invoices, heapKB flat ~1690 after GC; harness trend S1 no passing column) showed no growth, so no leak is confirmed. Reproduce with production traffic (incl. error/exception paths) or provide 3+ heap dumps from the live process.
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `billing.InvoiceFeed.SKUS`
- Region: heap
- Site: `src/billing/InvoiceFeed.java:9`
- Reason: not_exercised
- Remedy: Bundled load driver (200k invoices, heapKB flat ~1690 after GC; harness trend S1 no passing column) showed no growth, so no leak is confirmed. Reproduce with production traffic (incl. error/exception paths) or provide 3+ heap dumps from the live process.
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `billing.InvoiceFeed.NAMES`
- Region: heap
- Site: `src/billing/InvoiceFeed.java:10`
- Reason: not_exercised
- Remedy: Bundled load driver (200k invoices, heapKB flat ~1690 after GC; harness trend S1 no passing column) showed no growth, so no leak is confirmed. Reproduce with production traffic (incl. error/exception paths) or provide 3+ heap dumps from the live process.
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `billing.InvoiceService.pool`
- Region: non-heap
- Site: `src/billing/InvoiceService.java:17`
- Reason: not_exercised
- Remedy: Bundled load driver (200k invoices, heapKB flat ~1690 after GC; harness trend S1 no passing column) showed no growth, so no leak is confirmed. Reproduce with production traffic (incl. error/exception paths) or provide 3+ heap dumps from the live process.
- Supporting evidence: E6

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `billing.InvoiceService$WorkerFactory.newThread`
- Region: non-heap
- Site: `src/billing/InvoiceService.java:58`
- Reason: not_exercised
- Remedy: Bundled load driver (200k invoices, heapKB flat ~1690 after GC; harness trend S1 no passing column) showed no growth, so no leak is confirmed. Reproduce with production traffic (incl. error/exception paths) or provide 3+ heap dumps from the live process.
- Supporting evidence: E6

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: thread-local
- Owner: `billing.RenderContext.SEGMENTS`
- Region: heap
- Site: `src/billing/RenderContext.java:8`
- Writers: `src/billing/RenderContext.java:13`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Bundled load driver (200k invoices, heapKB flat ~1690 after GC; harness trend S1 no passing column) showed no growth, so no leak is confirmed. Reproduce with production traffic (incl. error/exception paths) or provide 3+ heap dumps from the live process.
- Supporting evidence: E6

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `billing.TaxTable.RATES`
- Region: heap
- Site: `src/billing/TaxTable.java:7`
- Reason: not_exercised
- Remedy: Bundled load driver (200k invoices, heapKB flat ~1690 after GC; harness trend S1 no passing column) showed no growth, so no leak is confirmed. Reproduce with production traffic (incl. error/exception paths) or provide 3+ heap dumps from the live process.
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
- E12: limitation: two captures cannot tell a warm-up fill from a leak: no monotone type is reported and no growth column is written

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
| E1 | run-config | static | dla 0.1.0 | repo da12a68e42b7ad806ec1746a9d7bc1189ef42f58 | — |
| E2 | run-config | static | dla 0.1.0 | repo da12a68e42b7ad806ec1746a9d7bc1189ef42f58 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo da12a68e42b7ad806ec1746a9d7bc1189ef42f58 | probe/probe.json |
| E4 | static-hit | static | spotbugs unknown | repo da12a68e42b7ad806ec1746a9d7bc1189ef42f58 | sweep/jvm-spotbugs.json |
| E5 | static-hit | static | dla 0.1.0 | repo da12a68e42b7ad806ec1746a9d7bc1189ef42f58 | sweep/jvm-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo da12a68e42b7ad806ec1746a9d7bc1189ef42f58 | census/jvm.json |
| E7 | workload | static | dla 0.1.0 | repo da12a68e42b7ad806ec1746a9d7bc1189ef42f58 | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo da12a68e42b7ad806ec1746a9d7bc1189ef42f58 | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (09a522c07a3d) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (e947c64313c9) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (e947c64313c9) | trend/T1.json |
| E12 | histogram-diff | live | jcmd-class-histogram 0.1.0 | artifact histograms/H1.json (290bdd982c33) | histograms/H1.json |
