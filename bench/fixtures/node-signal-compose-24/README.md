# node-signal-compose-24

Category `archetype-free` (trap half of a version pair, partner
`node-signal-compose-22`). Expected answer: a confirmed leak at `src/handler.js:12`
(alternative: the process-wide controller at `src/lifecycle.js:6`), labelled
`UNCLASSIFIED`, holder `registration-table`, region `heap`.

The pair differs only in the Node release (`[toolchain] node`, `.nvmrc` and
`engines.node` in `package.json`): `24.13.1` here, `22.12.0` in the partner. The
source, the tests and the prompt are identical.

## Mechanism

Every request builds `AbortSignal.any([lifecycle.signal, deadline.signal])`
(`src/handler.js:12`). `lifecycle.signal` belongs to one `AbortController` for the
whole process (`src/lifecycle.js`). `AbortSignal.any` makes the new signal a
dependant of every source signal, and the long-lived source keeps a record of its
dependants. On Node 24.13.1 the records for finished requests are never dropped from
the source: the derived `AbortSignal` objects themselves are collected, but the
`WeakRef` and `WeakCell` entries that the source holds for them stay (about 170 bytes
per request). On Node 22.12.0 the source does not accumulate them.

The leak is in the runtime's bookkeeping, not in a collection, listener, timer or
closure of the project: the project's own code releases its timer
(`Deadline.release`) and keeps nothing per request. That is why the cause is
`UNCLASSIFIED`. The finding is tied to `src/handler.js:12`, the line that creates the
dependants, and to the source signal (`src/lifecycle.js:6`).

## Measurements

Node v24.13.1 (the toolchain named in `expected.toml`). Harness outside `project/`
(`measure.js`): requires `load.js` (`runWorkload` is asynchronous), yields to the
event loop and calls `global.gc()` twice before each reading, runs the workload in
steps and reports `heapUsed` relative to the start. Command:
`node --expose-gc measure.js project 5000 20000 80000 160000`.

| requests | heap after GC (KB) | bytes per request |
|---|---|---|
| 5000   | 1015  | 208 |
| 20000  | 3773  | 193 |
| 80000  | 13421 | 172 |
| 160000 | 27131 | 174 |

Slope between 20000 and 160000 requests: (27131 - 3773) KB / 140000 = 171 bytes per
request, linear. The retained objects are `WeakRef` and `WeakCell` entries, not
`AbortSignal` instances.

Counterfactual (a scratch copy with `const scope = deadline.signal;` instead of the
composed signal): heap after GC 123, 304, 318 and 321 KB at 5000, 20000, 80000 and
160000 requests, i.e. flat. The same project on Node 22.12.0 is the partner fixture.

Workload: `node load.js --iterations 20000` runs in about one second. Unit tests:
`node --test` (7 pass on 24.13.1 and on 22.12.0).

## Minimal edit to the control

See `node-signal-compose-22/README.md`: the Node release only.

## Dynamic result

`dla-bench dynamic` (pinned binaries, `[toolchain] node = "24.13.1"`), archetypes on
and off: the leak is not confirmed (FAIL, a DLA limitation, not a fixture error). The
growth is seen: the harness trend passes on `node.heap_used` and `node.heap_total`,
and `summarize type-histogram-diff` over 4 snapshots lists `system / WeakCell`
(+5.7 MB) and `WeakRef` (+2.7 MB) as growing types. But no holder of the heap graph
grows (`heap-graph`: "no holder grows strictly in bytes and count across all
snapshots", top holder `BuiltinModule.map`), the in-process probe on the process-wide
signal is flat, and a generic type never ties to a finding, so no finding is tied to
`src/handler.js:12`. The run ends with the growth unresolved.
