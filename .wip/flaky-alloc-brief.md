# Brief F: two alloc-test verdicts that flip under load

Repository: /home/user/dla (Rust workspace, branch `main`). The working tree holds uncommitted,
checked changes of earlier builders. Leave them as they are; where you must edit one of those
files, add your own lines only. You are the only builder in the tree.
Scratch directory: /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u133
(create it; write nothing else outside the repository).

The machine has 4 cores. Use `cargo ... -j 3`, one cargo command at a time. Never touch
/root/dla-real. The container can be restarted without warning: keep each background job short
enough to lose, write its output to a file in your scratch directory, and after a restart look at
the tree and the files before you go on. Do not end your work while one of your background jobs
still runs: wait for it, so that your final message is the final report.

## What fails

Two tests of dla-cli fail now and then on the CI runners (and one of them locally under load),
each time with a verdict of `dla alloc-test` that is not the one the scene was built for. Each
costs a CI round. They are not to be weakened, skipped or retried inside the test: the verdict
rule is what must hold under load.

1. `crates/dla-cli/tests/alloc_test.rs::a_flat_file_backed_column_that_passed_the_gate_reads_non_leak_on_a_flat_run`
   (twice in a row on the Linux runner, 2026-11-25). The alloc-test record of one failure:
   `os.rss_file`: plateau `inconclusive` (main slope 95.6, extra slope 154.7, extra rise 162,500,
   noise 42,510), purge kind `trim`, before = after = 8,679,000, verdict `retained`, so the column
   `favors: leak` and `decided_by: os.rss_file`; `os.rss` and `os.vm_hwm` favor non-leak
   (plateau, purge `partial`); the overall verdict is `inconclusive` where the test wants non-leak.
   The rule is in `crates/dla-cli/src/alloc/judge.rs` (`combine`, `overall`): a plateau that is
   `inconclusive` with a purge that `retained` favors leak.
2. `crates/dla-cli/tests/alloc_test_jvm.rs::an_unbound_alloc_test_of_a_command_that_runs_the_projects_workload_can_refute_a_finding`
   (line 289; Linux runner and locally under load). Diagnosed earlier: a column with a plateau and
   a purge that `retained`, with fewer expected rises in the extra window than
   `MIN_EXPECTED_RISES` (3), reads `inconclusive` (the arm in `combine` before the plain
   `Plateau` one), so the refutation the test expects does not come. The window length alone
   decides it.

## The change

For each: reproduce it (run the test in a loop, then under load: start three busy loops such as
`sh -c 'while :; do :; done' &` and stop them by their PID afterwards, never by name; say how
many runs of how many failed before and after), find the general cause in the rule, and fix the
rule, not the test. Read what each part of a verdict means before you change it. Questions to
answer with evidence:

- File-backed resident memory (`os.rss_file`: page cache of mapped files, the binary and its
  libraries) is not released by a `trim` purge, which frees heap only. Can a `trim` purge say
  anything about such a column? If not, its purge verdict for that column is "does not apply",
  and the column is judged on its plateau alone (an `inconclusive` plateau then gives an
  `inconclusive` column, not a leak). Check every column and purge kind for the same mismatch
  (which purge kinds release which regions) and say what you found.
- When only the length of the extra window keeps a plateau from deciding (the `MIN_EXPECTED_RISES`
  arm), the tool can lengthen the window instead of answering `inconclusive`: an unbound
  alloc-test of a command runs the command again with more iterations. Find whether the tool
  already has a way to extend (the earlier note: "extend alloc-test window when only window length
  holds verdict"), and either extend once to the length that gives 3 expected rises, bounded by
  the time budget, or say why that cannot be done and what else is honest.
- Neither change may turn a real leak into a non-leak: run the leak scenes of
  `alloc_test.rs`, `alloc_test_jvm.rs` and the fixtures below and show they still read leak.

## Tests (each seen failing under a mutation you apply and revert)

- Unit tests of `combine` (or of the new rule) for each case above, and for the leak case each
  must not change.
- The two tests above, 20 runs each under the load described: report the counts before and after.
- Existing tests that change: say which and why. Do not weaken a test. Nothing under
  `bench/fixtures` is edited. The verdict rule decides every fixture, so at the end run the WHOLE
  dynamic acceptance once: `./target/debug/dla-bench dynamic` (all fixtures, 40 to 60 minutes;
  background, output in your scratch directory; nothing heavy beside it) and report the whole
  table and its totals line. One fixture is an expected failure (xfail); `jvm-pool-warmup` is
  the fixture with the same JVM mechanism as test 2: run it alone three times after your change
  and report all three.

## Rules

- No `git commit`, `git stash`, `git checkout` (in any form), `git reset`, `git restore` or
  branch change. Leave the tree dirty.
- Shared `target/` only (`cargo` from /home/user/dla). No `rm -rf` on variable paths. Do not
  kill processes by name. Do not run `dla consent`. Do not start any model, agent or verifier.
  No package installs.
- Run each cargo command on its own, in the foreground, piped through `tail` or `grep` (the
  loops under load and the whole dynamic run may run in the background, one at a time). If an
  action is denied, stop that action and report it; do not ask anyone to do it for you and do
  not try another way to the same end.
- Keep to this brief. If you find another defect, report it; do not fix it.
- Plain technical English, full sentences, no em dashes. No model names, no attribution lines.
- Mutations by a small script that asserts the old text occurs exactly once. Before you finish,
  `grep -rIl "MUT_" crates harness skills docs bench agents hooks data` must print nothing.
- `docs/VERIFY.md`: a row at the end per rule you change, state `corrected`.

## Checks before you report

`cargo fmt --all --check`; `cargo clippy -j 3 --workspace --all-targets -- -D warnings`;
`cargo build -j 3 -p dla-cli -p dla-bench`; `cargo test -j 3 -p dla-cli --bin dla`,
`--test alloc_test --test alloc_test_jvm --test holder_first --test py_site_columns
--test inconclusive` (if it exists), and the tests you added; `cargo test -j 3 -p dla-core --lib`.

## Report (final message; read by the orchestrator)

Two sentences of result first. Then per flake: the cause with its evidence, the rule change,
the runs under load before and after (counts), each test with its mutation. Then the region and
purge-kind table of the first question, the whole dynamic table, `jvm-pool-warmup` three times,
existing tests changed and why, anything in this brief that was wrong, what you did not verify.
Counts only.
