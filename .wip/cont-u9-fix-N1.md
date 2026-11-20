N1 REPORT (alloc-test reads a flat column as inconclusive when it happens to vote). Nothing committed or pushed. Both items are fixed and covered by tests.

STATE
- Item 1 (purge test calls a few KB of live growth "retained"): fixed. Every test written for it ran here.
- Item 2 (os.rss_file votes and always reads "retained"): fixed. Every test written for it ran here.
- Item 3 (look for more of the same): done, and I changed nothing more. The list is below.
- Only the N1 package was done. N2 and the VERIFY row edits that belong to it are not mine.

CAUSE, WITH EVIDENCE
1. Purge test vs plateau test.
   - judge::purge called any growth above 1e-9*level real growth. The plateau test of the same column treats a rise of at most MIN_BYTE_RISE (32 KiB) as a plateau.
   - go-gc-tuned-batch showed this in the earlier run's artifacts (alloc A1.json and A2.json, in the earlier run's dynamic.logs): go.heap_objects had purge growth 17310 and 20190 B and a drop of -1160. That is live objects, which a purge cannot return.
   - After the fix the same column passed the gate again by chance in my run. It read purge `no-growth` (growth 15880 and 19530 B) with a non-leak verdict.
2. os.rss_file.
   - I read the purge code of every harness: Python (sitecustomize.py), Node (dla-preload.js), JVM (DlaDump.java), Go (the two .tmpl files), native C driver, allocator shim and Rust driver.
   - No purge kind gives file-backed pages back. malloc_trim, FreeOSMemory, systemTrimNativeHeap and a forced gc release anonymous memory.
   - The Python trim purge demonstrably maps files itself. A local check showed RssFile 8592 kB, then 8796 kB after ctypes.CDLL, then 9096 kB after malloc_trim.
   - A local run through the prepared-trend test confirmed it. os.rss_file read plateau with purge retained and did not vote.
   - For the other purge kinds the mechanism (they run code not run before, so new file pages count as resident) is judged by reading, not measured. I could not reproduce those here.

FILES CHANGED
- crates/dla-cli/src/alloc/judge.rs: `purge()` takes `min_rise`. A growth at or below `max(1e-9*scale, min_rise)` is NoGrowth. `combine` is unchanged. Tests added.
- crates/dla-cli/src/alloc_cmd.rs: `test_column` passes its `min_rise` (32 KiB for a byte column, 0 otherwise) to `purge`. Tests added.
- crates/dla-cli/src/alloc/pick.rs: `os.rss_file` joins NO_RELEASE, with its reason in the comment. If nothing else can vote, passed NO_RELEASE columns now vote (new fallback). Tests added.
- crates/dla-cli/tests/alloc_test.rs: one command-level test.
- skills/detect-leak-anywhere/references/evidence-and-report.md: states the 32 KiB `no-growth` rule and the non-voting columns including os.rss_file.
- docs/VERIFY.md: two new rows at the end of the M7 table. One covers the purge minimum (corrected), the other os.rss_file (corrected). No plan or package names, no model names.
- I grepped for readers of "no-growth" and the verdict words. Only the old-ledger test data and judge.rs match. Verdict words are unchanged, so no reader needed an update.

TESTS (all in dla-cli; names as in the repo)
- judge::tests::a_growth_at_or_below_the_columns_minimum_rise_is_no_growth_for_the_purge: below, at and above the minimum for a byte column, and unchanged for a column with no minimum. SEEN FAILING with the minimum removed.
- alloc_cmd::tests::a_flat_go_column_with_a_few_kb_of_live_growth_reads_non_leak_not_retained: the Go shape, plateau with 17 KB growth, purge no-growth, non-leak. SEEN FAILING with the minimum removed.
- pick::tests::a_file_backed_column_that_passed_the_gate_is_tested_and_does_not_vote_beside_another: SEEN FAILING with os.rss_file taken out of NO_RELEASE.
- pick::tests::a_file_backed_column_votes_when_nothing_else_can: a guard. It also passes without the change, so it was not seen failing, and I judged it by reading.
- alloc_cmd::tests::growing_file_backed_memory_beside_growing_rss_still_reads_leak_and_names_both: the leak case. os.rss and os.rss_file both keep growing, the verdict is leak, and both columns are in the result and read leak. SEEN FAILING with the column taken out of the list.
- alloc_test::a_flat_file_backed_column_that_passed_the_gate_does_not_decide_a_flat_run: a real Python harness run of python-report-memo-bounded, with its first trend edited so that only os.rss_file passes. It reads non-leak with py.traced voting and os.rss_file not voting. SEEN FAILING with the column taken out of the list. It skips off Linux, where the column is not sampled.

FIXTURES
- I ran `dla-bench dynamic --parallel 1 --only <fixture>` for all 12, with copies of the binaries, on this tree.
- Baseline: I used the earlier run's dynamic.json (u8acc) as "before". I did not rebuild the old tree.
- All 12 pass. Counts (refuted, confirmed, rule_outs, leaks_found, passed_columns), after against before:
  - go-gc-tuned-batch 3, 0, true, 0, 2 against 3, 0, true, 0, 2
  - go-buffer-pool 5, 0, true, 0, 0 against 5, 0, true, 0, 0
  - go-ticker-modern 7, 0, true, 0, 0 against 7, 0, true, 0, 0
  - python-pool-fragmentation 3, 0, true, 0, 2 against 3, 0, true, 0, 2
  - jvm-order-cache-bounded 4, 0, true, 0, 0 against 4, 0, true, 0, 0
  - native-arena-reuse 6, 0, true, 0, 0 against 6, 0, true, 0, 0
  - python-report-memo-bounded 1, 0, true, 0, 3 against 1, 0, true, 0, 2
  - rust-tree-parent-links-weak 0, 0, false, 0, 4 against 0, 0, false, 0, 3. Same verdict as before and no rule-out, and the run still passes.
  - python-graph-churn 2, 0, true, 0, 2 against 0, 0, false, 0, 3. This was a failing fixture before.
  - node-price-board-flat 3, 0, true, 0, 0 against 0, 0, false, 0, 4. This was a failing fixture before.
  - node-template-cache 3, 0, true, 0, 0 against 1, 0, true, 0, 2
  - rust-batch-buffers 3, 0, true, 0, 1 against 2, 0, true, 0, 0
- The differences in refuted and passed_columns come from which columns passed the first trend by chance. I did not trace them further.
- The failures cannot be forced by rerunning, so the proof is the prepared-series tests above, not a loop.

ITEM 3: OTHER COLUMNS WITH "purge retained"
- I looked at the diagnostics of the three runs named in the brief (u8acc, u7acc, p2r3-acc/on). I also read every alloc A*.json of u8acc (voting flag, purge growth and drop).
- Fixed by the change:
  - go.heap_objects in go-gc-tuned-batch and go-ticker-modern. Same mechanism. Their extra-window rises were only a few KB.
  - os.rss_file in python-graph-churn.
- Left unchanged, because no mechanism other than real growth could be named:
  - os.vm_hwm, os.vm_data, native.peak_bytes: already non-voting.
  - os.rss: it votes. Its retained growth is megabytes that a trim or gc did not return (node-price-board-flat and node-template-cache in some runs). It is not a purge-step artifact.
  - os.rss_anon in rust-tree-parent-links-weak: 68 KB since the warm-up, trim returned nothing. That is above the 32 KiB minimum, and I found no mechanism.
  - node.heap_used in node-socket-heartbeat-cleared: about 250 KB of live growth since the warm-up, real retained growth.
  - go.total, node.heap_total and node.heap_used in the leak fixtures: real growth.
- Open: node-price-board-flat, node-template-cache, python-report-memo-bounded and rust-tree-parent-links-weak can still read inconclusive when os.rss passes the first trend by chance and votes. I did not change that, per the brief.

CHECKS RUN (on the whole tree, including the other package's edits at that moment)
- `cargo test -j 2 --no-fail-fast -p dla-cli`: every test binary ok, exit 0.
- `cargo clippy --workspace --all-targets -j 2 -- -D warnings`: clean.
- The Windows-gnu clippy command from the brief: clean.
- `cargo fmt --all --check`: clean.
- `target/debug/dla schema --check`: ok.
- `python3 bench/runner/test_grade.py`: 190 tests OK.
- Only dla-cli was touched, so only its tests were run. I did not run other crates' test suites.
- Scratch dir deleted by literal path.

ONLY CI CAN CONFIRM
- That the two chance events (a column passing the first trend that normally does not) stay absent over many runs on the runners. One green run cannot show that.
- The Linux-only command-level test on the macOS and Windows jobs: it should print its skip reason there.
- That the 12 fixtures pass on the runners' toolchains.