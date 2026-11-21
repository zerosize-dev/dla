# Real plugin run tokio-3 (second frozen build, snapshot 167233a), 2026-11-21

Started 18:55 UTC, 226 s wall, 31 turns, cost 0.47, Status COMPLETE, "no leak confirmed",
budget 1 h 17 min, 3 min used. Queue: 7 of 7 ranked targeted in 3 harnesses, executed 7,
second look 2 (grew 0). Evidence: u36/tokio-3 (run dir deleted).

- The second look worked as built: asked for after the first pass; harness run refused the
  missing VARIES.txt lines (twice) and hooks equal to H3; the model then wrote a different unit.
- H4 (second look, targets F1 and F5): root kept in state; c1 = root.child_token();
  c2 = c1.child_token(); c3 = root.child_token(); wait briefly; c1.cancel(); await
  c2.cancelled(); drops. One fixed sequence for every unit. native.live_bytes flat.
- The documented leak needs: grandchild made through a temporary middle token (dropped at
  once), then the FIRST token cancelled. Same operations, other order.

Cause: a second look that repeats one hand-picked sequence is one more sample. Fix: brief
wip-notes/choices-brief.md (builder G, scratch u52): units take their sequence from the key.
