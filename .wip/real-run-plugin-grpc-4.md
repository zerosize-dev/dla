# Real plugin run grpc-4 (grpc-go v1.84.0, frozen build, snapshot 0dbf4c9), 2026-11-21

Started 16:54 UTC, 2502 s wall, 238 turns, cost 6.33, Status INCOMPLETE (663 open),
61 harness runs, 140 series, budget 3 h, 34 min used. Evidence: u36/grpc-4 (report, ledger,
gate, series, trend, queue, drivers of H13 H15 H16 H56; run dir deleted).

## The documented leak was found and confirmed
F204 health.Server.updates (health/server.go:49), state confirmed, growth
probe:health.Server.updates 1 count/iteration (steady, gate pass, E48), label text: "Watch's
deferred cleanup deletes the inner watcher entry but never the per-service outer map entry
(health/server.go:109), so each distinct watched service name leaves an empty map".
Trigger: reproduced through the public API in H13 (varies the service name in the
Health.Watch request). Verifier: agrees on site and holder, label L-COLLECT vs UNCLASSIFIED.
This is PR 9411.

## Problems
1. 22 findings "confirmed"; about 18 are start-up registries (balancer, resolver, codec,
   compressor, metric, certprovider registries, global dial/server options, a test helper)
   grown by a driver that calls the registration function with a new name per unit. The
   trigger line says "reproduced through the public API"; the blind verifier reads every one
   of them N-INTENT and the report prints DISAGREES, but the headline count stays 22.
2. The final gate said 810 missing items; the model tried to stop 11 times and gave up with
   146 of 180 minutes left ("a very large amount of further hand-written driver work").
   Real actions: 64 queue candidates in batches, 2 open items, 3 triggers, one residual
   command. 569 items are whole-process columns of about 150 trends that all wait for the
   same two open items; 137 residual items likewise. go.released is among the listed columns.
3. 42 ranked candidates never targeted; 31 "not reached", 14 "missed once, no focus harness".

## Fix
Pass 9 for P3 (after pass 8): gate items collapse to root actions; verifier N-reading splits
the confirmed count; a direct call of the registration function is not "reproduced through
the public API".
