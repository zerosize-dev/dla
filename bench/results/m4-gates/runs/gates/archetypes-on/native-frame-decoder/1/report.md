# DLA report

Run: `20261108-192926-2576`

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
- Evidence classes: context, growth, static
- Holder: native-handle/global-ref
- Owner: `decoder.c:decode_frame.buf`
- Region: heap
- Shape: steady
- Site: `src/decoder.c:29`
- Annotation: Allocation at src/decoder.c:29 (malloc(raw_len)). decode_frame frees buf on rle_expand failure but returns FRAME_ERR_CHECKSUM at src/decoder.c:41 without free(buf); main.c skips frame_release on non-OK status, so each damaged frame (1 in damage_every) leaks raw_len bytes. Missing pairing: release on the checksum-mismatch return. Repair class: free on every error return. No taxonomy label commands available; left UNCLASSIFIED.
- Sibling hypotheses: F1.n1
- Suggested archetype (unconfirmed): L-OWNERSHIP
- Supporting evidence: E4, E8, E10

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
- Remedy: Static CRC table in checksum.c is fixed-size, built once; no growth column tied to it. Optional: confirm with a table-size probe.
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
- Remedy: Same allocation site as confirmed F1 (decoder.c:29); census duplicate. Fix F1 and re-run harness to confirm no residual growth.
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
- Remedy: pixels is released by frame_release on the OK path; leak is on the error path (F1). Re-run harness after fix.
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
- Remedy: wire buffer in main.c is freed at line 60 and is a single allocation; no growth tied to it.
- Supporting evidence: E5

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (leak) E11
  - Hypothesis: N-WARM: growth is a warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E11
  - Originally supported by: E10

## Blind spots and unknowns

### Coverage gaps

None recorded.

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
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (3a847b90b85a) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (e7ba06d56efa) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (e7ba06d56efa) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (d6b94d32f356) | alloc/A1.json |
