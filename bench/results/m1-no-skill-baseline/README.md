# M1 no-skill baseline

The reference every later milestone is measured against: Claude Code with no
DLA plugin, asked to diagnose each fixture and record its conclusions in
`findings.json`. Twenty fixtures, three runs each, graded by both graders.

- **Date:** 2026-11-07
- **Ablation:** `no-skill`, archetypes on (no archetype files exist yet)
- **Model:** pinned for every run; recorded as `model` in each `run.json`
- **Limits:** 80 turns and 1500 s per run, three runs in parallel
- **Isolation:** every session ran inside the Claude Code Bash sandbox with
  no network, writes confined to its working copy, and the repository, the
  answer keys and the orchestrating session's files unreadable (see
  `bench/README.md`)
- **Cost:** $5.53 in total, $0.092 per run; 38.7 s mean wall time

Command:

```
dla-bench run --ablation no-skill --runs 3 --fixtures all --parallel 3 \
  --model <model id> --max-turns 80 --timeout 1500 --budget-usd 100 \
  --deny-read <the orchestrating session's own temp subdirectory> \
  --out bench/results/m1-no-skill-baseline
```

## Grading

`metrics.json` was written by `dla-bench grade`. `bench/runner/grade.py`
produced an identical file, compared as parsed JSON with `dla-bench compare`.
`table.md` renders it. All 60 runs finished; none errored.

## Reading the numbers

| Metric | Value |
|---|---|
| Leak recall (confirmed at the right site) | 11/30 = 0.367 |
| Archetype-free leak recall | 4/9 = 0.444 |
| Grounded attribution (holder right, among found) | 11/11 = 1.0 |
| Second-leak recall (multi-leak) | 3/3 = 1.0 |
| Overclaim rate (impossible fixtures) | 0/6 = 0.0 |
| False confirmation rate (controls and non-leaks) | 4/27 = 0.148 |
| Non-leak precision | 9/12 = 0.75 |
| Trap miss rate / trap false-alarm rate | 12/15 = 0.8 / 1/15 = 0.067 |

**Mechanisms right, lines wrong.** Every one of the 30 expected leaks was
confirmed by the model in the right file. Only 11 named a line inside the
site or an alternative site. In the other 19 the model was right about the
file and the mechanism and wrong about the line, often by naming a line far
beyond the end of a short file. That pattern matches numbering read off
`cat -n` output covering several files at once. No leak was missed outright.

| Leak runs (30) | Count |
|---|---|
| Confirmed at the right site | 11 |
| Confirmed in the right file, wrong line | 19 |
| Not confirmed in the right file | 0 |

DLA records sites from tool output, so this is where later milestones are
expected to move the numbers; the recall metric is deliberately strict about
the line.

**False confirmations.** `rust-batch-buffers` (buffers that keep their
capacity after `clear()`, a high-water mark and not a leak) was reported as a
confirmed leak in all three runs, and the decoy control
`node-socket-heartbeat-cleared` once.

**Not measured in this ablation.** `label_accuracy` and `forced_fit_rate`
need taxonomy labels, which a run without the skill does not give;
`unexplained_growth_share` needs a coverage manifest from `dla`.

## Environment notes

- JVM attach tools (`jcmd`, `jmap`) do not work inside the sandbox, because
  the JVM writes its attach and performance files to `/tmp`, which the
  sandbox keeps read-only. No run in this baseline used them; the JVM
  fixtures' drivers print heap use after GC. This must be fixed before DLA's
  dynamic stages use `jcmd` (see `docs/VERIFY.md`).
- In `jvm-billing-private-repo` Maven also fails to write a lock file to
  `/tmp`; that fixture cannot build here in any case.
- A first attempt at this baseline was discarded. It had denied the
  sessions all of Claude Code's shared temporary directory, which broke
  every Bash call's state tracking and tools such as valgrind while the runs
  still looked healthy. `dla-bench run` now refuses such a path.

## Reproduce the grading

The raw runs were deleted after grading (see "Keeping results" in `bench/README.md`);
`metrics.json` and `table.md` are the record. The commands below need the runs.


```
dla-bench grade bench/results/m1-no-skill-baseline --out /tmp/rust.json
python3 bench/runner/grade.py bench/results/m1-no-skill-baseline --out /tmp/py.json
dla-bench compare /tmp/rust.json /tmp/py.json
dla-bench table bench/results/m1-no-skill-baseline
```

Transcripts and child stderr are kept locally and are not committed.
