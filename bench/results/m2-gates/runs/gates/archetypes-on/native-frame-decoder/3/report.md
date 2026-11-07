# DLA report

Run: `20261107-195134-49c1`

Status: COMPLETE

Findings: 5 enumerated, 5 dispositioned; confirmed 0, strong 0, refuted 0, suspected 1, unresolved 4

Residual: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: suspected_static_only(tool_missing)
- Evidence classes: static
- Site: `src/decoder.c:29`
- Writers: `src/decoder.c:29`
- Annotation: checksum mismatch path returns FRAME_ERR_CHECKSUM without free(buf); main.c does 'continue' on rejection and never owns the buffer. valgrind (outside dla): damage-every=16 -> 124 blocks/2.2MB lost at 2000 frames, 1248 blocks/21.9MB at 20000 frames (linear); damage 0 -> zero lost
- Suggested archetype (unconfirmed): L-OWNERSHIP
- Reason: tool_missing
- Remedy: Installed dla 0.1.0 lacks test/confirm/harness/sample/trend/gate. Static hit + manual valgrind (not ledger evidence) show linear loss. Fix class: release-exceptional - free(buf) before returning FRAME_ERR_CHECKSUM in decode_frame (src/decoder.c:40). Rerun with a dla release that has harness+confirm.
- Supporting evidence: E5

### F2: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: static/global
- Owner: `src.checksum.table`
- Region: off-heap
- Site: `src/checksum.c:3`
- Writers: `src/checksum.c:12`
- Annotation: heuristic name match over the project's sources
- Annotation: static CRC table, fixed size, written once; bounded, not growth
- Reason: tool_missing
- Remedy: Reviewed by hand: bounded / properly released apart from F1; no growth series measurable because dla lacks harness/trend commands
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.decoder.decode_frame`
- Region: off-heap
- Site: `src/decoder.c:29`
- Writers: `src/decoder.c:29`
- Removers: `src/decoder.c:34`, `src/decoder.c:51`
- Annotation: heuristic name match over the project's sources
- Annotation: malloc at :29 freed on rle failure (:34) and by caller via frame_release (:51), but NOT on the checksum-mismatch return at :40
- Reason: tool_missing
- Remedy: Reviewed by hand: bounded / properly released apart from F1; no growth series measurable because dla lacks harness/trend commands
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.frame.decoded_frame.pixels`
- Region: off-heap
- Site: `src/frame.h:39`
- Writers: `src/decoder.c:44`
- Removers: `src/decoder.c:51`
- Annotation: heuristic name match over the project's sources
- Annotation: pixels owned by caller; released by frame_release on success path; not the leak
- Reason: tool_missing
- Remedy: Reviewed by hand: bounded / properly released apart from F1; no growth series measurable because dla lacks harness/trend commands
- Supporting evidence: E6

### F5: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.main.main`
- Region: off-heap
- Site: `src/main.c:33`
- Removers: `src/main.c:60`
- Annotation: heuristic name match over the project's sources
- Annotation: wire buffer freed once at exit; bounded
- Reason: tool_missing
- Remedy: Reviewed by hand: bounded / properly released apart from F1; no growth series measurable because dla lacks harness/trend commands
- Supporting evidence: E6

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

- E6: partial coverage
- E6: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E6: limitation: writers and removers are textual name matches
- E7: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E7: limitation: workloads were discovered by reading files; none was run

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | native C frame decoder (framedec) in project |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | src/*.c, whole project |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | RSS grows proportional to frames; restart every few days |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | native C (Makefile) |
| `intake.build` | Does the project build locally? | answered | E1 | make; cc |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | framedec --iterations N synthetic source with damage-every |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 raw values |
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
