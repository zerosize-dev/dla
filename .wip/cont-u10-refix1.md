REPORT (refix; nothing committed or pushed; all 5 problems and the 3 minors in this pass's files addressed)

CAUSE (verified by the fixer, re-checked by the check)
- Totals of raw_sites[*].live_bytes in /tmp/dla-bench-dynamic-2919/jni-order-codec/.dla/20261113-001707-b611/harness/H1/out/: p1 test JVM (pid 10920) 41858379 against program (pid 10872, argv ends `MainKt 2220`) 47769802; p2 18908890 against 48416586; p17 19407935 against 144893809.
- Every record carries x = the point's iteration count, and only the program's argv carries it.
- The old reader kept the record with the most live bytes per file. When the test JVM's JIT arenas outweighed the program at point 1, capture 1 was the wrong process: "holder absent from dump 1".

REFIX ITEMS

1. Problem: a false "no shim record" limitation for harnesses without a shim.
   - Fixed in crates/dla-workload/src/harness/run.rs. New `shim_preloaded(manifest)` is true only when the preload variable (LD_PRELOAD or DYLD_INSERT_LIBRARIES) holds libdlashim. The limitation text moved into a pure `shim_missing_note(label, preloaded, pid, split)`, which returns nothing unless the shim is preloaded.
   - The manifest decides, as the check asked. This also covers a native harness whose shim `settle_shim_only` leaves in every process.
   - Tests: `a_missing_shim_record_is_noted_only_where_the_shim_is_preloaded`, `the_shim_counts_as_preloaded_only_with_the_shim_library_in_the_preload_variable` and `a_managed_dump_without_a_shim_harness_gets_no_limitation_and_a_run_without_capture_keeps_the_file`.
   - SEEN failing with the preloaded check removed: the first and the third.

2. Problem: a run without `--capture` deleted `shim-dump.jsonl` while `captures.json` still listed it.
   - Fixed in run.rs. `split_shim` now returns at once without `--capture`. Both stale-file removals (command mode and driver mode) happen only in a capturing run, like the other capture files. `dla alloc-test` runs through the same path, so it no longer deletes the files either.
   - Tested in the third test above. SEEN failing with the old removal restored.

3. Problem: argv words unshaped in the limitation line.
   - Fixed in crates/dla-cli/src/summarize/shim_groups.rs. `choose_process` takes the run's Redactor, and the argv words pass `red.name(...)` as in the raw file.
   - Test: `the_limitation_shapes_secret_looking_argv_words_and_omits_empty_brackets`. It checks that `password=hunter2hunter2` is absent in shapes mode and present with raw consent. SEEN failing with the redaction removed.

4. Problem: the VERIFY row was in the wrong table.
   - Moved to the end of the `## M7` table (right before `## Verification backlog`), 4 cells. The R2 table no longer carries it.
   - Re-worded to the check's minor item:
     - It names the adapted split test and every new test.
     - It says which tests were seen failing and that the pure unit tests of the choice, the argv words and the confirmation have no old counterpart and were not run against old code.
     - It says pid and argv are shown for the first capture and the first captures where the holder is absent.
   - The old binary is now stated as run (see the replay below).

5. Problem: decision 4 was done for a label only.
   - Fixed in bench/runner/src/dynamic/policy.rs. New `failed_summary_detail(log, ops)`, capped at MAX_DETAIL_LINES = 8, is used by `missing_evidence_detail` (the label, limited to the fix's operations) and by a refused confirmation (all failed summaries).
   - Test: `a_refused_confirmation_prints_every_failed_summary_up_to_a_cap`. It tests the function, not the confirm call site; I did not revert the call and see it fail.

MINOR ITEMS
- Wording: fixed in the VERIFY row (see 4). The `gate_detail` doc comment now says "the first few".
- bench/runner/src/dynamic/ops.rs `gate_detail`:
  - The two fixed lines ("stacks are keyed by", "capture-series rule") are skipped. At most DETAIL_LIMITATIONS = 3 limitation lines are shown.
  - The holder is matched by exact label first, then by prefix.
  - Test `a_failed_gate_shows_its_limitations_and_the_holder_per_capture_with_its_processes` was extended with the fixed lines, extra lines and a longer same-prefix stack. SEEN failing with the cap and the exact match removed.
- Test gap, raw file against `gate_detail`: `the_record_of_the_process_with_the_iteration_count_wins_over_a_larger_one` now reads the raw file and asserts `captures[].pid` and `argv_tail` (`java shop.codec.MainKt <x>`) for all 3 captures. It runs here, since gcc, nm and addr2line are present. A renamed field would now fail it.
- Test gap, `split_shim` in run.rs: now covered by the three run.rs tests above (note text, preloaded decision, capturing against not capturing).
- Empty brackets (`kept pid 10920 ()`): fixed; the brackets are omitted when argv has no words. Covered by the redaction test, which also checks `!contains("()")`.
- Driver-mode identity: no code change; the check's claim about real driver mode is not supported by evidence here.
  - In driver mode `x` is DLA_ITER and the step times all rounds, so the driver's argv does not carry it. The identity rule can only match by chance.
  - The harness writes exactly one record per `shim-dump.jsonl` in both modes (`split_shim` keeps the measured pid's last record), so the reader has nothing to choose between for harness-written files.
  - Only a hand-given or older multi-process driver file could hit the chance match. Where identity matches nothing, the reader falls back to size, as before, with the limitation line.
  - No driver-mode dump exists in the /tmp/dla-bench-dynamic-* directories (searched), so I could not check it on a real file. This is by reading run.rs only.
- Notes: the fewer-points note (p10 to p17 stale `dump.jsonl` no longer split) stands as designed, because only the points of a capturing run get a file. Both shim summaries still pass and tie.

STATE OF EACH DECISION
1. Harness split: done. `split_shim_dump` writes the measured process's shim record only and writes none when it has none. A limitation is added only where the shim is preloaded. A run without `--capture` leaves earlier files alone.
2. Reader: done. It groups by pid, drops tool processes, prefers the argv carrying the iteration count (helpers exported from dumps.rs and shared with `select_measured`), then falls back to size with a limitation line. The line shows redacted argv words and no empty brackets.
3. Audit of other readers: unchanged from the fixer's report.
   - Fine: `select_measured`, `select_checkpoints`, `capture::gather`/`prune`, vgdb, run/goleak, and the Node note near capture.rs:605 (it only groups commands).
   - Not fixed, outside this brief: `summarize/proc_fd.rs` and `summarize/harness_types.rs` read every process's records in file order.
4. Bench failure text: done for a label and now also for a refused confirmation. The text has limitations (capped), per-capture bytes/blocks, and the pid and argv tail of the first capture and the first absent ones. `dla schema --check` passes; EXPECTED_FAILURES is untouched.

REPLAY (real files, exact method)
- I copied the run directory's `harness` folder into a fresh dir (u10/oldrun). Its p1 shim-dump holds the test JVM's figures doubled, above the program's, and it holds all 17 files.
- I made a fresh run in it with `dla run init`. I ran `summarize type-histogram-diff` on `harness/H1/out/p*-native/shim-dump.jsonl` with both binaries.
- The older release binary (target/release/dla, built 2026-11-12, before this change) FAILS: reasons "nothing grows strictly across all 17 captures" and "stack:Java_shop_codec_Native_open: holder absent from dump 1", tied none.
- The current debug binary PASSES on the same files, with `passed: stack:Java_shop_codec_Native_open`.
- The release binary could not read the other copy's run (schema 7 against 6), so I used the fresh run.

CHECKS RUN (all green)
- `cargo test --no-fail-fast -p dla-cli --test summarize_native --test summarize_robust`: 27 and 6 passed.
- `cargo test -p dla-cli --bin dla summarize::shim_groups`: 6 passed (dla-cli has no lib target).
- `cargo test -p dla-workload --lib harness::`: 147 passed.
- `cargo test -p dla-bench`: all passed (146 lib tests, 147 and others in the integration binaries).
- `cargo test -p dla-cli --test harness_capture` (other pass's file, not changed by me): 11 passed in about 108 s.
- `cargo clippy --workspace --all-targets -j 2 -- -D warnings`: clean.
- `rustfmt --check` on all 9 files of this pass: clean. `cargo fmt --all --check` printed no diff.
- `dla schema --check`: ok (20 checked). `python3 bench/runner/test_grade.py`: OK.
- `dla-bench dynamic --dla-bin ./target/debug/dla --parallel 1 --only <f>`:
  - jni-order-codec: pass, 1 confirmed, 5 refuted, 1 with a claim, 1 with a rule-out. Same outcome as the check's run and u8acc.
  - native-vendor-codec, rust-private-registry and node-native-addon: pass, no failures.
  - The jni-order-codec run took about 15 minutes under load average about 10 from the other pass. A first sequential attempt hit the shell's time limit; I reran each fixture separately.
  - I did not compare against <sp>/u8acc/dynamic.json file by file; I compared the pass and no-failure outcomes only.
- Logs: /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u10/r3-jni.log, r3-rust-private-registry.log, r3-node-native-addon.log, r2-native-vendor-codec.log.

FILES CHANGED (this pass, cumulative)
- crates/dla-workload/src/harness/{capture,dumps,run}.rs
- crates/dla-cli/src/summarize/shim_groups.rs
- crates/dla-cli/tests/summarize_native.rs
- bench/runner/src/dynamic/{ops,policy}.rs and bench/runner/src/dynamic_cmd.rs
- docs/VERIFY.md (one row, now in the M7 table)

NOT DONE / CAVEATS
- The CI flake itself does not reproduce locally. Only the replay and the tests show the choice is now by identity.
- No test covers the confirm call site in policy.rs; the helper is tested.
- Driver-mode identity was not checked on a real dump (none at hand).
- Side notes, not touched: `accept_hidden_tool_gaps` tests `ends_with("label refused label-evidence-missing")`, but the failure line can end with a bracket. The `proc_fd.rs` and `harness_types.rs` readers are not changed.