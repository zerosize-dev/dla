# Real plugin run tokio-5 (tokio-util 0.7.1, fourth frozen build, snapshot 56aa221), 2026-11-21

Started 22:41 UTC, 360 s wall, 70 turns, cost 1.19, Status COMPLETE, budget 1 h 17 min, 5 min
used, 7 harness runs, 15 series, 2 captures. Evidence: u36/tokio-5 (run dir deleted).

## The documented leak is confirmed
F2, site src/sync/cancellation_token.rs:252 (child_token), state confirmed, label L-OWNERSHIP,
growth `stack:child_token (src/sync/cancellation_token.rs:218)` 39072 bytes/dump (steady,
gate pass, E32, alloc-stack E31). Explanation written by the model from the measurement:
"child_token boxes a node at cancellation_token.rs:218 linked in the parent's child list; if
the middle token is dropped before its child, Drop never reaches the remover at
cancellation_token.rs:129 (refcount != 0), so the node leaks."

How: first pass H1 flat; focus H2; second look H3 (targets F7, F3, F4) with the generated
skeleton: the model used all five slot variables (depth, ending, when, order, outcome), "the
units took 144 combinations of 5 choices"; heap grew; --capture alloc and summarize
alloc-stack-diff named child_token:218; the model added H4 for F2, alloc-test, refute sibling,
confirm.

## Left over
- Trigger "not checked (no harness series that grew is tied to it)".
- Verifier DISAGREES on site (:252 vs :205, the fn line), holder kind and label.
- Residual: 7 series above threshold (heap about 88 bytes per iteration unattributed), see O2.
