# DLA report

Run: `20261107-195658-1df8`

Status: COMPLETE

Findings: 3 enumerated, 3 dispositioned; confirmed 0, strong 0, refuted 0, suspected 3, unresolved 0

Residual: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: suspected_static_only(tool_missing)
- Evidence classes: static
- Site: `reports/builder.py:14`
- Writers: `reports/builder.py:37`
- Removers: `reports/builder.py:26`
- Annotation: census missed remover: render() clears every column at start (line 26-27)
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: tool_missing
- Remedy: Installed dla lacks test/confirm/refute/harness/gate/report. Manual tracemalloc over 18000 requests (reports.load): traced heap +~3KB total, RSS plateau 17172KB, lru_cache currsize capped at 128. Not growing; treated as non-leak (bounded cache B019 style issue, columns cleared each render).
- Supporting evidence: E5

### F2: **UNCLASSIFIED**

- State: suspected_static_only(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `reports.builder.ReportBuilder._columns`
- Region: heap
- Site: `reports/builder.py:12`
- Writers: `reports/builder.py:37`, `reports/builder.py:38`, `reports/builder.py:39`, `reports/builder.py:43`, `reports/builder.py:44`, `reports/builder.py:47`
- Annotation: heuristic name match over the project's sources
- Annotation: lru_cache maxsize=128 bounds entries; key includes self+account+month
- Reason: tool_missing
- Remedy: Installed dla lacks test/confirm/refute/harness/gate/report. Manual tracemalloc over 18000 requests (reports.load): traced heap +~3KB total, RSS plateau 17172KB, lru_cache currsize capped at 128. Not growing; treated as non-leak (bounded cache B019 style issue, columns cleared each render).
- Supporting evidence: E8

### F3: **UNCLASSIFIED**

- State: suspected_static_only(tool_missing)
- Evidence classes: static
- Holder: static/global
- Owner: `reports.builder.ReportBuilder.category_totals`
- Region: heap
- Site: `reports/builder.py:14`
- Reason: tool_missing
- Remedy: Installed dla lacks test/confirm/refute/harness/gate/report. Manual tracemalloc over 18000 requests (reports.load): traced heap +~3KB total, RSS plateau 17172KB, lru_cache currsize capped at 128. Not growing; treated as non-leak (bounded cache B019 style issue, columns cleared each render).
- Supporting evidence: E8

## Ruled out

None.

## Blind spots and unknowns

### Coverage gaps

- jvm/codeql: missing (install failed: /tmp/dla-bench/m2-gates/gates/archetypes-on/python-report-memo-bounded/3/config/plugins/data/dla/tools/codeql: Read-only file system (os error 30)). Install hint: dla install codeql
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
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | report service python package reports/ (ReportService) |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | whole repo, python only |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | RSS climbs over thousands of requests; daily restart |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | python |
| `intake.build` | Does the project build locally? | answered | E1 | pure python, no build |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | python3 -m reports.load --iterations N |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 raw |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 3600s |

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
