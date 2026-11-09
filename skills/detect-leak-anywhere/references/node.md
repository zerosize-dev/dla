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
- Raw captures: `dla harness run H1 --capture heap,alloc,nonheap` (`histogram` and `jfr` are JVM kinds and
  become a limitation). `alloc` writes sampling heap profiles (`alloc-<n>.heapprofile`), `nonheap` a
  `resources.json` of active resources by type and creation site (timers are exact, unref'd ones included),
  `heap` V8 snapshots (`heap-<n>.heapsnapshot`). Command mode takes one set per selected point, in separate
  processes, so it feeds `type-histogram-diff` only; a snapshot series for `heap-graph` needs driver mode
  (`--driver`, 3 or more snapshots of one process in one pid directory). Run the `dla summarize` commands
  the harness prints. Before a snapshot the agent checks that about 12 times `heapUsed` is available and
  skips it with an error if not (a coverage gap, not a failure).

## Attach backends
| Backend | Tier | Notes |
|---|---|---|
| `dla attach node-inspector --pid <P> --op memory` | 2 | `process.memoryUsage`; sends `SIGUSR1` only to an executable named `node` or `nodejs`, and the inspector stays open until the process exits |
| `dla attach node-inspector --pid <P> --op heap-snapshot` | 4 | peak extra memory about 12x `heapUsed` (measured 12.6x); the target blocks for seconds; refused as `insufficient-resources` when it does not fit |
| `dla sample --pid <P> --counters node-inspect` | 2 | same SIGUSR1 rule |

## Artifacts
- `.heapsnapshot` files land under `attach/P<n>/` (attach) or `harness/H<n>/out/` (harness). Read them
  with `dla summarize` (`node-heapsnapshot` for 3 or more snapshots of one process, `node-heapsnapshot-types`
  for any); never parse them by hand. Two snapshots from different pid directories are refused
  (`dumps-not-comparable`); an attach snapshot taken twice of the same pid is one process.
- What the series gives: a `holder:` growth column per holder with its kind (static/global,
  registration-table, closure/continuation, container-field). A holder the census labelled differently
  (the census calls a `setInterval` timer `blocked-stack`) is reported as `observed_holder` and `needed`
  prints the `dla ledger annotate F --holder <kind>` command; run it, then the column ties by holder.
  New candidates come from project holders only (at most 3 per run), without a site.
- `alloc-<n>.heapprofile`: `dla summarize alloc-stack-diff`. Frames of `dla-preload.js` and builtins are
  skipped. A `<script>.map` next to a bundled script is applied.
- `resources.json`: `dla summarize non-heap`. A group `Timeout@src/heartbeat.js:15` ties to the finding
  at that site. `getActiveResourcesInfo` totals are metrics and tie to nothing.

## Blind spots
- Snapshots can double the heap and perturb the process; growth that appears only with the
  instrument on is a possible artefact.
- Native memory (Buffers, array buffers, addons) grows outside the JS heap; DLA does not see
  inside an addon.
- An async-local store can hide the real holder: the cause is the registration.
- ES module probes are not supported (CommonJS only); closure variables cannot be inspected.
- Heap analysis tools lag new Node releases.
- A driver that keeps its own reference to the leaking object during a snapshot gives the object a
  second retainer, and the module variable stops dominating it: no static/global holder is reported.
- Snapshot object ids are assigned in traversal order, so two processes of one program can overlap; only
  the pid directory proves one process.
- `async_hooks` destroy events arrive a tick late, so resources closed in the current tick can still
  count as tracked (timers are exact).
- A heap profile frame is the function, not the allocation line.
