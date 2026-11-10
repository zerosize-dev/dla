# M2 gates run

DLA after milestone M2 (probe, static sweep, DLA-owned rules, holder
census, workload discovery), measured on the same twenty fixtures, three
runs each, with the same model, limits and sandbox as the no-skill
baseline in `../m1-no-skill-baseline`.

- **Date:** 2026-11-07
- **Ablation:** `gates` (skill, `dla` binary and report; no hooks, no
  verifier), archetypes on
- **Model:** pinned; recorded as `model` in each `run.json`
- **Limits:** 80 turns and 1500 s per run, three in parallel
- **Cost:** $10.77 in total, $0.18 per run; 63.7 s mean wall time
- **Grading:** `metrics.json` from `dla-bench grade`; the Python grader
  produced an identical file. All 60 runs finished, none errored, and
  every run was graded from its `dla` ledger.

## Compared with the no-skill baseline

| Metric | No-skill (M1) | Gates (M2) |
|---|---|---|
| Flagged recall (a candidate at the right site) | 11/30 = 0.367 | 30/30 = 1.0 |
| Leak recall (confirmed at the right site) | 11/30 = 0.367 | 0/30 = 0.0 |
| Archetype-free leak recall | 4/9 | 0/9 |
| False confirmation rate (controls and non-leaks) | 4/27 | 0/27 |
| Non-leak precision (ruled out with a discriminator) | 9/12 | 0/12 |
| Overclaim rate (impossible fixtures) | 0/6 | 0/6 |
| Trap miss rate | 12/15 | 15/15 |

**What moved.** With the sweep and the census, every expected leak in all
60 runs sits in the ledger as a candidate at the right `file:line`. The
no-skill model named the right line in 11 of 30 cases.

**What cannot move yet.** M2 has no way to test, confirm or refute a
candidate: `dla ledger test`, `confirm` and `refute` arrive in M3. Every
run therefore ends with its candidates recorded as `unresolved` with reason
`tool_missing`, so leak recall, non-leak precision and the trap rates read
zero or one by construction. The meaningful comparison for confirmation
starts with the M3 run.

## How the runs used DLA

Counted from the transcripts: all 60 runs ran `dla run init`, `dla probe`,
`dla sweep` and `dla census`; 59 ran `dla workload discover` and finished
with `dla report`; 55 recorded writers and removers with
`dla ledger annotate`. Runs that reached for commands from later milestones
(`ledger test`, `confirm`, `gate check`) got a refusal and recorded the
stage as `unresolved(tool_missing)`.

## Environment notes

- Three runs tried `dla install` for a missing analyzer. The plugin data
  directory is outside the sandbox's writable area (and the sandbox has no
  network), so the install failed, the command was printed, and the backend
  stayed a recorded gap. Outside the benchmark sandbox `dla install` writes
  to that directory normally.
- CodeQL, Sonar and Error Prone show as missing on the JVM and Python
  fixtures (see `docs/VERIFY.md`).
- JVM attach tools still do not work inside the sandbox; M2 does not use
  them.

## Reproduce the grading

The raw runs were deleted after grading (see "Keeping results" in `bench/README.md`);
`metrics.json` and `table.md` are the record. The commands below need the runs.


```
dla-bench grade bench/results/m2-gates --out /tmp/rust.json
python3 bench/runner/grade.py bench/results/m2-gates --out /tmp/py.json
dla-bench compare /tmp/rust.json /tmp/py.json
dla-bench table bench/results/m2-gates
```
