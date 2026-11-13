# node-report-heap

Category `non-leak` (cause `N-HEADROOM`, no pair). Expected answer: no confirmed
leak; the heap that looks high is garbage that the collector has not yet reclaimed,
and the growth is ruled out because the heap after a forced collection is flat.

## Mechanism

`reports.build` makes a month-end report from 40000 ledger lines: it copies every
line into per-category groups and sorts a second copy for the running balance
(`src/aggregate.js`). All of these objects are garbage once the report text is
rendered, but V8 does not collect after every report: the heap in use rises by about
15 MB per report until a collection runs, so it swings between 27 and 107 MB, and the
process keeps the resident memory it needed for the peak. `npm start` (`package.json`) runs the
service with `--max-old-space-size=3072`, a large old-space limit; the limit does not
change how much is live. What the service keeps is the last 24 rendered reports as
strings (`src/reports.js`, a `Map` that evicts its oldest entry past 24).

Control for the `Map`: its size is 24 from the 24th report on.

## Measurements

Node v22.23.4. Harness outside `project/`:

- `measure.js`: requires `load.js`, calls `global.gc()` twice, runs `runWorkload(N)`,
  calls `global.gc()` twice and reports `heapUsed` relative to the start.
- `pregc.js`: runs the workload in chunks of 5 reports and prints `heapUsed` after
  each chunk without forcing a collection.

Command: `node --expose-gc measure.js project N`.

| reports | heap after GC (KB, relative to start) | resident memory (MB, `node load.js`) |
|---|---|---|
| 60  | -665 | 212 |
| 240 | -664 | 243 |
| 480 | -663 | 242 |

`pregc.js` over 100 reports (without forced collections), heap in use per chunk of
5 reports in MB: 27, 105, 98, 79, 65, 57, 107, 69, 36, 27, 86, 52, 86, 52, 85, 79,
52, 89, 52, 86. After a forced collection the heap in use is 3.6 MB at 25 reports
and 2.9 MB at 50, 75 and 100 reports. The same run with
`--max-old-space-size=3072` gives the same figures.

Workload: `node load.js --iterations 60` runs in about 2.3 s (`--iterations 480` in
18 s). Unit tests: `node --test` (5 pass).

## Dynamic result

`dla-bench dynamic` (pinned binaries), archetypes on and off: pass. The census
candidates are refuted by the plateau test and nothing is confirmed; the rule-out is
recorded.
