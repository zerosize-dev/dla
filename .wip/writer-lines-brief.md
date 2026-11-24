# Brief W: "the function ran" is not "the line that adds to the holder ran" (Rust, Go, Node)

Repository: /home/user/dla (Rust workspace, branch `main`). The working tree holds uncommitted
changes that are in CI right now (28 files: Node/TypeScript work and a Rust driver change). Leave
them as they are; where you must edit one of those files, add your own lines only. You are the
only builder in the tree.
Scratch directory: /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u120
(create it; write nothing else outside the repository).

The machine has 4 cores and long measurements run on it beside you. Use `cargo ... -j 2`, run
one cargo command at a time, and build only the test binaries the checks below need.
`target/debug` was emptied: your first build is a full one. Never touch /root/dla-real.

## What happened (two real unattended runs on libraries the tool had never seen)

1. A Rust library. The documented leak is a `Vec` field of a channel sender. Its three writer
   lines (`self.pending_messages.push(value)`) sit under `if let Err(PushError::Full(value)) =
   self.tx.push(value)`: they run only when the ring buffer is full, and that happens only when
   the library is used without the set-up call that starts the thread that drains it. The
   model's driver made that set-up call, so the buffer never filled. `dla trend` answered
   `executed` for the target ("function force_send at ...:46 was executed"), every column was
   flat, the queue counted the target as measured, `dla ledger close-rest` closed the rest, the
   final gate passed, and the run ended COMPLETE after 93 seconds of a budget of 167 minutes.
   The target record of the harness already lists the writer lines
   (`harness/H2/targets.json`: `"writers": ["src/util/spsc.rs:38", ":49", ":55"]`), and the
   coverage export the tool reads has a count for every region; the tool keeps only "which
   functions ran" (`crates/dla-workload/src/harness/cover.rs`, `rust_export_functions`).
2. A Go library. The documented leak is a slice field appended in fourteen methods, each
   shaped like this:
   ```go
   func (m *meter) Int64Counter(name string, ...) (..., error) {
       if del, ok := m.delegate.Load().(metric.Meter); ok {
           return del.Int64Counter(name, ...)
       }
       m.mtx.Lock()
       defer m.mtx.Unlock()
       i := &siCounter{name: name, opts: options}
       m.instruments = append(m.instruments, i)
       return i, nil
   }
   ```
   The append runs only while no delegate is set: again a state the usual set-up removes. A
   driver that sets up the library first runs the function and never the append.

The general defect: a flat series says something about a holder only when the lines that add
to it ran in that series. Today the verdict is taken per function.

## The change

### 1. Counts for writer lines, from the coverage data the harness already collects

For each target of a harness that has writer sites (the `writers` of `targets.json`; they come
from the census annotation), record how often each writer line ran in each coverage run:

- Rust: the `llvm-cov export` document has, per function, `regions` (`[lineStart, colStart,
  lineEnd, colEnd, executionCount, fileId, expandedFileId, kind]`). The count of a line is the
  count of the innermost code region (kind 0) that covers it. Keep the function-level record
  as it is and add the writer-line counts beside it (find where `cov/*.rust.json` is written
  and read: `rust_export_functions`, `rust_function_lines`, the run step in
  `crates/dla-workload/src/harness/run.rs` near `instrument-coverage`).
- Go: the `-coverprofile` files in `cov/` have one line per block
  (`file:startLine.startCol,endLine.endCol numStmts count`). A writer line's count is the count
  of the block that covers it. Check which `-covermode` the harness uses; `set` gives 0 or 1,
  which is enough for "ran at all". Say in your report whether you changed the mode and why.
- Node: the `NODE_V8_COVERAGE` files have function ranges with counts, and block ranges where
  V8 recorded them. The tree's uncommitted work maps built positions to source lines through
  source maps (`ScriptMaps` in `crates/dla-parsers`, used in `cover.rs`): use the same mapping
  so that a writer at a `.ts` line is found. If block ranges are not there for a function, the
  writer line reads "unknown" and nothing changes for that target.
- Python already counts target lines in its agent (`DLA_LINES`, `line_counts` in the dumps,
  `LineRuns` in `crates/dla-core/src/harness.rs`). Do not rebuild that. Only make the target
  verdict of part 2 use it when the counted lines include the target's writer lines; if they do
  not today, say so in the report and leave Python as it is.
- JVM: the agent reports classes and methods only. Leave it; state it as a limit in the notes.

Where a count is missing (no coverage run, a tool missing, a line outside every region), the
answer is "unknown" and the verdict stays what it is today. Never guess.

### 2. The target verdict

`dla trend` (`crates/dla-cli/src/trend/verdicts.rs` and what feeds it) answers per target.
New rule, for a target that has writer sites of an adding kind (insert; not a reset or a
reassignment) and whose function ran:

- every such writer line has a known count of 0 in this series' coverage run: the verdict is
  a new one, `writer_not_executed`, with a detail that names the function that ran and each
  writer line with its count ("function force_send at src/util/spsc.rs:46 ran; the lines that
  add to the holder did not: :38 0, :49 0, :55 0").
- at least one such line ran: today's verdicts (`executed`, `grew`, ...) stay.
- any count unknown and none above 0: today's verdict stays, and the detail says the writer
  lines could not be counted and why.

Optional second step, only if the number of units of the coverage run is recorded or cheap to
record: in fresh-key mode a writer line that ran fewer than one time per ten units gets the
detail "ran K times in N units: the units do not feed the holder" under the same new verdict.
If this needs more than a small change, leave it out and say so.

### 3. The queue and the next harness

- A target with `writer_not_executed` is not measured: the queue (`crates/dla-workload/src/
  harness/queue.rs`, `Index::standing`, and `crates/dla-cli/src/ledger_queue.rs`) treats it as
  it treats a missed target ("missed once, no focus harness": it gets a focus harness; after a
  focus harness that also did not run a writer line it is settled as not reached, with a
  remedy that says the writer lines never ran). Find the existing "missed" path and join it;
  do not build a second mechanism. `dla ledger close-rest` and the gates then hold it back as
  they hold back a missed target.
- The target block of the next driver for such a target (`targets.rs`, `gen.rs`; all three
  runtimes share the block's shape) shows, per writer line that did not run: the line with its
  source text, its count in the last run, and the lines above it that decide whether it runs.
  Take those from the source text with a plain rule that works for formatted code in any
  language: walking up from the writer line to the start of its function, keep (a) each line
  with smaller indentation that opens a condition or loop (`if`, `else`, `match`, `switch`,
  `case`, `while`, `for`, `loop`, `when`, `try`, `catch`, `except`, `select`), and (b) each
  `if` at the same or smaller indentation whose block leaves early (`return`, `break`,
  `continue`, `throw`, `raise`, `panic`) before the writer line. Nearest first, six lines at
  most, each cut at 140 characters, each with its line number. Introduce them with one
  sentence: the unit must put the object in the state in which these lines let the writer
  run, and a state that normal set-up removes (a full buffer, a missing configuration, an
  error path, a second thread) needs a unit or a set-up that does not remove it.
- The same lines go into the remedy `dla trend` prints for the target, and into `_attention`
  of `dla harness gen` when a target of the harness had `writer_not_executed` in an earlier
  harness. `_attention` entries are cut at 160 characters when the output is over its bound
  (`crates/dla-cli/src/out.rs`): keep each entry short and put long text in the driver block.

### 4. Texts

`skills/detect-leak-anywhere/references/stages.md` (or the file that explains target verdicts:
find it), `references/rust.md`, `references/go.md`, `references/node.md`: two or three sentences
each, pinned in `crates/dla-cli/tests/skill_text.rs`. `docs/VERIFY.md`: rows at the end, state
`corrected`, "a Rust library the tool had not been run on" and "a Go library the tool had not
been run on". Do NOT touch `skills/detect-leak-anywhere/SKILL.md`.

## Tests (each seen failing under a mutation you apply and revert)

- Unit tests next to each reader: a region or block that covers the line gives its count; the
  innermost region wins; a line outside every region is unknown.
- CLI tests with real builds, guarded like the existing Rust and Go driver tests (find the
  helpers): a small crate, and a small Go module, with a holder whose writer line sits behind
  a state check (`if self.ready { return; } self.items.push(x)`, and the Go shape above). A
  driver whose set-up makes the check true: the function runs, `dla trend` answers
  `writer_not_executed` with the counts, the queue keeps the target and offers the focus
  harness, the next driver's block shows the writer line and the guard line, `close-rest`
  leaves it. A driver that does not make that set-up: the writer runs, the column grows, the
  verdict is as today.
- Node: one test with a function whose writer sits in a branch, if block ranges are there.
- Existing tests that change: say which and why. Do not weaken a test. Nothing under
  `bench/fixtures` is edited. Run `dla-bench dynamic --only <name>` (one name per call, prefix
  match) for two Rust fixtures, two Go fixtures and one Node fixture, a leak and a non-leak
  where there is a pair, and report each result; if one changes, stop and report it.

## Rules

- No `git commit`, `git stash`, `git checkout` (in any form), `git reset`, `git restore` or
  branch change. Leave the tree dirty.
- Shared `target/` only (`cargo` from /home/user/dla, `-j 2`). No `rm -rf` on variable paths. Do
  not kill processes by name. Do not run `dla consent`. Do not start any model, agent or
  verifier. No package installs.
- Run each cargo command on its own, in the foreground, piped through `tail` or `grep`. If an
  action is denied, stop that action and report it; do not ask anyone to do it for you and do
  not try another way to the same end.
- Keep to this brief. If you find another defect, report it; do not fix it.
- Plain technical English, full sentences, no em dashes. Never name a real third-party project,
  library or its APIs in the repository (the code shapes above are for you; in tests and texts
  use neutral names). No model names, no attribution lines.
- Mutations by a small script that asserts the old text occurs exactly once. Before you finish,
  `grep -rIl "MUT_" crates harness skills docs bench agents hooks data` must print nothing.
- If the schema of a record changes, update `schemas/` and run `./target/debug/dla schema
  --check`.

## Checks before you report

`cargo fmt --all --check`; `cargo clippy -j 2 --workspace --all-targets -- -D warnings`;
`cargo build -j 2 -p dla-cli -p dla-bench` and `./target/debug/dla schema --check`;
`cargo test -j 2 -p dla-workload --lib`, `cargo test -j 2 -p dla-core --lib`,
`cargo test -j 2 -p dla-cli --bin dla`, the Rust, Go and Node driver test files of dla-cli that
your change can touch (list them), `--test queue --test gate_flow --test skill_text
--test holder_first`, and the tests you added.

## Report (final message; read by the orchestrator)

Two sentences of result first. Then per part: files changed, what works, one real output
(first lines); where each count is read; the exact verdict rule; where the queue rule sits;
each test with its mutation; the five fixture results; existing tests changed and why;
anything in this brief that was wrong; what you did not verify. Counts only.
