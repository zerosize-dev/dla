REPORT (all four decisions done, nothing committed or pushed)

CAUSE, as I verified it
- Totals of raw_sites[*].live_bytes, printed by me from /tmp/dla-bench-dynamic-2919/jni-order-codec/.dla/20261113-001707-b611/harness/H1/out/:
  - p1-native: test JVM pid 10920 = 41858379 (argv ends `shop.codec.CodecTestKt`); program pid 10872 = 47769802 (argv ends `shop.codec.MainKt 2220`).
  - p2-native: test JVM pid 11039 = 18908890; program pid 10994 = 48416586.
  - p17-native: test JVM 19407935; program 144893809.
- Every record, tool and program alike, carries `x` equal to the point's iteration count (2220, 4440, ...). Only the program's argv carries it, as a token (`MainKt 2220`) or in `--iterations N`.
- The same holds in other real dumps: jvm-order-cache has a test JVM plus `shop.Main 1110`, and native and node dumps carry `--iterations N`.
- The old reader (`shim_run`) kept the record with the most live bytes per file. When the test JVM's JIT arenas outweigh the program at point 1, capture 1 is the wrong process and the leak's stack has no live block there ("holder absent from dump 1").

REPLAY on the real files
- Copied the whole run directory to <sp>/u10/replay/ and made a fresh run in the copy.
- Raised the test JVM's byte figures in p1 x2, to 83716758, which is above the program's 47769802.
- Ran `dla summarize type-histogram-diff` on the 17 shim-dump.jsonl files with the new binary. Gate: pass, `stack:Java_shop_codec_Native_open` passes.
- To show what the old rule read, I kept only the test JVM's record in p1 (oldview/) and ran the new binary on it. Gate: fail, with "nothing grows strictly across all 17 captures" and "stack:Java_shop_codec_Native_open: holder absent from dump 1".
- I did not run the old binary itself (built nothing extra). The old result is shown by feeding the new binary the record the old rule would choose, and by reading the code.
- The reader unit and integration tests also failed with the old shim_groups.rs restored.

DECISIONS
1. Done: crates/dla-workload/src/harness/capture.rs
   - New `split_shim_dump(label_dir, measured_pid) -> ShimSplit {NotApplicable, Written, Missing{others}}`.
   - It writes only the shim exit record of the measured process, and removes an older shim-dump.jsonl first.
   - It writes none when that process has no shim record with stacks, and never falls back to another process.
   - A native-only dump (already the shim's own) is left alone.
   - `shim_dump_commands` now only lists the existing shim-dump files.
   - crates/dla-workload/src/harness/run.rs: new `split_shim()` is called in command mode with `select_measured`'s pid, and in driver mode with the pid from `select_checkpoints`. Both are the same process `merge_shim` uses.
   - run.rs also removes a stale shim-dump.jsonl at the start of every point.
   - A Missing outcome adds a limitation: "<label>: pid P wrote no allocator shim record with stacks, so this point has no shim-dump.jsonl; the N shim record(s) of other processes were not used in its place".
   - The limitation text itself has no unit test. The run.rs helper needs the full run context. Only `ShimSplit::Missing` is tested.
2. Done: crates/dla-cli/src/summarize/shim_groups.rs and crates/dla-workload/src/harness/dumps.rs
   - dumps.rs now exports `is_tool_program`, `keep_with_n` and `largest_by`. `select_measured` itself uses them, so there is one implementation and `argv_has_n` is reused.
   - The reader groups records by pid (last record per pid), drops tool processes (unless nothing else is left), then prefers the processes whose argv carries the capture's `x`, and uses size only among the processes still left.
   - When size decided, a limitation line says: "<file>: N processes wrote shim records with stacks and none had X (the iteration count) in its argv [or: X is in the argv of several, or: the capture has no iteration count to tell them apart]; kept pid P (last argv words) as the one with most live bytes (B of b1, b2, ...)".
   - Each capture in the raw histogram file now has `pid` and `argv_tail` (last 3 argv words, long words keep their end).
3. Audit of other readers of dump files. No other reader picks by size among processes.
   - `select_measured` (identity first, then size with a limitation line): fine.
   - `select_checkpoints` (the process with most checkpoints, with a note): fine, driver mode with a note.
   - `capture::gather`/`prune` (identity by pid) and the Node note near capture.rs:605 (it only groups commands): fine.
   - vgdb.rs `max_by_key` (blocks, not processes) and run/goleak.rs (first record's pid; the Go agent is loaded only in the test binary): fine.
   - Not the same defect, not fixed. summarize/proc_fd.rs and summarize/harness_types.rs take every record of every process of a file in file order. A command-mode dump with a tool or test process next to the program would mix them. It is a different choice, outside this brief.
4. Done: bench `dynamic` failure text. This touches bench/runner/src/dynamic/ops.rs and policy.rs besides dynamic_cmd.rs, because the failure text is built there.
   - `SummarizeStep` gets `detail`, filled by the new `gate_detail()` for every summary whose gate failed.
   - It reads the limitation lines of the growth evidence from the run's ledger. For each `stack:<name>` the gate names (up to 2), it reads bytes/blocks per capture (`-` where absent) from the raw file, plus pid and last argv words of the first capture and of the captures where the holder is absent (up to 4).
   - When a label is refused `label-evidence-missing`, the policy adds those lines to the fixture's decision steps, which CI prints. The failure line itself is unchanged, and so is EXPECTED_FAILURES.
   - Side note, not touched because it is graded: `accept_hidden_tool_gaps` tests `ends_with("label refused label-evidence-missing")`, but the failure line can already end with a `[summarize ...]` bracket.
   - `dla schema --check` passes (nothing serialized changed; the raw file is plain json).

FILES CHANGED (mine)
- crates/dla-workload/src/harness/{capture,dumps,run}.rs
- crates/dla-cli/src/summarize/shim_groups.rs
- crates/dla-cli/tests/summarize_native.rs
- bench/runner/src/dynamic/{ops,policy}.rs and bench/runner/src/dynamic_cmd.rs
- docs/VERIFY.md: one new row at the end of the `## M7` table, "The shim reader read another process's record (`jni-order-codec` flake)"; 4 cells checked
- No doc stated the old rule, so no doc sentence was added.

TESTS, each seen failing with its hunk reverted, then restored
- capture.rs:
  - `the_split_keeps_the_measured_process_record_not_the_larger_one`. This is test (b). Failed with the pid choice replaced by "take any".
  - `a_measured_process_without_a_shim_record_gets_no_file_and_never_another_one`. Failed the same way.
  - `shim_records_of_a_jvm_dump_are_split_out_for_the_command` was adapted to call the split first.
- summarize_native.rs:
  - `the_record_of_the_process_with_the_iteration_count_wins_over_a_larger_one`. This is test (a), for both `type-histogram-diff` and `alloc-stack-diff`. Failed with the old shim_groups.rs.
  - `with_no_iteration_count_in_any_argv_the_largest_is_kept_and_the_limitation_says_so`. This is test (c). Failed with the old shim_groups.rs.
- shim_groups.rs unit tests (run as `cargo test -p dla-cli --bin dla summarize::shim_groups`; dla-cli has no lib target, so the brief's `--lib` form errors): tool dropped, identity, size fallback with limitation, lone process and lone tool, long argv word. These are new functions with no old counterpart, so I did not run them against the old code.
- bench:
  - `a_label_refused_for_missing_evidence_prints_the_steps_that_could_have_produced_it`, extended. Failed with the printing call removed.
  - `a_failed_gate_shows_its_limitations_and_the_holder_per_capture_with_its_processes`. This is new, with no old counterpart.

CHECKS RUN (all green)
- `cargo test -j 2 --no-fail-fast -p dla-cli --test summarize_native --test summarize_robust`: 27 + 6 passed.
- `cargo test -p dla-workload --lib harness::`: 143 passed.
- `cargo test -p dla-cli --test harness_capture`: 11 passed. This includes the real-JVM shim test `a_jni_library_gets_the_shim_and_its_native_stack_resolves_to_the_rust_line`. harness_capture.rs is the other pass's file; I ran it but did not change it.
- Full `cargo test -p dla-bench`: all green (146 lib tests + integration).
- `cargo clippy --workspace --all-targets -j 2 -- -D warnings`: clean.
- `cargo fmt --all --check`: clean.
- `dla schema --check`: ok.
- `python3 bench/runner/test_grade.py`: 190 OK.
- `dla-bench dynamic --dla-bin ./target/debug/dla --parallel 1 --only <f>`:
  - jni-order-codec: pass, 1 confirmed, leaks 2/2, no failures. Same as u8acc. Both shim summaries pass their gate (`gate pass, tied jvm/F19`).
  - native-vendor-codec, rust-private-registry, node-native-addon: pass, no failures. Same as u8acc.
- Outputs are in <sp>/u10/dyn-*.json and dyn-*.log.

NOT DONE / CAVEATS
- The old binary was not run (see the replay above).
- The Missing-record limitation text built in run.rs has no direct test.
- The fix cannot be shown to remove the CI flake itself, because the flake never reproduces locally. Only the replay and the tests show that the choice is now by identity.