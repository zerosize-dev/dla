# Real plugin run grpc-5 (grpc-go v1.84.0, third frozen build, snapshot 89bc8a8), 2026-11-21

Started 21:34 UTC, 2432 s wall, 316 turns, cost 8.32, Status INCOMPLETE (124 open), 47 harness
runs, 140 series, 5 captures, budget 3 h, 39 min used. Evidence: u36/grpc-5 (report, ledger,
gate, queue, transcript; run dir deleted).

## Good
- The documented leak is confirmed again: F202 health.Server.updates (health/server.go:49),
  label text names the deferred cleanup at :109 that deletes only the stream.
- The headline separates: confirmed 14, confirmed but disputed by the verifier (reads
  N-INTENT) 5; by trigger: reproduced through the public API 3, grown only by calling the
  registration function itself 10, not checked 6.

## Problems
1. F202's trigger reads "grown only by calling the registration function itself
   (Server.Watch) ... a leak only if run-time input reaches that call". Server.Watch is the
   RPC handler: remote clients call it. The direct-call reading is applied to any unit that
   calls the writer, whether or not the writer is a start-up registration. Same for
   LoadStore.ReporterForCluster, Stat.NewTimer.
2. The final gate has 124 actions: 120 are passed stack columns of the allocation captures
   that tie to no finding (stack:...http2Server..., loopyWriter, writeQuota, lrsclient...),
   each its own action with its own ; 3 triggers; 1 queue.
   The captures were taken because the residual item now names them.
3. The model tried to stop 28 times (316 turns): 59 ranked candidates untargeted, 139 of 180
   minutes left, "each batch needs a hand-written driver".
