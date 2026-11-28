# Round Y: a capture sees the state while the program still holds it, and a measured heap growth is decided

sp = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad. Scratch: `$sp/u177/`.

Repository: /home/user/dla, branch main at d9d0a4e (0.9.0-preview.16, rounds W and X committed). The working
tree is clean. `target/` was cleaned, so the first build is a full one.

## Why (real Rust run r9)

Kept files: /root/dla-real/r9-fastrace-keep/. The full run directory is
/root/dla-real/r9-fastrace/project/.dla/20261128-051153-38ac (read only).

The diagnosis, with records and code lines, is `$sp/u176/summary.txt`. Each cause has a verifier's VERIFY
and CORR part. The CORR parts are binding where they narrow a fix.

The run ended COMPLETE after 4 of 167 minutes, with no leak confirmed. The leak was a per-thread `Vec`: a
sender's pending list, which grows without limit once a bounded hand-off is full and nothing drains it.

The agent's focus driver H3 did reproduce it (it set a consumer that blocks forever):
- series S5 has in-process native.live_bytes 2.72 -> 3.58 -> 4.56 -> 6.53 MB;
- native.live_blocks stays at 12886 (one block resized);
- trend T4 passed native.live_bytes (stair).

Everything after that failed in general ways:

1. **C0: the allocation capture was blind to the growth.**
   - The shim (harness/native/dla_alloc_shim.c) writes its record from `__attribute__((destructor))
     dla_shim_fini` (~1441).
   - glibc's exit() runs the calling thread's thread-local destructors, then atexit handlers and C++ static
     destructors, before ELF destructors. So a Rust `thread_local!` (and C++ statics and thread_locals) on
     the main thread is freed before the dump.
   - Experiment `$sp/u175/tls`: a thread-local Vec of 1,000,000 u64 gives exit native.live_bytes 548. A
     static Mutex<Vec> gives 8,000,548.
   - `$sp/u176/captures` (analyst): a worker thread alive at exit is visible. A patched shim with an
     exported snapshot called from main before return shows 8,000,589 and writes exactly one record.
   - In r9, every H3 exit record has live_bytes 2,510,682 and blocks 12,810, at x = 440, 880, 1760 and 3520.
   - The same records' native.peak_bytes grows 2.68 -> 6.49 MB.
   - The same process's own checkpoints (in the same dump.jsonl, before the shim record) read 2.68 -> 6.49
     MB live.
   - Nothing compares these. `dla summarize alloc-stack-diff` said "nothing grows strictly across all 4
     captures" and "capture a point with more work".
2. **`dla trend S5` stdout hid native.live_bytes.**
   - The passed list is ordered by p-value, so rss columns come first.
   - The generic 2048-byte output bound (crates/dla-cli/src/out.rs) then cut arrays to 4, with only a
     bare `truncated:true`.
   - The stored evidence E24 has 6 passed columns. The agent saw 4 (os.rss, rss_anon, vm_hwm,
     native.peak_bytes) and wrote "live heap flat".
   - `dla run status` also lost `budget` to the fitter (call 37: KeyError).
3. **A blind capture counted as "the capture".**
   - Round X made leaving a heap residual unresolved need a summarized capture of the harness.
   - gate.rs `capture_of_series` (~1409) checks only that such a capture exists and when it was made.
   - ledger_queue.rs `holder_measured` returns `CapturesOnly` whenever a summary exists. close-rest then
     wrote "the captures of H2, H3 show no growth at its writer lines" into the documented holder F41.
4. **One free-text `inconclusive` closed the measured growth with 163 minutes left.**
   - O1 (heap 1117 B/iteration, S5, threshold 710) was resolved unresolved/inconclusive. Its remedy said
     "live heap flat" (false) and "needs a longer run".
   - Only `time_budget` is held to the budget (ledger_cmd.rs refuse_time_left).
   - The same resolve also covered rows of S9, S11 and S14 (another driver), which it did not name.
   - The gate text for O1 still said "attribute it first: dla harness run H3 --mode fresh --capture alloc"
     after that capture had run.
5. **Whole-heap growth can never point at a target of a runtime with no probe.**
   - H3 was a focus harness for F41. F41's writer line spsc-like :55 ran 1913 times (executed-S5.json).
     Its verdict was `executed`.
   - trend/candidates.rs gives a column without a site no candidates. verdicts.rs `one()` falls to
     coverage. Nothing offered `dla alloc-test H3 --finding F41`, which harness_about allows and
     alloc/pick.rs `region_primary` would judge on native.live_bytes.
6. Small, general:
   - rank.rs `NON_PRODUCT_DIRS` misses Cargo's `benches` (the plural-s strip gives "benche"), so a bench
     file led the queue.
   - The second-look slot text "fail or drop half-way" (dla-core/src/harness.rs ~859) let the agent write
     `std::mem::forget`, a leak of its own, which cost a harness and a finding.

## What to build

### Y1. A snapshot while the state is alive (fixes C0 at the source)
- **Shim export.** In the shim, export `dla_shim_snapshot(void)` (DLA_EXPORT, C ABI). The x of the record
  must be the driver's current x: either an argument or the current DLA_ITER logic; choose and document.
  - It writes the same record as `dump_record` does today: phase "exit", same fields. Add one field that
    says when it was taken: `"at":"snapshot"`, `"exit"` or `"signal"`.
  - The existing `dumped` exchange makes the destructor (and the SIGTERM path) write nothing after a
    snapshot. A process that never calls it keeps today's behaviour exactly.
  - `dla summarize`'s exit-record rule (X3: a shim capture without an exit record is unfinished) must
    keep working unchanged.
- **Drivers DLA generates call it** after the last unit and the last checkpoint, before any state of the
  units is dropped, and only when the symbol is present. Without the shim, or with DLA_SHIM_DISABLE=1, it
  is a no-op with no error.
  - Rust template (harness/rust/dla_harness.rs `main`, ~367-420): look the symbol up at run time with
    `dlsym(RTLD_DEFAULT, ...)` through an `extern "C"` declaration. No crate dependency and no link-time
    reference. RTLD_DEFAULT is null on glibc and `(void*)-2` on macOS: use cfg. Windows has no shim:
    compile the lookup out there. Check whether glibc before 2.34 needs `-ldl` for the pinned toolchain
    and handle it.
  - C template (harness/native/dla_harness.c): a weak declaration.
  - Every other generated driver that runs under the shim (find them: crates/dla-workload/src/harness/
    run.rs, capture.rs, templates.rs; for example Python through ctypes, if the Python path preloads the
    shim): call it where the language reaches a C symbol with no new dependency. List in VERIFY.md which
    paths call it and which do not. Y2 covers the ones that do not.
- Update the shim header's "Limits" comment and harness/native/README.md with the rule: the exit record
  holds only what survives exit (calling thread's thread-locals and static destructors are gone); a
  snapshot holds the live state.

### Y2. A capture is checked against the process's own readings
- **Keep the driver's readings.** When a capture's dump.jsonl holds the same pid's checkpoints and the
  shim record (crates/dla-workload/src/harness/capture.rs `split_shim_dump` ~732 and run.rs
  rust_alloc_points), keep the last checkpoint's x, native.live_bytes and native.live_blocks with the shim
  record, for example as `"driver":{...}`. Today they are thrown away.
- **Compare in `dla summarize`** for shim records (alloc-stack-diff, stack-groups, any summarize that
  reads shim records; crates/dla-cli/src/summarize/shim_groups.rs).
  - Compare GROWTH over the ordered captures, not totals: the exit record is normally somewhat below the
    in-process value; at x=440 it was 0.17 MB below.
  - The capture is **blind** when both hold:
    - the driver's live bytes rose by ΔD between the first and the last capture, with ΔD above a floor
      (state it; for example 64 KiB and 10% of the first value);
    - the shim totals rose by less than half of ΔD.
  - `peak_bytes rising while live is flat` is a hint printed next to it, never the trigger alone
    (transient buffers give the same shape).
  - A blind capture prints both series of numbers and the sentence that it cannot clear or explain
    growth the process measured.
  - Its `needed` names the snapshot (Y1). If the snapshot was taken and the capture is still blind, say
    that the state is held outside the shim's view, and name what to try.
  - Replace "capture a point with more work" in both branches (~1044-1051). Give the "x differs by less
    than 2x" advice only when the x values really differ by less than 2x.
  - The gate value stays what the stack rule computes; add `blind: true` with the numbers. A blind
    capture never passes a stack it did not see, and it must not become a leak verdict.
- **Downstream, a blind capture is not a capture:**
  - gate.rs `capture_of_series` and `uncaptured_residuals` (X1 rule): it does not satisfy "a summarized
    capture";
  - measured_holders.rs `scan.summarized` and ledger_queue.rs `holder_measured`: it is not `CapturesOnly`.
  - Without a checkpoint of the same pid (command-mode captures of the project's own processes), skip the
    check and add a limitation that the exit view may miss state freed at exit.

### Y3. Outputs that drop what matters say so
- **`dla trend` passed list.**
  - Put region-primary live columns (the ones alloc/pick.rs `region_primary` would judge: native.live_bytes,
    heap-live columns of each runtime) first, then other heap columns, then process columns.
  - Print per-region counts (for example `passed_by_region: {heap: 3, process: 3}`).
  - Never cut a passed list without naming what was cut. Do not merge rows across regions.
- **out.rs generic cut.** When it cuts an array, record the key and the count cut (for example
  `"truncated":{"passed":2}`, or a sibling `"<key>_cut": n`), not a bare true.
  - Keep the change small and schema-checked (`dla schema --check`).
  - Protect `budget` (remaining time) and `revisit` in `fit`'s DETAIL drop order for `run status`: they
    go last.

### Y4. A measured heap growth is not closed by text while budget remains
- **Refuse inconclusive while undecided.** Refuse `dla ledger resolve O<n> --as unresolved --reason
  inconclusive` for an item when all of the following hold:
  - one of its rows is above its threshold, in the heap region;
  - the row's series passed a region-primary heap column in its trend;
  - nothing decided it: no non-blind summarized capture of that harness that shows a growing stack, and
    no alloc-test on that harness;
  - `budget_over_floor` (gate.rs ~2689) is Some.
- **The refusal names one command that can decide:**
  - `dla alloc-test <H> --finding <F>` for each declared target of H that Y5 lists;
  - otherwise a capture with the snapshot.
- **Allowed after** an alloc-test on H ran (any outcome; an inconclusive one after its longest window
  counts as measured, as grown.rs's "longer" rule does), and at the floor. No deadlock.
- **A remedy that resolves an item with rows of several series above their thresholds must name each of
  those series (S<n>).** Refuse otherwise and list them. Rows under their thresholds need nothing.
- **The final gate's text for a heap item** whose harness capture was already summarized says that, and
  what it showed (blind with numbers, or flat). It does not repeat "attribute it first: ... --capture
  alloc".

### Y5. A no-probe target in a harness whose heap grew is a lead, not "executed"
- **When it applies:** in `dla trend`, a harness series passes a region-primary live heap column (not
  native.peak_bytes, os.vm_data or other high-water or process columns), and no site or probe column ties
  it to any declared target.
- **Which targets:** the declared targets of that harness whose writer lines ran (count above 0), in the
  heap region.
- **Their verdict** is a new non-settled one (for example `grew_untied`). Its detail is "the heap of H
  grew (S<n> native.live_bytes <shape>, +X MB) and nothing measured its holder; its writer lines ran N
  times". Several targets are all listed.
- **Effects:**
  - The queue does not settle them.
  - `dla trend` prints `dla alloc-test <H> --finding <F>` first in `next`.
  - grown.rs raises the existing "growth measured, not decided" obligation for them, so close-rest and
    resolve refuse while over the floor. Do NOT use Verdict::Changed (it is settled, queue.rs ~513-515).
  - An alloc-test on H decides them as today: favors leak gives the existing confirm route (b91cbe6 rule);
    non-leak settles.
- **Limits:**
  - It never credits the column in gap/residual.rs before a decision.
  - It never confirms by itself.
  - It is not given to targets that never ran.
- **Wording** (ledger_queue.rs `executed_outcome`, `default_remedy`):
  - for such a target: "the heap of H grew (S<n>, E<k>) and nothing decided it", never "write a unit"
    when its writer lines ran;
  - "the captures of H show no growth" only when a non-blind capture of H was summarized;
  - for a blind one: "the captures of H could not see the state (blind: ...)".

### Y6. Small general fixes
- rank.rs: match `benches` (plural with `es`), and check `examples`, `tests` and the other Cargo
  conventional directories against the same rule.
- dla-core/src/harness.rs ~859 slot text: "drop the object early (never forget or leak it)".
- `dla harness run`: warn (never fail) when a Rust unit's body contains `mem::forget`, `Box::leak` or
  `ManuallyDrop::new`, and say that the driver itself then leaks. Only the unit and set-up the agent
  wrote, not project code.

### Y7. A fixture pair for the per-thread class
- Add a new original Rust fixture pair under bench/fixtures (follow bench/FIXTURES.md and an existing
  Rust pair such as rust-batch-buffers for layout, answer key and README). Generic names that do not
  resemble the evidence project.
  - Leak fixture: a per-thread outbox (thread_local) that overflows into an unbounded Vec when a bounded
    hand-off is full and no consumer drains it.
  - Twin: the same API where the consumer drains (or the overflow is bounded).
- The dynamic bench must confirm the leak through the capture path (the snapshot makes the stack visible
  at the push line) and report no leak for the twin.
- Do not change any existing fixture, answer key or README.

## Tests (required)
- **Y1:**
  - tests/shim/alloc-shim.sh (or a new script there): a program with a thread-local buffer that calls
    the snapshot shows the buffer live; without the call the exit record does not.
  - One record per process in both cases.
  - Must run on Linux and macOS (the CI shim job runs both).
  - An integration test of the generated Rust driver (dla-workload, like the existing Rust capture
    tests): a unit that fills a thread_local gives a shim record with the bytes live.
- **Y2:** summarize tests with fixtures:
  - driver readings rising and shim totals flat gives `blind` with numbers, `needed` without "more work",
    and `capture_of_series` false;
  - both rising gives no blind;
  - no checkpoint gives the limitation and no blind;
  - a transient peak with live flat in both gives no blind.
- **Y3:**
  - a trend whose 6 passed columns include native.live_bytes prints it, and prints counts;
  - a generic cut names key and count;
  - `run status` under the bound keeps budget.
- **Y4:**
  - the refusal fires over the floor, names alloc-test, and clears after an alloc-test on H and at the
    floor;
  - the multi-series remedy refusal;
  - the gate text after a summarized capture.
- **Y5:**
  - a no-probe declared target with writer lines run, in a harness whose native.live_bytes passed, gets
    the new verdict;
  - close-rest refuses it over the floor and allows it at the floor;
  - a peak-only stair gives nothing;
  - a target that never ran gives nothing.
- **Y6:** a rank unit test for `benches/x.rs`.
- **Mutations:** each new test must fail under a mutation. Use a unique marker, then restore, then
  `grep -c` the marker = 0.

## Do not
- Do not change what confirms a leak, the trend gate, the capture rule for passing a stack, or
  `failed_beyond_process`.
- Do not use peak-only growth as a trigger for anything but a printed hint.
- Do not make blind captures or Y5 leads produce a leak verdict; only an alloc-test or a passing stack
  does, as today.
- Skills/agents/data/hooks: only sentences that state the old behaviour, plus at most one sentence in
  references/stages.md or the runtime reference saying a capture marked blind is no evidence of no
  growth. docs/VERIFY.md: add rows for Y1-Y7.

## Rules
- No commit, no push, no branch switch.
- `target/` is shared. Free disk only with `cargo clean -p <crate>`. Disk is about 10 GB free; watch
  it.
- Literal rm paths. Kill only your own processes, by pid. Never weaken a test to pass.
- The repository never names the evidence run's project or its APIs; fixtures and test names stay
  generic.
- Do not run any `dla` command against /root/dla-real (it writes the run's ledger).

## Checks
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `dla schema --check`
  (or the repo's schema check), the shim script, and the test targets you touched. Not the whole suite:
  I run it.
- Then the dynamic set once:
  `./target/debug/dla-bench dynamic --dla-bin ./target/debug/dla --parallel 2 --json $sp/u177/dyn.json`,
  after building dla and dla-bench (build dla-bench with `-p dla-bench` alone).
  - Expected: all fixtures pass plus 1 xfail (rust-tree-parent-links), with the new pair passing.
    jvm-pool-warmup is a known flake: if it alone fails, run it alone up to 3 times.
  - Compare confirmed/claim/rule-out/leaks per fixture with round X's runs in `$sp/u172/dyn-*.json`, and
    explain every difference.
  - Delete the dynamic logs afterwards.

## Report
Report as your final message (the Write tool may refuse a report file). Include:
- files changed;
- each item as implemented;
- tests and the mutation that made each fail;
- commands and results;
- the dynamic comparison;
- anything unsure or not done.
