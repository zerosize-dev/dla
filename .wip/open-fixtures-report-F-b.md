# F-b report (Python census and agent)

Items: 1 census gc-freeze DONE; 2 agent group py.gc_frozen DONE; 3 catalog entry DONE; 4 codec handle vocabulary DONE; 5 tests and docs DONE.
Causes confirmed on code before changing: python.rs had no gc call and no codec in the handle row; sitecustomize.py had no groups and built types from gc.get_objects(); series.rs defaulted py.gc_frozen to bytes. Brief and verdicts agreed; deviation: no unfreeze removers recorded (tracked name gc finds none), usage.rs untouched.

Files: crates/dla-census/src/lang/python.rs; harness/python/sitecustomize.py; harness/python/README.md; crates/dla-core/src/series.rs (one column_meta line + test lines); crates/dla-workload/tests/harness_capture_python.rs; crates/dla-cli/tests/alloc_test.rs; skills/detect-leak-anywhere/references/python.md; docs/VERIFY.md (4 rows in M7 table).

Tests (seen fail without the change): python.rs gc_freeze_in_a_function_is_listed_and_other_gc_calls_are_not and native_codec_contexts_are_handles_and_one_shot_functions_are_not (both failed with the change disabled); harness_capture_python.rs four freeze tests (3 failed with wrapper not installed; sum test failed with cumulative shares); series.rs names_map_to_the_documented_meaning (failed without the line); alloc_test.rs frozen_garbage_is_tied_to_the_freeze_site_by_a_count_column_and_reads_growth (failed without the line: favors non-leak, plateau). The no-freeze/bad-call test passes by design without the change.
Adversarial inputs tried: census async/nested/branch/module-level `if __name__`, aliases (zstandard as zs), type tests/annotations, lookalike names; agent: 4 threads freezing, `from gc import freeze`, unfreeze zeroes shares, bad call gc.freeze(1).

Checks: cargo test -j 2 --no-fail-fast -p dla-census -p dla-core -p dla-workload -p dla-cli: 84 test binaries all ok (run before last census test additions); then cargo test -p dla-census (78 ok), skill_text ok; clippy --workspace --all-targets -- -D warnings clean; cargo fmt --all --check clean; dla schema --check ok (20); python3 bench/runner/test_grade.py 190 OK. Windows clippy not run (dla-core touched only by a match arm and test; not run).
Bench (copies of target/debug binaries; scratch fixtures dir with 3144, 3145 + pair line, frozen-gc):
- dla-bench static --archetypes on/off on scratch: 3/3 pass, frozen-gc L1 and zstd-3144 L1 flagged by census.
- dla-bench static on bench/fixtures both modes: 60 pass; vs u11acc/static-off.json only python-zstd-options-3145 changed (2 -> 4 candidates, +2 census at reader.py:17, :21); python-graph-churn unchanged.
- dla-bench dynamic --parallel 1 --only python-frozen-gc: pass (claim, rule-out, UNCLASSIFIED).
- dynamic --only python-zstd-options (scratch pair): 3145 passes no claim; 3144 fails "leak L1 (archive/reader.py:17) is not confirmed": F1 resolved suspected_static_only/not_exercised, alloc-stack-diff gate pass "tied none" (needs the tie rule, not in this pass), refuted 1, passed columns 13.
- dynamic --only python- on bench/fixtures: 8/8 pass; vs u11acc/dynamic.json same confirmed/claims; passed_columns differ on os.* columns (graph-churn 5->4, pool-frag 1->3, memo-bounded 3->2, retry-chain 18->21, 3145 1->0, refuted 3->4 from the 2 new candidates).
Unverified: other Python versions (only 3.13 here); interpreters without gc.freeze; full dynamic of non-Python fixtures; Windows clippy; static compare used the u11acc static-off.json baseline, not a fresh before-run.
