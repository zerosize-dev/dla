# Builder I: one gate item per growing per-site column of a harness, not one per run

Repository: /home/user/dla. Work in the working tree. Do NOT commit, push or switch branches.
Other builders' finished changes are uncommitted in the tree; leave their files alone unless this
brief names them. You may add rows at the end of docs/VERIFY.md (re-read it just before editing).

## What a real run showed (do not name the project anywhere in the repository)

A Node run measured one harness three times: series S2 (keys repeated), S3 (fresh) and S5 (fresh,
more rounds), with trends T2, T3 and T5. The same 20 per-site listener columns
(`node.listeners@src/x.ts:<line>`) grew in all three. The final gate listed every one of them three
times, as `T2:node.listeners@...`, `T3:...` and `T5:...`, each with "column ... is not flat and is
tied to no finding. Fix: dla ledger add --origin growth-series --evidence E13 # column ...". That
was 60 of the 154 open items. A finding at a column's site ties that column in every trend (per-site
columns tie by their own site only), so the three items are one obligation. The model read
"154 items are still missing" and stopped with two thirds of its time budget left, saying it could
not clear the list.

## The change (crates/dla-core/src/gate.rs, `classify`, the final branch that adds the `trend_column` item for a non-flat column that is tied to no finding or only to undispositioned ones)

- For a column with a site (`crate::grown::column_site(c).is_some()`: listeners, timers,
  allocation-site counts and every other `<kind>@<file:line>` column), the item goes only to the
  newest trend among all trends of series of the same harness (`view.series_harness`) in which a
  column of the same name is non-flat. The others are left out. Trend ids are ordered by
  `trend_number`. The kept item's description ends with "(also not flat in T2, T3)" when there are
  others. Its fix cites the evidence of the kept trend. Write this as its own small predicate next
  to `superseded`. Do not change `superseded`: it serves whole-process columns and capture items
  too.
- Columns without a site, whole-process columns, type counts, capture-summary columns and series
  with no harness keep today's behaviour.
- The same predicate must apply wherever this per-column item is counted or shown: `dla gate
  check`, `dla run status`, the stop hook's count (`gate::actions` in gate_hooks.rs) and the
  report's open items, which all come from `gate::missing`. Check that, and also that the
  stop-hook digest still changes when something changes.

## Tests (crates/dla-core gate unit tests or a CLI test next to the existing trend_column ones; grep "is not flat and is tied to no finding" in crates/dla-cli/tests)

1. Three series of one harness, the same per-site column non-flat in all three: one item, on the
   newest trend, with "(also not flat in ...)".
2. The same column non-flat in a series of another harness: two items, one per harness.
3. A finding added at the column's site and closed: no item in any trend (as today).
4. A whole-process column (`node.heap_used`) non-flat in the three series: unchanged from today.
5. Mutation check: make the predicate always false, and see test 1 fail with three items.

## Rules

Shared `target/` only. Run `cargo test -p dla-core`, the dla-cli tests that mention
`trend_column` or "tied to no finding" (grep), `cargo clippy --workspace --all-targets -- -D warnings`
and `cargo fmt --all --check`. No `rm -rf "$VAR/..."`. Delete only literal paths. No benchmarks.
Never skip or weaken a test. If an existing test expected three items, show it in the report and
explain why one is now right. Do not weaken a test that checks something else. One VERIFY row with
neutral words. No project names, no attribution lines, no model names in the repository.

## Report (final message)

List the files changed. Give the rule as implemented, the tests with what each proves, the
mutation check and its result, the commands with pass or fail counts, existing tests whose
expectations changed and why, and anything you are unsure of.
