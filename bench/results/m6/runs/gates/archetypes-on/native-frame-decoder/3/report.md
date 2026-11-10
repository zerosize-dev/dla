# DLA report

Run: `20261110-173116-7d6c`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 6 enumerated, 6 dispositioned; confirmed 1, strong 0, refuted 1, suspected 4, unresolved 0

Residual: heap 1103.2103896103897 bytes/n unattributed of 1103.2103896103897 total (threshold 275.80259740259737) ABOVE THRESHOLD, see O1; heap 1079.0787878787878 bytes/n unattributed of 1079.0787878787878 total (threshold 269.76969696969695) ABOVE THRESHOLD, see O2; heap 1079.1845730027549 bytes/n unattributed of 1079.1845730027549 total (threshold 269.7961432506887) ABOVE THRESHOLD, see O2

Coverage: native 3520 iterations, 0 of 10 entry points exercised

## Findings

### F3: **L-OWNERSHIP**

- State: confirmed
- Label: L-OWNERSHIP (evidence: E9 growth-series, E14 alloc-stack)
- Growth: `native.live_blocks@src/decoder.c:29` 0.0625 count/n (steady, gate pass, E10)
- Evidence classes: context, growth, static
- Holder: native-handle/global-ref
- Owner: `src.decoder.decode_frame`
- Region: off-heap
- Shape: steady
- Site: `src/decoder.c:29`
- Writers: `src/decoder.c:29`
- Removers: `src/decoder.c:34`
- Annotation: heuristic name match over the project's sources
- Annotation: free only on RLE error path; checksum-failure return at line 40 leaks buf
- Verified: no
- Sibling hypotheses: F3.n1
- Supporting evidence: E5, E8, E10
- Explanation (inferred): decode_frame mallocs buf at decoder.c:29 (holder: native heap block via the caller-owned pixels pointer); free at decoder.c:34 runs only when the RLE expand fails, so the checksum-failure return at line 40 leaks buf on every damaged frame.
- Explanation structure: adds: src/decoder.c:29; removal: conditional at src/decoder.c:34

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Site: `src/decoder.c:29`
- Suggested archetype (unconfirmed): L-OWNERSHIP
- Reason: not_exercised
- Remedy: Same site and defect as confirmed F3 (decoder.c:29, detector duplicate); no separate action
- Supporting evidence: E4

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
- Remedy: Census holder lead only; no growth tied to it, growth is explained by F3
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
- Remedy: Census holder lead only; no growth tied to it, growth is explained by F3
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
- Remedy: Census holder lead only; no growth tied to it, growth is explained by F3
- Supporting evidence: E5

## Ruled out

- **F3.n1** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (leak) E11
  - Hypothesis: N-WARM: allocation is a one-time warm-up/high-water fill that plateaus
  - Refuted if: growth continues over extra cycles and tracks damaged-frame count
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
- E12: partial coverage (0 of 9 examined)
- E12: limitation: heaptrack slows the program down; points 1, 5, 9 of 9 ran under it and no series was written (dla harness run without --tool measures the series)
- E13: limitation: figures are the bytes heaptrack counts as leaked: still allocated when the program exits
- E13: limitation: heaptrack_print rounds sizes (3.02M): figures are accurate to about 1%; -F folded files carry exact bytes
- E14: limitation: figures are the bytes heaptrack counts as leaked: still allocated when the program exits
- E14: limitation: heaptrack_print rounds sizes (3.02M): figures are accurate to about 1%; -F folded files carry exact bytes
- E15: limitation: figures are the bytes heaptrack counts as leaked: still allocated when the program exits
- E15: limitation: heaptrack_print rounds sizes (3.02M): figures are accurate to about 1%; -F folded files carry exact bytes
- E15: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E16: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E18: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E19: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: partial coverage (1 of 3 examined)
- E20: limitation: F3: growth measured as a count; bytes not attributed
- E20: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E20: limitation: threshold is provisional (not calibrated)
- E21: partial coverage (1 of 3 examined)
- E21: limitation: F3: growth evidence is from S1, not S3; not subtracted
- E21: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E21: limitation: threshold is provisional (not calibrated)
- E22: partial coverage (1 of 3 examined)
- E22: limitation: F3: growth evidence is from S1, not S2; not subtracted
- E22: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E22: limitation: threshold is provisional (not calibrated)

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
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (58fe2fdb0f76) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (a98227b3c297) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (a98227b3c297) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (1e185116c1c8) | alloc/A1.json |
| E12 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (0b5bacce6ebf) | harness/H1/run-heaptrack.json |
| E13 | alloc-stack | live | heaptrack 0.1.0 | artifact allocs/A1.json (1f74bdb8c745) | allocs/A1.json |
| E14 | alloc-stack | live | heaptrack 0.1.0 | artifact allocs/A2.json (d1bd5fbc72b4) | allocs/A2.json |
| E15 | growth-series | live | heaptrack 0.1.0 | artifact series/S4.json (5d397a391634) | series/S4.json |
| E16 | growth-series | test | dla 0.1.0 | artifact series/S2.json (1915e2d997f1) | trend/T3.json |
| E17 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (1915e2d997f1) | trend/T3.json |
| E18 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (5d397a391634) | trend/T4.json |
| E19 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (5d397a391634) | trend/T4.json |
| E20 | residual | test | dla 0.1.0 | artifact series/S1.json (a98227b3c297) | gap/residual-S1-E20.json |
| E21 | residual | test | dla 0.1.0 | artifact series/S3.json (1e185116c1c8) | gap/residual-S3-E21.json |
| E22 | residual | test | dla 0.1.0 | artifact series/S2.json (1915e2d997f1) | gap/residual-S2-E22.json |
