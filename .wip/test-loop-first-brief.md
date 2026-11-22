# Brief T: on a JVM project with tests, the first pass of the queue is the project's own tests in a loop

Rules as in brief S (Sonnet only; no commits; shared `target/`, `-j 2`; small exact edits;
literal paths in `rm`; `<sp>/wip-notes/prune-superseded.sh` when disk is short (about 4 GB
free); no process kills by name; unique-marker mutations reversed by exact replacement; no
test, fixture or bench policy weakened; never `dla consent` on a real tree). A frozen
snapshot of the tree as you left it after brief S is in CI; your edits stay uncommitted.
Scratch `<sp>/u64/`, report `<sp>/u64/report.md`, short final message.
`<sp>` = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad

## Why

Your brief S result: a JUnit repeat over four test classes measured the documented cache of
the Java data library in about 90 seconds, with nothing to write, and the printed commands
carried it to `confirmed`. Two real runs on that project never got there: the model
hand-wrote drivers, said "each batch needs a hand-filled driver", and stopped after 10 and
30 harnesses with most of the budget left (388 ranked candidates). The tool points at
`--driver --next` everywhere and mentions the test loop in one clipped hint. 79 percent of
the ranked candidates there have a test class named after their owner class.

A queue that asks a model for hundreds of hand-written units will not be worked. The
project's own tests are units somebody already wrote.

## Wanted (JVM only; other runtimes are a later decision)

1. For a JVM runtime with a JUnit 5 test workload the tool can run (the conditions
   `--junit` has today), the queue's first-pass batch is a hookless test-loop harness:
   - `dla harness gen --lang jvm --junit --next` takes the next ranked candidates that
     have tests, chooses the test classes itself from the owner classes of those
     candidates (by test class name, then by mention; say the rule and its bounds: how
     many classes, how you keep one run short), declares them as targets with their
     probes, and prints the tests it chose and why. Nothing to fill.
   - How many targets one hookless harness can declare: the limit of 8 was set for units a
     model writes. Find what really bounds a hookless harness (the 64-column bound of a
     series, the probe walk time, the manifest) and choose the number from that, with a
     one-line reason next to the constant. Every declared target keeps its probe columns.
   - Ranked candidates with no test stay with the driver batch (`--driver --next`), as
     today.
2. Everything that names the next step names the test loop first while test-loop batches
   are left, with the count ("N candidates in M test-loop batches, nothing to write; then
   K candidates need a driver"): `dla run status`, the queue item of `dla gate check` and
   of the stop hook, `dla trend`'s next line, `dla ledger close-rest`, the report's Queue
   line. The `junit_hint` is not cut by the output bound.
3. What a test-loop reading means for the queue. Read the queue rules first and say what
   they do today with a JUnit repeat (is it "a driver" for "settled by two different
   drivers", for a miss, for the second look?). Wanted: `grew` goes the measured-growth
   route as now; a flat reading (`constant`, `executed`) is one reading, and the existing
   second look by a hand-written focus harness applies while budget lasts; `not_loaded`
   and not reached are a miss, and the existing focus rule applies. Change rules only where
   they do not already give this; name each change.
4. When a test-loop harness cannot build, or every unit fails (the existing
   `DLA-JUNIT every unit failed` cases, JUnit 4 or TestNG refused), the run output prints
   the driver batch command for the same targets, and the targets are not counted as
   measured.
5. One run must fit: time the first unit and fit the rounds the way the drivers do, or
   drop test classes, and say what was done. A test class that fails or needs a service
   must not sink the whole harness if the others pass (say what happens today).
6. `dla trend` named F144 first in its `next` for the column of F160 in your brief S run
   ("the first candidate tied to the column"). A probe column of one holder should point
   at the finding of that holder. Find why F144 is tied to it and fix the order or the
   tie, or report why it is right.
7. Do not change `bench/runner` or any fixture. Say which bench fixtures, if any, would
   now get a test-loop batch from the mechanical driver, and stop before changing how the
   bench drives the queue.

## Tests and checks

- Unit and integration tests in the style of the queue tests (`crates/dla-cli/tests/queue.rs`,
  `second_look.rs`, `harness_junit.rs`): the batch command and the counts for a JVM project
  with and without tests; test choice by name and by mention; the target bound; the
  fallback of 4; each seen failing once (unique marker, exact reverse).
- Real tree: a fresh copy of `/root/dla-real/src/spring-data-commons` (`<sp>/u64/spring`),
  unattended-runner environment, probe, sweep, census; then follow ONLY the commands the
  tool prints (gen, run, trend, and whatever trend prints) for the first six batches, with
  the attach disabled as in brief S. Report per batch: targets, tests chosen, seconds,
  verdict counts; in which batch the finding at `TypedPropertyPaths.java:53` lands and
  whether the printed route confirms it; how many batches the test-backed candidates
  need in all and the time that would take against the 3 hour budget.
- fmt; clippy on what you touched; `cargo test -j 2 --no-fail-fast -p dla-core
  -p dla-workload -p dla-cli`; `dla schema --check`; the skill text test; `stages.md`,
  `jvm.md` and `recovery.md` where the queue and the JUnit repeat are described;
  `docs/VERIFY.md` rows.

Report: what the rules were and what changed, tests with what was seen failing, the
real-tree table, what is unverified, anything wrong you saw outside this brief.
