# DLA report

Run: `20261108-193723-3154`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 4 enumerated, 4 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 2

Residual: heap 5438.552727272727 bytes/n unattributed of 5438.552727272727 total (threshold 1359.6381818181817) ABOVE THRESHOLD, see O1; non-heap 2132.4024242424243 bytes/n unattributed of 2132.4024242424243 total (threshold 533.1006060606061) ABOVE THRESHOLD, see O2

Coverage: go 3520 iterations, 15 of 19 entry points exercised

## Findings

### F2: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `go.goroutines@internal/notify/subscription.go:33` 1 count/n (steady, gate pass, E13)
- Evidence classes: context, growth, static
- Holder: blocked-stack
- Owner: `internal.notify.subscription.newSubscription -> s.run()`
- Region: non-heap
- Shape: steady
- Site: `internal/notify/subscription.go:33`
- Writers: `internal/notify/subscription.go:33`
- Annotation: Handler.Serve (handler.go:20) calls Hub.Open, which starts one Subscription worker goroutine (go s.run(), subscription.go:33) with a 4KB scratch buffer and 3 channels. Serve never calls sub.Close() (neither on success nor on error paths), so each request leaves a goroutine blocked in select on events/flushes/done forever; the goroutine pins the Subscription, scratch buffer, and hub. Missing pairing: Open without Close (class of repair: close the subscription when Serve returns, e.g. defer). Goroutines and memory grow ~1 per request.
- Sibling hypotheses: F2.n1
- Supporting evidence: E8, E11, E13

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `internal.notify.render.Renderer.layouts`
- Region: heap
- Site: `internal/notify/render.go:10`
- Reason: not_exercised
- Remedy: Static read-only map, no writer after construction; the heap growth in the harness is attributed to F2 (4KB scratch + stack per leaked subscription). Re-test after F2 is fixed: if go.heap_live then plateaus, refute.
- Supporting evidence: E8

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `internal.workload.workload.topics`
- Region: heap
- Site: `internal/workload/workload.go:19`
- Reason: not_exercised
- Remedy: Static read-only map, no writer after construction; the heap growth in the harness is attributed to F2 (4KB scratch + stack per leaked subscription). Re-test after F2 is fixed: if go.heap_live then plateaus, refute.
- Supporting evidence: E8

## Ruled out

- **F2.n1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (leak) E14
  - Hypothesis: N-WARM: goroutines plateau after warm-up
  - Refuted if: goroutine count keeps growing over extra cycles and after purge
  - Contradicting evidence: E14
  - Originally supported by: E13

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
- E10: limitation: harness written; nothing was run (dla harness run measures it)
- E10: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E10: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E11: partial coverage (9 of 9 examined)
- E11: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: points 1 to 9 come from S1; the extra points from S2
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: points 1 to 9 come from S1; the extra points from S4
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: points 1 to 9 come from S1; the extra points from S6
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: partial coverage (2 of 3 examined)
- E17: limitation: F2: growth measured as a count; bytes not attributed
- E17: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E17: limitation: threshold is provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | — |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | — |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | — |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | — |
| `intake.build` | Does the project build locally? | answered | E1 | — |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | — |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E1 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | — |
| E2 | run-config | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | probe/probe.json |
| E4 | static-hit | static | staticcheck 2025.1.1 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | sweep/go-staticcheck.json |
| E5 | static-hit | static | golangci-lint 2.5.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | sweep/go-golangci-lint.json |
| E6 | static-hit | static | govet 1.24.7 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | sweep/go-govet.json |
| E7 | static-hit | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | sweep/go-dla.json |
| E8 | census-holder | static | dla-census 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | census/go.json |
| E9 | workload | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | workload/workloads.json |
| E10 | workload | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | harness/H1/harness.json |
| E11 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (2c8026360ddd) | harness/H1/run-S1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S1.json (b1932a2d7262) | trend/T1.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (b1932a2d7262) | trend/T1.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S3.json (3ebf5f764db3) | alloc/A1.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S5.json (b1c54f814929) | alloc/A2.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S7.json (c9544b6b3a36) | alloc/A3.json |
| E17 | residual | test | dla 0.1.0 | artifact series/S1.json (b1932a2d7262) | gap/residual-S1-E17.json |
