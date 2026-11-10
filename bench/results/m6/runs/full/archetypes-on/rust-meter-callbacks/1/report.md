# DLA report

Run: `20261110-202356-34e6`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 10 enumerated, 10 dispositioned; confirmed 3, strong 0, refuted 3, suspected 0, unresolved 4

Residual: heap 56 bytes/n unattributed of 4165 total (threshold 1041.25); heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O1; heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O1; heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O1; heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O1; heap 0 bytes/n unattributed of 0 total (threshold 0); heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O1; heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O1; heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O1; heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O1

Coverage: rust 1760 iterations, 0 of 14 entry points exercised

## Findings

### F3: **L-OWNERSHIP**

- State: confirmed
- Label: L-OWNERSHIP (evidence: E14 growth-series, E13 alloc-stack)
- Growth: `native.live_blocks@src/session.rs:19` 1 count/n (steady, gate pass, E10)
- Evidence classes: context, discriminator, growth, static
- Holder: native-handle/global-ref
- Owner: `src.session.Session.open`
- Region: off-heap
- Shape: steady
- Site: `src/session.rs:19`
- Writers: `src/session.rs:19`
- Removers: `src/session.rs:38`
- Annotation: Box::into_raw leaks the Window; close() only removes the registry entry and never reconstitutes the Box
- Verified: agrees (verifier site `src/session.rs:16`, holder native-handle/global-ref, label L-OWNERSHIP; bundle 3c4fb00a7a51)
- Sibling hypotheses: F3.n1
- Supporting evidence: E5, E8, E10, E11
- Explanation (observed): Session::open at src/session.rs:19 leaks the Window via Box::into_raw as a native-handle/global-ref; the remover at src/session.rs:38 drops only the registry entry, and the box is freed only if someone calls Box::from_raw, which nobody does.
- Explanation structure: adds: src/session.rs:19; removal: conditional at src/session.rs:38; evidence: E14

### F4: **UNCLASSIFIED**

- State: confirmed
- Growth: `native.live_bytes@src/window.rs:22` 4096 bytes/n (steady, gate pass, E9)
- Evidence classes: context, discriminator, growth
- Holder: native-handle/global-ref
- Region: heap
- Shape: steady
- Site: `src/window.rs:22`
- Writers: `src/window.rs:22`
- Annotation: buffer of the leaked Window
- Verified: DISAGREES (site: finding `src/window.rs:22` vs verifier `src/session.rs:33`; label: finding UNCLASSIFIED vs verifier L-OWNERSHIP; bundle e4c4c30c2350); finding mechanism: Window::push at src/window.rs:22 fills the 4096-byte samples buffer of the Window that Session::open leaks as a native-handle/global-ref via Box::into_raw; nothing removes it, the box is never freed, so the buffer survives every closed session.; verifier mechanism: Session::open leaks a Box<Window> via Box::into_raw (session.rs:19) and passes the raw pointer to the registry as user data. Session::close calls registry.remove(self.id) and discards the returned user pointer, never calling Box::from_raw, so each session's Window (4096-byte samples buffer plus device String) is never freed. Registry entry is removed correctly; the heap block is orphaned, growing linearly per session.
- Sibling hypotheses: F4.n1
- Supporting evidence: E9, E10, E8, E17, E20
- Explanation (inferred): Window::push at src/window.rs:22 fills the 4096-byte samples buffer of the Window that Session::open leaks as a native-handle/global-ref via Box::into_raw; nothing removes it, the box is never freed, so the buffer survives every closed session.
- Explanation structure: adds: src/window.rs:22; removal: never

### F5: **UNCLASSIFIED**

- State: confirmed
- Growth: `native.live_bytes@src/window.rs:9` 13 bytes/n (steady, gate pass, E9)
- Evidence classes: context, discriminator, growth
- Holder: native-handle/global-ref
- Region: heap
- Shape: steady
- Site: `src/window.rs:9`
- Writers: `src/window.rs:9`
- Annotation: device name String of the leaked Window
- Verified: DISAGREES (site: finding `src/window.rs:9` vs verifier `src/session.rs:105`; label: finding UNCLASSIFIED vs verifier L-OWNERSHIP; bundle c51ed1817dab); finding mechanism: Window::new at src/window.rs:9 stores the device name String in the Window that Session::open leaks as a native-handle/global-ref via Box::into_raw; nothing removes it, the box is never freed, so the string survives every closed session.; verifier mechanism: Session::open leaks a Box<Window> via Box::into_raw (session.rs:91) and gives the raw pointer to the registry as callback user data. Session::close calls registry.remove(id), which returns the user pointer, but the result is discarded and never passed to Box::from_raw, so each session's Window (about 4 KiB sample buffer plus device String) is never freed; registry entry itself is removed.
- Sibling hypotheses: F5.n1
- Supporting evidence: E9, E10, E8, E17, E21
- Explanation (inferred): Window::new at src/window.rs:9 stores the device name String in the Window that Session::open leaks as a native-handle/global-ref via Box::into_raw; nothing removes it, the box is never freed, so the string survives every closed session.
- Explanation structure: adds: src/window.rs:9; removal: never

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `src/session.rs:19`
- Suggested archetype (unconfirmed): L-OWNERSHIP
- Reason: not_exercised
- Remedy: F1 is the detector hit for the same site and mechanism as confirmed F3; no separate action
- Supporting evidence: E4

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: context, static
- Holder: container-field
- Owner: `src.registry.Registry.entries`
- Region: heap
- Site: `src/registry.rs:15`
- Writers: `src/registry.rs:22`
- Removers: `src/registry.rs:29`
- Annotation: heuristic name match over the project's sources
- Sibling hypotheses: F2.n1
- Reason: not_exercised
- Remedy: Registry.entries has no growth column of its own; the program prints active=0 after the run, so entries are removed on close. Growth seen belongs to F3 (leaked Window).
- Supporting evidence: E5, E8

### F2.n1: Hypothesis N-INTENT: registry entries are removed on close (UNCLASSIFIED)

- State: unresolved(not_exercised)
- Evidence classes: growth
- Sibling hypothesis of: F2
- Hypothesis: N-INTENT: registry entries are removed on close
- Refuted if: entry count grows
- Reason: not_exercised
- Remedy: moot, see F2
- Supporting evidence: E17

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: native-handle/global-ref
- Region: heap
- Shape: steady
- Site: `src/window.rs:22`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F4 (same site src/window.rs:22, same leaked Window); fixing F3 closes it.
- Supporting evidence: E30, E31

## Ruled out

- **F3.n1** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (leak) E11
  - Hypothesis: N-WARM: one-time warm-up or allocator high-water, not per-session
  - Refuted if: growth continues linearly over extra cycles and after trim
  - Contradicting evidence: E11
  - Originally supported by: E10
- **F4.n1** (UNCLASSIFIED): discriminator E20
  - Ruled out by: plateau+purge (leak) E20
  - Hypothesis: N-WARM: warm-up high-water
  - Refuted if: growth continues over extra cycles
  - Contradicting evidence: E20
  - Originally supported by: E17
- **F5.n1** (UNCLASSIFIED): discriminator E21
  - Ruled out by: plateau+purge (leak) E21
  - Hypothesis: N-WARM: warm-up high-water
  - Refuted if: growth continues over extra cycles
  - Contradicting evidence: E21
  - Originally supported by: E17

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
- E12: limitation: valgrind slows the program down; points 1, 5, 9 of 9 ran under it and no series was written (dla harness run without --tool measures the series)
- E13: limitation: memcheck scans memory at exit: still-reachable blocks are live at exit, lost blocks have no pointer left; each file is one process
- E13: limitation: the files are ordered as given, and are runs of different lengths, not dumps of one process
- E14: limitation: memcheck scans memory at exit: still-reachable blocks are live at exit, lost blocks have no pointer left; each file is one process
- E14: limitation: the files are ordered as given, and are runs of different lengths, not dumps of one process
- E14: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E15: limitation: tested in a scratch copy; the project was not changed
- E15: limitation: gate thresholds are provisional (not calibrated); the 0.2 and 0.5 slope fractions are placeholders
- E15: limitation: exercised is null: the harness records entry points, not lines, so whether the edited line ran is not checked
- E15: limitation: native and Rust harnesses have no holder probes: the column is a site group from the allocator shim or an os column, and an edit elsewhere may not move os columns past the gate
- E15: limitation: the treated series lacks the control column native.live_blocks@src/session.rs:19: a patch can rename an owner or move a site, so a patch must keep the measured column to be judged
- E15: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: exercised entry points are not measured for rust: the allocator shim sees allocations, not calls
- E16: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E18: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E19: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: points 1 to 9 come from S1; the extra points from S6
- E20: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E20: limitation: gate and plateau thresholds are provisional (not calibrated)
- E21: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: points 1 to 9 come from S1; the extra points from S8
- E21: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E21: limitation: gate and plateau thresholds are provisional (not calibrated)
- E22: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: points 1 to 9 come from S1; the extra points from S10
- E22: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E22: limitation: gate and plateau thresholds are provisional (not calibrated)
- E23: partial coverage (1 of 3 examined)
- E23: limitation: F3: growth measured as a count; bytes not attributed
- E23: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E23: limitation: threshold is provisional (not calibrated)
- E24: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: partial coverage (1 of 3 examined)
- E26: limitation: F3: growth evidence is from S1, not S2; not subtracted
- E26: limitation: F4: growth measured as a count; bytes not attributed
- E26: limitation: F5: growth measured as a count; bytes not attributed
- E26: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E26: limitation: threshold is provisional (not calibrated)
- E27: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E27: limitation: points 1 to 9 come from S1; the extra points from S2
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E28: limitation: points 1 to 9 come from S1; the extra points from S2
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: partial coverage (1 of 3 examined)
- E29: limitation: F3: growth evidence is from S1, not S3; not subtracted
- E29: limitation: F4: growth evidence is from S1, not S3; not subtracted
- E29: limitation: F5: growth evidence is from S1, not S3; not subtracted
- E29: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E29: limitation: threshold is provisional (not calibrated)
- E30: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E30: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E30: limitation: gate thresholds are provisional (not calibrated)
- E31: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E31: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E31: limitation: gate thresholds are provisional (not calibrated)
- E32: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E32: limitation: gate thresholds are provisional (not calibrated)
- E33: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E33: limitation: gate thresholds are provisional (not calibrated)
- E34: partial coverage (1 of 3 examined)
- E34: limitation: F3: growth evidence is from S1, not S5; not subtracted
- E34: limitation: F4: growth evidence is from S1, not S5; not subtracted
- E34: limitation: F5: growth evidence is from S1, not S5; not subtracted
- E34: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E34: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E34: limitation: threshold is provisional (not calibrated)
- E35: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E35: limitation: gate thresholds are provisional (not calibrated)
- E36: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E36: limitation: gate thresholds are provisional (not calibrated)
- E37: partial coverage (1 of 3 examined)
- E37: limitation: F3: growth evidence is from S1, not S6; not subtracted
- E37: limitation: F4: growth evidence is from S1, not S6; not subtracted
- E37: limitation: F5: growth evidence is from S1, not S6; not subtracted
- E37: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E37: limitation: threshold is provisional (not calibrated)
- E38: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E38: limitation: points 1 to 9 come from S1; the extra points from S6
- E38: limitation: gate thresholds are provisional (not calibrated)
- E39: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E39: limitation: points 1 to 9 come from S1; the extra points from S6
- E39: limitation: gate thresholds are provisional (not calibrated)
- E40: partial coverage (1 of 3 examined)
- E40: limitation: F3: growth evidence is from S1, not S7; not subtracted
- E40: limitation: F4: growth evidence is from S1, not S7; not subtracted
- E40: limitation: F5: growth evidence is from S1, not S7; not subtracted
- E40: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E40: limitation: threshold is provisional (not calibrated)
- E41: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E41: limitation: gate thresholds are provisional (not calibrated)
- E42: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E42: limitation: gate thresholds are provisional (not calibrated)
- E43: partial coverage (1 of 3 examined)
- E43: limitation: F3: growth evidence is from S1, not S8; not subtracted
- E43: limitation: F4: growth evidence is from S1, not S8; not subtracted
- E43: limitation: F5: growth evidence is from S1, not S8; not subtracted
- E43: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E43: limitation: threshold is provisional (not calibrated)
- E44: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E44: limitation: points 1 to 9 come from S1; the extra points from S8
- E44: limitation: gate thresholds are provisional (not calibrated)
- E45: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E45: limitation: points 1 to 9 come from S1; the extra points from S8
- E45: limitation: gate thresholds are provisional (not calibrated)
- E46: partial coverage (1 of 3 examined)
- E46: limitation: F3: growth evidence is from S1, not S9; not subtracted
- E46: limitation: F4: growth evidence is from S1, not S9; not subtracted
- E46: limitation: F5: growth evidence is from S1, not S9; not subtracted
- E46: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E46: limitation: threshold is provisional (not calibrated)
- E47: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E47: limitation: gate thresholds are provisional (not calibrated)
- E48: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E48: limitation: gate thresholds are provisional (not calibrated)
- E49: partial coverage (1 of 3 examined)
- E49: limitation: F3: growth evidence is from S1, not S10; not subtracted
- E49: limitation: F4: growth evidence is from S1, not S10; not subtracted
- E49: limitation: F5: growth evidence is from S1, not S10; not subtracted
- E49: limitation: off-heap: no gap series for S10; run dla gap S10 first (node.external is used when present)
- E49: limitation: threshold is provisional (not calibrated)
- E50: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E50: limitation: points 1 to 9 come from S1; the extra points from S10
- E50: limitation: gate thresholds are provisional (not calibrated)
- E51: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E51: limitation: points 1 to 9 come from S1; the extra points from S10
- E51: limitation: gate thresholds are provisional (not calibrated)
- E52: partial coverage (1 of 3 examined)
- E52: limitation: F3: growth evidence is from S1, not S11; not subtracted
- E52: limitation: F4: growth evidence is from S1, not S11; not subtracted
- E52: limitation: F5: growth evidence is from S1, not S11; not subtracted
- E52: limitation: off-heap: no gap series for S11; run dla gap S11 first (node.external is used when present)
- E52: limitation: threshold is provisional (not calibrated)

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
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (8f875116448f) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (a3204414828e) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (a3204414828e) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (ff88fb975518) | alloc/A1.json |
| E12 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (19dad47f79f2) | harness/H1/run-valgrind.json |
| E13 | alloc-stack | live | valgrind-memcheck 0.1.0 | artifact allocs/A1.json (9690afb396f2) | allocs/A1.json |
| E14 | growth-series | live | valgrind-memcheck 0.1.0 | artifact series/S4.json (7070b595b8bf) | series/S4.json |
| E15 | intervention | test | dla 0.1.0 | artifact series/S5.json (05bab38980b2) | intervene/I1 |
| E16 | growth-series | test | dla 0.1.0 | artifact series/S2.json (16a0e557c61d) | trend/T4.json |
| E17 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (16a0e557c61d) | trend/T4.json |
| E18 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (7070b595b8bf) | trend/T5.json |
| E19 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (7070b595b8bf) | trend/T5.json |
| E20 | discriminator | test | dla 0.1.0 | artifact series/S7.json (d0c08f35cf4f) | alloc/A2.json |
| E21 | discriminator | test | dla 0.1.0 | artifact series/S9.json (742f70d7dfb9) | alloc/A3.json |
| E22 | discriminator | test | dla 0.1.0 | artifact series/S11.json (e9f43fe19054) | alloc/A4.json |
| E23 | residual | test | dla 0.1.0 | artifact series/S1.json (a3204414828e) | gap/residual-S1-E23.json |
| E24 | growth-series | test | dla 0.1.0 | artifact series/S2.json (16a0e557c61d) | trend/T9.json |
| E25 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (16a0e557c61d) | trend/T9.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S2.json (16a0e557c61d) | gap/residual-S2-E26.json |
| E27 | growth-series | test | dla 0.1.0 | artifact series/S3.json (ff88fb975518) | trend/T10.json |
| E28 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (ff88fb975518) | trend/T10.json |
| E29 | residual | test | dla 0.1.0 | artifact series/S3.json (ff88fb975518) | gap/residual-S3-E29.json |
| E30 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (7070b595b8bf) | trend/T11.json |
| E31 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (7070b595b8bf) | trend/T11.json |
| E32 | growth-series | test | dla 0.1.0 | artifact series/S5.json (05bab38980b2) | trend/T12.json |
| E33 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (05bab38980b2) | trend/T12.json |
| E34 | residual | test | dla 0.1.0 | artifact series/S5.json (05bab38980b2) | gap/residual-S5-E34.json |
| E35 | growth-series | test | dla 0.1.0 | artifact series/S6.json (b0e98ce877bf) | trend/T13.json |
| E36 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (b0e98ce877bf) | trend/T13.json |
| E37 | residual | test | dla 0.1.0 | artifact series/S6.json (b0e98ce877bf) | gap/residual-S6-E37.json |
| E38 | growth-series | test | dla 0.1.0 | artifact series/S7.json (d0c08f35cf4f) | trend/T14.json |
| E39 | unit-count-series | test | dla 0.1.0 | artifact series/S7.json (d0c08f35cf4f) | trend/T14.json |
| E40 | residual | test | dla 0.1.0 | artifact series/S7.json (d0c08f35cf4f) | gap/residual-S7-E40.json |
| E41 | growth-series | test | dla 0.1.0 | artifact series/S8.json (9989c3ab5640) | trend/T15.json |
| E42 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (9989c3ab5640) | trend/T15.json |
| E43 | residual | test | dla 0.1.0 | artifact series/S8.json (9989c3ab5640) | gap/residual-S8-E43.json |
| E44 | growth-series | test | dla 0.1.0 | artifact series/S9.json (742f70d7dfb9) | trend/T16.json |
| E45 | unit-count-series | test | dla 0.1.0 | artifact series/S9.json (742f70d7dfb9) | trend/T16.json |
| E46 | residual | test | dla 0.1.0 | artifact series/S9.json (742f70d7dfb9) | gap/residual-S9-E46.json |
| E47 | growth-series | test | dla 0.1.0 | artifact series/S10.json (c1d1f46455b3) | trend/T17.json |
| E48 | unit-count-series | test | dla 0.1.0 | artifact series/S10.json (c1d1f46455b3) | trend/T17.json |
| E49 | residual | test | dla 0.1.0 | artifact series/S10.json (c1d1f46455b3) | gap/residual-S10-E49.json |
| E50 | growth-series | test | dla 0.1.0 | artifact series/S11.json (e9f43fe19054) | trend/T18.json |
| E51 | unit-count-series | test | dla 0.1.0 | artifact series/S11.json (e9f43fe19054) | trend/T18.json |
| E52 | residual | test | dla 0.1.0 | artifact series/S11.json (e9f43fe19054) | gap/residual-S11-E52.json |
