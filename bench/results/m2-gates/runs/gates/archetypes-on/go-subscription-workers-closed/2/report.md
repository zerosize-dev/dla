# DLA report

Run: `20261107-194903-7d59`

Status: COMPLETE

Findings: 3 enumerated, 3 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 3

Residual: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `internal.notify.render.Renderer.layouts`
- Region: heap
- Site: `internal/notify/render.go:10`
- Writers: `internal/notify/render.go:19`
- Annotation: layouts map copied once in NewRenderer, never written afterwards; bounded by 4 topics
- Reason: no_harness
- Remedy: installed dla lacks harness/sample/trend/gate; ad hoc scratch-copy measurement showed no growth. Retest in the real service: the replay harness leaks nothing, so look at production-only callers that Open without Close
- Supporting evidence: E8

### F2: **UNCLASSIFIED**

- State: unresolved(no_harness)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `internal.notify.subscription.newSubscription -> s.run()`
- Region: non-heap
- Site: `internal/notify/subscription.go:33`
- Writers: `internal/notify/subscription.go:33`
- Removers: `internal/notify/subscription.go:89`
- Annotation: worker goroutine exits on done close; Handler.Serve defers sub.Close at handler.go:21 on every path. Measured: goroutines flat at 1 over 30x5000 requests
- Reason: no_harness
- Remedy: installed dla lacks harness/sample/trend/gate; ad hoc scratch-copy measurement showed no growth. Retest in the real service: the replay harness leaks nothing, so look at production-only callers that Open without Close
- Supporting evidence: E8

### F3: **UNCLASSIFIED**

- State: unresolved(no_harness)
- Evidence classes: static
- Holder: static/global
- Owner: `internal.workload.workload.topics`
- Region: heap
- Site: `internal/workload/workload.go:19`
- Annotation: read-only package slice of 4 topics, never appended
- Reason: no_harness
- Remedy: installed dla lacks harness/sample/trend/gate; ad hoc scratch-copy measurement showed no growth. Retest in the real service: the replay harness leaks nothing, so look at production-only callers that Open without Close
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
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | notifyd Go service |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | whole repo: cmd/notifyd, internal/notify, internal/workload |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | memory and thread count rise after each replay batch; daily OOM kill (as reported) |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E3 | go only |
| `intake.build` | Does the project build locally? | answered | E3 | go test passes; go run of scratch copy builds |
| `intake.workload` | What can exercise the suspected path? | answered | E9 | cmd/notifyd replay and internal/notify tests |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4, raw values |
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
