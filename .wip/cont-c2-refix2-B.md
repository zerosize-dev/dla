REFIX REPORT (second refix of the shim-record pass; nothing committed or pushed). Every "problem" and "minor" item of the check that lies in my files is fixed, each with a test I saw fail without the hunk. The tree compiled throughout.

STATE OF EACH ITEM

1. Problem, a refused confirmation prints no detail: FIXED. The check was right and the previous report was wrong.
   - `failed_summary_detail` and `MAX_DETAIL_LINES` (8) moved from policy.rs to bench/runner/src/dynamic/ops.rs. `missing_evidence_detail` in policy.rs calls `ops::failed_summary_detail(.., Some(ops))`.
   - The refused branch of `confirm` in ops.rs now calls `failed_summary_detail(.., None)` and writes each line as a step, after `<F> confirm refused (<code>)`.
   - Tests: `a_refused_confirmation_prints_the_detail_of_the_failed_summaries` drives `run` against the stand-in dla. I added FAKE_GATE and FAKE_REFUSE_CONFIRM to the stand-in, plus a limitation on E2. The test asserts the step `type-histogram-diff via b: limitation: ...` directly after the refusal step. It also asserts that nothing is added when the gate passed or when the confirmation went through.
   - `the_detail_of_failed_summaries_is_filtered_by_operation_and_capped` is the helper test, moved from policy.rs and renamed because the old name described behaviour that did not exist.
   - SEEN FAILING with the loop in `confirm` removed (panicked at the steps assertion). Restored, passes.

2. Problem, VERIFY row 522 (false sentence, test list, "each failed"): FIXED. I re-read the row right before the edit and changed only my row.
   - The "refused confirmation" claim is now true.
   - The test list names the two new confirmation tests (the old test name is gone) and `stale_capture_files_are_removed_at_the_start_of_a_capturing_run_only`.
   - The reverted-and-failed sentence now lists what I saw fail: the note test and the capture test of the harness run (not the preloaded-variable test), the redaction test with a long value, the one-line identity note, the failed-gate test, the confirmation test and the stale-file test.
   - It says in words that the preloaded-decision test and the filter-and-cap test were not run against old code.
   - The row still has 4 cells and balanced backticks, and has no plan or package names, model names or line numbers.

3. Problem, argv word cut before redaction (a `key=value` word with a 27+ character value was written as it is): FIXED in crates/dla-cli/src/summarize/shim_groups.rs.
   - `argv_tail(argv, &Redactor)` now passes each word through `red.name` and then cuts it to its last 28 characters.
   - The limitation line uses it directly. The capture stores the redacted tail, and the raw file writes `c.argv_tail` without a second redaction. It was a double pass before, and it is now a single pass in the right order.
   - Test: `the_limitation_shapes_secret_looking_argv_words_and_omits_empty_brackets` gained a case with `password=` plus a 34-character value, asserting that `mnopqrstuvwxyz` is absent. My first assertion looked at text outside the 28-character tail and passed against the old order, so I replaced it.
   - SEEN FAILING with the old order restored. The failure printed `...ghijklmnopqrstuvwxyzABCDEFGH`.
   - Not changed: a secret given as its own word (`--token CorrectHorseBatteryStaple`) is not shaped. That is how `Redactor::name` treats any text without a key. It lies in redact.rs, outside my files, and is the same for every other text the redactor sees.

4. Minor, test gap for the two `if capturing` guards: FIXED in crates/dla-workload/src/harness/run.rs.
   - The two inline removals (command mode and driver mode) are now one helper, `clear_stale_capture_files(paths, label, capturing)`, called from both places.
   - Test: `stale_capture_files_are_removed_at_the_start_of_a_capturing_run_only` checks that `shim-dump.jsonl` and the capture directory stay when not capturing and go when capturing.
   - SEEN FAILING with the guard made unconditional.
   - Limit: a call site that passed `true` instead of `capturing` would not be caught by this unit test. Both sites pass the local `capturing`.

5. Minor, chance identity: FIXED as the check suggested, with a limit.
   - In `choose_process`, when several processes were left after the tool filter and the iteration count in argv chose a single one, one limitation line is now written. It reads "several processes wrote shim records with stacks in a capture; the one whose argv carries the capture's iteration count was read (the same number in another process's argv would have chosen that one)".
   - The line is the same text for every capture, so a file gets one line, not 17. Harness-written files hold one record, so they get no line. A lone process, or a tool next to the program, gets no line either.
   - Test: `the_process_with_the_iteration_count_wins_over_a_larger_one_and_a_tool_is_dropped` now asserts the one line, and the absence of a line for tool plus program. SEEN FAILING with the insert disabled.
   - Not done: the selection itself is unchanged. Identity still wins over size, so the chance case is shown but not avoided.

6. Minor, wording of two doc comments: FIXED.
   - The `SummarizeStep.detail` comment in bench/runner/src/dynamic_cmd.rs now says "the first few limitation lines" and that the process is shown for the first capture and the first few the holder is absent from.
   - The `missing_evidence_detail` comment in policy.rs says the same. The `failed_summary_detail` comment now describes a real caller.

NOTES (nothing to change)
- Cap of 8 lines cutting from the end: left as is. Both shim summaries read the same files, so the first summary's holder lines carry the information.
- By-design differences from HEAD (two records of one pid, a program named like a listed tool): left as is.
- The three other fixtures (native-vendor-codec, rust-private-registry, node-native-addon) run no harness, so they say nothing about this change. I did not rerun them.
- Swapped test counts: this run gives 147 for `dla-workload --lib harness::`, and `dla-bench` lib tests are now 148 with the new tests. The dla-cli form is `--bin dla`, not `--lib`.
- The two /tmp/dla-bench-dynamic-* directories (26941, 29948) are the check's; I left them.

FILES CHANGED IN THIS REFIX
- bench/runner/src/dynamic/ops.rs
- bench/runner/src/dynamic/policy.rs
- bench/runner/src/dynamic_cmd.rs (a comment only)
- crates/dla-cli/src/summarize/shim_groups.rs
- crates/dla-workload/src/harness/run.rs
- docs/VERIFY.md (row 522 only)

CHECKS RUN (all green, `-j 2`, shared target/)
- `cargo test -p dla-workload --lib harness::`: 147 passed.
- `cargo test --no-fail-fast -p dla-bench`: all passed (148 lib tests and every integration binary).
- `cargo test -p dla-cli --bin dla summarize::`: 68 passed, including the 6 `shim_groups` tests.
- `cargo test --no-fail-fast -p dla-cli --test summarize_native --test summarize_robust`: 27 and 6 passed.
- `cargo clippy --all-targets -p dla-cli -p dla-workload -p dla-bench -- -D warnings`: clean.
- `rustfmt --edition 2024 --check` on the 8 Rust files of this pass: clean.
- `cargo build -p dla-cli -p dla-bench`: ok.
- `dla schema --check`: ok, 20 checked.
- `python3 bench/runner/test_grade.py`: OK.

NOT RUN
- No dynamic fixture runs. These changes touch the failure text of a refused confirmation, the argv words of a limitation line and a helper that clears stale files, and they are covered by tests.
- `cargo clippy --workspace` and `cargo fmt --all --check`: only the three packages and the pass's own files.
- `harness_capture` (the other pass's file): not run.
- Disk was 3385 MB free at the start, so no prune was needed.