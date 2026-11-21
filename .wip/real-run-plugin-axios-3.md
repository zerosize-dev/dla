# Real plugin run axios-3 (axios 1.15.1 with its runtime deps), 2026-11-21

Binary: 13:23 working-tree build (snapshot b6d36fd). Started 13:54 UTC, 5615 s wall, 118 turns,
Status COMPLETE, "no leak confirmed". Evidence: scratch u36/axios-3 (run.json, transcript,
project/.dla, lib).

## The leak was measured and not reported
H1 (first harness, plain sequential axios.get against a local server; Node's default agent
keeps connections alive) reproduced the documented leak:
- node.listeners@lib/adapters/http.js:924 slope exactly 1.0 per iteration, noise 0 (S1, S20, S22, S23)
- node.listeners 18 per iteration; node.heap_used (after GC) about 15 KB per iteration, fresh and repeated
- detector hit in the run file: MaxListenersExceededWarning, 11 error listeners added to [Socket] (site null)
trend: passed [], failed 64; every growing column has gate_failed ["n < 8"] only.

## Causes (all in the tool)
1. All 23 series have 4 to 7 points after warm-up: first H1 run cut by the 300 s time limit
   (8 and 7 of 9 points), later the model used --iterations 3..5. trend says nothing about
   columns that fail only for the point count, and such a series still settles targets as
   "executed".
2. A driver with open servers never exits: killed at the limit, no coverage data.
3. The detector hit has no site and is printed nowhere the model reads (harness run output
   was read with head -c 200).
4. F23 (census finding at http.js:924, registration-table) is tier 5, score -51: writers []
   gives w_none -40 and tier 5; never queued; closed "not targeted".
5. 30 above-threshold residual rows closed in a loop as unresolved; the open item text offers it.

## Checked by hand afterwards (same run dir, same binary)
dla harness run H1 --mode fresh (default length): 92 s, 9 points; dla trend S24 passes
node.listeners@lib/adapters/http.js:924 (slope 1.0, candidates F23, F22, F24), :717 (7.0),
:925, :971 and prints "dla ledger add --sibling-of F23 ...".

## Fix
Brief wip-notes/short-series-brief.md (S1 to S6), builder N (agent a747f7bdc3d1ef293), scratch u46.
