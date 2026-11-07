# DLA report

Run: `20261107-195145-21b6`

Status: COMPLETE

Findings: 2 enumerated, 2 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 2

Residual: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: registration-table
- Owner: `outlines.parser.parse`
- Region: heap
- Site: `outlines/parser.py:25`
- Writers: `outlines/models.py:10`, `outlines/parser.py:11`, `outlines/service.py:15`
- Annotation: heuristic name match over the project's sources
- Annotation: tree.by_path is per-Tree, dies with tree; census 'no remover' is a name-match artifact. Real holder: Node<->parent/children/tree reference cycles, freed only by cyclic GC
- Sibling hypotheses: F1.n1
- Reason: tool_missing
- Remedy: Installed dla lacks gate/harness/sample/trend/confirm; manual measurement shows cycle garbage deferred by gc.set_threshold(40000,20,20) in outlines/tuning.py:11; gc.collect() returns objects to baseline (34950->11762), default thresholds stay flat (~14k objs)
- Supporting evidence: E8

### F1.n1: Hypothesis N-HEADROOM (UNCLASSIFIED)

- State: unresolved(tool_missing)
- Evidence classes: static
- Sibling hypothesis of: F1
- Hypothesis: N-HEADROOM
- Refuted if: objects stay after gc.collect()
- Reason: tool_missing
- Remedy: Manual run: gc.collect() returned objects to baseline (N-HEADROOM supported), but ledger refute/confirm absent in this release
- Supporting evidence: E8

## Ruled out

None.

## Blind spots and unknowns

### Coverage gaps

- python/codeql: missing (codeql is not on PATH or in the tools directory). Install hint: dla install codeql

### Runtime blind spots

None recorded.

### Entry points not exercisable

- python: `outlines/tuning.py::install`: no_harness

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
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | outlines package (python), load via python3 -m outlines.load |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | from task text/repo: python-only, tiny package, memory graph grows over document queue, load.py workload, 60m budget |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | from task text/repo: python-only, tiny package, memory graph grows over document queue, load.py workload, 60m budget |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | from task text/repo: python-only, tiny package, memory graph grows over document queue, load.py workload, 60m budget |
| `intake.build` | Does the project build locally? | answered | E1 | from task text/repo: python-only, tiny package, memory graph grows over document queue, load.py workload, 60m budget |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | from task text/repo: python-only, tiny package, memory graph grows over document queue, load.py workload, 60m budget |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 raw values |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | from task text/repo: python-only, tiny package, memory graph grows over document queue, load.py workload, 60m budget |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | — |
| E2 | run-config | static | dla 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | — |
| E3 | probe | static | dla-probe 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | probe/probe.json |
| E4 | probe | static | dla-probe 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | probe/probe.json |
| E5 | static-hit | static | ruff 0.16.9 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | sweep/ruff.json |
| E6 | static-hit | static | pylint 4.1.3 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | sweep/pylint.json |
| E7 | static-hit | static | dla 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | sweep/dla.json |
| E8 | census-holder | static | dla-census 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | census/python.json |
| E9 | workload | static | dla 0.1.0 | repo f1f66527497444539ea38639bf2b1d0de1ce9a9a | workload/workloads.json |
