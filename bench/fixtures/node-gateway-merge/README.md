# node-gateway-merge

Category `multi-leak`. Two independent leaks in different modules.

- L1 `L-REGISTER`, share 0.62, holder `registration-table`, region `heap`: site
  `src/audit.js:18` (the `bus.on(...)` call; alternatives `src/audit.js:17`,
  `src/gateway.js:29`, `src/bus.js:6`).
- L2 `L-CAPTURE`, share 0.38, holder `closure/continuation`, region `heap`: site
  `src/trail.js:12` (lines 11 to 17; alternatives `src/trail.js:7`, `src/gateway.js:28`).

## Mechanism

L1. `audit.watch` runs once per request. It registers a listener for the event
`settled:<id>` on the process-wide `bus` (`src/bus.js`), a long-lived
`EventEmitter`. The listener runs once, when `gateway.handle` emits that event, and
is never removed. The emitter keeps one entry per request id in its event table, and
each entry keeps the listener, its closure context and the request's 32 stage
timings.

L2. `trail.track` gives every response an `explain` method for support staff. The
method reads `earlier`, the response served before it (line 11), and the module's
`Trail` keeps the newest response in `lastResponse` (line 17). Each response therefore keeps the
one before it through its own `explain` closure, and the chain from `lastResponse`
back to the first response is never released. The closure only reads two fields of
`earlier`, but it keeps the whole object.

The two chains are separate: the audit listener captures the request context and the
timings, never the response, and the trail keeps responses, never listeners.

## Measurements

Node v22.23.4. Harness outside `project/` (`measure.js`): requires `load.js`, calls
`global.gc()` twice, runs `runWorkload(N)`, calls `global.gc()` twice and reports
`heapUsed` relative to the start. Command: `node --expose-gc measure.js project N`
(one process per N). The two counterfactuals are scratch copies with one line
removed: `audit.watch(...)` in `src/gateway.js` (only L2 remains) or
`trail.track(response)` (only L1 remains).

| requests | both (KB) | L2 only (KB) | L1 only (KB) |
|---|---|---|---|
| 20000 | 14485 | 5495  | 9035  |
| 80000 | 57508 | 21841 | 35706 |

Slope with both leaks: 0.72 KB per request; L1 alone 0.45 KB per request (62% of the
total at 80000 requests), L2 alone 0.27 KB per request (38%). The shares in
`expected.toml` are those fractions. Both are linear in the number of requests; the
sum of the two counterfactuals is within 1% of the total.

Workload: `node load.js --iterations 20000` runs in well under a second. Unit tests:
`node --test` (5 pass).

## Dynamic result

`dla-bench dynamic` (pinned binaries), archetypes on and off: pass, both leaks
confirmed. L1 is confirmed from the growing `node.listeners` column at
`src/audit.js:18` and the `bus._events` probe. L2 is a closure chain behind a module
singleton created in a function scope (as in `node-price-board`), so no in-process
probe sees it: the heap-graph summary over 4 snapshots ties it by holder identity
and the policy drives it like a harness-tied finding.

An earlier shape of `src/trail.js` kept the newest response in a module-level
variable. The harness then put a probe on that variable, saw its size constant and
refuted the candidate with the `N-WARM` test before the heap-graph step could tie it
(see the report of this package for the limitation); the class behind a function
scope avoids that.
