# DLA report

Run: `20261107-194936-a0ef`

Status: COMPLETE

Findings: 4 enumerated, 4 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 4

Residual: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Site: `reports/builder.py:14`
- Writers: `reports/builder.py:14`
- Annotation: lru_cache(maxsize=None) on method; key includes self, a new ReportBuilder per request in service.py:14, so 0 hits and every call adds an entry pinning builder, store ref, column buffers. No remover (no cache_clear).
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: tool_missing
- Remedy: the installed dla release has no test/confirm/gate/harness/sample commands; manual tracemalloc run (outside the ledger) showed linear growth, cache hits=0. Rerun with a full dla release: ledger test, add N-WARM sibling, confirm, then fix by not using lru_cache on a method (cache at module/service level keyed on (account,month) with a bound, or drop it)
- Supporting evidence: E5

### F2: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Site: `reports/builder.py:14`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: tool_missing
- Remedy: the installed dla release has no test/confirm/gate/harness/sample commands; manual tracemalloc run (outside the ledger) showed linear growth, cache hits=0. Rerun with a full dla release: ledger test, add N-WARM sibling, confirm, then fix by not using lru_cache on a method (cache at module/service level keyed on (account,month) with a bound, or drop it)
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Site: `reports/builder.py:14`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: tool_missing
- Remedy: the installed dla release has no test/confirm/gate/harness/sample commands; manual tracemalloc run (outside the ledger) showed linear growth, cache hits=0. Rerun with a full dla release: ledger test, add N-WARM sibling, confirm, then fix by not using lru_cache on a method (cache at module/service level keyed on (account,month) with a bound, or drop it)
- Supporting evidence: E7

### F4: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: static/global
- Owner: `reports.builder.ReportBuilder.category_totals`
- Region: heap
- Site: `reports/builder.py:14`
- Reason: tool_missing
- Remedy: the installed dla release has no test/confirm/gate/harness/sample commands; manual tracemalloc run (outside the ledger) showed linear growth, cache hits=0. Rerun with a full dla release: ledger test, add N-WARM sibling, confirm, then fix by not using lru_cache on a method (cache at module/service level keyed on (account,month) with a bound, or drop it)
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
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | reports package, ReportService |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | from task statement/repo: python, RSS growth in report service, load via reports.load |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | from task statement/repo: python, RSS growth in report service, load via reports.load |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | from task statement/repo: python, RSS growth in report service, load via reports.load |
| `intake.build` | Does the project build locally? | answered | E1 | from task statement/repo: python, RSS growth in report service, load via reports.load |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | from task statement/repo: python, RSS growth in report service, load via reports.load |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 raw values |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | from task statement/repo: python, RSS growth in report service, load via reports.load |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | — |
| E2 | run-config | static | dla 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | probe/probe.json |
| E4 | probe | static | dla-probe 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | probe/probe.json |
| E5 | static-hit | static | ruff 0.16.9 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | sweep/ruff.json |
| E6 | static-hit | static | pylint 4.1.3 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | sweep/pylint.json |
| E7 | static-hit | static | dla 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | sweep/dla.json |
| E8 | census-holder | static | dla-census 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | census/python.json |
| E9 | workload | static | dla 0.1.0 | repo 23d503ee31146c8214ce799b28203f8f50074bf4 | workload/workloads.json |
