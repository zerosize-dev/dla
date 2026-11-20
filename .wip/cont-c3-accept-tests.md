ACCEPTANCE REPORT: tests, lint, schema (working tree of /home/user/dla, main at 9781542, uncommitted)

Verdict: every step passed; no test failed, so no steady/flaky reruns were needed. One thing to know first: the tree changed under me once.

TREE CHANGE DURING THE RUN
- From 03:43 to at least 04:27:06 UTC the tree was the state with `git diff HEAD | sha256sum` = e8705e36...64bd7 (29 files, 1879 added lines). Steps 1 to 6 as specified all ran on that state.
- At 04:27:50 someone else edited two doc files: `docs/VERIFY.md` (wording in the rows at lines 509, 520, 521) and `harness/jvm/README.md` (one long line re-wrapped into seven). No code file changed. Final state: hash bec8f90c...e28bf, 29 files, 1885 insertions, 209 deletions; unchanged from 04:33 to 04:35.
- On the final state I re-ran everything that reads those two files (see "Re-checks on the final tree"); all passed. The step 7 scan below is of the final state.

STEP 1: `cargo test -j 2 --no-fail-fast --workspace`, JDK 21.0.12.1 (JAVA_HOME=/usr/lib/jvm/java-21-openjdk-amd64): PASS, exit 0
- 143 `test result:` lines, all `ok`: 2744 passed, 0 failed, 5 ignored.
- Of these, 135 are test binaries (124 Running + 11 Doc-tests): 2736 passed, 0 failed, 5 ignored. The other 8 lines (1 passed, 4 filtered out each) are the child processes of `ledger_concurrency`.
- The 5 ignored are by design: `fake_java_target_sleeps` three times, `print_via_holders_of_a_real_dump` (needs DLA_HPROF), `explore_a_snapshot` (needs DLA_NODE_SNAPSHOT).
- Ran 03:43:24 to 04:10:38 with load average about 10 on 4 CPUs.

STEP 2: JVM-dependent binaries on JDK 25.0.5.1 (first on PATH, JAVA_HOME=/usr/lib/jvm/java-25-openjdk-amd64; java, javac, jar, jcmd all resolved there): PASS, three exit codes 0
- dla-sample (all): 128 passed, 0 failed, 1 ignored (lib 99, attach_tools 14, os 12, os_fd 3, doc 0).
- dla-cli: 112 passed, 0 failed, 2 ignored (alloc_test_jvm 9, attach 18, harness_capture 11, harness_capture_go 4, harness_capture_node 4, harness_gen 7, sample 18, summarize_conformance 41).
- dla-workload: 129 passed, 0 failed (harness_capture_node 7, harness_capture_python 16, harness_driver_build 5, harness_gen 15, harness_junit 31, harness_jvm_coverage 7, harness_run 40, harness_run_attempt 8).
- Run with `-- --show-output` so skip lines are visible. Tests that skipped for the Kotlin compiler on JDK 25 (they count as passed):
  1. dla-cli `harness_capture::a_jni_library_gets_the_shim_and_its_native_stack_resolves_to_the_rust_line`: "skipping: the Kotlin compiler that build.sh would use does not run on this JDK (exception: java.lang.IllegalArgumentException: 25.0.5.1)"
  2. dla-workload `harness_driver_build::a_gradle_kotlin_driver_runs_on_the_resolved_runtime_class_path`: "skipped: the Kotlin compiler of this test's Kotlin Gradle plugin cannot run on JDK 25 (JDK 24 or older needed)"
  3. dla-workload `harness_junit::a_kotlin_spring_boot_test_run_exercises_the_entry_points_of_the_classes_it_loads`: same text
  4. dla-workload `harness_junit::kotlin_gradle_project_series_grows_with_the_repeated_tests`: same text
- One other skip, not JDK related: dla-cli `attach::memray_records_a_python_process_when_memray_is_importable` ("memray is not importable").
- `summarize_conformance::every_adapter_is_conformant` ran 31 adapters and skipped none.
- Extra check on JDK 21: those four tests really run there. `a_jni_library...` passed in 129 s; `harness_driver_build` and `harness_junit` with filter `kotlin` passed 1 and 6; no skip line was printed.

STEP 3: clippy: PASS
- `cargo clippy --workspace --all-targets -j 2 -- -D warnings`: exit 0, no warnings.
- `cargo clippy --target x86_64-pc-windows-gnu -p dla-core -p dla-probe -p dla-report --all-targets -j 2 -- -D warnings`: exit 0, no warnings.
- The first run was partly cached (host re-checked 10 of 12 crates; the Windows run finished in 0.27 s with nothing re-checked). I therefore forced a full re-check by giving `-D warnings` twice, then ran both commands as specified again. All four runs exit 0; the host runs checked all 12 workspace crates including dla-bench and dla-it, the Windows runs checked dla-stats, dla-core, dla-probe, dla-report. These extra runs happened after the 04:27:50 doc edit.

STEP 4: `cargo fmt --all --check`: PASS, exit 0, no output (again on the final tree: exit 0).

STEP 5: `cargo build -j 2 -p dla-cli`: PASS, exit 0 (`dla 0.9.0-preview.2`). `target/debug/dla schema --check`: PASS, exit 0, `{"checked":20,"ok":true,"out":"schemas"}`.

STEP 6: `python3 bench/runner/test_grade.py`: PASS, "Ran 190 tests ... OK". `cargo test -j 2 -p dla-cli --test skill_text`: PASS, 13 passed, 0 failed.

Re-checks on the final tree (04:34, hash bec8f90c): all exit 0
- dla-backends `detector_flags` (reads docs/VERIFY.md): 2 passed.
- dla-core `archetypes_data` (reads the text under harness/): 12 passed.
- `skill_text`: 13 passed. `cargo fmt --all --check`, `dla schema --check`, `test_grade.py` (190 OK).
- The 17 changed VERIFY.md rows each have exactly 4 cells and paired backticks.
- Not re-run on the final tree: the full workspace test and the JDK 25 run. The later edit touched only the two doc files, and no other test reads them (the README is not embedded in the binary; `harness_run` only copies it).

STEP 7: `git status --short` and scan of added lines (final state)
- Status: 29 files, all ` M` (unstaged modifications); nothing untracked, nothing staged. By area: bench/runner/src 3 (dynamic/ops.rs, dynamic/policy.rs, dynamic_cmd.rs); dla-cli src 5 and tests 5; dla-parsers 1; dla-sample 2; dla-workload src 3 and tests 4; docs/VERIFY.md; harness/jvm (DlaDump.java, README.md); harness/python (README.md, sitecustomize.py); skills/detect-leak-anywhere/references/evidence-and-report.md.
- Model names or attribution text (claude, anthropic, sonnet, opus, fable, haiku, co-authored, generated with, noreply): 0 hits.
- Scratch or temp paths (/tmp, scratchpad, u7acc, u8acc, u9, u10, u11, ci-peek, /home/user, /root): 0 hits.
- Plan and package names (N1, N2, K1 to K4, U<n>, P<n>, B0, "pass A", "pass B", refix): 0 hits. The diff removes several from VERIFY.md ("B0", "B0 minor items", "P3/P4 (E2, P8')", "last U4 row").
- REA as a name: 0 hits.
- "round", every hit:
  - crates/dla-workload/tests/harness_jvm_coverage.rs:432 "(and the other way round)": ordinary.
  - docs/VERIFY.md:461 "after each round (warm-up 1, rounds 8, ...)": ordinary, driver rounds.
  - docs/VERIFY.md:488 "`DLA_WARM_ROUNDS` times", "also with 150 rounds", "passed in 4 rounds of two copies": ordinary, an env var and repetition counts.
  - docs/VERIFY.md:509 "`DLA_WARM_ROUNDS=0`": ordinary.
  - docs/VERIFY.md:447 "Independent checks of the phase-1 repair round found these": this is the work-stage sense of "round". It is not new: the same sentence is in HEAD, and the row appears as added only because another part of it was reworded. HEAD's VERIFY.md also has "fix round" three times. Not a leak introduced by this diff; your call whether to reword it.
- Short ids that matched the patterns and are ordinary:
  - `E0`..`E3`, `F1`, `H1`, `S1`, `A1`, `L2` are evidence, finding, harness, series, alloc and fixture-leak ids (mostly in bench/runner/src/dynamic/ops.rs tests).
  - `M2` at crates/dla-cli/tests/harness_capture.rs:391 is the Maven environment variable.
  - "M7 phase 1" / "M7 phase 2" at docs/VERIFY.md:447 is the document's own section name, already in HEAD.
  - `u32`/`u64` are Rust types.
- Outside the listed terms, for your judgment: docs/VERIFY.md has four new uses of "the checker" as an actor (line 509 twice: "the checker saw 2 of 24", "Measured by the checker"; line 518: "taken by the checker"; line 519: "Found by the checker"). HEAD already uses the phrase 3 times; it names no model or plan.

DISK
- Start 5577 MB. During the step 1 compile it fell to 2550 MB at 03:44:58; my watchdog ran prune-superseded.sh once ("pruned 111 units, 2850 MB"), back to 5327 MB. It never went below 1500.
- End: 4860 MB available.

Files are in /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u11acc-t/ (1.4 MB, kept):
- step1-test-workspace.log
- step1-per-binary.txt
- step2-jdk25-sample.log
- step2-jdk25-cli.log
- step2-jdk25-workload.log
- step2b-jdk21-kotlin.log
- step3-clippy-host.log
- step3-clippy-win.log
- step3b-clippy-forced.log
- step4-fmt.log
- step5-build-schema.log
- step6-test_grade.log
- step6-skill_text.log
- step8-final-tree-rechecks.log
- added-lines.txt (start state)
- added-lines-final.txt (final state)
- git-status.txt
- diskwatch.log

No repository file was edited, nothing committed or pushed, no process killed.