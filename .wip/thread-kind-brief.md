# Brief J: a holder that grows per thread gets a driver that varies the threads

Repository: /home/user/dla (Rust workspace, branch `main`). The working tree may hold
uncommitted changes of earlier builders. Leave them as they are; where you must edit one of
those files, add your own lines only. You are the only builder in the tree.
Scratch directory: /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u128
(create it; write nothing else outside the repository).

The machine has 4 cores. Use `cargo ... -j 3`, one cargo command at a time. Never touch
/root/dla-real.

## What happened (a real unattended run on a JVM library the tool had never seen)

The documented leak is a list field of a helper class that keeps one entry per thread. The
class holds a thread-local whose initializer adds an entry to the list; entries are removed
when the thread that owns one dies. On the runtime's common fork-join pool, whose worker
threads clear their thread-locals before each task, the initializer runs again for the same
live thread on every task, so the list grows without bound. On a plain thread, or on the
caller's own thread, it adds one entry per thread and stays flat.

- The census listed both: the thread-local (holder `thread-local`) and the list (holder
  `container-field`, one writer inside the initializer, two removers), same owner class.
- The first pass, the loop over the project's own tests, ran the initializer once per test
  thread. The list's target read `executed`, every column was flat, and the queue settled it.
- Four minutes into the run, `dla ledger close-rest` closed 58 candidates as unresolved and 28
  as static-only, the list among them, before any second look. (A separate change in progress
  makes `close-rest` leave candidates that wait for a second look; do not build that here.)
- The second-look recipe (`crates/dla-core/src/queue.rs`, the text near "a different driver
  (other call orders, objects derived from derived objects, failing and cancelling paths,
  overlapping lifetimes)", and the slot skeleton the driver templates carry for a second look)
  says nothing about threads. A driver that does what it says still calls from one thread.

The general point: a holder keyed by thread (a thread-local, or a container that a
thread-local's initializer fills, or a map keyed by a thread or a thread id) grows with the
number of distinct threads or thread lifetimes the code sees, and its leak appears only with
pooled, recycled or short-lived threads. Every runtime has such holders: `ThreadLocal` on the
JVM, `threading.local` in Python, `thread_local!` in Rust, `AsyncLocalStorage` and worker
threads in Node, per-goroutine state keyed by an id in Go.

## What changed in the tool since this brief was written (it is on `main`)

- `dla trend` writes the residual rows of its series, and while a heap row with at least 64 KiB
  of unattributed growth has no summarized capture, `dla harness gen --next` is refused with
  `capture-first` (a harness for `--probe` or `--entry` is not). A test scene whose heap grows by
  accident keeps under that floor by its own data; one whose growth is the point takes the
  capture and its summaries first (see `capture_if_the_gate_asks` in
  `crates/dla-cli/tests/writer_lines.rs`). Do not change that rule.
- `dla ledger close-rest` already leaves a candidate that waits for its second look.
- The container can be restarted without warning: keep each background job short enough to
  lose, write its output to a file in your scratch directory, and after a restart look at the
  tree and the files before you go on. Do not end your work while one of your background jobs
  still runs; wait for it, so that your final message is the final report.

## The change

1. Thread-keyed targets. A target is thread-keyed when any of these holds, from the census
   facts and annotations the tool already has (find them; add a census fact only where none
   exists, and say where):
   - its holder kind is `thread-local`;
   - its owner class (the owner up to its last segment) also owns a holder of kind
     `thread-local` in the census;
   - one of its adding writer sites lies in a function that is a thread-local initializer
     (JVM `ThreadLocal.withInitial(...)` lambda or method reference target, an overridden
     `initialValue()`; Python a `threading.local` subclass `__init__`; Rust the initializer of
     a `thread_local!`); check what the census parsers can see cheaply and do only those;
   - its key type is a thread or a thread id (`Thread`, `Long` named like a thread id,
     `threading.Thread`, `ThreadId`): only if the census records key types today.
2. The driver block of a thread-keyed target (`crates/dla-workload/src/harness/targets.rs`
   and the per-runtime templates) says, in one short paragraph: this holder is kept per
   thread; units must come from many threads and from pooled threads, not only from the
   caller's thread: per runtime, the concrete forms (JVM: `ForkJoinPool.commonPool()` tasks,
   an `ExecutorService` with a few threads, a new thread per unit that ends; Python: a
   `ThreadPoolExecutor`, a new thread per unit; Rust: `std::thread::spawn` per unit and a
   small pool of threads that live across units; Node: `worker_threads`, and async contexts;
   Go: goroutines per unit). Keep it general; do not name a library.
3. The second look. For a thread-keyed target the second-look skeleton gets one more slot,
   `threads`, whose choices are: the caller's thread, a small fixed pool, the runtime's shared
   pool where it has one, a new thread per unit that ends. `dla harness run` refuses a
   second-look driver for such a target whose unit does not use the slot, exactly as it
   refuses unused slots today (find that check and extend it).
4. The first pass. When a thread-keyed target reads `executed` and flat in a run where all
   units came from one thread (the JVM test loop and every driver today), it is not settled:
   it waits for its second look whatever the queue's other rules say, with the reason "kept per
   thread; the units came from one thread". Say how the tool knows the units came from one
   thread (the driver templates run units on the caller's thread unless the hooks start
   threads; the agents report thread counts: `jvm.threads`, `os.threads`); if it cannot know,
   treat every first-pass run as one thread.
5. Texts: `skills/detect-leak-anywhere/references/stages.md` (second look), `jvm.md`,
   `python.md`, `rust.md`, `node.md`, `go.md`: one or two sentences each, pinned in
   `crates/dla-cli/tests/skill_text.rs`. `docs/VERIFY.md`: one row at the end, state
   `corrected`, "a JVM library the tool had not been run on". Do NOT touch
   `skills/detect-leak-anywhere/SKILL.md`.

## Tests (each seen failing under a mutation you apply and revert)

- Unit tests of the thread-keyed rule: each condition alone makes a target thread-keyed; a
  container in a class without a thread-local is not.
- A JVM CLI test with a real build, guarded like the existing JVM driver tests: a class with a
  `ThreadLocal.withInitial(this::init)` whose `init` adds to a list, and a `use()` that calls
  `threadLocal.get()` then `threadLocal.remove()` (so every call on any thread re-runs the
  initializer and the list grows), and a second class where `use()` does not remove (one entry
  per thread). The first pass with units on one thread: the target is not settled; the driver
  block names the thread forms; a second-look driver without the `threads` slot is refused;
  with units on a fixed pool of four threads the first class grows and the second stays at
  four entries.
- Existing tests that change: say which and why. Do not weaken a test. Nothing under
  `bench/fixtures` is edited. The new "not settled" rule changes the queue, so at the end run
  the WHOLE dynamic acceptance once: `./target/debug/dla-bench dynamic` (all fixtures, 40 to 60
  minutes; background, output in your scratch directory; nothing heavy beside it) and report
  the whole table and its totals line. One fixture is an expected failure (xfail);
  `jvm-pool-warmup` fails about one run in three for a reason of its own (if it fails, run it
  alone twice more). A fixture that fails for another reason: report its first lines; do not
  change the tool to make it pass without saying why the change is right in general.

## Rules

- No `git commit`, `git stash`, `git checkout` (in any form), `git reset`, `git restore` or
  branch change. Leave the tree dirty.
- Shared `target/` only (`cargo` from /home/user/dla). No `rm -rf` on variable paths. Do not
  kill processes by name. Do not run `dla consent`. Do not start any model, agent or verifier.
  No package installs.
- Run each cargo command on its own, in the foreground, piped through `tail` or `grep`. If an
  action is denied, stop that action and report it; do not ask anyone to do it for you and do
  not try another way to the same end.
- Keep to this brief. If you find another defect, report it; do not fix it.
- Plain technical English, full sentences, no em dashes. Never name a real third-party project,
  library or its APIs in the repository. No model names, no attribution lines.
- Mutations by a small script that asserts the old text occurs exactly once. Before you finish,
  `grep -rIl "MUT_" crates harness skills docs bench agents hooks data` must print nothing.
- If the schema of a record changes, update `schemas/` and run `./target/debug/dla schema
  --check`.

## Checks before you report

`cargo fmt --all --check`; `cargo clippy -j 3 --workspace --all-targets -- -D warnings`;
`cargo build -j 3 -p dla-cli -p dla-bench` and `./target/debug/dla schema --check`;
`cargo test -j 3 -p dla-core --lib`, `cargo test -j 3 -p dla-census`,
`cargo test -j 3 -p dla-workload --lib`, `cargo test -j 3 -p dla-cli --bin dla`, the JVM driver
and second-look test files of dla-cli (list them), `--test queue --test gate_flow --test
skill_text --test second_look`, and the tests you added.

## Report (final message; read by the orchestrator)

Two sentences of result first. Then per part: files changed, what works, one real output
(first lines); the exact thread-keyed rule; each test with its mutation; the fixture results;
existing tests changed and why; anything in this brief that was wrong; what you did not
verify. Counts only.
