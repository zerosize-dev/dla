# DLA report

Run: `20261110-180919-9604`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 6 enumerated, 6 dispositioned; confirmed 1, strong 0, refuted 1, suspected 4, unresolved 0

Residual: heap 1103.2103896103897 bytes/n unattributed of 1103.2103896103897 total (threshold 275.80259740259737) ABOVE THRESHOLD, see O1; heap 1079.0787878787878 bytes/n unattributed of 1079.0787878787878 total (threshold 269.76969696969695) ABOVE THRESHOLD, see O1; heap 1079.1845730027549 bytes/n unattributed of 1079.1845730027549 total (threshold 269.7961432506887) ABOVE THRESHOLD, see O1; off-heap 146.61818181818182 bytes/n unattributed of 146.61818181818182 total (threshold 36.654545454545456) ABOVE THRESHOLD, see O2; off-heap 65.86181818181818 bytes/n unattributed of 65.86181818181818 total (threshold 16.465454545454545) ABOVE THRESHOLD, see O2; off-heap 100.17851239669422 bytes/n unattributed of 100.17851239669422 total (threshold 25.04462809917355) ABOVE THRESHOLD, see O2

Coverage: native 3520 iterations, 0 of 10 entry points exercised

## Findings

### F1: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (decode_frame mallocs the pixel buffer at decoder.c:29 and returns FRAME_ERR_CHECKSUM without free(buf); every damaged frame leaks raw_len bytes)
- Growth: `stack:decode_frame` 1950208 bytes/dump (steady, gate pass, E13)
- Evidence classes: context, growth, static
- Holder: native-handle/global-ref
- Region: off-heap
- Shape: steady
- Site: `src/decoder.c:29`
- Writers: `src/decoder.c:29`
- Annotation: malloc'd pixel buffer leaked on checksum-mismatch return path
- Annotation: checksum-mismatch return at decoder.c:46 skips free(buf); other error path frees
- Verified: no
- Sibling hypotheses: F1.n1
- Supporting evidence: E4, E9, E13, E11
- Explanation (inferred): decode_frame in src/decoder.c:29 mallocs buf (native-handle/global-ref holder); the FRAME_ERR_CHECKSUM return has no remover found, never frees it, so each damaged frame leaks raw_len bytes.
- Explanation structure: adds: src/decoder.c:29; removal: not-found

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
- Remedy: No growth tied to this census holder; static lead only (checksum table is static; pixels owned by caller and freed in main)
- Supporting evidence: E5

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.decoder.decode_frame`
- Region: off-heap
- Site: `src/decoder.c:29`
- Removers: `src/decoder.c:34`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Census duplicate of F1 (same site decoder.c:29); covered by F1
- Supporting evidence: E5

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
- Remedy: No growth tied to this census holder; static lead only (checksum table is static; pixels owned by caller and freed in main)
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
- Remedy: No growth tied to this census holder; static lead only (checksum table is static; pixels owned by caller and freed in main)
- Supporting evidence: E5

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (leak) E16
  - Hypothesis: N-WARM: allocator/warm-up retention plateaus
  - Refuted if: growth continues over the extra cycles and after purge
  - Contradicting evidence: E16
  - Originally supported by: E11

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
- E8: partial coverage (0 of 9 examined)
- E8: limitation: valgrind slows the program down; points 1, 5, 9 of 9 ran under it and no series was written (dla harness run without --tool measures the series)
- E9: partial coverage (9 of 9 examined)
- E9: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: exercised entry points are not measured for native: the allocator shim sees allocations, not calls
- E10: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: memcheck scans memory at exit: still-reachable blocks are live at exit, lost blocks have no pointer left; each file is one process
- E12: limitation: the files are ordered as given, and are runs of different lengths, not dumps of one process
- E13: limitation: memcheck scans memory at exit: still-reachable blocks are live at exit, lost blocks have no pointer left; each file is one process
- E13: limitation: the files are ordered as given, and are runs of different lengths, not dumps of one process
- E13: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E14: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E14: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E15: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: points 1 to 9 come from S1; the extra points from S3
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: points 1 to 9 come from S1; the extra points from S3
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: points 1 to 9 come from S1; the extra points from S3
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: partial coverage (1 of 3 examined)
- E21: limitation: F1: growth evidence is from S2, not S3; not subtracted
- E21: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E21: limitation: threshold is provisional (not calibrated)
- E22: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: partial coverage (2 of 3 examined)
- E23: limitation: F1: growth measured as a count; bytes not attributed
- E23: limitation: threshold is provisional (not calibrated)
- E24: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: partial coverage (2 of 3 examined)
- E25: limitation: F1: growth evidence is from S2, not S3; not subtracted
- E25: limitation: threshold is provisional (not calibrated)
- E26: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E26: limitation: points 1 to 9 come from S1; the extra points from S3
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: partial coverage (2 of 3 examined)
- E27: limitation: F1: growth evidence is from S2, not S4; not subtracted
- E27: limitation: threshold is provisional (not calibrated)

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
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (5792d589c8e8) | harness/H1/run-valgrind.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (5792d589c8e8) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (9c2814ac4357) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (9c2814ac4357) | trend/T1.json |
| E12 | alloc-stack | live | valgrind-memcheck 0.1.0 | artifact allocs/A1.json (d5b4007c6777) | allocs/A1.json |
| E13 | growth-series | live | valgrind-memcheck 0.1.0 | artifact series/S2.json (733bf629805d) | series/S2.json |
| E14 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (733bf629805d) | trend/T2.json |
| E15 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (733bf629805d) | trend/T2.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S4.json (811bef396bcf) | alloc/A1.json |
| E17 | growth-series | test | dla 0.1.0 | artifact series/S3.json (11f9276bcea0) | trend/T4.json |
| E18 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (11f9276bcea0) | trend/T4.json |
| E19 | growth-series | test | dla 0.1.0 | artifact series/S4.json (811bef396bcf) | trend/T5.json |
| E20 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (811bef396bcf) | trend/T5.json |
| E21 | residual | test | dla 0.1.0 | artifact series/S3.json (11f9276bcea0) | gap/residual-S3-E21.json |
| E22 | non-heap | test | dla 0.1.0 | artifact series/S5.json (b11caf469f6c) | trend/T6.json |
| E23 | residual | test | dla 0.1.0 | artifact series/S1.json (9c2814ac4357) | gap/residual-S1-E23.json |
| E24 | non-heap | test | dla 0.1.0 | artifact series/S6.json (d15e74769ff2) | trend/T7.json |
| E25 | residual | test | dla 0.1.0 | artifact series/S3.json (11f9276bcea0) | gap/residual-S3-E25.json |
| E26 | non-heap | test | dla 0.1.0 | artifact series/S7.json (f54aebb529c9) | trend/T8.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S4.json (811bef396bcf) | gap/residual-S4-E27.json |
