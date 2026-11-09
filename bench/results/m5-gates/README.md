# M5 gates run

This run measures DLA after milestone M5 (backends and attribution).
- The ledger refuses a leak confirmation whose cited growth is not tied to the finding (schema 4).
- `dla summarize` gives four operations: heap-graph, type-histogram-diff, alloc-stack-diff and
  non-heap. Thirty adapters sit behind them, covering JVM, Rust and native code, Node, Go and
  Python.
- `dla harness run --capture` and `--tool` capture the inputs; a series of harness heap captures
  is a per-point series over n.
- `dla correlate` handles leaks across a foreign-function boundary.

It uses the twenty-five fixtures of `../m4-gates`, three runs each, with the same model, limits and
sandbox.

- **Date:** 2026-11-09
- **Ablation:** `gates` (the skill, the `dla` binary, the worker agent and the report; no hooks and
  no verifier), with archetypes on.
- **Model:** pinned, and recorded as `model` in each `run.json`.
- **Binary and skill:** built from commit 6077247.
- **Limits:** 80 turns and 1500 s per run, three runs in parallel, a 20 minute `dla` budget per child
  (`--child-budget`), `--budget-usd 40` as a stop.
- **Cost:** $18.66 in total, $0.25 per run. Mean wall time was 183 s; the longest run took 1319 s
  (rust-batch-buffers, run 1).
- **Grading:** `metrics.json` comes from `dla-bench grade`, and the Python grader produced an
  identical file. All 75 runs finished and none errored.
  - 70 runs were graded from their `dla` ledger.
  - The other 5 recorded no finding at all: the three symptom-no-target runs, and two
    jvm-rfq-dump-series-capped runs, whose summary passed no holder.

## Compared with M4

| Metric | Gates (M4) | Gates (M5) |
|---|---|---|
| Leak recall (confirmed at the right site or owner) | 36/39 | **39/39** |
| Archetype-free leak recall | 9/9 | 9/9 |
| Second-leak recall (multi-leak fixtures) | 3/6 | **6/6** |
| Flagged recall | 39/39 | 39/39 |
| Grounded attribution (right holder for a found leak) | 32/36 | 33/39 |
| False confirmation rate (controls and non-leaks) | 2/30 | **0/30** |
| Non-leak precision | 12/12 | 12/12 |
| Overclaim rate (impossible fixtures) | 0/12 | 0/12 |
| Trap miss rate | 0/18 | 0/18 |
| Trap false-alarm rate | 2/18 | **0/18** |
| Source overclaim rate (heap-dump fixtures) | 0/9 | 0/9 |

**Recall is complete.** Every expected leak of the twenty-five fixtures is confirmed at the right
site or owner in every run.

The three leaks M4 missed are found:
- jni-order-codec's native leak (`native/src/lib.rs`). In all three runs the JVM worker ran
  `dla correlate`, which ties the allocator shim's native site to a candidate in the managed worker.
  The confirmation goes through the normal ledger rules.
- Its second-leak counterpart.

**No false confirmation, and no trap false alarm.** M4's two false confirmations were on the
node-socket-heartbeat-cleared decoy. Both cited growth that was not tied to the confirmed holder, and
the ledger now refuses exactly that (`growth-not-tied`). No run in this benchmark was refused for
it: every confirmation cited tied growth.

**Holder attribution: 33 of 39.** All six misses are the two Node fixtures:
- **node-retry-scheduler, all three runs: `container-field`.** The fixture expects `static/global`.
  The confirmed finding is `RetryScheduler.latest`, an instance field. The fixture names the kind of
  the root that holds it (the module variable `shared`). The census gives the field's own kind, and
  the ledger does not change a holder that is already set. This is the same miss as in M3 and M4.
- **node-socket-heartbeat, all three runs: `closure/continuation`.** The fixture expects
  `registration-table`. M4 got this right in 2 of 3 runs. The new Node heap-snapshot analysis
  classifies the variable the interval callback captures (`session`) as the holder. It does not see
  that the path runs through the timer registration (`Timeout._onTimeout` in the timer list), and the
  runs followed it. This is a regression of the M5 snapshot analysis: a registration on the
  retention path should outrank the closure variables below it.

## How the runs used DLA

Counted from the transcripts.

**Entry paths:** 63 runs started `--entry repository`, 9 `--entry artifact` and 3 `--entry symptom`.

**Harness and analysis:** 55 runs ran `dla harness run`, 54 `trend` and 54 `alloc-test`. 33 runs moved
findings with `dla ledger confirm` and 57 with `dla ledger refute`.

**Captures and summaries.** 40 runs captured with `--capture`, and the runs called these operations:

| Operation | Adapter | Calls | Runs |
|---|---|---|---|
| heap-graph | dla-hprof (JVM, including the artifact fixtures) | 23 | 13 |
| heap-graph | node-heapsnapshot (per-point series) | 10 | 5 |
| type-histogram-diff | jcmd-class-histogram | 6 | 4 |
| type-histogram-diff | dla-hprof-histogram | 4 | 2 |
| type-histogram-diff | py-types | 4 | 2 |
| alloc-stack-diff | go-pprof-heap | 14 | 7 |
| alloc-stack-diff | py-tracemalloc | 14 | 7 |
| alloc-stack-diff | node-heapprofile | 2 | 1 |
| non-heap | node-resources | 14 | 6 |
| non-heap | go-goroutine | 12 | 6 |

**Native fixtures:** native-frame-decoder confirmed through the allocator shim's site columns, with
no capture. The 3 native-vendor-codec runs (an impossible fixture with a vendor binary and no
source) ran `--tool valgrind` and made no claim. No run used heaptrack.

**Workers and correlation:** the 3 jni-order-codec runs dispatched the jvm and rust workers, merged
both slices, and ran `dla correlate` in the JVM worker.

**Goroutine leak profile: not measured here.** In every Go run the coverage row `goroutineleak`
reads `blocked`: "the goroutineleak profile needs Go 1.27 or newer (go1.24.7 here) and
GOTOOLCHAIN=local forbids switching to go1.27.0".
- The benchmark runner has set `GOTOOLCHAIN=local` for children since M1. The leak pass respects
  that setting, so the go1.27 toolchain the runner had linked into each child's module cache was
  never used.
- The mechanical acceptance (`dla-bench dynamic`, outside the runner's environment) does run it.
- The runner is changed after this run to `GOTOOLCHAIN=auto`, with `GOPROXY=off` as before, so the
  next benchmark measures it offline.

**Verifier:** 9 runs wrote a `dla bundle`. Under `gates` the verifier agent is removed, so the 8
attempts to start one failed and the runs went on without verification, as the skill says.

**Not used:** `dla attach` in 1 run only, and no `dla install` (11 runs read its help). Benchmark
children cannot attach to processes started outside their sandbox (`docs/VERIFY.md`).

## The smoke run

Before this run, the same build ran 4 fixtures once each: go-subscription-workers, jni-order-codec,
node-socket-heartbeat and python-ingest-two-leaks. It cost $1.45. Leak recall was 6 of 6 and
grounded attribution 6 of 6, including `registration-table` on node-socket-heartbeat. That holder
did not repeat in the three runs above. The smoke results are not kept.

## Notes

- **Labels** read 0/30 by construction until `dla ledger label` arrives (M6).
- **The mechanical acceptance** (`dla-bench dynamic`) on the same build: 22 of 25 pass, with 3
  expected failures (the two Node holders above and rust-tree-parent-links, whose `parent` back-link
  needs reading the code).
- **Cost:** wall time per run rose from 178 s to 183 s and cost per run from $0.24 to $0.25. The
  capture steps add little.

## Reproduce the grading

```
dla-bench grade bench/results/m5-gates --out /tmp/rust.json
python3 bench/runner/grade.py bench/results/m5-gates --out /tmp/py.json
dla-bench compare /tmp/rust.json /tmp/py.json
dla-bench table bench/results/m5-gates
```
