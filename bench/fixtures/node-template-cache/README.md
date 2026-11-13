# node-template-cache

Category `non-leak` (cause `N-WARM`, no pair). Expected answer: no confirmed leak;
the heap rises while the compiled-template cache fills during warm-up and stays flat
afterwards.

## Mechanism

`templateFor` in `src/cache.js` compiles a notice template the first time its notice
kind and language are needed and keeps the compiled form in a `Map` keyed by
`kind@locale`. The service has 14 notice kinds and 5 languages, so the map can hold
at most 70 entries. Each compiled template keeps its parts (about 900 literal runs
and placeholders from a 21 KB source) and is about 75 KB, so the cache costs about
5 MB once every combination has been seen. The replay in `load.js` picks kinds and
languages from a fixed mix and sees all 70 combinations within about 300 notices.
After that every call is a hit and the heap does not move.

The look-alike is a `Map` that only ever gets entries; the discriminator is that its
key space is closed (14 x 5), so the entry count stops at 70.

## Measurements

Node v22.23.4. Harness outside `project/` (`measure.js`): requires `load.js`, calls
`global.gc()` twice, runs `runWorkload(N)` once, calls `global.gc()` twice and
reports `heapUsed` relative to the start. Command:
`node --expose-gc measure.js project N` (one process per N).

| notices | heap after GC (KB) | templates cached |
|---|---|---|
| 100   | 4355 | 63 |
| 300   | 5169 | 70 |
| 1000  | 5167 | 70 |
| 4000  | 5183 | 70 |
| 16000 | 5185 | 70 |
| 64000 | 5185 | 70 |

Plateau at 5.17 MB from 300 notices on; the slope between 1000 and 64000 notices is
18 bytes per 1000 notices. Workload: `node load.js --iterations 20000` runs in about
0.4 s. Unit tests: `node --test` (6 pass).

## Dynamic result

`dla-bench dynamic` (pinned binaries), archetypes on and off: pass. The cache
candidate is refuted by the plateau test and nothing is confirmed.
