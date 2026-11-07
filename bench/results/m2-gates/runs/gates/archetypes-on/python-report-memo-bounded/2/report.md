# DLA report

Run: `20261107-195021-1cce`

Status: COMPLETE

Findings: 3 enumerated, 3 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 3

Residual: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(no_harness)
- Evidence classes: static
- Site: `reports/builder.py:14`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: no_harness
- Remedy: dla release lacks harness/sample/trend/confirm; manual tracemalloc+RSS (24k and 100k reqs) showed no growth: heap ~150KB flat, RSS 15400kB flat. Likely not the leak; look outside this code (real store, server layer, deployment).
- Supporting evidence: E5

### F2: **UNCLASSIFIED**

- State: unresolved(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `reports.builder.ReportBuilder._columns`
- Region: heap
- Site: `reports/builder.py:12`
- Writers: `reports/builder.py:37`, `reports/builder.py:38`, `reports/builder.py:39`, `reports/builder.py:43`, `reports/builder.py:44`, `reports/builder.py:47`
- Removers: `reports/builder.py:21`
- Annotation: heuristic name match over the project's sources
- Annotation: census missed remover: column.clear() at start of every render; lengths reset per request
- Reason: no_harness
- Remedy: dla release lacks harness/sample/trend/confirm; manual tracemalloc+RSS (24k and 100k reqs) showed no growth: heap ~150KB flat, RSS 15400kB flat. Likely not the leak; look outside this code (real store, server layer, deployment).
- Supporting evidence: E8

### F3: **UNCLASSIFIED**

- State: unresolved(no_harness)
- Evidence classes: static
- Holder: static/global
- Owner: `reports.builder.ReportBuilder.category_totals`
- Region: heap
- Site: `reports/builder.py:14`
- Writers: `reports/builder.py:14`
- Annotation: lru_cache maxsize=128 on method: bounded (currsize pinned at 128 over 24000 reqs; keys pin self, but one builder per service). Eviction is the remover.
- Reason: no_harness
- Remedy: dla release lacks harness/sample/trend/confirm; manual tracemalloc+RSS (24k and 100k reqs) showed no growth: heap ~150KB flat, RSS 15400kB flat. Likely not the leak; look outside this code (real store, server layer, deployment).
- Supporting evidence: E8

## Ruled out

None.

## Blind spots and unknowns

### Coverage gaps

- python/codeql: missing (codeql is not on PATH or in the tools directory). Install hint: dla install codeql

### Runtime blind spots

None recorded.

### Entry points not exercisable

- python: `reports/builder.py::ReportBuilder.category_totals`: no_harness

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
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | reports package (python report service) |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | from task text/repo: python, reports.load harness, 60m budget |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | from task text/repo: python, reports.load harness, 60m budget |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | from task text/repo: python, reports.load harness, 60m budget |
| `intake.build` | Does the project build locally? | answered | E1 | from task text/repo: python, reports.load harness, 60m budget |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | from task text/repo: python, reports.load harness, 60m budget |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 raw values |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | from task text/repo: python, reports.load harness, 60m budget |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 193963928cbca22594b461b4b8d3fe954c299cf0 | — |
| E2 | run-config | static | dla 0.1.0 | repo 193963928cbca22594b461b4b8d3fe954c299cf0 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 193963928cbca22594b461b4b8d3fe954c299cf0 | probe/probe.json |
| E4 | probe | static | dla-probe 0.1.0 | repo 193963928cbca22594b461b4b8d3fe954c299cf0 | probe/probe.json |
| E5 | static-hit | static | ruff 0.16.9 | repo 193963928cbca22594b461b4b8d3fe954c299cf0 | sweep/ruff.json |
| E6 | static-hit | static | pylint 4.1.3 | repo 193963928cbca22594b461b4b8d3fe954c299cf0 | sweep/pylint.json |
| E7 | static-hit | static | dla 0.1.0 | repo 193963928cbca22594b461b4b8d3fe954c299cf0 | sweep/dla.json |
| E8 | census-holder | static | dla-census 0.1.0 | repo 193963928cbca22594b461b4b8d3fe954c299cf0 | census/python.json |
| E9 | workload | static | dla 0.1.0 | repo 193963928cbca22594b461b4b8d3fe954c299cf0 | workload/workloads.json |
