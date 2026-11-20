FIXTURE ACCEPTANCE OF THE WORKING TREE (main at 9781542 plus the uncommitted work of passes A and B)

Outcome: all steps pass. Static 60/60 with and without archetypes; dynamic exit 0 with 59 pass and 1 xfail (rust-tree-parent-links), 0 failures. python-graph-churn, the one failure of the earlier run, passes. No fixture failed, so step 5 had nothing to rerun. Two things to know: the tree's docs changed under the run (step 1), and the shim summaries of jni-order-codec now read 9 captures where the earlier run read 17 (step 6).

sp = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad

STEP 1, build and copy: PASS
- `cargo build -j 2 -p dla-cli -p dla-bench`: exit 0, nothing to compile (log: <sp>/u11acc/build.log).
- At the copy (03:33:30 UTC): `git status --short | wc -l` = 29; `git diff HEAD | sha256sum` = e8705e366cb0c55eaa062941078a2048359e3132f4bd870acf9614b07ea64bd7.
- Copies in <sp>/u11acc/bin/: dla sha256 4e1cc4b9..., dla-bench 715b8cbd.... All runs below used these copies.
- The tree changed during the run. At the end: 29 files, diff sha256 bec8f90cdc8dcf32.... Only docs/VERIFY.md and harness/jvm/README.md were modified after the copy (both 04:27:50 UTC). No file under crates/, bench/, harness/ (except that README), skills/, agents/ or data/ is newer than the copy, and the READMEs are not compiled in. Diff without those two files: sha256 55e0cd9b4debb5f0....
- Another agent rebuilt target/debug/dla at 04:12 and dla-bench at 03:44; the copies were not affected.

STEP 2, static: PASS (2 of 2)
- Archetypes on: exit 0, "ok: 60 fixture(s) pass" (<sp>/u11acc/static-on.out).
- `--archetypes off`: exit 0, "ok: 60 fixture(s) pass" (<sp>/u11acc/static-off.out).
- I added `--json` to both calls. `dla-bench compare` says both result files equal those of <sp>/u8acc.

STEP 3, dynamic: PASS
- Exit 0 after 91 min 41 s (03:45:50 to 05:17:31 UTC), `--parallel 2`.
- "totals: 59/60 pass, 1 xfail (expected failures, not counted as failures), 19 confirmed, 122 refuted, 25 with a claim, 50 with a rule-out"; "ok: 59 fixture(s) pass, 1 expected failure(s)".
- xfail: rust-tree-parent-links, expected failure "leak L1 (src/section.rs:27) is not confirmed".
- Six fixtures carry one error each, as in the earlier run (all category impossible, harness did not build or run): go-private-module, jvm-billing-private-repo, native-vendor-codec, node-native-addon, python-vendor-wheel, rust-private-registry.
- Load: another agent's `cargo test -j 2 --no-fail-fast --workspace` ran beside it for about the first 50 minutes (load average 13 to 20).
- Files: <sp>/u11acc/dynamic.json, <sp>/u11acc/dynamic.out, <sp>/u11acc/dynamic.logs/.

STEP 4, comparison with <sp>/u8acc/dynamic.json (full listing: <sp>/u11acc/compare.txt)
Earlier totals: 58/60 pass, 1 xfail, 19 confirmed, 121 refuted, 25 claims, 48 rule-outs. No fixture differs in confirmed, claims or leaks found. Ten fixtures differ in pass/fail, refuted or rule-out:

1. python-graph-churn (non-leak): FAIL to pass, refuted 0 to 2, rule-out no to yes.
   - Earlier: os.rss_file passed the first trend, voted, and read plateau with purge retained (10.85 to 11.22 MB across the purge); both unbound tests were inconclusive.
   - Now: os.rss_file did not pass the first trend; every voting column read non-leak (details in step 6).
   - The purge step no longer moves the column: os.rss_file changes by 0 B in all 32 Python purge records of this run, against +131072 to +368640 B in all 30 of the earlier run. os.rss never rises across the purge now (largest change -4096 B; earlier median +544768 B).
2. python-report-memo-bounded (decoy-control): refuted 1 to 4.
   - Earlier: os.rss voted and read plateau with purge retained, because RSS rose 553 to 557 KB across the trim purge itself; both unbound tests were inconclusive.
   - Now: os.rss falls 12 to 16 KB across the purge and reads no-growth; both unbound tests read non-leak.
3. python-zstd-options-3145 (decoy-control): refuted 2 to 3. py.traced passed the first trend this time (earlier: nothing), so a region finding F3 was added and refuted.
4. go-buffer-pool (non-leak): refuted 5 to 6. os.vm_hwm passed the first trend this time (earlier: nothing), so a region finding F6 was added and refuted.
5. go-gc-tuned-batch (non-leak): refuted 3 to 2. Nothing passed the first trend this time (earlier: go.heap_live and go.heap_objects, whose region finding F3 was refuted).
6. go-ticker-modern (decoy-control): refuted 7 to 1; rule-out still yes.
   - go.heap_live and go.heap_objects passed the first trend this time (stair, rise 2842 B; earlier: nothing), so os.rss voted.
   - Unbound test A1 read inconclusive through os.rss alone: plateau, purge retained (before 6.009 MB, after 5.915 MB, growth since warm-up 622.6 KB, drop 94.2 KB, noise of the column 708.9 KB). The 7 open candidates stayed open.
   - go.heap_objects read non-leak with purge no-growth (growth 9208 B, below the 32 KiB minimum). This is the new purge minimum of pass A at work on a voting column.
   - The region finding F8 (growth in go.heap_live) was refuted by its own test A2, where os.rss read released.
7. jvm-extension-reload-stopped (decoy-control): refuted 1 to 3. Earlier the unbound test read leak (jvm.heap_used and jvm.heap_used.pre_gc growth-continues, rise 74.8 KB against noise 10.9 KB). Now the same columns plateau and os.rss reads released, so it refuted 2 of 2. F4 (growth in jvm.heap_used) stays unresolved in both runs.
8. jvm-invoice-render-trimmed (decoy-control): refuted 8 to 4.
   - os.rss and os.rss_anon passed the first trend this time (stair, rise 44 MB; earlier only os.vm_hwm, which does not vote).
   - Both unbound tests (A5, A6) read leak through them: growth-continues (+27 MB over the extra window), purge retained (5.6 to 11.6 MB given back of 93 MB growth).
   - Three open candidates and F8 (growth in os.rss) stayed unresolved. No claim was made, so the fixture passes. It ran while the load average was 13 to 20.
9. node-price-board-flat (decoy-control): refuted 0 to 4, rule-out no to yes.
   - Earlier: os.pss, os.pss_anon and os.private_dirty passed the first trend, so os.rss voted and read plateau with purge retained (the Node gc purge gave back 0 of 2.27 MB); inconclusive.
   - Now: only os.vm_data passed, which does not count, so node.heap_used decided: plateau, purge no-growth (growth 22420 B). os.rss read plateau and retained again but was reported, not voting.
10. rust-alloc-threads (non-leak): refuted 10 to 9. Nothing passed the first trend this time (earlier: os.pss, os.pss_anon and os.private_dirty, whose region finding F10 was refuted).

Nineteen more fixtures differ only in the count of columns that passed a trend gate (old to new): jni-order-codec 14 to 10, jvm-extension-reload 36 to 35, jvm-invoice-render 8 to 12, jvm-order-cache 13 to 15, jvm-pool-warmup 1 to 2, jvm-rule-compiler 22 to 23, jvm-session-index 16 to 19, jvm-session-index-pruned 0 to 1, native-glyph-cache 2 to 4, native-split-freelist 12 to 9, node-gateway-merge 14 to 17, node-price-board 10 to 11, node-signal-compose-22 8 to 1, node-template-cache 2 to 1, python-pool-fragmentation 2 to 1, python-report-memo 17 to 20, python-retry-chain 21 to 18, rust-ledger-sync 17 to 20, rust-tree-parent-links 13 to 16. The logs show no cause beyond different columns passing; their verdicts are unchanged.

STEP 5, reruns of failures: nothing to rerun
- No fixture failed, so there are no failing-run logs to keep.
- Neither failure text occurred in any row: no "no rule-out on a non-leak fixture", no "label-evidence-missing", no "holder absent".
- Scan of all 131 alloc files of the run (43 unbound):
  - os.rss_file was not a tested column in any of them, so the pass A rule for that column was not exercised by this run.
  - The new purge minimum applied on voting columns in go-ticker-modern (go.heap_objects, growth 9208 and 9752 B) and node-price-board-flat (node.heap_used, growth 22420 and 22380 B).
  - The only non-leak fixture with an unbound test that did not read non-leak is node-template-cache (A2, A3 inconclusive: os.rss plateau, purge retained, 0 B given back of about 19 MB). The earlier run had the same; its rule-out comes from the probe-bound test of F2.
- One chance path remains open, seen in go-ticker-modern (item 6): a heap column passes the first trend by chance, os.rss then votes, and os.rss reads retained on a growth inside its own noise. On a decoy-control it costs nothing. On a non-leak Go fixture it would need the region finding's own test to read the same way before the rule-out is lost.

STEP 6, jni-order-codec
- Full run: `type-histogram-diff via shim-stack-groups` (log 043) and `alloc-stack-diff via shim-alloc-stacks` (log 044) each read 9 captures, p1-native to p9-native/shim-dump.jsonl, ordered by x. Both: gate pass, tied jvm/F19, passed `stack:Java_shop_codec_Native_open (native/src/lib.rs:40)`.
- The earlier run read 17 (p1 to p17). The split file is now written only by the capturing run (9 points); the 8 extra points left by alloc-test keep a dump.jsonl but get no shim-dump.jsonl.
- Which process, full run: its logs do not name the pid per capture, because the summaries' raw files (histograms/H2.json, H3.json) are not collected into the log dir. What they show:
  - The holder is present at all 9 captures, from 6836576 B in 278 blocks at capture 1 to 61430816 B in 2498 blocks at capture 9 (series S19 and S20). The last figure equals the harness's own figure for the measured process.
  - Evidence jvm/E27 to jvm/E30 carry no "several processes" limitation.
- Which process, directly: I ran the fixture once more alone (exit 0, 1/1 pass, same figures: 1 confirmed, 5 refuted, leaks 2/2) and copied the raw files while it ran.
  - H2 and H3 each list 9 captures with pids 5932, 6053, 6169, 6281, 6403, 6519, 6643, 6759, 6874 and argv ending `shop.codec.MainKt <n>`, n = 2220 to 19980. These pids equal the capture directory names, the process the harness measured.
  - Each of the 17 dump.jsonl files holds two shim exit records with stacks: the test JVM (`shop.codec.CodecTestKt`, 18.0 to 45.8 MB live) and the program (37.7 to 137.7 MB). Each of the 9 shim-dump.jsonl files holds one record, the program's.
  - The test JVM's record was not the larger one at any of the 17 points (closest at p1: 40.87 against 52.18 MB). So the chance event behind the CI failure did not occur in my runs; the choice by identity is shown by the file contents, not by a run where size would have chosen wrongly.
- Files: <sp>/u11acc/jni-snap/dump-records.txt, <sp>/u11acc/jni-snap/histograms/, <sp>/u11acc/jni-solo.json, <sp>/u11acc/jni-solo.logs/.

STEP 6, the three fixtures' unbound alloc-tests (full run)
- go-gc-tuned-batch (non-leak, refuted 2, rule-out yes): one unbound test, E14, on series S3 (points 1 to 9 from S1, 8 extra from S2), purge free-os-memory.
  - The first trend passed nothing, so the basis is "no column of memory in use passed the gate ...: the runtime heap (go.heap_live) decides, os.rss is reported".
  - Voting: go.heap_live only: plateau (rise 11160 B, noise 2710), purge not-run; non-leak.
  - Reported: os.rss plateau, purge released (23.33 to 7.99 MB); os.vm_hwm plateau.
  - Verdict non-leak, refuted 2 of 2 open candidates. go.heap_objects did not pass the first trend, so it was not tested here.
- python-graph-churn (non-leak, refuted 2, rule-out yes): two unbound tests, E13 (series S3) and E14 (series S5), purge trim, basis "every column that passed the gate in the latest trend of the harness, and os.rss".
  - The first trend passed os.rss_anon, os.pss_anon, os.private_dirty (stairs, rise 222 KB, noise 53 KB), py.traced and py.gc_collections.
  - Voting, all plateau and non-leak: py.traced (purge not-run, gc headroom); os.rss_anon (purge released, 29.27 to 19.88 MB); os.pss_anon and os.private_dirty (purge not-run); os.rss (purge released, 40.60 to 31.20 MB).
  - Reported, not voting: os.vm_hwm inconclusive (plateau, retained).
  - E13 refuted 1 of 1 open candidates; E14 refuted F2 (growth in os.rss_anon).
- python-report-memo-bounded (its category is decoy-control; refuted 4, rule-out yes): E13 is bound to F3 (probe column, non-leak). Two unbound tests, E14 (series S5) and E15 (series S7), purge trim, same basis as above.
  - The first trend passed py.traced, py.traced_peak and py.traced.pre_gc.
  - Voting, all plateau and non-leak: py.traced.pre_gc, py.traced_peak, py.traced (purge not-run); os.rss (purge no-growth: 23.90 to 23.88 MB, growth since warm-up negative).
  - Reported: os.vm_hwm non-leak.
  - E14 refuted 2 of 2 open candidates; E15 refuted F4 (growth in py.traced).

DISK
- `df --output=avail -B1M /`: 3418 MB at the start, 3228 at the copy, 5358 at the start of the dynamic run (space was freed by someone else), 4826 MB at the end. Never below 3000, so the prune script was not run.
- <sp>/u11acc holds 212 MB; nothing was deleted. The bench work directories /tmp/dla-bench-dynamic-14246 and /tmp/dla-bench-dynamic-30191 were removed by the tool itself.
- No commit, no push, no repository file edited.