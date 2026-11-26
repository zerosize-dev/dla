# Builder N: a merge keeps a leak verdict, and growth in steps is not read as a plateau

Repository: /home/user/dla. Work in the working tree. Do NOT commit, push or switch branches.
The tree has uncommitted, checked work of other builders (one round is in CI now). Do not revert
or reformat files you did not write. Add one row per cause at the end of docs/VERIFY.md (re-read
it first). Do not touch harness/, bench/fixtures/ or any fixture README or answer key.

## What a real run showed (do not name the project in the repository)

An unattended run on a Rust library, whose documented leak is a `Vec` field of a sender that
grows once a bounded ring is full and nothing drains it. The run found it and then lost it in two
ways, and ended COMPLETE after 5 minutes of a 167-minute budget with no leak confirmed.

1. **The merge.** F41 is the census finding of the field. A driver made for it (`dla harness gen
   --lang rust --driver --probe F41`, harness H3) ran, and `dla alloc-test H3 --finding F41`
   (E27) favored leak (`os.rss_anon` growth continues). Its non-leak sibling was refuted. A second
   finding F45 (origin growth-series, at a writer line of the same field, same owner) got two
   alloc-tests on H3: E36 inconclusive, E37 non-leak. The model closed F45 `unresolved
   inconclusive` and then ran `dla ledger duplicate F41 --of F45`. After the merge nothing held
   F41: `grown::route_to_confirm` skips a merged finding, and F45's newest alloc-test is non-leak,
   so the leak verdict of a driver made for the field was gone.
2. **The plateau.** E37 decided by `native.live_bytes` (the allocator shim's live bytes, tied to
   the finding). The field is a `Vec` that doubles its capacity, so the column grows in steps
   (the trend gave shape `stair`). In the extra window the column rose by 62.9 MB, but the noise
   (the larger of the extra window's MAD and the main window's scatter around its trend) was
   37.3 MB, because the steps themselves are the scatter. 62.9 MB is under three noise units, and
   the extra slope (9 533 B per unit) was under half the main slope (35 750). So `plateau()` in
   `crates/dla-cli/src/alloc/judge.rs` read `plateau`, the column favored non-leak, and it
   decided. In the same window `os.rss` grew 118.8 MB (growth continues). The same driver at a
   tenth of the work per unit (E27) showed the steps clearly and read `never-grew` on that
   column.
   The recorded alloc-tests and trends of that run (numbers only) are in
   /root/dla-real/r4-fastrace-keep/alloc/A1.json..A3.json and /root/dla-real/r4-fastrace-keep/T*.json.
   Read them and build the unit tests from those numbers. Name nothing of the project.

## The change

1. **A merge keeps the leak verdict (dla-core ledger / grown, dla-cli ledger_m6).** Choose one of
   these and say why:
   (a) `dla ledger duplicate D --of C` is refused while D has a route to confirm
   (`grown::route_to_confirm(view, D)` is Some) and C has none. The fix is `dla ledger duplicate C
   --of D` (merge the other way, so the canonical is the finding the leak verdict is about), or
   D's own route.
   (b) The route of a canonical C also considers the alloc-tests of the findings merged into it
   (`ledger.duplicates_of`): the newest alloc-test of the merged family on a harness about any
   of its members decides, as `route_to_confirm` does for one family.
   Whichever you choose, the case above must end with a leak-side finding that cannot be closed
   unresolved while a leak-favoring alloc-test on a driver made for it is the newest word.
   Also: a merge of a finding into one that is already closed (`unresolved` or
   `suspected_static_only`) must not take an open finding's measured growth out of the gate. Check
   what `grown_unmeasured` and the final gate do with a merged duplicate, and say.
2. **Growth in steps is not a plateau (judge.rs `plateau`, gauges).** When the extra window's rise
   is clearly positive (more than one noise unit, and more than `min_rise`) and the column's
   main window rose in separate steps (`count_rises` exists for this), the column must not read
   `plateau`. A step-shaped column grows by jumps: its scatter around a straight fit is the step
   size, so "within three noise units" says nothing. Two readings to tell apart:
   - the extra window reaches a new maximum above the main window's maximum by more than the
     noise (a step happened in the extra window): growth continues;
   - no new maximum, but the extra window is shorter than the gaps between the main window's steps
     (`Rises` and `points_needed` already measure this): inconclusive, with the `--extra` that
     would cover a step, not plateau.
   Keep the present verdict for a column without steps, and for a real plateau (a step-shaped
   column whose extra window stays at or under the main window's maximum for longer than the
   longest gap between steps). Read the existing tests of `plateau`, `count_rises` and `Rises`
   first: F (an earlier builder) added the wider extra window rule, and these tests pin it.
   Never weaken them.
3. **Docs.** One sentence each in skills/detect-leak-anywhere/references/stages.md where merges
   (`ledger duplicate`) and where the plateau test are explained. Do NOT edit SKILL.md. VERIFY rows.

## Tests

- judge.rs unit tests with the numbers of E37 (the extra window rose 62.9 MB with a new maximum):
  not plateau. With the numbers of a real plateau (build one: steps in the main window, then a
  flat extra window longer than the longest step gap): plateau. A stair whose extra window is
  shorter than its step gap and has no new maximum: inconclusive, and the `--extra` advice names a
  window that covers a gap.
- An alloc-test unit or CLI test: E37's columns combined (`native.live_bytes` no longer
  non-leak, `os.rss` growth continues) no longer favor non-leak.
- CLI tests for the merge (find the `ledger duplicate` tests): the scene above (D with a leak
  route on its own driver, C with a newer non-leak alloc-test) either refuses the merge with the
  other-way fix (a), or keeps the route on C (b); and then a close of the canonical with
  `--reason inconclusive` is refused (`leak-favored`).
- Mutation checks: drop the new max rule (the E37 test fails); drop the merge rule (the merge
  test fails).

## Checks to run

`cargo test -p dla-core --lib`, `cargo test -p dla-cli --lib`, the dla-cli test files you touch
plus `--test alloc_test`, `--test grown_chain`, `--test ledger` (whatever the duplicate tests
are in), then `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all
--check`. Then the dynamic acceptance on every fixture whose verdict could move with the plateau
rule: at least the Rust, Go and native fixtures and their `-bounded`/`-flat`/`-freed` twins
(the non-leak twins must stay non-leak), with `./target/debug/dla-bench dynamic --dla-bin
./target/debug/dla --parallel 2 --only <substring> --json <scratch file>` per substring, from the
repository root. A non-leak twin that turns leak is a regression of your rule: fix the rule, not
the fixture.

## Rules

Shared `target/` only. Disk is tight: before building check `df -h /`. Under 3 GB free, delete
stale hash dirs older than 120 minutes under the literal paths
/home/user/dla/target/debug/build/dla-cli, .../dla-core, .../dla-workload and .../dla-bench with
`find <literal path> -mindepth 1 -maxdepth 1 -type d -mmin +120 -exec rm -rf {} +`. No
`rm -rf "$VAR/..."`. Never kill processes by name. Never skip or weaken a test. No project names,
attribution lines or model names in the repository. Scratch:
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u144/.

## Report (final message)

Files changed; for each cause the rule as implemented and why that design; the tests with what
each proves; the mutation checks; every command run with pass/fail counts; dynamic fixtures with
results (every non-leak twin by name); existing tests whose expectations changed and why;
anything you are unsure of.
