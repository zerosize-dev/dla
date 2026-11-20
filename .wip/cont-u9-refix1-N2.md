Package N2 refix: every "problem" and "minor" item is handled (seven minors, three with code or test changes, four wording or doc). The four notes needed nothing. Nothing is committed or pushed. I did not re-measure the 16-copy load, so the stated limit rests on the checker's figures.

**Item states**

1. **Warm-up test fails under heavy load (minor, `alloc_test_jvm.rs`, VERIFY row 509): fixed by stating the limit, not by loosening the relation.**
   - The checker's own data says the 0.75 relation separates warm from cold: two cold runs measured 0.85 to 1.22 on it.
   - At the brief's load (four at once) and at eight at once, nothing failed. At sixteen copies at once, the failures were all that relation, summed over two pairs. The 0.85 relation never failed a two-pair sum.
   - The cause is the warm-up's 3 s cap, which warms less on an overloaded machine.
   - Loosening 0.75 would put it inside the cold range and make the test unable to tell a missing warm-up from a present one.
   - The test comment and row 509 now say it is the first relation to give way (4 of 16 on each JDK at sixteen copies, even with the second pair).
   - Re-run after the comment edit: `alloc_test_jvm` passes, 9 of 9 on JDK 21.
   - Code behaviour is unchanged, so there is no new test for it.

2. **VERIFY row 518, "total 5" (minor): corrected to "total 6 where 2 is expected".** The test feeds four recorder threads, so the skip off gives 6. The "total 3" figure for leaving out only `JFR Recording Scheduler` stays. I did not re-run it; the checker had run it on a scratch copy and got 6.

3. **VERIFY row 488, inexact warm-up sentence (minor): rewritten.** It now says the test asserts the rise from the first reading to point 15 under 0.85 of the cold rise, measured once more before failing. On Linux with a cold rise of 250 KB or more it also asserts the 0.75 relation. It points to the warm-up row.

4. **Coverage agent, class location spelled through a link named like a test directory (minor): covered by a new test, and the fix already covers it.**
   - New case in `harness_jvm_coverage::locations_and_the_source_file_attribute_are_read_by_the_agent`.
   - The case: a link `outer/java/test` pointing to `ws/main`, a location spelled through it, and the project given by its real path `ws/main/proj`.
   - Seen failing: with the HEAD `DlaDump.java` the test prints `FAIL location spelled through a link named test above the project: got true, want false`. With the current agent all 7 tests in the file pass.
   - A second case, the same location with the project given through that link too, passes with the old agent. It is judged by reading and the row says so.
   - The checker's wording ("link named `out/test`") did not reproduce as a failing case. `hasTestPart` only counts a `test` part that follows `kotlin`, `java`, `scala`, `groovy`, `classes` or `out`, so my link sits under `java/`.
   - I restored the current `DlaDump.java` from a copy and `git diff` shows it unchanged from before this refix.
   - Row 519 now describes this case.

5. **Over-long comment lines in `harness_driver_build.rs` and `harness_junit.rs` (minor): wrapped.** The comment now ends on its own line, `(kotlin_compiler_runs)`.

6. **Wording in `alloc_test_jvm.rs` and row 509 (minor): corrected.** The 0.85 line now "lies between the two ranges measured here but inside the one measured elsewhere (0.72 once)". The comment says "250 KB or more", matching the code.

7. **Row 519 ends "confirms the link spellings there" (minor): reworded.** It now says the link spellings on the macOS runner are "to be confirmed in CI", matching the status.

**Notes, no change needed:** row 511 (the tests that failed only locally stay unnamed because the count was dropped), the mixed-spelling location check (`below`, unchanged by this pass), the plan names left in M7 rows the items did not name, and the early-rise relation being keyed to Linux rather than to the CPU.

**CI run of 9781542 (the last step):** I did not re-run `ci-wait-branch.sh`. The fixer's result stands: run 44608346577 concluded `failure` (job `dynamic acceptance on the fixtures`, fixture `jni-order-codec`, 58 of 60 passing plus 1 expected failure), and the checker confirmed it from the log. No "to be confirmed in CI" row was changed, and the two edited rows keep "to be confirmed in CI".

**Files changed in this refix**
- `/home/user/dla/crates/dla-workload/tests/harness_jvm_coverage.rs`
- `/home/user/dla/crates/dla-cli/tests/alloc_test_jvm.rs` (comment only)
- `/home/user/dla/crates/dla-workload/tests/harness_driver_build.rs` and `/home/user/dla/crates/dla-workload/tests/harness_junit.rs` (comment wrap)
- `/home/user/dla/docs/VERIFY.md`: rows 488, 509, 518 and 519 only, re-read before each edit. All M7 table rows still have 4 cells, checked by a script that respects `\|`.

**Checks run**
- `cargo test -j 2 --no-fail-fast -p dla-workload`: all test binaries ok, no failures.
- `harness_jvm_coverage`: 7 of 7 pass.
- `alloc_test_jvm`: 9 of 9 pass.
- `cargo clippy --workspace --all-targets -j 2 -- -D warnings`: clean.
- Windows-gnu clippy on `dla-core`, `dla-probe` and `dla-report`: clean.
- `cargo fmt --all --check`: clean.
- `target/debug/dla schema --check`: ok.
- `python3 bench/runner/test_grade.py`: OK.

**Only CI can confirm:** the macOS runner for the 0.85 line, the Linux-only early-rise relation and the link spellings of the coverage agent. How often `jni-order-codec` fails is also open, since one green run cannot show it.