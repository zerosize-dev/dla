# DLA report

Run: `20261110-162619-8b88`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 8 enumerated, 8 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 6

Residual: heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O1; heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O1; heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O1

Coverage: rust 1760 iterations, 0 of 14 entry points exercised

## Findings

### F1: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Window boxed with Box::into_raw in Session::open (session.rs:19) and passed as callback user pointer; Session::close ignores the pointer returned by Registry::remove, so the allocation is never reclaimed (ownership transfer never paired with Box::from_raw).)
- Growth: `native.live_blocks@src/session.rs:19` 1 count/n (steady, gate pass, E10)
- Evidence classes: context, discriminator, growth, static
- Holder: native-handle/global-ref
- Region: heap
- Shape: steady
- Site: `src/session.rs:19`
- Writers: `src/session.rs:19`
- Removers: `src/registry.rs:29`
- Annotation: Box::into_raw'd Window; Session::close drops the pointer returned by Registry::remove, never Box::from_raw
- Verified: no
- Sibling hypotheses: F1.n1
- Suggested archetype (unconfirmed): L-OWNERSHIP
- Supporting evidence: E4, E8, E10, E11
- Explanation (observed): Session::open at src/session.rs:19 leaks a Box<Window> via Box::into_raw (holder native-handle/global-ref); Session::close calls Registry::remove (src/registry.rs:29) and frees memory only when the returned user pointer is used, but it is discarded, so Box::from_raw never runs.
- Explanation structure: adds: src/session.rs:19; removal: conditional at src/registry.rs:29; evidence: E14

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.registry.Registry.entries`
- Region: heap
- Site: `src/registry.rs:15`
- Writers: `src/registry.rs:22`
- Removers: `src/registry.rs:29`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Registry.entries is removed per session (registry.rs:29) and active=0 at exit; no growth column tied to it. Not a holder of the leak.
- Supporting evidence: E5

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.session.Session.open`
- Region: off-heap
- Site: `src/session.rs:19`
- Reason: not_exercised
- Remedy: Consequence of F1 (same leaked Window allocation at session.rs:19); fixed by freeing the pointer in Session::close, no separate verdict
- Supporting evidence: E5

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: native-handle/global-ref
- Region: heap
- Shape: steady
- Site: `src/window.rs:22`
- Annotation: samples Vec of the leaked Window (consequence of F1)
- Reason: not_exercised
- Remedy: Consequence of F1 (same leaked Window allocation at session.rs:19); fixed by freeing the pointer in Session::close, no separate verdict
- Supporting evidence: E10

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: native-handle/global-ref
- Region: heap
- Shape: steady
- Site: `src/window.rs:9`
- Annotation: device name String of the leaked Window (consequence of F1)
- Reason: not_exercised
- Remedy: Consequence of F1 (same leaked Window allocation at session.rs:19); fixed by freeing the pointer in Session::close, no separate verdict
- Supporting evidence: E9

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Heaptrack stack column of the F1 allocation; same mechanism, no separate verdict
- Supporting evidence: E27

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Heaptrack stack column of the F1 allocation; same mechanism, no separate verdict
- Supporting evidence: E28

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (leak) E11
  - Hypothesis: N-WARM: one-time warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles and no plateau after purge
  - Contradicting evidence: E11
  - Originally supported by: E10

## Blind spots and unknowns

### Coverage gaps

- rust/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- rust/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)
- rust/rapx: blocked (cannot copy the project: Permission denied (os error 13))

### Runtime blind spots

- rust: the allocator shim does not record which entry points ran

### Entry points not exercisable

None recorded.

### Evidence limitations and unknowns

- E5: partial coverage
- E5: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E5: limitation: writers and removers are textual name matches
- E6: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E6: limitation: workloads were discovered by reading files; none was run
- E7: limitation: harness written; nothing was run (dla harness run measures it)
- E7: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E8: partial coverage (9 of 9 examined)
- E8: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: exercised entry points are not measured for rust: the allocator shim sees allocations, not calls
- E9: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: gate thresholds are provisional (not calibrated)
- E10: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: points 1 to 9 come from S1; the extra points from S2
- E11: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E11: limitation: gate and plateau thresholds are provisional (not calibrated)
- E12: partial coverage (0 of 9 examined)
- E12: limitation: heaptrack slows the program down; points 1, 5, 9 of 9 ran under it and no series was written (dla harness run without --tool measures the series)
- E13: limitation: .dla/20261110-162619-8b88/harness/H1/out/p1-native/heaptrack.txt elides 1 group(s) of stacks ("and N from M other places"): their bytes are not read, so a growing site can be missing; print with -m0 -s 50
- E13: limitation: .dla/20261110-162619-8b88/harness/H1/out/p5-native/heaptrack.txt elides 1 group(s) of stacks ("and N from M other places"): their bytes are not read, so a growing site can be missing; print with -m0 -s 50
- E13: limitation: .dla/20261110-162619-8b88/harness/H1/out/p9-native/heaptrack.txt elides 1 group(s) of stacks ("and N from M other places"): their bytes are not read, so a growing site can be missing; print with -m0 -s 50
- E13: limitation: figures are the bytes heaptrack counts as leaked: still allocated when the program exits
- E13: limitation: heaptrack_print rounds sizes (3.02M): figures are accurate to about 1%; -F folded files carry exact bytes
- E14: limitation: .dla/20261110-162619-8b88/harness/H1/out/p1-native/heaptrack.txt elides 1 group(s) of stacks ("and N from M other places"): their bytes are not read, so a growing site can be missing; print with -m0 -s 50
- E14: limitation: .dla/20261110-162619-8b88/harness/H1/out/p5-native/heaptrack.txt elides 1 group(s) of stacks ("and N from M other places"): their bytes are not read, so a growing site can be missing; print with -m0 -s 50
- E14: limitation: .dla/20261110-162619-8b88/harness/H1/out/p9-native/heaptrack.txt elides 1 group(s) of stacks ("and N from M other places"): their bytes are not read, so a growing site can be missing; print with -m0 -s 50
- E14: limitation: figures are the bytes heaptrack counts as leaked: still allocated when the program exits
- E14: limitation: heaptrack_print rounds sizes (3.02M): figures are accurate to about 1%; -F folded files carry exact bytes
- E14: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E15: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E17: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E18: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: points 1 to 9 come from S1; the extra points from S2
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: points 1 to 9 come from S1; the extra points from S2
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E25: limitation: points 1 to 9 come from S1; the extra points from S2
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E26: limitation: points 1 to 9 come from S1; the extra points from S2
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E27: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E28: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: partial coverage (1 of 3 examined)
- E29: limitation: F1: growth measured as a count; bytes not attributed
- E29: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E29: limitation: threshold is provisional (not calibrated)
- E30: partial coverage (1 of 3 examined)
- E30: limitation: F1: growth evidence is from S1, not S2; not subtracted
- E30: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E30: limitation: threshold is provisional (not calibrated)
- E31: partial coverage (1 of 3 examined)
- E31: limitation: F1: growth evidence is from S1, not S3; not subtracted
- E31: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E31: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo 8ba6ce044aa1c1ebf7e8fc28516f4590d46dd902 | — |
| E2 | run-config | static | dla 0.1.0 | repo 8ba6ce044aa1c1ebf7e8fc28516f4590d46dd902 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 8ba6ce044aa1c1ebf7e8fc28516f4590d46dd902 | probe/probe.json |
| E4 | static-hit | static | dla 0.1.0 | repo 8ba6ce044aa1c1ebf7e8fc28516f4590d46dd902 | sweep/rust-dla.json |
| E5 | census-holder | static | dla-census 0.1.0 | repo 8ba6ce044aa1c1ebf7e8fc28516f4590d46dd902 | census/rust.json |
| E6 | workload | static | dla 0.1.0 | repo 8ba6ce044aa1c1ebf7e8fc28516f4590d46dd902 | workload/workloads.json |
| E7 | workload | static | dla 0.1.0 | repo 8ba6ce044aa1c1ebf7e8fc28516f4590d46dd902 | harness/H1/harness.json |
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (4ab086c6d712) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (7a973d0644e4) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (7a973d0644e4) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (89a083685c22) | alloc/A1.json |
| E12 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (e3187b613e8c) | harness/H1/run-heaptrack.json |
| E13 | alloc-stack | live | heaptrack 0.1.0 | artifact allocs/A1.json (e28ef1dd211b) | allocs/A1.json |
| E14 | growth-series | live | heaptrack 0.1.0 | artifact series/S4.json (0e6bff2a8762) | series/S4.json |
| E15 | growth-series | test | dla 0.1.0 | artifact series/S2.json (ac503f663306) | trend/T3.json |
| E16 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (ac503f663306) | trend/T3.json |
| E17 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (0e6bff2a8762) | trend/T4.json |
| E18 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (0e6bff2a8762) | trend/T4.json |
| E19 | growth-series | test | dla 0.1.0 | artifact series/S3.json (89a083685c22) | trend/T5.json |
| E20 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (89a083685c22) | trend/T5.json |
| E21 | growth-series | test | dla 0.1.0 | artifact series/S1.json (7a973d0644e4) | trend/T6.json |
| E22 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (7a973d0644e4) | trend/T6.json |
| E23 | growth-series | test | dla 0.1.0 | artifact series/S2.json (ac503f663306) | trend/T7.json |
| E24 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (ac503f663306) | trend/T7.json |
| E25 | growth-series | test | dla 0.1.0 | artifact series/S3.json (89a083685c22) | trend/T8.json |
| E26 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (89a083685c22) | trend/T8.json |
| E27 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (0e6bff2a8762) | trend/T9.json |
| E28 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (0e6bff2a8762) | trend/T9.json |
| E29 | residual | test | dla 0.1.0 | artifact series/S1.json (7a973d0644e4) | gap/residual-S1-E29.json |
| E30 | residual | test | dla 0.1.0 | artifact series/S2.json (ac503f663306) | gap/residual-S2-E30.json |
| E31 | residual | test | dla 0.1.0 | artifact series/S3.json (89a083685c22) | gap/residual-S3-E31.json |
