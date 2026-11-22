# Brief U: what the verifier is given for a test-loop finding, and a wrong `not_loaded` on the JVM

Repository: /home/user/dla (Rust workspace: crates dla-core, dla-workload, dla-cli; plugin files
under agents/, skills/, harness/). You are a builder. Work in the working tree as it is (it holds
uncommitted work that is already checked; build on it, do not revert it).

Scratchpad (sp): /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad
Your scratch directory: <sp>/u67 (create it). Your report: <sp>/u67/report.md.

## Hard rules

- No `git commit`, `git stash`, `git checkout`, `git reset`, `git restore` in /home/user/dla.
- Never kill a process by name. No `rm -rf "$VAR/..."`: only literal paths, only under <sp>/u67.
- Use the shared `target/` directory only (`cargo ... -j 2`). Disk is tight: before a whole-suite run
  execute `bash <sp>/wip-notes/prune-superseded.sh`.
- Nothing in the repository may name a model, an AI assistant, or any third-party skill collection.
  No attribution lines. Plain technical English, short sentences.
- A marker mutation is an exact text replacement that carries a unique marker such as
  `/*MUT_U1*/`; apply it with a script that asserts the old text occurs once, see the test fail,
  reverse it with the exact reverse replacement. At the end
  `grep -rIl "MUT_" crates harness skills docs bench agents hooks data` must print nothing.
- Do not run `dla consent`. A scratch run directory is initialised with
  `DLA_CONSENT_TIERS=1-4 DLA_CONSENT_RAW_VALUES=1` in the environment of `dla run init`.
- If something you are told here is wrong, say so in the report; do not widen the task.

## Evidence to read first

A real run of the plugin on a Java library (kept without its scratch copies):
`<sp>/u36/spring-7/dla` is the run directory (ledger.jsonl, report.md, trend/, series/, bundle/,
harness/H*/{harness.json,tests.json,targets.json,probes.json,driver,logs}),
`<sp>/u36/spring-7/transcript.jsonl` the model's transcript.

What happened: the queue's test loop (harness H3, series S3: the project's own JUnit 5 tests in a
loop) read `grew` for finding F160, the static cache `TypedPropertyPaths.resolved`
(`src/main/java/org/springframework/data/core/TypedPropertyPaths.java:53`), 13.75 entries per run of
the tests. The printed route confirmed it. Then the independent verifier disagreed with the label:
it read the cache as intended retention (`N-INTENT`): "a deliberate metadata cache ... soft
references, so the JVM reclaims entries under memory pressure ... The bundle has no entry-path
directory, so it was judged on holder and mechanism alone. I did not confirm that a caller can drive
unbounded growth." The report therefore says "confirmed 0, confirmed but disputed by the verifier 2".

This cache is a leak the project itself fixed later: its key is the lambda object the caller
passes, compared by identity. A Kotlin property reference makes a new object each time it is
evaluated, so no lookup ever hits and every call adds an entry. `harness/H3/tests.json` shows the two
test classes chosen for F160: `TypedPropertyPathUnitTests` (Java) and `KTypedPropertyPathUnitTests`
(Kotlin). The verifier was told neither: `bundle/F160` holds only two evidence files, and the trigger
line of the report names the workload of the first class of the batch
(`...QuerydslBindingsUnitTests.java`), which has nothing to do with F160.

The second finding the verifier disputed, F436 (`ReturnedType.cache`, keyed by types and a factory
hash; the tests build a new factory per test), is a fair `N-INTENT`. The change below must not push
the verifier to call every cache a leak.

## Part 1: the verifier gets the tests and a rule for caches

1a. `dla bundle <F>` (crates/dla-cli/src/bundle_cmd.rs; see how `entry_path_files` writes
    `entry-path/<H>/`). When a harness that targeted the finding and in whose trend the finding
    read `grew` is a test loop (`HarnessManifest::is_test_loop()`), and `harness/<H>/tests.json`
    names test classes for this finding (`classes[].for[].finding`), write
    `test-loop/<H>/tests.json` into the bundle: the harness id, the classes chosen for THIS finding
    only (`class`, `file`, `tests`, `by`), and a one-sentence `note` saying these are the project's
    own tests that ran in a loop while the holder grew. Add a `test_loop` entry to `index.json` the
    way `entry_path` is added. The files are part of the manifest like every other bundle file. At
    most two harnesses. A finding with no such harness gets no directory (the bundle is unchanged).

1b. `agents/verifier.md` (do not touch the block between `<!-- archetypes:begin -->` and
    `<!-- archetypes:end -->`; find which tests pin this file, e.g. `skill_text`, and keep them
    green). Add, in the file's own style:
    - A paragraph for `test-loop/`: when the bundle has this directory, the holder grew while the
      project's own tests ran in a loop; `tests.json` names the test classes chosen for the
      finding. The tests are callers the project's authors wrote. Read them and the holder's code
      and judge the trigger from them as from an entry-path driver: does a test reach the holder
      the way a program that uses the library does (then a caller can drive the growth, and say
      which call), or does the test build, per test, something a program builds once (a new
      factory, registry, context or mock: then the growth is the test's; say so)?
    - In step 2, a rule for a cache or memo: it is intended retention only when a later call can
      find what an earlier call stored. Check the key: who creates it and how it is compared. When
      the key is an object compared by identity that callers create per call (a lambda or closure,
      a method or property reference, a new wrapper, an object without its own equality), no
      lookup hits and every call adds an entry: that is a defect, not a memo. Weak or soft
      references do not settle it: weakly referenced keys bound a map only when nothing else keeps
      a key alive, and softly referenced entries stay until the heap is nearly full. Say what the
      references bound and what they do not.
    Keep both short. The verifier still decides; the text gives it the question to ask.

1c. The trigger of a finding that grew in a test loop (crates/dla-core/src/trigger.rs, `varies`):
    instead of `the project's workload <id>` (the workload of the batch's first class), say
    `the project's own tests <simple names of the classes chosen for the finding>` from
    `harness/<H>/tests.json`; when the file is missing or names no class for the finding, keep the
    current text. Check where `varies` is printed (report, ledger list, gate) and that the texts
    still read well and stay inside any length bound.

1d. One sentence in the skill reference that describes the verifier step (find it under
    skills/detect-leak-anywhere/), saying the bundle of a test-loop finding carries the tests chosen
    for it. Keep `cargo test -p dla-cli --test skill_text` green.

## Part 2: an instance-field holder on the JVM reads `not_loaded` although its class loaded

The JVM agent (harness/jvm/DlaDump.java, about line 870-900) reads a `jvm-class` probe as "one
record per static field of reference type" of the class. For a class that is loaded and has no such
static field (the holder is an instance field, ledger holder `container-field`) it writes no record
and no limitation; for a class that is not loaded it writes `class not loaded: <class>`, which
becomes a series limitation `probe <class>: class not loaded: <class>`.

`dla trend` (crates/dla-cli/src/trend/verdicts.rs, function `one`, the branch "The harness has a
probe for the finding and the series has no column for it") then says `not_loaded`: "the workload
never loaded that code". In the evidence run that is wrong for F8 (series S1, harness H2) and F60
(S52, S53, harness H33): no limitation says their class was not loaded, their own test class or
driver ran. The queue counts `not_loaded` as a miss and asks for a focus harness with a hand-written
driver, which can never read anything else. The model wrote such drivers and gave up.

Change: for a `ProbeTarget::JvmClass { class }` with no column, when no limitation of the series
says `class not loaded: <class>`, the class loaded and the probe had no static field to read: give
the coverage verdict (`coverage_verdict(inp, f)`: `executed`, `not_executed` or `unknown`) with a
detail that starts by saying so (the probe read no static field of <class>: the holder is not a
static field, so its size is not measured; then the coverage reason). Keep `not_loaded` when the
limitation is there. Check how the agent spells a nested class in that limitation
(`Outer.Inner` as the probe gives it) so the match is exact, and that the list of limitations is
not cut somewhere between the agent and the series (if it is, say so and do not rely on absence).
Tests: unit tests next to the existing verdict tests; if `crates/dla-workload/tests/harness_jvm_coverage.rs`
has a real-agent test for an unloaded class (near "class not loaded: zz.shop.Never"), add the
loaded-class case there too.

Report only (no change): what the driver block says a `jvm-class` probe reads for an
instance-field holder (crates/dla-workload/src/harness/targets.rs: "the static fields of {class}"),
and whether the same wrong reading exists for `JvmStatic` with "no such field".

## Checks you run

- `cargo fmt --all`, `cargo clippy -j 2 --workspace --all-targets -- -D warnings`,
  `./target/debug/dla schema --check`.
- The tests you add, each seen failing with a marker mutation of the rule it guards.
- The whole suite once at the end: `bash <sp>/wip-notes/prune-superseded.sh` then
  `cargo test -j 2 --no-fail-fast --workspace > <sp>/u67/test-all.log 2>&1`; count the
  `^test result:` lines and the failures from the log.
- A replay on the evidence without a model: copy `/root/dla-real/src/spring-data-commons` to
  `<sp>/u67/proj`, copy `<sp>/u36/spring-7/dla` to `<sp>/u67/proj/.dla/20261122-064704-b5c7`, write
  the run id into `<sp>/u67/proj/.dla/current` (look at how the tool reads `current`), and with
  `DLA_PLUGIN_ROOT=/home/user/dla` run `target/debug/dla --project <sp>/u67/proj bundle F160`:
  show the new `test-loop/` file and the index entry; `... trend S1` and `... trend S52`: show the
  verdicts of F8 and F60 before (from `trend/T1.json`, `trend/T54.json` of the evidence) and after;
  `... report`: show the trigger line of F160. Do not start a verifier or any other model. Leave
  `<sp>/u67/proj` in place: the orchestrator uses it next. If a command refuses because the copy
  lacks something (harness scratch, state), say what and stop that step.

## Report

`<sp>/u67/report.md`: what you changed (files), the tests and the mutation that each was seen
failing with, the replay outputs, the whole-suite counts, and what is not verified. Lead with the
result; plain sentences.
