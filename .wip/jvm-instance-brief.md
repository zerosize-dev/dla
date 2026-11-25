# Brief K: a JVM probe measures an instance field of the objects that are alive

Repository: /home/user/dla (Rust workspace, branch `main`). The working tree may hold
uncommitted, checked changes of earlier builders. Leave them as they are; where you must edit one
of those files, add your own lines only. You are the only builder in the tree.
Scratch directory: /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u132
(create it; write nothing else outside the repository).

The machine has 4 cores. Use `cargo ... -j 3`, one cargo command at a time. Never touch
/root/dla-real. The container can be restarted without warning: keep each background job short
enough to lose, write its output to a file in your scratch directory, and after a restart look at
the tree and the files before you go on. Do not end your work while one of your background jobs
still runs: wait for it, so that your final message is the final report.

## What is wrong

A JVM probe reads static fields only (`harness/jvm/DlaDump.java`: `probes`, `probeField`,
`probeOne`; `crates/dla-core/src/harness.rs` `ProbeTarget::JvmClass` and `JvmStatic`). For a
census candidate whose holder is an instance field (`container-field`, annotation `lifetime:
object`), `crates/dla-workload/src/harness/probes.rs` `targets_for` gives only a `jvm-class`
probe of the owner class, which reads that class's static fields: none of them is the field. The
probe record then says the class is loaded, `dla trend` reads the target `executed`, and every
column is flat, whatever the field holds.

A real unattended run on a JVM library missed its documented leak this way: a list held in an
instance field of a helper class (one helper per encoder, kept for the life of the logger
context) that a thread-local initializer fills. The run's first pass read it flat and the queue
settled it; another builder has since made thread-keyed targets wait for a second look, but a
second look cannot see growth either while the field is never read.

## The change

1. A new probe target `jvm-instance { class, field }` (`ProbeTarget::JvmInstance`, schema updated,
   `dla schema --check`). At each checkpoint the agent finds the live instances of `class` (a
   subclass counts) that are reachable from roots, and records for `field`: the number of
   instances found, the sum of their sizes (a `Collection` or `Map` size, an array length; for
   another type the reachable count only), and the objects reachable from the field summed over
   the instances (bounded like the static probe's count). The record keeps the same shape as a
   static probe's (`size`, `reachable`) plus `instances`, so `dla trend`, the columns
   (`probe:<id>`, `.size`, `.bytes` where they exist) and the tie rules read it unchanged; check
   that.
2. Roots of the walk: the static fields of the loaded classes of the project (not of the JDK or
   of libraries), the values of the thread-locals of every live thread (the agent already reads
   them: `threadLocalValue`), and the state object of the driver and the instance of the JUnit
   harness where there is one (find how `DlaHarness.java` and `DlaJUnitHarness.java` hold them).
   The walk is breadth-first with a bound on objects and on time (say what you chose and why,
   and measure it on the real project below); when the bound stops it, the record says so in
   its limitations and counts what it found. It never loads a class and never calls project
   code (no `size()` of a project type: read the field values the way the static probe does; for
   a JDK collection `size()` is fine).
3. `targets_for`: a JVM candidate whose holder is an instance field gets a `jvm-instance` probe
   of its owner class and field as its specific probe, ahead of the `jvm-class` one. Decide from
   the census facts you have (`kind`, the annotation's `lifetime`, whether the field is static);
   say which you used. `targets.rs` describes the new probe in the driver block in one line.
4. Texts: `skills/detect-leak-anywhere/references/jvm.md` and `harness/jvm/README.md`, a few
   sentences, pinned in `crates/dla-cli/tests/skill_text.rs`. `docs/VERIFY.md`: one row at the
   end, state `corrected`, "a JVM library the tool had not been run on". Do NOT touch
   `skills/detect-leak-anywhere/SKILL.md`.
5. Report, do not fix: for Python, Node and Go, does a census candidate whose holder is an
   instance attribute or field of an object get a probe that measures it? One line each, with the
   code you read.

## The real case

A copy of the library is at
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u107/logstash-try/project
with a filled Maven home at `u107/logstash-try/m2` (offline; JDK 21 is installed). Copy both to
your scratch directory and work on the copies. Environment for the tool:
`DLA_CONSENT_TIERS=1-4 DLA_CONSENT_RAW_VALUES=1 DLA_PLUGIN_ROOT=/home/user/dla`, and for Maven
`MAVEN_USER_HOME=<your m2 copy>` and `MAVEN_OPTS=-Dmaven.repo.local=<your m2 copy>/repository`
(add `-o` where the tool lets you pass Maven arguments, or find how it runs Maven offline).
The documented holder is the `threadValues` list of the class `ThreadLocalHolder` in
`net.logstash.logback.util` (these names are for you only: never write them into the
repository). `dla run init`, `dla probe`, `dla census --lang jvm`, find its finding, then `dla
harness gen --lang jvm --driver --probe <F>` and fill the hooks so that one unit uses the helper
through its public API from a task of the runtime's shared fork-join pool. Show the probe record
of the new kind and the trend of its column over a run: it should grow. Then with the units on
the caller's thread: it should stay flat. Report the commands and the first lines of each output.
Delete the run directory of your copy when you are done.

## Tests (each seen failing under a mutation you apply and revert)

- A JVM CLI test with a real build, guarded like the existing JVM driver tests: a class
  `Registry` with an instance field `List<String> items`, held by the driver's state; the unit
  adds to it. The `jvm-instance` probe column grows, the target reads as growing (not
  `executed` and flat), and the record counts 1 instance. A second class whose unit removes what
  it adds stays flat. An instance held only through a thread-local of the main thread is found.
- A unit test of `targets_for`: an instance-field container gets the `jvm-instance` probe first; a
  static field still gets `jvm-static`.
- Existing tests that change: say which and why. Do not weaken a test. Nothing under
  `bench/fixtures` is edited. The JVM fixtures use instance and static holders, so at the end run
  the WHOLE dynamic acceptance once: `./target/debug/dla-bench dynamic` (all fixtures, 40 to 60
  minutes; background, output in your scratch directory; nothing heavy beside it) and report the
  whole table and its totals line. One fixture is an expected failure (xfail); `jvm-pool-warmup`
  fails about one run in three for a reason of its own (if it fails, run it alone twice more). A
  fixture that fails for another reason: report its first lines; do not change the tool to make it
  pass without saying why the change is right in general.

## Rules

- No `git commit`, `git stash`, `git checkout` (in any form), `git reset`, `git restore` or
  branch change. Leave the tree dirty.
- Shared `target/` only (`cargo` from /home/user/dla). No `rm -rf` on variable paths. Do not
  kill processes by name. Do not run `dla consent`. Do not start any model, agent or verifier.
  No package installs, no network fetches (the Maven home is offline).
- Run each cargo command on its own, in the foreground, piped through `tail` or `grep` (the
  whole dynamic run is the exception: background). If an action is denied, stop that action and
  report it; do not ask anyone to do it for you and do not try another way to the same end.
- Keep to this brief. If you find another defect, report it; do not fix it.
- Plain technical English, full sentences, no em dashes. Never name a real third-party project,
  library or its APIs in the repository (tests and texts use neutral names). No model names, no
  attribution lines.
- Mutations by a small script that asserts the old text occurs exactly once. Before you finish,
  `grep -rIl "MUT_" crates harness skills docs bench agents hooks data` must print nothing.

## Checks before you report

`cargo fmt --all --check`; `cargo clippy -j 3 --workspace --all-targets -- -D warnings`;
`cargo build -j 3 -p dla-cli -p dla-bench` and `./target/debug/dla schema --check`;
`cargo test -j 3 -p dla-core --lib`, `cargo test -j 3 -p dla-workload --lib`,
`cargo test -j 3 -p dla-cli --bin dla`, the JVM driver, probe and summary test files of dla-cli
(list them), `--test skill_text --test thread_kind --test queue`, and the tests you added.

## Report (final message; read by the orchestrator)

Two sentences of result first. Then per part: files changed, what works, one real output (first
lines); the walk's bounds and what you measured; the real case before and after (the probe
record and the trend of its column on the pool and on one thread); each test with its mutation;
the answers of part 5; the whole table of the dynamic run; existing tests changed and why;
anything in this brief that was wrong; what you did not verify. Counts only.
