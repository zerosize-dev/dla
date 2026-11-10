# DLA report

Run: `20261110-203408-b26c`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 3 enumerated, 3 dispositioned; confirmed 0, strong 0, refuted 3, suspected 0, unresolved 0

Residual: heap 1.9076515151515152 bytes/n unattributed of 1.9076515151515152 total (threshold 3.9312968181817056); heap 1.3232954545454545 bytes/n unattributed of 1.3232954545454545 total (threshold 0.6289015909090908) ABOVE THRESHOLD, see O1; heap 0.5963068181818182 bytes/n unattributed of 0.5963068181818182 total (threshold 0.535336534090909) ABOVE THRESHOLD, see O1; heap 0.9481818181818182 bytes/n unattributed of 0.9481818181818182 total (threshold 0.835406590909427) ABOVE THRESHOLD, see O1; heap 0.5742694805194806 bytes/n unattributed of 0.5742694805194806 total (threshold 0.5984192386361394); heap 0.5833333333333334 bytes/n unattributed of 0.5833333333333334 total (threshold 0.472699090909315) ABOVE THRESHOLD, see O1; heap 0.5333333333333334 bytes/n unattributed of 0.5333333333333334 total (threshold 0.31067209090909087) ABOVE THRESHOLD, see O1; heap 0.18636363636363631 bytes/n unattributed of 0.18636363636363631 total (threshold 0.5307034090909091); heap 0.39642734159779613 bytes/n unattributed of 0.39642734159779613 total (threshold 0.6533574163222332); heap 0.9929545454545454 bytes/n unattributed of 0.9929545454545454 total (threshold 0.7073879318183163) ABOVE THRESHOLD, see O1; heap 0.5647727272727272 bytes/n unattributed of 0.5647727272727272 total (threshold 0.5094752727272727) ABOVE THRESHOLD, see O1

Coverage: python 3520 iterations, 10 of 10 entry points exercised

## Findings

No findings to report.

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (non-leak) E13
  - Contradicting evidence: E13
  - Originally supported by: E4
- **F2** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (non-leak) E13
  - Contradicting evidence: E13
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
- E13: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E13: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E13: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E13: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E13: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E13: limitation: points 1 to 9 come from S1; the extra points from S2
- E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- E14: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E14: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E14: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E14: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E14: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E14: limitation: points 1 to 9 come from S1; the extra points from S4
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E15: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E15: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E15: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E15: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E15: limitation: points 1 to 9 come from S1; the extra points from S6
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E16: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E16: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E16: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E16: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E16: limitation: points 1 to 9 come from S1; the extra points from S8
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E17: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E17: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E17: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E17: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E17: limitation: points 1 to 9 come from S1; the extra points from S10
- E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E17: limitation: gate and plateau thresholds are provisional (not calibrated)
- E18: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: partial coverage (1 of 3 examined)
- E28: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E28: limitation: threshold is provisional (not calibrated)
- E29: partial coverage (1 of 3 examined)
- E29: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E29: limitation: threshold is provisional (not calibrated)
- E30: partial coverage (1 of 3 examined)
- E30: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E30: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E30: limitation: threshold is provisional (not calibrated)
- E31: partial coverage (1 of 3 examined)
- E31: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E31: limitation: threshold is provisional (not calibrated)
- E32: partial coverage (1 of 3 examined)
- E32: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E32: limitation: threshold is provisional (not calibrated)
- E33: partial coverage (1 of 3 examined)
- E33: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E33: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E33: limitation: threshold is provisional (not calibrated)
- E34: partial coverage (1 of 3 examined)
- E34: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E34: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E34: limitation: threshold is provisional (not calibrated)
- E35: partial coverage (1 of 3 examined)
- E35: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E35: limitation: threshold is provisional (not calibrated)
- E36: partial coverage (1 of 3 examined)
- E36: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E36: limitation: threshold is provisional (not calibrated)
- E37: partial coverage (1 of 3 examined)
- E37: limitation: off-heap: no gap series for S10; run dla gap S10 first (node.external is used when present)
- E37: limitation: threshold is provisional (not calibrated)
- E38: partial coverage (1 of 3 examined)
- E38: limitation: off-heap: no gap series for S11; run dla gap S11 first (node.external is used when present)
- E38: limitation: threshold is provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | — |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | — |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | — |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E3 | — |
| `intake.build` | Does the project build locally? | answered | E3 | — |
| `intake.workload` | What can exercise the suspected path? | answered | E8 | — |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
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
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (7fa48e4aa891) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (3072429dd17e) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (3072429dd17e) | trend/T1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S3.json (9321b1037452) | alloc/A1.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S5.json (b792b7bfe099) | alloc/A2.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S7.json (624451ae82d2) | alloc/A3.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S9.json (b027b6a8f8db) | alloc/A4.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S11.json (c4474f703499) | alloc/A5.json |
| E18 | growth-series | test | dla 0.1.0 | artifact series/S2.json (177d27567798) | trend/T7.json |
| E19 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (177d27567798) | trend/T7.json |
| E20 | growth-series | test | dla 0.1.0 | artifact series/S4.json (987faa4ea87c) | trend/T8.json |
| E21 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (987faa4ea87c) | trend/T8.json |
| E22 | growth-series | test | dla 0.1.0 | artifact series/S6.json (587a5cbf4247) | trend/T9.json |
| E23 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (587a5cbf4247) | trend/T9.json |
| E24 | growth-series | test | dla 0.1.0 | artifact series/S8.json (4c78fedde12a) | trend/T10.json |
| E25 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (4c78fedde12a) | trend/T10.json |
| E26 | growth-series | test | dla 0.1.0 | artifact series/S10.json (72ce6805c3b4) | trend/T11.json |
| E27 | unit-count-series | test | dla 0.1.0 | artifact series/S10.json (72ce6805c3b4) | trend/T11.json |
| E28 | residual | test | dla 0.1.0 | artifact series/S1.json (3072429dd17e) | gap/residual-S1-E28.json |
| E29 | residual | test | dla 0.1.0 | artifact series/S2.json (177d27567798) | gap/residual-S2-E29.json |
| E30 | residual | test | dla 0.1.0 | artifact series/S3.json (9321b1037452) | gap/residual-S3-E30.json |
| E31 | residual | test | dla 0.1.0 | artifact series/S4.json (987faa4ea87c) | gap/residual-S4-E31.json |
| E32 | residual | test | dla 0.1.0 | artifact series/S5.json (b792b7bfe099) | gap/residual-S5-E32.json |
| E33 | residual | test | dla 0.1.0 | artifact series/S6.json (587a5cbf4247) | gap/residual-S6-E33.json |
| E34 | residual | test | dla 0.1.0 | artifact series/S7.json (624451ae82d2) | gap/residual-S7-E34.json |
| E35 | residual | test | dla 0.1.0 | artifact series/S8.json (4c78fedde12a) | gap/residual-S8-E35.json |
| E36 | residual | test | dla 0.1.0 | artifact series/S9.json (b027b6a8f8db) | gap/residual-S9-E36.json |
| E37 | residual | test | dla 0.1.0 | artifact series/S10.json (72ce6805c3b4) | gap/residual-S10-E37.json |
| E38 | residual | test | dla 0.1.0 | artifact series/S11.json (c4474f703499) | gap/residual-S11-E38.json |
