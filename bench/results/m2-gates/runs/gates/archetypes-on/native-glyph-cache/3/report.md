# DLA report

Run: `20261107-195302-ab84`

Status: COMPLETE

Findings: 13 enumerated, 13 dispositioned; confirmed 0, strong 0, refuted 0, suspected 13, unresolved 0

Residual: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `src.cache.entry.chain`
- Region: off-heap
- Site: `src/cache.c:9`
- Writers: `src/cache.c:130`
- Annotation: heuristic name match over the project's sources
- Reason: no_harness
- Remedy: This dla release lacks gate/harness/sample/trend/gap/confirm commands; manual RSS runs (maxrss via wait4) show a plateau, not growth, so no leak is confirmed. Holder is the bounded glyph LRU cache (entries x bitmap bytes); to verify, run a service-like workload with the production capacity and compare RSS plateau vs capacity
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `src.cache.entry.newer`
- Region: off-heap
- Site: `src/cache.c:10`
- Writers: `src/cache.c:63`, `src/cache.c:66`, `src/cache.c:74`
- Removers: `src/cache.c:71`
- Annotation: heuristic name match over the project's sources
- Reason: no_harness
- Remedy: This dla release lacks gate/harness/sample/trend/gap/confirm commands; manual RSS runs (maxrss via wait4) show a plateau, not growth, so no leak is confirmed. Holder is the bounded glyph LRU cache (entries x bitmap bytes); to verify, run a service-like workload with the production capacity and compare RSS plateau vs capacity
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `src.cache.entry.older`
- Region: off-heap
- Site: `src/cache.c:11`
- Writers: `src/cache.c:59`, `src/cache.c:72`
- Removers: `src/cache.c:66`
- Annotation: heuristic name match over the project's sources
- Reason: no_harness
- Remedy: This dla release lacks gate/harness/sample/trend/gap/confirm commands; manual RSS runs (maxrss via wait4) show a plateau, not growth, so no leak is confirmed. Holder is the bounded glyph LRU cache (entries x bitmap bytes); to verify, run a service-like workload with the production capacity and compare RSS plateau vs capacity
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `src.cache.glyph_cache.buckets`
- Region: off-heap
- Site: `src/cache.c:15`
- Writers: `src/cache.c:48`, `src/cache.c:131`
- Removers: `src/cache.c:152`
- Annotation: heuristic name match over the project's sources
- Reason: no_harness
- Remedy: This dla release lacks gate/harness/sample/trend/gap/confirm commands; manual RSS runs (maxrss via wait4) show a plateau, not growth, so no leak is confirmed. Holder is the bounded glyph LRU cache (entries x bitmap bytes); to verify, run a service-like workload with the production capacity and compare RSS plateau vs capacity
- Supporting evidence: E6

### F5: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `src.cache.glyph_cache.head`
- Region: off-heap
- Site: `src/cache.c:17`
- Writers: `src/cache.c:61`, `src/cache.c:75`
- Annotation: heuristic name match over the project's sources
- Annotation: LRU head/tail maintained by list_unlink/list_push_front; heuristic 'no remover' is a false lead
- Reason: no_harness
- Remedy: This dla release lacks gate/harness/sample/trend/gap/confirm commands; manual RSS runs (maxrss via wait4) show a plateau, not growth, so no leak is confirmed. Holder is the bounded glyph LRU cache (entries x bitmap bytes); to verify, run a service-like workload with the production capacity and compare RSS plateau vs capacity
- Supporting evidence: E6

### F6: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `src.cache.glyph_cache.tail`
- Region: off-heap
- Site: `src/cache.c:18`
- Writers: `src/cache.c:65`, `src/cache.c:77`
- Annotation: heuristic name match over the project's sources
- Reason: no_harness
- Remedy: This dla release lacks gate/harness/sample/trend/gap/confirm commands; manual RSS runs (maxrss via wait4) show a plateau, not growth, so no leak is confirmed. Holder is the bounded glyph LRU cache (entries x bitmap bytes); to verify, run a service-like workload with the production capacity and compare RSS plateau vs capacity
- Supporting evidence: E6

### F7: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.cache.cache_create`
- Region: off-heap
- Site: `src/cache.c:43`
- Removers: `src/cache.c:50`
- Annotation: heuristic name match over the project's sources
- Reason: no_harness
- Remedy: This dla release lacks gate/harness/sample/trend/gap/confirm commands; manual RSS runs (maxrss via wait4) show a plateau, not growth, so no leak is confirmed. Holder is the bounded glyph LRU cache (entries x bitmap bytes); to verify, run a service-like workload with the production capacity and compare RSS plateau vs capacity
- Supporting evidence: E6

### F8: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.cache.cache_create`
- Region: off-heap
- Site: `src/cache.c:48`
- Writers: `src/cache.c:131`
- Removers: `src/cache.c:152`
- Annotation: heuristic name match over the project's sources
- Reason: no_harness
- Remedy: This dla release lacks gate/harness/sample/trend/gap/confirm commands; manual RSS runs (maxrss via wait4) show a plateau, not growth, so no leak is confirmed. Holder is the bounded glyph LRU cache (entries x bitmap bytes); to verify, run a service-like workload with the production capacity and compare RSS plateau vs capacity
- Supporting evidence: E6

### F9: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.cache.cache_lookup`
- Region: off-heap
- Site: `src/cache.c:118`
- Writers: `src/cache.c:106`
- Removers: `src/cache.c:123`, `src/cache.c:96`
- Annotation: heuristic name match over the project's sources
- Annotation: entry calloc; freed by evict_oldest (cache.c:~96 free(victim)) and cache_destroy; count capped by 'while count>=capacity' in cache_lookup. valgrind: 0 bytes in use at exit. Bound is entry COUNT, not bytes.
- Reason: no_harness
- Remedy: This dla release lacks gate/harness/sample/trend/gap/confirm commands; manual RSS runs (maxrss via wait4) show a plateau, not growth, so no leak is confirmed. Holder is the bounded glyph LRU cache (entries x bitmap bytes); to verify, run a service-like workload with the production capacity and compare RSS plateau vs capacity
- Supporting evidence: E6

### F10: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `src.glyph.glyph_bitmap.coverage`
- Region: off-heap
- Site: `src/glyph.h:18`
- Writers: `src/raster.c:59`, `src/raster.c:43`
- Removers: `src/raster.c:65`
- Annotation: heuristic name match over the project's sources
- Annotation: coverage bitmap w*h bytes (avg ~380B at default sizes); released by raster_release in evict_oldest. Total held = entries x bitmap size, so RSS scales with capacity (measured maxrss 2.2MB@256, 3.2MB@2048, 9.7MB@16384, 13MB@65536 entries-limited) and is flat vs iterations (3.2MB from 2k to 1M runs).
- Reason: no_harness
- Remedy: This dla release lacks gate/harness/sample/trend/gap/confirm commands; manual RSS runs (maxrss via wait4) show a plateau, not growth, so no leak is confirmed. Holder is the bounded glyph LRU cache (entries x bitmap bytes); to verify, run a service-like workload with the production capacity and compare RSS plateau vs capacity
- Supporting evidence: E6

### F11: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `src.glyph.placed_glyph.bitmap`
- Region: off-heap
- Site: `src/glyph.h:47`
- Writers: `src/layout.c:17`
- Annotation: heuristic name match over the project's sources
- Reason: no_harness
- Remedy: This dla release lacks gate/harness/sample/trend/gap/confirm commands; manual RSS runs (maxrss via wait4) show a plateau, not growth, so no leak is confirmed. Holder is the bounded glyph LRU cache (entries x bitmap bytes); to verify, run a service-like workload with the production capacity and compare RSS plateau vs capacity
- Supporting evidence: E6

### F12: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.main.main`
- Region: off-heap
- Site: `src/main.c:49`
- Writers: `src/main.c:49`
- Removers: `src/main.c:82`
- Annotation: cache_destroy called at exit
- Reason: no_harness
- Remedy: This dla release lacks gate/harness/sample/trend/gap/confirm commands; manual RSS runs (maxrss via wait4) show a plateau, not growth, so no leak is confirmed. Holder is the bounded glyph LRU cache (entries x bitmap bytes); to verify, run a service-like workload with the production capacity and compare RSS plateau vs capacity
- Supporting evidence: E6

### F13: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.raster.raster_glyph`
- Region: off-heap
- Site: `src/raster.c:27`
- Writers: `src/raster.c:43`, `src/raster.c:52`
- Annotation: heuristic name match over the project's sources
- Reason: no_harness
- Remedy: This dla release lacks gate/harness/sample/trend/gap/confirm commands; manual RSS runs (maxrss via wait4) show a plateau, not growth, so no leak is confirmed. Holder is the bounded glyph LRU cache (entries x bitmap bytes); to verify, run a service-like workload with the production capacity and compare RSS plateau vs capacity
- Supporting evidence: E6

## Ruled out

None.

## Blind spots and unknowns

### Coverage gaps

None recorded.

### Runtime blind spots

None recorded.

### Entry points not exercisable

None recorded.

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
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | textrun C text rendering service (glyph LRU cache), project root |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | src/*.c, tests/test_cache.c |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | RSS rises with traffic, high after minutes (from task text) |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | native C, Makefile |
| `intake.build` | Does the project build locally? | answered | E1 | make |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | textrun --iterations N --capacity C |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 raw values |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 60m |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | — |
| E2 | run-config | static | dla 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | probe/probe.json |
| E4 | probe | static | dla-probe 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | probe/probe.json |
| E5 | static-hit | static | dla 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | sweep/dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | census/native.json |
| E7 | workload | static | dla 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | workload/workloads.json |
