# Fix pass U7: the 14 items left open (orchestrator brief)

Repo /home/user/dla, branch main. Do NOT commit or push. HEAD (d1fca56) holds the round as work in
progress; the round's base is c65e617. So `git diff c65e617` (with the working tree) is the whole
round, and `git diff HEAD` shows only this pass. The open items, with file:line and what the last
independent checks saw, are in <sp>/u6/open-items-final.md (three sections). Line numbers may have
moved a little: find the code by name.

sp = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad

Why the round exists: three real runs on Kotlin Spring Boot services stopped with most of the time
budget left, never ran a generated harness, reported "0 of 225 entry points exercised", closed a
finding `time_budget` at minute 10 of 60, and counted non-leak explanations as confirmed.

## Rules
- Disk: if `df --output=avail -B1M /` is below 3000, run `bash <sp>/prune-superseded.sh`; if still
  below 1500, wait up to 30 minutes, then stop and report. Cargo with `-j 2`, the shared `target/`
  only. Never kill processes by name. Delete only your own scratch dir `<sp>/u7-<key>/` by literal
  path (never `rm -rf "$VAR/..."`).
- Every changed behaviour gets a test you SEE fail without the change (revert the hunk, run,
  restore). Say which you saw fail and which you judged by reading. A test that depends on a real
  JVM or timing must pass 8 of 8 runs while another copy of the same test binary runs beside it.
- docs/VERIFY.md: the rows are in the `## M7` table (`| Item | Status | Evidence | Resolution |`),
  exactly 4 cells (escape pipes inside cells as `\|`, never nest backticks in one code span). Update
  the row of the item you change; "corrected" only when its tests ran; name every test; say how each
  was checked in words that match what you did. No internal plan or package names (P4..P9, L1..L3,
  M1..M3, U7, "round", "refix", "integration check", scratch dir names), no model names, no
  attribution text, no byte counts or line numbers that go stale.
- Do not edit bench/fixtures/** or EXPECTED_FAILURES in bench/runner/src/dynamic_cmd.rs.
- The report's `Status:` line must not change. Before changing another report line or a fix text,
  grep crates/*/tests, the goldens, bench/runner/src and bench/runner/grade.py for exact matches and
  update every reader.
- SKILL.md stays within the skill_text limit with its pinned blocks and order.
- Every refusal or final-gate item keeps an escape an unattended run can take.
- Up to two packages edit the tree at once. Re-read a shared file (docs/VERIFY.md, gate.rs,
  gate_hooks.rs, references/*.md) right before each edit and change only your own lines; never
  revert what you did not write. Run `rustfmt` on your own files, not `cargo fmt --all`. If a test
  binary vanishes or is busy ("never executed", "No such file", "Text file busy"), another agent
  rebuilt target/: rerun it.
- Before reporting: `cargo test -j 2 --no-fail-fast` for every crate you touched (all test
  binaries), `cargo clippy --workspace --all-targets -j 2 -- -D warnings`, `cargo fmt --all --check`
  (if it fails only on files you do not own, say so), `target/debug/dla schema --check` (regenerate
  with `target/debug/dla schema --out schemas` after rebuilding if a serialized type or its doc
  changed), `python3 bench/runner/test_grade.py`, clippy for x86_64-pc-windows-gnu on dla-core,
  dla-probe and dla-report if touched, and the `dla-bench dynamic --parallel 1 --only <f>` fixtures
  your package names (compare with <sp>/p2r3-acc/on/result.json; a worse outcome is a problem).
- Report: each item's state (fixed / already fixed / wrong, with evidence), files changed, tests and
  which you saw fail, checks run, anything not done and why.

## M1: budget and time (section "Budget, time and reasons", 6 items)
Decisions:
1. A worker's refusal never moves the core to the floor. In the core (worker None), count only
   refusals recorded by the core itself (`worker` null): match exactly, in `nothing_fits_since` and
   wherever the core reads `time_refusals.jsonl`. A worker counts only its own. Test in gate_hooks
   with a worker's `none_fits` line: the core's Stop hook still says to measure what is open.
2. Test the writer of `none_fits` (true at a spent deadline and when no smaller run is suggested,
   false with a suggestion).
3. After a `none_fits` refusal of the core's own command with most of the budget left, the hook's
   reason says what happened: a command was refused with nothing smaller that fits. It must not
   say that too few minutes remain while printing a large number of minutes.
4. A refusal recorded before the budget was last raised does not count: an extension already
   clears it; an auto budget raised by a later sizing gate should too, if the state records when
   it was raised (if it does not, say so and leave it).
5. The Stop hook's reason is clipped at a fixed length: make sure the budget line and the pointer
   to the full list survive the clip when item fixes are long (clip the item texts first). Test it.
6. Fix the docs named in the items: the `estimate` field doc in state.rs and the regenerated
   schema, the TIME_REFUSALS_FILE doc in toolctx.rs, and the VERIFY row on the Stop hook (no nested
   backticks, evidence wording that matches what was done, no dangling clause).
Bench fixtures at the end: jvm-order-cache, go-subscription-workers.

## M2: coverage warnings and harness steps (section "Coverage warnings and harness steps", 4 items)
Decisions:
1. The harness the step names is the one the model must act on next. Prefer the newest hook driver
   that built (it ran, or its last attempt is not a build failure) over an older one that never
   built. The build-failure step ("fix the build error" or record `build_fails`) is given only when
   no hook driver of that runtime has built; it never hides a driver that built and ran, and the
   `build_fails` escape is not offered when one did. Cases to test: an old driver with a failed
   build and a newer driver that ran with 0 of N and with a minority exercised (the step names the
   newer one: fill its DLA-HOOK bodies, then run it); only a failed driver (the build step); a
   failed driver and a JUnit repeat harness that ran. The gate fix, the Stop hook and the report's
   Coverage warning give the same step.
2. When the build failed and no iteration ran, the warning says it once (cause and step are not
   the same sentence twice).
3. Docs state the rule as built: stages.md (the build-failure sentence), evidence-and-report.md
   (the list of warning steps includes the command-harness case: make a hook driver with
   `--driver`), and the VERIFY row on the build-failure step.
Bench fixtures at the end: jni-order-codec, go-private-module, jvm-billing-private-repo,
native-vendor-codec, rust-private-registry, python-vendor-wheel.

## M3: JVM non-heap verdicts (section "JVM non-heap verdicts and CodeQL", 4 items)
The code is `agent_drift` and its helpers in crates/dla-cli/src/alloc_cmd.rs. Three passes went back
and forth here, so build exactly this rule:
1. A required column "rises" by its kind:
   - a thread count (`jvm.threads`, `os.threads`): its plateau reads continues, OR its net rise
     over the extra window is over a small jitter. Measure the jitter: run an idle JVM driver's
     alloc-test at least 6 times, half of them while a CPU-heavy job runs beside it (for example a
     second copy of the same test), and read the net extra rise of both thread columns. Set the
     constant to the largest rise seen plus 1, and at least 2 (HotSpot starts and stops compiler
     threads by itself). The slow leaks the check saw rose 6 and 10 threads.
   - a `jvm.nmt.*` byte category: only when its plateau reads continues (bytes jitter);
     `jvm.nmt.thread` also counts as rising when a thread count rises.
   - the class count (`jvm.classes_loaded`): not flat when its plateau reads continues, or reads
     inconclusive with a net rise over the class jitter. Measure that jitter the same way (idle
     driver, under load too; the check saw a rise over 3 under load on a driver that defines no
     class) and set the constant to the largest idle rise seen plus a margin.
2. Order: first collect every rising required column and whether the class count is flat. If any
   thread or NMT column rises: `Drift::Growing`, naming ALL growing columns (the class count too
   when it is not flat), and `next` holds the thread capture whenever a thread column rises (never
   `--extra` for a thread leak). Only when no required column rises and the class count is not flat:
   `Drift::Classes`. Only when nothing rises and the class count is flat: `Drift::Agent`.
3. The class note states true numbers: when the plateau reads continues say it keeps rising and
   by how much (no "more than the jitter" at a rise equal to it); with no `jvm.classes_loaded`
   column in the results there is no class note with a made-up 0: treat the class count as unknown,
   run the rising check, and if nothing rises give the agent-drift reading with a caution that the
   class count was not recorded; on a partial run do not say "the verdict stands" (the partial cap
   makes it inconclusive).
4. Tests: unit cases for each branch (thread count inconclusive with a net rise of 10 is not
   `Drift::Agent`; classes and threads both rising names both and keeps the capture; a thread net
   rise within the jitter on an idle driver is `Drift::Agent`; no class column), and the end-to-end
   cases: a driver that starts a daemon thread every 10th unit, with no trend, keeps the leak
   reading and names the thread count; the existing
   `a_driver_that_keeps_daemon_threads_is_not_blamed_on_the_agents_drift` passes 8 of 8 under load;
   the idle driver at the default extra and at extra 100 never reads leak, 8 of 8 under load.
5. Docs say exactly this and no more: jvm.md (the drift paragraph), harness/jvm/README.md, the
   caution text in the code, and the VERIFY row on the non-heap verdict (no self-contradiction, the
   measured jitter figures, only tests that pass repeatedly listed as passing).
Bench fixtures at the end: jvm-order-cache, jvm-rule-compiler, jvm-pool-warmup, jni-order-codec.
