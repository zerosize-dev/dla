# DLA report

Run: `20261107-194138-dd92`

Status: COMPLETE

Findings: 6 enumerated, 6 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 6

Residual: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Site: `ingest/parser.py:29`
- Reason: tool_missing
- Remedy: Duplicate detector hit for F3 (parser.py:29)
- Supporting evidence: E7

### F2: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Site: `ingest/registry.py:3`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: tool_missing
- Remedy: Duplicate detector hit for F6 (registry.py:3)
- Supporting evidence: E7

### F3: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: static/global
- Owner: `ingest.parser._newest`
- Region: heap
- Site: `ingest/parser.py:3`
- Writers: `ingest/parser.py:29`
- Annotation: heuristic name match over the project's sources
- Annotation: _newest rebinds each parse; Record.previous links to prior record, so the chain is never cut. No remover anywhere (reset() in registry does not touch it).
- Reason: tool_missing
- Remedy: Confirm with dla harness+trend once available; ad-hoc: ~6.4 MiB retained at 8000 batches via Record.previous chain, freed to 11 KB by parser._newest=None
- Supporting evidence: E8

### F4: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `ingest.pipeline.Pipeline.per_sensor`
- Region: heap
- Site: `ingest/pipeline.py:15`
- Writers: `ingest/pipeline.py:25`
- Annotation: heuristic name match over the project's sources
- Annotation: Keyed by sensor (64 distinct), bounded; likely N-HWM/intended, not a leak.
- Reason: tool_missing
- Remedy: Bounded by 64 sensor keys; rule out as N-HWM once ledger refute exists
- Supporting evidence: E8

### F5: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: registration-table
- Owner: `ingest.pipeline.Pipeline.process_batch`
- Region: heap
- Site: `ingest/pipeline.py:28`
- Writers: `ingest/pipeline.py:28`
- Annotation: Per-batch subscribe site feeding F6; each watcher holds marks dict sized by per_sensor (up to 64 keys).
- Reason: tool_missing
- Remedy: Same as F6: subscribe site pipeline.py:28 is the writer; confirm via dla ledger confirm once available
- Supporting evidence: E8

### F6: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: registration-table
- Owner: `ingest.registry._LISTENERS`
- Region: heap
- Site: `ingest/registry.py:3`
- Writers: `ingest/registry.py:7`, `ingest/pipeline.py:28`
- Removers: `ingest/registry.py:24`
- Annotation: heuristic name match over the project's sources
- Annotation: Only remover is reset(), called by tests only; pipeline subscribes one CheckpointWatcher per batch and never unsubscribes. fire() does not drop listeners.
- Reason: tool_missing
- Remedy: Install a dla release with ledger test/confirm and harness/sample/trend; evidence: tracemalloc ad-hoc run showed 1 CheckpointWatcher per batch (8000 listeners at 8000 batches, ~15 MiB incl. marks dicts at watchers.py:8), freed by registry.reset()
- Supporting evidence: E8

## Ruled out

None.

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

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | see run init/probe; python ingest worker, RSS grows per batch, build ok, runner consent tiers 1-4 |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | see run init/probe; python ingest worker, RSS grows per batch, build ok, runner consent tiers 1-4 |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | see run init/probe; python ingest worker, RSS grows per batch, build ok, runner consent tiers 1-4 |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | see run init/probe; python ingest worker, RSS grows per batch, build ok, runner consent tiers 1-4 |
| `intake.build` | Does the project build locally? | answered | E1 | see run init/probe; python ingest worker, RSS grows per batch, build ok, runner consent tiers 1-4 |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | see run init/probe; python ingest worker, RSS grows per batch, build ok, runner consent tiers 1-4 |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E1 | see run init/probe; python ingest worker, RSS grows per batch, build ok, runner consent tiers 1-4 |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | see run init/probe; python ingest worker, RSS grows per batch, build ok, runner consent tiers 1-4 |

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
