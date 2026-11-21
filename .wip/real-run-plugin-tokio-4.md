# Real plugin run tokio-4 (third frozen build, snapshot 89bc8a8), 2026-11-21

Started 21:10 UTC, 1370 s wall, 47 turns, cost 0.66, Status COMPLETE, "no leak confirmed",
budget 1 h 17 min, 22 min used. Queue: 7 of 7 ranked targeted in 4 harnesses, executed 7,
second look 7 (grew 0). Evidence: u36/tokio-4 (run dir deleted).

- The model wrote all three second-look units with choices (H2: 4 choices, H3: 4, H4: 2).
- H3 (F5, F6, F7, the cancellation-token candidates): chain of depth 1..4 under a long-lived
  parent, waiters spawned on every level, cancel at a chosen position or abort or nothing,
  drop order forward or reversed. Every token stays alive in a vector until after the cancel.
- Left out: the recipe's slot "release of the intermediates: at once (a temporary), before
  the ending call, after it". The documented leak needs a middle token dropped before the
  first token is cancelled.

Cause: the recipe is prose, the model takes the slots it likes. Fix: M2 for G (scratch u52):
the unit hook of a second look is a generated skeleton with one named variable per slot, and
harness run refuses while a slot variable is unused (a slot can be given up explicitly, at
most two, and is then listed).
