# Builder S4: three review fixes, then one block that grows is growth after a capture

Repository: /home/user/dla, branch main. The working tree holds the uncommitted, verified work of
two builders (S2: a stair reading in crates/dla-stats/src/stair.rs and the trend gate rescue in
classify.rs; S3: Rust held to the capture-first rule in crates/dla-core/src/gate.rs and its
callers). Keep it: build on it, do not revert it. Do NOT commit, push or switch branches. Do not
touch bench/fixtures/, any fixture README or answer key, or skills/detect-leak-anywhere/SKILL.md.
Read each file just before you edit it. Nothing else runs on this machine. The repository must not
name the project of an evidence run or its APIs: say "a Rust library", "a `Vec` field".

## Part A: three bugs a review found in S2 and S3 (do these first)

A1. crates/dla-stats/src/classify.rs: `rescued` does not look at the Spike test that
`classify_fit` runs first, so a gauge staircase whose last reading falls back to its first value
(about 30 points, equal rises, growing gaps, final point back at the start) passes the gate as
growth with `gate_note` set while its shape reads `spike`. A stair's flat end stays at its last
step's level: no rescue when the last reading is below the highest one by more than the noise
tolerance (`tol`) or by more than one scatter of the step reading (pick one, say why).

A2. `step_growth` takes "separate steps" as an x gap of at least 1.5 times the mean point spacing,
while its doc says two spacings. A ramp with one skipped sample (x = 0,1,2,3,5,6,7; y =
0,100,200,300,400,400,400) passes as Stair. Make "separate" mean what a stair is: at least one
point between two consecutive rises that is not itself a rise (count points, not x), and make the
doc say so. The real series of S2's test (crates/dla-stats/tests/shapes.rs, rises at 880, 1320,
2200 with points every 440) must still pass.

A3. crates/dla-cli/src/trend_cmd.rs `capture_entry`: the summarize-first branch falls back to the
first `summarize` command of `captures.json`, which can be `type-histogram-diff` or `non-heap`,
whose records (HistogramDiff, NonHeap) never clear the item (`measured_holders::scan_in` counts
RetentionPath and AllocStack only): the trend would then lead with a command that never releases
it, and drop the capture that would. Lead only with a command whose summary clears the item
(`alloc-stack-diff`, or the heap summary `heap-graph`), else the capture fix as before.

A4. skills/detect-leak-anywhere/references/rust.md (~line 42) and stages.md (~line 135) say the
first `next` entry is the capture while the harness has no summarized capture: once the capture
was taken, it is the summary command. One clause each.

Tests: each fix gets one that fails without it (A1: the falling-back staircase keeps today's
verdict; A2: the ramp with a skipped sample keeps today's verdict and the real series still
passes; A3: a harness whose `captures.json` lists only a histogram command gets the capture fix,
one that lists `alloc-stack-diff` gets that).

## Part B: one block that grows

### What a real run showed

The unattended run on a Rust library built a driver for the library's documented leak, a `Vec`
field that grows while the consumer does not drain (a `Vec<u64>`-like buffer: one heap block whose
capacity doubles). The allocator shim's live bytes went from 128 MB to 1.0 GB in doubling steps.
S3 makes the model take `dla harness run H --capture alloc` and `dla summarize alloc-stack-diff`
on its `out/p<k>-alloc/shim-dump.jsonl` files. Builder S3 ran that chain on a Rust crate with a
`Vec<u64>` static written by `push` (scratch: /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u162/rs
and rs2; reuse them): the push site is in `top` (+402 MB), but `monotone` is 0, `tied` is empty,
`needed` says "capture a point with more work", and the finding gets no lead. A field with one
block per push (`Box`, `String`) passes and ties.

A read-only map (verified at the cited lines) found why, and that it is not Rust's alone:

- crates/dla-cli/src/summarize/rule.rs `gate`/`gate_at` (~49-125) passes a holder or site only
  when its bytes AND its count rise strictly at every step. Every summary uses it: hprof.rs:808
  (JVM heap dumps, dominated count), node_heapsnapshot.rs:872 and 906, histogram.rs:193,
  growth.rs:319 (every growth column), alloc_stack.rs:536 (Go pprof inuse, Python tracemalloc and
  memray, JFR, dhat, heaptrack, valgrind), shim_groups.rs:710 (Rust shim), vgdb.rs:801,
  non_heap.rs:178. A holder that keeps one growing array or buffer (a Rust `Vec<u64>`, a Go `[]int`
  or `bytes.Buffer`, a Java `long[]` or `byte[]` behind a list of primitives, a
  `ByteArrayOutputStream` or `StringBuilder`, a Python `bytearray`) has a constant block count and
  bytes that step up: it never passes, so it is never tied to a finding. No caller turns the Err
  into a refutation, but the leak is lost.
- rule.rs `plateau()` (~158-198, used by hprof.rs:1135 and node_heapsnapshot.rs:1206): `grew`
  needs every count delta > 0, so a constant-count holder is never Leak; one whose last interval is
  flat reads NonLeak (N-WARM, N-HWM), which lifts a leak lead (grown.rs ~1403-1408). A block that
  doubles is flat between doublings, so its last interval can be flat while it still grows.
- The four Rust allocation-stack points (crates/dla-workload/src/harness/run.rs
  `rust_alloc_points`, ALLOC_POINTS = 4) run `manifest.step * k` units for k = 1..4. A capacity
  that doubles can be equal at two of them (step 2^20: capacities 2^20, 2^21, 2^22, 2^22), so its
  bytes do not even rise at every step. shim_groups.rs calls `rule::gate` without the captures'
  x, so the last-over-first test is not a rate. The in-run captures of the other runtimes are
  spread evenly too (crates/dla-workload/src/harness/capture.rs `select_points`).
- After a pass, the chain for Rust is not the one the other runtimes get: the shim summary's
  growth columns are named `stack:<label>` (shim_groups.rs ~941); grown.rs `is_alloc_site_column`,
  `site_column_add`, `finding_is_at_site` know `py.blocks@` columns only; `grown_unmeasured` needs
  a series bound to a harness, and a summary series has none (growth.rs ~288, view.rs ~165); a
  4-point summary series can never pass `dla trend` (8 points). A tied passing GrowthSeries does
  count for a leak label (labels.rs ~262-290), but close-rest decides from the trend targets'
  verdicts only (crates/dla-cli/src/ledger_queue.rs ~285-300, `best_verdict`), so a finding with
  a tied passing allocation-stack summary can still be closed "its code ran and showed no growth".

### The change

B1. **One block, or a fixed set of blocks, that grows passes the dump-series rule.** In rule.rs, a
series whose count is the same positive number at every dump and whose bytes rise strictly at
every step, with the existing last-over-first test, passes. Its `Growth` says it was this kind
(a field or shape name such as `resized`, shown where the summaries show the shape), and the
summaries' texts that say "bytes and count" say what passes now. A count that rises at some steps
and not at others still fails, as today. Change it in rule.rs so every caller gets it; check each
caller's reason and `needed` texts (shim_groups.rs ~1020 "capture a point with more work" is wrong
for a constant count that failed only on a flat pair: say what would show it).

B2. **`plateau()` agrees with B1.** A constant-count series whose bytes rose at every step is
Leak. A constant-count series that rose and then ends flat is Inconclusive, not NonLeak: one block
that is resized is flat between resizes, and the alloc-test (crates/dla-cli/src/alloc/judge.rs,
with the stair reading over a longer window) decides whether it stopped. A series flat from the
first dump, and a series whose count rose and then stopped (a warm-up fill of many objects), keep
today's NonLeak.

B3. **Capture points that a resized block cannot fall between.** A container that grows its
capacity by a factor of at most two reallocates between any two points whose units differ by a
factor of two or more. Make the four Rust allocation-stack points geometric (`step` x 1, 2, 4, 8,
or the nearest that keeps the existing run-time bound of a capture run: check it, see task notes
in crates/dla-workload about capture runs ending within their limit), record their x, and make
shim_groups.rs use `gate_at` with the captures' x when every capture has one. Then look at
`select_points` for the in-run captures of the other runtimes: if spreading them geometrically
over the run's points (each at least twice the units of the one before where the run allows) is
local and keeps the tests of the earlier round on Go in-run heap-profile timing green, do it;
otherwise report what blocks it with numbers.

B4. **A tied passing allocation-stack summary stands for its finding as a passed trend column
does.** Trace what a tied passing heap-graph holder of the JVM gives its finding (the dump leads
of grown.rs; `decided_by_alloc_test`, `route_to_confirm`), and what a tied passing
allocation-stack summary of Go or Python gives. Make a tied passing summary growth column of a
capture of a harness made for the finding (Rust `stack:` columns included) give the finding the
same lead, and refuse its close as "its code ran and showed no growth" (close-rest and
`dla ledger resolve`) while that lead stands, routing it to the alloc-test as the other leads do.
Build the smallest step of the same kind as the existing ones; if what is missing is large, stop
and report exactly what it is.

### Tests (each must fail with its change taken out: check it and report)

- rule.rs: constant count, bytes strictly rising: passes (with the kind); constant count with one
  flat pair: fails; a count that rises then stops: fails; `plateau()` per B2 (the existing tests
  whose expectation changes, `missing_and_flat_holders` and any other: say which and why).
- shim_groups: four shim dumps of one block whose capacity doubles at geometric x pass, tied to the
  finding at the writer line; the same block at x = step x 1..4 with a flat pair does not (say so in
  the test: the spacing is why).
- run.rs: the points of `rust_alloc_points` are geometric.
- An end-to-end test on a Rust crate whose leak is a `Vec<u64>` field pushed by a function
  (crates/dla-cli/tests/rust_driver.rs has the pattern: `capture_alloc_names_the_leaking_function`):
  after `dla harness run --capture alloc` and the printed `dla summarize` command, the site ties to
  the finding, the finding has the lead, and close-rest does not close it as no growth.
- A decoy that must stay a non-leak: a bounded buffer that fills and stops before the second point
  (a `Vec` cleared with capacity kept, as bench fixture rust-batch-buffers does: read its source,
  not its README answer key) does not pass at the summary.
- B4: one test at the level where the lead is decided.

## Checks (after your last edit)

`df -h /` first (about 3 GB free; a full dla-cli test build takes 4-5 GB: if under 6 GB run
`cargo clean -p dla-cli -p dla-core -p dla-workload -p dla-bench` first, the approved way to free
the build directory; never delete build directories by hand). Then `cargo test -p dla-stats`,
`cargo test -p dla-core`, `cargo test -p dla-cli --no-fail-fast` (whole suite),
`cargo test -p dla-workload --no-fail-fast`, `cargo test -p dla-report`,
`cargo test -p dla-bench --lib`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo fmt --all --check`, `./target/debug/dla schema --check`. Then the whole dynamic set from the
repository root: `./target/debug/dla-bench dynamic --dla-bin ./target/debug/dla --parallel 2 --json
<scratch file>` (expected 63 pass and 1 expected failure, `rust-tree-parent-links`; the known flaky
one is `jvm-pool-warmup`: if it fails, run it alone three times and report). Say which fixtures'
summaries now pass a constant-count holder and whether any decoy now gets a lead or a claim.
Delete its logs after. Docs: one row at the end of docs/VERIFY.md (re-read the end first) and the
sentence of rust.md that says a `Vec` field growing as one buffer does not pass (~line 48-51), and
any reference that states the dump-series rule as "bytes and count" (grep `dominated count`,
`count strictly`, `bytes and count` in skills/ and docs/).

## Rules

Shared `target/` only. If a build or the dynamic run fails for lack of space, stop and report it.
No `rm -rf "$VAR/..."`. Never kill processes by name. Never skip or weaken a test. No project
names, attribution lines or model names in the repository. Never use a foreground sleep; wait on
long commands with run_in_background. Scratch:
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u164/.

## Report (final message)

Files changed, split into Part A and Part B; each rule as implemented; what the real `Vec<u64>`
chain reads now at each step (summary, tie, lead, close-rest); B3's cost (time of the capture run
before and after) and what was done with `select_points`; B4 as built or what is missing; which
existing test expectations changed and why; tests with their mutation checks; every command with
pass/fail counts, run after the last edit; the dynamic result; anything you are unsure of.
