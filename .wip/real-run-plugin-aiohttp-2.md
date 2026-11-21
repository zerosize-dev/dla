# Real plugin run aiohttp-2 (2026-11-21 03:13 to about 05:14 UTC), queue build (= main 6a7ee29 without the later fixes)

Plugin copy of the working tree at 03:12, binary = the queue build. Model per run.py default,
74 turns, about 2 h wall (most of it harness loops), budget 3 h auto, 1 h 32 min used.

What happened
- Static stage: 187 candidates, 92 ranked. Batch 1 (H1, 8 targets): the agent installed the
  runtime dependencies with `pip install --target $TMPDIR/deps` (PYTHONPATH), filled the hooks,
  F8 `aiohttp/web_app.py:82` (the documented leak) and F9 `aiohttp/web_log.py:61` GREW; both
  confirmed about 5 minutes into the run (N-WARM sibling refuted by alloc-test).
- Then 14 more harnesses: one loopback client+server driver reused for the remaining batches
  (a shell loop over `dla harness gen --next`, fill, run, trend). Queue line of the report:
  "89 of 90 ranked candidates targeted in 15 harnesses: grew 1, executed 65, not reached 23;
  1 not targeted".
- `dla ledger close-rest` closed 104 non-ranked candidates early, the rest at the end.
- Labels (L-COLLECT), explanations, gates passed; then the verifier (blind, bundle only)
  DISAGREED on both: "reads as non-leak: N-INTENT ... gains one entry per new handler/app
  combination ... the harness varies the key on every unit, which real use doesn't do".
- Final answer to the user: "Report status is COMPLETE, but both confirmed findings are
  disputed and I don't think either is a leak that real use would trigger."

So the documented leak was measured and confirmed, and then talked down: the driver fed new
keys straight into the cached function, nothing showed that a caller of the public API can do
that (in the real leak, requests to paths with no route create a new handler object per
request), and the verifier judged by reading.

Defects seen
1. No step asks "who feeds the keys?": a holder that grows under direct injection is reported
   as a confirmed leak and then disputed; nothing reproduces it through the public entry path.
2. The report's `Queue:` line leaves out findings that left the candidate state (the two
   confirmed ones are not in "grew").
3. The `Residual:` header line has one clause per series (48 series) with unrounded floats.
4. `Coverage: python 3520 iterations, 0 of 0 entry points exercised` for hook drivers.
5. Final gate asked for a residual row per series: 176 items, cleared by a shell loop over
   `dla gap S<n> --residual` and canned `inconclusive` closes (busywork, no information).
6. `dla alloc-test H1 --finding F9` failed until PYTHONPATH was exported again (in the fix list).
7. Census noise in the queue: TypeVar assignments (`aiohttp.helpers._T`, `_MsgT`, `_RetType`).
8. The verifier put F9's site at line 75 (the writer), the finding says 61 (the declaration):
   a site disagreement that is only a declaration/writer difference.
