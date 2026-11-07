# DLA report

Run: `20261107-194216-4023`

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
- Annotation: layouts map copied once in NewRenderer, never written afterwards; bounded
- Reason: tool_missing
- Remedy: this dla release lacks harness/sample/trend/gate/confirm/refute commands; manual scratch replay showed no growth in this tree, so the reported leak is not reproduced here. Re-run on the deployed build/config or with real pprof goroutine+heap profiles from the pod, and check callers other than Handler.Serve.
- Supporting evidence: E8

### F2: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `internal.notify.subscription.newSubscription -> s.run()`
- Region: non-heap
- Site: `internal/notify/subscription.go:33`
- Writers: `internal/notify/subscription.go:33`
- Removers: `internal/notify/subscription.go:90`
- Annotation: worker goroutine spawned per Open; Handler.Serve defers sub.Close() (handler.go:23) which closes done and waits for exit on all paths. Scratch replay of 4x2000 requests: goroutines=2 flat, heap ~129KB flat.
- Reason: tool_missing
- Remedy: this dla release lacks harness/sample/trend/gate/confirm/refute commands; manual scratch replay showed no growth in this tree, so the reported leak is not reproduced here. Re-run on the deployed build/config or with real pprof goroutine+heap profiles from the pod, and check callers other than Handler.Serve.
- Supporting evidence: E8

### F3: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: static/global
- Owner: `internal.workload.workload.topics`
- Region: heap
- Site: `internal/workload/workload.go:19`
- Annotation: topics is a fixed read-only slice; bounded
- Reason: tool_missing
- Remedy: this dla release lacks harness/sample/trend/gate/confirm/refute commands; manual scratch replay showed no growth in this tree, so the reported leak is not reproduced here. Re-run on the deployed build/config or with real pprof goroutine+heap profiles from the pod, and check callers other than Handler.Serve.
- Supporting evidence: E8

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

- E8: partial coverage
- E8: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E8: limitation: writers and removers are textual name matches
- E9: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E9: limitation: workloads were discovered by reading files; none was run

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | repository: Go notifyd service |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | whole repo, internal/notify + workload |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | memory and threads(goroutines) rise after each replay batch; daily OOM |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | go only |
| `intake.build` | Does the project build locally? | answered | E3 | go build ok (E3) |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | internal/workload replay driver |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 raw values |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 60m |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 23911e0074f1c23bd2fa28d182500e99d68a2f3b | — |
| E2 | run-config | static | dla 0.1.0 | repo 23911e0074f1c23bd2fa28d182500e99d68a2f3b | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 23911e0074f1c23bd2fa28d182500e99d68a2f3b | probe/probe.json |
| E4 | static-hit | static | staticcheck 2025.1.1 | repo 23911e0074f1c23bd2fa28d182500e99d68a2f3b | sweep/staticcheck.json |
| E5 | static-hit | static | golangci-lint 2.5.0 | repo 23911e0074f1c23bd2fa28d182500e99d68a2f3b | sweep/golangci-lint.json |
| E6 | static-hit | static | govet 1.24.7 | repo 23911e0074f1c23bd2fa28d182500e99d68a2f3b | sweep/govet.json |
| E7 | static-hit | static | dla 0.1.0 | repo 23911e0074f1c23bd2fa28d182500e99d68a2f3b | sweep/dla.json |
| E8 | census-holder | static | dla-census 0.1.0 | repo 23911e0074f1c23bd2fa28d182500e99d68a2f3b | census/go.json |
| E9 | workload | static | dla 0.1.0 | repo 23911e0074f1c23bd2fa28d182500e99d68a2f3b | workload/workloads.json |
