# DLA report

Run: `20261110-203138-eeaa`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 10 enumerated, 10 dispositioned; confirmed 0, strong 0, refuted 0, suspected 10, unresolved 0

Residual: not measured

Coverage: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: suspected_static_only(env_unavailable)
- Evidence classes: static
- Holder: container-field
- Owner: `src.ring.ring.items`
- Region: off-heap
- Site: `src/ring.c:6`
- Writers: `src/ring.c:17`, `src/ring.c:34`
- Removers: `src/ring.c:28`
- Annotation: heuristic name match over the project's sources
- Reason: env_unavailable
- Remedy: Not exercised: libacmecodec and acme/codec.h are absent, so capture cannot be built or run. Bounded fixed-size container or paired create/destroy per static census; see F4 for the suspected unreleased-frame path.
- Supporting evidence: E4

### F2: **UNCLASSIFIED**

- State: suspected_static_only(env_unavailable)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.ring.ring_create`
- Region: off-heap
- Site: `src/ring.c:14`
- Removers: `src/ring.c:19`
- Annotation: heuristic name match over the project's sources
- Reason: env_unavailable
- Remedy: Not exercised: libacmecodec and acme/codec.h are absent, so capture cannot be built or run. Bounded fixed-size container or paired create/destroy per static census; see F4 for the suspected unreleased-frame path.
- Supporting evidence: E4

### F3: **UNCLASSIFIED**

- State: suspected_static_only(env_unavailable)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.ring.ring_create`
- Region: off-heap
- Site: `src/ring.c:17`
- Writers: `src/ring.c:34`
- Removers: `src/ring.c:28`
- Annotation: heuristic name match over the project's sources
- Reason: env_unavailable
- Remedy: Not exercised: libacmecodec and acme/codec.h are absent, so capture cannot be built or run. Bounded fixed-size container or paired create/destroy per static census; see F4 for the suspected unreleased-frame path.
- Supporting evidence: E4

### F4: **UNCLASSIFIED**

- State: suspected_static_only(env_unavailable)
- Evidence classes: static
- Holder: container-field
- Owner: `src.ring.ring_put.items`
- Region: off-heap
- Site: `src/ring.c:34`
- Writers: `src/ring.c:17`
- Removers: `src/ring.c:28`
- Annotation: heuristic name match over the project's sources
- Reason: env_unavailable
- Remedy: Static suspect: ring_put (src/ring.c:34) overwrites the oldest slot without releasing the evicted frame pointer; sink_push (src/sink.c:29) stores the decoder's acme_frame pointer in the ring. If libacmecodec allocates a frame per on_frame callback (caller-owned), every evicted frame leaks, growing with packets. If frames are decoder-owned/borrowed, the ring holds dangling pointers instead. Build and run with libacmecodec (valgrind alloc-stack-diff) and check the vendor header for acme_frame ownership/release API.
- Supporting evidence: E4

### F5: **UNCLASSIFIED**

- State: suspected_static_only(env_unavailable)
- Evidence classes: static
- Holder: closure/continuation
- Owner: `src.session.session_open.on_frame`
- Region: heap
- Site: `src/session.c:33`
- Reason: env_unavailable
- Remedy: Not exercised: libacmecodec and acme/codec.h are absent, so capture cannot be built or run. Bounded fixed-size container or paired create/destroy per static census; see F4 for the suspected unreleased-frame path.
- Supporting evidence: E4

### F6: **UNCLASSIFIED**

- State: suspected_static_only(env_unavailable)
- Evidence classes: static
- Holder: closure/continuation
- Owner: `src.session.session_open.on_status`
- Region: heap
- Site: `src/session.c:34`
- Reason: env_unavailable
- Remedy: Not exercised: libacmecodec and acme/codec.h are absent, so capture cannot be built or run. Bounded fixed-size container or paired create/destroy per static census; see F4 for the suspected unreleased-frame path.
- Supporting evidence: E4

### F7: **UNCLASSIFIED**

- State: suspected_static_only(env_unavailable)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.session.session_open`
- Region: off-heap
- Site: `src/session.c:37`
- Removers: `src/session.c:49`
- Annotation: heuristic name match over the project's sources
- Reason: env_unavailable
- Remedy: Not exercised: libacmecodec and acme/codec.h are absent, so capture cannot be built or run. Bounded fixed-size container or paired create/destroy per static census; see F4 for the suspected unreleased-frame path.
- Supporting evidence: E4

### F8: **UNCLASSIFIED**

- State: suspected_static_only(env_unavailable)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.session.codec`
- Region: off-heap
- Site: `src/session.h:9`
- Writers: `src/session.c:37`
- Removers: `src/session.c:49`
- Annotation: heuristic name match over the project's sources
- Reason: env_unavailable
- Remedy: Not exercised: libacmecodec and acme/codec.h are absent, so capture cannot be built or run. Bounded fixed-size container or paired create/destroy per static census; see F4 for the suspected unreleased-frame path.
- Supporting evidence: E4

### F9: **UNCLASSIFIED**

- State: suspected_static_only(env_unavailable)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.sink.sink_init`
- Region: off-heap
- Site: `src/sink.c:5`
- Removers: `src/sink.c:15`
- Annotation: heuristic name match over the project's sources
- Reason: env_unavailable
- Remedy: Not exercised: libacmecodec and acme/codec.h are absent, so capture cannot be built or run. Bounded fixed-size container or paired create/destroy per static census; see F4 for the suspected unreleased-frame path.
- Supporting evidence: E4

### F10: **UNCLASSIFIED**

- State: suspected_static_only(env_unavailable)
- Evidence classes: static
- Holder: container-field
- Owner: `src.sink.sink.replay`
- Region: off-heap
- Site: `src/sink.h:14`
- Writers: `src/sink.c:5`
- Removers: `src/sink.c:15`
- Annotation: heuristic name match over the project's sources
- Reason: env_unavailable
- Remedy: Not exercised: libacmecodec and acme/codec.h are absent, so capture cannot be built or run. Bounded fixed-size container or paired create/destroy per static census; see F4 for the suspected unreleased-frame path.
- Supporting evidence: E4

## Ruled out

None.

## Blind spots and unknowns

### Coverage gaps

- native/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- native/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)

### Runtime blind spots

None recorded.

### Entry points not exercisable

None recorded.

### Evidence limitations and unknowns

- E4: partial coverage
- E4: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E4: limitation: writers and removers are textual name matches

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | — |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | — |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | — |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E3 | — |
| `intake.build` | Does the project build locally? | unresolved | — | build_fails: Install libacmecodec and acme/codec.h (vendor decoder) so 'make' can link capture |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | — |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo fa1b220e399559e4fca0a35fda04e6278bc9da19 | — |
| E2 | run-config | static | dla 0.1.0 | repo fa1b220e399559e4fca0a35fda04e6278bc9da19 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo fa1b220e399559e4fca0a35fda04e6278bc9da19 | probe/probe.json |
| E4 | census-holder | static | dla-census 0.1.0 | repo fa1b220e399559e4fca0a35fda04e6278bc9da19 | census/native.json |
| E5 | static-hit | static | dla 0.1.0 | repo fa1b220e399559e4fca0a35fda04e6278bc9da19 | sweep/native-dla.json |
