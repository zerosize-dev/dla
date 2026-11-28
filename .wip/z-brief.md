# Round Z: three general defects a real Rust run showed

Repository /home/user/dla on `main` (clean at 38c6207). Do not commit, push, stash, switch branches or
run `cargo clean`. Build with `export CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_PROFILE_TEST_DEBUG=line-tables-only`
(the disk is small: run only the test targets you touch, never the whole workspace, never another target dir).
Scratch: /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u181 (make it).

Rules: no Claude or model names anywhere; never name a third-party project or its APIs in the repository
(say "a real Rust library", "a bounded ring with an overflow list"); never weaken or skip an existing test;
every new test must fail under a mutation of the fix (apply it with a unique marker, run, restore, `touch`
the file, check the marker count is 0). Plain English in comments and docs, short sentences. Keep every
existing behaviour that this brief does not change.

## What the run showed (facts, already checked by hand)

A real Rust library leaks: `Sender::pending_messages: Vec<T>` gets a push (three lines, all
`self.pending_messages.push(value)`) only when a bounded ring of 10240 slots is full, and with no consumer
nothing drains the ring. The run's drivers made for the finding grew the heap. Then:

1. **Coverage at one step misses writers that start after a threshold.** The Rust coverage process
   (`rust_cover_point` in `crates/dla-workload/src/harness/run.rs`; Go has the same shape in
   `go_cover_point`) runs ONE round of `manifest.step` units (440 here). At 440 units the ring is not full,
   so the writer lines count 0 and `dla trend` gave the target `writer_not_executed` in every series of the
   driver made for it, while its heap grew. Re-running the same instrumented binary by hand with the series'
   full length (`DLA_STEP=440 DLA_ROUNDS=9`, 3960 units) counted the writer lines 4197 and 1399 times.
2. **A capture series gives its harness's targets no verdict.** The allocation-stack series that
   `dla summarize` writes from a harness's captures (`SeriesSource` kind `dump`, `capture_harness()` names
   the harness) gets `targets: []` in its trend, because `trend::verdicts::compute` uses
   `series.source.run_harness()`, which is `None` for kind `dump`. Its passed column, tied to the finding by
   its site (`tie.rule == "site"`, the column's `finding`/`candidates`), never became `grew`. So
   `trigger::Triggers::grew_in_a_harness` is false, no trigger item is raised (`trigger::items`), the report
   says "Trigger: not checked (no harness series that grew is tied to it)", and the run ended COMPLETE after
   5 of 167 minutes with the public-API trigger never checked.
3. **The verifier chose `N-HWM` against the alloc-test in its own bundle.** The bundle held an alloc-test
   (`discriminator`, E32) on the finding: `native.live_bytes` `favors: leak`, plateau verdict
   `growth-continues` (extra window rose 573 KB at 186 B/unit). The verifier wrote "the growth is a doubling
   realloc of one block (61 KB to 3.9 MB), so it is the burst high-water mark ... Each later send drains it
   back into the ring ... I did not see evidence that the collector stops draining". The remover runs only
   when the ring has room, and nothing makes room. The verifier did not follow the condition and did not
   weigh E32.

## Z1. A coverage run that saw no target writer runs again at the series' length (Rust and Go drivers)

In `rust_cover_point` and `go_cover_point`: after the one-step coverage process is read, if the harness has
targets with writer lines (the `writers` that `executed_record_with` counts) and ANY target writer line
counted 0 (or every writer line of some target counted 0 — choose the rule that is clearest and say which in
the doc comment; a line that could not be counted, `count: null`, is not 0), and the main run of this mode
did not time out, run the coverage process once more with the units of the main run of this mode (the same
rounds and step the measured process ran, in one process, warm-up 0, no captures, no sampler), within the
time left of the same timeout. Its counts replace the first ones; `fresh_units` (Rust) is the units of that
process. Record a limitation-style note in the run record that says it plainly, e.g. "the coverage of
p1-fresh was taken again at 3960 units: a writer line of F41 had not run in one step of 440 (a writer that
starts after a threshold, such as a full buffer)". If the second process fails or times out, keep the first
counts and add a limitation naming the log. Do not run the second process when every writer line already
ran, or when there are no writer lines. Check every reader of `fresh_units` and of the per-line `fresh`
counts (grep the workspace) still reads right when the units are the full length (per-unit ratios must
divide by the recorded units, never assume one step).

Tests: a Rust integration test (extend `crates/dla-cli/tests/rust_driver.rs` or the closest existing test
that builds a tiny Rust project with coverage; skip cleanly, like the existing ones, when the llvm tools are
missing) with a tiny crate whose holder is pushed only after a bounded buffer of N slots is full, N larger
than one step's pushes and smaller than the series' total: before the fix the writer line counts 0 and the
target reads `writer_not_executed`; after it the count is > 0, `fresh_units` is the full length and the note
is in the run record. If a Go test of the same shape is cheap with the existing Go test helpers, add it;
otherwise a unit test of the decision function (when to run again, which units) for Go is enough. Unit-test
the decision function itself too.

## Z2. A capture series gives `grew` to the harness targets its passed columns are tied to

In `crates/dla-cli/src/trend/verdicts.rs` `compute`: for a capture series (`is_capture_series`, source kind
`dump`) whose `capture_harness()` has a manifest, each declared target of that harness that a PASSED column
of this series is tied to (the column's `finding`, or its `candidates` when exactly that rule already ties
columns to findings elsewhere — reuse the existing tie semantics, do not invent a looser one) gets the
verdict `grew`, with a detail that names the column and the captures, e.g. "the allocation stack
force_send (spsc.rs:55) grew across the 4 captures of H5 (stair, +184 B/unit)". Targets no passed column is
tied to get NO verdict from a capture series (absent: a capture series cannot say a line ran or a holder
stayed flat); never `constant`, `executed` or `writer_not_executed` from a capture series. Keep the
non-capture path unchanged.

Then check every consumer of trend target verdicts and of `Verdict::Grew` (grep: `Verdict::Grew`,
`grew_in_a_harness`, `verdicts.get(`, queue/second-look/`grown`/`grew_untied`/measured_holders/gate/report
code) and make sure a `grew` that comes from a capture series is read correctly: it must raise the trigger
item (`trigger::items`) for a confirmed or strong leak-side finding while the budget is over its floor, exactly
as a `grew` from a plain series of a driver made for the finding does (the driver is not entry-path, so the
trigger stays Unchecked and the item asks for `dla harness gen --lang <rt> --driver --entry F`), and the
report's trigger reason for such a finding must no longer say "no harness series that grew is tied to it".
Do not let it count as a trigger `Reproduced` unless the capture harness itself is entry-path or the
project's own workload (the existing rule). Do not let it settle or confirm anything that a plain `grew`
would not.

Tests: a dla-core or dla-cli test that builds a run with a harness H (targets F) and a capture series of H
with one passed stack column tied to F by site: the trend has `targets: [{finding: F, verdict: grew}]`; a
second target G of H with no tied column has no verdict; `trigger::items` raises the trigger item for F
when F is confirmed and the budget is over the floor; the report trigger line names the entry-path harness
that is missing, not "no harness series that grew". Mutations: drop the capture branch; give untied targets
a verdict.

## Z3. A non-leak reading that the bundle's alloc-test measured false must face it

(a) `agents/verifier.md`, in the procedure, plain short rules:
- `N-WARM` and `N-HWM` say the level stops. When the bundle holds an alloc-test (`discriminator`) whose
  column of the live heap (`native.live_bytes`, `jvm.heap_used`, `go.heap_live`, `node.heap_used`,
  `py.traced`) says `favors: leak` with the plateau verdict `growth-continues`, the level was measured NOT
  to stop over the extra window. Choose `N-WARM` or `N-HWM` then only if you name the line that bounds or
  empties the holder and say why the extra window did not reach it, and cite the evidence id.
- One block that doubles in size across the captures (a vector or buffer reallocated) is also what a list
  that never shrinks looks like. It is a high-water mark only when the same batch repeats and the level stops.
- A remover that runs only on a condition (the ring has room, a flag is set, a consumer is present) removes
  nothing until something makes the condition true. Find that code (read the project files: follow who
  drains, who consumes, who calls the remover) before you count the remover.
- A non-leak reading that says the memory is given back later (drained by a consumer, falls when the load
  falls: `N-SCALE`; kept on purpose: `N-INTENT`) needs the line that gives it back or bounds it, and that
  line must run in the program the drivers measured. A stated purpose ("commands must not be lost") does
  not bound a list that gains an entry per unit and that nothing consumes: that is a defect.
(Baseline measured on the real run's bundle with the current verifier: 3 runs gave N-HWM, N-INTENT and
L-COLLECT; the run itself gave N-HWM. The N-INTENT run wrote that nothing bounds the list and still chose it.)
Keep the rest of the file; keep the archetype block markers intact (the archetype list between the markers is
generated: if a generator exists, change its source, not only the generated text; check with grep).

(b) `dla ledger verify` (find it in `crates/dla-cli/src/ledger_cmd.rs` or nearby): refuse with a new code
`verify-contradicts-measurement` (add it to `crates/dla-core/src/error.rs` beside the others, and to the
error-code documentation and schema lists wherever codes are listed; grep for an existing code such as
`remedy-misses-series` to find every place) when `--label` is `N-WARM` or `N-HWM` and the ledger holds a
discriminator (alloc-test) record on that finding with a live heap column `favors: leak` and plateau verdict
`growth-continues`, unless the `--mechanism` text names that evidence id (e.g. `E32`). The refusal text says
what was measured (the column, the extra rise and slope), that `N-WARM`/`N-HWM` say the level stops, and the
fix: name the line that bounds or empties the holder and why the extra window did not reach it, citing the
evidence id, or choose another reading. Any other label, or no such record, is unchanged.

Tests: dla-cli integration test(s) for the refusal and its two releases (another label; a mechanism that
cites the id), and that a discriminator that favors non-leak does not refuse. Mutations for each.

## Docs

- `docs/VERIFY.md`: one row per item (Z1, Z2, Z3) in the table that holds the Y rows, same 5-cell format as
  the Y rows (claim | status `corrected` | what changed and the paths that do and do not follow it | the
  check | limits). State the real-run facts above in general words (no project names).
- Skill references where a text the agent reads changes (grep `skills/` for `writer_not_executed`, the
  trigger reason, `N-HWM`): update only what is now wrong.

## Checks to run and report

`cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test -p dla-core --lib`;
`cargo test -p dla-cli --bin dla`; `cargo test -p dla-workload --lib`; the integration targets you touched
and any target whose source references the functions you changed (grep); `cargo run -q -p dla-cli -- schema --check`
if a schema-bearing type or an error code list changed. Report each mutation and the test that caught it.

Final message: files changed, what each change does, tests and mutations, command results, anything not done
or unsure.
