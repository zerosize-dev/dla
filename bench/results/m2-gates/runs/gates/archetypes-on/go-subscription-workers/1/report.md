# DLA report

Run: `20261107-194042-de9c`

Status: COMPLETE

Findings: 3 enumerated, 3 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 3

Residual: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `internal.notify.render.Renderer.layouts`
- Region: heap
- Site: `internal/notify/render.go:10`
- Annotation: layouts map filled once in NewRenderer from fixed 4-entry Layouts(); no per-request writer; bounded
- Reason: tool_missing
- Remedy: installed dla lacks gate/harness/sample/trend/ledger test|confirm; run those on F2 (Subscription goroutine never closed in Handler.Serve) when available
- Supporting evidence: E9

### F2: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `internal.notify.subscription.newSubscription -> s.run()`
- Region: non-heap
- Site: `internal/notify/subscription.go:33`
- Writers: `internal/notify/subscription.go:33`
- Annotation: go s.run() started per Hub.Open (called from handler.go:18 Serve per request); no Close() call in Serve, so worker blocks in select forever, holding its channels, 4KB scratch and stack
- Reason: tool_missing
- Remedy: installed dla lacks gate/harness/sample/trend/ledger test|confirm; run those on F2 (Subscription goroutine never closed in Handler.Serve) when available
- Supporting evidence: E9

### F3: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: static/global
- Owner: `internal.workload.workload.topics`
- Region: heap
- Site: `internal/workload/workload.go:19`
- Annotation: read-only fixed 4-element slice; bounded
- Reason: tool_missing
- Remedy: installed dla lacks gate/harness/sample/trend/ledger test|confirm; run those on F2 (Subscription goroutine never closed in Handler.Serve) when available
- Supporting evidence: E9

## Ruled out

None.

## Blind spots and unknowns

### Coverage gaps

None recorded.

### Runtime blind spots

None recorded.

### Entry points not exercisable

- go: `internal/notify/hub.go::Hub.Opened`: no_harness
- go: `internal/workload/workload.go::NewRNG`: no_harness

### Evidence limitations and unknowns

- E9: partial coverage
- E9: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E9: limitation: writers and removers are textual name matches
- E10: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E10: limitation: workloads were discovered by reading files; none was run

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | notifyd Go service, repo root |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | whole repo, go module example.com/notifyd |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | memory and thread count rise after each replay batch; daily OOMKill |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E3 | go only |
| `intake.build` | Does the project build locally? | answered | E4 | go build ok |
| `intake.workload` | What can exercise the suspected path? | answered | E3 | go test; internal/workload |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 raw values |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 3600s |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | — |
| E2 | run-config | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | probe/probe.json |
| E4 | probe | static | dla-probe 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | probe/probe.json |
| E5 | static-hit | static | staticcheck 2025.1.1 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | sweep/staticcheck.json |
| E6 | static-hit | static | golangci-lint 2.5.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | sweep/golangci-lint.json |
| E7 | static-hit | static | govet 1.24.7 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | sweep/govet.json |
| E8 | static-hit | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | sweep/dla.json |
| E9 | census-holder | static | dla-census 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | census/go.json |
| E10 | workload | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | workload/workloads.json |
