# DLA report

Run: `20261107-195028-ff95`

Status: COMPLETE

Findings: 5 enumerated, 5 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 5

Residual: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `internal.render.batch.Renderer.pool`
- Region: heap
- Site: `internal/render/batch.go:23`
- Writers: `internal/render/batch.go:37`, `internal/render/batch.go:60`
- Annotation: heuristic name match over the project's sources
- Annotation: Renderer.pool: reference only; not a collection
- Reason: tool_missing
- Remedy: dla release lacks harness/sample/trend/gate/confirm; manual scratch measurement shows plateau for pool, no growth
- Supporting evidence: E8

### F2: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `internal.render.batch.Renderer.index`
- Region: heap
- Site: `internal/render/batch.go:24`
- Writers: `internal/render/batch.go:40`
- Removers: `internal/render/batch.go:61`
- Annotation: heuristic name match over the project's sources
- Annotation: index emptied every batch
- Reason: tool_missing
- Remedy: dla release lacks harness/sample/trend/gate/confirm; manual scratch measurement shows plateau for pool, no growth
- Supporting evidence: E8

### F3: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `internal.render.pool.BufferPool.free`
- Region: heap
- Site: `internal/render/pool.go:11`
- Writers: `internal/render/pool.go:27`, `internal/render/pool.go:37`
- Removers: `internal/render/pool.go:26`
- Annotation: heuristic name match over the project's sources
- Annotation: BufferPool.free: Put appends all buffers, no cap or shrink; bounded by max batch 400. Measured plateau heapinuse ~1.9MB from batch 250 to 2000, created=400 idle=400 (scratch test outside source).
- Reason: tool_missing
- Remedy: dla release lacks harness/sample/trend/gate/confirm; manual scratch measurement shows plateau for pool, no growth
- Supporting evidence: E8

### F4: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: static/global
- Owner: `internal.workload.workload.skus`
- Region: heap
- Site: `internal/workload/workload.go:27`
- Reason: tool_missing
- Remedy: dla release lacks harness/sample/trend/gate/confirm; manual scratch measurement shows plateau for pool, no growth
- Supporting evidence: E8

### F5: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: static/global
- Owner: `internal.workload.workload.descs`
- Region: heap
- Site: `internal/workload/workload.go:28`
- Reason: tool_missing
- Remedy: dla release lacks harness/sample/trend/gate/confirm; manual scratch measurement shows plateau for pool, no growth
- Supporting evidence: E8

## Ruled out

None.

## Blind spots and unknowns

### Coverage gaps

None recorded.

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
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | Go invoiced renderer; symptom: RSS rise after start; go builds; consent from runner |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | Go invoiced renderer; symptom: RSS rise after start; go builds; consent from runner |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | Go invoiced renderer; symptom: RSS rise after start; go builds; consent from runner |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | Go invoiced renderer; symptom: RSS rise after start; go builds; consent from runner |
| `intake.build` | Does the project build locally? | answered | E1 | Go invoiced renderer; symptom: RSS rise after start; go builds; consent from runner |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | Go invoiced renderer; symptom: RSS rise after start; go builds; consent from runner |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E1 | Go invoiced renderer; symptom: RSS rise after start; go builds; consent from runner |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | Go invoiced renderer; symptom: RSS rise after start; go builds; consent from runner |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo dd879aa91b6b7202ad9d226ebe261d07e9c68353 | — |
| E2 | run-config | static | dla 0.1.0 | repo dd879aa91b6b7202ad9d226ebe261d07e9c68353 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo dd879aa91b6b7202ad9d226ebe261d07e9c68353 | probe/probe.json |
| E4 | static-hit | static | staticcheck 2025.1.1 | repo dd879aa91b6b7202ad9d226ebe261d07e9c68353 | sweep/staticcheck.json |
| E5 | static-hit | static | golangci-lint 2.5.0 | repo dd879aa91b6b7202ad9d226ebe261d07e9c68353 | sweep/golangci-lint.json |
| E6 | static-hit | static | govet 1.24.7 | repo dd879aa91b6b7202ad9d226ebe261d07e9c68353 | sweep/govet.json |
| E7 | static-hit | static | dla 0.1.0 | repo dd879aa91b6b7202ad9d226ebe261d07e9c68353 | sweep/dla.json |
| E8 | census-holder | static | dla-census 0.1.0 | repo dd879aa91b6b7202ad9d226ebe261d07e9c68353 | census/go.json |
| E9 | workload | static | dla 0.1.0 | repo dd879aa91b6b7202ad9d226ebe261d07e9c68353 | workload/workloads.json |
