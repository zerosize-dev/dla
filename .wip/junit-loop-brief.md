# Brief S: the project's own tests in a loop must work where a JVM cannot be attached to (real runs spring-5, spring-6)

Rules as in brief Q (Sonnet only; no commits; shared `target/`, `-j 2`; small exact edits;
literal paths in `rm`; `<sp>/wip-notes/prune-superseded.sh` when disk is short (about 4 GB
free: delete copies and build outputs as you go); no process kills by name; unique-marker
mutations reversed by exact replacement; no test, fixture or bench policy weakened; never
`dla consent` on a real tree (unattended-runner environment on `dla run init`)). CI is
running on `main`; your edits stay uncommitted. Scratch `<sp>/u63/`, report
`<sp>/u63/report.md`, short final message.
`<sp>` = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad

## What happened

Read `<sp>/wip-notes/real-run-plugin-spring-6.md` first (and `...-spring-5.md`). Evidence:
`<sp>/u36/spring-6/`, `<sp>/u36/spring-5/` (run.json, transcript.jsonl, `dla/`).

- spring-6 ran on the build with your `key:` line. The model reached the documented cache
  again (F160), saw the line three times in the driver file, wrote one method reference and
  read it constant. The line is true and changed nothing. The natural trigger is a Kotlin
  caller, and the project's own Kotlin tests are such callers.
- In spring-5 the model tried the JUnit repeat (`dla harness gen --lang jvm --junit`) and
  every unit failed: `Could not initialize plugin: interface org.mockito.plugins.MockMaker`,
  root cause `Could not self-attach to current VM using external process`. In spring-6 it
  did not try it at all. So on this project every candidate costs a hand-written unit, the
  model says so ("each batch needs a hand-filled driver") and stops after 10 harnesses with
  164 of 180 budget minutes left.

A mocking library that instruments classes can be loaded as an agent at JVM start-up; then
nothing attaches to the running JVM. Newer JDKs warn about dynamic agent loading and
hardened containers refuse it, so this is not only this machine.

## Wanted

1. Reproduce the failure on a copy of `/root/dla-real/src/spring-data-commons`
   (`<sp>/u63/spring`; Maven home warm copy: `/root/dla-real/m2warm`, see how
   `/root/dla-real/run.py --warm-m2` uses it): a JUnit repeat over a small test class that
   uses mocks. Say exactly which process the attach starts and why it fails here.
2. Make the JUnit repeat start its JVM so that the attach is not needed: when an agent jar
   of the mocking stack is on the resolved test class path (find out which jar carries the
   start-up agent entry in the versions you meet: read the manifests, do not assume), add
   it as a start-up agent of the loop's JVM, next to DLA's own agent, and say so in one
   line of the run output and in the harness manifest. No project names in code or text.
   If a start-up agent is not possible for a version, say what is and keep the failure
   message honest (the existing `DLA-JUNIT every unit failed` text names the root cause:
   add the fix the user can apply).
   Check that DLA's own agent and the added one work together (order, retransformation),
   on the bench JVM fixtures that use the JUnit repeat if any do, and on the spring copy.
3. Then measure, on the spring copy: a JUnit repeat with a probe for the finding at
   `TypedPropertyPaths.java:53` over the project's tests that exercise that class (Kotlin
   and Java ones: find them by name and by import; say which you chose and why).
   Report the probe per point, what `dla trend` reads, and how far the printed commands
   carry (sibling, `ledger test`, `alloc-test`, refute, confirm). Stop at the first command
   that is refused or wrong. If the tests do not grow it, say what they do instead.
4. Report, without changing it yet: for a JVM project with a JUnit 5 test workload and
   driver support, what do `dla harness gen --next`, `dla run status`, `dla trend` and the
   queue lines print about the JUnit repeat? In spring-6 the model was never pointed at
   it. Say where the tool could offer the test loop as the first, hookless harness of a
   batch (probes for the batch's targets, tests chosen by the owner classes of the
   targets), what the limits are (8 probes a harness, the time of one run of the chosen
   tests), and how many of the 388 ranked candidates of the spring-6 ledger have a test
   class that names their owner class. This is input for a decision, not a change.
5. Small: the `key:` line is printed in full for every target (three times in one driver
   file in spring-6, about 750 characters each). Print the long explanation once per
   driver file and one short line per target that points at it. Keep the tests.

## Tests and checks

- A test for 2 at the level the JUnit repeat is tested today (the argument list of the
  loop's JVM and the manifest when the class path has such a jar, and when it has none),
  seen failing once by a unique marker, exact reverse.
- fmt; clippy on what you touched; `cargo test -j 2 --no-fail-fast -p dla-core
  -p dla-workload -p dla-cli`; `dla schema --check` if a manifest field is added;
  `skills/detect-leak-anywhere/references/jvm.md` where the JUnit repeat is described;
  `docs/VERIFY.md` row.

Report: 1 to 5 with the outputs, what changed, tests with what was seen failing, what is
unverified, anything wrong you saw outside this brief.
