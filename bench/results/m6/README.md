# M6 run

This run measures DLA after milestone M6 (archetypes, gates and verifier):
- `data/archetypes.toml` and one `<lang>-archetypes.md` table per runtime, readable only after
  `dla gate check --stage classify` passes.
- Ledger schema 6: `dla ledger label`, `strong` and `explain`, each checked against the evidence.
- `dla gate check --stage <s> | --final`, integrity seals, and the hooks (write, read and stop).
- `dla intervene`, a blind verifier bundle, and the final SKILL.md.

It has three arms over the 34 fixtures (the 25 of `../m5-gates` plus nine new archetype pairs):

| Arm | Plugin variant | Runs per fixture | Runs |
|---|---|---|---|
| gates, archetypes on | skill, `dla`, worker agent; no hooks, no verifier | 3 | 102 |
| gates, archetypes off | the same without archetype tables, known leaks and archetype-marked text | 3 | 102 |
| full, archetypes on | gates plus the hooks and the verifier agent | 1 | 34 |

- **Date:** 2026-11-10
- **Model:** pinned, recorded as `model` in each `run.json`.
- **Binary and skill:** built from commit efc97f5.
- **Limits:**
  - 80 turns and 1500 s per run, with three runs in parallel.
  - A 20 minute `dla` budget per child (`--child-budget`).
  - A usage stop at 130 (`--budget-usd`).
- **Usage:** Claude Code reports `total_cost_usd` per run. It is the API-equivalent value of the tokens; the children used a subscription login, so it is usage, not a charge.

  | Arm | Reported usage (USD) | Per run | Mean wall time | Turns per run |
  |---|---|---|---|---|
  | gates, archetypes on | 40.25 | 0.39 | 212 s | 25.3 |
  | gates, archetypes off | 37.52 | 0.37 | 190 s | 23.9 |
  | full, archetypes on | 14.51 | 0.43 | 277 s | 24.9 |
  | total | 91.93 | | | |

- **Grading:** `metrics.json` comes from `dla-bench grade`. The Python grader produced an equal file.
- **Timeouts:** three runs timed out at 1500 s and are kept as results, not retried.
  - python-report-memo-bounded, gates on #2 and gates off #1.
  - go-buffer-pool, full #1.
  - See "Failures".

## Compared with M5 (the 25 fixtures both runs share)

M5's runs are regraded here with the current fixtures. native-frame-decoder now has a decoy pair,
so M5's trap denominators read 21 and 18 rather than the committed 18 and 18.

| Metric | M5 gates | M6 gates, on | M6 gates, off | M6 full, on (1 run) |
|---|---|---|---|---|
| Leak recall | 39/39 | 39/39 | 39/39 | 13/13 |
| Archetype-free leak recall | 9/9 | 9/9 | 9/9 | 3/3 |
| Second-leak recall | 6/6 | 6/6 | 6/6 | 2/2 |
| Grounded attribution | 33/39 | **38/39** | 33/39 | 13/13 |
| Label accuracy | 0/30 | **26/30** | 0/30 | 9/10 |
| Forced-fit rate (archetype-free) | 0/9 | 3/9 | 0/9 | 0/3 |
| False confirmation rate | 0/30 | 1/30 | 0/30 | 0/10 |
| Non-leak precision | 12/12 | 12/12 | 12/12 | 3/4 |
| Overclaim rate (impossible fixtures) | 0/12 | 0/12 | 0/12 | 0/4 |
| Trap miss rate | 0/21 | 0/21 | 0/21 | 0/7 |
| Trap false-alarm rate | 0/18 | 1/18 | 0/18 | 0/6 |
| Source overclaim rate | 0/9 | 0/9 | 0/9 | 0/3 |

**Attribution:** grounded attribution rises from 33/39 to 38/39 with the archetype material. The
off arm stays at M5's 33/39.
- The two Node holders M5 missed are now right in most runs.
- **Labels:** M5 could not label, so its label accuracy was 0 by construction. M6 labels 26 of 30
  correctly.
- **The costs:** three forced fits and one false confirmation, both explained below.

## All 34 fixtures

| Metric | gates, on | gates, off | full, on (1 run) |
|---|---|---|---|
| Leak recall | 49/51 | 48/51 | 16/17 |
| Archetype-free leak recall | 9/9 | 9/9 | 3/3 |
| Second-leak recall | 6/6 | 6/6 | 2/2 |
| Grounded attribution | 47/49 | 41/48 | 15/16 |
| Label accuracy | 32/40 | 0/39 | 11/13 |
| Forced-fit rate | 3/9 | 0/9 | 0/3 |
| False confirmation rate | 1/45 | 0/45 | 0/15 |
| Non-leak precision | 12/12 | 12/12 | 3/4 |
| Overclaim rate | 0/12 | 0/12 | 0/4 |
| Trap miss rate | 2/33 | 3/33 | 1/11 |
| Trap false-alarm rate | 1/33 | 0/33 | 0/11 |
| Source overclaim rate | 0/9 | 0/9 | 0/3 |

## M6 acceptance

1. **Each shipped archetype row passes its decoy and control pair.** Not met for two of eleven rows,
   on the gates/archetypes-on arm with 3 runs per pair:

   | Row | Pair | Leak found | Decoy claims | Passes |
   |---|---|---|---|---|
   | jvm L-COLLECT | jvm-order-cache, jvm-rfq-dump-series | 6/6 | 0/6 | yes |
   | jvm L-TLS | jvm-invoice-render | 3/3 | 0/3 | yes |
   | jvm L-UNLOAD | jvm-extension-reload | 3/3 | 0/3 | yes |
   | go L-WORKER | go-subscription-workers | 3/3 | 0/3 | yes |
   | node L-REGISTER | node-socket-heartbeat | 3/3 | **1/3** | no |
   | node L-CAPTURE | node-price-board | **1/3** | 0/3 | no |
   | python L-COLLECT | python-report-memo | 3/3 | 0/3 | yes |
   | rust L-CYCLE | rust-tree-parent-links | 3/3 | 0/3 | yes |
   | rust L-OWNERSHIP | rust-meter-callbacks | 3/3 | 0/3 | yes |
   | native L-HANDLE | native-frame-decoder | 3/3 | 0/3 | yes |

2. **Recall with archetypes on is at least recall with them off.** Met:
   - overall, 49/51 against 48/51;
   - on archetype-free fixtures, 9/9 against 9/9;
   - on the shared 25 fixtures, 39/39 against 39/39.
3. **Read and write gates are proven by tests that try the blocked actions.** Met.
   - `crates/dla-cli/tests/gate_hooks.rs` and `gate_flow.rs` try each blocked action.
   - A live check ran on Claude Code 2.1.321 (`docs/VERIFY.md`, "Gates and hooks (M6)").
   - In this run, the stop hook blocked in three full-arm children (1, 1 and 3 attempts).

## Failures

Each failure below is traced from its run's transcript and ledger. The fix list is in "Open items".

**node-price-board (L-CAPTURE), 2 of 3 runs missed.**
- Every run confirmed the right holder (`closure/continuation`) and mechanism.
- Two runs annotated the wrong line (`board.js:14`, the signature, and `board.js:10`, in the
  constructor). They had read the source with an unnumbered `cat`.
- The site check matches by line, so those runs count as misses.
- `dla ledger annotate --site` accepts any `file:line`.

**node-socket-heartbeat-cleared, 1 of 3 runs claimed a leak.** This is a fixture defect.
- The run wrote its own unit, which ends sessions from the server side.
- The fixture's loopback socket never emits `close` on `destroy()`, as a real `net.Socket` does, so
  the session really is retained there.
- The same test double is in the leak fixture of the pair.

**Forced fits (3/9), all node-retry-scheduler.**
- The finding is a pointer chain through an object's own field, and its expected label is
  UNCLASSIFIED.
- Its evidence met the structural rules of L-COLLECT (one run) and L-STALE (two runs).
- `dla ledger label --suggest` lists "met" rows first and never offers UNCLASSIFIED.

**Label misses (8 of 40 found leaks on the gates/on arm):**
- 3: the model chose UNCLASSIFIED after the first refusal, instead of running the command the
  refusal named.
- 2: `--suggest` hid L-UNLOAD behind its six-row cap.
- 1: `dla ledger label` accepted L-OWNERSHIP, which has no native row, on a C finding.
- 2: jni-order-codec's worker finding got no label decision at all.

**Timeouts (3):**
- **python-report-memo-bounded, twice.** The model planned 8 points of 3000 iterations each, with
  tracemalloc capture in both key modes, then ran a backgrounded `alloc-test --extra 3` past the
  runner's limit. Neither `harness run` nor `alloc-test` checks the run's remaining budget before it
  starts.
- **go-buffer-pool, full arm #1.** Two overlapping `dla alloc-test` commands (one backgrounded)
  appended evidence with the same ledger sequence number. Every later ledger command was refused
  (`bad-seq`). The model aborted the run, started again and ran out of time. This is a correctness
  defect in the ledger's append path.

**INCOMPLETE reports:** 16 of 102 (gates on), 18 of 102 (gates off), 1 of 34 (full).
- Most are jni-order-codec and a few large Rust and JVM runs. They ended with undispositioned
  candidates or untied whole-process columns.
- **With the stop hook:** the full arm, which has it, completed 32 of 34 against 85 of 102 on the
  gates arm. Its one stamp, jni-order-codec #1, names an untied `jvm.class:` column.

## The verifier (full arm)

The blind verifier ran for every confirmed leak-side finding: 22 of 22.

| Item | Value |
|---|---|
| Holder agreement | 20/22 |
| Label agreement | 15/22 |
| Site agreement (within 3 lines) | 7/22 |

- **Site:** the bundle no longer names the site. The verifier names the site from the allocation
  and retention frames, which often point at a caller or a different line of the same structure.
- **Disagreements:** each one opens an item in the ledger. The report shows both mechanisms.

## The goroutine leak profile

**Coverage:** every Go run's coverage row reads `goroutineleak: ran`. 6 of 9 per gates arm used
go1.27.0. In M5 it was blocked in every run.
- **What changed:** the runner now copies the host's checksum database cache into each child's
  module cache, so the toolchain switch verifies offline.

## Open items (fixes found by this run)

The fixes are measured in `../m6-followup/README.md` and `../m6-followup-2/README.md`.

1. **Ledger sequence race (correctness).** Concurrent `dla` writers must take the sequence number and
   append under one lock. This must be fixed in M6.
2. **Long commands and the time budget.** `harness run` and `alloc-test` should refuse a plan that
   cannot finish in the remaining budget, or stop at the deadline with a partial, labelled result.
3. **`ledger annotate --site/--writer`.** Refuse a line that is outside the file, blank or a comment.
   Warn when the line does not contain the holder's identifier. The skill should say to take
   `file:line` from `grep -n`.
4. **`ledger label --suggest`.**
   - List every archetype with its missing kinds (no cap).
   - Always add an UNCLASSIFIED row with "check the summary against your mechanism".
   - Name the holder that blocks a row.
5. **`ledger label`.** Refuse or warn on an archetype that has no row for the finding's runtime.
6. **The final gate.** A confirmed leak-side finding needs a label decision (an ID, or UNCLASSIFIED
   with a note).
7. **The skill's UNCLASSIFIED rule.** UNCLASSIFIED only when no archetype fits. When the best fit is
   refused for missing evidence, run the named command once and retry.
8. **Fixtures.** In node-socket-heartbeat and node-socket-heartbeat-cleared, the loopback socket's
   `destroy()` must emit `close`. Re-measure both READMEs.
9. **Node reference.** Timers, listeners and subscriptions are `registration-table` holders.

## Reproduce the grading

The raw runs were deleted after grading (see "Keeping results" in `bench/README.md`);
`metrics.json` and `table.md` are the record. The commands below need the runs.


```
dla-bench grade bench/results/m6 --out /tmp/rust.json
python3 bench/runner/grade.py bench/results/m6 --out /tmp/py.json
dla-bench compare /tmp/rust.json /tmp/py.json
dla-bench table bench/results/m6
```
