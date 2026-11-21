# Real plugin run axios-5 (second frozen build, snapshot 167233a), 2026-11-21

Started 18:55 UTC, 2924 s wall, 100 turns, cost 1.97, Status COMPLETE, budget 3 h, 48 min used,
9 harness runs, 28 series. Evidence: u36/axios-5 (run dir deleted).

## The documented leak is confirmed
F27, site lib/adapters/http.js:924 (socket error listener), state confirmed, label L-REGISTER;
its own column node.listeners@lib/adapters/http.js:924 passes at 1.0 per unit (T3). The model
followed the detector line and the printed commands (alloc-test on F27 favoured leak).

## Problems
1. Six findings confirmed for one cause. F25 (:903), F26 (:908), F28 (:925) have their own
   site columns passing (listeners on the retained request objects: consequences). F14 (:393)
   and F18 (:730) have no column at their site; they were tied through
   node.listeners@lib/adapters/http.js:717 (req = transport.request(...), 7 runtime-internal
   listeners per unit), a column with 20 candidates because the census lists :717 as a
   "writer" of every registration on req. The report's Growth line of F27 quotes :717 too.
2. Trigger "not checked" on all of them; verifier DISAGREES on site (none) and label.
3. Headline: confirmed 6, confirmed but disputed by the verifier 2, suspected 17, unresolved 29.
4. Good: F3 (validator deprecatedWarnings) now measured by the .size probe and its trigger
   reads "not found" after two public-API workloads (keys come from the library's own schema).

## Fix
N resumed: per-site count columns tie and list candidates by own site only; the own-site
column is the one quoted.
