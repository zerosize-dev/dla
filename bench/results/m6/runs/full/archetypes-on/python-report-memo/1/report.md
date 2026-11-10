# DLA report

Run: `20261110-202859-b92e`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 6 enumerated, 6 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 4

Residual: heap 2261.388068181818 bytes/iteration unattributed of 2261.388068181818 total (threshold 565.3470170454545) ABOVE THRESHOLD, see O1; heap 2237.1104166666664 bytes/iteration unattributed of 2237.1104166666664 total (threshold 559.2776041666666) ABOVE THRESHOLD, see O1; heap 2354.028693181818 bytes/iteration unattributed of 2354.028693181818 total (threshold 588.5071732954545) ABOVE THRESHOLD, see O1

Coverage: python 7040 iterations, 0 of 0 entry points exercised

## Findings

### F4: **L-COLLECT**

- State: confirmed
- Label: L-COLLECT (evidence: E13 unit-count-series, E7 census-holder)
- Growth: `probe:reports.builder.ReportBuilder.category_totals` 35.95 count/iteration (steady, gate pass, E13)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `reports.builder.ReportBuilder.category_totals`
- Region: heap
- Shape: steady
- Site: `reports/builder.py:14`
- Writers: `reports/builder.py:14`
- Removers: `reports/builder.py:14`
- Annotation: functools.lru_cache(maxsize=None) on a method keys on self; each request builds a new ReportBuilder so entries (and the builder, store, columns) are never evicted
- Verified: agrees (verifier site `reports/builder.py:14`, holder static/global, label L-COLLECT; bundle 285b6a1feb09)
- Sibling hypotheses: F4.n1
- Supporting evidence: E7, E9, E13
- Explanation (inferred): reports/builder.py:14 category_totals is wrapped in lru_cache(maxsize=None), a static/global holder whose key includes self; handle() builds a new ReportBuilder per request so the key never repeats and nothing is evicted.
- Explanation structure: adds: reports/builder.py:14; removal: wrong-key at reports/builder.py:14

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `reports/builder.py:14`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: Duplicate of confirmed F4 (same site reports/builder.py:14); fix F4 by removing lru_cache from the method or caching per store instead of per builder
- Supporting evidence: E4

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `reports/builder.py:14`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: Duplicate of confirmed F4 (same site reports/builder.py:14); fix F4 by removing lru_cache from the method or caching per store instead of per builder
- Supporting evidence: E5

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `reports/builder.py:14`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: Duplicate of confirmed F4 (same site reports/builder.py:14); fix F4 by removing lru_cache from the method or caching per store instead of per builder
- Supporting evidence: E6

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Duplicate of confirmed F4 (same site reports/builder.py:14); fix F4 by removing lru_cache from the method or caching per store instead of per builder
- Supporting evidence: E16

## Ruled out

- **F4.n1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (leak) E14
  - Hypothesis: N-WARM: bounded warm-up cache that plateaus
  - Refuted if: growth continues over the extra cycles and with repeated keys
  - Contradicting evidence: E14
  - Originally supported by: E11

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
- E8: limitation: harness written; nothing was run (dla harness run measures it)
- E8: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E8: limitation: pytest-memray not installed: standard-library driver used
- E9: partial coverage (9 of 9 examined)
- E9: limitation: p1-fresh: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E9: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E9: limitation: p1-repeated: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E9: limitation: no entry points are listed for this runtime: run dla workload discover before the harness to see which ones ran
- E10: limitation: p1-fresh: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E10: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: p1-fresh: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E11: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: p1-repeated: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E12: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: p1-repeated: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E13: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: pytest-memray not installed: standard-library driver used
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: partial coverage (1 of 3 examined)
- E17: limitation: F4: growth evidence is from S2, not S1; not subtracted
- E17: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E17: limitation: threshold is provisional (not calibrated)
- E18: partial coverage (1 of 3 examined)
- E18: limitation: F4: growth measured as a count; bytes not attributed
- E18: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E18: limitation: threshold is provisional (not calibrated)
- E19: partial coverage (1 of 3 examined)
- E19: limitation: F4: growth evidence is from S2, not S3; not subtracted
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
| E1 | run-config | static | dla 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | — |
| E2 | run-config | static | dla 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | probe/probe.json |
| E4 | static-hit | static | ruff 0.16.9 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | sweep/python-ruff.json |
| E5 | static-hit | static | pylint 4.1.3 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | sweep/python-pylint.json |
| E6 | static-hit | static | dla 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | sweep/python-dla.json |
| E7 | census-holder | static | dla-census 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | census/python.json |
| E8 | workload | static | dla 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (e51a0a943402) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (e2cd2ff20300) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (e2cd2ff20300) | trend/T1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S2.json (75464face3d2) | trend/T2.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (75464face3d2) | trend/T2.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S3.json (79093f1e4ef3) | alloc/A1.json |
| E15 | growth-series | test | dla 0.1.0 | artifact series/S3.json (79093f1e4ef3) | trend/T4.json |
| E16 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (79093f1e4ef3) | trend/T4.json |
| E17 | residual | test | dla 0.1.0 | artifact series/S1.json (e2cd2ff20300) | gap/residual-S1-E17.json |
| E18 | residual | test | dla 0.1.0 | artifact series/S2.json (75464face3d2) | gap/residual-S2-E18.json |
| E19 | residual | test | dla 0.1.0 | artifact series/S3.json (79093f1e4ef3) | gap/residual-S3-E19.json |
