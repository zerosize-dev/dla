# Pass 12 (after pass 11): the total of a capture summary must follow the capture rule

Same rules as passes 8 to 11 (Sonnet only; no commits; shared `target/`, `-j 2`; small exact
edits, no reformatting; literal paths in `rm`; no process kills by name; unique-marker
mutations reversed by exact replacement; no test, fixture or bench policy weakened; never
`dla consent` on a real tree). No real plugin run is on this machine now, so `dla-bench
dynamic --only <prefix>` is allowed (binary copies, `DLA_PLUGIN_ROOT=/home/user/dla`).
Another builder is working in `harness/python/` and `crates/dla-cli/tests/alloc_test.rs`:
do not touch those. Scratch `<sp>/u57/`; append to `<sp>/u49/report.md` under "Pass 12";
short final message.
`<sp>` = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad

## What happened

CI on the frozen snapshot with your pass 11 (56aa221): dynamic acceptance 62/64, one failure
(log: `<sp>/u57/ci-dynamic.log`, lines 150 to 196):

    go-subscription-workers: gate check --final: 1 item(s) missing: trend_column
    T6:metric:goroutine/total: column metric:goroutine/total of T6 (steady) is never tied to a
    finding; it is the sum of its group columns, which account for it once each non-flat one
    is tied ...

The same fixture passed in both CI runs of the commit before pass 11 (d297716). In the
failed run F2 was confirmed (alloc-test, goroutine column at subscription.go:33; the
goroutine summary "tied F2"), the driver followed two printed `dla ledger add --origin
growth-series --evidence E23|E24` lines, and then the total of T6 stayed, with a fix text
that is not a command ("tie the group columns of T6 first (their own trend_column items)").
The bench output cut the "(not yet: ...)" list and the artifact cannot be downloaded here.

## What I read in the code (check it, do not take it as given)

`gate.rs`, `classify`: for the total of a summarize-made series, `group_columns_state`
asks that every non-flat `group:` column of the trend is tied to a dispositioned finding.
Pass 11.2 took the own items of untied group columns away: they go to `captured`, and
`capture_item` lists nothing when the capture is `decided` (a column with the same evidence
is tied to an alloc-tested finding) or `stands_for` (a growth-series finding supports the
evidence). So a trend with one group tied to the alloc-tested finding by its site and a
second, untied non-flat group has no item for the second group, and the total still waits
for it: an item that no printed command can clear. Whether the second group is non-flat
differs from run to run, which would explain green before and red now.

## Wanted

1. First a test that shows the failure on the current tree (a final gate over a summary
   trend: total non-flat; one non-flat group tied by `candidates` to a confirmed finding
   with an alloc-test; a second non-flat group with the same evidence tied to nothing; no
   other item). It must list the total today. If it does not, my reading is wrong: stop and
   find the real cause from the log before changing anything.
2. One rule for both places: a non-flat group column counts for its total when it is tied to
   a dispositioned finding, superseded, covered by an open item (as today), or accounted for
   by its capture exactly as `capture_item` decides (`decided` or `stands_for`). Take the
   predicate out of `capture_item` and use it in both, so they cannot drift again.
3. When the total is still listed, its fix must be something to run: the capture's own
   `dla ledger add --origin growth-series --evidence E` line when that is what is missing,
   or the id of the finding whose disposition it waits for. No "their own trend_column
   items" text when no such item exists.
4. Check every other reader of the group state for the same gap (the stage gate with
   `is_final` false, `gate::actions` grouping and `missing_dependents`, `ledger close-rest`,
   `ledger resolve`, `run status`, the report's capture lines, `trend_cmd.rs` near the
   comment "The total of a summarize-made series is the sum of its group columns"), and the
   `.pre_gc` twin of a total. Say what you found for each.
5. Do not change `bench/runner` policy or any fixture. If the bench driver would still meet
   an item without a runnable command, say so and stop.

## Tests and checks

- The test of 1 seen failing before the fix and passing after; one marker mutation on the
  new predicate seen failing; a test that the total is still listed (with a runnable fix)
  when the capture is neither decided nor stood for.
- `dla-bench dynamic --only go-subscription` three times, and `--only go-` once: say the
  results and, for go-subscription-workers, whether T6 had a second non-flat group in any
  of the runs (read the run's trend file: say which run showed the case, or that none did).
- fmt; clippy on what you touched; `cargo test -j 2 -p dla-core -p dla-report -p dla-cli`
  (`--no-fail-fast`; name failures that are not yours); `dla schema --check`;
  `docs/VERIFY.md` row; docs only where the rule is described.

Report: what the cause was (with the evidence), what changed, tests with what was seen
failing, the bench results, what is unverified.
