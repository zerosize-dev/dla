# DLA report

Run: `20261107-195500-2f46`

Status: COMPLETE

Findings: 10 enumerated, 10 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 10

Residual: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(build_fails)
- Evidence classes: static
- Holder: container-field
- Owner: `src.ring.ring.items`
- Region: off-heap
- Site: `src/ring.c:6`
- Writers: `src/ring.c:17`, `src/ring.c:34`
- Removers: `src/ring.c:28`
- Annotation: heuristic name match over the project's sources
- Reason: build_fails
- Remedy: Install vendor libacmecodec + acme/codec.h, build capture, run harness with --iterations 2000/4000/8000 and trend RSS; read the header for frame ownership (is there an acme_frame_release?); then test F4/F5 and sibling N-HWM/N-ALLOC
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: unresolved(build_fails)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.ring.ring_create`
- Region: off-heap
- Site: `src/ring.c:14`
- Removers: `src/ring.c:19`
- Annotation: heuristic name match over the project's sources
- Reason: build_fails
- Remedy: Install vendor libacmecodec + acme/codec.h, build capture, run harness with --iterations 2000/4000/8000 and trend RSS; read the header for frame ownership (is there an acme_frame_release?); then test F4/F5 and sibling N-HWM/N-ALLOC
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: unresolved(build_fails)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.ring.ring_create`
- Region: off-heap
- Site: `src/ring.c:17`
- Writers: `src/ring.c:34`
- Removers: `src/ring.c:28`
- Annotation: heuristic name match over the project's sources
- Reason: build_fails
- Remedy: Install vendor libacmecodec + acme/codec.h, build capture, run harness with --iterations 2000/4000/8000 and trend RSS; read the header for frame ownership (is there an acme_frame_release?); then test F4/F5 and sibling N-HWM/N-ALLOC
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(build_fails)
- Evidence classes: static
- Holder: container-field
- Owner: `src.ring.ring_put.items`
- Region: off-heap
- Site: `src/ring.c:34`
- Writers: `src/ring.c:17`, `src/ring.c:34`, `src/session.c:12`
- Removers: `src/ring.c:28`
- Annotation: heuristic name match over the project's sources
- Annotation: ring_put overwrites the oldest slot's pointer with no release of the evicted frame; ring stores borrowed pointers (acme_frame from on_frame). If the vendor codec hands frame ownership to the callback, each evicted frame leaks (one per packet once the 32-slot ring is full). Unverified: acme/codec.h and libacmecodec are not on this machine.
- Reason: build_fails
- Remedy: Install vendor libacmecodec + acme/codec.h, build capture, run harness with --iterations 2000/4000/8000 and trend RSS; read the header for frame ownership (is there an acme_frame_release?); then test F4/F5 and sibling N-HWM/N-ALLOC
- Supporting evidence: E6

### F5: **UNCLASSIFIED**

- State: unresolved(build_fails)
- Evidence classes: static
- Holder: closure/continuation
- Owner: `src.session.session_open.on_frame`
- Region: heap
- Site: `src/session.c:33`
- Writers: `src/session.c:12`
- Annotation: on_frame passes the codec-owned frame to sink_push, which keeps the raw pointer past the callback; no copy and no release anywhere in the project. Lead, unverified.
- Reason: build_fails
- Remedy: Install vendor libacmecodec + acme/codec.h, build capture, run harness with --iterations 2000/4000/8000 and trend RSS; read the header for frame ownership (is there an acme_frame_release?); then test F4/F5 and sibling N-HWM/N-ALLOC
- Supporting evidence: E6

### F6: **UNCLASSIFIED**

- State: unresolved(build_fails)
- Evidence classes: static
- Holder: closure/continuation
- Owner: `src.session.session_open.on_status`
- Region: heap
- Site: `src/session.c:34`
- Reason: build_fails
- Remedy: Install vendor libacmecodec + acme/codec.h, build capture, run harness with --iterations 2000/4000/8000 and trend RSS; read the header for frame ownership (is there an acme_frame_release?); then test F4/F5 and sibling N-HWM/N-ALLOC
- Supporting evidence: E6

### F7: **UNCLASSIFIED**

- State: unresolved(build_fails)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.session.session_open`
- Region: off-heap
- Site: `src/session.c:37`
- Removers: `src/session.c:49`
- Annotation: heuristic name match over the project's sources
- Reason: build_fails
- Remedy: Install vendor libacmecodec + acme/codec.h, build capture, run harness with --iterations 2000/4000/8000 and trend RSS; read the header for frame ownership (is there an acme_frame_release?); then test F4/F5 and sibling N-HWM/N-ALLOC
- Supporting evidence: E6

### F8: **UNCLASSIFIED**

- State: unresolved(build_fails)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.session.codec`
- Region: off-heap
- Site: `src/session.h:9`
- Writers: `src/session.c:37`
- Removers: `src/session.c:49`
- Annotation: heuristic name match over the project's sources
- Reason: build_fails
- Remedy: Install vendor libacmecodec + acme/codec.h, build capture, run harness with --iterations 2000/4000/8000 and trend RSS; read the header for frame ownership (is there an acme_frame_release?); then test F4/F5 and sibling N-HWM/N-ALLOC
- Supporting evidence: E6

### F9: **UNCLASSIFIED**

- State: unresolved(build_fails)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.sink.sink_init`
- Region: off-heap
- Site: `src/sink.c:5`
- Removers: `src/sink.c:15`
- Annotation: heuristic name match over the project's sources
- Reason: build_fails
- Remedy: Install vendor libacmecodec + acme/codec.h, build capture, run harness with --iterations 2000/4000/8000 and trend RSS; read the header for frame ownership (is there an acme_frame_release?); then test F4/F5 and sibling N-HWM/N-ALLOC
- Supporting evidence: E6

### F10: **UNCLASSIFIED**

- State: unresolved(build_fails)
- Evidence classes: static
- Holder: container-field
- Owner: `src.sink.sink.replay`
- Region: off-heap
- Site: `src/sink.h:14`
- Writers: `src/sink.c:5`
- Removers: `src/sink.c:15`
- Annotation: heuristic name match over the project's sources
- Reason: build_fails
- Remedy: Install vendor libacmecodec + acme/codec.h, build capture, run harness with --iterations 2000/4000/8000 and trend RSS; read the header for frame ownership (is there an acme_frame_release?); then test F4/F5 and sibling N-HWM/N-ALLOC
- Supporting evidence: E6

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

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | Repository: C capture tool (src/*.c, Makefile) |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | Whole repository; vendor libacmecodec is a closed dependency |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | RSS grows proportionally to packets decoded; GBs in long sessions (user report, no measurements yet) |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | native (C) only |
| `intake.build` | Does the project build locally? | answered | E1 | make; links -lacmecodec |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | capture --iterations N feeds N 2048-byte packets |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4, raw values |
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
