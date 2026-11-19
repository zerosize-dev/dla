# CI repair U8: green on the three runners (orchestrator brief)

Repo /home/user/dla, branch main at 63a30f2 (the work is checked on this Linux machine: tests,
clippy with the previous toolchain, schema, graders, dynamic fixtures). Do NOT commit or push;
`git diff HEAD` shows only this repair. CI (.github/workflows/ci.yml, runs on every push) is red
for reasons that this machine did not show. The logs of the failed jobs of the last run are in
<sp>/u7/ci/ (`j-clippyandtestubuntulatest.log`, `j-clippyandtestmacoslatest.log`,
`j-clippyandtestwindowslatest.log`, `j-dynamicacceptanceonthefixtures.log`,
`j-staticacceptanceonthefixtures.log`; `mac.log` is an older macOS run). Read the part of the log
your package names before changing anything.

sp = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad

What changed around us since CI was last green:
- The `ubuntu-latest` runner is now Ubuntu 26.04. Its `sleep` (and every coreutils tool) is one
  multi-call binary that picks the tool by the name it is started under. Its default JDK is newer
  than 21 (JDK 25 is installed here at /usr/lib/jvm/java-25-openjdk-amd64; the default `java` on
  this machine is still 21). Only the dynamic-acceptance job pins Java 21 (setup-java).
- `stable` Rust is 1.100 (installed here now; `target/debug` was cleared, so the first build is a
  full one).

## Rules
- Disk: if `df --output=avail -B1M /` is below 3000, run `bash <sp>/prune-superseded.sh`; if still
  below 1500, wait up to 30 minutes, then stop and report. Cargo with `-j 2`, the shared `target/`
  only. Never kill processes by name. Delete only your own scratch dir `<sp>/u8-<key>/` by literal
  path (never `rm -rf "$VAR/..."`).
- Never skip, ignore, disable or weaken a test just to get green. A test changes only when it
  asserted something that is not true on a supported platform; say what was untrue. A product
  defect the test exposed is fixed in the product.
- Every changed behaviour gets a test you SEE fail without the change (revert, run, restore). Say
  which you saw fail and which you judged by reading. Say plainly what you could not reproduce
  here (macOS, Windows): those are verified by CI afterwards.
- docs/VERIFY.md: add one row per item at the end of the `## M7` table
  (`| Item | Status | Evidence | Resolution |`, exactly 4 cells, pipes inside cells as `\|`, no
  nested backticks). "corrected" only when its tests ran here; "corrected, to be confirmed in CI"
  for what only the macOS or Windows runner can show. Name every test. No internal plan or
  package names (K1..K4, U8, "round", "refix", scratch dir names), no model names, no
  attribution text, no byte counts or line numbers.
- The report's `Status:` line must not change. Before changing another output line or fix text,
  grep crates/*/tests, the goldens, bench/runner/src and bench/runner/grade.py for exact matches.
- Up to two packages edit the tree at once. Re-read a shared file (docs/VERIFY.md, a shared test
  file) right before each edit and change only your own lines; never revert what you did not
  write. Run `rustfmt` on your own files, not `cargo fmt --all`. If a test binary vanishes or is
  busy, another agent rebuilt target/: rerun it.
- Before reporting: `cargo test -j 2 --no-fail-fast` for every crate you touched (all test
  binaries), `cargo clippy --workspace --all-targets -j 2 -- -D warnings`,
  `cargo clippy --target x86_64-pc-windows-gnu -p dla-core -p dla-probe -p dla-report --all-targets -j 2 -- -D warnings`,
  `cargo fmt --all --check`, `target/debug/dla schema --check`, `python3 bench/runner/test_grade.py`.
- Report: each item's state, cause found (with the log line or the local reproduction), files
  changed, tests and which you saw fail, checks run, what only CI can confirm.

## K1: Rust 1.100 lints, Windows, small doc leftovers
1. Windows job: clippy 1.100 fails in crates/dla-probe/src/registry.rs (`is_executable`,
   `needless_bool` once the `#[cfg(unix)]` block is compiled out). The job stops at the first
   failing crate, so look for more: run clippy with `--target x86_64-pc-windows-gnu` and
   `--target x86_64-pc-windows-msvc` and `--target aarch64-apple-darwin` on every workspace crate
   that can be checked without a foreign C compiler (try each crate separately; report which could
   not be checked and why), and read every `#[cfg(windows)]`, `#[cfg(not(unix))]` and
   `#[cfg(target_os = "macos")]` block of the crates that could not be checked for the same lint
   shapes (an `if` that returns a bool literal followed by a cfg block, unused variables or imports
   on one platform, needless returns).
2. Doc leftovers named by the last check: doc comment lines over 100 columns added lately
   (crates/dla-core/src/gate.rs near the harness-not-generated item, crates/dla-core/src/harness.rs
   near "Why it failed", crates/dla-core/src/toolctx.rs near the time refusals); docs/VERIFY.md
   row "A generated harness must be run..." (the sentence that says the step is the build for a
   run refused for time is wrong: for that case the step is `dla harness run <H>`; and the count
   "eleven of the nineteen tests" is stale: drop the total); the M7 rows whose titles carry tags
   like "(U2, rule)" (remove the tags, keep the titles); the row in the "M6 benchmark follow-up"
   table with an unescaped pipe inside a code span; harness/jvm/README.md has a line holding only
   the word "The"; skills/detect-leak-anywhere/references/jvm.md has two lines far over the wrap
   width; jvm.md and harness/jvm/README.md say a class leak of 8 or fewer classes "reads
   inconclusive, never a leak", true only when the count does not read continues: say that.

## K2: the Ubuntu 26.04 runner
Log: j-clippyandtestubuntulatest.log.
1. Fake targets. Tests copy or link `sleep` under the name `java` (crates/dla-sample/tests/
   attach_tools.rs, crates/dla-cli/tests/attach.rs, and whatever else does it: grep for `sleep`
   in crates/*/tests and bench/runner). With a multi-call coreutils that binary exits at once
   ("process N is not running", "exited or was replaced before the attach started"). Make the
   fake target independent of how `sleep` is built, on Linux and macOS (and Windows where those
   tests run): for example the test binary's own copy started under the name `java` with a
   helper test that sleeps, or a small program compiled once. Reproduce the failure first by
   putting a multi-call stand-in first on PATH (a script or busybox link named `sleep` that
   refuses an unknown name), then show the tests pass with it. Check every failing test of the
   log: `attach` (5), `summarize_conformance` (1), `dla-sample attach_tools` (6).
2. JDK 25. `jcmd <pid> GC.heap_info` on JDK 25 prints
   `garbage-first heap   total reserved 262144K, committed 221184K, used 3204K [...]` where JDK 21
   prints `total 262144K, used 3204K`; `parse_heap_info` (crates/dla-sample/src/jcmd.rs) rejects
   it, so the jcmd backend is blocked on JDK 25. This is a product defect. Make every collector's
   layout of JDK 25 parse (G1, Parallel, Serial, ZGC, Shenandoah if this build has them), keep JDK
   21 and older working, and keep the meaning of the columns (say in the docs which figure
   "committed" now comes from if it changes). Then run every JVM-dependent test of the workspace
   with JDK 25 first on PATH and JAVA_HOME set to it (dla-sample, dla-cli: sample, attach,
   alloc_test_jvm, harness tests; dla-workload: harness_*, harness_jvm_coverage, harness_junit,
   harness_driver_build; dla-census if it runs a JVM) and fix what fails in the product (the
   agent harness/jvm/DlaDump.java, NMT category names, thread dumps, class histograms, JFR), each
   with a test. Add JDK 25 heap_info samples to the parser's unit tests. Run the same tests on
   JDK 21 again at the end. docs: where the supported JDK range is stated, state it truthfully.
3. `harness_driver_build::a_gradle_kotlin_driver_runs_on_the_resolved_runtime_class_path` fails
   on the runner: the Kotlin Gradle plugin 1.9.24 "was not found" in the plugin repositories.
   Find the cause from the log (offline resolution, a Gradle of the runner that no longer takes
   that plugin version, a proxy) and from what the test and the product do (does the product run
   Gradle offline unless consent allows downloads? does the test need the network?). If the test
   needs the network or a tool version the machine may not have, it must detect that and say why
   it did not run its JVM part, the way the other tool-dependent tests do; if the product is at
   fault, fix the product.

## K3: the macOS runner
Log: j-clippyandtestmacoslatest.log (and mac.log). Nothing here can be run on macOS: reason from
the log and the code, reproduce on Linux what can be reproduced, and add the diagnostics a failed
assertion needs (the tail of the harness log, the values compared) so the next CI run explains
itself.
1. `harness_jvm_coverage` (3 tests): the project class count is 0 where 2 are expected. The
   likely cause is a path compared as text: on macOS the temp dir is under `/var/folders/...`,
   which is a symlink to `/private/var/...`, so the JVM's code-source location and the project
   root differ as strings. Find where the agent (harness/jvm/DlaDump.java) and
   crates/dla-workload/src/harness/cover.rs decide "project class" and "test location", and
   compare real paths. Reproduce on Linux with a project reached through a symlinked directory
   (the test's temp dir under a symlink), see the count drop to 0 before the fix, and keep that
   as a test. This is a product defect for any project under a symlinked path.
2. `alloc_test_jvm::a_driver_that_keeps_daemon_threads_is_not_blamed_on_the_agents_drift`: on
   macOS the harness itself exits 1 ("every measured process of mode fresh exited with a non-zero
   code"). Read what the driver does and what the agent reads at a checkpoint (thread counts, NMT,
   `os.threads` off Linux) and find what can fail on macOS; make the test print the harness log
   tail when the run fails.
3. `alloc_test_jvm::the_agents_warm_up_takes_most_of_its_own_early_non_heap_growth_off_an_idle_driver`:
   on macOS the warm run's early rise (364 KB) is not below the cold run's (335 KB). Decide from
   the code whether the warm-up runs on macOS at all (then it is a defect) or whether the test
   asserts a relation that only holds for some JVM builds; assert what is true everywhere and
   say what is not claimed.
4. `harness_run::a_point_that_outlasts_the_deadline_is_cut_and_the_partial_series_says_so`
   (mac.log): on a slow runner the estimate (7 s) is over the 3 s left before the test's deadline,
   so the run is refused instead of cut. Give the test a deadline that a slow machine meets while
   it still cuts a point (derive it from a measured dry step, not a constant), and show it still
   fails without the cut logic.

## K4: the benchmark jobs
Logs: j-dynamicacceptanceonthefixtures.log, j-staticacceptanceonthefixtures.log.
1. Eight new fixtures do not pass yet (they failed before this work too): go-feed-merge,
   go-header-index, go-ticker-legacy, go-trace-context, node-signal-compose-24, python-frozen-gc,
   python-zstd-options-3144, rust-slot-arena. Move those eight directories, unchanged, from
   bench/fixtures/ to bench/fixtures-open/ (use `git mv` semantics by moving the directories; do
   not edit their files), with a short bench/fixtures-open/README.md: these fixtures are written
   and reviewed but DLA does not pass them yet, the failure of each (from <sp>/u7acc/dynamic.json
   and static-on.json), and that they move back when they pass. Make everything that names or
   counts fixtures agree: bench/FIXTURES.md, bench/README.md, tests of dla-bench that list or
   count fixtures, policy or expectation tables keyed by fixture id (leave entries for the eight
   in place if nothing breaks, so moving back is a one-step change; remove only what a test
   rejects), CI steps, and the docs/VERIFY.md row on the new fixtures (state: eight open, where
   they are, why). Nothing may scan bench/fixtures-open/ as fixtures. `dla-bench static` (both
   archetype modes) and `dla-bench dynamic --parallel 2` must then exit 0 here on all remaining
   fixtures (compare with <sp>/p2r3-acc/on/result.json: no fixture that passed there may fail).
2. The dynamic job failed before running anything: `uv python install 3.14.4` found no download
   on the runner. After the move, check every remaining fixture's `[toolchain]` pin (Python, Go,
   Node, Java versions in bench/fixtures/*/expected.toml) against what the job can install: read
   how the runner resolves toolchains (bench/runner/src, the job's steps, the pinned
   setup-uv/setup-go/setup-node versions) and test each pin's availability from here where
   possible (`uv python list`, the Go and Node download indexes). If a pin cannot be installed on
   the runner, fix the cause (a newer installer pin, a different exact release that still shows
   the fixture's behaviour only if the fixture's README allows it), never by dropping the pin.
3. The static job also failed on go-trace-context, which passes static here: after the move it
   is gone from the job, but say why the two machines differed (which analyzers each had) in the
   README of bench/fixtures-open/.
