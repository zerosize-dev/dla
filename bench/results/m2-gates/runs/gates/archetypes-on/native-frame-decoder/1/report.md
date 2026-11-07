# DLA report

Run: `20261107-193916-ed0c`

Status: COMPLETE

Findings: 5 enumerated, 5 dispositioned; confirmed 0, strong 0, refuted 0, suspected 3, unresolved 2

Residual: not measured

## Findings

### F2: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: static/global
- Owner: `src.checksum.table`
- Region: off-heap
- Site: `src/checksum.c:3`
- Writers: `src/checksum.c:12`
- Annotation: heuristic name match over the project's sources
- Reason: no_harness
- Remedy: Fixed-size 1KB CRC table, built once; bounded, not a growth source
- Supporting evidence: E5

### F4: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `src.frame.decoded_frame.pixels`
- Region: off-heap
- Site: `src/frame.h:39`
- Writers: `src/decoder.c:44`
- Removers: `src/decoder.c:51`
- Annotation: heuristic name match over the project's sources
- Reason: no_harness
- Remedy: Owned by caller; released via frame_release in main.c on success path; not the leak
- Supporting evidence: E5

### F5: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.main.main`
- Region: off-heap
- Site: `src/main.c:33`
- Removers: `src/main.c:60`
- Annotation: heuristic name match over the project's sources
- Reason: no_harness
- Remedy: Single wire buffer freed at main.c:60; bounded
- Supporting evidence: E5

### F1: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Site: `src/decoder.c:29`
- Writers: `src/decoder.c:29`
- Removers: `src/decoder.c:34`
- Annotation: malloc at :29 freed on rle failure (:34) but NOT on checksum mismatch return at :39 (FRAME_ERR_CHECKSUM); measured: damage-every 16 -> maxrss 13.3MB@2k, 13.6MB@8k, 36.0MB@32k frames; damage-every 0 -> flat 13.5MB at all sizes
- Suggested archetype (unconfirmed): L-OWNERSHIP
- Reason: tool_missing
- Remedy: Installed dla lacks test/confirm/harness/sample/trend/gate; manual RSS measurement supports it. Rerun with full release: dla ledger test F1; confirm.
- Supporting evidence: E4

### F3: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.decoder.decode_frame`
- Region: off-heap
- Site: `src/decoder.c:29`
- Removers: `src/decoder.c:34`
- Annotation: heuristic name match over the project's sources
- Annotation: same allocation as F1; leaked on checksum-mismatch path src/decoder.c:39
- Reason: tool_missing
- Remedy: Duplicate of F1 (same allocation); merge with F1 once confirm exists
- Supporting evidence: E5

## Ruled out

None.

## Blind spots and unknowns

### Coverage gaps

None recorded.

### Runtime blind spots

None recorded.

### Entry points not exercisable

- native: `src/decoder.c::frame_status_name`: no_harness

### Evidence limitations and unknowns

- E5: partial coverage
- E5: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E5: limitation: writers and removers are textual name matches
- E6: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E6: limitation: workloads were discovered by reading files; none was run

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | framedec native C frame decoder |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | src/*.c |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | RSS grows proportional to frames |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | native C |
| `intake.build` | Does the project build locally? | answered | E1 | Makefile |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | framedec main / tests |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 raw |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 60m |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 9f73e3aec980742f1c03b45fbd3ec9eca8187243 | — |
| E2 | run-config | static | dla 0.1.0 | repo 9f73e3aec980742f1c03b45fbd3ec9eca8187243 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 9f73e3aec980742f1c03b45fbd3ec9eca8187243 | probe/probe.json |
| E4 | static-hit | static | dla 0.1.0 | repo 9f73e3aec980742f1c03b45fbd3ec9eca8187243 | sweep/dla.json |
| E5 | census-holder | static | dla-census 0.1.0 | repo 9f73e3aec980742f1c03b45fbd3ec9eca8187243 | census/native.json |
| E6 | workload | static | dla 0.1.0 | repo 9f73e3aec980742f1c03b45fbd3ec9eca8187243 | workload/workloads.json |
