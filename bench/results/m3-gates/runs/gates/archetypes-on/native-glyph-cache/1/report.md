# DLA report

Run: `20261108-123533-6db3`

Status: COMPLETE

Findings: 15 enumerated, 15 dispositioned; confirmed 0, strong 0, refuted 1, suspected 0, unresolved 14

Residual: not measured

Coverage: native 3520 iterations, 0 of 7 entry points exercised

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.cache.entry.chain`
- Region: off-heap
- Site: `src/cache.c:9`
- Writers: `src/cache.c:130`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: static census holder; bounded by LRU cache, alloc-test (E12) shows plateau
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.cache.entry.newer`
- Region: off-heap
- Site: `src/cache.c:10`
- Writers: `src/cache.c:63`, `src/cache.c:66`, `src/cache.c:74`
- Removers: `src/cache.c:71`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: static census holder; bounded by LRU cache, alloc-test (E12) shows plateau
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.cache.entry.older`
- Region: off-heap
- Site: `src/cache.c:11`
- Writers: `src/cache.c:59`, `src/cache.c:72`
- Removers: `src/cache.c:66`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: static census holder; bounded by LRU cache, alloc-test (E12) shows plateau
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.cache.glyph_cache.buckets`
- Region: off-heap
- Site: `src/cache.c:15`
- Writers: `src/cache.c:48`, `src/cache.c:131`
- Removers: `src/cache.c:152`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: static census holder; bounded by LRU cache, alloc-test (E12) shows plateau
- Supporting evidence: E6

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.cache.glyph_cache.head`
- Region: off-heap
- Site: `src/cache.c:17`
- Writers: `src/cache.c:61`, `src/cache.c:75`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: static census holder; bounded by LRU cache, alloc-test (E12) shows plateau
- Supporting evidence: E6

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.cache.glyph_cache.tail`
- Region: off-heap
- Site: `src/cache.c:18`
- Writers: `src/cache.c:65`, `src/cache.c:77`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: static census holder; bounded by LRU cache, alloc-test (E12) shows plateau
- Supporting evidence: E6

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.cache.cache_create`
- Region: off-heap
- Site: `src/cache.c:43`
- Removers: `src/cache.c:50`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: static census holder; bounded by LRU cache, alloc-test (E12) shows plateau
- Supporting evidence: E6

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.cache.cache_create`
- Region: off-heap
- Site: `src/cache.c:48`
- Writers: `src/cache.c:131`
- Removers: `src/cache.c:152`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: static census holder; bounded by LRU cache, alloc-test (E12) shows plateau
- Supporting evidence: E6

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.cache.cache_lookup`
- Region: off-heap
- Site: `src/cache.c:118`
- Writers: `src/cache.c:106`
- Removers: `src/cache.c:123`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: static census holder; bounded by LRU cache, alloc-test (E12) shows plateau
- Supporting evidence: E6

### F10: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.glyph.glyph_bitmap.coverage`
- Region: off-heap
- Site: `src/glyph.h:18`
- Writers: `src/raster.c:59`
- Removers: `src/raster.c:65`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: static census holder; bounded by LRU cache, alloc-test (E12) shows plateau
- Supporting evidence: E6

### F11: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.glyph.placed_glyph.bitmap`
- Region: off-heap
- Site: `src/glyph.h:47`
- Writers: `src/layout.c:17`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: static census holder; bounded by LRU cache, alloc-test (E12) shows plateau
- Supporting evidence: E6

### F12: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.main.main`
- Region: off-heap
- Site: `src/main.c:49`
- Reason: not_exercised
- Remedy: static census holder; bounded by LRU cache, alloc-test (E12) shows plateau
- Supporting evidence: E6

### F13: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.raster.raster_glyph`
- Region: off-heap
- Site: `src/raster.c:27`
- Writers: `src/raster.c:43`, `src/raster.c:52`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: static census holder; bounded by LRU cache, alloc-test (E12) shows plateau
- Supporting evidence: E6

### F14.n1: Hypothesis N-INTENT bounded LRU cache (capacity 2048) fills then plateaus; N-WARM (UNCLASSIFIED)

- State: unresolved(time_budget)
- Evidence classes: growth
- Sibling hypothesis of: F14
- Hypothesis: N-INTENT bounded LRU cache (capacity 2048) fills then plateaus; N-WARM
- Refuted if: memory keeps growing past capacity fill / entries exceed capacity
- Reason: time_budget
- Remedy: non-leak sibling that the discriminator favors; no further action
- Supporting evidence: E10

## Ruled out

- **F14** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (non-leak) E12
  - Hypothesis: Glyph cache entries retained without release (leak)
  - Refuted if: RSS/peak plateau once entries reach capacity 2048 and stays flat past that
  - Contradicting evidence: E12
  - Originally supported by: E10, E8

## Blind spots and unknowns

### Coverage gaps

None recorded.

### Runtime blind spots

- native: the allocator shim does not record which entry points ran

### Entry points not exercisable

None recorded.

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

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | textrun native C glyph cache service |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | from task args / Makefile (make; ./textrun --iterations N) |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | from task args / Makefile (make; ./textrun --iterations N) |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | from task args / Makefile (make; ./textrun --iterations N) |
| `intake.build` | Does the project build locally? | answered | E1 | from task args / Makefile (make; ./textrun --iterations N) |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | from task args / Makefile (make; ./textrun --iterations N) |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4, raw values |
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
| E8 | workload | static | dla 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (df07ba5f3aa8) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (59bd4d0e36b9) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (59bd4d0e36b9) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (9f92a4c8e0f6) | alloc/A1.json |
