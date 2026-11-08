# M3 gates run

This run measures DLA after milestone M3 (the dynamic core): loop harnesses,
sampling, trend tests and the slope gate, alloc-test, the gap and residual
split, and the ledger rules for test, confirm and refute. It uses the same
twenty fixtures, three runs each, and the same model, limits and sandbox as
`../m1-no-skill-baseline` and `../m2-gates`.

- **Date:** 2026-11-08
- **Ablation:** `gates` (the skill, the `dla` binary and the report; no hooks
  and no verifier), with archetypes on.
- **Model:** pinned, and recorded as `model` in each `run.json`.
- **Limits:** 80 turns and 1500 s per run, three runs in parallel.
- **Cost:** $13.88 in total, $0.23 per run. Mean wall time was 185 s; the
  longest run took 877 s.
- **Grading:** `metrics.json` comes from `dla-bench grade`, and the Python
  grader produced an identical file. All 60 runs finished, none errored,
  and every run was graded from its `dla` ledger.

## Compared with M1 and M2

| Metric | No skill (M1) | Gates (M2) | Gates (M3) |
|---|---|---|---|
| Leak recall (confirmed at the right site) | 11/30 = 0.367 | 0/30 | **29/30 = 0.967** |
| Archetype-free leak recall | 4/9 | 0/9 | **9/9** |
| Second-leak recall (multi-leak fixture) | 3/3 | 0/0 | 3/3 |
| Flagged recall (a candidate at the right site) | 11/30 | 30/30 | 30/30 |
| Grounded attribution (right holder for a found leak) | 11/11 | n/a | 23/29 |
| False confirmation rate (controls and non-leaks) | 4/27 | 0/27 | **0/27** |
| Non-leak precision (ruled out with a discriminator) | 9/12 | 0/12 | **12/12** |
| Overclaim rate (impossible fixtures) | 0/6 | 0/6 | 0/6 |
| Trap miss rate | 12/15 | 15/15 | **1/15** |
| Trap false-alarm rate | 1/15 | 0/15 | 0/15 |

**Confirmation now works.** With harnesses and the M3 ledger rules, 29 of the
30 expected leaks end confirmed at the right `file:line`. No control or
non-leak fixture is confirmed. Every non-leak fixture ends with growth ruled
out by a discriminator, and neither impossible fixture reaches a claim. The
no-skill baseline confirmed 11 leaks, and confirmed 4 times on a fixture with
no leak.

**What M3's acceptance requires, and the benchmark shows:**
- **Non-leak fixtures end as refuted with discriminator evidence.** All 12
  non-leak runs do (fragmentation, a plateauing cache, a pool high-water mark,
  deferred collection), and so do the 15 decoy controls.
- **Leak fixtures reach confirmed from loop tests.** 29 of 30 leaks are
  confirmed, each from a growth series that passed the slope gate, with a site,
  a holder and a refuted non-leak sibling.
- **No fixture reaches confirmed when its harness cannot run.** None of the 6
  runs on the two impossible fixtures (private dependency, missing vendor
  header) claims anything.

**The one miss** is a rust-tree-parent-links run. The model confirmed the
`children` edge of the `Rc` cycle (`section.rs:11`). The fixture expects the
`parent` back-link (`section.rs:27`, or the alternatives `:10` and `:16-22`),
which is the edge a repair would make `Weak`. The growth and the cycle are
both real; the miss is which edge the finding names.

**Holder attribution** is right for 23 of 29 found leaks. Every miss is on the
two Node fixtures:
- **node-retry-scheduler:** the runs named the `latest` field of the module
  singleton a `container-field`. The fixture expects `static/global`.
- **node-socket-heartbeat:** the runs named the interval timer
  `closure/continuation` or `blocked-stack`. The fixture expects
  `registration-table`.

**Labels** read 0/20 by construction, because `dla ledger label` arrives in M6.
Every found leak stays `UNCLASSIFIED` until then.

**Unexplained growth share** is 0.52 over the 8 runs that used
`dla gap --residual`. The residual subtracts only growth measured in bytes. A
finding confirmed from a count (entries, goroutines or timers per site)
contributes nothing, so most of the heap growth stays "unexplained" even when
the leak is confirmed. The threshold and the attribution of counts are on the
M7 calibration list.

## How the runs used DLA

Counted from the transcripts:
- **Harness:** all 60 runs ran `dla harness gen`, and 58 ran `dla harness run`.
  The other two were native-vendor-codec runs: after the probe showed the
  build failing on the missing vendor header, they resolved the candidates
  as `suspected_static_only` or `unresolved` (`build_fails`) without trying
  the harness.
- **Analysis:** 54 runs ran `dla trend` and 58 ran `dla alloc-test`.
- **Ledger:** 45 runs moved findings with `dla ledger test`, 41 with
  `dla ledger confirm` and 54 with `dla ledger refute`; 54 annotated
  holders or sites.
- **Rarer commands:** 14 runs used `dla gap`, 8 of them with `--residual`.
  4 used `dla sample`, 1 used driver mode and 1 used `trend --against`.
- **Report:** every run finished with `dla report`.
- **Gate check:** runs that called `dla gate check` (it arrives in M6) got a
  refusal and recorded the stage as `unresolved(tool_missing)`.

## Grading note

M3 lets `dla ledger annotate` set a holder or a site the finding was added
without. The graders now apply that annotation, as the ledger itself does on
replay (GRADING.md, "From ledger.jsonl"). This changed only the M3 numbers:
regrading the M1 and M2 results gives files identical to the ones committed
there.

## Deterministic acceptance

`dla-bench dynamic` drives every fixture through the same stages with a fixed
policy that reads only `dla`'s own output, with no model involved. It passes
19 of 20 fixtures. The one expected failure is rust-tree-parent-links, for
the reason above: the mechanical policy cannot choose which edge of a cycle to
weaken. CI runs it on every push.

## Environment notes

- **JVM:** the JVM harness measures in-process, through the DiagnosticCommand
  MBean and JFR to a file. It needs no attach, so the read-only `/tmp` of the
  sandbox does not block it. `dla sample --pid` on a JVM still needs `jcmd`
  attach.
- **Missing libraries:** goleak, dhat-rs, tokio and pytest-memray are not
  available offline in the sandbox. The harnesses used their standard-library
  drivers, and the missing tools are recorded as coverage gaps.
- **Thresholds:** the slope gate, the plateau and purge fractions, and the
  residual threshold are provisional (see `docs/VERIFY.md`). M7 calibrates
  them.

## Reproduce the grading

```
dla-bench grade bench/results/m3-gates --out /tmp/rust.json
python3 bench/runner/grade.py bench/results/m3-gates --out /tmp/py.json
dla-bench compare /tmp/rust.json /tmp/py.json
dla-bench table bench/results/m3-gates
```
