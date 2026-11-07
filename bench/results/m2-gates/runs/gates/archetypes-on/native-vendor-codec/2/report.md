# DLA report

Run: `20261107-194909-5566`

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
- Remedy: Bounded or paired holder per static read; not exercised because capture cannot build without libacmecodec. Rebuild with the vendor library, then sample RSS vs --iterations and compare N-ALLOC/N-WARM siblings.
- Supporting evidence: E5

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
- Remedy: Bounded or paired holder per static read; not exercised because capture cannot build without libacmecodec. Rebuild with the vendor library, then sample RSS vs --iterations and compare N-ALLOC/N-WARM siblings.
- Supporting evidence: E5

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
- Remedy: Bounded or paired holder per static read; not exercised because capture cannot build without libacmecodec. Rebuild with the vendor library, then sample RSS vs --iterations and compare N-ALLOC/N-WARM siblings.
- Supporting evidence: E5

### F4: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.ring.ring_put.items`
- Region: off-heap
- Site: `src/ring.c:34`
- Writers: `src/ring.c:17`, `src/ring.c:34`
- Removers: `src/ring.c:28`
- Annotation: heuristic name match over the project's sources
- Annotation: ring is fixed at 32 slots, overwrites oldest without releasing the pointed-to frame; ring never owns items (ring.h:6-7). Bounded in count; pointee ownership is the open question
- Reason: tool_missing
- Remedy: Suspected static only: sink_push (sink.c:24) keeps frames handed to on_frame in a 32-slot ring; overwritten frames are never released. If libacmecodec hands out frames that the caller must release/ref-count, this would be the unreleased-frame holder. Unverified: vendor lib/header absent, build fails; dla release lacks harness/sample/trend. Install libacmecodec + acme/codec.h, read its frame ownership contract, run capture --iterations N under a sampler at 2+ sizes.
- Supporting evidence: E5

### F5: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: closure/continuation
- Owner: `src.session.session_open.on_frame`
- Region: heap
- Site: `src/session.c:33`
- Writers: `src/session.c:33`
- Annotation: on_frame registered with vendor codec; each frame f is passed to sink_push which stores the borrowed pointer in a 32-slot ring (bounded). Whether the vendor requires a release call per frame is unknown: header acme/codec.h not available
- Reason: tool_missing
- Remedy: Suspected static only: sink_push (sink.c:24) keeps frames handed to on_frame in a 32-slot ring; overwritten frames are never released. If libacmecodec hands out frames that the caller must release/ref-count, this would be the unreleased-frame holder. Unverified: vendor lib/header absent, build fails; dla release lacks harness/sample/trend. Install libacmecodec + acme/codec.h, read its frame ownership contract, run capture --iterations N under a sampler at 2+ sizes.
- Supporting evidence: E5

### F6: **UNCLASSIFIED**

- State: unresolved(build_fails)
- Evidence classes: static
- Holder: closure/continuation
- Owner: `src.session.session_open.on_status`
- Region: heap
- Site: `src/session.c:34`
- Reason: build_fails
- Remedy: Bounded or paired holder per static read; not exercised because capture cannot build without libacmecodec. Rebuild with the vendor library, then sample RSS vs --iterations and compare N-ALLOC/N-WARM siblings.
- Supporting evidence: E5

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
- Remedy: Bounded or paired holder per static read; not exercised because capture cannot build without libacmecodec. Rebuild with the vendor library, then sample RSS vs --iterations and compare N-ALLOC/N-WARM siblings.
- Supporting evidence: E5

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
- Remedy: Bounded or paired holder per static read; not exercised because capture cannot build without libacmecodec. Rebuild with the vendor library, then sample RSS vs --iterations and compare N-ALLOC/N-WARM siblings.
- Supporting evidence: E5

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
- Remedy: Bounded or paired holder per static read; not exercised because capture cannot build without libacmecodec. Rebuild with the vendor library, then sample RSS vs --iterations and compare N-ALLOC/N-WARM siblings.
- Supporting evidence: E5

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
- Remedy: Bounded or paired holder per static read; not exercised because capture cannot build without libacmecodec. Rebuild with the vendor library, then sample RSS vs --iterations and compare N-ALLOC/N-WARM siblings.
- Supporting evidence: E5

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

- E5: partial coverage
- E5: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E5: limitation: writers and removers are textual name matches
- E6: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E6: limitation: workloads were discovered by reading files; none was run

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | capture tool (C, src/*.c), links vendor libacmecodec |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | whole repo: src/ and tests/ |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | RSS grows with packets decoded; OOM-kill in long sessions |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | native (C11) only |
| `intake.build` | Does the project build locally? | answered | E1 | make capture fails: vendor lib/header acme/codec.h absent on this machine |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | capture --iterations N feeds N random 2048B packets; not runnable here |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E1 | runner consent tiers 1-4 raw values (E2) |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 3600s |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo fa1b220e399559e4fca0a35fda04e6278bc9da19 | — |
| E2 | run-config | static | dla 0.1.0 | repo fa1b220e399559e4fca0a35fda04e6278bc9da19 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo fa1b220e399559e4fca0a35fda04e6278bc9da19 | probe/probe.json |
| E4 | static-hit | static | dla 0.1.0 | repo fa1b220e399559e4fca0a35fda04e6278bc9da19 | sweep/dla.json |
| E5 | census-holder | static | dla-census 0.1.0 | repo fa1b220e399559e4fca0a35fda04e6278bc9da19 | census/native.json |
| E6 | workload | static | dla 0.1.0 | repo fa1b220e399559e4fca0a35fda04e6278bc9da19 | workload/workloads.json |
