# DLA report

Run: `20261110-165908-2fa9`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 10 enumerated, 10 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 8

Residual: heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O1; heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O1; heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O1

Coverage: rust 1760 iterations, 0 of 14 entry points exercised

## Findings

### F1: **L-OWNERSHIP**

- State: confirmed
- Label: L-OWNERSHIP (evidence: E18 growth-series, E17 alloc-stack); region does not match the archetype's
- Growth: `native.live_blocks@src/session.rs:19` 1 count/n (steady, gate pass, E10)
- Evidence classes: context, discriminator, growth, static
- Holder: native-handle/global-ref
- Region: heap
- Shape: steady
- Site: `src/session.rs:19`
- Writers: `src/session.rs:19`
- Removers: `src/registry.rs:29`
- Annotation: Session::open leaks Box<Window> via Box::into_raw; close() only removes the registry entry and never reclaims the raw pointer
- Verified: no
- Sibling hypotheses: F1.n1
- Suggested archetype (unconfirmed): L-OWNERSHIP
- Supporting evidence: E4, E8, E10, E11
- Explanation (observed): Holder native-handle/global-ref at src/session.rs:19: Session::open turns a boxed Window into a raw pointer via Box::into_raw; Session::close calls Registry::remove (registry.rs:29), which frees the table entry only if nothing else owns it, never Box::from_raw, so every Window leaks.
- Explanation structure: adds: src/session.rs:19; removal: conditional at src/registry.rs:29; evidence: E18

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
- Remedy: Growth is the same per-session Window allocation leaked by confirmed F1 (src/session.rs:19); covered by F1, fix F1 and re-measure
- Supporting evidence: E5

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.session.Session.open`
- Region: off-heap
- Site: `src/session.rs:19`
- Reason: not_exercised
- Remedy: Same site and growth as confirmed F1 (src/session.rs:19); duplicate census candidate, covered by F1
- Supporting evidence: E5

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Growth is the same per-session Window allocation leaked by confirmed F1 (src/session.rs:19); covered by F1, fix F1 and re-measure
- Supporting evidence: E15

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Growth is the same per-session Window allocation leaked by confirmed F1 (src/session.rs:19); covered by F1, fix F1 and re-measure
- Supporting evidence: E14

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Growth is the same per-session Window allocation leaked by confirmed F1 (src/session.rs:19); covered by F1, fix F1 and re-measure
- Supporting evidence: E9

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Growth is the same per-session Window allocation leaked by confirmed F1 (src/session.rs:19); covered by F1, fix F1 and re-measure
- Supporting evidence: E12

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Growth is the same per-session Window allocation leaked by confirmed F1 (src/session.rs:19); covered by F1, fix F1 and re-measure
- Supporting evidence: E13

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Growth is the same per-session Window allocation leaked by confirmed F1 (src/session.rs:19); covered by F1, fix F1 and re-measure
- Supporting evidence: E19, E20

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (leak) E11
  - Hypothesis: N-WARM: warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles and after purge
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
- E16: partial coverage (0 of 9 examined)
- E16: limitation: valgrind slows the program down; points 1, 5, 9 of 9 ran under it and no series was written (dla harness run without --tool measures the series)
- E17: limitation: memcheck scans memory at exit: still-reachable blocks are live at exit, lost blocks have no pointer left; each file is one process
- E17: limitation: the files are ordered as given, and are runs of different lengths, not dumps of one process
- E18: limitation: memcheck scans memory at exit: still-reachable blocks are live at exit, lost blocks have no pointer left; each file is one process
- E18: limitation: the files are ordered as given, and are runs of different lengths, not dumps of one process
- E18: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E19: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E19: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E20: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: partial coverage (1 of 3 examined)
- E21: limitation: F1: growth measured as a count; bytes not attributed
- E21: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E21: limitation: threshold is provisional (not calibrated)
- E22: partial coverage (1 of 3 examined)
- E22: limitation: F1: growth evidence is from S1, not S2; not subtracted
- E22: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E22: limitation: threshold is provisional (not calibrated)
- E23: partial coverage (1 of 3 examined)
- E23: limitation: F1: growth evidence is from S1, not S3; not subtracted
- E23: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E23: limitation: threshold is provisional (not calibrated)

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
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
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
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (13c532c0b497) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (d279a380fc28) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (d279a380fc28) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (b56dbf92d5b6) | alloc/A1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S2.json (eb41db932981) | trend/T3.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (eb41db932981) | trend/T3.json |
| E14 | growth-series | test | dla 0.1.0 | artifact series/S3.json (b56dbf92d5b6) | trend/T4.json |
| E15 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (b56dbf92d5b6) | trend/T4.json |
| E16 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (14c2cc333e50) | harness/H1/run-valgrind.json |
| E17 | alloc-stack | live | valgrind-memcheck 0.1.0 | artifact allocs/A1.json (69caa2a65b1d) | allocs/A1.json |
| E18 | growth-series | live | valgrind-memcheck 0.1.0 | artifact series/S4.json (df84afb05041) | series/S4.json |
| E19 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (df84afb05041) | trend/T5.json |
| E20 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (df84afb05041) | trend/T5.json |
| E21 | residual | test | dla 0.1.0 | artifact series/S1.json (d279a380fc28) | gap/residual-S1-E21.json |
| E22 | residual | test | dla 0.1.0 | artifact series/S2.json (eb41db932981) | gap/residual-S2-E22.json |
| E23 | residual | test | dla 0.1.0 | artifact series/S3.json (b56dbf92d5b6) | gap/residual-S3-E23.json |
