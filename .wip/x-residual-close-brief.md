# Round X: what a run may claim before it stops, and the set-up that leaves the consumer out

sp = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad. Scratch: `$sp/u172/`.
Repository: /home/user/dla, branch main. The working tree already holds round W, which is not committed
(one verdict for a capture series: see `git diff`, and `docs/VERIFY.md`). Build on it; do not revert it.

## Why (the same real Rust run, r8)
Kept files (small): /root/dla-real/r8-fastrace-keep/. Diagnosis with code references: `$sp/u169/summary.txt`
(sections residual-and-stop and model-decisions). Its CORR lines are binding.

After round W, four general holes are left.

1. **One residual item stood for six series.**
   - `dla gap --residual` keys the open item on the region only (crates/dla-cli/src/gap/residual.rs
     `item_region`, `open_item_for`). It opened O1 for a heap row of 0.94 bytes per iteration (S4).
   - Later rows of 2 MB per iteration (S5, S6, S7, S9) got `Action::Keep(O1)`.
   - The model resolved O1 as "allocator noise", citing only the small row. gate.rs `residual_covers` and
     `residual_item_resolved` then counted every row as covered.
   - The report still printed "ABOVE THRESHOLD, see O1" next to a resolved O1.
2. **close-rest wrote a claim no measurement made.** ledger_queue.rs ~296 maps `Verdict::Executed` to
   "its code ran and showed no growth". The runtime had no probe for the holder, and no column of any run
   measured it.
3. **The model summarized a capture while it was still being written, and ran the same harness twice at
   once.**
   - It read p4's `dump.jsonl` while the first `dla harness run H4` was still writing it.
   - It took the parse error as "the run was cut off" and started a second `dla harness run H4`, which
     overlapped the first and wrote the same `out/` files.
   - It then summarized p1-p3 only. The run record also counted 0 capture files.
4. **The set-up was slowed, not changed.**
   - Two drivers missed the adding lines of the target with one set-up. The queue's set-up hold
     (crates/dla-core/src/queue.rs `setup_hold_sentence`, ~283-301) told it "change `setup()` too (for
     example, leave out what consumes or resets that state)".
   - That sentence reached the model cut short ("...leave out what..."), because the output bound cuts an
     entry at 160 characters and the action is at the end.
   - The next driver kept the consumer and slowed it. That reached the writer line, but the consumer still
     drained what it added.
   - The leak needs the consumer absent.

## What to build

X0. **Pin round W's harness on heap-graph series (a test gap).**
- Round W made the JVM heap-graph summary (`summarize/hprof.rs`, `series_part`) and the Node
  heap-snapshot summary (`summarize/node_heapsnapshot.rs`) write `source.harness` when every dump lies
  under one `harness/<H>/`.
- No test fails when that line is removed: I mutated it in hprof.rs and `summarize_jvm` and
  `capture_verdict` stayed green.
- Add assertions, in existing tests where they fit:
  - dumps under `harness/H1/out/...` give `source.harness = "H1"`;
  - dumps outside any harness, or under two harnesses, give none.
- Do this for both summaries and mutate each line.

X1. **A residual item covers the rows it was written for.**
- An open residual item records the series and the largest residual it was opened for, or that a later
  row was added to it. A row of another series, or a row larger than what the item recorded, re-opens the
  item if it is resolved, or adds the row to it if it is open. In either case the text names the largest
  row (series, bytes per unit, threshold).
- Do not key items by (region, series): that can open dozens of items. One item per region is fine
  if it lists its rows.
- `residual_covers` / `residual_item_resolved`: a resolved item covers a row only if the resolution was
  made after the row was listed on it. A row added after the resolution is uncovered.
- A resolution must cite a measurement of the largest listed row's series or harness: an alloc-test, a
  capture summary of that harness, or the row's own evidence. Read `refuse_uncaptured_residual`
  (crates/dla-cli/src/ledger_cmd.rs ~1259). It must require the capture or attribution of the row's own
  series, not just "the harness has any summarized capture".
- A row under its threshold stays a non-leak and needs no item, as now.
- Report (crates/dla-report/src/lib.rs, the residual rows): for a row whose item is closed, print
  "resolved as <reason>: <remedy, clipped>" instead of a bare "see O1". For a row the resolution does not
  cover, print that it is not covered.

X2. **close-rest says what was measured.**
- Where `default_remedy` maps the best verdict to words, `Executed` reads "its code ran and showed no
  growth" only when some run measured the holder. That is a probe column, a site or holder column tied to
  the finding, or a capture summary of that harness with a column at the finding's site or writers.
- Otherwise it reads "its code ran; its holder was not measured (<the runtime> has no probe for it)". If
  the harness has an allocation-stack or heap capture summary with no column at the writer lines, add "and
  the captures of <H> show no growth at its writer lines".
- Check every other text that says "showed no growth" for a verdict (the report's Findings lines,
  queue texts, trend verdict detail), and make it the same.
- Do not change which candidates close-rest closes, except as X1 or round W already hold them.

X3. **No summary of a capture that is still being written, and no second run of a running harness.**
- `dla summarize` on a shim dump (`dump.jsonl` / `shim-dump.jsonl` of Rust and native drivers): a dump
  with no exit record is refused, or left out with a limitation if other captures are complete.
  The text: "capture <label> has no exit record: it is still being written or its process was cut; wait
  for `dla harness run <H>` to end (dla run status), then summarize every capture it lists". Name the
  harness from the path.
  - Find the equivalent for a truncated heap snapshot, hprof or heap profile. Do it only where the
    parser can tell "incomplete" from "unsupported" today; otherwise list that as not done.
- `dla harness run <H>` while another `dla harness run <H>` command is alive (registered in the run state;
  the pid is alive) is refused with code `harness-running`. The text says the pid, when it started, and
  the fix: wait for it, or `dla run stop` to cut it.
  - Today `limit::stop_leftovers` stops processes that a cut command left behind. Keep that.
  - A dead command's leftovers still go as now.
  - A second harness, or a run of another harness, is not refused.
- The run record of a capture run counts its capture files and bytes (from `out/p*-alloc`, and the
  other capture kinds the record lists). r8's record said 0 files.

X4. **"Leave the consumer out" reaches the model, and a held target's driver shows the set-up to
change.**
- `setup_hold_sentence`: put the action first, so the first 160 characters carry it. For example:
  "{id}: change `setup()`: leave out what consumes or resets the state (absent, not slowed: a slow
  consumer still drains it); {who} missed its adding lines with the same set-up; ...".
  - Keep the rest of the sentence, and keep it one sentence.
  - Find where `next` and `_attention` entries are cut (the 160-character output bound) and check that
    no other hold or refusal sentence has its command or action after character 160. Fix any you find
    the same way: action first.
- The driver block that `dla harness gen` writes for a target with a set-up hold (targets.rs
  `add_writer_gaps`, gen.rs ~3098) says the same in its first line. It points at the set-up function of
  the skeleton for that runtime.
- Skill text, short. Add one sentence where the references describe `writer_not_executed` and the set-up
  hold (references/stages.md ~326 and ~341, references/rust.md ~28):
  "for a guard that fills when nothing drains it (a full buffer, a backlog), run the set-up with the
  consumer absent; a slowed consumer is a different state, not a substitute".
  Do not add more than that.
- No change to when the hold applies (queue.rs `setup_hold`, `same_setup_hold`).

## Tests (required)
- X1:
  - a dla-cli integration test. Two series of one region: a small row opens O1, a large row of another
    series is added to it. Resolving O1 citing only the small row is refused, or does not cover the large
    row. A row added after a resolution re-opens it. The report prints "resolved as".
  - An under-threshold row needs nothing.
- X2: a unit or integration test where close-rest closes a target whose best verdict is `Executed`.
  - With no measured holder, the remedy says "not measured".
  - With a probe column of the holder, it says "showed no growth".
- X3:
  - summarize of three complete shim dumps and one without its exit record: the incomplete one is
    named, with the wait text.
  - A second `harness run` of the same harness while a registered command is alive is refused;
    the same after that command is dead is not refused.
  - The capture-file count of a capture run is not 0.
- X4: the hold sentence's first 160 characters contain "leave out" and "absent"; the driver block of a
  held target starts with it.
- A mutation check for each new test: a unique marker, then restore, then `grep -c` the marker = 0.

## Do not
- Do not change what confirms a leak, the capture rule, the trend gate, or what counts toward
  `failed_beyond_process`.
- Do not make close-rest refuse or hold more than X1 and round W already do.
- Do not add a remover-in-the-same-function tie or hint.
- Do not make an under-threshold residual row an item.
- Skills/agents/data/hooks: only the one sentence in X4 and texts that state the old behaviour.
  docs/VERIFY.md: add or update rows for what changed.

## Rules
- No commit, no push, no branch switch. Do not touch round W's changes except where an item needs it, and
  say where.
- `target/` is shared. Free disk only with `cargo clean -p dla-cli -p dla-core -p dla-workload -p dla-bench`.
- Literal rm paths. Kill only your own processes, by pid. Never weaken a test to pass.
- The repository never names the evidence run's project or its APIs; fixtures and test names stay
  generic.

## Checks
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and the test
  targets you touched. Not the whole suite: I run it.
- Then the dynamic set once:
  `./target/debug/dla-bench dynamic --dla-bin ./target/debug/dla --parallel 2 --json $sp/u172/dyn.json`
  after building dla and dla-bench.
  - Expected: 63 pass plus 1 xfail (rust-tree-parent-links). jvm-pool-warmup is a known flake: if it
    alone fails, run it alone up to 3 times.
  - Compare confirmed/claim/rule-out/leaks per fixture with `$sp/u170/dyn.json`, which is round W's
    run, and explain every difference.
  - Delete the dynamic logs afterwards.

Report as your final message (the Write tool may refuse a report file). Include:
- files changed;
- each item as implemented;
- tests and the mutation that made each fail;
- commands and results;
- the dynamic comparison;
- anything unsure or not done.
