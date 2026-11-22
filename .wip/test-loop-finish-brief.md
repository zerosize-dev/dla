# Brief T2: finish and check the test-loop-first queue (another builder wrote it and ran out of room)

You are a builder on the repository /home/user/dla (a Rust workspace, the `dla` tool of a
Claude Code plugin that finds memory leaks). Another builder wrote a change and had to stop
before checking it. Finish it. You do not commit anything.

`<sp>` = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad

## Rules (binding)

- No commits, no `git checkout`/`git stash`/`git reset` on the working tree: the tree holds
  uncommitted work. Baseline for diffs: `git diff 0302e2b38f0af4beac1bdad6b701a764d45c20ed -- . ':!.wip'`
  (the new file `crates/dla-core/src/testloop.rs` is untracked: read it directly).
- Shared `target/`; always `cargo ... -j 2`. Disk is short (about 5.8 GB free; a full test
  cycle needs about 5 GB): run `bash <sp>/wip-notes/prune-superseded.sh` before each full
  `cargo test`, never delete anything under `target/` yourself. Literal paths in `rm`; no
  `rm -rf "$VAR..."`. Never stop a process by name.
- No test, fixture or bench policy is weakened to make something pass. Do not edit
  `bench/` at all. No names of real projects, people, models or assistants in code,
  comments or docs (`docs/VERIFY.md` may name a real library where a row is about a run on
  it, as its other rows do).
- A rule you add or rely on gets a test that you have SEEN FAIL once: put a unique marker
  mutation in the product code (for example `/*MUT_T2a*/ false &&`), run the test, see it
  fail, and reverse the mutation by exact replacement. Before you finish:
  `grep -rI "MUT_" crates harness skills docs` must print nothing.
- Never run `dla consent`. On a real tree, init with the unattended environment:
  `DLA_CONSENT_TIERS=1-4 DLA_CONSENT_RAW_VALUES=1 dla run init ...`.
- Small exact edits; no reformatting of code you do not change; `cargo fmt --all` at the end.
- Scratch `<sp>/u65/`; report `<sp>/u65/report.md`; a short final message. If you run short
  of room, write what you have into the report FIRST, then hand back.

## Read first

1. `<sp>/wip-notes/test-loop-first-brief.md`: the brief the change was written for (what
   is wanted and why).
2. `<sp>/wip-notes/RESUME.md`, the last section ("brief T, interim hand-back"): what the
   first builder says it wrote and what it left undone.
3. The diff against the baseline above, and `crates/dla-core/src/testloop.rs`.

## To do, in this order

1. One test fails: `the_loop_fits_its_runs_into_the_time_limit_by_lowering_the_step_and_dropping_the_slowest_class`
   in `crates/dla-workload/tests/harness_junit.rs` (panic at about line 1659; log of the
   last run: `<sp>/u64/hj.log`). Find whether the fitting code in
   `harness/jvm/DlaJUnitHarness.java` is wrong or the test's numbers are (a 40 s limit, a
   test that sleeps 2.5 s, 9 points). Fix the product if it is wrong; change the test's
   numbers only if you can show they cannot hold on any machine, and say why. Run only
   that test while you work on it (`cargo test -j 2 -p dla-workload --test harness_junit <name>`).
2. `cargo fmt --all`; `cargo clippy -j 2 --workspace --all-targets -- -D warnings`;
   `cargo run -q -j 2 -p dla-cli -- schema --check` (if a schema file changed on purpose,
   say which field and regenerate it the way the repository does); then, after the prune
   script, `cargo test -j 2 --no-fail-fast --workspace > <sp>/u65/test-all.log 2>&1` and
   report the counts (`grep -E "^test result:"`), every `FAILED` line, and whether each
   failure is from this change.
3. Marker mutations, each seen failing and reversed, for the rules that have none yet
   (the first builder did the core queue ones): the choice of test classes by name and by
   mention (`testloop.rs`); the target bound of a hookless harness; `--junit --next`
   taking the queue's batch; the driver fallback text of a failed test-loop harness
   (`run.rs`); a class with no passing test being dropped (`DlaJUnitHarness.java`); the
   declared-target columns leading in `dla trend` (`summary::declared_first`).
4. Real tree. A fresh copy of `/root/dla-real/src/spring-data-commons` at `<sp>/u65/spring`
   (delete `<sp>/u64/spring` and `<sp>/u64/bin` first, they are the first builder's).
   Build `dla` (`cargo build -j 2 -p dla-cli`) and copy the binary to `<sp>/u65/bin/dla`;
   use the copy. Maven needs the warm repository: see how `/root/dla-real/run.py` uses
   `--warm-m2 /root/dla-real/m2warm` and do the same for your shell (do not edit run.py).
   Init (unattended environment), `dla probe`, `dla sweep --lang jvm`, `dla census --lang jvm`.
   Then follow ONLY the commands the tool prints (gen, run, trend, and what trend prints)
   for the first six batches. Run the loop's JVM with the attach disabled
   (`JAVA_TOOL_OPTIONS='-XX:+DisableAttachMechanism'` added to what is already in that
   variable), as the earlier check did. Report per batch: targets, tests chosen, seconds,
   verdict counts. For the batch that holds the finding at
   `src/main/java/org/springframework/data/core/TypedPropertyPaths.java:53`: follow the
   printed route (sibling, `ledger test`, `alloc-test`, refute, confirm) to the end and
   say whether it ends `confirmed`; stop at the first printed command that is refused or
   wrong and quote it. Then: how many test-loop batches the queue needs in all, and the
   time that takes at the measured seconds per batch against the 3 hour budget.
5. Report only, change nothing: (a) which fixtures under `bench/fixtures/` are JVM projects
   with JUnit 5 tests, and what `dla-bench dynamic` (read `bench/runner/src/dynamic/`)
   would now do when the queue's batch command is `--junit --next` for them; (b) the
   second look is capped at 9 candidates a runtime: after a first pass of test-loop
   batches with hundreds of flat readings, what do `dla run status` and the final gate
   say is left, and is that honest; (c) do `dla ledger list` and the schemas show the new
   queue fields correctly.
6. Docs: check that `skills/detect-leak-anywhere/references/jvm.md`, `stages.md` and
   `recovery.md` say what the tool now does (the first builder edited them) and that the
   skill text test passes (`cargo test -j 2 -p dla-cli skill_text`); one `docs/VERIFY.md`
   row for this change exists: correct it to what you have verified.
7. Delete `<sp>/u65/spring` when done (keep the small outputs you quote).

## Report

`<sp>/u65/report.md`: for each of 1 to 6 what you found and did, with the outputs; the
mutations with what was seen failing; the real-tree table; what is unverified; anything
wrong you saw outside this brief.
