# M6 follow-up run

This run checks the fixes for the open items of the M6 run (`../m6/README.md`, "Open items"),
committed as 1017550. It re-runs the fixtures those items came from, together with the partner of
each pair, so that every pair is measured whole:

| Fixture | Why |
|---|---|
| node-price-board, node-price-board-flat | L-CAPTURE pair; wrong lines annotated (open item 3) |
| node-socket-heartbeat, node-socket-heartbeat-cleared | L-REGISTER pair; the loopback double (item 8), Node holders (item 9) |
| node-retry-scheduler | forced fits (items 4 and 7) |
| jvm-extension-reload, jvm-extension-reload-stopped | L-UNLOAD hidden by the `--suggest` cap (item 4) |
| native-frame-decoder, native-frame-decoder-freed | L-OWNERSHIP accepted on a C finding (item 5) |
| jni-order-codec | the worker finding left unlabelled (item 6) |
| python-report-memo, python-report-memo-bounded | the timeouts (item 2) |
| go-buffer-pool | the ledger sequence race (item 1) |

- **Arm:** gates, archetypes on (skill, `dla`, worker agent; no hooks, no verifier); 3 runs per
  fixture, 39 runs.
- **Date:** 2026-11-11.
- **Model:** pinned, recorded as `model` in each `run.json`.
- **Binary and skill:** built from commit 1017550.
- **Limits:** as in M6 (80 turns and 1500 s per run, three in parallel, a 20 minute `dla` budget
  per child).
- **Usage:** reported `total_cost_usd` 18.70 (0.48 per run; the API-equivalent value of the
  tokens, not a charge). Mean wall time 188 s, 29.3 turns per run; no run timed out.
- **Grading:** `metrics.json` comes from `dla-bench grade`; the Python grader produced an equal
  file.

## Compared with M6 on the same 13 fixtures

The M6 column regrades M6's gates/archetypes-on runs of these 13 fixtures.

| Metric | M6 | Follow-up |
|---|---|---|
| Leak recall | 22/24 | **24/24** |
| Archetype-free leak recall | 3/3 | 3/3 |
| Second-leak recall | 3/3 | 3/3 |
| Grounded attribution | 20/22 | 22/24 |
| Label accuracy | 12/19 | **17/21** |
| Forced-fit rate (archetype-free) | 3/3 | 3/3 |
| False confirmation rate | 1/18 | 1/18 |
| Non-leak precision | 3/3 | 3/3 |
| Trap miss rate | 2/15 | **0/15** |
| Trap false-alarm rate | 1/15 | 1/15 |
| Timeouts | 1 | **0** |

## The archetype pairs

| Row | Pair | Leak found | Decoy claims | Labels |
|---|---|---|---|---|
| node L-CAPTURE | node-price-board / -flat | **3/3** (M6 1/3) | 0/3 | 1/3 L-CAPTURE |
| node L-REGISTER | node-socket-heartbeat / -cleared | 3/3 | **1/3** | 3/3 L-REGISTER |
| jvm L-UNLOAD | jvm-extension-reload / -stopped | 3/3 | 0/3 | 2/3 L-UNLOAD (M6 1/3) |
| native L-HANDLE | native-frame-decoder / -freed | 3/3 | 0/3 | 3/3 L-HANDLE (M6 2/3) |
| python L-COLLECT | python-report-memo / -bounded | 3/3 | 0/3 | 2/3 L-COLLECT |

jni-order-codec labelled both leaks in all three runs (L-HANDLE 3/3, M6 1/3; L-COLLECT 3/3).

## The open items of M6

1. **Ledger sequence race.** No `bad-seq` refusal in 39 runs; go-buffer-pool finished all three
   runs (286, 287 and 144 s; in M6 the race cost a run of the full arm).
2. **Long commands and the time budget.** python-report-memo-bounded finished in 405, 170 and
   290 s (M6: one timeout on this arm and one on the archetypes-off arm). No `time-budget` refusal and no partial run occurred, so the
   estimate never had to stop a command here.
3. **Site lines.** node-price-board found the leak at the right line in all three runs (M6 1/3).
   No `invalid-site` refusal occurred; the warning that a line lacks the owner's identifier fired
   47 times in 14 runs.
4. **`--suggest`.** Every row is listed; the closing UNCLASSIFIED row appeared in 21 runs.
   jvm-extension-reload saw L-UNLOAD in every run.
5. **A label without a row in the runtime's reference.** The warning fired in 10 runs and was
   followed in none: L-STALE on node (node-retry-scheduler, 3 runs), L-REGISTER on jvm
   (jvm-extension-reload #2). In jni-order-codec it fired wrongly: the C finding of the JVM worker
   was judged by the worker's runtime, not by its site.
6. **The final gate's label decision.** Every confirmed finding had a label decision before the
   gate asked for one.
7. **The skill's UNCLASSIFIED rule.** See "Label misses" below.
8. **The heartbeat fixtures.** The loopback double now emits `close`. The decoy was still claimed
   once: see below.
9. **Node holders.** node-socket-heartbeat grounded its holder in all three runs.

## Failures

**node-socket-heartbeat-cleared #3 claimed a leak, and the claim is true.** The run wrote its
own harness and found a second defect in the fixture's code: `Session.receive` went on dispatching
the frames of a chunk after one of them (`bye`) had closed the session, so a `subscribe` that
followed put the closed session into `Router.topics` after `close()` had released its topics.
Both fixtures of the pair now stop dispatching once the session is closed (with a unit test that
fails without the line).

**Label misses (4 of 21 labelled leaks, plus 3 forced fits):**
- **node-price-board #1 and #3 (UNCLASSIFIED for L-CAPTURE).** The model confirmed the finding
  for the captured closure variable `previous`. The node backend's retention records list only
  the three largest holders, never the holder of the finding named by `--finding`, and the label's
  holder match ignored `closure:` keys. The node holder keys also carried the path of the
  harness's working copy, and an owner set by hand could not be corrected.
- **jvm-extension-reload #2 (L-REGISTER for L-UNLOAD).** After the first refusal, the fix text named
  only the non-heap and trend producers. The model produced a class-loader record, which ties to no
  finding and which a confirmed finding can no longer cite, and then took a row `--suggest` listed
  as met. The retention route (`summarize heap-graph --finding`) was never named.
- **python-report-memo #3 (UNCLASSIFIED for L-COLLECT).** Eight confirmed findings described the
  one leak. One took L-COLLECT; the final gate's label item had the rest labelled UNCLASSIFIED,
  and the grader takes the first. The core ledger had no command to merge duplicates.
- **node-retry-scheduler, all three runs (L-STALE, expected UNCLASSIFIED).** The fixture's README
  argues against the collection, registration, closure and worker rows, but not against L-STALE,
  whose summary describes the mechanism (finished attempt records kept reachable through
  `previous`). This is recorded as open in `docs/VERIFY.md`; the fixture is unchanged.

**INCOMPLETE reports:** 8 of 39 (M6 on the same fixtures: 7 of 39). The gates arm has no stop hook.
jni-order-codec stays incomplete in every run on `jvm.class:*` columns, whose fix text
(`dla trend S<n>`) repeats.

The fixes for these misses are measured in `../m6-followup-2/README.md`.

## Reproduce the grading

The raw runs were deleted after grading (see "Keeping results" in `bench/README.md`);
`metrics.json` and `table.md` are the record.

```
dla-bench grade bench/results/m6-followup --out /tmp/rust.json
python3 bench/runner/grade.py bench/results/m6-followup --out /tmp/py.json
dla-bench compare /tmp/rust.json /tmp/py.json
dla-bench table bench/results/m6-followup
```
