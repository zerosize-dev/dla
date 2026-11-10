# DLA report

Run: `20261110-164717-207b`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 6 enumerated, 6 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 4

Residual: heap 1103.2103896103897 bytes/n unattributed of 1103.2103896103897 total (threshold 275.80259740259737) ABOVE THRESHOLD, see O1; heap 1079.0787878787878 bytes/n unattributed of 1079.0787878787878 total (threshold 269.76969696969695) ABOVE THRESHOLD, see O1; heap 1079.1845730027549 bytes/n unattributed of 1079.1845730027549 total (threshold 269.7961432506887) ABOVE THRESHOLD, see O1; heap 1103.2103896103897 bytes/n unattributed of 1103.2103896103897 total (threshold 275.80259740259737) ABOVE THRESHOLD, see O1

Coverage: native 3520 iterations, 0 of 10 entry points exercised

## Findings

### F1: **L-HANDLE**

- State: confirmed
- Label: L-HANDLE (evidence: E9 growth-series, E18 alloc-stack); region does not match the archetype's
- Growth: `native.live_blocks@src/decoder.c:29` 0.0625 count/n (steady, gate pass, E10)
- Evidence classes: context, growth, static
- Holder: native-handle/global-ref
- Region: heap
- Shape: steady
- Site: `src/decoder.c:29`
- Writers: `src/decoder.c:29`
- Removers: `src/decoder.c:34`
- Annotation: malloc at 29 freed on the payload-error path (34) and by frame_release, but not on the checksum-mismatch return at 40
- Verified: no
- Sibling hypotheses: F1.n1
- Suggested archetype (unconfirmed): L-OWNERSHIP
- Supporting evidence: E4, E8, E10
- Explanation (observed): decode_frame (src/decoder.c:29) mallocs buf; holder native-handle/global-ref, a raw heap block. The buffer is freed (src/decoder.c:34) only when the payload fails; if the checksum mismatches, the return skips free and the damaged frame's buffer leaks.
- Explanation structure: adds: src/decoder.c:29; removal: conditional at src/decoder.c:34; evidence: E10

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `src.checksum.table`
- Region: off-heap
- Site: `src/checksum.c:3`
- Writers: `src/checksum.c:12`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: static table built once (bounded, 1 KiB); no growth tied
- Supporting evidence: E5

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.decoder.decode_frame`
- Region: off-heap
- Site: `src/decoder.c:29`
- Removers: `src/decoder.c:34`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: F3 is the census entry for the same allocation as F1 (src/decoder.c:29); its growth is explained and confirmed under F1, no separate test needed
- Supporting evidence: E5

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.frame.decoded_frame.pixels`
- Region: off-heap
- Site: `src/frame.h:39`
- Writers: `src/decoder.c:44`
- Removers: `src/decoder.c:51`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: decoded_frame.pixels is released by frame_release on success; growth is from the checksum-failure path, recorded as F1
- Supporting evidence: E5

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.main.main`
- Region: off-heap
- Site: `src/main.c:33`
- Removers: `src/main.c:60`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: main.c wire buffer is freed at src/main.c:60; no growth tied
- Supporting evidence: E5

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (leak) E11
  - Hypothesis: N-WARM
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E11
  - Originally supported by: E10

## Blind spots and unknowns

### Coverage gaps

- native/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- native/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)

### Runtime blind spots

- native: the allocator shim does not record which entry points ran

### Entry points not exercisable

- native: `src/decoder.c::frame_status_name`: no_harness

### Evidence limitations and unknowns

- E5: partial coverage
- E5: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E5: limitation: writers and removers are textual name matches
- E6: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E6: limitation: workloads were discovered by reading files; none was run
- E7: limitation: harness written; nothing was run (dla harness run measures it)
- E7: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E8: partial coverage (9 of 9 examined)
- E8: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: exercised entry points are not measured for native: the allocator shim sees allocations, not calls
- E9: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: gate thresholds are provisional (not calibrated)
- E10: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: points 1 to 9 come from S1; the extra points from S2
- E11: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E11: limitation: gate and plateau thresholds are provisional (not calibrated)
- E12: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: points 1 to 9 come from S1; the extra points from S2
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: points 1 to 9 come from S1; the extra points from S2
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: partial coverage (0 of 9 examined)
- E16: limitation: valgrind slows the program down; points 1, 5, 9 of 9 ran under it and no series was written (dla harness run without --tool measures the series)
- E17: partial coverage (0 of 9 examined)
- E17: limitation: heaptrack slows the program down; points 1, 5, 9 of 9 ran under it and no series was written (dla harness run without --tool measures the series)
- E18: limitation: figures are the bytes heaptrack counts as leaked: still allocated when the program exits
- E18: limitation: heaptrack_print rounds sizes (3.02M): figures are accurate to about 1%; -F folded files carry exact bytes
- E19: limitation: figures are the bytes heaptrack counts as leaked: still allocated when the program exits
- E19: limitation: heaptrack_print rounds sizes (3.02M): figures are accurate to about 1%; -F folded files carry exact bytes
- E19: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E20: limitation: tested in a scratch copy; the project was not changed
- E20: limitation: gate thresholds are provisional (not calibrated); the 0.2 and 0.5 slope fractions are placeholders
- E20: limitation: exercised is null: the harness records entry points, not lines, so whether the edited line ran is not checked
- E20: limitation: native and Rust harnesses have no holder probes: the column is a site group from the allocator shim or an os column, and an edit elsewhere may not move os columns past the gate
- E20: limitation: the treated series lacks the control column native.live_blocks@src/decoder.c:29: a patch can rename an owner or move a site, so a patch must keep the measured column to be judged
- E20: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: exercised entry points are not measured for native: the allocator shim sees allocations, not calls
- E21: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E21: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E22: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: partial coverage (1 of 3 examined)
- E27: limitation: F1: growth measured as a count; bytes not attributed
- E27: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E27: limitation: threshold is provisional (not calibrated)
- E28: partial coverage (1 of 3 examined)
- E28: limitation: F1: growth evidence is from S1, not S2; not subtracted
- E28: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E28: limitation: threshold is provisional (not calibrated)
- E29: partial coverage (1 of 3 examined)
- E29: limitation: F1: growth evidence is from S1, not S3; not subtracted
- E29: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E29: limitation: threshold is provisional (not calibrated)
- E30: partial coverage (1 of 3 examined)
- E30: limitation: F1: growth evidence is from S1, not S5; not subtracted
- E30: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E30: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo 9f73e3aec980742f1c03b45fbd3ec9eca8187243 | — |
| E2 | run-config | static | dla 0.1.0 | repo 9f73e3aec980742f1c03b45fbd3ec9eca8187243 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 9f73e3aec980742f1c03b45fbd3ec9eca8187243 | probe/probe.json |
| E4 | static-hit | static | dla 0.1.0 | repo 9f73e3aec980742f1c03b45fbd3ec9eca8187243 | sweep/native-dla.json |
| E5 | census-holder | static | dla-census 0.1.0 | repo 9f73e3aec980742f1c03b45fbd3ec9eca8187243 | census/native.json |
| E6 | workload | static | dla 0.1.0 | repo 9f73e3aec980742f1c03b45fbd3ec9eca8187243 | workload/workloads.json |
| E7 | workload | static | dla 0.1.0 | repo 9f73e3aec980742f1c03b45fbd3ec9eca8187243 | harness/H1/harness.json |
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (718218465b47) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (952dc9dcf67e) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (952dc9dcf67e) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (9de921e20951) | alloc/A1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S2.json (83e579495a76) | trend/T3.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (83e579495a76) | trend/T3.json |
| E14 | growth-series | test | dla 0.1.0 | artifact series/S3.json (9de921e20951) | trend/T4.json |
| E15 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (9de921e20951) | trend/T4.json |
| E16 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (046e7a8660a8) | harness/H1/run-valgrind.json |
| E17 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (c24f8e87ffb5) | harness/H1/run-heaptrack.json |
| E18 | alloc-stack | live | heaptrack 0.1.0 | artifact allocs/A1.json (486afcc33efd) | allocs/A1.json |
| E19 | growth-series | live | heaptrack 0.1.0 | artifact series/S4.json (a7cf62b3ec12) | series/S4.json |
| E20 | intervention | test | dla 0.1.0 | artifact series/S6.json (4a42854557ae) | intervene/I1 |
| E21 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (a7cf62b3ec12) | trend/T6.json |
| E22 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (a7cf62b3ec12) | trend/T6.json |
| E23 | growth-series | test | dla 0.1.0 | artifact series/S5.json (bde06cf1f825) | trend/T7.json |
| E24 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (bde06cf1f825) | trend/T7.json |
| E25 | growth-series | test | dla 0.1.0 | artifact series/S6.json (4a42854557ae) | trend/T8.json |
| E26 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (4a42854557ae) | trend/T8.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S1.json (952dc9dcf67e) | gap/residual-S1-E27.json |
| E28 | residual | test | dla 0.1.0 | artifact series/S2.json (83e579495a76) | gap/residual-S2-E28.json |
| E29 | residual | test | dla 0.1.0 | artifact series/S3.json (9de921e20951) | gap/residual-S3-E29.json |
| E30 | residual | test | dla 0.1.0 | artifact series/S5.json (bde06cf1f825) | gap/residual-S5-E30.json |
