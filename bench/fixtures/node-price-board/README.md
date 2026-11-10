# node-price-board

Category `archetype-leak` (trap half of a pair, partner `node-price-board-flat`).
Cause `L-CAPTURE`, holder `closure/continuation`, region `heap`.

## Mechanism

`Board.publish` in `src/board.js` builds a snapshot per update and stores it in the
field `latest` (line 7 initialises it, line 31 assigns it). Inside `publish`, two
closures are created in the same function scope: `changeNote` (line 16), which reads
`previous`, the snapshot that was current before this one (line 15), and the
snapshot's `describe` method (line 26), which reads
`symbol`, `snapshot` and `ticks`. V8 allocates one context object per function
invocation for every variable that any inner closure captures, and every closure
created in that invocation shares it. `describe` is stored on the new snapshot, so
it keeps the shared context alive, and that context holds `previous`. Each snapshot
therefore keeps the one before it, with its ticks and averages (about 10 KB), and the
chain from `latest` back to the first update is never released. `changeNote` itself
is called only when `options.verbose` is set, which the service never does.

Background: closures keep their lexical environment alive (MDN,
https://developer.mozilla.org/en-US/docs/Web/JavaScript/Closures); the shared-context
effect for a function that is never called was described in a Meteor engineering
post ("An interesting kind of JavaScript memory leak",
https://blog.meteor.com/an-interesting-kind-of-javascript-memory-leak-8b47d2e7f156;
the page returned HTTP 403 to the fetch tool of the build environment, so it was not
read here; the code in this fixture is original).

Expected site `src/board.js:15-17` (the captured `previous` and the closure that
reads it); alternatives `src/board.js:7` (the field), `src/board.js:26-28`
(`describe`) and `src/board.js:31` (`this.latest = snapshot`). Both fixtures of the pair share the same `task.prompt`.

## Measurements

Node v22. Harness outside `project/` (`measure.js`): requires `load.js`, calls
`global.gc()` twice, runs `runWorkload(N)`, calls `global.gc()` twice and reports
`heapUsed` relative to the heap before the run. Command:
`node --expose-gc measure.js project N`.

| updates | heap after GC (KB) |
|---|---|
| 500   | 4965 |
| 2000  | 19766 |
| 8000  | 78982 |
| 32000 | 315798 |

Slope about 9.9 KB per update, linear in the iteration count. Workload:
`node load.js --iterations 2000`. Unit tests: `node --test` (5 pass).

## Dynamic result

`dla-bench dynamic --only node-price-board` (pinned binaries): the trap's leak is
confirmed and the decoy is not. A closure chain is invisible to the in-process probes
(they walk object properties, not closure contexts), so no passing growth column is
tied to the census finding for `Board.latest` before the summarize step. The
heap-graph summary over the 4 snapshots (gate pass) ties it by holder identity and
annotates its holder as `closure/continuation`. The policy then drives a finding that
a summarize call ties like one that a harness column ties: it adds the N-WARM sibling,
tests the finding, refutes the sibling with the `favors leak` discriminator of the
heap-graph call and confirms the finding citing the call's growth evidence. A finding
that is already refuted is never driven again. The decoy (`node-price-board-flat`) has
no passing column and nothing is confirmed.

## Minimal edit to the control

See `node-price-board-flat/README.md`.
