# Fix pass U9: two flaky non-leak verdicts and the items the last checks left open

Repo /home/user/dla, branch main at 9781542 (version 0.9.0-preview.2; CI was green on this tree
on Ubuntu 26.04, macOS and Windows). Do NOT commit or push; `git diff HEAD` shows only this pass.

sp = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad

## Rules
- Disk: if `df --output=avail -B1M /` is below 3000, run `bash <sp>/prune-superseded.sh`; if still
  below 1500, wait up to 30 minutes, then stop and report. Cargo with `-j 2`, the shared `target/`
  only (the version changed, so the first build is a full one). Never kill processes by name.
  Delete only your own scratch dir by literal path (never `rm -rf "$VAR/..."`).
- Never skip, ignore, disable or weaken a test just to get green. A test changes only when it
  asserted something that is not true while the product works; say what was untrue and show that
  the changed test still fails when the product is broken. A product defect is fixed in the product.
- Every changed behaviour gets a test you SEE fail without the change (revert, run, restore). Say
  which you saw fail and which you judged by reading.
- docs/VERIFY.md: rows are in the `## M7` table (`| Item | Status | Evidence | Resolution |`,
  exactly 4 cells, pipes inside cells as `\|`, no nested backticks). Update the row of an item you
  change, add a row for a new item at the end. "corrected" only when its tests ran here. Name
  every test. No internal plan or package names (N1, N2, K1..K4, U8, U9, P3, B0, "round", "refix",
  scratch dir names), no model names, no attribution text, no byte counts or line numbers.
- The report's `Status:` line must not change. Before changing another output line, a verdict
  word or a fix text, grep crates/*/tests, the goldens, bench/runner/src, bench/runner/grade.py,
  skills/, agents/ and harness/*/README.md for exact matches and update every reader.
- Do not edit bench/fixtures/**, bench/fixtures-open/** or EXPECTED_FAILURES in
  bench/runner/src/dynamic_cmd.rs. Do not change the bench policy so a verdict is not needed.
- Two packages edit the tree at once. Re-read a shared file (docs/VERIFY.md) right before each
  edit and change only your own lines; never revert what you did not write. Run `rustfmt` on your
  own files, not `cargo fmt --all`. If a test binary vanishes or is busy, another agent rebuilt
  target/: rerun it.
- Before reporting: `cargo test -j 2 --no-fail-fast` for every crate you touched (all test
  binaries), `cargo clippy --workspace --all-targets -j 2 -- -D warnings`,
  `cargo clippy --target x86_64-pc-windows-gnu -p dla-core -p dla-probe -p dla-report --all-targets -j 2 -- -D warnings`,
  `cargo fmt --all --check`, `target/debug/dla schema --check`, `python3 bench/runner/test_grade.py`.
- Report: each item's state, the cause with its evidence, files changed, tests and which you saw
  fail, checks run, what was not done and why, what only CI can confirm.

## N1: alloc-test reads a flat column as inconclusive when it happens to vote (product)

Two non-leak fixtures failed once each with `no rule-out on a non-leak fixture`, and pass in
most runs. Both times the first growth test passed, by chance, a column that normally does not
pass; the unbound `dla alloc-test` then votes with "every column that passed the gate in the
latest trend of the harness, and os.rss" (crates/dla-cli/src/alloc/pick.rs), that column read
`plateau` with a `retained` purge, `judge::combine` (crates/dla-cli/src/alloc/judge.rs) calls
that a contradiction and returns inconclusive, and one inconclusive vote makes the whole verdict
inconclusive (`judge::overall`). The evidence:

1. go-gc-tuned-batch, on the CI runner (job log <sp>/u8/ci-peek/dynamic_acceptance_on_the_fixtures.log,
   search the fixture name; the run's artifact cannot be downloaded from here). The columns
   `go.heap_live` and `go.heap_objects` passed the first trend. They are the same bytes after a
   forced collection, but only `go.heap_objects` is in the `free-os-memory` purge record, so only
   it gets a purge verdict: `go.heap_live non-leak (plateau, purge not-run)`,
   `go.heap_objects inconclusive (plateau, same slope and rise, purge retained)`. A local run of
   the same fixture (<sp>/u8acc/dynamic.logs/go-gc-tuned-batch/series/S3.json and S5.json) shows
   the sizes: the column is 164520 at the end of the warm-up and 181832 or 184712 before the
   purge, a growth of 17 to 20 KB, and a purge cannot give live objects back. `judge::purge`
   calls any growth above `1e-9 * before` growth. The plateau test of the same column has a
   minimum for exactly this, `MIN_BYTE_RISE` (32 KiB: "the live heap of a correct Go program
   moves by a few KB"; a smaller rise "is a plateau whatever the noise reads"). The two tests of
   one column disagree on what growth is.
   Decision: the purge test uses the same minimum as the plateau test. A growth at or below the
   column's minimum rise (the `min_rise` that `test_column` already computes: `MIN_BYTE_RISE`
   for a byte column, zero otherwise) is `no-growth`. Nothing else in `judge::purge` changes:
   growth above the minimum reads released, partial or retained as today.
2. python-graph-churn, in a full local run under load (<sp>/u8acc/dynamic.logs/python-graph-churn/:
   008-trend-S1.log, alloc/A1.json and A2.json, series/S3.json; the solo rerun that passed is in
   <sp>/u8acc/rerun-pgc.logs/). `os.rss_file` passed the first trend as a stair on a flat, noisy
   series (10.55 to 10.74 MB over 17 points). Its purge record reads before 10.85 MB, after
   11.22 MB: the reading before the purge is above every point of the series, and the column
   rises again across the purge. The purge step maps files itself (the `trim` purge of the Python
   harness loads the C library through ctypes), and no purge kind gives file-backed pages back.
   So the purge verdict of `os.rss_file` is `retained` whenever it is taken, for a reason that is
   not growth of the program. pick.rs already has the rule for such columns: `NO_RELEASE`
   (`os.vm_hwm`, `native.peak_bytes`, `os.vm_data`: "the purge test cannot release them and their
   verdict is at best inconclusive; one such vote would block every non-leak verdict. They are
   reported and vote only when nothing else can, even after passing the gate").
   Decision: `os.rss_file` joins that list, with its reason in the comment. A growth of
   file-backed memory still shows in `os.rss`, which always votes; check that with a test (a
   series where `os.rss_file` and `os.rss` both keep growing reads leak, and names both).
   First confirm the mechanism: read the Python, Node, JVM, Go and native harness purge code
   (harness/*/) and say for each purge kind whether its own step can move `os.rss_file`.
3. Look for more of the same before you stop: in <sp>/u8acc/dynamic.json, <sp>/u7acc/dynamic.json
   and <sp>/p2r3-acc/on/result.json every `alloc-test` diagnostic lists the columns with their
   verdicts (`<column> inconclusive (plateau ..., purge retained)`). About 28 column readings per
   run have this shape; most are not voting (`os.vm_hwm`) or belong to fixtures whose verdict is
   meant to be inconclusive. For each distinct column name with this shape, say whether it can
   vote and whether its `retained` is real (growth above the minimum that the purge kind could
   have released) or an artifact like the two above. Change nothing for a column unless you can
   name the mechanism; list what you looked at.
Do not change `judge::combine`: a plateau with a retained purge over real growth stays
inconclusive, and a retained purge still turns an undecided plateau into leak.
Tests (each seen failing without its change): the purge verdict at, below and above the minimum
for a byte column, and unchanged for a column with no minimum; `test_column` on a series with
the Go shape above (plateau, growth of 17 KB since the warm-up, purge before equal to the last
point, after slightly higher) reads non-leak with purge `no-growth`; the pick tests for
`os.rss_file` (passed the gate with another votable column: tested, not voting; nothing else can
vote: it votes); a command-level test of `dla alloc-test` over prepared series and trend files
where a flat `os.rss_file` passed the gate and the overall verdict is non-leak (see how
crates/dla-cli/tests/alloc_test*.rs build their runs); the leak case of item 2.
Then run these fixtures before and after with `dla-bench dynamic --parallel 1 --only <fixture>`
(build `dla` and `dla-bench` first; copy the binaries to your scratch dir so a rebuild by the
other package does not swap them mid-run): go-gc-tuned-batch, go-buffer-pool, go-ticker-modern,
python-graph-churn, python-pool-fragmentation, python-report-memo-bounded, node-price-board-flat,
node-template-cache, rust-tree-parent-links-weak, rust-batch-buffers, native-arena-reuse,
jvm-order-cache-bounded. Every one must pass; compare each one's refuted, confirmed and rule-out
counts with <sp>/u8acc/dynamic.json and explain any difference. The two failures cannot be made
to happen by rerunning (the first trend has to pass the extra column by chance), so the proof is
the prepared-series tests, not a loop.
Docs: the references and harness READMEs that state the purge rule or the voting columns
(grep `retained`, `no-growth`, `vote`, `rss_file` under skills/, agents/, harness/, docs/) say
what the code now does. VERIFY: one new row per decision.

## N2: items the last checks left open (tests, comments, docs)

The list is <sp>/u9/open-items.json (14 items: where, text, and the direction the checker
proposed). Handle every one; where an item is wrong, say why with evidence. Notes on some:
1. The JVM warm-up test (crates/dla-cli/tests/alloc_test_jvm.rs): its start-value and first-step
   assertions fail in about 1 of 12 pairs when four harness runs share the machine, with the
   warm-up working. Replace them with a relation that holds when the warm-up works and fails
   when it is off, and prove both by measurement on JDK 21 and JDK 25
   (/usr/lib/jvm/java-25-openjdk-amd64): with the warm-up on, the test passes at least 19 of 20
   runs while three other copies of the measurement run beside it; with the warm-up disabled
   (`DLA_WARM_ROUNDS=0` on both sides, or the warm-up code switched off) it fails at least 9 of 10
   runs. The checker's numbers: the rise from the first reading to point 15 was at most 0.58 of
   the cold run's in 41 Linux pairs and 0.66 and 0.74 in the two macOS logs; two cold runs
   against each other gave 0.72 to 1.38. A single threshold between those ranges is thin, so
   consider measuring again before failing (a second pair) and keep the early-rise comparison
   running on JDK 25 (the 500 KB line switches it off there). The macOS figures are in
   <sp>/u7/ci/j-clippyandtestmacoslatest.log and <sp>/u7/ci/mac.log. Then make the VERIFY row
   state the measured ranges as ranges and drop "true on every build".
2. The JNI test `a_jni_library_gets_the_shim_and_its_native_stack_resolves_to_the_rust_line`
   (crates/dla-cli/tests/harness_capture.rs) fails on JDK 25 when the only Kotlin compiler is the
   cached 2.0.21 one, which cannot start on JDK 25 (`IllegalArgumentException: 25.0.5.1`). Its
   guard checks only that a compiler exists. Make the guard check that the compiler it will use
   runs on the JDK in use (the helper the three Kotlin Gradle tests got in
   crates/dla-workload/tests/ shows the way; do not duplicate it badly: a one-file compile is
   enough) and print the skip with the reason. It must still run and pass on JDK 21 here.
3. The flight recorder's fourth thread, `JFR Recording Scheduler`, joins the list in
   crates/dla-parsers/src/jvm/thread_dump.rs, its unit test and the limitation line.
4. The coverage agent (harness/jvm/DlaDump.java, `isTestLocation` and the location check): a
   project spelled through a link is still read as test output when a directory above it is
   named like one. Fix it as the checker describes and add the test (a project below a directory
   named `test`, given through a link): it must fail without the change.
5. docs/VERIFY.md rows named in the items: correct the count and the list (thirteen tests; run it
   again yourself before writing a number, or give the list without a count), the row title that
   does not describe its row, the references to "the last U4 row" (name the row by its title),
   and the plan names still in the table (`B0 minor items` in a title and its pointer,
   `P3/P4 (E2, P8')` in a status cell): reword them by what they are about.
6. Last step, after everything else: run
   `bash <sp>/u8/ci-wait-branch.sh 9781542c3196c86f935dfc3b077988d5914bf648 <sp>/u9/ci-main-peek main 30`.
   If it prints `DONE ... conclusion=success`, change the status of each row that says
   "to be confirmed in CI" and whose item a runner job covers to say that the CI run of commit
   9781542 confirmed it, naming the job (for example `clippy and test (macos-latest)`), and keep
   as open what one green run cannot show (how often a rare failure happens). If it prints
   PENDING or another conclusion, leave those rows as they are and say so in the report.

## For the agents of the CI phase: a third fixture that fails by chance (not in N1 or N2)

`jni-order-codec` failed the dynamic job of the CI run on main (commit 9781542; job log
<sp>/u8/ci-main/dynamic_acceptance_on_the_fixtures.log) and passed in the run before it on the
same code: `leak L1 (L-HANDLE) label refused label-evidence-missing`, because both summaries of
the native shim failed their gate (`type-histogram-diff via shim-stack-groups`,
`alloc-stack-diff via shim-alloc-stacks`: "nothing grows strictly across all 17 captures;
stack:Java_shop_codec_Native_open (native/src/lib.rs:40): holder absent from dump 1"). The same
happened once on an earlier CI run and never here. So the stack of the leak is missing from the
first native capture on a slow or loaded runner; the gate in
crates/dla-cli/src/summarize/rule.rs needs the holder in every dump. If a CI attempt of this
pass fails on this fixture, this is the starting point: find why the first capture can miss the
stack (when the first capture is taken against the first allocation of that site, what the shim
records before the library is loaded, how the capture and the workload are ordered in
harness/jvm and the shim), try to reproduce it under CPU load with
`dla-bench dynamic --parallel 1 --only jni-order-codec` while other work runs, and fix the cause
in the product. Do not loosen the gate to hide it, and do not touch the fixture. Whether or not
you can reproduce it, make `dla-bench dynamic` print what a reader needs when a fixture fails
(the reasons and the per-capture values of the gate that failed), because the per-fixture logs
of a CI run cannot be downloaded here.
