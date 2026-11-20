Package N2: all 14 items are handled; nothing is committed or pushed. Every test binary of the four crates I touched passed (71 "test result: ok" lines, none failed), and all the required checks are clean (they ran while the other package's changes were in the tree). Item 6 did not turn green, so no "to be confirmed in CI" row was changed.

**Item 1: JVM warm-up test (alloc_test_jvm.rs)**
- Cause: the start-value and first-step assertions depend on how much CPU the JVM gets, because the agent's warm-up stops after 3 s. I reproduced it: 2 of 88 loaded pairs measured here failed the old assertions with the warm-up working (JDK 25 first step 64 KB cold against 51 KB warm; JDK 21 ratio 0.84). The 500 KB line also switched the early-rise comparison off on JDK 25 (cold rise 0.33 to 0.42 MB here).
- Fix, test only: the test now asserts one relation. The rise from the first reading to point 15 must be under 0.85 of the cold run's. If the first pair fails, a second pair is measured and the two are judged together (sums).
- On Linux only, a second relation applies when the cold rise from point 1 to point 15 is 250 KB or more: the warm rise must be under 0.75 of it. It runs on JDK 25. It is skipped on macOS, where the rise is the drift alone.
- Measured here with four harness runs at once, JDK 21 and 25:

| Pairs | Ratio of rise from first reading, warm over cold |
|---|---|
| Warm-up on, 88 pairs | 0.28 to 0.54 |
| Two cold runs, 64 pairs | 0.88 to 1.15 |

  The checker's figures were up to 0.58 on Linux, 0.66 and 0.74 on macOS, and 0.72 to 1.38 for two cold runs. The second relation measured 0.29 to 0.64 here.
- With three other copies of the test beside each run: warm-up on passed 20 of 20 on JDK 21 and 20 of 20 on JDK 25 (no run needed its second pair). With `DLA_WARM_ROUNDS=0`, 12 of 12 failed on JDK 21 and 12 of 12 on JDK 25.
- The VERIFY row now gives these as ranges and drops "true on every build". The 0.85 line is close to the macOS figures (0.74 at most), so only the macOS runner can confirm it.

**Item 2: JNI test guard (harness_capture.rs)**
- New helper `kotlin_compiler_runs`: it compiles a one-line Kotlin file with the compiler `build.sh` would pick (`kotlinc`, else the embeddable 2.0.21 compiler of `~/.m2`). It skips with the reason printed when that compiler does not run on the JDK in use.
- Seen failing: the old guard on JDK 25 failed with `IllegalArgumentException: 25.0.5.1`. The new guard skips on JDK 25 and the test runs and passes on JDK 21 here.

**Item 3: flight recorder's fourth thread**
- `JFR Recording Scheduler` is added to the list in `thread_dump.rs`, to its unit test and to the limitation line in `jvm_nonheap.rs`.
- Seen failing: the unit test fails with the name removed (total 3 where 2 is expected).
- I did not run `dla attach` or take real dumps myself; the evidence is the checker's dumps, quoted in the row.

**Item 4: coverage agent and links (DlaDump.java)**
- Cause confirmed: `isTestLocation` judged the whole location path when the location was under the project only after links were resolved.
- Fix: `isTestLocation` now takes the part below the project for each nesting pair of written or resolved location and root. The whole location is used only when no pair nests.
- Tests: new `harness_jvm_coverage::a_project_below_a_directory_named_test_and_given_by_a_link_counts_its_classes` (project 0 where 2 expected with the old agent, seen failing). New cases in `locations_and_the_source_file_attribute_are_read_by_the_agent`: the first case was seen failing, and the rest are judged by reading.
- `harness/jvm/README.md` got one sentence, and VERIFY got a new row at the end.

**Item 5: VERIFY rows and comments**
- Item 1 of the checker's list (the JDK 22 to 24 pin) was right. The row and both test comments (`harness_driver_build.rs`, `harness_junit.rs`) now say it is judged by reading.
- Item 3 (stale comments): `attach.rs` now says "above", and the `Sleeper::start` doc in `attach_tools.rs` is corrected. The other "helper test below" comments are right because the helper really is below there.
- Item 4 (over-long line): the line in `jcmd.rs` is wrapped.
- Item 5 (row 511): it now names the six tests that failed on the runner and gives no local count, since that count depends on the stand-in `sleep`.
- Items 10 to 13:
  - Row 466: I ran it myself with the rule made to never apply, and 13 of the 20 `gate_harness` tests fail. All 13 and the 7 that pass are named. The one `gate.rs` edit was reverted at once (`git diff` shows no change in `dla-core`).
  - The row 467 title is now "Skill text for running a generated harness before closing".
  - The two "last U4 row" references name "Fixes after the second independent check of the JUnit driver".
  - The title of row 454 and its pointer in row 447 are reworded. The pointer now points to the backlog row about a fixture's `[toolchain]`.
  - The `P3/P4 (E2, P8')` status cell in row 455 is reworded by what it was about.
- Plan names remain outside my rows: rows 438, 450, 451 and 469, and the "Verification backlog" rows around 594 to 596 (`B0`, `P7`). I left them because they were not named in the items.
- The text claims about thread-dump behaviour, links and the guard were checked by running the tests above, not by reading alone.

**Item 6: CI run of commit 9781542**
- Run 44608346577, conclusion `failure`; the other jobs succeeded. The failing job is `dynamic acceptance on the fixtures`. The failing fixture is `jni-order-codec`, with `label-evidence-missing [summarize alloc-stack-diff via shim-alloc-stacks: gate fail, tied none, reasons: nothing grows strictly across all 17 captures`; the run had 58/60 passing plus 1 expected failure.
- This is the rare failure that row 447 item (9) already describes. It is not part of N2 or N1.
- Because of this conclusion I left every "to be confirmed in CI" row unchanged.

**Files changed**
- Product: `crates/dla-parsers/src/jvm/thread_dump.rs`, `crates/dla-cli/src/summarize/jvm_nonheap.rs`, `harness/jvm/DlaDump.java`.
- Comments and tests: `crates/dla-sample/src/attach/jcmd.rs`, `crates/dla-sample/tests/attach_tools.rs`, `crates/dla-cli/tests/attach.rs`, `crates/dla-cli/tests/harness_capture.rs`, `crates/dla-cli/tests/alloc_test_jvm.rs`, `crates/dla-workload/tests/harness_driver_build.rs`, `crates/dla-workload/tests/harness_junit.rs`, `crates/dla-workload/tests/harness_jvm_coverage.rs`.
- Docs: `harness/jvm/README.md`, and `docs/VERIFY.md` rows 447, 454, 455, 461, 466, 467, 509, 511, 512, 513, 518 plus one new row at the end of the M7 table. Every row in the M7 table still has 4 cells.
- Not mine (the other package's): `judge.rs`, `pick.rs`, `alloc_cmd.rs`, `alloc_test.rs`, `evidence-and-report.md`.

**Checks run**
- `cargo test -j 2 --no-fail-fast` on `dla-parsers`, `dla-sample`, `dla-workload` and `dla-cli`: all passed, on JDK 21 as default PATH.
- `cargo clippy --workspace --all-targets -j 2 -- -D warnings`: clean.
- `cargo clippy --target x86_64-pc-windows-gnu -p dla-core -p dla-probe -p dla-report --all-targets -j 2 -- -D warnings`: clean.
- `cargo fmt --all --check`: clean.
- `target/debug/dla schema --check`: ok.
- `python3 bench/runner/test_grade.py`: OK.

**Not done, and what only CI can confirm**
- The macOS runner has to confirm the 0.85 line and the Linux-only early-rise relation.
- The macOS runner also has to confirm the link-spelling cases of the coverage agent.
- I did not run the other package's fixture runs; the scratch directory is removed.
- I ran `prune-superseded.sh` once, when disk fell below 3000 MB; it freed about 4 GB and nothing of mine failed afterwards.