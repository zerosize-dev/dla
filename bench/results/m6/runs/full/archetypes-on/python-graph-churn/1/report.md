# DLA report

Run: `20261110-200345-31ef`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 2 enumerated, 2 dispositioned; confirmed 0, strong 0, refuted 2, suspected 0, unresolved 0

Residual: heap 1.224431818181818 bytes/n unattributed of 1.224431818181818 total (threshold 0.5097640909090909) ABOVE THRESHOLD, see O1; heap 0.9458333333333332 bytes/n unattributed of 0.9458333333333332 total (threshold 0.510847159090909) ABOVE THRESHOLD, see O1; heap 1.118452380952381 bytes/n unattributed of 1.118452380952381 total (threshold 0.28434952272731007) ABOVE THRESHOLD, see O1; heap 1.3725378787878788 bytes/n unattributed of 1.3725378787878788 total (threshold 0.6293227840910212) ABOVE THRESHOLD, see O1; heap 1.1894886363636363 bytes/n unattributed of 1.1894886363636363 total (threshold 0.2973721590909091) ABOVE THRESHOLD, see O1; heap 0.6664772727272728 bytes/n unattributed of 0.6664772727272728 total (threshold 0.3337655113636363) ABOVE THRESHOLD, see O2; heap 0.9967021614748888 bytes/n unattributed of 0.9967021614748888 total (threshold 0.2878982050222658) ABOVE THRESHOLD, see O2

Coverage: python 3520 iterations, 13 of 13 entry points exercised

## Findings

No findings to report.

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (non-leak) E13
  - Contradicting evidence: E13
  - Originally supported by: E7
- **F2** (UNCLASSIFIED): discriminator E24
  - Ruled out by: plateau+purge (non-leak) E24
  - Hypothesis: N-HEADROOM: deferred cyclic garbage. Node parent/children/tree cycles in outlines/models.py are only freed by the cyclic GC, which tuning.py:11 delays with gc threshold 40000; py.gc_objects.pre_gc sawtooths to ~50k but post-GC stays flat
  - Refuted if: post-collection object count or RSS keeps growing over extra cycles
  - Contradicting evidence: E24
  - Originally supported by: E19

## Blind spots and unknowns

### Coverage gaps

- python/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- python/codeql: missing (codeql is not on PATH or in the tools directory). Install hint: dla install codeql
- python/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)

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
- E10: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E11: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E13: limitation: points 1 to 9 come from S1; the extra points from S2
- E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- E14: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E14: limitation: points 1 to 9 come from S1; the extra points from S4
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: partial coverage (1 of 3 examined)
- E19: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E19: limitation: threshold is provisional (not calibrated)
- E20: partial coverage (1 of 3 examined)
- E20: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E20: limitation: threshold is provisional (not calibrated)
- E21: partial coverage (1 of 3 examined)
- E21: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E21: limitation: threshold is provisional (not calibrated)
- E22: partial coverage (1 of 3 examined)
- E22: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E22: limitation: threshold is provisional (not calibrated)
- E23: partial coverage (1 of 3 examined)
- E23: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E23: limitation: threshold is provisional (not calibrated)
- E24: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E24: limitation: points 1 to 9 come from S1; the extra points from S6
- E24: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E24: limitation: gate and plateau thresholds are provisional (not calibrated)
- E25: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: partial coverage (1 of 3 examined)
- E27: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E27: limitation: threshold is provisional (not calibrated)
- E28: partial coverage (1 of 3 examined)
- E28: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
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
| E4 | static-hit | static | ruff 0.16.9 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | sweep/python-ruff.json |
| E5 | static-hit | static | pylint 4.1.3 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | sweep/python-pylint.json |
| E6 | static-hit | static | dla 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | sweep/python-dla.json |
| E7 | census-holder | static | dla-census 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | census/python.json |
| E8 | workload | static | dla 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | workload/workloads.json |
| E9 | workload | static | dla 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | harness/H1/harness.json |
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (79bf13656d68) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (77b0e4251f0b) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (77b0e4251f0b) | trend/T1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S3.json (9f78f20a871e) | alloc/A1.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S5.json (eb59fc4f6879) | alloc/A2.json |
| E15 | growth-series | test | dla 0.1.0 | artifact series/S2.json (ac1de3245bb5) | trend/T4.json |
| E16 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (ac1de3245bb5) | trend/T4.json |
| E17 | growth-series | test | dla 0.1.0 | artifact series/S4.json (f649957b596a) | trend/T5.json |
| E18 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (f649957b596a) | trend/T5.json |
| E19 | residual | test | dla 0.1.0 | artifact series/S1.json (77b0e4251f0b) | gap/residual-S1-E19.json |
| E20 | residual | test | dla 0.1.0 | artifact series/S2.json (ac1de3245bb5) | gap/residual-S2-E20.json |
| E21 | residual | test | dla 0.1.0 | artifact series/S3.json (9f78f20a871e) | gap/residual-S3-E21.json |
| E22 | residual | test | dla 0.1.0 | artifact series/S4.json (f649957b596a) | gap/residual-S4-E22.json |
| E23 | residual | test | dla 0.1.0 | artifact series/S5.json (eb59fc4f6879) | gap/residual-S5-E23.json |
| E24 | discriminator | test | dla 0.1.0 | artifact series/S7.json (fbca37440ec5) | alloc/A3.json |
| E25 | growth-series | test | dla 0.1.0 | artifact series/S6.json (bc81281e83da) | trend/T7.json |
| E26 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (bc81281e83da) | trend/T7.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S7.json (fbca37440ec5) | gap/residual-S7-E27.json |
| E28 | residual | test | dla 0.1.0 | artifact series/S6.json (bc81281e83da) | gap/residual-S6-E28.json |
