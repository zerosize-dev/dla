# DLA report

Run: `20261109-195414-9805`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 8 enumerated, 8 dispositioned; confirmed 2, strong 0, refuted 2, suspected 4, unresolved 0

Residual: not measured

Coverage: python 3520 iterations, 8 of 11 entry points exercised

## Findings

### F3: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:ingest.parser._newest.bytes` 655.5 bytes/n (steady, gate pass, E11)
- Evidence classes: context, discriminator, growth, static
- Holder: static/global
- Owner: `ingest.parser._newest`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:3`
- Writers: `ingest/parser.py:29`
- Annotation: heuristic name match over the project's sources
- Annotation: Module-global parser._newest; each Record keeps .previous to the prior Record, so every Record ever parsed stays reachable as a singly linked chain. Missing pairing: the previous link (or only the value needed for delta) should be dropped.
- Sibling hypotheses: F3.n1
- Supporting evidence: E7, E10, E11, E12, E17

### F6: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:ingest.registry._LISTENERS.bytes` 1906 bytes/n (steady, gate pass, E11)
- Evidence classes: context, discriminator, growth, static
- Holder: registration-table
- Owner: `ingest.registry._LISTENERS`
- Region: heap
- Shape: steady
- Site: `ingest/registry.py:3`
- Writers: `ingest/registry.py:7`
- Removers: `ingest/registry.py:24`
- Annotation: heuristic name match over the project's sources
- Annotation: registry._LISTENERS['checkpoint'] gets a new CheckpointWatcher per batch (pipeline.py:28) and is never unsubscribed; only registry.reset() clears it (tests). Each watcher holds a dict of marks for all known sensors. Missing pairing: unsubscribe after the watcher fires/at batch end.
- Sibling hypotheses: F6.n1
- Supporting evidence: E7, E10, E11, E12, E18

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Site: `ingest/parser.py:29`
- Reason: not_exercised
- Remedy: Detector hit at parser.py:29 is the same site as confirmed F3; no separate column.
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Site: `ingest/registry.py:3`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: Detector hit at registry.py:3 is the same holder as confirmed F6; no separate column.
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `ingest.pipeline.Pipeline.per_sensor`
- Region: heap
- Site: `ingest/pipeline.py:15`
- Writers: `ingest/pipeline.py:25`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: per_sensor keys are bounded by 64 sensor ids; no probe column grew for it, only process-wide columns (growth-not-tied). Re-test with a probe on per_sensor if sensor ids become unbounded.
- Supporting evidence: E7

### F5: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `ingest.pipeline.Pipeline.process_batch`
- Region: heap
- Site: `ingest/pipeline.py:28`
- Reason: not_exercised
- Remedy: Census entry for the process_batch subscribe call; the growth is the registry table, confirmed as F6.
- Supporting evidence: E7

## Ruled out

- **F3.n1** (UNCLASSIFIED): discriminator E17
  - Ruled out by: plateau+purge (leak) E17
  - Hypothesis: N-WARM: warm-up/cache fill that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E17
  - Originally supported by: E12
- **F6.n1** (UNCLASSIFIED): discriminator E18
  - Ruled out by: plateau+purge (leak) E18
  - Hypothesis: N-WARM: warm-up/cache fill that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E18
  - Originally supported by: E12

## Blind spots and unknowns

### Coverage gaps

- python/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- python/codeql: missing (codeql is not on PATH or in the tools directory). Install hint: dla install codeql
- python/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)
- python/memray: missing (memray is not installed in the target's python interpreter, so --capture alloc has the tracemalloc report only). Install hint: pip install memray in the interpreter the workload runs under, then dla harness run <H> --capture alloc

### Runtime blind spots

None recorded.

### Entry points not exercisable

None recorded.

### Evidence limitations and unknowns

- E7: partial coverage
- E7: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E7: limitation: writers and removers are textual name matches
- E8: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E8: limitation: workloads were discovered by reading files; none was run
- E9: limitation: harness written; nothing was run (dla harness run measures it)
- E9: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E10: partial coverage (9 of 9 examined)
- E10: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E10: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E10: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E10: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E10: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E11: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E11: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E11: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E11: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E11: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E12: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E12: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E12: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E12: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: partial coverage
- E13: limitation: the agent lists the 300 types with the most instances, so a type outside the list at a capture has no figure there
- E13: limitation: bytes are shallow sys.getsizeof sizes: what an object points to is counted under its own type
- E13: limitation: objects are those the garbage collector tracks, counted after a full collection; the files are ordered as given
- E13: limitation: each histogram lists only the largest types; a type outside the list has no figure
- E14: limitation: the agent lists the 300 types with the most instances, so a type outside the list at a capture has no figure there
- E14: limitation: bytes are shallow sys.getsizeof sizes: what an object points to is counted under its own type
- E14: limitation: objects are those the garbage collector tracks, counted after a full collection; the files are ordered as given
- E14: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E15: limitation: tracemalloc sees allocations made after it started (interpreter start-up excluded) and only through Python's allocators; memory a C extension takes with malloc is not in it
- E15: limitation: each file lists the largest stacks by live bytes and, after the first, the stacks that grew most since the first file: a stack outside the list has no figure in that file
- E15: limitation: stacks are Python frames only, as deep as DLA_TRACEMALLOC_FRAMES (default 10); function names are the module of the frame
- E15: limitation: the files are ordered as given
- E16: limitation: tracemalloc sees allocations made after it started (interpreter start-up excluded) and only through Python's allocators; memory a C extension takes with malloc is not in it
- E16: limitation: each file lists the largest stacks by live bytes and, after the first, the stacks that grew most since the first file: a stack outside the list has no figure in that file
- E16: limitation: stacks are Python frames only, as deep as DLA_TRACEMALLOC_FRAMES (default 10); function names are the module of the frame
- E16: limitation: the files are ordered as given
- E16: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E17: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E17: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E17: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E17: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E17: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E17: limitation: points 1 to 9 come from S1; the extra points from S4
- E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E17: limitation: gate and plateau thresholds are provisional (not calibrated)
- E18: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E18: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E18: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E18: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E18: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E18: limitation: points 1 to 9 come from S1; the extra points from S6
- E18: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E18: limitation: gate and plateau thresholds are provisional (not calibrated)
- E19: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E19: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E19: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E19: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E19: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E19: limitation: points 1 to 9 come from S1; the extra points from S8
- E19: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E19: limitation: gate and plateau thresholds are provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | — |
| E2 | run-config | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | probe/probe.json |
| E4 | static-hit | static | ruff 0.16.9 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | sweep/python-ruff.json |
| E5 | static-hit | static | pylint 4.1.3 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | sweep/python-pylint.json |
| E6 | static-hit | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | sweep/python-dla.json |
| E7 | census-holder | static | dla-census 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | census/python.json |
| E8 | workload | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | workload/workloads.json |
| E9 | workload | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | harness/H1/harness.json |
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (8543c35ef80a) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (ab5696a54f49) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (ab5696a54f49) | trend/T1.json |
| E13 | histogram-diff | live | py-types 0.1.0 | artifact histograms/H1.json (92971017567c) | histograms/H1.json |
| E14 | growth-series | live | py-types 0.1.0 | artifact series/S2.json (b3066361e4d1) | series/S2.json |
| E15 | alloc-stack | live | py-tracemalloc 0.1.0 | artifact allocs/A1.json (956be95bb76c) | allocs/A1.json |
| E16 | growth-series | live | py-tracemalloc 0.1.0 | artifact series/S3.json (fa1f89ce1118) | series/S3.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S5.json (e53007cd946c) | alloc/A1.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S7.json (32d993398da6) | alloc/A2.json |
| E19 | discriminator | test | dla 0.1.0 | artifact series/S9.json (dafe59317a19) | alloc/A3.json |
