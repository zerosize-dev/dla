# DLA report

Run: `20261108-132104-5cb4`

Status: COMPLETE

Findings: 8 enumerated, 8 dispositioned; confirmed 2, strong 0, refuted 2, suspected 0, unresolved 4

Residual: heap 2678.6479437229436 bytes/n unattributed of 2678.6479437229436 total (threshold 669.6619859307359) ABOVE THRESHOLD, see O1

Coverage: python 3520 iterations, 8 of 11 entry points exercised

## Findings

### F3: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:ingest.parser._newest` 15.97 count/n (steady, gate pass, E13)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `ingest.parser._newest`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:3`
- Writers: `ingest/parser.py:29`
- Annotation: heuristic name match over the project's sources
- Annotation: _newest rebinds to each new Record; record.previous = _newest (parser.py:27) chains every record ever parsed; no remover/cut of previous
- Sibling hypotheses: F3.n1
- Supporting evidence: E8, E11, E13

### F6: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:ingest.registry._LISTENERS` 7.974 count/n (steady, gate pass, E13)
- Evidence classes: context, growth, static
- Holder: registration-table
- Owner: `ingest.registry._LISTENERS`
- Region: heap
- Shape: steady
- Site: `ingest/registry.py:3`
- Writers: `ingest/registry.py:7`, `ingest/pipeline.py:28`
- Removers: `ingest/registry.py:24`
- Annotation: heuristic name match over the project's sources
- Annotation: pipeline subscribes one CheckpointWatcher per batch; no unsubscribe; only reset() (tests) clears
- Sibling hypotheses: F6.n1
- Supporting evidence: E8, E11, E13

### F1: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Site: `ingest/parser.py:29`
- Reason: tool_missing
- Remedy: static detector hit duplicates confirmed F3 (same parser.py chain via Record.previous); no separate ledger merge command to fold it
- Supporting evidence: E7

### F2: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Site: `ingest/registry.py:3`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: tool_missing
- Remedy: static detector hit duplicates confirmed F6 (registry._LISTENERS)
- Supporting evidence: E7

### F4: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: discriminator, static
- Holder: container-field
- Owner: `ingest.pipeline.Pipeline.per_sensor`
- Region: heap
- Site: `ingest/pipeline.py:15`
- Writers: `ingest/pipeline.py:25`
- Annotation: heuristic name match over the project's sources
- Annotation: keyed by sensor, 64 sensors max; bounded
- Reason: time_budget
- Remedy: per_sensor is bounded by 64 sensor keys by source reading; no per-holder probe column grew; whole-process discriminator is confounded by F3/F6. Re-test after fixing F3/F6
- Supporting evidence: E8, E16

### F5: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: registration-table
- Owner: `ingest.pipeline.Pipeline.process_batch`
- Region: heap
- Site: `ingest/pipeline.py:28`
- Reason: tool_missing
- Remedy: subscribe call site pipeline.py:28 is the writer of confirmed F6
- Supporting evidence: E8

## Ruled out

- **F3.n1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (leak) E14
  - Hypothesis: N-WARM
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E14
  - Originally supported by: E13
- **F6.n1** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (leak) E15
  - Hypothesis: N-WARM
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
- E17: partial coverage (1 of 3 examined)
- E17: limitation: F3: growth measured as a count; bytes not attributed
- E17: limitation: F6: growth measured as a count; bytes not attributed
- E17: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E17: limitation: threshold is provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | sensor ingest worker, python package ingest/ |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | whole ingest package |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | RSS grows with batches processed; nightly restart |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | python |
| `intake.build` | Does the project build locally? | answered | E1 | pure python, no build |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | python3 -m ingest.load --iterations N |
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
| E11 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (502ab6be92cc) | harness/H1/run-S1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S1.json (94ca0198aea5) | trend/T1.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (94ca0198aea5) | trend/T1.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S3.json (46fb13161fcd) | alloc/A1.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S5.json (e3cbecc40683) | alloc/A2.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S7.json (976e7f115487) | alloc/A3.json |
| E17 | residual | test | dla 0.1.0 | artifact series/S1.json (94ca0198aea5) | gap/residual-S1-E17.json |
