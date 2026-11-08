# DLA report

Run: `20261108-131742-f6e0`

Status: COMPLETE

Findings: 4 enumerated, 4 dispositioned; confirmed 1, strong 0, refuted 1, suspected 2, unresolved 0

Residual: not measured

Coverage: go 3520 iterations, 15 of 19 entry points exercised

## Findings

### F2: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `go.goroutines@internal/notify/subscription.go:33` 1 count/n (steady, gate pass, E14)
- Evidence classes: context, growth, static
- Holder: blocked-stack
- Owner: `internal.notify.subscription.newSubscription -> s.run()`
- Region: non-heap
- Shape: steady
- Site: `internal/notify/subscription.go:33`
- Writers: `internal/notify/handler.go:17`
- Removers: `internal/notify/subscription.go:93`
- Annotation: Handler.Serve opens a Subscription per request (spawns goroutine, 4096B scratch) and never calls Close on any path
- Sibling hypotheses: F2.n1
- Supporting evidence: E9, E12, E14

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `internal.notify.render.Renderer.layouts`
- Region: heap
- Site: `internal/notify/render.go:10`
- Annotation: layouts map is fixed at construction; only read
- Reason: not_exercised
- Remedy: read-only map; no growth observed
- Supporting evidence: E9

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `internal.workload.workload.topics`
- Region: heap
- Site: `internal/workload/workload.go:19`
- Annotation: topics slice is read-only
- Reason: not_exercised
- Remedy: read-only slice; no growth observed
- Supporting evidence: E9

## Ruled out

- **F2.n1** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (leak) E15
  - Hypothesis: N-WARM warm-up cache/pool plateau
  - Refuted if: goroutines keep growing over extra cycles after purge
  - Contradicting evidence: E15
  - Originally supported by: E14

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
- E11: limitation: harness written; nothing was run (dla harness run measures it)
- E11: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E11: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E12: partial coverage (9 of 9 examined)
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: points 1 to 9 come from S1; the extra points from S2
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | notifyd Go service, cmd/notifyd, internal/notify |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | whole repo |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | memory and thread count grow after each batch; OOMKilled daily |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E3 | go only |
| `intake.build` | Does the project build locally? | answered | E4 | go build ok |
| `intake.workload` | What can exercise the suspected path? | answered | E3 | go test / replay workload; see discover |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 raw values |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 60m |

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
| E11 | workload | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | harness/H1/harness.json |
| E12 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (2c8026360ddd) | harness/H1/run-S1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S1.json (5ecf8e054be2) | trend/T1.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (5ecf8e054be2) | trend/T1.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S3.json (d6c6f9057c8c) | alloc/A1.json |
