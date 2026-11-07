# DLA report

Run: `20261107-195319-a701`

Status: COMPLETE

Findings: 6 enumerated, 6 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 6

Residual: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Site: `ingest/parser.py:29`
- Writers: `ingest/parser.py:29`
- Annotation: record.previous = _newest chains every Record to its predecessor; _newest global never cleared, so the whole history is reachable
- Reason: tool_missing
- Remedy: Chain via Record.previous from parser._newest (parser.py:29). Needs dla harness/sample/trend/ledger confirm; installed dla lacks them. Manual tracemalloc: clearing _newest drops retained memory.
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Site: `ingest/registry.py:3`
- Writers: `ingest/registry.py:7`, `ingest/pipeline.py:28`
- Removers: `ingest/registry.py:24`
- Annotation: subscribe per batch in process_batch; only remover is reset() (tests only); fire never unsubscribes
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: tool_missing
- Remedy: CheckpointWatcher subscribed per batch (pipeline.py:28) never unsubscribed; registry._LISTENERS grows (2000 listeners per 2000 batches, watchers.py:8 12MiB/8000 batches). Needs dla confirm tooling.
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: static/global
- Owner: `ingest.parser._newest`
- Region: heap
- Site: `ingest/parser.py:3`
- Writers: `ingest/parser.py:29`
- Annotation: heuristic name match over the project's sources
- Annotation: same holder as F1: _newest pointer; no remover; previous links make chain unbounded
- Reason: tool_missing
- Remedy: Chain via Record.previous from parser._newest (parser.py:29). Needs dla harness/sample/trend/ledger confirm; installed dla lacks them. Manual tracemalloc: clearing _newest drops retained memory.
- Supporting evidence: E7

### F4: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `ingest.pipeline.Pipeline.per_sensor`
- Region: heap
- Site: `ingest/pipeline.py:15`
- Writers: `ingest/pipeline.py:25`
- Annotation: heuristic name match over the project's sources
- Annotation: keyed by sensor, 64 sensors max: bounded, expected N-INTENT/bounded
- Reason: tool_missing
- Remedy: per_sensor bounded by 64 sensors; likely N-INTENT, not tested
- Supporting evidence: E7

### F5: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: registration-table
- Owner: `ingest.pipeline.Pipeline.process_batch`
- Region: heap
- Site: `ingest/pipeline.py:28`
- Writers: `ingest/pipeline.py:28`
- Annotation: one CheckpointWatcher subscribed per batch, never unsubscribed
- Reason: tool_missing
- Remedy: CheckpointWatcher subscribed per batch (pipeline.py:28) never unsubscribed; registry._LISTENERS grows (2000 listeners per 2000 batches, watchers.py:8 12MiB/8000 batches). Needs dla confirm tooling.
- Supporting evidence: E7

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
- Annotation: reset() is only called by tests; watcher holds marks dict sized to all known sensors plus sensors tuple
- Reason: tool_missing
- Remedy: CheckpointWatcher subscribed per batch (pipeline.py:28) never unsubscribed; registry._LISTENERS grows (2000 listeners per 2000 batches, watchers.py:8 12MiB/8000 batches). Needs dla confirm tooling.
- Supporting evidence: E7

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

- E7: partial coverage
- E7: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E7: limitation: writers and removers are textual name matches
- E8: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E8: limitation: workloads were discovered by reading files; none was run

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | sensor ingest worker, python package ingest/ |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | whole repo |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | RSS grows with batches processed; nightly restart |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E3 | python only |
| `intake.build` | Does the project build locally? | answered | E3 | pure python, no build |
| `intake.workload` | What can exercise the suspected path? | answered | E8 | python -m ingest.load --iterations N |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4, raw values |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 3600s |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | — |
| E2 | run-config | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | probe/probe.json |
| E4 | static-hit | static | ruff 0.16.9 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | sweep/ruff.json |
| E5 | static-hit | static | pylint 4.1.3 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | sweep/pylint.json |
| E6 | static-hit | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | sweep/dla.json |
| E7 | census-holder | static | dla-census 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | census/python.json |
| E8 | workload | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | workload/workloads.json |
