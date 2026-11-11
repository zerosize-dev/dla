# node-socket-heartbeat-cleared

Category `decoy-control` (control half of a pair, partner `node-socket-heartbeat`).
Expected answer: no confirmed leak; the `setInterval` per connection is ruled out
(`N1`, `N-WARM`: memory plateaus).

## Mechanism

Identical to the partner except that `Session.close` calls
`clearInterval(this.heartbeat)`. The interval created at `src/heartbeat.js:15` is
removed from the timer table when the connection closes, so a session becomes
unreachable as soon as the gateway drops it. The `setInterval` per connection and
the closure over the session look as suspicious as in the trap.

## Distinguishing edit (trap vs control)

One added line; 564 lines of `diff -ru` output, run from `bench/fixtures`
(`diff -ru node-socket-heartbeat/project node-socket-heartbeat-cleared/project`):

```
diff -ru node-socket-heartbeat/project/src/session.js node-socket-heartbeat-cleared/project/src/session.js
--- node-socket-heartbeat/project/src/session.js	2026-11-07 15:38:20.937276161 +0000
+++ node-socket-heartbeat-cleared/project/src/session.js	2026-11-07 15:38:44.514210621 +0000
@@ -62,6 +62,7 @@
   close(reason) {
     if (this.closed) return;
     this.closed = true;
+    clearInterval(this.heartbeat);
     for (const topic of [...this.topics]) this.router.unsubscribe(this, topic);
     this.socket.destroy();
     this.metrics.inc(`closed_${reason}`);
```

The loopback socket's `destroy()` emits `close` once, asynchronously, like
`net.Socket`; the partner's README says why. The measurements were retaken after
that change.

## Measurements

Same harness and workload as the partner (`node --expose-gc measure-sock.js project N`).

| connections | open sessions | live intervals | heap after GC (KB) |
|---|---|---|---|
| 2000  | 0 | 0 | 284 |
| 8000  | 0 | 0 | 373 |
| 16000 | 0 | 0 | 407 |

Live intervals are 0 and heap after GC stays under 0.5 MB at every count and does not
scale with the count (0.15 KB per connection at 2000, 0.03 KB at 16000). During the run at most 33 sessions (the replay
window of 32 plus the one being opened) exist at once. Contrast with the trap: 16000
live intervals and 67 MB at 16000 connections.

Workload: `node load.js --iterations 8000` about 0.5 s. Unit tests: `node --test`
(9 pass).
