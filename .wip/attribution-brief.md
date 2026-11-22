# Brief V: measure which test class feeds a holder that grew in the test loop

Repository: /home/user/dla (Rust workspace: crates dla-core, dla-workload, dla-cli; plugin files
under agents/, skills/, harness/). You are a builder. The working tree holds uncommitted work from
an earlier brief (U) and from the orchestrator; build on it, do not revert it.

Scratchpad (sp): /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad
Your scratch directory: <sp>/u69 (create it). You cannot write a report file with the Write tool:
give the report as your final message (you may also write it with a shell redirect to
<sp>/u69/report.md).

## Hard rules

- No `git commit`, `git stash`, `git checkout`, `git reset`, `git restore` in /home/user/dla.
- Never kill a process by name. No `rm -rf "$VAR/..."`: only literal paths, only under <sp>/u69.
- Use the shared `target/` directory only (`cargo ... -j 2`). Disk is tight (about 5 GB free):
  run `bash <sp>/wip-notes/prune-superseded.sh` before a whole-suite run.
- Nothing in the repository may name a model, an AI assistant, or any third-party skill collection.
  No attribution lines. Plain technical English, short sentences.
- A marker mutation is an exact text replacement that carries a unique marker such as
  `/*MUT_W1*/`; apply it with a script that asserts the old text occurs once, see the test fail,
  reverse it with the exact reverse replacement. At the end
  `grep -rIl "MUT_" crates harness skills docs bench agents hooks data` must print nothing.
- Do not run `dla consent`, and do not start any model, verifier or agent.
- If something you are told here is wrong, say so in the report; do not widen the task.

## What was measured (the reason for this brief)

A real run on a Java library: the test loop (one harness, 9 test classes of the project, run in a
loop in one JVM) read `grew` for the static cache `org.springframework.data.core.TypedPropertyPaths.resolved`
(finding F160), 13.75 entries per run of the tests. Brief U then named, in the bundle and in the
trigger line, the two test classes the queue had CHOSEN for F160 by name
(`TypedPropertyPathUnitTests`, `KTypedPropertyPathUnitTests`).

Running each of the nine classes alone shows that choice is not the cause:

| class alone | `TypedPropertyPaths.resolved` |
|---|---|
| `KPropertyReferenceUnitTests` (Kotlin, 4 tests; chosen for another finding) | grew, 13.3 entries per run, steady |
| `PropertyReferenceUnitTests` (Java) | constant at 267 (fills once) |
| `TypedPropertyPathUnitTests` (Java) | one step in bytes, then flat |
| `KTypedPropertyPathUnitTests` (Kotlin) | constant at 1 |
| the other five | class not loaded |

So the bundle and the trigger line of brief U point the verifier at the wrong files, and the
verifier (which reads them as callers) concluded "intended cache". The tool has to MEASURE which
test class feeds the holder, and say "not measured" until it has.

A scratch run where you can reproduce this without a model: `/root/dla-real/attr/project` (the
library, a run with the census done; F160 is the cache; harnesses H1..H9 are the nine single-class
runs above). `source /root/dla-real/attr/env.sh` gives the Maven settings; then OVERRIDE
`DLA_BIN` and `DLA_PLUGIN_ROOT` for your build:
`export DLA_PLUGIN_ROOT=/home/user/dla DLA_BIN=/home/user/dla/target/debug/dla`.
Use `target/debug/dla --project /root/dla-real/attr/project ...`. You may add harnesses to that run.

## The change

Attribution rides on `dla alloc-test <H> --finding <F>`, which the printed route of every grown
target already contains and which reruns harness H with more rounds. No new command and no new
step for the caller.

1. Agent (harness/jvm/DlaDump.java): a public static method that returns the number of objects
   reachable from one static field, the same figure the field's probe record reports as
   `reachable`, for a class and field spelled as a probe spells them (`findClass` now maps a nested
   `Outer.Inner`); a negative value when the class is not loaded or has no such field. It must not
   load the class.

2. Test loop (harness/jvm/DlaJUnitHarness.java): when the environment has `DLA_ATTRIBUTE`
   (`<class>#<field>`) and `DLA_ATTRIBUTE_OUT` (a file path), the execution listener (`Tally`)
   reads that figure when a test CLASS starts and when it finishes, and adds the difference to
   that class (a negative reading counts as 0). Keep the sums of the first run of the tests (the
   one-time fill) apart from the sums of the later runs. At the end write JSON to
   `DLA_ATTRIBUTE_OUT`: the probe, the number of later runs, and for EVERY class that ran its
   first-run sum and its later-runs sum (classes the loop dropped are listed as dropped). Without
   the two variables the loop behaves exactly as now. If the tests run in parallel the per-class
   figures overlap: state that in the JSON when the JUnit configuration parameter
   `junit.jupiter.execution.parallel.enabled` is true (the launcher gives access to configuration
   parameters; if you cannot read it cheaply, say so in the report and leave it out).

3. `RunOptions` (crates/dla-workload/src/harness/run.rs) gets an optional attribution target;
   `run_points` sets the two variables for a test-loop harness (`manifest.is_test_loop()`), with
   the output file under the harness's own directory in the run directory.
   `dla alloc-test <H> --finding <F>` (crates/dla-cli/src/alloc_cmd.rs) sets it when H is a test
   loop and F has a `jvm-static` probe in the manifest (class and field from the probe target).
   After the run it reads the file and:
   - stores it at `harness/<H>/attribution-<F>.json` in the run directory (the stable place the
     trigger and the bundle read);
   - adds to its own output and to the result of the discriminator evidence it records:
     `fed_by`: the classes whose later-runs sum is positive, each with `per_run` (later sum divided
     by the later runs), largest first, and `ran_flat`: the names of the classes that ran and added
     nothing in the later runs. Keep the output inside the 2 KB bound (names can be simple names
     there; the file has the full ones).
   A harness that is not a test loop, or a finding without a `jvm-static` probe: nothing changes.

4. Trigger (crates/dla-core/src/trigger.rs): for a finding that grew in a test loop, `varies` is
   - with an attribution file that names feeders:
     `the project's own tests <simple names of the feeders> (<per_run> entries per run)` (at most
     three names, then `and N more`);
   - with an attribution file and no feeder (nothing grew per class): say so
     (`no single test class added entries per run in <H>`);
   - without the file: `the project's own tests in <H>; which test class feeds it is not measured
     (dla alloc-test <H> --finding <F>)`.
   Remove brief U's use of the by-name choice (`testloop::chosen_for`) for the trigger: a class
   chosen for the finding is not a class that feeds it. Keep `the project's workload <id>` for
   harnesses that are not test loops.

5. Bundle (crates/dla-cli/src/bundle_cmd.rs, `test_loop_files` from brief U):
   `test-loop/<H>/tests.json` holds, with an attribution file: `fed_by` (class, file, per_run) and
   `ran_and_added_nothing` (class, file), and a `note` that the figures were measured per test
   class while the tests ran in a loop; without one: `classes_run` (every class of the harness,
   class and file) and a `note` that which of them feeds the holder was not measured. The file of
   a class comes from the harness's own `tests.json` (`classes[].class` -> `file`) when it has one;
   omit `file` otherwise. Keep the index entry and the manifest handling as they are.

6. Texts:
   - `agents/verifier.md`, the `test-loop/` paragraph: say that `tests.json` names the test
     classes that fed the holder with the entries each added per run, and the classes that ran
     and added nothing, when that was measured (else only the classes that ran). Keep the rest of
     the paragraph (the reasoning about repeated rounds and about the key) as it is. Do not touch
     the archetypes block. Update the pins in `crates/dla-cli/tests/skill_text.rs`.
   - one sentence each in `skills/detect-leak-anywhere/references/jvm.md` (test loop) and
     `references/stages.md` (verify step) so they agree with the tool.
   - `docs/VERIFY.md`: one row (`corrected`) for: the trigger and the bundle of a test-loop finding
     named the classes chosen for it, not the classes that feed it; what is verified and what is
     not. Follow the style of the neighbouring rows; no model names.

## Tests

- Real loop (crates/dla-workload/tests/harness_junit.rs, the Gradle fixtures there): a project
  with a static holder and two test classes, one that adds an entry per run (a key that is a new
  object each time) and one that calls the same code with a constant key; run the harness with the
  attribution option; the JSON names the first class with a positive later-runs sum and the second
  with zero. Without the option no file is written.
- Trigger and bundle: unit and binary tests with a hand-written attribution file (extend the
  tests brief U added in `crates/dla-core/src/trigger_tests.rs` and `crates/dla-cli/tests/trigger.rs`):
  feeders named; no feeders; no file ("not measured" text with the command).
- alloc-test: where a test with a real JDK exists for the JVM alloc-test
  (`crates/dla-cli/tests/alloc_test_jvm.rs`), add the test-loop case only if the fixtures make it
  cheap; otherwise test the function that turns the attribution file into `fed_by`/`ran_flat`
  and the stored file, and say what is not covered.
- Each new rule seen failing with a marker mutation, then reversed.

## Checks you run

- `cargo fmt --all`, `cargo clippy -j 2 --workspace --all-targets -- -D warnings`,
  `./target/debug/dla schema --check`.
- The whole suite once at the end (prune first):
  `cargo test -j 2 --no-fail-fast --workspace > <sp>/u69/test-all.log 2>&1`; count the
  `^test result:` lines and the failures from the log.
- Real tree, no model, in `/root/dla-real/attr/project` with the environment above:
  `dla harness gen --lang jvm --junit --probe F160 --tests <the nine classes>` (their names:
  org.springframework.data.querydsl.binding.QuerydslBindingsUnitTests,
  org.springframework.data.querydsl.binding.QuerydslBindingsFactoryUnitTests,
  org.springframework.data.mapping.SimpleTypeHolderUnitTests,
  org.springframework.data.convert.MappingContextTypeInformationMapperUnitTests,
  org.springframework.data.core.PropertyReferenceUnitTests,
  org.springframework.data.core.KPropertyReferenceUnitTests,
  org.springframework.data.core.PropertyPathUnitTests,
  org.springframework.data.core.TypedPropertyPathUnitTests,
  org.springframework.data.core.KTypedPropertyPathUnitTests), `dla harness run <H>`,
  `dla trend <S>`, then the commands its `next` prints for F160 up to and including
  `dla alloc-test <H> --finding F160`. Show: the alloc-test output (`fed_by`, `ran_flat`), the
  attribution file, the run time of the alloc-test with and (from the earlier run of the same
  batch, if you make one) without attribution, `dla bundle F160` (the `test-loop/` file), and the
  trigger line of F160 in `dla report` after the printed refute and confirm commands. Expected:
  `KPropertyReferenceUnitTests` feeds it with about 13 entries per run and the others add nothing.
  If the figures disagree with the table above, report what you see; do not tune anything to match.

## Report

Lead with the result. What changed (files), the tests and the mutation each was seen failing
with, the real-tree outputs, the whole-suite counts, what is not verified.
