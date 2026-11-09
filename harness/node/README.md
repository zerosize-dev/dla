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

## Raw captures (`DLA_CAPTURE`)

`dla harness run H<n> --capture heap,alloc,nonheap` sets `DLA_CAPTURE`, `DLA_CAPTURE_DIR` and
`DLA_CAPTURE_DUMPS`; the agent writes files to `<DLA_CAPTURE_DIR>/<pid>/` and lists each one in the
`captures` array of the record that triggered it (`kind`, `file`, `role`, `bytes`, `ms`, or `error`
when it could not be written). `histogram` and `jfr` are JVM captures and become a limitation here.

| Kind | File | Format | Written | `dla summarize` |
|---|---|---|---|---|
| `heap` | `heap-<n>.heapsnapshot` | V8 heap snapshot (`v8.writeHeapSnapshot`, after a forced gc) | driver: at the checkpoints of the JVM rule (end of warm-up, last round, evenly spaced between; `DLA_CAPTURE_DUMPS`, default 4); command: at exit of the selected points | `heap-graph` over 3 or more snapshots of ONE process (driver mode); `type-histogram-diff` over snapshots of any points |
| `heap` | `scripts.json` | `{"dla_scripts":1,"root","scripts":{"<script id>":"<url>"},"maps":{"<id>":"<sourceMapURL>"}}` | next to each snapshot (rewritten; the last wins) | sidecar of the snapshots (`Debugger.scriptParsed`); never passed to `dla summarize`: Node 22 snapshots carry the script path on the function's shared info, so the adapters do not read it (a fallback for older Node) |
| `alloc` | `alloc-<n>.heapprofile` | inspector sampling heap profile (`{head, samples}`; `callFrame.url` is a `file://` url, builtins have url `""` and line `-1`) | driver: at the same checkpoints (each ends a window, a new one starts); command: one at exit | `alloc-stack-diff` over the profiles |
| `nonheap` | `resources.json` | see below | once, at the last round (driver) or at exit (command) | `non-heap` over the files of the points |

Sampling starts at agent load (`HeapProfiler.startSampling`, interval `DLA_HEAPPROF_INTERVAL`
bytes, default 8192, live objects only: a forced gc runs before each profile is read). A profile
covers the objects allocated in its window that are still alive when it is read: a driver's
profile 2 does not hold what profile 1 held. The profile can show the agent's own frames
(`dla-preload.js`); attribute to the nearest project frame.

`resources.json`:

```json
{"dla_resources":1,"pid":123,"runtime":"node","x":4000,"t_ms":812,
 "active":{"TCPSocketWrap":20,"Timeout":1},
 "types":{"Timeout":{"active":0,"tracked":2000,"created":null,"source":"timers",
                     "sites":{"src/heartbeat.js:15":2000},"unattributed":0},
          "TCPSocketWrap":{"active":20,"tracked":20,"created":40,"source":"async_hooks",
                           "sites":{"src/conn.js:3":10},"unattributed":10}},
 "limitations":["..."]}
```

- `active`: `process.getActiveResourcesInfo()` counts (ref'd handles only; an `unref()`'d timer is not in it).
- `types.<T>.tracked`: resources created and not yet destroyed. `Timeout` comes from the agent's timer
  wrappers (`source: "timers"`, exact). Other types come from an `async_hooks` hook
  (`source: "async_hooks"`); node queues the destroy event until the next tick, so a handle closed in
  the current tick can still be counted (compare with `active`). Type names follow
  `getActiveResourcesInfo` for the handle types it knows (`TCPSocketWrap`, `TCPServerWrap`,
  `PipeWrap`, ...). Promises, ticks, immediates and request objects are not tracked.
- `sites`: first stack frame inside `DLA_PROJECT`, as for timers (at most 40 per type, 40 types); a
  socket a server accepted takes its server's site; `unattributed` counts the rest.

Snapshots are tier 4 data (memory contents): they stay in the run directory. Before each snapshot the
agent checks that about 12 times `heapUsed` is available (`/proc/meminfo` `MemAvailable`, the cgroup
headroom; `DLA_CAPTURE_MEM_BYTES` overrides) and that the snapshots of the process stay under 3 GiB;
otherwise the `captures` entry has an `error` and no file is written. A snapshot blocks the process
for about 30 ms per MB of heap.

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
