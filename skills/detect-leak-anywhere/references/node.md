# Node.js and TypeScript

Worker name and runtime id: `node`. Facts from `harness/node/README.md` and `docs/VERIFY.md`.

## Harness
- Command mode (Node 18+): `dla-preload.js` is loaded through `NODE_OPTIONS=--require ...`
  (appended). It does nothing without `DLA_HARNESS_OUT`, in worker threads, and in
  npm/npx/yarn/pnpm/corepack, so `npm run <script>` measures the script's `node` process.
  Driver mode uses `dla-harness.js` with the usual hooks.
- What the agent wraps: timers (`setInterval`, `setTimeout`, clear and close variants) and
  EventEmitter/EventTarget listeners, each attributed to the first project stack frame. A
  `once` listener that fired or an aborted-signal listener is not counted.
- What the dump measures (after `gc()` twice; the agent enables `gc` itself if needed):
  `node.heap_used`, `node.heap_total`, `node.external`, `node.array_buffers`, native contexts,
  `node.malloced_memory`, `node.active.<Type>`, `node.timers`, `node.listeners` (grouped by
  `file:line`), the `os.*` set, and module-binding probes (`NodeModule`, `NodeBinding`; CommonJS only).
- Purge: `gc`.

## Attach backends
| Backend | Tier | Notes |
|---|---|---|
| `dla attach node-inspector --pid <P> --op memory` | 2 | `process.memoryUsage`; sends `SIGUSR1` only to an executable named `node` or `nodejs`, and the inspector stays open until the process exits |
| `dla attach node-inspector --pid <P> --op heap-snapshot` | 4 | peak extra memory about 12x `heapUsed` (measured 12.6x); the target blocks for seconds; refused as `insufficient-resources` when it does not fit |
| `dla sample --pid <P> --counters node-inspect` | 2 | same SIGUSR1 rule |

## Artifacts
- `.heapsnapshot` files land under `attach/P<n>/`. No `dla summarize` adapter reads
  V8 snapshots in this release (cut 2). Record the snapshot as an artifact; do not parse it by hand.

## Blind spots
- Snapshots can double the heap and perturb the process; growth that appears only with the
  instrument on is a possible artefact.
- Native memory (Buffers, array buffers, addons) grows outside the JS heap; DLA does not see
  inside an addon.
- An async-local store can hide the real holder: the cause is the registration.
- ES module probes are not supported (CommonJS only); closure variables cannot be inspected.
- Heap analysis tools lag new Node releases.
