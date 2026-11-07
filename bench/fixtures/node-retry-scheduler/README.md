# node-retry-scheduler

Category `archetype-free` (no pair). Expected answer: a confirmed leak at
`src/scheduler.js:58`, labelled `UNCLASSIFIED`, holder `closure/continuation`,
region `heap`.

## Mechanism

`RetryScheduler` runs delivery attempts from a singly linked `PendingList`, walked
once per `tick()`. When an attempt fails and attempts remain, `attempt()` builds a
retry closure (`() => this.attempt(job, attempt + 1)`), appends it to the
scheduler's `issued` array (line 58) and links a new pending node that runs the
same closure. Pending nodes are unlinked as soon as they run, and `inflight` and
`deadLetters` are cleaned (dead letters are capped at 100), but `issued` is only
appended to; `stats()` reads its length. Each closure captures the job and its
payload (about 20 order rows), so every retry ever scheduled stays reachable from
the long-lived scheduler. Fixed share of jobs retry (about 45% at the default
flaky share), so growth tracks job count linearly.

Ambiguity note for graders: the code is shaped like a collection that only gets
appended to, which a taxonomy could also read as `L-COLLECT`; FIXTURES.md fixes the
expected label as `UNCLASSIFIED` with holder `closure/continuation`, which is what
the key says.

## Measurements

Node v22. Harness outside `project/` (`measure-retry.js`): runs `runWorkload(N)`
exported by `load.js`, calls `global.gc()` twice, reads `heapUsed` relative to the
heap before the run and `scheduler.stats()`. Command (from this directory):
`node --expose-gc measure-retry.js project N`.

| jobs | pending | inflight | retried | issued | heap after GC (KB) | bytes per job |
|---|---|---|---|---|---|---|
| 2000  | 0 | 0 | 902   | 902   | 2926  | 1498 |
| 8000  | 0 | 0 | 3712  | 3712  | 11860 | 1518 |
| 32000 | 0 | 0 | 14924 | 14924 | 47176 | 1510 |

Slope between 2000 and 32000 jobs: (47176 - 2926) KB / 30000 = 1.51 KB per job (about
3.3 KB per retry closure), linear; `pending` and `inflight` are 0 at every point,
so the retained memory is not work in progress.

Counterfactual (a scratch copy with the `this.issued.push(retry)` line removed):
heap after GC is 39, 61 and 63 KB at 2000, 8000 and 32000 jobs, i.e. flat.

Workload: `node load.js --iterations 32000` runs in 0.3 s. Unit tests:
`node --test` (5 pass).
