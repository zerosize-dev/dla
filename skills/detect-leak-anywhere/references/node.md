# Node.js and TypeScript

Worker name and runtime id: `node`. Facts from `harness/node/README.md` and `docs/VERIFY.md`.

Archetype rows (what to look for, how to confirm, how to rule out): read `references/node-archetypes.md` only after `dla gate check --stage classify` passes.

## Harness
- Command mode (Node 18+): `dla-preload.cjs` is loaded through `NODE_OPTIONS=--require ...`
  (appended). It does nothing without `DLA_HARNESS_OUT`, in worker threads, and in
  npm/npx/yarn/pnpm/corepack, so `npm run <script>` measures the script's `node` process.
  Driver mode uses `dla-harness.cjs` with the usual hooks: `projectRequire(file)` for CommonJS, `await projectImport(file)` for ES modules (`.mjs`, or `.js` under `"type": "module"`). Both files are `.cjs`, so they load in any package type.
- `dla trend` writes the residual rows of its series, and `dla harness gen --next` is refused with `capture-first` while a heap row over its threshold has no summarized capture of its harness. In a heap summary the edge `weak_refs_keep_during_job` of V8's strong root list (the objects a `WeakRef` was read from in the current job: the agent reads its own `WeakRef`s when it counts listeners) is skipped like a weak edge, `notes` says how many objects only that edge reached, and the agent's own statics (`listenerRefs`, `mapsByFile`, every script under the run's `agent/`) and symbol fields (`<symbol dla.listeners>`, `<symbol dla.timer>`) are no holders: what they keep counts for the object that owns it. A `node.listeners@<file:line>` column counts the listeners registered at that line that are still attached to live objects after a forced collection; it does not count calls to the line.
- A library written in TypeScript that is built before it runs: the work copy of a git project holds the build output that git ignores when a `package.json` (the root one and those of the packages the copy holds) names it as an entry point (`main`, `module`, `bin`, `exports`, `files`), so `projectRequire('lib/index.js')` loads in the copy; `node_modules` beside such a `package.json` is a symbolic link to the project's own (on Windows a symbolic link too, and the manifest says when it could not be made). `harness gen` prints `build_output_kept` and `node_modules_linked`. The census reads the sources: declaration files (`*.d.ts`, `*.d.mts`, `*.d.cts`) and a JavaScript file that ends with a `sourceMappingURL` comment whose map lists a source file of the project are skipped. A site (listeners, timers, resources, detector hits) and a function the coverage reports in a file with a source map are at the original position, `src/a/b.ts:139` instead of `lib/node/chunk-AB12.js:1076`; the record keeps the built position in `built_sites`, and a file without a map, or without an entry, keeps the built position. A heap residual above its threshold is not left unresolved before its harness has a summarized capture: the gate lists `dla harness run <H> --mode <mode> --capture alloc,heap` and the `dla summarize` commands it prints, and `dla ledger resolve <O> --as unresolved` is refused (`capture-not-summarized`) until a retention or allocation-stack summary computed from capture files of that harness exists (a record whose result file lies in `allocs/` counts through the captures its raw file lists), and a row whose unattributed growth over the span of its series is under 64 KiB asks for no capture.
- The V8 coverage files also count the writer lines of the targets: the innermost range of a function that has block coverage (`isBlockCoverage`) that covers the line, at the source line through the source map, so a writer at a `.ts` line is found; a function V8 gave no block ranges for leaves the line unknown. A function that ran whose writer lines all have a count of 0 reads `writer_not_executed`, with the line and the conditions above it in `remedy`.
- `dla harness run` prints `_attention` first (a detector hit with its site, a short or cut series, a lowered step, missing coverage). `dla trend` lists columns that fail only for their point count in `short`: measure longer before reading "nothing grew". The driver ends by itself and lowers its step to fit the time limit.
- What the agent wraps: timers (`setInterval`, `setTimeout`, clear and close variants) and
  EventEmitter/EventTarget listeners, each attributed to the first project stack frame. A
  `once` listener that fired or an aborted-signal listener is not counted.
- What the dump measures (after `gc()` twice; the agent enables `gc` itself if needed):
  `node.heap_used`, `node.heap_total`, `node.external`, `node.array_buffers`, native contexts,
  `node.malloced_memory`, `node.active.<Type>`, `node.timers`, `node.listeners` (grouped by
  `file:line`), the `os.*` set, and module-binding probes (`NodeModule`, `NodeBinding`; ES modules need Node 22.15+ or 23.5+, else the probe says so).
  A probe whose root the agent cannot enter (a function, a leaf or opaque value, or an instance of a class
  with no own keys, as a class with only `#private` fields has) writes no `probe:` column and says so in a
  limitation; it never reads as a flat line. Such a finding is ruled on by the whole-process columns and a
  holder column, not refuted by a probe.
- Purge: `gc`.
- Raw captures: `dla harness run H1 --capture heap,alloc,nonheap` (`histogram` and `jfr` are JVM kinds and
  become a limitation). `alloc` writes sampling heap profiles (`alloc-<n>.heapprofile`), `nonheap` a
  `resources.json` of active resources by type and creation site (timers are exact, unref'd ones included),
  `heap` V8 snapshots (`heap-<n>.heapsnapshot`). Command mode takes one set per selected point, in separate
  processes. Its snapshots feed `type-histogram-diff`, and `heap-graph` forms a per-point series over n
  from 3 or more of them (listed in `harness/H<n>/captures.json`: kind heap, distinct x, one label). Driver
  mode (`--driver`) gives 3 or more snapshots of one process in one pid directory. Run the `dla summarize` commands
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
  with `dla summarize` (`node-heapsnapshot` for 3 or more snapshots of one process or a per-point series
  of one harness, `node-heapsnapshot-types` for any); never parse them by hand. Two snapshots from
  different pid directories are refused (`dumps-not-comparable`) unless they are command-mode captures of
  one harness (see the next item); an attach snapshot taken twice of the same pid is one process.
- Per-point series: 3 or more command-mode heap captures of one harness form a series over n. `heap-graph`
  orders them by n, skips the one-process checks, keeps the format checks, uses the same strict-growth
  gate and `dump-plateau` discriminator, and says `series_kind: per-point` with the n values. The harness
  prints the command when it has 3 or more files of one format. Measured on copies of the fixtures: on
  `node-retry-scheduler` the module variable `src.scheduler.shared` tied to its census finding
  (static/global).
- What the series gives: a `holder:` growth column per holder. The holder kind is the kind of the
  retention root, not of the last link: the path from the GC roots to the holder decides. A timer or
  event-listener entry on the path gives registration-table; a module variable or a `global` property
  gives static/global; a function-context variable gives closure/continuation; anything else gives
  container-field. Each passing column in the result carries `holder_kind`. A holder that differs from the
  finding's is reported as `observed_holder`. If the finding has no holder, `needed` prints
  `dla ledger annotate F --holder <kind>`; run it, then the column ties by holder. If the finding has a
  different holder, `needed` prints `dla ledger annotate F --holder <kind> --evidence <growth evidence id>`:
  the ledger changes a holder already set only with that growth evidence, when one of its passing columns
  is tied to F and observed that kind. Any other change of a set holder is refused.
  New candidates come from project holders only (at most 3 per run), without a site and with the root kind.
- Runtime bookkeeping that the project feeds shows as a registration holder. The cells of a
  `FinalizationRegistry` whose weak target is still alive are grouped by that target and named by the
  module variable that keeps the target alive (a reverse walk of at most two property hops, not through a
  `Module`'s `exports` object or the wrapper parameters `module`, `exports`, `require`, `__filename`,
  `__dirname`): the view is `registry:finalization:<module>.<variable>`, kind registration-table, and it ties
  to the census finding of that variable by owner. Cells without a target (the object is gone, the cleanup is
  pending) are a backlog and are not counted; a target with no owner or with two owners gives no holder. When
  the same owner also has a plain `static:` holder, the registry one is the bound holder.
  Holders inside `node:` modules are not tracked; when the only holders that grow strictly are those, the
  result says so in `reasons` and `needed` points at the project's own registration or at a probe or
  allocation stack, not at another snapshot.
- `alloc-<n>.heapprofile`: `dla summarize alloc-stack-diff`. Frames of `dla-preload.cjs` and builtins are
  skipped. A `<script>.map` next to a bundled script is applied.
- `resources.json`: `dla summarize non-heap`. A group `Timeout@src/heartbeat.js:15` ties to the finding
  at that site. `getActiveResourcesInfo` totals are metrics and tie to nothing.

## Blind spots
- Snapshots can double the heap and perturb the process; growth that appears only with the
  instrument on is a possible artefact.
- Native memory (Buffers, array buffers, addons) grows outside the JS heap; DLA does not see
  inside an addon.
- An async-local store can hide the real holder: the cause is the registration.
- Timers (`setInterval`, `setTimeout`), event listeners and subscriptions are `registration-table`
  holders: the timer table or the emitter's listener list keeps the callback, and the callback keeps what
  it closes over. They are not `closure/continuation`. That holder is for a function-context variable
  that holds a chain of values, not for the callback a registration keeps. If a finding for a timer or a
  listener has another holder, correct it: `dla ledger annotate F --holder registration-table --evidence <growth evidence id>`.
  Growth evidence from `dla summarize non-heap --backend node-resources` on 3 or more `resources.json`
  files also backs this correction when its passing `Timeout` or listener group is tied to F by the
  creation site, and only the finding's own site: a group tied through a writer's site is refused
  (the output prints the command, and `observed_holder` names the finding).
- ES-module probes read live bindings through `module.registerHooks` (Node 22.15+ or 23.5+); closure variables cannot be inspected.
- Heap analysis tools lag new Node releases.
- `--capture nonheap` uses `async_hooks`; promise-heavy targets can run up to about 10x slower.
- `scripts.json` next to the snapshots is informational; no adapter reads it.
- A driver that keeps its own reference to the leaking object during a snapshot gives the object a
  second retainer, and the module variable stops dominating it: no static/global holder is reported.
- Snapshot object ids are assigned in traversal order, so two processes of one program can overlap; only
  the pid directory proves one process.
- `async_hooks` destroy events arrive a tick late, so resources closed in the current tick can still
  count as tracked (timers are exact).
- A heap profile frame is the function, not the allocation line.
