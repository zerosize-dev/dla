# DLA report

Run: `20261109-203742-029c`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 6 enumerated, 6 dispositioned; confirmed 1, strong 0, refuted 1, suspected 4, unresolved 0

Residual: not measured

Coverage: native 3520 iterations, 0 of 10 entry points exercised

## Findings

### F1: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `native.live_blocks@src/decoder.c:29` 0.0625 count/n (steady, gate pass, E10)
- Evidence classes: context, discriminator, growth, static
- Holder: native-handle/global-ref
- Region: heap
- Shape: steady
- Site: `src/decoder.c:29`
- Annotation: decode_frame returns FRAME_ERR_CHECKSUM without free(buf) (src/decoder.c:29 malloc); rle failure path frees it. Damaged frames (every 16th in synth workload; slope 1 block/16 iterations) leak raw_len bytes each. Missing pairing: free of buf on checksum-failure path. UNCLASSIFIED label.
- Sibling hypotheses: F1.n1
- Suggested archetype (unconfirmed): L-OWNERSHIP
- Supporting evidence: E4, E8, E10, E11

### F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: context, static
- Holder: static/global
- Owner: `src.checksum.table`
- Region: off-heap
- Site: `src/checksum.c:3`
- Writers: `src/checksum.c:12`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: No growth tied to this census holder; F2 overlaps or is unrelated to confirmed F1 (decoder.c:29 buf). Add a dedicated probe/driver if it must be ruled out.
- Supporting evidence: E5, E8

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: context, static
- Holder: native-handle/global-ref
- Owner: `src.decoder.decode_frame`
- Region: off-heap
- Site: `src/decoder.c:29`
- Removers: `src/decoder.c:34`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: No growth tied to this census holder; F3 overlaps or is unrelated to confirmed F1 (decoder.c:29 buf). Add a dedicated probe/driver if it must be ruled out.
- Supporting evidence: E5, E8

### F4: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: context, static
- Holder: container-field
- Owner: `src.frame.decoded_frame.pixels`
- Region: off-heap
- Site: `src/frame.h:39`
- Writers: `src/decoder.c:44`
- Removers: `src/decoder.c:51`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: No growth tied to this census holder; F4 overlaps or is unrelated to confirmed F1 (decoder.c:29 buf). Add a dedicated probe/driver if it must be ruled out.
- Supporting evidence: E5, E8

### F5: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: context, static
- Holder: native-handle/global-ref
- Owner: `src.main.main`
- Region: off-heap
- Site: `src/main.c:33`
- Removers: `src/main.c:60`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: No growth tied to this census holder; F5 overlaps or is unrelated to confirmed F1 (decoder.c:29 buf). Add a dedicated probe/driver if it must be ruled out.
- Supporting evidence: E5, E8

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (leak) E11
  - Hypothesis: N-WARM: one-time warm-up allocation
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
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (98c41cadbf16) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (7ee438cd4142) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (7ee438cd4142) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (394309913799) | alloc/A1.json |
