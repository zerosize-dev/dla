# DLA report

Run: `20261110-202809-5080`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 15 enumerated, 15 dispositioned; confirmed 1, strong 0, refuted 1, suspected 13, unresolved 0

Residual: heap 0 bytes/n unattributed of 0 total (threshold 0)

Coverage: native 3520 iterations, 0 of 7 entry points exercised

## Findings

### F14.n1: Hypothesis N-WARM: cache fills to its 2048-entry capacity during warm-up then plateaus (UNCLASSIFIED)

- State: confirmed
- Explained by N-WARM (discriminator E11)
- Evidence classes: context, discriminator, growth
- Sibling hypothesis of: F14
- Hypothesis: N-WARM: cache fills to its 2048-entry capacity during warm-up then plateaus
- Refuted if: memory keeps growing linearly after the cache reports entries==capacity
- Supporting evidence: E9, E8, E11

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.cache.entry.chain`
- Region: off-heap
- Site: `src/cache.c:9`
- Writers: `src/cache.c:130`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: census holder is a field of the bounded LRU glyph cache (cache.c); no separate probe; whole-cache plateau shown by F14 alloc-test E11 (N-WARM)
- Supporting evidence: E5

### F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.cache.entry.newer`
- Region: off-heap
- Site: `src/cache.c:10`
- Writers: `src/cache.c:63`, `src/cache.c:66`, `src/cache.c:74`
- Removers: `src/cache.c:71`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: census holder is a field of the bounded LRU glyph cache (cache.c); no separate probe; whole-cache plateau shown by F14 alloc-test E11 (N-WARM)
- Supporting evidence: E5

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.cache.entry.older`
- Region: off-heap
- Site: `src/cache.c:11`
- Writers: `src/cache.c:59`, `src/cache.c:72`
- Removers: `src/cache.c:66`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: census holder is a field of the bounded LRU glyph cache (cache.c); no separate probe; whole-cache plateau shown by F14 alloc-test E11 (N-WARM)
- Supporting evidence: E5

### F4: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.cache.glyph_cache.buckets`
- Region: off-heap
- Site: `src/cache.c:15`
- Writers: `src/cache.c:48`, `src/cache.c:131`
- Removers: `src/cache.c:152`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: census holder is a field of the bounded LRU glyph cache (cache.c); no separate probe; whole-cache plateau shown by F14 alloc-test E11 (N-WARM)
- Supporting evidence: E5

### F5: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.cache.glyph_cache.head`
- Region: off-heap
- Site: `src/cache.c:17`
- Writers: `src/cache.c:61`, `src/cache.c:75`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: census holder is a field of the bounded LRU glyph cache (cache.c); no separate probe; whole-cache plateau shown by F14 alloc-test E11 (N-WARM)
- Supporting evidence: E5

### F6: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.cache.glyph_cache.tail`
- Region: off-heap
- Site: `src/cache.c:18`
- Writers: `src/cache.c:65`, `src/cache.c:77`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: census holder is a field of the bounded LRU glyph cache (cache.c); no separate probe; whole-cache plateau shown by F14 alloc-test E11 (N-WARM)
- Supporting evidence: E5

### F7: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.cache.cache_create`
- Region: off-heap
- Site: `src/cache.c:43`
- Removers: `src/cache.c:50`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: census holder is a field of the bounded LRU glyph cache (cache.c); no separate probe; whole-cache plateau shown by F14 alloc-test E11 (N-WARM)
- Supporting evidence: E5

### F8: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.cache.cache_create`
- Region: off-heap
- Site: `src/cache.c:48`
- Writers: `src/cache.c:131`
- Removers: `src/cache.c:152`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: census holder is a field of the bounded LRU glyph cache (cache.c); no separate probe; whole-cache plateau shown by F14 alloc-test E11 (N-WARM)
- Supporting evidence: E5

### F9: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.cache.cache_lookup`
- Region: off-heap
- Site: `src/cache.c:118`
- Writers: `src/cache.c:106`
- Removers: `src/cache.c:123`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: census holder is a field of the bounded LRU glyph cache (cache.c); no separate probe; whole-cache plateau shown by F14 alloc-test E11 (N-WARM)
- Supporting evidence: E5

### F10: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.glyph.glyph_bitmap.coverage`
- Region: off-heap
- Site: `src/glyph.h:18`
- Writers: `src/raster.c:59`
- Removers: `src/raster.c:65`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: census holder is a field of the bounded LRU glyph cache (cache.c); no separate probe; whole-cache plateau shown by F14 alloc-test E11 (N-WARM)
- Supporting evidence: E5

### F11: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.glyph.placed_glyph.bitmap`
- Region: off-heap
- Site: `src/glyph.h:47`
- Writers: `src/layout.c:17`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: census holder is a field of the bounded LRU glyph cache (cache.c); no separate probe; whole-cache plateau shown by F14 alloc-test E11 (N-WARM)
- Supporting evidence: E5

### F12: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.main.main`
- Region: off-heap
- Site: `src/main.c:49`
- Reason: not_exercised
- Remedy: census holder is a field of the bounded LRU glyph cache (cache.c); no separate probe; whole-cache plateau shown by F14 alloc-test E11 (N-WARM)
- Supporting evidence: E5

### F13: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `src.raster.raster_glyph`
- Region: off-heap
- Site: `src/raster.c:27`
- Writers: `src/raster.c:43`, `src/raster.c:52`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: census holder is a field of the bounded LRU glyph cache (cache.c); no separate probe; whole-cache plateau shown by F14 alloc-test E11 (N-WARM)
- Supporting evidence: E5

## Ruled out

- **F14** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (non-leak) E11
  - Hypothesis: glyph cache entries (LRU, capacity 2048 entries) grow native.peak_bytes / rss_anon after traffic starts
  - Refuted if: alloc-test shows plateau and purge returns memory, i.e. bounded fill
  - Contradicting evidence: E11
  - Originally supported by: E9, E10

## Blind spots and unknowns

### Coverage gaps

- native/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- native/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)

### Runtime blind spots

- native: the allocator shim does not record which entry points ran

### Entry points not exercisable

None recorded.

### Evidence limitations and unknowns

- E5: partial coverage
- E5: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E5: limitation: writers and removers are textual name matches
- E6: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E6: limitation: workloads were discovered by reading files; none was run
- E7: limitation: harness written; nothing was run (dla harness run measures it)
- E7: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E8: partial coverage (9 of 9 examined)
- E8: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: exercised entry points are not measured for native: the allocator shim sees allocations, not calls
- E9: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: gate thresholds are provisional (not calibrated)
- E10: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: points 1 to 9 come from S1; the extra points from S2
- E11: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E11: limitation: gate and plateau thresholds are provisional (not calibrated)
- E12: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: 4096 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: partial coverage (1 of 3 examined)
- E14: limitation: F14.n1: no passing column is tied to it; nothing attributed
- E14: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E14: limitation: threshold is provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | — |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | — |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | — |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | — |
| `intake.build` | Does the project build locally? | answered | E1 | — |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | — |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E1 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | — |
| E2 | run-config | static | dla 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | probe/probe.json |
| E4 | static-hit | static | dla 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | sweep/native-dla.json |
| E5 | census-holder | static | dla-census 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | census/native.json |
| E6 | workload | static | dla 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | workload/workloads.json |
| E7 | workload | static | dla 0.1.0 | repo d0c47f0665da5f79b9bb9f48e308207bc2c88349 | harness/H1/harness.json |
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (982f3f69d06a) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (b4ec3bca6b92) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (b4ec3bca6b92) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (973ddf9645e8) | alloc/A1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S2.json (ecc4351bf67f) | trend/T3.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (ecc4351bf67f) | trend/T3.json |
| E14 | residual | test | dla 0.1.0 | artifact series/S1.json (b4ec3bca6b92) | gap/residual-S1-E14.json |
