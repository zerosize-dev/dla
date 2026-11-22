# Brief Z: the Python census writes the workload discovery result when the project has test files

Repository: /home/user/dla (Rust workspace, branch `main`, a dirty working tree that holds three
unfinished changes on top of commit 29cb6f6; all of it is to stay). Scratch directory for your
files: /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u80 (create it).

## The defect

The last change made the project's own tests in a loop the first command of an open Python
queue (`dla harness gen --lang python --pytest`). The queue learns that the project has Python
test files from the workload discovery result of the run (`<run>/workload/workloads.json`, or
`<run>/workload/<worker>.json` for a worker): see `python_tests_of` in
`crates/dla-core/src/testloop.rs` and its use in `crates/dla-core/src/queue.rs`.

Nothing writes that result in a normal Python run. `dla workload discover` is a separate command
and the agents that drive the tool do not run it (checked in the transcripts of earlier real
runs). So after `dla probe`, `dla sweep --lang python`, `dla census --lang python` the queue's
command is the driver batch, and the loop is never offered. The feature is inert.

The JVM has the same dependence and solved it: `dla census --lang jvm` writes the discovery
result when the run has none (`discover_for_the_queue` in `crates/dla-cli/src/census_cmd.rs`).

## The change

`dla census --lang python` writes the workload discovery result, exactly as `dla workload discover`
would write it (the same file, the same evidence record, the same language choice as the JVM
branch: `effective_lang(None, worker)`), when ALL of these hold:

1. the census found candidates or skipped duplicates (the same test as the JVM branch);
2. the run has no discovery result yet (an existing file is never rewritten);
3. the discovery lists at least one Python test file, by the rule the queue uses
   (`dla_core::testloop::python_tests(&report).is_some()`).

Condition 3 is the difference from the JVM branch and it is deliberate. A discovery result
turns on a ledger rule (`harness-not-generated`: a finding cannot be closed with a reason that
needs no exercise while entry points were discovered, no harness exists and budget remains). A
Python project with no test file must behave as it does today: no discovery result is written
and that rule does not start to fire. The test
`gate_flow::stages_fail_and_then_pass_through_a_swept_project` (crates/dla-cli/tests/gate_flow.rs)
pins this and must pass UNCHANGED.

The project must be scanned once, not twice. `dla_workload::discover` (crates/dla-workload/src/lib.rs)
scans, builds the report and writes it. Give it a variant that builds the report, asks a
predicate, and writes only when the predicate says yes (for example
`discover_when(ctx, lang, wanted: impl FnOnce(&WorkloadReport) -> bool) -> Result<Option<StageOutcome>, Refusal>`,
with `discover` calling it). Nothing is written, no evidence record is added and no stage
directory is created when the predicate says no.

The census output gets the same kind of note as the JVM branch under the key `workloads`:
"discovered for the measurement queue (N workload(s), evidence E): the queue of a Python project
that has test files starts with the project's own tests; dla workload discover repeats it".
No `workloads` key when nothing was written.

The JVM branch keeps its behaviour exactly (it writes with no test-file condition).

## Tests (each seen failing under a mutation you apply and revert)

Add CLI tests (crates/dla-cli/tests/pytest_loop.rs is the natural place; read its helpers first):

a. A project with a ranked Python candidate and a test file. After `probe`, `sweep --lang python`,
   `census --lang python` and NO `workload discover`: the discovery file exists, the census output
   has the `workloads` note, and `dla run status` names `dla harness gen --lang python --pytest` as
   the command of the Python queue.
b. The same project without any test file: no discovery file, no `workloads` key, the queue's
   command is the driver batch, and a finding can still be closed as it could before (the
   `harness-not-generated` refusal does not appear where it did not appear before your change;
   use the closing call of the gate_flow test as the model).
c. A discovery result that is already there stays byte for byte the same after the census.
d. A project with test files where the census finds no candidate and skips no duplicate: no
   discovery file.

Add a unit test in dla-workload for the new function: predicate false writes nothing (no file,
no evidence record), predicate true gives the same file as `discover`.

The existing test `the_queue_names_the_loop_once_the_workload_discovery_lists_test_files` in
pytest_loop.rs was written for the old behaviour. Read it; keep it if it is still true, and
change it only as far as the new behaviour requires. Say what you changed and why.

Mutations: apply each with a small script that asserts the old text occurs exactly once (the
earlier builder's script is at .../scratchpad/u78/mut.py; write your own in your scratch
directory), run the test, see it fail, revert. Before you finish,
`grep -rIl "MUT_" crates harness skills docs bench agents hooks data` must print nothing.

## Texts

- `skills/detect-leak-anywhere/references/stages.md`: the paragraph that says "`dla census --lang jvm`
  writes the workload discovery result when the run has none, so the test loop needs no step of
  its own. A Python project with test files starts the same way ..." must say that
  `dla census --lang python` writes it too when the project has Python test files.
- `skills/detect-leak-anywhere/references/python.md`: the pytest-loop bullet says "The harness runs
  every test file the workload discovery lists ..."; add that the census writes the discovery
  result when the run has none and the project has test files, so no separate step is needed.
- `docs/VERIFY.md`: the last row ("The Python queue had no first pass over the project's own
  tests ...") describes the queue's first command; add one sentence there that the Python census
  writes the discovery result when the project has test files and the run has none. Do not add
  a new row.
- Do NOT touch `skills/detect-leak-anywhere/SKILL.md` (it is at its size limit).
- `crates/dla-cli/tests/skill_text.rs` pins skill texts: add a pin for the new stages.md sentence
  in the style of the neighbouring pins.

Writing rules for every text and comment: plain technical English, full sentences, no em dashes,
no marketing words. Never name a real third-party project, library or its APIs in the repository
(say "a Python library with C extension modules" when an example is needed). No model names, no
tool vendor names, no attribution lines.

## Rules

- No commit, stash, checkout, reset, restore, or branch change. Leave the tree dirty.
- Use the shared `target/` directory only (plain `cargo` from /home/user/dla, `-j 2`).
- Do not delete directories with `rm -rf` on variable paths; use literal paths.
- Do not kill processes by name.
- Do not run `dla consent`. Do not start any model, agent or verifier.
- If an action is denied, stop and report it; do not ask anyone to do it for you.
- Keep to this brief. If you find another defect, report it; do not fix it.
- Some tests need pytest. System `python3` has none; a Python 3.12 with pytest is at
  /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u72/pillow/.venv/bin
  (put it first on PATH for those test runs). Under that PATH two tests of
  `harness_capture_python` (the gc.freeze ones) fail for a reason that is not yours (that
  interpreter build starts with a non-zero freeze count); ignore them.

## Checks before you report

1. `cargo fmt --all --check` clean.
2. `cargo clippy -j 2 --workspace --all-targets -- -D warnings` clean.
3. `cargo build -j 2 -p dla-cli` and `./target/debug/dla schema --check` clean.
4. These test targets pass with the default PATH and again with the venv first on PATH:
   `cargo test -j 2 -p dla-cli --test pytest_loop --test gate_flow --test queue --test keep_going --test skill_text --test census`
   (drop a name that does not exist) and `cargo test -j 2 -p dla-workload --lib` and
   `cargo test -j 2 -p dla-core --lib queue`.
5. Do NOT run the whole workspace suite; the orchestrator runs it.

## Report (your final message; it is read by the orchestrator, not by a person)

Lead with the result in two sentences. Then: files changed; each test with the mutation it was
seen failing under; any existing test you changed and why; anything in this brief that was wrong;
what you did not verify. Keep command output out of the report except the counts.
