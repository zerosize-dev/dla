# DLA report

Run: `20261108-125236-7c87`

Status: COMPLETE

Findings: 7 enumerated, 7 dispositioned; confirmed 2, strong 0, refuted 2, suspected 0, unresolved 3

Residual: not measured

Coverage: native 3520 iterations, 0 of 10 entry points exercised

## Findings

### F1: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `native.live_blocks@src/decoder.c:29` 0.0625 count/n (steady, gate pass, E11)
- Evidence classes: context, growth, static
- Holder: native-handle/global-ref
- Owner: `src.decoder.decode_frame`
- Region: off-heap
- Shape: steady
- Site: `src/decoder.c:29`
- Writers: `src/decoder.c:29`
- Annotation: malloc buf; checksum-mismatch return at decoder.c:40 skips free (other error path at :34 frees)
- Sibling hypotheses: F1.n1
- Suggested archetype (unconfirmed): L-OWNERSHIP
- Supporting evidence: E5, E9, E11

### F3: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `native.live_blocks@src/decoder.c:29` 0.0625 count/n (steady, gate pass, E11)
- Evidence classes: context, growth, static
- Holder: native-handle/global-ref
- Owner: `src.decoder.decode_frame`
- Region: off-heap
- Shape: steady
- Site: `src/decoder.c:29`
- Writers: `src/decoder.c:29`
- Removers: `src/decoder.c:34`, `src/decoder.c:51`
- Annotation: heuristic name match over the project's sources
- Annotation: freed on payload error and by frame_release; NOT on checksum failure return
- Sibling hypotheses: F3.n1
- Supporting evidence: E6, E9, E11

### F2: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: static
- Holder: static/global
- Owner: `src.checksum.table`
- Region: off-heap
- Site: `src/checksum.c:3`
- Writers: `src/checksum.c:12`
- Annotation: heuristic name match over the project's sources
- Annotation: fixed 256-entry CRC table, bounded, not growth
- Reason: env_unavailable
- Remedy: No per-site growth column at this site; region-level growth is explained by F1 (same workload). Re-test after F1's missing free on the checksum-failure path is fixed; a per-site probe for this holder is needed.
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: static
- Holder: container-field
- Owner: `src.frame.decoded_frame.pixels`
- Region: off-heap
- Site: `src/frame.h:39`
- Writers: `src/decoder.c:44`
- Removers: `src/decoder.c:51`
- Annotation: heuristic name match over the project's sources
- Annotation: caller frees via frame_release in main.c on success; checksum failure never assigns it
- Reason: env_unavailable
- Remedy: No per-site growth column at this site; region-level growth is explained by F1 (same workload). Re-test after F1's missing free on the checksum-failure path is fixed; a per-site probe for this holder is needed.
- Supporting evidence: E6

### F5: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.main.main`
- Region: off-heap
- Site: `src/main.c:33`
- Removers: `src/main.c:60`
- Annotation: heuristic name match over the project's sources
- Annotation: wire buffer allocated once, freed at exit
- Reason: env_unavailable
- Remedy: No per-site growth column at this site; region-level growth is explained by F1 (same workload). Re-test after F1's missing free on the checksum-failure path is fixed; a per-site probe for this holder is needed.
- Supporting evidence: E6

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (leak) E12
  - Hypothesis: N-WARM: growth is warm-up/cache fill and plateaus
  - Refuted if: growth continues linearly over the extra cycles after purge
  - Contradicting evidence: E12
  - Originally supported by: E11
- **F3.n1** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (leak) E13
  - Hypothesis: N-WARM: growth is warm-up and plateaus
  - Refuted if: growth continues over extra cycles
  - Contradicting evidence: E13
  - Originally supported by: E11

## Blind spots and unknowns

### Coverage gaps

None recorded.

### Runtime blind spots

- native: the allocator shim does not record which entry points ran

### Entry points not exercisable

- native: `src/decoder.c::frame_status_name`: no_harness

### Evidence limitations and unknowns

- E6: partial coverage
- E6: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E6: limitation: writers and removers are textual name matches
- E7: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E7: limitation: workloads were discovered by reading files; none was run
- E8: limitation: harness written; nothing was run (dla harness run measures it)
- E8: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E9: partial coverage (9 of 9 examined)
- E9: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: exercised entry points are not measured for native: the allocator shim sees allocations, not calls
- E10: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: points 1 to 9 come from S1; the extra points from S2
- E12: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E12: limitation: gate and plateau thresholds are provisional (not calibrated)
- E13: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: points 1 to 9 come from S1; the extra points from S4
- E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- E14: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: points 1 to 9 come from S1; the extra points from S6
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: points 1 to 9 come from S1; the extra points from S8
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: points 1 to 9 come from S1; the extra points from S10
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | native C frame decoder (framedec) |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | whole project src/ |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | RSS grows proportional to frames processed |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | native C |
| `intake.build` | Does the project build locally? | answered | E1 | make |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | framedec --iterations N |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4, raw values |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 60m |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 9f73e3aec980742f1c03b45fbd3ec9eca8187243 | — |
| E2 | run-config | static | dla 0.1.0 | repo 9f73e3aec980742f1c03b45fbd3ec9eca8187243 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 9f73e3aec980742f1c03b45fbd3ec9eca8187243 | probe/probe.json |
| E4 | probe | static | dla-probe 0.1.0 | repo 9f73e3aec980742f1c03b45fbd3ec9eca8187243 | probe/probe.json |
| E5 | static-hit | static | dla 0.1.0 | repo 9f73e3aec980742f1c03b45fbd3ec9eca8187243 | sweep/dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo 9f73e3aec980742f1c03b45fbd3ec9eca8187243 | census/native.json |
| E7 | workload | static | dla 0.1.0 | repo 9f73e3aec980742f1c03b45fbd3ec9eca8187243 | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo 9f73e3aec980742f1c03b45fbd3ec9eca8187243 | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (df6864ec245f) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (55e9bc742cf5) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (55e9bc742cf5) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (7bfc66c7b1eb) | alloc/A1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S5.json (b2bfd83acddc) | alloc/A2.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S7.json (d59737ce957f) | alloc/A3.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S9.json (79620bb6dca0) | alloc/A4.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S11.json (890955583644) | alloc/A5.json |
