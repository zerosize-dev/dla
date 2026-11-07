# DLA report

Run: `20261107-194120-4fd0`

Status: COMPLETE

Findings: 13 enumerated, 13 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 13

Residual: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.cache.entry.chain`
- Region: off-heap
- Site: `src/cache.c:9`
- Writers: `src/cache.c:130`
- Removers: `src/cache.c:71`
- Annotation: heuristic name match over the project's sources
- Annotation: x
- Annotation: hash chain; unlinked in bucket_remove on eviction
- Reason: tool_missing
- Remedy: Installed dla lacks gate/harness/sample/trend/ledger test|confirm|refute. Manual measurement: textrun RSS flat ~4MB at 20k..800k iterations, entries plateau at 2048/2048, evictions track misses, all cache entries freed in cache_destroy; no unbounded growth found. Re-run with full dla release to formally refute.
- Supporting evidence: E5

### F2: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.cache.entry.newer`
- Region: off-heap
- Site: `src/cache.c:10`
- Writers: `src/cache.c:63`, `src/cache.c:66`, `src/cache.c:74`
- Removers: `src/cache.c:71`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: Installed dla lacks gate/harness/sample/trend/ledger test|confirm|refute. Manual measurement: textrun RSS flat ~4MB at 20k..800k iterations, entries plateau at 2048/2048, evictions track misses, all cache entries freed in cache_destroy; no unbounded growth found. Re-run with full dla release to formally refute.
- Supporting evidence: E5

### F3: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.cache.entry.older`
- Region: off-heap
- Site: `src/cache.c:11`
- Writers: `src/cache.c:59`, `src/cache.c:72`
- Removers: `src/cache.c:66`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: Installed dla lacks gate/harness/sample/trend/ledger test|confirm|refute. Manual measurement: textrun RSS flat ~4MB at 20k..800k iterations, entries plateau at 2048/2048, evictions track misses, all cache entries freed in cache_destroy; no unbounded growth found. Re-run with full dla release to formally refute.
- Supporting evidence: E5

### F4: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.cache.glyph_cache.buckets`
- Region: off-heap
- Site: `src/cache.c:15`
- Writers: `src/cache.c:48`, `src/cache.c:131`
- Removers: `src/cache.c:152`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: Installed dla lacks gate/harness/sample/trend/ledger test|confirm|refute. Manual measurement: textrun RSS flat ~4MB at 20k..800k iterations, entries plateau at 2048/2048, evictions track misses, all cache entries freed in cache_destroy; no unbounded growth found. Re-run with full dla release to formally refute.
- Supporting evidence: E5

### F5: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.cache.glyph_cache.head`
- Region: off-heap
- Site: `src/cache.c:17`
- Writers: `src/cache.c:61`, `src/cache.c:75`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: Installed dla lacks gate/harness/sample/trend/ledger test|confirm|refute. Manual measurement: textrun RSS flat ~4MB at 20k..800k iterations, entries plateau at 2048/2048, evictions track misses, all cache entries freed in cache_destroy; no unbounded growth found. Re-run with full dla release to formally refute.
- Supporting evidence: E5

### F6: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.cache.glyph_cache.tail`
- Region: off-heap
- Site: `src/cache.c:18`
- Writers: `src/cache.c:65`, `src/cache.c:77`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: Installed dla lacks gate/harness/sample/trend/ledger test|confirm|refute. Manual measurement: textrun RSS flat ~4MB at 20k..800k iterations, entries plateau at 2048/2048, evictions track misses, all cache entries freed in cache_destroy; no unbounded growth found. Re-run with full dla release to formally refute.
- Supporting evidence: E5

### F7: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.cache.cache_create`
- Region: off-heap
- Site: `src/cache.c:43`
- Removers: `src/cache.c:50`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: Installed dla lacks gate/harness/sample/trend/ledger test|confirm|refute. Manual measurement: textrun RSS flat ~4MB at 20k..800k iterations, entries plateau at 2048/2048, evictions track misses, all cache entries freed in cache_destroy; no unbounded growth found. Re-run with full dla release to formally refute.
- Supporting evidence: E5

### F8: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.cache.cache_create`
- Region: off-heap
- Site: `src/cache.c:48`
- Writers: `src/cache.c:131`
- Removers: `src/cache.c:152`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: Installed dla lacks gate/harness/sample/trend/ledger test|confirm|refute. Manual measurement: textrun RSS flat ~4MB at 20k..800k iterations, entries plateau at 2048/2048, evictions track misses, all cache entries freed in cache_destroy; no unbounded growth found. Re-run with full dla release to formally refute.
- Supporting evidence: E5

### F9: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.cache.cache_lookup`
- Region: off-heap
- Site: `src/cache.c:118`
- Writers: `src/cache.c:106`
- Removers: `src/cache.c:123`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: Installed dla lacks gate/harness/sample/trend/ledger test|confirm|refute. Manual measurement: textrun RSS flat ~4MB at 20k..800k iterations, entries plateau at 2048/2048, evictions track misses, all cache entries freed in cache_destroy; no unbounded growth found. Re-run with full dla release to formally refute.
- Supporting evidence: E5

### F10: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.glyph.glyph_bitmap.coverage`
- Region: off-heap
- Site: `src/glyph.h:18`
- Writers: `src/raster.c:59`
- Removers: `src/raster.c:65`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: Installed dla lacks gate/harness/sample/trend/ledger test|confirm|refute. Manual measurement: textrun RSS flat ~4MB at 20k..800k iterations, entries plateau at 2048/2048, evictions track misses, all cache entries freed in cache_destroy; no unbounded growth found. Re-run with full dla release to formally refute.
- Supporting evidence: E5

### F11: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.glyph.placed_glyph.bitmap`
- Region: off-heap
- Site: `src/glyph.h:47`
- Writers: `src/layout.c:17`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: Installed dla lacks gate/harness/sample/trend/ledger test|confirm|refute. Manual measurement: textrun RSS flat ~4MB at 20k..800k iterations, entries plateau at 2048/2048, evictions track misses, all cache entries freed in cache_destroy; no unbounded growth found. Re-run with full dla release to formally refute.
- Supporting evidence: E5

### F12: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.main.main`
- Region: off-heap
- Site: `src/main.c:49`
- Reason: tool_missing
- Remedy: Installed dla lacks gate/harness/sample/trend/ledger test|confirm|refute. Manual measurement: textrun RSS flat ~4MB at 20k..800k iterations, entries plateau at 2048/2048, evictions track misses, all cache entries freed in cache_destroy; no unbounded growth found. Re-run with full dla release to formally refute.
- Supporting evidence: E5

### F13: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.raster.raster_glyph`
- Region: off-heap
- Site: `src/raster.c:27`
- Writers: `src/raster.c:43`, `src/raster.c:52`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: Installed dla lacks gate/harness/sample/trend/ledger test|confirm|refute. Manual measurement: textrun RSS flat ~4MB at 20k..800k iterations, entries plateau at 2048/2048, evictions track misses, all cache entries freed in cache_destroy; no unbounded growth found. Re-run with full dla release to formally refute.
- Supporting evidence: E5

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

- E5: partial coverage
- E5: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E5: limitation: writers and removers are textual name matches
- E6: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E6: limitation: workloads were discovered by reading files; none was run

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | textrun: C glyph cache text renderer (src/*.c) |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | whole repo, RSS growth |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | RSS rises under traffic, high after minutes |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | native C |
| `intake.build` | Does the project build locally? | answered | E1 | Makefile, cc |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | textrun --iterations N --capacity C |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4, raw values |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 60m |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | — |
| E2 | run-config | static | dla 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | probe/probe.json |
| E4 | static-hit | static | dla 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | sweep/dla.json |
| E5 | census-holder | static | dla-census 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | census/native.json |
| E6 | workload | static | dla 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | workload/workloads.json |
