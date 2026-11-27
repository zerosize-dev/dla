# Builder S: a step in the extra window is growth, and a too-short window does not decide

Repository: /home/user/dla, branch main, clean tree. Work in the working tree. Do NOT commit, push
or switch branches. Do not touch harness/, bench/fixtures/, any fixture README or answer key, or
skills/detect-leak-anywhere/SKILL.md. Read each file just before you edit it. Nothing else runs on
this machine. The repository must not name the project of an evidence run or its APIs: describe it
as "a Rust library" and its holder as "a `Vec` field", nothing more.

## What a real run showed

An unattended run on a Rust library built a driver made for the library's documented leak (a `Vec`
field that grows once a bounded ring is full) and measured the growth: the allocator shim's
`native.live_bytes` grew in steps (trend shape `stair`, slope 4468 bytes/unit, gate passed). Then
`dla alloc-test` on that driver read the column as a plateau, favored non-leak, and the finding was
refuted. Its own data had the next step in it. The evidence is at
/root/dla-real/r6-fastrace-keep/ (series/S16.json is the harness run, series/S17.json the
alloc-test series, alloc/A1.json the alloc-test result; read them, do not copy them into the repo).

`native.live_bytes` of the alloc-test series, x (units) and bytes:

    main:  440 2663109 | 880 3646149 | 1320 5612229 | 1760 5612229 | 2200 9544389 | 2640 9544389
           3080 9544389 | 3520 17408709 | 3960 17408709
    extra: 4400 17408709 | 4840 17408709 | 5280 17408709 | 5720 17408709 | 6160 17408709
           6600 17408709 | 7040 33137349 | 7480 33137525

(check the split of main and extra against the code and A1.json; the alloc-test result says
`extra_rise 0.0, extra_slope 0.0, main_slope 4468, noise 1457000, verdict plateau` for this
column, while `os.rss` read `growth-continues`.)

A `Vec` (and any array- or hash-backed container: a Go slice or map, a Java ArrayList or HashMap,
a Python list or dict) grows its allocation by doubling its capacity. Its allocated bytes rise in
steps, and each flat stretch is about as long as all the growth before it. Two defects in
crates/dla-cli/src/alloc/judge.rs follow from that.

1. **A late step in the extra window is invisible to the plateau test.** `plateau()` (gauge path)
   fits the extra window with Theil-Sen: six flat points and two high ones give a median slope of
   0, so `rise` is 0. The stair reading (`stair()`, from an earlier fix) would see the new maximum
   (33.1 MB over the main window's 17.4 MB plus the noise), but `plateau()` calls it only when
   `rise > noise` (the "extra window rose clearly" guard), so it never runs here.
   Fix: for a gauge whose ordinary verdict is plateau, read the steps also when the extra window's
   robust rise is at or under the noise: a new maximum that holds (the extra window ENDS above the
   main window's maximum by more than one noise unit, by more than the column's minimum rise, and
   by a material amount: use the existing `min_rise` / `MATERIAL` notions) is growth that
   continues, whatever the Theil-Sen slope says. A spike that returns (the extra window's last
   point back within one noise unit of the main maximum) is not. A column that did not grow in
   separate steps in the main window (`stair()` returns None: a ramp or a level) keeps the old
   verdict. Check whether `stair()`'s current `NewMax` (which uses the extra window's maximum, not
   its end) is pinned by a test; if you change it to "ends above", say which tests changed and why.

2. **"Stopped" comes too early for steps that grow apart.** `stair()` reads `Stopped` when the
   extra window is longer than the longest gap between the main window's steps. For a doubling
   capacity each gap is about twice the one before (here the steps came at 880, 1320, 2200, 3520,
   then 7040), so a window that ends before the next step reads as a plateau. Fix: measure the flat
   stretch from the last step (in the main window, or in the extra window if one came there) to the
   end of the extra window, and compare it with the expected next gap: when the gaps grow (each at
   least the one before, allowing one point spacing of slack), the expected next gap is
   2 x (last gap + one point spacing); otherwise the longest gap. Under it: `TooShort` with
   `points_needed` that reaches it; over it: `Stopped`. The alloc-test already extends its window
   when a column is inconclusive with `points_needed` and the time fits (crates/dla-cli/src/
   alloc_cmd.rs: the helpers around `window_fits` and the `--extra` note): check that this path is
   taken for the new `TooShort` and that the printed `--extra` covers it.

3. **An inconclusive alloc-test that asks for a longer window decides nothing while the budget can
   pay for it.** crates/dla-core/src/grown.rs `decided_by_alloc_test`: an alloc-test on a hook
   driver made for the finding decides its growth "whatever it favors; inconclusive: the close is
   the model's to make". So a model can close the finding `unresolved inconclusive` and stop with
   most of the budget left, though the alloc-test named the window that would decide it. Change:
   an inconclusive alloc-test whose deciding column (the primary column, else the voting ones)
   needs more extra points than it ran (`points_needed`, stair or rises) does not decide while
   `budget_over_floor` is Some for the run (the worker's, in a worker view): the lead stays, with
   the command `dla alloc-test <H> --finding <F> --extra <n>` as its first fix, so `ledger resolve`
   refuses the close, the gate lists it and the Stop hook holds the stop the way it does for other
   undecided growth. At or under the floor, or once an alloc-test ran with at least that many extra
   points, the old rule holds. Find out whether the alloc-test evidence records `points_needed`
   per column (the Plateau JSON); if not, record it (`#[serde(default)]`-safe on read: older
   evidence has none and keeps the old rule). Keep the texts short and in the style of the other
   grown.rs fixes.

## Tests (each must fail with its fix taken out: check it and report)

- judge.rs: the series above (main/extra as the code splits them) reads `Continues` for the
  column (fails today: it reads plateau). The same with the extra window cut after 6600 (no step
  yet) reads `Inconclusive` with `points_needed` that reaches past 7040 (fails today: `Stopped`
  or plateau). A staircase that stops (steps, then flat for longer than 2 x (last gap + spacing))
  reads plateau. A spike (one high point in the extra window, back down at its end) is not
  `Continues`. A ramp keeps its old verdict (an existing test may already cover it: name it).
- alloc_cmd: an alloc-test whose primary column is a staircase cut short extends or prints
  `--extra` with a count that covers the expected gap (find the existing tests near the
  `points_needed` assertions in alloc_cmd.rs).
- grown.rs (and one CLI test, find the grown-chain tests in crates/dla-cli/tests): with budget
  over the floor, an inconclusive alloc-test on the driver made for F that names more extra points
  than it ran leaves the lead (its fix is the alloc-test with `--extra n`) and
  `dla ledger resolve F --as unresolved --reason inconclusive` is refused; with budget at the
  floor, or after an alloc-test with that many extra points, the close is allowed as before; an
  evidence record without `points_needed` keeps the old rule.

## Checks (after your last edit)

`cargo test -p dla-core`, `cargo test -p dla-cli --no-fail-fast` (the whole suite), `cargo test -p
dla-workload --no-fail-fast`, `cargo test -p dla-report`, `cargo test -p dla-bench --lib`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check`. Then the whole
dynamic set from the repository root: `./target/debug/dla-bench dynamic --dla-bin
./target/debug/dla --parallel 2 --json <scratch file>` (expected 63 pass and 1 expected failure,
`rust-tree-parent-links`); delete its logs after. Docs: one row at the end of docs/VERIFY.md for
the cause (re-read it before you write), and the sentence of
skills/detect-leak-anywhere/references/ that describes the alloc-test plateau reading (find it;
grep `stair`, `plateau`, `--extra`) where it now says something false.

## Rules

Shared `target/` only. Check `df -h /` before building (about 3.7 GB free); do not delete build
directories; if a build or the dynamic run fails for lack of space, stop and report it. No
`rm -rf "$VAR/..."`. Never kill processes by name. Never skip or weaken a test. No project names,
attribution lines or model names in the repository. Scratch:
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u159/.

## Report (final message)

Files changed; for each fix the rule as implemented (exact conditions and constants), what the
real series above now reads (both the full one and the one cut at 6600), and whether any existing
test's expectation changed and why; whether `points_needed` was already in the evidence; tests
with what each proves and its mutation check; every command with pass/fail counts, run after the
last edit; the dynamic result; anything you are unsure of.
