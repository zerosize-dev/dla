# DLA report

Run: `20261108-123738-95bb`

Status: COMPLETE

Findings: 9 enumerated, 9 dispositioned; confirmed 2, strong 0, refuted 2, suspected 3, unresolved 2

Residual: not measured

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
- Annotation: _newest rebound per record; record.previous=_newest links every Record to its predecessor, no remover -> unbounded chain
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
- Writers: `ingest/registry.py:7`, `ingest/registry.py:6`
- Removers: `ingest/registry.py:24`
- Annotation: heuristic name match over the project's sources
- Annotation: subscribe per batch (pipeline.py:28); only remover is reset(), test-only
- Sibling hypotheses: F6.n1
- Supporting evidence: E8, E11, E12

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Site: `ingest/parser.py:29`
- Writers: `ingest/parser.py:28`
- Annotation: same chain as F3
- Reason: not_exercised
- Remedy: Same site/chain as confirmed F3; no separate column
- Supporting evidence: E7

### F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Site: `ingest/registry.py:3`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: Same holder as confirmed F6
- Supporting evidence: E7

### F5: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `ingest.pipeline.Pipeline.process_batch`
- Region: heap
- Site: `ingest/pipeline.py:28`
- Writers: `ingest/pipeline.py:28`
- Annotation: subscribes a new CheckpointWatcher every batch, never unsubscribed; each watcher holds marks dict of all known sensors
- Reason: not_exercised
- Remedy: Writer site of confirmed F6 (pipeline.py:28); fix there
- Supporting evidence: E8

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: context, static
- Holder: container-field
- Owner: `ingest.pipeline.Pipeline.per_sensor`
- Region: heap
- Site: `ingest/pipeline.py:15`
- Writers: `ingest/pipeline.py:25`, `ingest/pipeline.py:26`
- Annotation: heuristic name match over the project's sources
- Annotation: keyed by sensor; bounded by 64 sensors
- Sibling hypotheses: F4.n1
- Reason: not_exercised
- Remedy: per_sensor has no probe column; keyed by 64 sensors so bounded by code; alloc-test verdict reflected whole-process growth from F3/F6, not F4
- Supporting evidence: E8, E11

### F4.n1: Hypothesis N-SCALE: bounded by 64 sensor keys (UNCLASSIFIED)

- State: unresolved(not_exercised)
- Evidence classes: growth
- Sibling hypothesis of: F4
- Hypothesis: N-SCALE: bounded by 64 sensor keys
- Refuted if: per_sensor size keeps growing past 64 keys
- Reason: not_exercised
- Remedy: see F4
- Supporting evidence: E13

## Ruled out

- **F3.n1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (leak) E14
  - Hypothesis: N-WARM: bounded cache/warm-up plateau
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E14
  - Originally supported by: E13
- **F6.n1** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (leak) E15
  - Hypothesis: N-WARM: bounded cache/warm-up plateau
  - Refuted if: growth continues over the extra cycles
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

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | sensor ingest worker, python package ingest/ |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | whole repo, ingest/ package |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | RSS grows with batches processed; nightly restart |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | python |
| `intake.build` | Does the project build locally? | answered | E1 | pure python, no build; python3 -m ingest.load |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | ingest.load --iterations N |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4, raw values |
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
| E11 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (733614e50cd7) | harness/H1/run-S1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S1.json (f1d5087602b1) | trend/T1.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (f1d5087602b1) | trend/T1.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S3.json (b7f613b7d7f4) | alloc/A1.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S5.json (a958dd268bb9) | alloc/A2.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S7.json (9b59bb0e3dcf) | alloc/A3.json |
