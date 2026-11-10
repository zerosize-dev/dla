# DLA report

Run: `20261110-160749-1848`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 6 enumerated, 6 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 4

Residual: heap 1103.2103896103897 bytes/n unattributed of 1103.2103896103897 total (threshold 275.80259740259737) ABOVE THRESHOLD, see O1; heap 1079.0787878787878 bytes/n unattributed of 1079.0787878787878 total (threshold 269.76969696969695) ABOVE THRESHOLD, see O2; heap 1079.1845730027549 bytes/n unattributed of 1079.1845730027549 total (threshold 269.7961432506887) ABOVE THRESHOLD, see O1

Coverage: native 3520 iterations, 0 of 10 entry points exercised

## Findings

### F1: **L-HANDLE**

- State: confirmed
- Label: L-HANDLE (evidence: E14 growth-series, E13 alloc-stack)
- Growth: `stack:decode_frame` 1950208 bytes/dump (steady, gate pass, E14)
- Evidence classes: context, growth, static
- Holder: native-handle/global-ref
- Region: off-heap
- Shape: steady
- Site: `src/decoder.c:29`
- Writers: `src/decoder.c:29`
- Removers: `src/decoder.c:34`
- Annotation: free only on the payload-error path; checksum-mismatch return at line 40 skips free
- Verified: no
- Sibling hypotheses: F1.n1
- Suggested archetype (unconfirmed): L-OWNERSHIP
- Supporting evidence: E4, E8, E14
- Explanation (observed): decode_frame mallocs buf at src/decoder.c:29 (holder native-handle/global-ref); buf is freed only when the payload is bad, at src/decoder.c:34, so the checksum-mismatch return at line 40 leaks raw_len bytes per damaged frame.
- Explanation structure: adds: src/decoder.c:29; removal: conditional at src/decoder.c:34; evidence: E14

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
- Remedy: Static CRC table; read-only, no growth tied to it
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
- Remedy: Census duplicate of F1 (same site src/decoder.c:29); fixing F1 covers it
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
- Remedy: decoded_frame.pixels field; its growth is the F1 allocation at decoder.c:29
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
- Remedy: main.c wire buffer is a single allocation freed at exit; no growth
- Supporting evidence: E5

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (leak) E11
  - Hypothesis: N-WARM: live bytes at decoder.c:29 level off after warm-up
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
- E12: partial coverage (0 of 9 examined)
- E12: limitation: valgrind slows the program down; points 1, 5, 9 of 9 ran under it and no series was written (dla harness run without --tool measures the series)
- E13: limitation: memcheck scans memory at exit: still-reachable blocks are live at exit, lost blocks have no pointer left; each file is one process
- E13: limitation: the files are ordered as given, and are runs of different lengths, not dumps of one process
- E14: limitation: memcheck scans memory at exit: still-reachable blocks are live at exit, lost blocks have no pointer left; each file is one process
- E14: limitation: the files are ordered as given, and are runs of different lengths, not dumps of one process
- E14: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E15: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E17: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E18: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: partial coverage (1 of 3 examined)
- E19: limitation: F1: growth evidence is from S4, not S1; not subtracted
- E19: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E19: limitation: threshold is provisional (not calibrated)
- E20: partial coverage (1 of 3 examined)
- E20: limitation: F1: growth evidence is from S4, not S3; not subtracted
- E20: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E20: limitation: threshold is provisional (not calibrated)
- E21: partial coverage (1 of 3 examined)
- E21: limitation: F1: growth evidence is from S4, not S2; not subtracted
- E21: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
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
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (f1049f707e4e) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (4a8e32dcd1bd) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (4a8e32dcd1bd) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (23db9143f4dd) | alloc/A1.json |
| E12 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (760b2190a128) | harness/H1/run-valgrind.json |
| E13 | alloc-stack | live | valgrind-memcheck 0.1.0 | artifact allocs/A1.json (9ae3dce67c07) | allocs/A1.json |
| E14 | growth-series | live | valgrind-memcheck 0.1.0 | artifact series/S4.json (06e6500813bf) | series/S4.json |
| E15 | growth-series | test | dla 0.1.0 | artifact series/S2.json (3379cdcc4fc4) | trend/T3.json |
| E16 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (3379cdcc4fc4) | trend/T3.json |
| E17 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (06e6500813bf) | trend/T4.json |
| E18 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (06e6500813bf) | trend/T4.json |
| E19 | residual | test | dla 0.1.0 | artifact series/S1.json (4a8e32dcd1bd) | gap/residual-S1-E19.json |
| E20 | residual | test | dla 0.1.0 | artifact series/S3.json (23db9143f4dd) | gap/residual-S3-E20.json |
| E21 | residual | test | dla 0.1.0 | artifact series/S2.json (3379cdcc4fc4) | gap/residual-S2-E21.json |
