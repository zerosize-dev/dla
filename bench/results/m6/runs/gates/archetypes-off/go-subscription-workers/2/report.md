# DLA report

Run: `20261110-185647-787a`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 4 enumerated, 4 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 2

Residual: heap 5441.145454545454 bytes/iteration unattributed of 5441.145454545454 total (threshold 1360.2863636363636) ABOVE THRESHOLD, see O1; heap 5441.554545454545 bytes/iteration unattributed of 5441.554545454545 total (threshold 1360.3886363636364) ABOVE THRESHOLD, see O3; heap 5441.839669421488 bytes/iteration unattributed of 5441.839669421488 total (threshold 1360.459917355372) ABOVE THRESHOLD, see O3; non-heap 2122.4727272727273 bytes/iteration unattributed of 2122.4727272727273 total (threshold 530.6181818181818) ABOVE THRESHOLD, see O2; non-heap 2122.4727272727273 bytes/iteration unattributed of 2122.4727272727273 total (threshold 530.6181818181818) ABOVE THRESHOLD, see O4; non-heap 2134.8848484848486 bytes/iteration unattributed of 2134.8848484848486 total (threshold 533.7212121212121) ABOVE THRESHOLD, see O4

Coverage: go 7040 iterations, 0 of 0 entry points exercised

## Findings

### F2: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Per-request Subscription worker goroutine (subscription.go:33) parked on select forever because Handler.Serve (handler.go:20) never calls Subscription.Close; each pins a 4KB scratch buffer, 4 channels and a stack.)
- Growth: `go.goroutines@internal/notify/subscription.go:33` 1 count/iteration (steady, gate pass, E14)
- Evidence classes: context, growth, static
- Holder: blocked-stack
- Owner: `internal.notify.subscription.newSubscription -> s.run()`
- Region: non-heap
- Shape: steady
- Site: `internal/notify/subscription.go:33`
- Writers: `internal/notify/subscription.go:33`
- Annotation: Handler.Serve opens a Subscription per request and never calls Close; run() goroutine blocks on select forever, pinning scratch buffer and channels
- Verified: no
- Sibling hypotheses: F2.n1
- Supporting evidence: E8, E10, E14
- Explanation (inferred): Blocked-stack holder at subscription.go:33: Handler.Serve opens a Subscription per request via Hub.Open and never calls Close, so each run goroutine stays blocked in select and is never removed.
- Explanation structure: adds: internal/notify/subscription.go:33; removal: never

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `internal.notify.render.Renderer.layouts`
- Region: heap
- Site: `internal/notify/render.go:10`
- Reason: not_exercised
- Remedy: census lead only; no growth tied in harness
- Supporting evidence: E8

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `internal.workload.workload.topics`
- Region: heap
- Site: `internal/workload/workload.go:19`
- Reason: not_exercised
- Remedy: topics is a read-only package var; probe size constant
- Supporting evidence: E8

## Ruled out

- **F2.n1** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (leak) E15
  - Hypothesis: N-WARM: goroutine count is a warm-up plateau
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E15
  - Originally supported by: E14

## Blind spots and unknowns

### Coverage gaps

- go/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- go/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)

### Runtime blind spots

None recorded.

### Entry points not exercisable

None recorded.

### Evidence limitations and unknowns

- E8: partial coverage
- E8: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E8: limitation: writers and removers are textual name matches
- E9: limitation: harness written; nothing was run (dla harness run measures it)
- E9: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E9: limitation: goleak not available offline: the goleak check is skipped
- E10: partial coverage (9 of 9 examined)
- E10: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E10: limitation: no entry points are listed for this runtime: run dla workload discover before the harness to see which ones ran
- E11: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: goleak not available offline: the goleak check is skipped
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: partial coverage (2 of 3 examined)
- E16: limitation: F2: growth evidence is from S2, not S1; not subtracted
- E16: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E16: limitation: threshold is provisional (not calibrated)
- E17: partial coverage (2 of 3 examined)
- E17: limitation: F2: growth measured as a count; bytes not attributed
- E17: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E17: limitation: threshold is provisional (not calibrated)
- E18: partial coverage (2 of 3 examined)
- E18: limitation: F2: growth evidence is from S2, not S3; not subtracted
- E18: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E18: limitation: threshold is provisional (not calibrated)
- E19: partial coverage (2 of 3 examined)
- E19: limitation: F2: growth measured as a count; bytes not attributed
- E19: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E19: limitation: threshold is provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | go repo notifyd; whole repo; unattended, consent E2 |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | go repo notifyd; whole repo; unattended, consent E2 |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | go repo notifyd; whole repo; unattended, consent E2 |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | go repo notifyd; whole repo; unattended, consent E2 |
| `intake.build` | Does the project build locally? | answered | E1 | go repo notifyd; whole repo; unattended, consent E2 |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | go repo notifyd; whole repo; unattended, consent E2 |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E1 | go repo notifyd; whole repo; unattended, consent E2 |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | go repo notifyd; whole repo; unattended, consent E2 |

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
| E9 | workload | static | dla 0.1.0 | repo 071d2e89d988c3ca686b967716ee79389e273c1d | harness/H1/harness.json |
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (cf713f34e5a2) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (e3d38ac868d4) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (e3d38ac868d4) | trend/T1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S2.json (b9067c3ecf17) | trend/T2.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (b9067c3ecf17) | trend/T2.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S3.json (7c457768d9a8) | alloc/A1.json |
| E16 | residual | test | dla 0.1.0 | artifact series/S1.json (e3d38ac868d4) | gap/residual-S1-E16.json |
| E17 | residual | test | dla 0.1.0 | artifact series/S2.json (b9067c3ecf17) | gap/residual-S2-E17.json |
| E18 | residual | test | dla 0.1.0 | artifact series/S3.json (7c457768d9a8) | gap/residual-S3-E18.json |
| E19 | residual | test | dla 0.1.0 | artifact series/S2.json (b9067c3ecf17) | gap/residual-S2-E19.json |
