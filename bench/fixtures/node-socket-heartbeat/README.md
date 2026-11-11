# node-socket-heartbeat

Category `archetype-leak` (trap half of a pair, partner `node-socket-heartbeat-cleared`).
Expected answer: `L-REGISTER`, holder `registration-table`, region `heap`, site
`src/heartbeat.js:15`.

## Mechanism

`Gateway.accept` starts a heartbeat per connection (`startHeartbeat`, which calls
`setInterval` once per session and stores the handle in `session.heartbeat`). The
callback closes over the `Session`, which owns its decoder, topic set and the last
64 decoded messages (about 4 KB of heap per connection after a short exchange).
`Session.close` unsubscribes topics and destroys the socket, and the gateway drops
the session from its `sessions` map, but nothing ever calls `clearInterval`. The
timer table therefore keeps one entry per connection ever accepted and each entry
keeps its session alive. The timers are `unref()`'d, so the process still exits and
`node --test` does not hang; this also means `process.getActiveResourcesInfo()`
does not list them, which is why the measurement below counts intervals with a
wrapper around `setInterval`/`clearInterval`.

The loopback socket stands in for `net.Socket`: `destroy()` emits `close` once,
asynchronously (next tick), as a real socket does, so a session the server ends
(idle timeout, protocol error, `bye`) is released from the `sessions` map like one
the client hangs up. Earlier revisions never emitted `close` on `destroy()`, which
kept such sessions in the map and let a decoy be confirmed honestly as a session
leak. The measurements below were retaken after the change.

Real code around the site: framing, topic fan-out, per-session `recent` window
(capped at 64), a `sessions` map that is removed on close, metrics counters.

## Distinguishing edit

The partner adds one line in `Session.close`: `clearInterval(this.heartbeat);`.
See the diff in the partner's README.

## Measurements

Node v22 (`node --version`). Harness outside `project/`
(`measure-sock.js`): wraps `setInterval`/`clearInterval` to count live intervals,
runs `runWorkload(N)` exported by `load.js`, calls `global.gc()` twice and reads
`process.memoryUsage().heapUsed`, reported relative to the heap before the run.
Command (from this directory): `node --expose-gc measure-sock.js project N`.

| connections | open sessions | live intervals | heap after GC (KB) | bytes per connection |
|---|---|---|---|---|
| 2000  | 0 | 2000  | 8681  | 4445 |
| 8000  | 0 | 8000  | 33900 | 4339 |
| 16000 | 0 | 16000 | 67434 | 4316 |

Slope between 2000 and 16000 connections: (67434 - 8681) KB / 14000 = 4.10 KB per
connection, linear in the iteration count; live intervals equal connections served
while open sessions are 0 at every point.

Workload: `node load.js --iterations 8000` takes about 0.5 s (`--iterations 100000`
about 5 s). Unit tests: `node --test` (9 pass).
