# DLA report

Run: `20261107-194657-6dc6`

Status: COMPLETE

Findings: 5 enumerated, 5 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 5

Residual: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `internal.notify.render.Renderer.layouts`
- Region: heap
- Site: `internal/notify/render.go:10`
- Annotation: Renderer.layouts copied once at construction, read-only afterwards; no writer after init. Not growing.
- Reason: tool_missing
- Remedy: Read-only after construction; no growth observed. Needs trend to formally refute.
- Supporting evidence: E8

### F2: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `internal.notify.subscription.newSubscription -> s.run()`
- Region: non-heap
- Site: `internal/notify/subscription.go:33`
- Writers: `internal/notify/handler.go:20`, `internal/notify/hub.go:28`
- Removers: `internal/notify/subscription.go:88`
- Annotation: Handler.Serve opens a Subscription per request (go s.run() at subscription.go:33) and never calls Close (only remover, subscription.go:88, has no caller in Serve). Goroutine blocked in select forever; also pins 4KB scratch + channels + hub ref. Measured in scratch copy: +2000 goroutines and ~10.9MB heap per 2000 requests, linear; closing the sub gives 0 growth.
- Sibling hypotheses: F2.n1, F2.n2
- Reason: tool_missing
- Remedy: Installed dla lacks ledger test/confirm, harness, sample, trend, gap, gate. Manual measurement (scratch copy, forced GC, 5x2000 requests): goroutines 2002->10002 and heap 11.0MB->54.5MB linear; with Close() added: zero growth. Re-run with a full dla release to confirm F2.
- Supporting evidence: E8

### F2.n1: Hypothesis N-WARM (UNCLASSIFIED)

- State: unresolved(tool_missing)
- Evidence classes: context
- Sibling hypothesis of: F2
- Hypothesis: N-WARM
- Refuted if: goroutine count and heap plateau across repeated equal batches
- Reason: tool_missing
- Remedy: Manual test: growth is linear across 5 equal batches with forced GC (no plateau, so not warm-up or deferred collection); formal refute needs ledger refute, absent in this release.
- Supporting evidence: E9

### F2.n2: Hypothesis N-HEADROOM (UNCLASSIFIED)

- State: unresolved(tool_missing)
- Evidence classes: context
- Sibling hypothesis of: F2
- Hypothesis: N-HEADROOM
- Refuted if: growth persists after forced runtime.GC()
- Reason: tool_missing
- Remedy: Manual test: growth is linear across 5 equal batches with forced GC (no plateau, so not warm-up or deferred collection); formal refute needs ledger refute, absent in this release.
- Supporting evidence: E9

### F3: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: static/global
- Owner: `internal.workload.workload.topics`
- Region: heap
- Site: `internal/workload/workload.go:19`
- Annotation: workload.topics is a fixed 4-element read-only slice. Not growing.
- Reason: tool_missing
- Remedy: Read-only fixed slice; no growth. Needs trend to formally refute.
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
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | from task statement and repo: Go notifyd replay, goroutine/memory growth, budget 60m |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | from task statement and repo: Go notifyd replay, goroutine/memory growth, budget 60m |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | from task statement and repo: Go notifyd replay, goroutine/memory growth, budget 60m |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | from task statement and repo: Go notifyd replay, goroutine/memory growth, budget 60m |
| `intake.build` | Does the project build locally? | answered | E1 | from task statement and repo: Go notifyd replay, goroutine/memory growth, budget 60m |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | from task statement and repo: Go notifyd replay, goroutine/memory growth, budget 60m |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 raw values |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | from task statement and repo: Go notifyd replay, goroutine/memory growth, budget 60m |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | — |
| E2 | run-config | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | probe/probe.json |
| E4 | static-hit | static | staticcheck 2025.1.1 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | sweep/staticcheck.json |
| E5 | static-hit | static | golangci-lint 2.5.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | sweep/golangci-lint.json |
| E6 | static-hit | static | govet 1.24.7 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | sweep/govet.json |
| E7 | static-hit | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | sweep/dla.json |
| E8 | census-holder | static | dla-census 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | census/go.json |
| E9 | workload | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | workload/workloads.json |
