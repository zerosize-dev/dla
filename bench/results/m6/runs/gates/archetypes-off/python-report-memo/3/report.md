# DLA report

Run: `20261110-193826-760f`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 11 enumerated, 11 dispositioned; confirmed 0, strong 1, refuted 1, suspected 3, unresolved 6

Residual: heap 2371.978522727273 bytes/n unattributed of 2371.978522727273 total (threshold 592.9946306818182) ABOVE THRESHOLD, see O1; heap 2324.2 bytes/n unattributed of 2324.2 total (threshold 581.05) ABOVE THRESHOLD, see O1; heap 2344.0154356060607 bytes/n unattributed of 2344.0154356060607 total (threshold 586.0038589015152) ABOVE THRESHOLD, see O1; heap 0.5041666666666667 bytes/n unattributed of 0.5041666666666667 total (threshold 0.3243187499998879) ABOVE THRESHOLD, see O1

Coverage: python 3520 iterations, 10 of 10 entry points exercised

## Findings

### F4: **UNCLASSIFIED**

- State: strong
- Label: UNCLASSIFIED (Process-lifetime functools.lru_cache(maxsize=None) on method ReportBuilder.category_totals keys on self; a new ReportBuilder is created per request so the cache pins every builder, its column buffers and the cached totals, never evicted)
- Growth: `probe:reports.builder.ReportBuilder.category_totals` 36.73 count/n (steady, gate pass, E12)
- Evidence classes: context, discriminator, growth, intervention, static
- Strong: second class E26 intervention that removed the growth, independent of the growth record E12 (dla)
- Holder: static/global
- Owner: `reports.builder.ReportBuilder.category_totals`
- Region: heap
- Shape: steady
- Site: `reports/builder.py:14`
- Writers: `reports/builder.py:14`
- Annotation: functools.lru_cache(maxsize=None) on a method keys on self; each request builds a new ReportBuilder, so the global cache pins every builder (and its column buffers) forever
- Verified: no
- Sibling hypotheses: F4.n1
- Supporting evidence: E7, E10, E12, E17, E26
- Explanation (inferred): ReportBuilder.category_totals at reports/builder.py:14 is wrapped in an unbounded lru_cache, a static/global holder; each request's new builder (self) is a key, so entries are added per request and never removed or evicted.
- Explanation structure: adds: reports/builder.py:14; removal: never

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Site: `reports/builder.py:14`
- Reason: not_exercised
- Remedy: Static detector hit at the same site and mechanism as confirmed F4 (reports/builder.py:14); covered by F4's harness confirmation, no separate exercise
- Supporting evidence: E4

### F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Site: `reports/builder.py:14`
- Reason: not_exercised
- Remedy: Static detector hit at the same site and mechanism as confirmed F4 (reports/builder.py:14); covered by F4's harness confirmation, no separate exercise
- Supporting evidence: E5

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Site: `reports/builder.py:14`
- Reason: not_exercised
- Remedy: Static detector hit at the same site and mechanism as confirmed F4 (reports/builder.py:14); covered by F4's harness confirmation, no separate exercise
- Supporting evidence: E6

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Growth column is a symptom of instances pinned by confirmed F4 (reports/builder.py:14 lru_cache); no independent holder; re-measure after removing the cache
- Supporting evidence: E18

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Growth column is a symptom of instances pinned by confirmed F4 (reports/builder.py:14 lru_cache); no independent holder; re-measure after removing the cache
- Supporting evidence: E19

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Growth column is a symptom of instances pinned by confirmed F4 (reports/builder.py:14 lru_cache); no independent holder; re-measure after removing the cache
- Supporting evidence: E23

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Growth column is a symptom of instances pinned by confirmed F4 (reports/builder.py:14 lru_cache); no independent holder; re-measure after removing the cache
- Supporting evidence: E25

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Growth column is a symptom of instances pinned by confirmed F4 (reports/builder.py:14 lru_cache); no independent holder; re-measure after removing the cache
- Supporting evidence: E20

### F10: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Growth column is a symptom of instances pinned by confirmed F4 (reports/builder.py:14 lru_cache); no independent holder; re-measure after removing the cache
- Supporting evidence: E21

## Ruled out

- **F4.n1** (UNCLASSIFIED): discriminator E17
  - Ruled out by: plateau+purge (leak) E17
  - Hypothesis: N-WARM: bounded warm-up cache fill that plateaus
  - Refuted if: growth continues over the extra cycles and persists after purge
  - Contradicting evidence: E17
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
- E18: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E18: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E19: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E20: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E21: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E24: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E24: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E24: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E24: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E24: limitation: points 1 to 9 come from S1; the extra points from S4
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E25: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E25: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E25: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E25: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E25: limitation: points 1 to 9 come from S1; the extra points from S4
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: tested in a scratch copy; the project was not changed
- E26: limitation: gate thresholds are provisional (not calibrated); the 0.2 and 0.5 slope fractions are placeholders
- E26: limitation: exercised is null: the harness records entry points, not lines, so whether the edited line ran is not checked
- E26: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E26: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E27: partial coverage (1 of 3 examined)
- E27: limitation: F4: growth measured as a count; bytes not attributed
- E27: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E27: limitation: threshold is provisional (not calibrated)
- E28: partial coverage (1 of 3 examined)
- E28: limitation: F4: growth evidence is from S1, not S4; not subtracted
- E28: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E28: limitation: threshold is provisional (not calibrated)
- E29: partial coverage (1 of 3 examined)
- E29: limitation: F4: growth evidence is from S1, not S5; not subtracted
- E29: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E29: limitation: threshold is provisional (not calibrated)
- E30: partial coverage (1 of 3 examined)
- E30: limitation: F4: growth evidence is from S1, not S6; not subtracted
- E30: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E30: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | — |
| E2 | run-config | static | dla 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | probe/probe.json |
| E4 | static-hit | static | ruff 0.16.9 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | sweep/python-ruff.json |
| E5 | static-hit | static | pylint 4.1.3 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | sweep/python-pylint.json |
| E6 | static-hit | static | dla 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | sweep/python-dla.json |
| E7 | census-holder | static | dla-census 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | census/python.json |
| E8 | workload | static | dla 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | workload/workloads.json |
| E9 | workload | static | dla 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | harness/H1/harness.json |
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (b94f6e544261) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (64ad5543c622) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (64ad5543c622) | trend/T1.json |
| E13 | histogram-diff | live | py-types 0.1.0 | artifact histograms/H1.json (23e8ebd1ee72) | histograms/H1.json |
| E14 | growth-series | live | py-types 0.1.0 | artifact series/S2.json (6909e4a4f4cd) | series/S2.json |
| E15 | alloc-stack | live | py-tracemalloc 0.1.0 | artifact allocs/A1.json (be40321cf475) | allocs/A1.json |
| E16 | growth-series | live | py-tracemalloc 0.1.0 | artifact series/S3.json (2f29b11375df) | series/S3.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S5.json (f5bb8208029f) | alloc/A1.json |
| E18 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (6909e4a4f4cd) | trend/T3.json |
| E19 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (6909e4a4f4cd) | trend/T3.json |
| E20 | growth-series | dump | dla 0.1.0 | artifact series/S3.json (2f29b11375df) | trend/T4.json |
| E21 | unit-count-series | dump | dla 0.1.0 | artifact series/S3.json (2f29b11375df) | trend/T4.json |
| E22 | growth-series | test | dla 0.1.0 | artifact series/S4.json (508aaa993c1a) | trend/T5.json |
| E23 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (508aaa993c1a) | trend/T5.json |
| E24 | growth-series | test | dla 0.1.0 | artifact series/S5.json (f5bb8208029f) | trend/T6.json |
| E25 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (f5bb8208029f) | trend/T6.json |
| E26 | intervention | test | dla 0.1.0 | artifact series/S6.json (bb5ef032050c) | intervene/I1 |
| E27 | residual | test | dla 0.1.0 | artifact series/S1.json (64ad5543c622) | gap/residual-S1-E27.json |
| E28 | residual | test | dla 0.1.0 | artifact series/S4.json (508aaa993c1a) | gap/residual-S4-E28.json |
| E29 | residual | test | dla 0.1.0 | artifact series/S5.json (f5bb8208029f) | gap/residual-S5-E29.json |
| E30 | residual | test | dla 0.1.0 | artifact series/S6.json (bb5ef032050c) | gap/residual-S6-E30.json |
