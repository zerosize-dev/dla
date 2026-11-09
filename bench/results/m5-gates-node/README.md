# M5 follow-up: the two Node fixtures

This run re-measures the two fixtures whose holders the M5 gates run missed (`../m5-gates`), after the
holder fix of commit 6c65258.
- Heap-graph now reports the retention root's holder kind: a timer or listener registration on the
  path, or a module variable at the root.
- Ledger schema 5 accepts `dla ledger annotate F --holder H --evidence E` when heap-graph growth
  evidence tied to F observed holder H.

The model, limits and sandbox are the same as `../m5-gates`; three runs per fixture.

- **Date:** 2026-11-09
- **Binary and skill:** built from commit 6c65258.
- **Cost:** $1.75 for 6 runs; mean wall time 76 s.
- **Grading:** `metrics.json` comes from `dla-bench grade`, and the Python grader produced an identical
  file once it applied ledger schema 5 holder corrections (the record's `replaced_holder`, now in
  `../../GRADING.md`).

| Fixture | Expected holder | M4 | M5 gates | This run |
|---|---|---|---|---|
| node-retry-scheduler | static/global | 0/3 | 0/3 | 1/3 |
| node-socket-heartbeat | registration-table | 2/3 | 0/3 | 2/3 |

Leak recall stays 6/6.

**node-retry-scheduler.**
- Run 1 confirmed `RetryScheduler.latest` and corrected its census holder from `container-field`
  to `static/global` with the new annotation, citing the per-point heap-graph growth evidence.
- Runs 2 and 3 confirmed the same finding without that correction. All three runs saw
  `observed_holder` in the heap-graph output.
- The skill tells the model to run the printed command; whether it does is up to the run.

**node-socket-heartbeat.** Two runs confirmed the timer finding at `src/heartbeat.js:15` with
`registration-table`, and one with `closure/continuation`.

Neither fixture is a clean pass yet. The next gates benchmark (M6) measures them again with the rest.
