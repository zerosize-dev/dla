# DLA report

Run: `20261110-173412-1aaf`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 4 enumerated, 4 dispositioned; confirmed 0, strong 0, refuted 2, suspected 0, unresolved 2

Residual: heap 2.0142045454545454 bytes/n unattributed of 2.0142045454545454 total (threshold 1.0541863636363638) ABOVE THRESHOLD, see O2; heap 1.143712121212121 bytes/n unattributed of 1.143712121212121 total (threshold 0.42362406818195264) ABOVE THRESHOLD, see O2; heap 1.1652462121212122 bytes/n unattributed of 1.1652462121212122 total (threshold 0.34946398295454545) ABOVE THRESHOLD, see O2; heap 1.1295454545454544 bytes/n unattributed of 1.1295454545454544 total (threshold 0.2823863636363636) ABOVE THRESHOLD, see O2; heap 1.0036363636363637 bytes/n unattributed of 1.0036363636363637 total (threshold 0.2509090909090909) ABOVE THRESHOLD, see O2

Coverage: python 3520 iterations, 13 of 13 entry points exercised

## Findings

### F1.n1: Hypothesis N-HEADROOM: parent/child/tree reference cycles are reclaimed only when gen0 reaches the raised threshold (outlines/tuning.py:11), so memory is a bounded sawtooth (UNCLASSIFIED)

- State: unresolved(not_exercised)
- Evidence classes: discriminator
- Sibling hypothesis of: F1
- Hypothesis: N-HEADROOM: parent/child/tree reference cycles are reclaimed only when gen0 reaches the raised threshold (outlines/tuning.py:11), so memory is a bounded sawtooth
- Refuted if: growth continues over the extra cycles
- Reason: not_exercised
- Remedy: sibling N-HEADROOM stands (F1 refuted by E15); no further action unless growth continues past plateau
- Supporting evidence: E15

### F2.n1: Hypothesis N-HEADROOM: cyclic Node/Tree garbage waits for the raised gen0 threshold (outlines/tuning.py:11), bounded sawtooth (UNCLASSIFIED)

- State: unresolved(not_exercised)
- Evidence classes: discriminator
- Sibling hypothesis of: F2
- Hypothesis: N-HEADROOM: cyclic Node/Tree garbage waits for the raised gen0 threshold (outlines/tuning.py:11), bounded sawtooth
- Refuted if: growth continues over the extra cycles
- Reason: not_exercised
- Remedy: sibling N-HEADROOM stands (F2 refuted by E19)
- Supporting evidence: E19

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (non-leak) E15
  - Contradicting evidence: E15
  - Originally supported by: E8
- **F2** (UNCLASSIFIED): discriminator E19
  - Ruled out by: plateau+purge (non-leak) E19
  - Contradicting evidence: E19
  - Originally supported by: E16

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

- E8: partial coverage
- E8: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E8: limitation: writers and removers are textual name matches
- E9: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E9: limitation: workloads were discovered by reading files; none was run
- E10: limitation: harness written; nothing was run (dla harness run measures it)
- E10: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E11: partial coverage (9 of 9 examined)
- E11: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E11: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E11: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E11: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E11: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
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
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: partial coverage (1 of 3 examined)
- E14: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E14: limitation: threshold is provisional (not calibrated)
- E15: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E15: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E15: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E15: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E15: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E15: limitation: points 1 to 9 come from S1; the extra points from S2
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: partial coverage (1 of 3 examined)
- E18: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E18: limitation: threshold is provisional (not calibrated)
- E19: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E19: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E19: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E19: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E19: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E19: limitation: points 1 to 9 come from S1; the extra points from S4
- E19: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E19: limitation: gate and plateau thresholds are provisional (not calibrated)
- E20: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E20: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E20: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E20: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E20: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E20: limitation: points 1 to 9 come from S1; the extra points from S4
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E21: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E21: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E21: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E21: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E21: limitation: points 1 to 9 come from S1; the extra points from S4
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: partial coverage (1 of 3 examined)
- E24: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E24: limitation: threshold is provisional (not calibrated)
- E25: partial coverage (1 of 3 examined)
- E25: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E25: limitation: threshold is provisional (not calibrated)
- E26: partial coverage (1 of 3 examined)
- E26: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E26: limitation: threshold is provisional (not calibrated)
- E27: partial coverage (1 of 3 examined)
- E27: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E27: limitation: threshold is provisional (not calibrated)
- E28: partial coverage (1 of 3 examined)
- E28: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E28: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | — |
| E2 | run-config | static | dla 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | — |
| E3 | probe | static | dla-probe 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | probe/probe.json |
| E4 | probe | static | dla-probe 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | probe/probe.json |
| E5 | static-hit | static | ruff 0.16.9 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | sweep/python-ruff.json |
| E6 | static-hit | static | pylint 4.1.3 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | sweep/python-pylint.json |
| E7 | static-hit | static | dla 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | sweep/python-dla.json |
| E8 | census-holder | static | dla-census 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | census/python.json |
| E9 | workload | static | dla 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | workload/workloads.json |
| E10 | workload | static | dla 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | harness/H1/harness.json |
| E11 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (672b09c20a5f) | harness/H1/run-S1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S1.json (d1cb20ce02ac) | trend/T1.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (d1cb20ce02ac) | trend/T1.json |
| E14 | residual | test | dla 0.1.0 | artifact series/S1.json (d1cb20ce02ac) | gap/residual-S1-E14.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S3.json (9e3b072cd9d3) | alloc/A1.json |
| E16 | growth-series | test | dla 0.1.0 | artifact series/S2.json (68c5a3d55cf6) | trend/T3.json |
| E17 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (68c5a3d55cf6) | trend/T3.json |
| E18 | residual | test | dla 0.1.0 | artifact series/S1.json (d1cb20ce02ac) | gap/residual-S1-E18.json |
| E19 | discriminator | test | dla 0.1.0 | artifact series/S5.json (3fa741ab09da) | alloc/A2.json |
| E20 | growth-series | test | dla 0.1.0 | artifact series/S5.json (3fa741ab09da) | trend/T5.json |
| E21 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (3fa741ab09da) | trend/T5.json |
| E22 | growth-series | test | dla 0.1.0 | artifact series/S4.json (f2d30d450100) | trend/T6.json |
| E23 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (f2d30d450100) | trend/T6.json |
| E24 | residual | test | dla 0.1.0 | artifact series/S1.json (d1cb20ce02ac) | gap/residual-S1-E24.json |
| E25 | residual | test | dla 0.1.0 | artifact series/S2.json (68c5a3d55cf6) | gap/residual-S2-E25.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S3.json (9e3b072cd9d3) | gap/residual-S3-E26.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S4.json (f2d30d450100) | gap/residual-S4-E27.json |
| E28 | residual | test | dla 0.1.0 | artifact series/S5.json (3fa741ab09da) | gap/residual-S5-E28.json |
