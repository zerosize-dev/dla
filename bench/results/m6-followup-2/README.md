# M6 follow-up run 2

This run checks the label-path fixes that the first follow-up run (`../m6-followup/README.md`,
"Failures") led to, committed as 3b8e732 and b91499d:
- the node retention records now name the holder of `--finding`, and `closure:` keys name their
  variable;
- label refusals name the producer of every closest alternative;
- an archetype with no row in the finding's runtime reference is refused unless `--note` says why;
- `dla ledger duplicate` merges duplicates of one leak;
- the heartbeat fixtures stop dispatching once a session is closed.

It re-runs the 8 fixtures those fixes touch, with the partner of each Node pair.

- **Arm:** gates, archetypes on; 3 runs per fixture, 24 runs.
- **Date:** 2026-11-11.
- **Model:** pinned, recorded as `model` in each `run.json`.
- **Binary and skill:** built from commit b91499d.
- **Limits:** as in M6.
- **Usage:** reported `total_cost_usd` 12.22 (0.51 per run; the API-equivalent value of the
  tokens, not a charge). Mean wall time 215 s, at most 472 s; no run timed out or errored.
- **Grading:** `metrics.json` comes from `dla-bench grade`; the Python grader produced an equal
  file.

## The same 8 fixtures in M6 and both follow-up runs

The first two columns regrade the M6 and follow-up-1 runs of these 8 fixtures.

| Metric | M6 | Follow-up 1 | Follow-up 2 |
|---|---|---|---|
| Leak recall | 19/21 | 21/21 | 21/21 |
| Archetype-free leak recall | 3/3 | 3/3 | 3/3 |
| Grounded attribution | 17/19 | 19/21 | 18/21 |
| Label accuracy | 10/16 | 14/18 | 13/18 |
| Forced-fit rate (archetype-free) | 3/3 | 3/3 | **0/3** |
| False confirmation rate | 1/6 | 1/6 | **0/6** |
| Trap miss rate | 2/12 | 0/12 | 0/12 |
| Trap false-alarm rate | 1/6 | 1/6 | **0/6** |

## The Node pairs (the M6 closing condition)

| Row | Pair | Leak found | Decoy claims | Labels |
|---|---|---|---|---|
| node L-CAPTURE | node-price-board / -flat | 3/3 | 0/3 | **3/3** L-CAPTURE (follow-up 1: 1/3) |
| node L-REGISTER | node-socket-heartbeat / -cleared | 3/3 | **0/3** (follow-up 1: 1/3) | 1/3 L-REGISTER |

Both Node rows pass their pairs: every leak run found the leak, and no decoy run claimed one.

## By fix

- **L-CAPTURE.** All three runs labelled the closure finding; the retention record of
  `summarize heap-graph --finding` named its holder.
- **Forced fits.** node-retry-scheduler was labelled UNCLASSIFIED in all three runs (follow-up 1:
  L-STALE in all three). The label of L-STALE on a Node finding was refused in each run for lack of
  a Node row, and the model then chose UNCLASSIFIED.
- **Duplicates.** `dla ledger duplicate` was used in 14 of 24 runs. python-report-memo merged its
  duplicates into the labelled finding and labelled L-COLLECT in all three runs (follow-up 1: 2/3).
- **L-UNLOAD.** jvm-extension-reload labelled L-UNLOAD in all three runs (follow-up 1: 2/3). Two runs
  name a holder other than the fixture's.
- **The heartbeat decoy.** No claim in three runs, after the second fix to its test double.

## Label misses (5 of 18)

- **jni-order-codec L1 (L-HANDLE), all three runs.** The leak is in the library's Rust JNI code
  (`native/src/lib.rs`), so the finding's runtime is rust, whose reference has no L-HANDLE row.
  - #2: `ledger label --as L-HANDLE` was refused; the refusal offered `--note` to keep the label,
    and the model chose UNCLASSIFIED.
  - #3: it took L-OWNERSHIP, a rust row.
  - #1: it chose UNCLASSIFIED when `--suggest` said L-HANDLE lacked alloc-stack evidence, without
    running the command the skill says to run once.
  - Follow-up 1, where this was a warning, had L-HANDLE in all three runs. The refusal trades this
    loss for the forced fits above.
- **node-socket-heartbeat #1 and #3 (UNCLASSIFIED for L-REGISTER).** The census gives the timer
  finding the holder `blocked-stack`. The evidence that observes the timer table is the
  `node-resources` capture, and a holder change accepts only heap-graph growth (`dla-hprof`,
  `dla-v8-snapshot`). L-REGISTER stays blocked. This is the same gap that makes the fixture an
  expected failure of the mechanical acceptance.

**INCOMPLETE reports:** 7 of 24 (the gates arm has no stop hook).

## Reproduce the grading

The raw runs were deleted after grading (see "Keeping results" in `bench/README.md`);
`metrics.json` and `table.md` are the record.

```
dla-bench grade bench/results/m6-followup-2 --out /tmp/rust.json
python3 bench/runner/grade.py bench/results/m6-followup-2 --out /tmp/py.json
dla-bench compare /tmp/rust.json /tmp/py.json
dla-bench table bench/results/m6-followup-2
```
