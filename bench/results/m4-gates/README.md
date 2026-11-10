# M4 gates run

This run measures DLA after milestone M4 (orchestration and entry paths):
the router (repository, running process, heap dumps, symptom only), one worker
per runtime writing its own ledger slice, `dla ledger merge` and its
cross-worker gate, `dla summarize heap-graph` with dump-series confirmation,
attach backends, run locking, budgets and abort. It uses the twenty fixtures of
`../m3-gates` plus five new ones, three runs each, with the same model, limits
and sandbox as before, except that children may now write `/tmp` and use
subagents.

- **Date:** 2026-11-08
- **Ablation:** `gates` (the skill, the `dla` binary, the worker agent and the
  report; no hooks and no verifier), with archetypes on.
- **Model:** pinned, and recorded as `model` in each `run.json`.
- **Binary and skill:** built from commit 62c98cf. Later commits changed only
  tests, `own_uid` on macOS (attach only) and documentation.
- **Limits:** 80 turns and 1500 s per run, three runs in parallel, a 20 minute
  `dla` budget per child (`--child-budget`).
- **Cost:** $18.31 in total, $0.24 per run. Mean wall time was 178 s; the
  longest run took 943 s.
- **Grading:** `metrics.json` comes from `dla-bench grade`, and the Python
  grader produced an identical file. All 75 runs finished and none errored.
  69 runs were graded from their `dla` ledger. The other 6 recorded no
  finding at all: the symptom-no-target runs, and the
  jvm-rfq-dump-series-capped runs, whose summary passed no holder.

## Compared with M3

The first table is the twenty fixtures both milestones ran (the M4 numbers are
the same runs, graded with the new fixtures left out). The last column is all
twenty-five fixtures.

| Metric | No skill (M1) | Gates (M2) | Gates (M3) | Gates (M4), same 20 | Gates (M4), all 25 |
|---|---|---|---|---|---|
| Leak recall (confirmed at the right site or owner) | 11/30 | 0/30 | 29/30 | **30/30** | 36/39 |
| Archetype-free leak recall | 4/9 | 0/9 | 9/9 | 9/9 | 9/9 |
| Second-leak recall (multi-leak fixtures) | 3/3 | 0/0 | 3/3 | 3/3 | 3/6 |
| Flagged recall | 11/30 | 30/30 | 30/30 | 30/30 | 39/39 |
| Grounded attribution (right holder for a found leak) | 11/11 | n/a | 23/29 | **26/30** | 32/36 |
| False confirmation rate (controls and non-leaks) | 4/27 | 0/27 | 0/27 | **2/27** | 2/30 |
| Non-leak precision | 9/12 | 0/12 | 12/12 | 12/12 | 12/12 |
| Overclaim rate (impossible fixtures) | 0/6 | 0/6 | 0/6 | 0/6 | 0/12 |
| Trap miss rate | 12/15 | 15/15 | 1/15 | **0/15** | 0/18 |
| Trap false-alarm rate | 1/15 | 0/15 | 0/15 | 2/15 | 2/18 |
| Source overclaim rate (heap-dump fixtures) | | | | | 0/9 |

**Recall is complete on the M3 fixtures.** Every expected leak of the twenty
M3 fixtures is confirmed at the right site in every run, including the
rust-tree-parent-links run M3 missed: all three runs now name the `parent`
back-link. Holder attribution rose from 23/29 to 26/30.

**The regression: two false confirmations.** Two of the three
node-socket-heartbeat-cleared runs (the decoy control of the heartbeat leak)
confirmed the `Gateway.sessions` map (`src/gateway.js:15` and `:21`). The only
passing growth they cited was listener counts registered at `gateway.js:24`
and `:25`, a different site. The report says so under each finding: "Growth:
not tied to this finding". The ledger lets this through. Since M3,
`dla ledger confirm` prints a warning, not a refusal, when no cited passing
column is tied to the finding (by finding, site, owner or candidates).
Over the three benchmark runs made so far, untied confirmations were:

| Run | Untied confirmations | Of those, correct |
|---|---|---|
| M3 | 2 | 1 (node-socket-heartbeat, a timer column one line from the site) |
| M4, first run (below) | 4 | 1 (jni-order-codec's native leak, from `os.rss` alone) |
| M4, this run | 3 | 0 (the two false confirmations, and a jni-order-codec finding at `Native.kt:9`, which matches no leak) |

Making the tie a refusal would have prevented both false confirmations, and
it would have cost the two correct untied confirmations above. This decision
is left open for M5.

## The new fixtures

- **jni-order-codec** (Kotlin calling a Rust library through JNI, two leaks):
  all three runs used `dla run dispatch --workers jvm,rust`. Each started one
  worker subagent per runtime in one message, waited for both, ran
  `dla ledger merge` (both slices merged) and reported COMPLETE. The Kotlin
  leak (`Audit.trail`) is confirmed in 3 of 3 runs. The Rust native leak is
  flagged in 3 of 3 and confirmed in none: its growth shows only in the JVM
  process's resident memory, and the Rust worker has no workload of its own.
  The mechanical acceptance (`dla-bench dynamic`) lists the same leak as an
  expected failure.
- **jvm-rfq-dump-series** (three heap dumps, no source): 3 of 3 runs confirmed
  `quotes.RfqDesk.OPEN` by owner, with no source line, after refuting an
  N-WARM sibling with the dump-plateau test.
- **jvm-rfq-dump-series-capped** (the decoy: the same holder stops growing at
  the third dump): no claim in any run.
- **jvm-rfq-single-dump** (one dump): no claim in any run. Two runs ended
  with the holder `suspected_static_only` and one `unresolved`; two remedies
  ask for two more dumps of the same process, the third names the missing
  source instead.
- **symptom-no-target** (no source, nothing named): every run recorded the
  target as unresolved and stopped without a claim.

No run on a heap-dump fixture claimed a source line (source overclaim 0/9).

## How the runs used DLA

Counted from the transcripts:
- **Entry paths:** 63 runs started `--entry repository`, 9 `--entry artifact`
  and 3 `--entry symptom`.
- **Workers:** the 3 jni-order-codec runs dispatched 6 workers, all as blocking
  subagent calls, and merged both slices each time. No other run dispatched; the
  other fixtures have one runtime.
- **Heap dumps:** the 9 artifact runs each ran `dla summarize heap-graph` once
  over all their dumps.
- **Harness and analysis:** 61 runs ran `dla harness gen`, 56 `harness run`,
  55 `trend` and 59 `alloc-test`; 38 runs moved findings with
  `dla ledger confirm` and 57 with `dla ledger refute`.
- **Verifier:** 11 runs wrote a `dla bundle`. Under `gates` the verifier agent
  is removed, so the 8 attempts to start one failed and the runs went on
  without verification, as the skill says.
- **Not used:** `dla attach` and `dla run abort` (no fixture has a running
  process to attach to; benchmark children cannot see processes started
  outside their sandbox, see `docs/VERIFY.md`).

## The first M4 run

An earlier run of this benchmark (the same fixtures and limits, at commit
59a957e) found a defect in the new router, and was not kept.
- **What went wrong:** in 5 of 75 runs, all on repository fixtures whose prompt
  describes the service ("the edge gateway process") rather than naming a path,
  the model followed the skill's "never guess a target from the working
  directory". It opened a symptom run and stopped after three turns, with the
  source tree right there.
- **Its numbers:** leak recall 34/39, archetype-free recall 8/9, trap miss 2/18,
  no false confirmation (0/30), source overclaim 0/9.
- **The fix:** the router now takes the repository Claude was started in as the
  target, unless the request names another one (commit 62c98cf).
- **This run:** every repository fixture took the repository path.

## Notes

- **Unexplained growth share** reads 1.25 over 4 runs because two
  jni-order-codec runs measured a negative off-heap slope, which lowers the
  total while the residual is clamped at zero. The formula is on the M7
  calibration list with the other thresholds.
- **Labels** read 0/27 by construction until `dla ledger label` arrives (M6).
- **node-retry-scheduler** holders are still `container-field` in all three
  runs where the fixture expects `static/global` (the module singleton's field),
  as in M3.

## Reproduce the grading

The raw runs were deleted after grading (see "Keeping results" in `bench/README.md`);
`metrics.json` and `table.md` are the record. The commands below need the runs.


```
dla-bench grade bench/results/m4-gates --out /tmp/rust.json
python3 bench/runner/grade.py bench/results/m4-gates --out /tmp/py.json
dla-bench compare /tmp/rust.json /tmp/py.json
dla-bench table bench/results/m4-gates
```
