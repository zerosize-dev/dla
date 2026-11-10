# DLA report

Run: `20261110-193310-5c66`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 12 enumerated, 12 dispositioned; confirmed 4, strong 0, refuted 4, suspected 1, unresolved 3

Residual: heap 56 bytes/n unattributed of 4165 total (threshold 1041.25); heap 56 bytes/n unattributed of 4165 total (threshold 1041.25); heap 56 bytes/n unattributed of 4165 total (threshold 1041.25); heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O1; heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O1; heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O1; heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O1; heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O1; heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O1

Coverage: rust 1760 iterations, 0 of 14 entry points exercised

## Findings

### F1: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Raw pointer ownership leak: Box<Window> from Box::into_raw in Session::open is never reclaimed in Session::close (the pointer returned by Registry::remove is dropped unused).)
- Growth: `native.live_blocks@src/session.rs:19` 1 count/n (steady, gate pass, E10)
- Evidence classes: context, discriminator, growth, static
- Holder: native-handle/global-ref
- Region: heap
- Shape: steady
- Site: `src/session.rs:19`
- Writers: `src/session.rs:19`
- Annotation: Session::open leaks a Box<Window> via Box::into_raw (session.rs:19); Session::close discards the pointer returned by Registry::remove (session.rs:36) and never calls Box::from_raw, so each session's Window, its 512-slot Vec and device String are never freed
- Verified: no
- Sibling hypotheses: F1.n1
- Supporting evidence: E4, E8, E10, E11
- Explanation (inferred): Session::open at src/session.rs:19 stores a Box<Window> as a native-handle/global-ref raw pointer (owner src.session.Session.open); Session::close drops the pointer registry.remove returns and never frees it, so removal never happens.
- Explanation structure: adds: src/session.rs:19; removal: never

### F3: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Raw pointer ownership leak: Box<Window> from Box::into_raw in Session::open is never reclaimed in Session::close (the pointer returned by Registry::remove is dropped unused).)
- Growth: `native.live_blocks@src/session.rs:19` 1 count/n (steady, gate pass, E10)
- Evidence classes: context, discriminator, growth, static
- Holder: native-handle/global-ref
- Owner: `src.session.Session.open`
- Region: off-heap
- Shape: steady
- Site: `src/session.rs:19`
- Writers: `src/session.rs:19`
- Annotation: Session::open leaks a Box<Window> via Box::into_raw (session.rs:19); Session::close discards the pointer returned by Registry::remove (session.rs:36) and never calls Box::from_raw, so each session's Window, its 512-slot Vec and device String are never freed
- Verified: no
- Sibling hypotheses: F3.n1
- Supporting evidence: E5, E8, E10, E16
- Explanation (inferred): Session::open at src/session.rs:19 stores a Box<Window> as a native-handle/global-ref raw pointer (owner src.session.Session.open); Session::close drops the pointer registry.remove returns and never frees it, so removal never happens.
- Explanation structure: adds: src/session.rs:19; removal: never

### F7: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Raw pointer ownership leak: Box<Window> from Box::into_raw in Session::open is never reclaimed in Session::close (the pointer returned by Registry::remove is dropped unused).)
- Growth: `native.live_bytes@src/window.rs:22` 4096 bytes/n (steady, gate pass, E9)
- Evidence classes: context, discriminator, growth
- Holder: native-handle/global-ref
- Owner: `src.session.Session.open`
- Region: heap
- Shape: steady
- Site: `src/window.rs:22`
- Writers: `src/session.rs:19`
- Annotation: Session::open leaks a Box<Window> via Box::into_raw (session.rs:19); Session::close discards the pointer returned by Registry::remove (session.rs:36) and never calls Box::from_raw, so each session's Window, its 512-slot Vec and device String are never freed
- Verified: no
- Sibling hypotheses: F7.n1
- Supporting evidence: E9, E10, E12, E13, E14, E15, E8, E17
- Explanation (inferred): The 512-slot samples Vec allocated at src/window.rs:22 lives inside the Window that Session::open leaks (native-handle/global-ref owner src.session.Session.open); Session::close never frees it, so removal never happens.
- Explanation structure: adds: src/window.rs:22; removal: never

### F8: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Raw pointer ownership leak: Box<Window> from Box::into_raw in Session::open is never reclaimed in Session::close (the pointer returned by Registry::remove is dropped unused).)
- Growth: `native.live_bytes@src/window.rs:9` 13 bytes/n (steady, gate pass, E9)
- Evidence classes: context, discriminator, growth
- Holder: native-handle/global-ref
- Owner: `src.session.Session.open`
- Region: heap
- Shape: steady
- Site: `src/window.rs:9`
- Writers: `src/session.rs:19`
- Annotation: Session::open leaks a Box<Window> via Box::into_raw (session.rs:19); Session::close discards the pointer returned by Registry::remove (session.rs:36) and never calls Box::from_raw, so each session's Window, its 512-slot Vec and device String are never freed
- Verified: no
- Sibling hypotheses: F8.n1
- Supporting evidence: E9, E10, E12, E13, E14, E15, E8, E18
- Explanation (inferred): The device String cloned at src/window.rs:9 lives inside the Window that Session::open leaks (native-handle/global-ref owner src.session.Session.open); Session::close never frees it, so removal never happens.
- Explanation structure: adds: src/window.rs:9; removal: never

### F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.registry.Registry.entries`
- Region: heap
- Site: `src/registry.rs:15`
- Writers: `src/registry.rs:22`
- Removers: `src/registry.rs:29`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Registry.entries is removed on Session::close (registry.rs remove) and active=0 at exit; not a growth source. Only the Window allocation leaks (F1).
- Supporting evidence: E5

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Duplicate of confirmed F1/F7/F8 (same leaked Window allocation at src/session.rs:19); no separate test needed
- Supporting evidence: E10

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: native-handle/global-ref
- Owner: `src.session.Session.open`
- Region: heap
- Shape: steady
- Site: `src/window.rs:22`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F1/F7/F8 (same leaked Window allocation at src/session.rs:19); no separate test needed
- Supporting evidence: E10

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: native-handle/global-ref
- Owner: `src.session.Session.open`
- Region: heap
- Shape: steady
- Site: `src/window.rs:9`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F1/F7/F8 (same leaked Window allocation at src/session.rs:19); no separate test needed
- Supporting evidence: E10

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (leak) E11
  - Hypothesis: N-WARM: live blocks plateau after warm-up
  - Refuted if: growth continues over the extra cycles and survives purge
  - Contradicting evidence: E11
  - Originally supported by: E10
- **F3.n1** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (leak) E16
  - Hypothesis: N-WARM: live blocks plateau after warm-up
  - Refuted if: growth continues over the extra cycles and survives purge
  - Contradicting evidence: E16
  - Originally supported by: E10
- **F7.n1** (UNCLASSIFIED): discriminator E17
  - Ruled out by: plateau+purge (leak) E17
  - Hypothesis: N-WARM: live blocks plateau after warm-up
  - Refuted if: growth continues over the extra cycles and survives purge
  - Contradicting evidence: E17
  - Originally supported by: E10
- **F8.n1** (UNCLASSIFIED): discriminator E18
  - Ruled out by: plateau+purge (leak) E18
  - Hypothesis: N-WARM: live blocks plateau after warm-up
  - Refuted if: growth continues over the extra cycles and survives purge
  - Contradicting evidence: E18
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
- E16: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: points 1 to 9 come from S1; the extra points from S4
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: points 1 to 9 come from S1; the extra points from S6
- E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E17: limitation: gate and plateau thresholds are provisional (not calibrated)
- E18: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: points 1 to 9 come from S1; the extra points from S8
- E18: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E18: limitation: gate and plateau thresholds are provisional (not calibrated)
- E19: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: partial coverage (1 of 3 examined)
- E25: limitation: F1: growth measured as a count; bytes not attributed
- E25: limitation: F3: growth measured as a count; bytes not attributed
- E25: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E25: limitation: threshold is provisional (not calibrated)
- E26: partial coverage (1 of 3 examined)
- E26: limitation: F1: growth evidence is from S1, not S2; not subtracted
- E26: limitation: F3: growth evidence is from S1, not S2; not subtracted
- E26: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E26: limitation: threshold is provisional (not calibrated)
- E27: partial coverage (1 of 3 examined)
- E27: limitation: F1: growth evidence is from S1, not S3; not subtracted
- E27: limitation: F3: growth evidence is from S1, not S3; not subtracted
- E27: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E27: limitation: threshold is provisional (not calibrated)
- E28: partial coverage (1 of 3 examined)
- E28: limitation: F1: growth evidence is from S1, not S4; not subtracted
- E28: limitation: F3: growth evidence is from S1, not S4; not subtracted
- E28: limitation: F7: growth evidence is from S1, not S4; not subtracted
- E28: limitation: F8: growth evidence is from S1, not S4; not subtracted
- E28: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E28: limitation: threshold is provisional (not calibrated)
- E29: partial coverage (1 of 3 examined)
- E29: limitation: F1: growth evidence is from S1, not S5; not subtracted
- E29: limitation: F3: growth evidence is from S1, not S5; not subtracted
- E29: limitation: F7: growth evidence is from S1, not S5; not subtracted
- E29: limitation: F8: growth evidence is from S1, not S5; not subtracted
- E29: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E29: limitation: threshold is provisional (not calibrated)
- E30: partial coverage (1 of 3 examined)
- E30: limitation: F1: growth evidence is from S1, not S6; not subtracted
- E30: limitation: F3: growth evidence is from S1, not S6; not subtracted
- E30: limitation: F7: growth evidence is from S1, not S6; not subtracted
- E30: limitation: F8: growth evidence is from S1, not S6; not subtracted
- E30: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E30: limitation: threshold is provisional (not calibrated)
- E31: partial coverage (1 of 3 examined)
- E31: limitation: F1: growth evidence is from S1, not S7; not subtracted
- E31: limitation: F3: growth evidence is from S1, not S7; not subtracted
- E31: limitation: F7: growth evidence is from S1, not S7; not subtracted
- E31: limitation: F8: growth evidence is from S1, not S7; not subtracted
- E31: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E31: limitation: threshold is provisional (not calibrated)
- E32: partial coverage (1 of 3 examined)
- E32: limitation: F1: growth evidence is from S1, not S8; not subtracted
- E32: limitation: F3: growth evidence is from S1, not S8; not subtracted
- E32: limitation: F7: growth evidence is from S1, not S8; not subtracted
- E32: limitation: F8: growth evidence is from S1, not S8; not subtracted
- E32: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E32: limitation: threshold is provisional (not calibrated)
- E33: partial coverage (1 of 3 examined)
- E33: limitation: F1: growth evidence is from S1, not S9; not subtracted
- E33: limitation: F3: growth evidence is from S1, not S9; not subtracted
- E33: limitation: F7: growth evidence is from S1, not S9; not subtracted
- E33: limitation: F8: growth evidence is from S1, not S9; not subtracted
- E33: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E33: limitation: threshold is provisional (not calibrated)

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
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (f8c30b5c9b44) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (42d8b4903c0e) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (42d8b4903c0e) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (fcfd8aefc0d4) | alloc/A1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S2.json (9cf36239fb95) | trend/T3.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (9cf36239fb95) | trend/T3.json |
| E14 | growth-series | test | dla 0.1.0 | artifact series/S3.json (fcfd8aefc0d4) | trend/T4.json |
| E15 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (fcfd8aefc0d4) | trend/T4.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S5.json (8701e754c531) | alloc/A2.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S7.json (9a7f361a9027) | alloc/A3.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S9.json (8954ef652d1c) | alloc/A4.json |
| E19 | growth-series | test | dla 0.1.0 | artifact series/S4.json (01f69078ad58) | trend/T8.json |
| E20 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (01f69078ad58) | trend/T8.json |
| E21 | growth-series | test | dla 0.1.0 | artifact series/S6.json (4b32c513a9e9) | trend/T9.json |
| E22 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (4b32c513a9e9) | trend/T9.json |
| E23 | growth-series | test | dla 0.1.0 | artifact series/S8.json (3e641ee4374d) | trend/T10.json |
| E24 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (3e641ee4374d) | trend/T10.json |
| E25 | residual | test | dla 0.1.0 | artifact series/S1.json (42d8b4903c0e) | gap/residual-S1-E25.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S2.json (9cf36239fb95) | gap/residual-S2-E26.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S3.json (fcfd8aefc0d4) | gap/residual-S3-E27.json |
| E28 | residual | test | dla 0.1.0 | artifact series/S4.json (01f69078ad58) | gap/residual-S4-E28.json |
| E29 | residual | test | dla 0.1.0 | artifact series/S5.json (8701e754c531) | gap/residual-S5-E29.json |
| E30 | residual | test | dla 0.1.0 | artifact series/S6.json (4b32c513a9e9) | gap/residual-S6-E30.json |
| E31 | residual | test | dla 0.1.0 | artifact series/S7.json (9a7f361a9027) | gap/residual-S7-E31.json |
| E32 | residual | test | dla 0.1.0 | artifact series/S8.json (3e641ee4374d) | gap/residual-S8-E32.json |
| E33 | residual | test | dla 0.1.0 | artifact series/S9.json (8954ef652d1c) | gap/residual-S9-E33.json |
