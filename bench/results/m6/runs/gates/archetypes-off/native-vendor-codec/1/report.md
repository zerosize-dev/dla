# DLA report

Run: `20261110-182922-ab87`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 10 enumerated, 10 dispositioned; confirmed 0, strong 0, refuted 0, suspected 1, unresolved 9

Residual: not measured

Coverage: not measured

## Findings

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
- Remedy: install libacmecodec + acme/codec.h, run dla harness run with --tool valgrind/heaptrack; check whether acme_frame passed to on_frame is owned by the caller (must be released) or borrowed; sink_push/ring_put store the borrowed pointer (src/ring.c:34) without copy or release
- Supporting evidence: E5

### F1: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: static
- Holder: container-field
- Owner: `src.ring.ring.items`
- Region: off-heap
- Site: `src/ring.c:6`
- Writers: `src/ring.c:17`, `src/ring.c:34`
- Removers: `src/ring.c:28`
- Annotation: heuristic name match over the project's sources
- Reason: env_unavailable
- Remedy: cannot build/run: vendor library libacmecodec absent; exercise with dla harness once installed
- Supporting evidence: E5

### F2: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.ring.ring_create`
- Region: off-heap
- Site: `src/ring.c:14`
- Removers: `src/ring.c:19`
- Annotation: heuristic name match over the project's sources
- Reason: env_unavailable
- Remedy: cannot build/run: vendor library libacmecodec absent; exercise with dla harness once installed
- Supporting evidence: E5

### F3: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.ring.ring_create`
- Region: off-heap
- Site: `src/ring.c:17`
- Writers: `src/ring.c:34`
- Removers: `src/ring.c:28`
- Annotation: heuristic name match over the project's sources
- Reason: env_unavailable
- Remedy: cannot build/run: vendor library libacmecodec absent; exercise with dla harness once installed
- Supporting evidence: E5

### F5: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: static
- Holder: closure/continuation
- Owner: `src.session.session_open.on_frame`
- Region: heap
- Site: `src/session.c:33`
- Reason: env_unavailable
- Remedy: cannot build/run: vendor library libacmecodec absent; exercise with dla harness once installed
- Supporting evidence: E5

### F6: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: static
- Holder: closure/continuation
- Owner: `src.session.session_open.on_status`
- Region: heap
- Site: `src/session.c:34`
- Reason: env_unavailable
- Remedy: cannot build/run: vendor library libacmecodec absent; exercise with dla harness once installed
- Supporting evidence: E5

### F7: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.session.session_open`
- Region: off-heap
- Site: `src/session.c:37`
- Removers: `src/session.c:49`
- Annotation: heuristic name match over the project's sources
- Reason: env_unavailable
- Remedy: cannot build/run: vendor library libacmecodec absent; exercise with dla harness once installed
- Supporting evidence: E5

### F8: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.session.codec`
- Region: off-heap
- Site: `src/session.h:9`
- Writers: `src/session.c:37`
- Removers: `src/session.c:49`
- Annotation: heuristic name match over the project's sources
- Reason: env_unavailable
- Remedy: cannot build/run: vendor library libacmecodec absent; exercise with dla harness once installed
- Supporting evidence: E5

### F9: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.sink.sink_init`
- Region: off-heap
- Site: `src/sink.c:5`
- Removers: `src/sink.c:15`
- Annotation: heuristic name match over the project's sources
- Reason: env_unavailable
- Remedy: cannot build/run: vendor library libacmecodec absent; exercise with dla harness once installed
- Supporting evidence: E5

### F10: **UNCLASSIFIED**

- State: unresolved(env_unavailable)
- Evidence classes: static
- Holder: container-field
- Owner: `src.sink.sink.replay`
- Region: off-heap
- Site: `src/sink.h:14`
- Writers: `src/sink.c:5`
- Removers: `src/sink.c:15`
- Annotation: heuristic name match over the project's sources
- Reason: env_unavailable
- Remedy: cannot build/run: vendor library libacmecodec absent; exercise with dla harness once installed
- Supporting evidence: E5

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

- E5: partial coverage
- E5: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E5: limitation: writers and removers are textual name matches

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | — |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | — |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | — |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | — |
| `intake.build` | Does the project build locally? | unresolved | — | env_unavailable: install libacmecodec and acme/codec.h so make can link |
| `intake.workload` | What can exercise the suspected path? | unresolved | — | env_unavailable: needs the vendor library to run ./capture --iterations N |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E1 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo fa1b220e399559e4fca0a35fda04e6278bc9da19 | — |
| E2 | run-config | static | dla 0.1.0 | repo fa1b220e399559e4fca0a35fda04e6278bc9da19 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo fa1b220e399559e4fca0a35fda04e6278bc9da19 | probe/probe.json |
| E4 | static-hit | static | dla 0.1.0 | repo fa1b220e399559e4fca0a35fda04e6278bc9da19 | sweep/native-dla.json |
| E5 | census-holder | static | dla-census 0.1.0 | repo fa1b220e399559e4fca0a35fda04e6278bc9da19 | census/native.json |
