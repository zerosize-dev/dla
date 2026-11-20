# F-a report: a registration leak can be labelled on its allocation stack

## Items
1. DONE. data/archetypes.toml: L-REGISTER requires now ends with ["growth-series","alloc-stack"] (4 alternatives; new one last).
2. DONE. Tests:
   - crates/dla-cli/tests/ledger_m6.rs: `the_shipped_l_register_row_labels_a_go_finding_on_growth_series_and_alloc_stack` (shipped archetypes.toml + known-leaks.toml, evidence E4,E6 named; kinds == [growth-series, alloc-stack]); `growth_alone_does_not_label_a_go_finding_l_register` (label-evidence-missing). The old synthetic-row test is kept; its comment now says it shows only the known-leaks route and not that the shipped row is met on Go.
   - crates/dla-core/tests/archetypes_data.rs: `l_register_ends_with_the_growth_and_alloc_stack_alternative` (last alternative, 4 alternatives, first stays unit-count-series).
   - SEEN FAILING with the data hunk reverted (git stash): the ledger_m6 positive test (label-evidence-missing) and the archetypes_data test (left [GrowthSeries, NonHeap]). The growth-alone test passes both ways by design (guards the refusal); judged by reading.
3. DONE. skills/detect-leak-anywhere/references/go-archetypes.md: L-WORKER fingerprint says a timer/ticker alone before Go 1.23 is the known-leaks row's case (L-REGISTER, registration-table), label accepted on growth-series + alloc-stack.
4. DONE. bench/README.md: label refusal text replaced (accepted on growth-series + alloc-stack; archetypes off: leak still unconfirmed because the holder comes from the known-leaks row).
5. docs/VERIFY.md: one new row in the M7 table ("A registration leak on Go could not be labelled L-REGISTER"), 4 cells.
Not in this package, noted in the row: known-leaks route counting a row only when the probe lists it true.

## Files changed (mine)
data/archetypes.toml, crates/dla-cli/tests/ledger_m6.rs, crates/dla-core/tests/archetypes_data.rs, skills/detect-leak-anywhere/references/go-archetypes.md, bench/README.md, docs/VERIFY.md (one row). Nothing under bench/fixtures*.

## Checks
- cargo test -j 2 --no-fail-fast -p dla-core -p dla-cli: all binaries pass (70 ok) except dla-cli `cli::committed_schemas_are_up_to_date`: schema drift for static-hit.schema.json, caused by F-d's change in crates/dla-core/src/static_hit.rs (not mine; F-d must run `dla schema --out schemas`). `dla schema --check` fails for the same file.
- cargo clippy --workspace --all-targets -D warnings: fails in crates/dla-backends/src/sweep.rs:260 (needless_update, F-d's file); so dla-cli clippy could not complete. Not mine. No windows clippy needed (no dla-core/probe/report source touched).
- cargo fmt --all --check: no diffs. python3 bench/runner/test_grade.py: OK. skill_text: 13 passed.
## Bench (debug binaries copied to scratch)
- dynamic --parallel 1, archetypes on, scratch copy of go-ticker-legacy + go-ticker-modern (pair line added in copy), the other 5 go-* fixtures, node-socket-heartbeat, node-socket-heartbeat-cleared, node-gateway-merge, python-ingest-two-leaks: 11/11 pass. go-ticker-legacy label kinds [growth-series, alloc-stack]; node-socket-heartbeat and node-gateway-merge still label on unit-count-series; python-ingest-two-leaks first label unit-count-series (unchanged). Same pass outcome as before for the six go fixtures (all pass in u11acc too).
- dynamic, pair, --archetypes off: go-ticker-legacy FAIL "leak L1 (internal/reserve/wait.go:11) is not confirmed" (expected, holder from known-leaks row); go-ticker-modern pass.
- static (60 fixtures of bench/fixtures) archetypes on and off: 60/60 pass; static on the pair (scratch) on and off: pass.
## Unverified
- Full dynamic suite not rerun (only the 11 above). Model-benchmark effect of L-REGISTER now met on Go not measured. dla-cli clippy not completed (blocked by F-d's sweep.rs error).
