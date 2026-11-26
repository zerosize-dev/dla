# Builder L: growth measured on a shared loop stays a lead until a driver made for the finding decides it

Repository: /home/user/dla. Work in the working tree. Do NOT commit, push or switch branches.
The tree has uncommitted, checked work from other builders that is in CI now. Do not revert or
reformat files you did not write. Add one row at the end of docs/VERIFY.md (re-read it first).
Do not touch harness/ (the agents), bench/fixtures/ or any fixture README or answer key.

## What a real run showed (do not name the project in the repository)

An unattended run on a JVM library, 3-hour budget. The documented leak is a holder field
(call it the holder) that grows in one thread arrangement. The run:

1. Ran the project's JUnit test loop (`dla harness gen --lang jvm --junit`, harness H4). Thirteen
   findings' probe columns grew on it, the holder among them (`probe:...holder` +35 per
   iteration, gate pass).
2. Ran `dla alloc-test H4 --finding F` for each. Every one favored leak (plateau
   `growth-continues`, extra cycles kept rising).
3. The model added and refuted each non-leak sibling, then its own script skipped every
   confirm ("measured only in a JUnit loop/shared driver where many encoders are alive together;
   an isolated driver and heap-graph retention path would decide"), and closed all thirteen in
   one loop: `dla ledger resolve F --as unresolved --reason inconclusive --remedy "<that>"`.
   Every close was accepted. That was 10 minutes into a 3-hour run; the run never came back to
   them, and the documented leak ended "unresolved".

The model's judgment was right: a test loop keeps many objects of the program alive together,
so growth there may be kept by another holder. But the tool let it stop at the judgment
instead of making it do what it said would decide.

## Why the tool accepted it (read these first)

- `crates/dla-core/src/grown.rs`, `grown_unmeasured`: a passed growth column of a harness
  series that points at a leak-side root makes a lead (`Grown`), unless `alloc_tested(view,
  &root)` is true. `alloc_tested` is true for ANY `plateau+purge` discriminator bound to the
  family, whatever it favors and on whatever harness it ran.
- `crates/dla-cli/src/ledger_cmd.rs`, `refuse_grown_unmeasured`: refuses a close while a lead
  exists. So any alloc-test lifted it.
- `grown.rs`, `route_to_confirm` / `route_for`: a route to confirm (and `refuse_leak_favored`)
  exists only when the newest alloc-test on a harness about the finding favors leak AND ran on a
  hook driver made for the finding (`harness_about` returns `Target` or `EntryPath`). An
  alloc-test of a test loop, of the project's own workload, or on a harness about the finding
  only through a probe, a column, cited evidence or a capture gives no route (its doc comment
  says why). So after a leak-favoring alloc-test on the test loop, nothing held the finding.
- The dump-lead rule (`dump_grown`, same file) already says this for dumps: "an alloc-test that
  favors leak does not lift it: on a driver not made for the finding it gives no confirm route".
  Only an alloc-test favoring non-leak lifts a dump lead there. The trend-column lead lacks it.
- The pytest loop already has the right commands (`commands(..., pytest_loop: true)`): a small
  driver of its own first, the alloc-test on that harness, not on the loop.

## The change

1. **The lift (dla-core, grown.rs).** In `grown_unmeasured` only, replace `alloc_tested(view,
   &root)` with a stricter "decided by an alloc-test": take the alloc-tests bound to the family
   (`bound_alloc_tests`) whose harness is about the root (`harness_about`, the manifest loaded
   from the run directory, as in `route_to_confirm`). The newest one decides. The lead is lifted
   when that one favors non-leak, or when it ran on a hook driver made for the finding
   (`m.is_hook_driver()` and `Target | EntryPath`), whatever it favors (favors leak: the confirm
   route of `route_to_confirm` and `refuse_leak_favored` take over; inconclusive on its own
   driver: the close is the model's to make). When it favors leak or is inconclusive on a
   harness not made for the finding, the lead stays. With no run directory (unit views without
   manifests) keep today's behavior. Leave the other callers of `alloc_tested` (gate.rs
   `capture_accounted`, grown.rs `detector_leads`) as they are, unless a test shows they must
   follow; say which if you change one. Do not change `dump_grown`.
2. **What the lead says.** Record on `Grown` the shared alloc-test that did not decide it (its
   evidence id, its harness, what it favored, and why the harness is not made for the finding:
   a test loop, the project's own workload, or about it only through a probe, column, cited
   evidence or capture). `grew_text` stays. The commands (`commands()` / `own_commands()`):
   the sibling add when missing, the test when the state allows it, then a driver made for the
   finding and its alloc-test, generalizing the pytest-loop branch to every runtime (the
   runtime from the harness manifest):
   `dla harness gen --lang <rt> --driver --probe <F>   (a driver made for <F>: its unit calls the code that adds to the holder, and nothing else of the program; on <H> many objects are alive together, so growth there may be kept by another holder)`,
   `dla harness run <H'>`, `dla trend <S>`,
   `dla alloc-test <H'> --finding <F>   (on the harness just made, not on <H>)`.
   Check that `dla harness gen --lang <rt> --driver --probe <F>` is accepted for jvm, node,
   python, go and rust, and makes a harness whose `targets` include F (so `harness_about` says
   `Target`); if a runtime needs another flag, use the one that works and say so.
   Keep the `same_leak` duplicate offer as it is.
3. **The refusal (ledger_cmd.rs, `refuse_grown_unmeasured`).** A new branch for this case:
   message "<F> cannot be closed: <grew_text>, and the alloc-test <E> on <H> (<why: a test loop
   / the project's own workload / ...>, not a driver made for <F>) favored <leak|inconclusive>:
   growth there may be kept by another holder", wanted "an alloc-test of <F> on a driver made
   for it (growth measured is decided, not closed)", fix the commands joined as today. Code
   stays `growth-not-measured`. `time_budget` keeps its own rule (allowed only at or under the
   floor, by `refuse_time_left`), and an aborted run is not held, as today.
4. **The gate item and close-rest.** The final gate's growth item for such a lead (grown.rs,
   where the `BlockingItem` of kind `Growth` is built with "and no alloc-test decided it") says
   the same thing in its own words, with the same fix. `dla ledger close-rest` must leave such a
   finding open (check `ledger_queue.rs`; it reads `grown_unmeasured`). Make sure the reason text
   close-rest prints for a left finding does not claim "no alloc-test".
5. **`dla alloc-test` output (alloc_cmd.rs).** When the result favors leak on a harness about
   the bound finding but not made for it, add one sentence to the output (a `hint`-style field;
   find how the command already prints extra sentences, e.g. `stacks_hint`) saying that a
   confirm from <H> may count growth another holder keeps and that a driver made for <F> decides
   it alone, with the `harness gen` command. Do NOT change the `next` list unless the bench
   policy (bench/runner/src/dynamic/policy.rs and ops.rs) is unaffected: first find out whether
   the mechanical policy follows `next` after an alloc-test on a test loop or a workload harness
   and which fixtures do that. Report what you found.
6. **The Stop hook (crates/dla-cli/src/gate_hooks.rs).** Builder K made `held_queue`: while the
   budget is over the floor, nothing was refused for time with nothing smaller that fits, and
   the final gate's missing items include a `queue`, a stop is not counted toward `STOP_LIMIT`.
   Extend it to a missing item of kind `growth` (a measured growth not decided) when no queue is
   open: same conditions, `held_note` names the item and its first command instead of "the queue
   is open" (for example "... and growth measured on <H> is not decided (<F>). Run <first
   command>."). A queue item, when present, still wins. Workers: same rule as K's, no abort
   clause for a worker.
7. **Docs.** skills/detect-leak-anywhere/references/stages.md: one sentence where alloc-test
   and leads are explained (find the dump-lead sentence): growth on a test loop or the
   project's own workload is decided by an alloc-test on a driver made for the finding, or by
   one that favors non-leak; a leak or inconclusive result on the loop does not let it close.
   Do NOT edit SKILL.md (its byte budget is full) unless a test requires it. VERIFY row.

## Tests

dla-core unit tests (grown.rs has a test module with views built in code; follow
`only_a_driver_made_for_the_finding_gives_a_route` and the pytest-loop tests):
1. A test-loop series grows a column tied to F1; an alloc-test on the loop favors leak: F1 is a
   lead, its commands name `harness gen --lang <rt> --driver --probe F1` and the alloc-test on
   the new harness, not on the loop.
2. The same with the loop alloc-test inconclusive: still a lead.
3. The loop alloc-test favors non-leak: lifted.
4. Then a driver made for F1 (targets F1, hook driver) gets an alloc-test: lifted for each of
   leak (and `route_to_confirm` is Some), inconclusive (route None, lead gone) and non-leak.
5. Order: a made-for-it alloc-test inconclusive, then a newer loop alloc-test favoring leak: the
   newest decides, so the lead is back (or argue otherwise in the report if you find a reason;
   the rule must be written in the doc comment either way).
6. A command-mode harness with a workload (OwnWorkload) behaves like the test loop.

CLI tests (crates/dla-cli/tests; find the tests of `growth-not-measured` and the shared helpers
they use, e.g. grown_chain.rs, grown_dump.rs, ledger_close tests):
7. `dla ledger resolve F1 --as unresolved --reason inconclusive --remedy x` after a loop
   alloc-test favoring leak is refused with `growth-not-measured`, the message names the
   alloc-test and the harness, the fix names `--driver --probe F1`.
8. After an alloc-test on a driver made for F1 that is inconclusive, the same resolve is accepted.
9. `close-rest` leaves F1 open in case 7.
10. `--reason time_budget` at or under the floor is accepted in case 7 (as today).
11. Stop hook: budget over the floor, no queue, a growth item open: four stops are blocked
    without being counted, the reason names the growth item's command; at or under the floor the
    third is let through; with a queue open the queue note wins.

Mutation checks: (a) make the new lift return `alloc_tested` again and see tests 1, 2 and 7
fail; (b) drop the made-for-it condition (lift on any alloc-test favoring leak) and see a test
fail; (c) make the hook ignore growth items and see test 11 fail.

Existing tests: some may expect a loop alloc-test that favors leak or is inconclusive to lift a
lead (search for `alloc_tested`, `growth-not-measured`, pytest-loop tests, `close-rest` tests).
Change one only as far as the new rule requires, and list each with why. Never weaken a test.

## Checks to run

- `cargo test -p dla-core --lib`, `cargo test -p dla-cli` (all its test binaries),
  `cargo test --workspace` once at the end (it includes dla-report golden tests),
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check`.
- The dynamic acceptance on the fixtures that use a test loop or a workload harness and an
  alloc-test (find them from the policy and fixture files), with the debug build:
  `./target/debug/dla-bench dynamic --dla-bin ./target/debug/dla --parallel 4 --only <substring> --json <scratch file>`
  (run from the repository root; `--only` takes a substring, run it once per substring). Report
  pass/fail per fixture. If a fixture fails, find out why before changing anything: the fixture
  is never changed to pass.

## Rules

Shared `target/` only. If `df -h /` shows less than 3 GB free, delete stale hash dirs older than
120 minutes under the literal paths /home/user/dla/target/debug/build/dla-cli and
/home/user/dla/target/debug/build/dla-core with
`find <literal path> -mindepth 1 -maxdepth 1 -type d -mmin +120 -exec rm -rf {} +`.
No `rm -rf "$VAR/..."`. Never kill processes by name (another bench may be running from a copied
binary; leave it alone). Never skip or weaken a test. No project names, attribution lines or
model names in the repository. Scratch files go under
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u140/.

## Report (final message)

Files changed; the rule as implemented (conditions, texts); what the bench policy does after a
loop alloc-test and whether `next` changed; the tests with what each proves; the mutation
checks; every command run with pass/fail counts; dynamic fixtures run with results; existing
tests whose expectations changed and why; anything you are unsure of.
