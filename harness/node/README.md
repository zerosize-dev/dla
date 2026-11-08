# Node agent and driver

| File | Role |
|---|---|
| `dla-preload.js` | agent; loaded with `NODE_OPTIONS=--require <abs path>/dla-preload.js` |
| `dla-harness.js` | loop driver template |

## Run

Command mode (Node 18+; tested on 22):

```
cd <scratch project root>
NODE_OPTIONS="--require <harness dir>/dla-preload.js" \
DLA_HARNESS_OUT=... DLA_HARNESS_PROBES=... DLA_PROJECT=<scratch project root> \
NODE_V8_COVERAGE=<harness dir>/cov node load.js --iterations N
```

Append to an existing `NODE_OPTIONS` rather than replacing it. The agent does nothing when
`DLA_HARNESS_OUT` is unset, in worker threads, and in `npm`/`npx`/`yarn`/`pnpm`/`corepack`
(they inherit `NODE_OPTIONS` but are never measured), so `npm run <script>` measures the script's
`node` process. Child `node` processes the workload starts write their own records to the same file
(one line per pid).

`--expose-gc` is accepted in `NODE_OPTIONS` on Node 22, but the agent does not need it: it enables
`gc` with `v8.setFlagsFromString('--expose-gc')` + `vm.runInNewContext('gc')` when the global is
missing. Without any `gc` the dump says so in `limitations` and `gc_forced` is false.

## What the agent wraps

- Timers: `setInterval`, `setTimeout`, `clearInterval`, `clearTimeout` (global and `require('timers')`),
  `Timeout.prototype.close`, `refresh` and `Symbol.dispose`. A registration is live from the call
  until it is cleared, closed, or (timeouts) fired. `unref()` does not change the count.
  `setImmediate`, `timers/promises` and timers Node creates internally are not counted.
- Listeners: `EventEmitter.prototype` `on`, `addListener`, `prependListener`, `once`,
  `prependOnceListener`, `off`, `removeListener`, `removeAllListeners`, and `EventTarget.prototype`
  `addEventListener`/`removeEventListener`. At dump time each record is reconciled with
  `events.getEventListeners`, so a `once` listener that fired, an aborted `signal` listener and a
  duplicate registration are not counted. A listener counts only while its emitter is alive.
- Records hang off the timer/emitter object itself (hidden symbol property), so they do not keep a
  dead emitter alive. Listener functions of a live emitter are held once more by the record.
- Each registration is attributed to the first stack frame inside `DLA_PROJECT` (skipping
  `node_modules` and the agent), as `relative/file.js:line`. Registrations with no project frame count
  in the totals only.
- `Module.prototype._compile` of the probed files: an accessor is appended inside the module wrapper
  so `const`/`let`/`class` bindings are reachable. CommonJS only; an ES module probe reports
  `error: "module has no probe accessor ..."`.

## What the dump records

- `values` after `gc()` twice (and a third time when the agent had to drop the weak references of
  dead emitters): `node.heap_used`, `node.heap_total`, `node.external`, `node.array_buffers`,
  `os.rss` (from `process.memoryUsage()`), the `os.*` figures from `/proc/self/status`,
  `node.native_contexts`, `node.detached_contexts`, `node.malloced_memory` (`v8.getHeapStatistics()`),
  `node.active.<Type>` (counts from `process.getActiveResourcesInfo()`), and `node.timers` /
  `node.listeners` (live registrations, all sites). `values_pre_gc` holds the four `node.*` memory
  values and `os.rss` read just before the first collection.
- `groups`: `node.timers` and `node.listeners`, each `{ "src/heartbeat.js:15": 8000, ... }`.
- `probes` (one record per measured binding, `path` = `file:name`, one-level breakdown as
  `file:name.prop`):
  - `NodeModule {file, names[]}`: each named top-level binding that holds data (functions are
    skipped; a class contributes its static fields). When `names` is empty the agent scans the
    file for top-level `const|let|var|class|function` declarations.
  - `NodeBinding {file, name}`: that one binding; a numeric primitive is reported with `size` = its
    value (a counter variable).
  - `size`: `Map`/`Set` size or array length, else null. `reachable`: objects reached by an
    iterative walk bounded at 2,000,000 objects (`WeakSet` visited), over own data properties,
    array elements and `Map`/`Set` contents. Functions (closure variables cannot be inspected),
    typed arrays, buffers, `WeakMap`, promises, `Timeout`/`Socket`-like handles, `process` and
    `globalThis` are counted but not followed. `bytes` is always null (no cheap estimate).
  - The breakdown reports each own object-valued property of the binding (not of arrays, maps, sets),
    capped at 64, each walked on its own; a truncated walk sets `error`.
- `purge`: `DLA_PURGE=gc` records `os.rss` and `node.heap_used` before the forced collection and
  after it; any other kind is a limitation (the node purge is `gc` only).
- Coverage is written by Node itself to `NODE_V8_COVERAGE` (the harness sets it to `<harness>/cov`);
  the agent adds nothing to the dump.
- `detectors`: `MaxListenersExceededWarning` raised during the run.
- Ticks: `DLA_TICK_MS` writes a `tick` record (no collection, no probes) from an unref'd timer. A
  workload that never yields to the event loop (a synchronous main) produces none.

The agent's own bookkeeping is small but not zero: a hidden property per timer handle and per
emitter, one weak reference per emitter, and one record entry per live listener.

## Driver

`NODE_OPTIONS="--require <harness dir>/dla-preload.js" node <harness dir>/dla-harness.js` from the
scratch project root. The driver requires the agent (`DLA_AGENT` or the file next to it, the same
module instance as the preloaded one when the path is identical) and calls `checkpoint(done, last)`
after every round; the agent then writes no `exit` record. The loop yields to the event loop
(`setImmediate`) at the end of each round so timers and I/O callbacks run.

### Placeholders

| Placeholder | Where | Meaning |
|---|---|---|
| `@@DLA_STEP@@` | `dla-harness.js` | units per round (default for `DLA_STEP`) |
| `@@DLA_WARMUP@@` | `dla-harness.js` | warm-up rounds (default for `DLA_WARMUP`) |
| `@@DLA_ROUNDS@@` | `dla-harness.js` | measured rounds (default for `DLA_ROUNDS`) |
| `@@DLA_REPEAT@@` | `dla-harness.js` | repeated key space size (default for `DLA_REPEAT`) |

Environment read at run time: `DLA_KEY_MODE` (`fresh`|`repeated`), `DLA_AGENT` (absolute path of the
agent when the driver is not next to it), plus the agent variables above.

### Hooks

| Hook | Signature | Fill with |
|---|---|---|
| `DLA-HOOK setup` | `async function setup()` | load the project modules with `projectRequire('src/x.js')` and build the objects under test; keep them in `state` |
| `DLA-HOOK unit` | `async function unit(i, key)` | one unit of the workload for `key`; derive ids from `key` so a repeated key reproduces an earlier request; may return a promise |
