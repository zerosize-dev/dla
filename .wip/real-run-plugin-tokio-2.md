# Real plugin run tokio-2 (frozen build, snapshot 0dbf4c9), 2026-11-21

Started 16:59 UTC, 78 s wall, 17 turns, cost 0.33, Status COMPLETE, "no leak confirmed",
budget 1 h 17 min (auto), 1 min used. Evidence: u36/tokio-2 (build output trimmed).

- One harness (H1), 9 points, fresh and repeated. Unit: ReusableBoxFuture new/set/poll/drop;
  CancellationToken::new + one child_token + cancelled() future + cancel for key % 3 == 0 +
  drops in two orders; spawn_pinned. No grandchild token, so the documented leak (grandchild,
  then parent.cancel()) was never triggered.
- native.live_bytes flat (94367 at 7 of 9 points), os.rss +4 KB per point (page noise, passed
  the gate; trend next[0] = harness run --capture alloc, not followed).
- All 7 ranked candidates read executed -> queue empty -> close-rest -> gate -> COMPLETE.
- The candidates at the documented lines are tier 5: F1 cancellation_token.rs:156
  (CancellationToken.new, score -52) and F2 :252 (child_token, -52); F3 :294 (register) was rank 4.

Cause: a flat reading of one arbitrary unit settles a candidate for good, whatever budget is
left. Fix: brief wip-notes/second-look-brief.md (builder G, scratch u50).
