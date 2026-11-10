# DLA report

Run: `20261110-182100-7c6e`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 10 enumerated, 10 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 8

Residual: heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O1; heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O1; heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O1

Coverage: rust 1760 iterations, 0 of 14 entry points exercised

## Findings

### F3: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Session::open leaks a Box<Window> via Box::into_raw; Session::close discards the pointer returned by Registry::remove, so the Window (samples Vec 4KiB + device String) is never freed)
- Growth: `native.live_blocks@src/session.rs:19` 1 count/n (steady, gate pass, E10)
- Evidence classes: context, discriminator, growth, static
- Holder: native-handle/global-ref
- Owner: `src.session.Session.open`
- Region: off-heap
- Shape: steady
- Site: `src/session.rs:19`
- Writers: `src/session.rs:19`
- Removers: `src/registry.rs:29`
- Annotation: Box::into_raw leaks Window; Session::close discards registry.remove's returned pointer
- Annotation: registry.remove returns the user pointer but session.rs:close ignores it; no Box::from_raw anywhere
- Verified: no
- Sibling hypotheses: F3.n1
- Supporting evidence: E5, E8, E10, E11
- Explanation (inferred): Session::open leaks a Box<Window> (native-handle holder at src/session.rs:19 via Box::into_raw); Registry::remove at registry.rs:29 drops only the entry, and the Window is freed only when the returned pointer is reclaimed, which Session::close never does.
- Explanation structure: adds: src/session.rs:19; removal: conditional at src/registry.rs:29

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `src/session.rs:19`
- Reason: not_exercised
- Remedy: Same leak as confirmed F3 (Window allocations at window.rs:9/22 are retained by the Box leaked at session.rs:19); fixing F3 releases them
- Supporting evidence: E4

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
- Remedy: Registry.entries is removed per session (registry.rs:29) and active=0 at exit; no growth column observed at registry.rs:15
- Supporting evidence: E5

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Same leak as confirmed F3 (Window allocations at window.rs:9/22 are retained by the Box leaked at session.rs:19); fixing F3 releases them
- Supporting evidence: E10

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Same leak as confirmed F3 (Window allocations at window.rs:9/22 are retained by the Box leaked at session.rs:19); fixing F3 releases them
- Supporting evidence: E9

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Same leak as confirmed F3 (Window allocations at window.rs:9/22 are retained by the Box leaked at session.rs:19); fixing F3 releases them
- Supporting evidence: E13

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Same leak as confirmed F3 (Window allocations at window.rs:9/22 are retained by the Box leaked at session.rs:19); fixing F3 releases them
- Supporting evidence: E12

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Same leak as confirmed F3 (Window allocations at window.rs:9/22 are retained by the Box leaked at session.rs:19); fixing F3 releases them
- Supporting evidence: E15

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Same leak as confirmed F3 (Window allocations at window.rs:9/22 are retained by the Box leaked at session.rs:19); fixing F3 releases them
- Supporting evidence: E14

## Ruled out

- **F3.n1** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (leak) E11
  - Hypothesis: N-WARM: growth is a warm-up fill that plateaus
  - Refuted if: growth continues linearly over extra cycles and after purge
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
- E12: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: points 1 to 9 come from S1; the extra points from S2
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: points 1 to 9 come from S1; the extra points from S2
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: partial coverage (1 of 3 examined)
- E16: limitation: F3: growth measured as a count; bytes not attributed
- E16: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E16: limitation: threshold is provisional (not calibrated)
- E17: partial coverage (1 of 3 examined)
- E17: limitation: F3: growth evidence is from S1, not S2; not subtracted
- E17: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E17: limitation: threshold is provisional (not calibrated)
- E18: partial coverage (1 of 3 examined)
- E18: limitation: F3: growth evidence is from S1, not S3; not subtracted
- E18: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E18: limitation: threshold is provisional (not calibrated)

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
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (0c8d5f6bad3b) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (269d75ff990f) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (269d75ff990f) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (0835fdf857e1) | alloc/A1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S2.json (ef1a1b9a4ab4) | trend/T3.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (ef1a1b9a4ab4) | trend/T3.json |
| E14 | growth-series | test | dla 0.1.0 | artifact series/S3.json (0835fdf857e1) | trend/T4.json |
| E15 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (0835fdf857e1) | trend/T4.json |
| E16 | residual | test | dla 0.1.0 | artifact series/S1.json (269d75ff990f) | gap/residual-S1-E16.json |
| E17 | residual | test | dla 0.1.0 | artifact series/S2.json (ef1a1b9a4ab4) | gap/residual-S2-E17.json |
| E18 | residual | test | dla 0.1.0 | artifact series/S3.json (0835fdf857e1) | gap/residual-S3-E18.json |
