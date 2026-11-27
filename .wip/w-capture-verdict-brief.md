# Round W: one verdict for a capture series, and measured growth stays open until decided

sp = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad. Scratch: `$sp/u170/`.
Report: `$sp/u170/report.md`. Repository: /home/user/dla (branch main, HEAD 6796351 = 0.9.0-preview.15).

## Why (a real library run, r8)
The Rust run took allocation stacks of its driver H4 at 1, 2 and 4 times the step.
`dla summarize alloc-stack-diff` PASSED two stack columns at one project site, growing 0.52 -> 1.06 ->
2.12 GB. It also passed a resized single block at a second site. Neither column tied to a finding.
The run still ended COMPLETE after 8 of 167 minutes with "no leak confirmed". The chain that lost it is
general: it applies to every runtime's capture summaries (heap graphs, type histograms, shim stacks).

1. The final gate demands `dla trend` on every series, summary series included
   (crates/dla-core/src/gate.rs ~1596-1606: only gap series are exempt).
   `dla trend` (crates/dla-cli/src/trend_cmd.rs:211) judges a summarize-written series
   (source.kind Dump, 3-5 points) with the harness time-series gate.
   That is `GateThresholds::default()`, min_points 8, plus an exact Mann-Kendall test whose best p at
   n=3 is 1/6. So every column of every capture series fails "n < 8" and "p >= 0.05", whatever the data.
   The counter gate also fails a constant count of 1 with "delta < 1", where the summary said "resized".
2. The final gate's capture-summary item (gate.rs ~1719-1735, ~1862-1895, evidence_for_column)
   then offered `dla ledger add --origin growth-series --evidence` of the FAILING trend records
   (E59/E60), never the passing summary record (E34).
   The findings F45/F46 made that way carry no site (ledger_cmd.rs ~394-405 takes the site from
   passed columns only).
3. A summarize-written series has no `source.harness` (crates/dla-cli/src/summarize/growth.rs writes
   none; view.rs:163-170 fills series_harness from it). So harness_about could not reach H4, and
   `dla alloc-test H4 --finding F45` was refused with harness-not-about.
4. `counts_as_growth` (crates/dla-core/src/grown.rs:1074) counts only a passing column or a Python
   `py.blocks@` site column. The live-summary lead `dump_grown` needs a passed column that already
   names a finding, with no site fallback. So nothing held the findings.
   The model closed both with `dla ledger resolve ... --as unresolved --reason inconclusive`
   and its own hypothesis ("a reporter backlog") and no alloc-test. resolve accepted it.
   capture_accounted (gate.rs) counts a closed finding as accounting for the stacks whatever its
   state.
   The final gate passed and the stop hook released.

Kept files from that run (small): /root/dla-real/r8-fastrace-keep/ (ledger.jsonl, series/S8.json,
trend/T13.json, histogram-H1.json, harness/H4/captures.json, harness/H4/out/p*-alloc/shim-dump.jsonl,
gate/, transcript.jsonl). Full diagnosis with code references: `$sp/u169/summary.txt`
(the verifier corrections in it are binding, see "Do not" below).

## What to build

W1. One rule judges a capture series, in `dla summarize` and in `dla trend`.
- For a series whose source.kind is Dump (written by `dla summarize`), `dla trend` judges each column
  with the capture rule `dla summarize` uses (crates/dla-cli/src/summarize/rule.rs `gate_at`, with
  the series' own x: MIN_DUMPS 3, bytes rising at every pair, a count rising or one constant positive
  count (resized), the last interval's share of the first). Move the rule to one place both commands
  call; do not copy it. Harness series keep the 8-point gate exactly as now.
- Carry `resized` into the trend column. A count column of a resized site reads resized, not "flat,
  delta < 1"; do not change the counter gate for harness series.
- Per column, the trend's pass or fail on a summary series equals the summary's verdict. Write in
  the trend's thresholds/limitations which rule judged it.
- Check every reader of trend results for the changed outcome: gate capture_item, grown.rs
  (dump_grown, grown_unmeasured, counts_as_growth), machine.rs only_whole_process,
  ledger_cmd.rs (site from passed columns), report rendering.

W2. A summarize-written series names its harness when every capture is from one harness.
- When every capture label a summary read lies under one `harness/<H>/`, write `source.harness = H`.
  Look at the logic of `dla_core::measured_holders::capture_harness_strict`: every label must map,
  and all to one harness. Otherwise write none, as now.
- Check every effect of summary series entering `series_harness` / `harness_series`: harness_about
  (intended: an alloc-test of H for a finding that cites the series' evidence is accepted),
  residual/gap rows per harness, queue and coverage, gate items keyed by harness series. A summary
  series must not count as a harness run (attempts, executed, budget).

W3. A passing growth that ties to nothing becomes a finding that stands on a passing record.
- The final gate's capture-summary item offers evidence whose gate PASSED, and its text says the
  summary passed. After W1 the trend record of a summary series passes too; either passing record
  is fine.
- `dla ledger add --origin growth-series --evidence <E>` takes the site from the passing column.
- `dla summarize` prints that exact command when a column passes and ties to nothing. Do it for
  shim stacks (shim_groups.rs ~1061) and the equivalent places of heap-graph and histogram
  summaries.
- Once that finding exists it is a lead, as a tied dump holder already is. counts_as_growth,
  grown_unmeasured and dump_grown count it. If W1/W2 do not already make that so, add a site match
  for a passed column with no named finding (as grown_unmeasured does with finding_is_at_site).

W4. A measured, passing growth is closed only by a decision while budget is left.
- `dla ledger resolve <F> --as unresolved --reason inconclusive` is refused while the budget is above
  its floor for a lead under W3, and so is any close that claims no growth. The refusal prints the
  alloc-test on the harness about F, or `dla harness gen --driver --probe F` if none is about F.
  First check whether the existing refuse_grown_unmeasured already does this once W1-W3 hold; close
  the hole only if it does not.
- capture_accounted (gate.rs) counts a finding as accounting for a passing summary's stacks only if
  the finding is open, or decided (an alloc-test or discriminator), or closed for time or
  not-exercised at the budget floor. A close by text alone does not count.
- No deadlock: at the budget floor a close with the time reason is allowed.

## Tests (required)
- An end-to-end dla-cli integration test of the r8 shape, through rightsize like the other
  integration tests. Build the scenario yourself, or take fixture files from the kept copy; small
  files only, and never name the project in the repository (fixture and test names stay generic).
  The steps:
  - a Rust shim allocation-stack summary of 3 captures of one harness passes at a site that no
    finding is at;
  - the final gate lists the series;
  - `dla trend` passes the same column with `resized` carried;
  - the gate item offers the passing evidence;
  - `ledger add` makes a finding with the site;
  - `resolve --reason inconclusive` is refused and names the alloc-test on that harness;
  - `alloc-test <H> --finding F` is accepted (the harness is about the finding).
- Negative tests:
  - captures that rise and then stop fail in summary and trend alike, give no lead, and can be closed;
  - captures from two harnesses give the series no harness;
  - a harness series of 8+ points is judged exactly as before (an existing test should already pin
    this; name it).
- At least one non-Rust capture summary (JVM or Node heap graph) goes through W1 and W3 in a test.
- A mutation check for each new test: a unique marker, then restore, then `grep -c` the marker = 0.

## Do not
- Do not relax the capture rule. A bounded warm-up still reads the same in summary and trend.
- Do not change what confirms a leak. A tie plus a refuted non-leak explanation is still required.
- Do not change the 8-point gate for harness series.
- Do not change what counts toward failed_beyond_process.
- Do not make a remover in the same function tie or block anything.
- Do not exempt summary series from the final gate's trend demand. The demand is what put the growth
  in front of the model; make the trend agree instead.
- Do not edit skills/, agents/, data/ or hooks/ unless a text there states the old behaviour; if so,
  change only that text. docs/VERIFY.md: add or update rows for what changed.

## Rules
- No commit, no push, no branch switch.
- `target/` is shared; free disk only with `cargo clean -p dla-cli -p dla-core -p dla-workload -p dla-bench`.
- Literal rm paths. Kill only your own processes, by pid. Never weaken a test to pass.
- The repository never names the evidence run's project or its APIs.

## Checks
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and the test
  targets you touched. Not the whole suite: I run it.
- Then the dynamic set once:
  `./target/debug/dla-bench dynamic --dla-bin ./target/debug/dla --parallel 2 --json $sp/u170/dyn.json`
  after building dla and dla-bench.
  - Expected: 63 pass plus 1 xfail (rust-tree-parent-links). jvm-pool-warmup is a known flake: if it
    alone fails, run it alone up to 3 times.
  - Compare confirmed/claim/rule-out/leaks per fixture with `$sp/u164/dyn2.json` and explain every
    difference.
  - Delete the dynamic logs afterwards.

Report: files changed, each item as implemented, tests and the mutation that made each fail, commands
and results, dynamic comparison, anything unsure.
