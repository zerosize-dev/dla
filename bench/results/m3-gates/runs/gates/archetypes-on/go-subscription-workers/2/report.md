# DLA report

Run: `20261108-125454-1103`

Status: COMPLETE

Findings: 4 enumerated, 4 dispositioned; confirmed 1, strong 0, refuted 1, suspected 2, unresolved 0

Residual: not measured

Coverage: go 7040 iterations, 0 of 19 entry points exercised

## Findings

### F2: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `go.goroutines@internal/notify/subscription.go:33` 1 count/iteration (steady, gate pass, E17)
- Evidence classes: context, growth, static
- Holder: blocked-stack
- Owner: `internal.notify.subscription.newSubscription -> s.run()`
- Region: non-heap
- Shape: steady
- Site: `internal/notify/subscription.go:33`
- Writers: `internal/notify/hub.go:30`, `internal/notify/handler.go:20`
- Removers: `internal/notify/subscription.go:88`
- Annotation: Hub.Open spawns one goroutine per Serve; Close (the only stopper) is never called by Handler.Serve
- Sibling hypotheses: F2.n1
- Supporting evidence: E8, E11, E17

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: discriminator, static
- Holder: container-field
- Owner: `internal.notify.render.Renderer.layouts`
- Region: heap
- Site: `internal/notify/render.go:10`
- Writers: `internal/notify/render.go:19`
- Annotation: layouts copied once at NewRenderer, read-only afterwards; no writers post-construction
- Reason: not_exercised
- Remedy: Read-only after construction (no writers); heap growth in the same region is explained by F2's per-subscription buffers. Re-check with a per-site heap profile if desired.
- Supporting evidence: E8, E20

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: discriminator, static
- Holder: static/global
- Owner: `internal.workload.workload.topics`
- Region: heap
- Site: `internal/workload/workload.go:19`
- Annotation: topics: fixed slice, read-only
- Reason: not_exercised
- Remedy: Read-only after construction (no writers); heap growth in the same region is explained by F2's per-subscription buffers. Re-check with a per-site heap profile if desired.
- Supporting evidence: E8, E20

## Ruled out

- **F2.n1** (UNCLASSIFIED): discriminator E19
  - Ruled out by: plateau+purge (leak) E19
  - Hypothesis: N-WARM warm-up cache fill, plateaus
  - Refuted if: goroutine count keeps growing over extra cycles and after purge
  - Contradicting evidence: E19
  - Originally supported by: E17

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
- E10: limitation: goleak not available offline: the goleak check is skipped
- E11: partial coverage (9 of 9 examined)
- E11: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E11: limitation: no Go coverage profile was written to cov/
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: fresh keys S1 against repeated keys S2
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E19: limitation: goleak not available offline: the goleak check is skipped
- E19: limitation: gate and plateau thresholds are provisional (not calibrated)
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E20: limitation: goleak not available offline: the goleak check is skipped
- E20: limitation: gate and plateau thresholds are provisional (not calibrated)
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E21: limitation: goleak not available offline: the goleak check is skipped
- E21: limitation: gate and plateau thresholds are provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | Local Go repo notifyd (cmd/notifyd, internal/notify, internal/workload) |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | Whole repository |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | Memory and thread count rise after each replay batch; OOMKilled ~daily |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | go |
| `intake.build` | Does the project build locally? | answered | E1 | go build ./... (go 1.24) |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | internal/workload replay driver; discover via dla |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E1 | Runner consented tiers 1-4 and raw values (E2) |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 60m |

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
| E10 | workload | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | harness/H1/harness.json |
| E11 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (d9fea6deb736) | harness/H1/run-S1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S1.json (d74ae9a52dbe) | trend/T1.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (d74ae9a52dbe) | trend/T1.json |
| E14 | growth-series | test | dla 0.1.0 | artifact series/S2.json (f53526aa1bd1) | trend/T2.json |
| E15 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (f53526aa1bd1) | trend/T2.json |
| E16 | growth-series | test | dla 0.1.0 | artifact series/S1.json (d74ae9a52dbe) | trend/T3.json |
| E17 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (d74ae9a52dbe) | trend/T3.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S1.json (d74ae9a52dbe) | trend/T3-keys.json |
| E19 | discriminator | test | dla 0.1.0 | artifact series/S3.json (24d8e7294bfb) | alloc/A1.json |
| E20 | discriminator | test | dla 0.1.0 | artifact series/S4.json (053defee1fc6) | alloc/A2.json |
| E21 | discriminator | test | dla 0.1.0 | artifact series/S5.json (39039a69a63c) | alloc/A3.json |
