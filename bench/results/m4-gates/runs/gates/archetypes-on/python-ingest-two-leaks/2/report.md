# DLA report

Run: `20261108-200919-8563`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 8 enumerated, 8 dispositioned; confirmed 2, strong 0, refuted 2, suspected 0, unresolved 4

Residual: not measured

Coverage: python 3520 iterations, 8 of 11 entry points exercised

## Findings

### F3: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:ingest.parser._newest.bytes` 655.5 bytes/n (steady, gate pass, E11)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `ingest.parser._newest`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:3`
- Writers: `ingest/parser.py:29`
- Annotation: heuristic name match over the project's sources
- Annotation: parse_line stores each Record in module global _newest and sets record.previous = old _newest, building an unbounded singly linked chain of every Record ever parsed; nothing ever cuts previous. Only the prior value is needed for delta, so the link should not be retained (class of repair: keep just the last value, not the Record chain).
- Sibling hypotheses: F3.n1
- Supporting evidence: E7, E10, E11

### F6: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:ingest.registry._LISTENERS.bytes` 1906 bytes/n (steady, gate pass, E11)
- Evidence classes: context, growth, static
- Holder: registration-table
- Owner: `ingest.registry._LISTENERS`
- Region: heap
- Shape: steady
- Site: `ingest/registry.py:3`
- Writers: `ingest/registry.py:7`, `ingest/pipeline.py:28`
- Removers: `ingest/registry.py:24`, `ingest/registry.py:20`
- Annotation: heuristic name match over the project's sources
- Annotation: Pipeline.process_batch subscribes a new CheckpointWatcher per batch to registry._LISTENERS and never unsubscribes; reset() is test-only. Each watcher holds a dict of marks over all known sensors plus a sensors tuple, so memory grows per batch (class of repair: deregister the listener once fired/no longer needed).
- Sibling hypotheses: F6.n1
- Supporting evidence: E7, E10, E11

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `ingest/parser.py:29`
- Reason: not_exercised
- Remedy: Detector hit at parser.py:29 is the same site as confirmed F3; no separate growth series. No action beyond F3.
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `ingest/registry.py:3`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: Detector hit at registry.py:3 is the same holder as confirmed F6; no separate growth series. No action beyond F6.
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `ingest.pipeline.Pipeline.per_sensor`
- Region: heap
- Site: `ingest/pipeline.py:15`
- Writers: `ingest/pipeline.py:25`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: per_sensor is keyed by 64 sensor ids so is bounded by design; its probe column did not grow independently (alloc-test only saw process-wide growth explained by F3/F6). Re-check if sensor id cardinality becomes unbounded.
- Supporting evidence: E7

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `ingest.pipeline.Pipeline.process_batch`
- Region: heap
- Site: `ingest/pipeline.py:28`
- Reason: not_exercised
- Remedy: process_batch is the subscribe call site of F6, not a separate holder; process-wide growth is explained by F3/F6. Fixing F6 covers it.
- Supporting evidence: E7

## Ruled out

- **F3.n1** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (leak) E13
  - Hypothesis: N-WARM: bounded warm-up cache
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E13
  - Originally supported by: E12
- **F6.n1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (leak) E14
  - Hypothesis: N-WARM: bounded warm-up cache
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E14
  - Originally supported by: E12

## Blind spots and unknowns

### Coverage gaps

- python/codeql: missing (codeql is not on PATH or in the tools directory). Install hint: dla install codeql

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
- E15: limitation: points 1 to 9 come from S1; the extra points from S6
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E16: limitation: points 1 to 9 come from S1; the extra points from S8
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | — |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | — |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | — |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E3 | — |
| `intake.build` | Does the project build locally? | answered | E3 | — |
| `intake.workload` | What can exercise the suspected path? | answered | E3 | — |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E3 | — |

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
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (dcd59c5a64fa) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (a18ea512d46b) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (a18ea512d46b) | trend/T1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S3.json (511576645152) | alloc/A1.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S5.json (4d64c6f65d08) | alloc/A2.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S7.json (beecd08105eb) | alloc/A3.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S9.json (d0dc62f0b86d) | alloc/A4.json |
