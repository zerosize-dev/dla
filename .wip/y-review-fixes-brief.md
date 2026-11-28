# Round Y review fixes (three confirmed defects)

Repository /home/user/dla, working tree holds round Y (uncommitted; do not commit, push, stash or switch
branches). Build with `CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_PROFILE_TEST_DEBUG=line-tables-only`
exported (the disk is small: never run `cargo clean`, never build in another target dir, run only the
test targets you touch, not the whole workspace). Scratch dir: /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u178.

Rules: no Claude or model names anywhere in code, comments or docs; never name a third-party project or its
APIs in the repository; never weaken or skip an existing test; every new test must fail under a mutation of
the fix (apply the mutation with a unique marker, run, restore, `touch` the file after restoring so cargo
rebuilds, check the marker count is 0). Plain English in comments, short sentences.

## F1. A newer blind capture must not be hidden by an older summary of the same harness

`crates/dla-core/src/measured_holders.rs` `scan_in`: a blind AllocStack record (`result.blind == true`)
goes into `Scan.blind` and `continue`s; it never revokes an earlier non-blind summary of the same harness.
`ledger.evidence()` iterates by id, not by time. Gate code (`crates/dla-core/src/gate.rs`
`uncaptured_residuals`, `undecided_heap_of`, `open_item_text`) then reads the harness as summarized (and
`Flat`, or `grew` from an older record) and drops the blind text.

Rule to implement: for the allocation-stack summaries of a harness, the NEWEST one decides (by
`ledger.evidence_order(id)`, a `RecordOrder`; treat a missing order as oldest). If the newest is blind, the
harness's allocation-stack summaries count for nothing: not in `summarized`, not in `summarized_at`, not in
`grew`, and `blind[h]` holds its text. If the newest is not blind, `blind` has no entry for h and the
non-blind records count as now (summarized_at = max ts of the non-blind ones, grew = any non-blind one shows
growth; keep the current union semantics). Retention-path records are never blind and are not revoked by a
blind allocation-stack record (a heap walk is a different measurement); they still count as now. Keep the
rest of `scan_in` (holders, harnesses, the retention loop) unchanged; do the allocation-stack bookkeeping so
the result does not depend on the iteration order (e.g. collect (order, h, blind, text, shows_growth) and
resolve per harness after the loop). Update the `Scan` field docs.

Tests (dla-core unit tests near the existing measured_holders tests, or crates/dla-cli/tests/capture_blind.rs,
whichever already builds such ledgers more easily):
- older non-blind flat summary of H, newer blind one: H not in summarized/summarized_at/grew, blind[H] set;
  `undecided_heap_of` (if reachable in the test setup) reports `CaptureSaid::Blind`, and `open_item_text`
  says blind.
- older summary that shows growth, newer blind one: H not in grew.
- older blind, newer non-blind: H summarized, no blind entry.
- the outcome is the same whichever evidence id sorts first (ids chosen so that id order and time order differ).

## F2. `heap-growth-undecided` must always name a command that can release it

`crates/dla-core/src/gate.rs` `UndecidedHeap::commands()`: with no `grew_untied` lead it returns only
`dla harness run H --mode M --capture alloc ...`. After a capture that was summarized flat
(`CaptureSaid::Flat`) or blind (`CaptureSaid::Blind`), re-running the capture cannot release the refusal; only
an alloc-test of the harness can (`grown::alloc_measured_on`; `dla alloc-test H` takes `--finding` as
optional).

Change: commands are, in order:
- with leads: one `dla alloc-test H --finding F` per lead (as now), nothing else;
- no lead and `capture` is `None` (no summarized capture yet): the capture command first, then
  `dla alloc-test H` (the capture can show the stack and name the holder; the alloc-test decides the growth);
- no lead and `capture` is `Some(Flat)` or `Some(Blind)`: only `dla alloc-test H` (a capture again cannot
  change what it said).
Check `crates/dla-cli/src/ledger_cmd.rs` `refuse_undecided_heap` and `gate.rs` `residual_item_fix` still read
right with the new list, and that `bench/runner/src/dynamic/policy.rs` `follow_undecided_heap` /
`harness_of_undecided` still parse it (`dla alloc-test H` with no `<` is run as printed). Check that
`dla alloc-test H` with no `--finding` really runs on a harness and records `result.harness == H` with
`test == "plateau+purge"` so `alloc_measured_on` sees it (read `crates/dla-cli/src/alloc_cmd.rs` and the
alloc-test evidence writer; if it records the harness under another key, make `alloc_measured_on` read it).

Tests: extend `crates/dla-cli/tests/heap_decided.rs` (or the gate unit tests): no lead + Flat capture -> the
fix names `dla alloc-test H` and not `--capture`; no lead + no capture -> capture then alloc-test; leads ->
the per-lead alloc-tests. Also one end-to-end case: after `dla alloc-test H` (no finding) the same
`ledger resolve O1 --as unresolved` is no longer refused with `heap-growth-undecided` (if building that in the
test is too heavy, write the alloc-test evidence record the way the command writes it and say so).

## F3. `<key>_cut` must describe the array that is finally printed

`crates/dla-cli/src/out.rs`: `shrink` now inserts `<key>_cut: n` when it cuts an array. `bound` then
restores the PROTECTED keys with `protected_copy(.., max_arr.max(32))` and the KEPT_WHOLE keys whole (also
inside `summary`), so a `files_cut: 12` can stand next to a `files` list of all 20 items. The last-resort loop
cuts PROTECTED arrays with `protected_copy(x, max_str, max_arr)` and names nothing.

Change: after any key is replaced (both loops, and the nested `summary` KEPT_WHOLE case), set
`<key>_cut` from the original array length minus the length of the array actually kept, and remove it when
nothing was cut. Do it in one small helper used at every place a key is restored.

Tests in `out.rs`: a protected key (`files`, 20 short items) in an output large enough to be bound: no
`files_cut`; the same with 40 items: `files_cut == 8`; a last-resort case where a protected array is cut:
`<key>_cut` present with the right count. Existing out.rs tests must still pass.

## Checks to run and report

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test -p dla-core --lib`, `cargo test -p dla-cli --bin dla` (or the bin's unit tests target), and
  the integration targets: heap_decided, capture_blind, residual_rows, rust_driver, summarize_native,
  harness_capture, and any other target whose source references the functions you changed (grep).
- `cargo run -q -p dla-cli -- schema --check` if a schema-bearing type changed.
- Each mutation and the test that caught it.

Final message: files changed, what each fix does, tests and their mutations, command results, anything not
done or unsure.
