# node-token-cache

Category `archetype-free` (no pair). Expected answer: a confirmed leak at
`src/cache.js:36` (alternatives: the `entries` field at `src/cache.js:10` and the call
at `src/gateway.js:28`), labelled `UNCLASSIFIED`, holder `static/global`, region
`off-heap`.

## Mechanism

`TokenCache` remembers every token that passed signature verification, so a client
that presents the same token again skips the HMAC. Each entry holds the token's raw
bytes, `Buffer.from(token)`, for a constant-time comparison on a hit. An entry is
dropped only when the same token is looked up again after it expired
(`src/cache.js:21`). A client that is issued a new token never presents the old one
again, so the old entry stays in the map of the gateway's single, process-wide cache
(the gateway is created once, in `src/service.js:10`).

Small `Buffer.from(string)` results (under 4 KB) are cut from a shared 8 KB pool slab.
`Gateway.authenticate` makes a transient buffer of the request body (about 2.1 KB) on
every request, so a slab holds only three or four requests. A token buffer that stays
reachable keeps its whole slab alive, even though only 100 bytes of it are the token.
Each remembered token therefore pins about 3 KB of backing store (`external` memory),
against about 0.5 KB of heap for the map entry, the claims and the `Buffer` object.
Growth is linear in the number of tokens issued (about one per five requests here).

The growth does not fit a taxonomy row: the container that grows is a `Map` of
verified tokens (`L-COLLECT` is a heap row, and Node ships no such row), the memory
is mostly off-heap slab pinning rather than the entries themselves, and nothing
registers a callback or captures a scope.

Background: `Buffer.from(string)` uses the pre-allocated pool for small sizes, and a
slice keeps the pool's `ArrayBuffer` alive (Node.js documentation, `Buffer.poolSize`
and `Buffer.allocUnsafe`, https://nodejs.org/api/buffer.html; the "Buffer pool
pinning" effect was reproduced in the measurements below, it was not taken from a
third-party source).

## Measurements

Node v22.23.4 (no `[toolchain]`: the effect is the same on v24). Harness outside
`project/` (`measure.js`): requires `load.js`, calls `global.gc()` twice, runs
`runWorkload(N)` once, calls `global.gc()` twice and reports `heapUsed` and
`external` relative to the start. Command: `node --expose-gc measure.js project N`.

| requests | tokens remembered | heap after GC (KB) | external after GC (KB) |
|---|---|---|---|
| 5000   | 1008  | 622  | 3016   |
| 20000  | 4008  | 2177 | 12016  |
| 80000  | 16008 | 8244 | 48016  |
| 320000 | 64008 | 32462 | 192016 |

External memory grows about 3.0 KB and the heap about 0.5 KB per remembered token;
the slope is constant from 5000 to 320000 requests. Resident memory after 80000
requests is 158 MB against 59 MB at the start (`node load.js`).

Counterfactual (a scratch copy with the `this.entries.set(...)` line removed): heap
after GC 150 and 154 KB and external 8 KB at 20000 and 80000 requests, i.e. flat.

Workload: `node load.js --iterations 20000` runs in about 0.15 s. Unit tests:
`node --test` (5 pass).

## Dynamic result

`dla-bench dynamic` (pinned binaries), archetypes on and off: pass. The harness
column on the process-wide gateway (`probe:src/service.js:gateway`) grows, the
heap-graph summary over 4 snapshots ties the cache, and the finding is confirmed at
`src/cache.js:36` with holder `static/global` (the cache hangs off a module-level
gateway, the same holder the dynamic acceptance reads for `node-retry-scheduler`).
`node.external` and `node.array_buffers` grow with a passing trend, which is where the
off-heap region comes from.
