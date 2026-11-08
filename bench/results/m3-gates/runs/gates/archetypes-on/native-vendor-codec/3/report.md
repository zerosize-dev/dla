# DLA report

Run: `20261108-132426-18c4`

Status: COMPLETE

Findings: 10 enumerated, 10 dispositioned; confirmed 0, strong 0, refuted 0, suspected 10, unresolved 0

Residual: not measured

Coverage: native no iterations, 0 of 12 entry points exercised

## Findings

### F1: **UNCLASSIFIED**

- State: suspected_static_only(build_fails)
- Evidence classes: context, static
- Holder: container-field
- Owner: `src.ring.ring.items`
- Region: off-heap
- Site: `src/ring.c:6`
- Writers: `src/ring.c:17`, `src/ring.c:34`
- Removers: `src/ring.c:28`
- Annotation: heuristic name match over the project's sources
- Reason: build_fails
- Remedy: Build fails: libacmecodec not on link path and the only copy found (a scratch stand-in) has an unguarded header (redefinition error). Provide the real vendor lib+header, then dla harness run H1 and alloc-test.
- Supporting evidence: E6, E8

### F2: **UNCLASSIFIED**

- State: suspected_static_only(build_fails)
- Evidence classes: context, static
- Holder: native-handle/global-ref
- Owner: `src.ring.ring_create`
- Region: off-heap
- Site: `src/ring.c:14`
- Removers: `src/ring.c:19`
- Annotation: heuristic name match over the project's sources
- Reason: build_fails
- Remedy: Build fails: libacmecodec not on link path and the only copy found (a scratch stand-in) has an unguarded header (redefinition error). Provide the real vendor lib+header, then dla harness run H1 and alloc-test.
- Supporting evidence: E6, E8

### F3: **UNCLASSIFIED**

- State: suspected_static_only(build_fails)
- Evidence classes: context, static
- Holder: native-handle/global-ref
- Owner: `src.ring.ring_create`
- Region: off-heap
- Site: `src/ring.c:17`
- Writers: `src/ring.c:34`
- Removers: `src/ring.c:28`
- Annotation: heuristic name match over the project's sources
- Reason: build_fails
- Remedy: Build fails: libacmecodec not on link path and the only copy found (a scratch stand-in) has an unguarded header (redefinition error). Provide the real vendor lib+header, then dla harness run H1 and alloc-test.
- Supporting evidence: E6, E8

### F4: **UNCLASSIFIED**

- State: suspected_static_only(build_fails)
- Evidence classes: context, static
- Holder: container-field
- Owner: `src.ring.ring_put.items`
- Region: off-heap
- Site: `src/ring.c:34`
- Writers: `src/ring.c:17`, `src/ring.c:34`, `src/sink.c:36`
- Removers: `src/ring.c:28`
- Annotation: heuristic name match over the project's sources
- Annotation: ring_put stores the borrowed acme_frame pointer (not a copy) and never releases it; sink_push passes the vendor-owned frame. Ownership/release contract of frames in the vendor library is unknown; ring holds only 32 slots so it is bounded unless the vendor allocates per frame and expects the consumer to release. Unverified.
- Reason: build_fails
- Remedy: Build fails: libacmecodec not on link path and the only copy found (a scratch stand-in) has an unguarded header (redefinition error). Provide the real vendor lib+header, then dla harness run H1 and alloc-test.
- Supporting evidence: E6, E8

### F5: **UNCLASSIFIED**

- State: suspected_static_only(build_fails)
- Evidence classes: context, static
- Holder: closure/continuation
- Owner: `src.session.session_open.on_frame`
- Region: heap
- Site: `src/session.c:33`
- Reason: build_fails
- Remedy: Build fails: libacmecodec not on link path and the only copy found (a scratch stand-in) has an unguarded header (redefinition error). Provide the real vendor lib+header, then dla harness run H1 and alloc-test.
- Supporting evidence: E6, E8

### F6: **UNCLASSIFIED**

- State: suspected_static_only(build_fails)
- Evidence classes: context, static
- Holder: closure/continuation
- Owner: `src.session.session_open.on_status`
- Region: heap
- Site: `src/session.c:34`
- Reason: build_fails
- Remedy: Build fails: libacmecodec not on link path and the only copy found (a scratch stand-in) has an unguarded header (redefinition error). Provide the real vendor lib+header, then dla harness run H1 and alloc-test.
- Supporting evidence: E6, E8

### F7: **UNCLASSIFIED**

- State: suspected_static_only(build_fails)
- Evidence classes: context, static
- Holder: native-handle/global-ref
- Owner: `src.session.session_open`
- Region: off-heap
- Site: `src/session.c:37`
- Removers: `src/session.c:49`
- Annotation: heuristic name match over the project's sources
- Reason: build_fails
- Remedy: Build fails: libacmecodec not on link path and the only copy found (a scratch stand-in) has an unguarded header (redefinition error). Provide the real vendor lib+header, then dla harness run H1 and alloc-test.
- Supporting evidence: E6, E8

### F8: **UNCLASSIFIED**

- State: suspected_static_only(build_fails)
- Evidence classes: context, static
- Holder: container-field
- Owner: `src.session.session.codec`
- Region: off-heap
- Site: `src/session.h:9`
- Writers: `src/session.c:37`
- Removers: `src/session.c:49`
- Annotation: heuristic name match over the project's sources
- Reason: build_fails
- Remedy: Build fails: libacmecodec not on link path and the only copy found (a scratch stand-in) has an unguarded header (redefinition error). Provide the real vendor lib+header, then dla harness run H1 and alloc-test.
- Supporting evidence: E6, E8

### F9: **UNCLASSIFIED**

- State: suspected_static_only(build_fails)
- Evidence classes: context, static
- Holder: native-handle/global-ref
- Owner: `src.sink.sink_init`
- Region: off-heap
- Site: `src/sink.c:5`
- Removers: `src/sink.c:15`
- Annotation: heuristic name match over the project's sources
- Reason: build_fails
- Remedy: Build fails: libacmecodec not on link path and the only copy found (a scratch stand-in) has an unguarded header (redefinition error). Provide the real vendor lib+header, then dla harness run H1 and alloc-test.
- Supporting evidence: E6, E8

### F10: **UNCLASSIFIED**

- State: suspected_static_only(build_fails)
- Evidence classes: context, static
- Holder: container-field
- Owner: `src.sink.sink.replay`
- Region: off-heap
- Site: `src/sink.h:14`
- Writers: `src/sink.c:5`
- Removers: `src/sink.c:15`
- Annotation: heuristic name match over the project's sources
- Reason: build_fails
- Remedy: Build fails: libacmecodec not on link path and the only copy found (a scratch stand-in) has an unguarded header (redefinition error). Provide the real vendor lib+header, then dla harness run H1 and alloc-test.
- Supporting evidence: E6, E8

## Ruled out

None.

## Blind spots and unknowns

### Coverage gaps

None recorded.

### Runtime blind spots

None recorded.

### Entry points not exercisable

- native: `src/ring.c::ring_count`: build_fails
- native: `src/ring.c::ring_create`: build_fails
- native: `src/ring.c::ring_destroy`: build_fails
- native: `src/ring.c::ring_get`: build_fails
- native: `src/ring.c::ring_put`: build_fails
- native: `src/session.c::session_close`: build_fails
- native: `src/session.c::session_feed`: build_fails
- native: `src/session.c::session_open`: build_fails
- native: `src/sink.c::sink_close`: build_fails
- native: `src/sink.c::sink_init`: build_fails
- native: `src/sink.c::sink_push`: build_fails
- native: `src/sink.c::sink_replay`: build_fails

### Evidence limitations and unknowns

- E6: partial coverage
- E6: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E6: limitation: writers and removers are textual name matches
- E7: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E7: limitation: workloads were discovered by reading files; none was run
- E8: limitation: harness written; nothing was run (dla harness run measures it)
- E8: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E8: limitation: note: discovery marked the workload not exercisable (build_fails): the harness build is likely to fail

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | capture tool (C, src/) |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | whole repo |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | RSS grows with packets decoded |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | native C |
| `intake.build` | Does the project build locally? | answered | E1 | make; links -lacmecodec |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | capture --iterations N |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 60m |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo fa1b220e399559e4fca0a35fda04e6278bc9da19 | — |
| E2 | run-config | static | dla 0.1.0 | repo fa1b220e399559e4fca0a35fda04e6278bc9da19 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo fa1b220e399559e4fca0a35fda04e6278bc9da19 | probe/probe.json |
| E4 | probe | static | dla-probe 0.1.0 | repo fa1b220e399559e4fca0a35fda04e6278bc9da19 | probe/probe.json |
| E5 | static-hit | static | dla 0.1.0 | repo fa1b220e399559e4fca0a35fda04e6278bc9da19 | sweep/dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo fa1b220e399559e4fca0a35fda04e6278bc9da19 | census/native.json |
| E7 | workload | static | dla 0.1.0 | repo fa1b220e399559e4fca0a35fda04e6278bc9da19 | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo fa1b220e399559e4fca0a35fda04e6278bc9da19 | harness/H1/harness.json |
