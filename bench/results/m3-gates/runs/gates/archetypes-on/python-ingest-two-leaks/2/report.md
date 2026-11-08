# DLA report

Run: `20261108-125918-6fe0`

Status: COMPLETE

Findings: 8 enumerated, 8 dispositioned; confirmed 2, strong 0, refuted 2, suspected 0, unresolved 4

Residual: heap 117.09126082251078 bytes/n unattributed of 2678.7893668831166 total (threshold 669.6973417207791)

Coverage: python 3520 iterations, 8 of 11 entry points exercised

## Findings

### F3: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:ingest.parser._newest.bytes` 655.5 bytes/n (steady, gate pass, E12)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `ingest.parser._newest`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:3`
- Writers: `ingest/parser.py:29`
- Annotation: heuristic name match over the project's sources
- Annotation: module global _newest; each Record.previous links to prior record, never cleared -> unbounded linked list of all Records
- Sibling hypotheses: F3.n1
- Supporting evidence: E8, E11, E12

### F6: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:ingest.registry._LISTENERS.bytes` 1906 bytes/n (steady, gate pass, E12)
- Evidence classes: context, growth, static
- Holder: registration-table
- Owner: `ingest.registry._LISTENERS`
- Region: heap
- Shape: steady
- Site: `ingest/registry.py:3`
- Writers: `ingest/registry.py:7`
- Removers: `ingest/registry.py:24`
- Annotation: heuristic name match over the project's sources
- Annotation: remover reset() only used by tests; never in pipeline. Each watcher also holds marks dict of known sensors
- Sibling hypotheses: F6.n1
- Supporting evidence: E8, E11, E12

### F1: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: context, static
- Site: `ingest/parser.py:29`
- Annotation: same as F3
- Reason: time_budget
- Remedy: duplicate/site of confirmed F3 or F6 (same mechanism); no separate disposition needed
- Supporting evidence: E7, E18

### F2: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: context, static
- Site: `ingest/registry.py:3`
- Annotation: same as F6/F5
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: time_budget
- Remedy: duplicate/site of confirmed F3 or F6 (same mechanism); no separate disposition needed
- Supporting evidence: E7, E18

### F4: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: context, static
- Holder: container-field
- Owner: `ingest.pipeline.Pipeline.per_sensor`
- Region: heap
- Site: `ingest/pipeline.py:15`
- Writers: `ingest/pipeline.py:25`
- Annotation: heuristic name match over the project's sources
- Annotation: keyed by sensor (64 fixed) - bounded
- Reason: time_budget
- Remedy: per_sensor is keyed by 64 fixed sensors and has no growing probe column; residual after F3/F6 is under threshold. No action expected
- Supporting evidence: E8, E18

### F5: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: context, static
- Holder: registration-table
- Owner: `ingest.pipeline.Pipeline.process_batch`
- Region: heap
- Site: `ingest/pipeline.py:28`
- Writers: `ingest/pipeline.py:28`
- Annotation: subscribes a new CheckpointWatcher every batch, no unsubscribe
- Reason: time_budget
- Remedy: duplicate/site of confirmed F3 or F6 (same mechanism); no separate disposition needed
- Supporting evidence: E8, E18

## Ruled out

- **F3.n1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (leak) E14
  - Hypothesis: N-WARM cache warm-up plateau
  - Refuted if: growth continues over the extra cycles after purge
  - Contradicting evidence: E14
  - Originally supported by: E13
- **F6.n1** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (leak) E15
  - Hypothesis: N-WARM cache warm-up plateau
  - Refuted if: growth continues over the extra cycles after purge
  - Contradicting evidence: E15
  - Originally supported by: E13

## Blind spots and unknowns

### Coverage gaps

- python/codeql: missing (codeql is not on PATH or in the tools directory). Install hint: dla install codeql

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
- E11: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E12: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E14: limitation: points 1 to 9 come from S1; the extra points from S2
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E15: limitation: points 1 to 9 come from S1; the extra points from S4
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E16: limitation: points 1 to 9 come from S1; the extra points from S6
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E17: limitation: points 1 to 9 come from S1; the extra points from S8
- E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E17: limitation: gate and plateau thresholds are provisional (not calibrated)
- E18: partial coverage (1 of 3 examined)
- E18: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E18: limitation: threshold is provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | ingest worker, python package ingest/ |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | whole ingest package |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | RSS grows with batches processed; nightly restart |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E3 | python |
| `intake.build` | Does the project build locally? | answered | E4 | python build ok |
| `intake.workload` | What can exercise the suspected path? | answered | E9 | python -m ingest.load --iterations N |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 raw values |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 60m |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | — |
| E2 | run-config | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | probe/probe.json |
| E4 | probe | static | dla-probe 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | probe/probe.json |
| E5 | static-hit | static | ruff 0.16.9 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | sweep/ruff.json |
| E6 | static-hit | static | pylint 4.1.3 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | sweep/pylint.json |
| E7 | static-hit | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | sweep/dla.json |
| E8 | census-holder | static | dla-census 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | census/python.json |
| E9 | workload | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | workload/workloads.json |
| E10 | workload | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | harness/H1/harness.json |
| E11 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (ab0630a0a4c1) | harness/H1/run-S1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S1.json (1cac5e22c083) | trend/T1.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (1cac5e22c083) | trend/T1.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S3.json (7eeaa5c59a4a) | alloc/A1.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S5.json (b3a3eb7d3efd) | alloc/A2.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S7.json (fbd781b78e27) | alloc/A3.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S9.json (ab58c40ccbb9) | alloc/A4.json |
| E18 | residual | test | dla 0.1.0 | artifact series/S1.json (1cac5e22c083) | gap/residual-S1-E18.json |
