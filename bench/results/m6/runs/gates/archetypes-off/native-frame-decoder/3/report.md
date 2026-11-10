# DLA report

Run: `20261110-192018-0ca0`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 6 enumerated, 6 dispositioned; confirmed 1, strong 0, refuted 1, suspected 4, unresolved 0

Residual: heap 1103.2103896103897 bytes/n unattributed of 1103.2103896103897 total (threshold 275.80259740259737) ABOVE THRESHOLD, see O1; heap 1079.0787878787878 bytes/n unattributed of 1079.0787878787878 total (threshold 269.76969696969695) ABOVE THRESHOLD, see O1; heap 1079.1845730027549 bytes/n unattributed of 1079.1845730027549 total (threshold 269.7961432506887) ABOVE THRESHOLD, see O1; heap 1103.2103896103897 bytes/n unattributed of 1103.2103896103897 total (threshold 275.80259740259737) ABOVE THRESHOLD, see O2

Coverage: native 3520 iterations, 0 of 10 entry points exercised

## Findings

### F1: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (malloc'd pixel buffer in decode_frame (src/decoder.c:29) is not freed on the FRAME_ERR_CHECKSUM early return (:39); every damaged frame leaks raw_len bytes)
- Growth: `native.live_blocks@src/decoder.c:29` 0.0625 count/n (steady, gate pass, E10)
- Evidence classes: context, discriminator, growth, static
- Holder: native-handle/global-ref
- Region: heap
- Shape: steady
- Site: `src/decoder.c:29`
- Writers: `src/decoder.c:29`
- Removers: `src/decoder.c:34`
- Annotation: buf malloc'd at :29 is freed on rle failure (:34) and via frame_release, but not on the checksum-mismatch return (:39)
- Verified: no
- Sibling hypotheses: F1.n1
- Supporting evidence: E4, E8, E10, E11
- Explanation (inferred): The native-handle/global-ref holder buf in decode_frame (src/decoder.c:29) is malloc'd and freed only when rle_expand fails (src/decoder.c:34) or the caller releases it; the checksum mismatch return leaks it.
- Explanation structure: adds: src/decoder.c:29; removal: conditional at src/decoder.c:34

### F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `src.checksum.table`
- Region: off-heap
- Site: `src/checksum.c:3`
- Writers: `src/checksum.c:12`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Census name-match lead only; no tied growth column, release path reviewed by reading: not the growth source
- Supporting evidence: E5

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: growth, static
- Holder: native-handle/global-ref
- Owner: `src.decoder.decode_frame`
- Region: off-heap
- Site: `src/decoder.c:29`
- Removers: `src/decoder.c:34`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Census duplicate of F1 (same malloc at src/decoder.c:29); tracked and confirmed as F1
- Supporting evidence: E5, E10

### F4: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.frame.decoded_frame.pixels`
- Region: off-heap
- Site: `src/frame.h:39`
- Writers: `src/decoder.c:44`
- Removers: `src/decoder.c:51`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Census name-match lead only; no tied growth column, release path reviewed by reading: not the growth source
- Supporting evidence: E5

### F5: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.main.main`
- Region: off-heap
- Site: `src/main.c:33`
- Removers: `src/main.c:60`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Census name-match lead only; no tied growth column, release path reviewed by reading: not the growth source
- Supporting evidence: E5

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (leak) E11
  - Hypothesis: N-WARM: warm-up cache fill
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
- E14: limitation: tested in a scratch copy; the project was not changed
- E14: limitation: gate thresholds are provisional (not calibrated); the 0.2 and 0.5 slope fractions are placeholders
- E14: limitation: exercised is null: the harness records entry points, not lines, so whether the edited line ran is not checked
- E14: limitation: native and Rust harnesses have no holder probes: the column is a site group from the allocator shim or an os column, and an edit elsewhere may not move os columns past the gate
- E14: limitation: the treated series lacks the control column native.live_blocks@src/decoder.c:29: a patch can rename an owner or move a site, so a patch must keep the measured column to be judged
- E14: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: exercised entry points are not measured for native: the allocator shim sees allocations, not calls
- E15: limitation: tested in a scratch copy; the project was not changed
- E15: limitation: gate thresholds are provisional (not calibrated); the 0.2 and 0.5 slope fractions are placeholders
- E15: limitation: exercised is null: the harness records entry points, not lines, so whether the edited line ran is not checked
- E15: limitation: native and Rust harnesses have no holder probes: the column is a site group from the allocator shim or an os column, and an edit elsewhere may not move os columns past the gate
- E15: limitation: the treated series lacks the control column native.live_blocks@src/decoder.c:29: a patch can rename an owner or move a site, so a patch must keep the measured column to be judged
- E15: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: exercised entry points are not measured for native: the allocator shim sees allocations, not calls
- E16: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: partial coverage (1 of 3 examined)
- E18: limitation: F1: growth measured as a count; bytes not attributed
- E18: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E18: limitation: threshold is provisional (not calibrated)
- E19: partial coverage (1 of 3 examined)
- E19: limitation: F1: growth evidence is from S1, not S2; not subtracted
- E19: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E19: limitation: threshold is provisional (not calibrated)
- E20: partial coverage (1 of 3 examined)
- E20: limitation: F1: growth evidence is from S1, not S3; not subtracted
- E20: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E20: limitation: threshold is provisional (not calibrated)
- E21: partial coverage (1 of 3 examined)
- E21: limitation: F1: growth evidence is from S1, not S5; not subtracted
- E21: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E21: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo 9f73e3aec980742f1c03b45fbd3ec9eca8187243 | — |
| E2 | run-config | static | dla 0.1.0 | repo 9f73e3aec980742f1c03b45fbd3ec9eca8187243 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 9f73e3aec980742f1c03b45fbd3ec9eca8187243 | probe/probe.json |
| E4 | static-hit | static | dla 0.1.0 | repo 9f73e3aec980742f1c03b45fbd3ec9eca8187243 | sweep/native-dla.json |
| E5 | census-holder | static | dla-census 0.1.0 | repo 9f73e3aec980742f1c03b45fbd3ec9eca8187243 | census/native.json |
| E6 | workload | static | dla 0.1.0 | repo 9f73e3aec980742f1c03b45fbd3ec9eca8187243 | workload/workloads.json |
| E7 | workload | static | dla 0.1.0 | repo 9f73e3aec980742f1c03b45fbd3ec9eca8187243 | harness/H1/harness.json |
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (62fda7b0fb0b) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (12faa842e02e) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (12faa842e02e) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (652a47af103d) | alloc/A1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S2.json (c7ecedf4fcfe) | trend/T3.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (c7ecedf4fcfe) | trend/T3.json |
| E14 | intervention | test | dla 0.1.0 | artifact series/S4.json (3fb27bdb5ea1) | intervene/I1 |
| E15 | intervention | test | dla 0.1.0 | artifact series/S6.json (5854900bc93d) | intervene/I2 |
| E16 | growth-series | test | dla 0.1.0 | artifact series/S5.json (e5731aac548a) | trend/T6.json |
| E17 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (e5731aac548a) | trend/T6.json |
| E18 | residual | test | dla 0.1.0 | artifact series/S1.json (12faa842e02e) | gap/residual-S1-E18.json |
| E19 | residual | test | dla 0.1.0 | artifact series/S2.json (c7ecedf4fcfe) | gap/residual-S2-E19.json |
| E20 | residual | test | dla 0.1.0 | artifact series/S3.json (652a47af103d) | gap/residual-S3-E20.json |
| E21 | residual | test | dla 0.1.0 | artifact series/S5.json (e5731aac548a) | gap/residual-S5-E21.json |
