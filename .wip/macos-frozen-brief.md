# Brief P: a Python harness run lost its groups on macOS CI (flaky), and nothing said so

Rules as in your earlier briefs (Sonnet only; no commits; shared `target/`, `-j 2`; small
exact edits; literal paths in `rm`; no process kills by name; unique-marker mutations
reversed by exact replacement; no test weakened, no retry loop that hides a failure; never
`dla consent` on a real tree). Another builder is working in `crates/dla-core/src/gate.rs`
and the gate tests: do not touch those. Scratch `<sp>/u58/`, report `<sp>/u58/report.md`,
short final message.
`<sp>` = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad

## What happened

CI, job "clippy and test (macos-latest)" (image macos-26-arm64), test
`frozen_garbage_is_tied_to_the_freeze_site_by_a_count_column_and_reads_growth`
(`crates/dla-cli/tests/alloc_test.rs`) failed once: `<sp>/u58/ci-macos-failure.txt`.
The same test passed on the same image in the two CI runs of the commit before (the
Python agent, the harness run code and the test are identical in both commits), so it is
a flake. What the failed run's `dla trend S1` printed:

    columns 15, failed 12, passed: py.traced, py.traced.pre_gc, py.traced_peak
    (slope 904.03 each), other_targets {"not_loaded": 1}, targets []

The same steps on Linux (my copy of the test's project and its run directory:
`<sp>/u56/frozen`, trend output `<sp>/u56/trend-linux.json`): columns 41, passed 8,
among them `py.gc_frozen@batcher/worker.py:23` (slope 10, candidates F1) and the
`os.*` columns, other_targets {"grew": 1}.

So in the failed run the workload ran and leaked as designed at every point (py.traced
rises by the same 904 bytes per unit), but the series had no `py.gc_frozen` group at all
(`not_loaded` is the verdict of a probe without a column) and far fewer columns. The
records of that run were reduced, and neither `dla harness run` nor `dla trend` said so:
the trend read the target as "not loaded", which is false.

## Wanted

1. Find which path gives such a series: records with the tracemalloc values of the end of
   the workload but without the groups (`py.gc_frozen`, and whatever else is missing
   between 41 and 15 columns: say which columns a macOS record normally has, from the code,
   since there is no macOS machine here). Candidates to check, none of them established:
   the exit dump taken by another path than the full one (a checkpoint already written, so
   `_exit_dump` returns; a time limit; a deadline in the probe walk); `_install_gc_freeze`
   not having run or `_freeze["called"]` not set in the process that dumps; a group dropped
   on the Rust side when records disagree between points; two test processes sharing a
   file. Five tests of that file run in parallel and three of them run Python harnesses.
   Reproduce the reduced series on Linux by forcing the path (load, a short limit, an
   injected delay: your choice, but show the same symptom: no group, `not_loaded`,
   `py.traced` intact). If you cannot reproduce it, say so plainly and give the two most
   likely paths with the code lines.
2. Fix the cause if it is in the product. Whatever the cause, a reduced record must not
   pass silently: the run says it (`_attention` of `dla harness run`, the series'
   limitations), and `dla trend` does not call a target `not_loaded` when the record that
   would carry its column was reduced (it is `unknown`, with the reason and the command
   that measures again). Use the existing fields and wording where they exist.
3. Make the test explain itself next time: when the column did not pass, the panic message
   also carries the run summary's `_attention`, the series' limitations and the column
   names of the series (a small helper in the test file; the assertions stay as they are).

## Tests and checks

- A test for the rule of 2 (reduced record -> said in the run output, target `unknown`,
  not `not_loaded`), seen failing once by a unique-marker mutation, exact reverse.
- The failing test itself 10 times in a row with the whole file in parallel
  (`cargo test -j 2 -p dla-cli --test alloc_test`), and once more under load (run the
  dla-workload Python harness tests at the same time): say the results.
- fmt; clippy on what you touched; `cargo test -j 2 -p dla-workload -p dla-cli`
  (`--no-fail-fast`; name failures that are not yours); `dla schema --check` if a field
  changes; `harness/python/README.md` and `docs/VERIFY.md` rows where the behaviour is
  described.

Report: the cause (established or not, with the evidence), what changed, tests with what
was seen failing, what is unverified.
