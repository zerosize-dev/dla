# node-retry-scheduler

Category `archetype-free` (no pair). Expected answer: a confirmed leak at
`src/scheduler.js:31` (alternatives: the `latest` field at line 15 and its
assignment at line 32), labelled `UNCLASSIFIED`, holder `static/global`, region
`heap`.

## Mechanism

`RetryScheduler` runs delivery attempts from a singly linked `PendingList`, walked
once per `tick()`. Every attempt is a plain record created by `enqueue()`
(`job`, `attempt`, `due`, `previous`, `next`). `enqueue()` sets the record's
`previous` field to the scheduler's `latest` field (line 31) and then stores the
record in `latest` (line 32), so each attempt keeps the attempt scheduled before
it and the scheduler keeps the newest one. The dispatcher process uses a single
scheduler for its whole life (`sharedScheduler()`, a module-level singleton), so
the newest attempt, and through the `previous` fields every earlier attempt with
the job and its payload (about 20 order rows) it carries, stays reachable.

No array, map, set, timer, listener or closure grows: the pending list unlinks a
node as soon as it runs, `inflight` is emptied, dead letters are capped at 100,
and attempts are records rather than closures. The retention is a linked chain
through the objects' own fields, anchored by one field of a long-lived object, so
it fits no collection, registration, closure or worker row of the taxonomy.
Growth tracks the number of jobs linearly (every job contributes its first
attempt, about 45% add a retry).

## Measurements

Node v22. Harness outside `project/` (`measure-retry.js`, kept with the
authoring notes): loads `project/load.js`, runs `runWorkload(N)` in a fresh
process, calls `global.gc()` twice, reads `heapUsed` relative to the heap
before the run and `scheduler.stats()`. Command:
`node --expose-gc measure-retry.js project N`.

| jobs | pending | inflight | retried | heap after GC (KB) | bytes per job |
|---|---|---|---|---|---|
| 2000  | 0 | 0 | 902   | 8383   | 4292 |
| 8000  | 0 | 0 | 3712  | 33428  | 4279 |
| 32000 | 0 | 0 | 14924 | 133595 | 4275 |

Slope between 2000 and 32000 jobs: (133595 - 8383) KB / 30000 = 4.18 KB per job,
linear; `pending` and `inflight` are 0 at every point, so the retained memory is
not work in progress.

Counterfactual (a scratch copy with the `this.latest = record;` line removed):
heap after GC is 41, 65 and 65 KB at 2000, 8000 and 32000 jobs, i.e. flat.

Workload: `node load.js --iterations 32000` runs in well under a second. Unit tests:
`node --test` (6 pass).
