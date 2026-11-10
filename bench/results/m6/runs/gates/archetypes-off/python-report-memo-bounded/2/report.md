# DLA report

Run: `20261110-190624-de39`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 5 enumerated, 5 dispositioned; confirmed 0, strong 0, refuted 3, suspected 0, unresolved 2

Residual: heap 0.7647727272727273 bytes/n unattributed of 0.7647727272727273 total (threshold 0.6737406136363636) ABOVE THRESHOLD, see O1; heap 0.5765151515151515 bytes/n unattributed of 0.5765151515151515 total (threshold 0.32606832167844235) ABOVE THRESHOLD, see O2; heap 0.4837121212121212 bytes/n unattributed of 0.4837121212121212 total (threshold 0.47542990384603306) ABOVE THRESHOLD, see O2; heap 0.5768181818181818 bytes/n unattributed of 0.5768181818181818 total (threshold 0.41632030069922826) ABOVE THRESHOLD, see O2

Coverage: python 6160 iterations, 10 of 10 entry points exercised

## Findings

### F2.n1: Hypothesis N-WARM (UNCLASSIFIED)

- State: unresolved(not_exercised)
- Evidence classes: discriminator
- Sibling hypothesis of: F2
- Hypothesis: N-WARM
- Refuted if: growth continues over the extra cycles
- Reason: not_exercised
- Remedy: Non-leak sibling N-WARM; its parent was refuted by plateau (no growth over extra cycles)
- Supporting evidence: E14

### F3.n1: Hypothesis N-WARM (UNCLASSIFIED)

- State: unresolved(not_exercised)
- Evidence classes: discriminator
- Sibling hypothesis of: F3
- Hypothesis: N-WARM
- Refuted if: growth continues over the extra cycles
- Reason: not_exercised
- Remedy: Non-leak sibling N-WARM; its parent was refuted by plateau (no growth over extra cycles)
- Supporting evidence: E13

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (non-leak) E15
  - Contradicting evidence: E15
  - Originally supported by: E4
- **F2** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (non-leak) E14
  - Contradicting evidence: E14
  - Originally supported by: E7
- **F3** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (non-leak) E13
  - Contradicting evidence: E13
  - Originally supported by: E7

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
- E10: partial coverage (12 of 12 examined)
- E10: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E10: limitation: p5-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E10: limitation: p8-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E10: limitation: p12-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E10: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E11: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E11: limitation: p5-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E11: limitation: p8-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E11: limitation: p12-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E11: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E12: limitation: p5-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E12: limitation: p8-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E12: limitation: p12-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E12: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- E14: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: partial coverage (1 of 3 examined)
- E16: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E16: limitation: threshold is provisional (not calibrated)
- E17: limitation: tracemalloc sees allocations made after it started (interpreter start-up excluded) and only through Python's allocators; memory a C extension takes with malloc is not in it
- E17: limitation: each file lists the largest stacks by live bytes and, after the first, the stacks that grew most since the first file: a stack outside the list has no figure in that file
- E17: limitation: stacks are Python frames only, as deep as DLA_TRACEMALLOC_FRAMES (default 10); function names are the module of the frame
- E17: limitation: the files are ordered as given
- E18: limitation: tracemalloc sees allocations made after it started (interpreter start-up excluded) and only through Python's allocators; memory a C extension takes with malloc is not in it
- E18: limitation: each file lists the largest stacks by live bytes and, after the first, the stacks that grew most since the first file: a stack outside the list has no figure in that file
- E18: limitation: stacks are Python frames only, as deep as DLA_TRACEMALLOC_FRAMES (default 10); function names are the module of the frame
- E18: limitation: the files are ordered as given
- E18: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E19: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E19: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E20: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: partial coverage (1 of 3 examined)
- E21: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E21: limitation: threshold is provisional (not calibrated)
- E22: partial coverage (1 of 3 examined)
- E22: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E22: limitation: threshold is provisional (not calibrated)
- E23: partial coverage (1 of 3 examined)
- E23: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E23: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo 193963928cbca22594b461b4b8d3fe954c299cf0 | — |
| E2 | run-config | static | dla 0.1.0 | repo 193963928cbca22594b461b4b8d3fe954c299cf0 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 193963928cbca22594b461b4b8d3fe954c299cf0 | probe/probe.json |
| E4 | static-hit | static | ruff 0.16.9 | repo 193963928cbca22594b461b4b8d3fe954c299cf0 | sweep/python-ruff.json |
| E5 | static-hit | static | pylint 4.1.3 | repo 193963928cbca22594b461b4b8d3fe954c299cf0 | sweep/python-pylint.json |
| E6 | static-hit | static | dla 0.1.0 | repo 193963928cbca22594b461b4b8d3fe954c299cf0 | sweep/python-dla.json |
| E7 | census-holder | static | dla-census 0.1.0 | repo 193963928cbca22594b461b4b8d3fe954c299cf0 | census/python.json |
| E8 | workload | static | dla 0.1.0 | repo 193963928cbca22594b461b4b8d3fe954c299cf0 | workload/workloads.json |
| E9 | workload | static | dla 0.1.0 | repo 193963928cbca22594b461b4b8d3fe954c299cf0 | harness/H1/harness.json |
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (dd1edb799439) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (9677a2e2c0dc) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (9677a2e2c0dc) | trend/T1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S2.json (58bf02102290) | alloc/A1.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S3.json (905d60cf0fdd) | alloc/A2.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S4.json (d93c06d4c22d) | alloc/A3.json |
| E16 | residual | test | dla 0.1.0 | artifact series/S1.json (9677a2e2c0dc) | gap/residual-S1-E16.json |
| E17 | alloc-stack | live | py-tracemalloc 0.1.0 | artifact allocs/A1.json (0c298de45a6c) | allocs/A1.json |
| E18 | growth-series | live | py-tracemalloc 0.1.0 | artifact series/S5.json (120ea2c32d05) | series/S5.json |
| E19 | growth-series | dump | dla 0.1.0 | artifact series/S5.json (120ea2c32d05) | trend/T5.json |
| E20 | unit-count-series | dump | dla 0.1.0 | artifact series/S5.json (120ea2c32d05) | trend/T5.json |
| E21 | residual | test | dla 0.1.0 | artifact series/S2.json (58bf02102290) | gap/residual-S2-E21.json |
| E22 | residual | test | dla 0.1.0 | artifact series/S3.json (905d60cf0fdd) | gap/residual-S3-E22.json |
| E23 | residual | test | dla 0.1.0 | artifact series/S4.json (d93c06d4c22d) | gap/residual-S4-E23.json |
