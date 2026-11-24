# Brief H: what a measured holder implies (Python), and an unsound non-leak verdict

Repository: /home/user/dla (Rust workspace, branch `main`, clean working tree at the start,
`target/` holds a debug build of the current tree).
Scratch directory: /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u108
(create it; write nothing else outside the repository).

## What happened (a real unattended run on a Python library the tool had never seen)

The library's API package has a module-level provider object that keeps a list. A method of the
provider creates an object and appends it to that list on one branch (when no real provider is
installed) and returns early on the other branch. That list is a documented leak.

1. Eleven minutes into the run, a driver made for another finding was run with
   `--capture alloc,heap`. `dla summarize heap-graph <file> --finding <that other finding>` wrote
   a retention record whose first holder was `static:<module>.<the provider global>` with five
   growing lines of the provider's file. No finding existed at that holder. The command's
   output was piped through `head`, the model looked only at the finding it had asked about,
   and nothing in the gate, in `dla run status` or in `dla harness gen --next` ever mentioned
   the holder again. The run went on through the static queue.
2. Thirty-eight minutes in, the loop over the project's tests showed 48 growing
   `py.blocks@file:line` columns tied to no finding. The gate asked for one
   `dla ledger add --origin growth-series ... --holder <kind>` per line. The model added 48
   findings in a shell loop with the holder `container-field` for each, one of them at the
   line that creates the leaked object.
3. Ninety-five minutes later the model wrote one driver for many of them and ran
   `dla alloc-test <that driver> --finding <the finding at the leak line>`. In that driver a
   real provider was installed, so the method took the early-return branch. The alloc-test
   wrote a discriminator with `favors: non-leak` and the text "the function of the line ... ran
   in this series and none of its blocks stayed" (crates/dla-cli/src/alloc_cmd.rs, the match
   near line 1183: the growth column is absent from the series and FUNCTION coverage says the
   function ran). `dla ledger refute` accepted it. The documented leak was refuted.

Two defects of the tool, both general:

- A retention record that names a project object holding growing lines has no consequence
  unless the model happens to read it.
- "The function ran" is taken for "the line ran". A line on a branch that the unit does not
  take allocates nothing, so nothing stays, and the tool calls that evidence against a leak.

## The change

### 1. A line's own execution count (Python agent and the alloc-test verdict)

- `harness/python/sitecustomize.py`: for every site the harness declares as a target and for
  the site of the finding an alloc-test is about, the agent counts how often that LINE was
  executed and writes the count at every point (find how targets and their sites reach the
  agent, and how function coverage is recorded today; use the same channel). Use
  `sys.monitoring` LINE events limited to the code objects that hold a target line where the
  interpreter has it (3.12 and newer), and a fallback for older interpreters that a test can
  force. The cost must stay small: no global line tracing. Say in your report what one target
  costs per call of its function.
- `crates/dla-cli/src/alloc_cmd.rs` (the match near line 1183) and whatever feeds it: when the
  finding's growth column is absent from the alloc-test series,
  - the line's count rose over the measured points: `favors: non-leak`, text "the line ran N
    times in this series and none of its blocks stayed";
  - the function ran but the line's count did not rise: `favors: inconclusive`, text "the
    function of <site> ran but the line itself did not (a branch the unit does not take)";
  - no line count in the series (an old agent, another runtime, a point that lost sections):
    `favors: inconclusive`, never non-leak. Other runtimes that use this path: say which do and
    what they have (do NOT change their verdicts in this brief beyond what this rule forces;
    report it).
  The `next` of an inconclusive alloc-test names the harness of the series that showed the
  growth: "`dla alloc-test <H of that series> --finding <F>`: the growth was measured there".

### 2. A holder nobody covers is a gate item, first

A Python retention record (`crates/dla-cli/src/summarize/py_retain.rs`) lists holders. Call a
holder "of the project" when its root is a module global or a class attribute of a module whose
file is a project source file that is not test code, and "of the tests" when the root is in
test code or in the harness itself. Test code: the driver module (`__main__`, the generated
harness module), a file whose path has a directory named `test` or `tests`, a file named
`test_*.py`, `*_test.py` or `conftest.py`, and modules of the test runner. Frames and the native
entry stay as they are. Record the kind in the holder entry (`"of": "project" | "tests"`); the
agent may have to write the root's file for that: find out, and keep old capture files readable.

- `dla gate check --stage classify` and `--final` (crates/dla-core/src/gate.rs): for every
  holder of the project that keeps at least one major site and that no finding covers (no
  finding has that owner and none sits at one of its kept lines with that owner), one missing
  item, sorted BEFORE the per-line "is not flat and is tied to no finding" items. Its fix is
  the literal command: `dla ledger add --origin growth-series --evidence <the retention record>
  --owner <owner> --site <the line the adapter's finding_site picks> --holder static/global
  --region heap`. The per-line items of the lines such a holder keeps are not listed beside it
  (one finding at the holder covers them, as today).
- A growing line whose only major holders are of the tests is not a gate item. The retention
  record lists such lines under the holder as today; `dla trend` and the gate say in one line
  how many lines the test code holds. A line with no retention record at all keeps today's
  item, but its fix becomes the capture, not forty-eight adds: when a series has untied growing
  `py.blocks` lines and its harness has no retention record, the gate lists ONE item for the
  series with the fix `dla harness run <H> --mode <mode of the series> --capture alloc,heap`
  and the two `dla summarize` commands, and the per-line add commands only after a retention
  record of that harness exists (for the lines it could not root).
- `dla summarize heap-graph ... --finding <F>` still cuts the record to F for the bundle, but
  its output starts with an `_attention` key when the capture holds a holder of the project
  that no finding covers: the owner, the count of lines, the add command. (`_attention` is the
  key the output bound never cuts: see crates/dla-cli/src/out.rs.)
- `dla run status` leads with the same item before the queue item, and
  `dla harness gen --next` (any language, any mode) is refused with a new code
  `measured-holder-first` while the gate would list such an item for the run: the fix is the
  add command. Nothing else is refused: a series with untied lines and no capture only puts
  the capture item first in `run status` and in the gate.

### 3. Texts

`skills/detect-leak-anywhere/references/python.md` and `references/stages.md`: what a holder of
the project and of the tests is, that a holder of the project with no finding is the first
thing to act on, that a line the unit does not execute proves nothing. `agents/verifier.md`:
one sentence, a `favors: inconclusive` alloc-test whose text says the line did not run is not
evidence against the leak. `docs/VERIFY.md`: two rows at the end in the style of the last rows
(state `corrected`; no project name: say "a Python library the tool had not been run on").
Do NOT touch `skills/detect-leak-anywhere/SKILL.md`. Pin the new sentences in
`crates/dla-cli/tests/skill_text.rs`.

## Tests (each seen failing under a mutation you apply and revert)

- Agent (`harness/python` tests; run them with the machine's `python3`): a target line on a
  branch counts only when the branch is taken; a target line in a function called 10 times
  counts 10; the forced fallback gives the same counts; a harness with no target writes no
  counts and installs no line events.
- CLI, with real Python runs guarded by `have_python()` (read
  `crates/dla-cli/tests/py_site_columns.rs`, `alloc_test.rs`, `harness_capture_python.rs`,
  `summarize_python.rs` for helpers): a small project with a module-level registry that a
  function appends to on one branch only.
  - A driver that takes the appending branch shows the line growing; a second driver whose
    unit calls the function on the other branch: `dla alloc-test` there is `inconclusive` with
    the new text, `dla ledger refute` with it is refused, and its `next` names the first
    harness. A third driver that executes the line and releases the object: `non-leak` with
    the count in the text.
  - `--capture alloc,heap` on the first driver and `summarize heap-graph`: the gate's first
    item is the holder with the literal add command; `dla harness gen --next` is refused with
    `measured-holder-first`; after the add command (copied from the item) both are gone and
    the line's column is tied; `--finding <another finding>` output starts with `_attention`.
  - A driver that keeps the objects in its own module global: the holder is `of: tests`, the
    line is no gate item, the gate says how many lines the test code holds.
  - A series with untied growing lines and no retention record: one capture item for the
    series, no per-line add items; after the capture the unrooted lines (if any) get them.
- Existing tests that change: say which and why. Do not weaken a test. Do not edit anything
  under `bench/fixtures`. `crates/dla-core/tests/m6_rules.rs` must pass unchanged.

Mutations by a small script that asserts the old text occurs exactly once. Before you finish,
`grep -rIl "MUT_" crates harness skills docs bench agents hooks data` must print nothing.

## Rules

- No `git commit`, `git stash`, `git checkout` (in any form), `git reset`, `git restore` or
  branch change. Leave the tree dirty. Undo your own edits by hand or with your mutation script.
- Shared `target/` only (`cargo` from /home/user/dla, `-j 3`). No `rm -rf` on variable paths. Do
  not kill processes by name. Do not run `dla consent`. Do not start any model, agent or
  verifier. No package installs.
- Run each cargo command on its own, in the foreground, piped through `tail` or `grep`. If an
  action is denied, stop that action and report it; do not ask anyone to do it for you and do
  not try another way to the same end.
- Keep to this brief. If you find another defect, report it; do not fix it.
- Plain technical English, full sentences, no em dashes. Never name a real third-party project,
  library or its APIs in the repository. No model names, no attribution lines.
- If a golden file or a schema file changes, say which and why (`./target/debug/dla schema
  --check` must stay clean; regenerate with `dla schema --out schemas` only if a schema really
  changes).
- Do not run the whole workspace suite. The machine's `python3` is 3.13; use no other
  interpreter. Touch nothing under /root/dla-real and nothing in other scratch directories.

## Checks before you report

1. `cargo fmt --all --check` clean.
2. `cargo clippy -j 3 --workspace --all-targets -- -D warnings` clean.
3. `cargo build -j 3 -p dla-cli` then `./target/debug/dla schema --check` clean.
4. `cargo test -j 3 -p dla-core --lib`, `cargo test -j 3 -p dla-core --test m6_rules`,
   `cargo test -j 3 -p dla-cli --bin dla`, `cargo test -j 3 -p dla-workload --lib`,
   `cargo test -j 3 --no-fail-fast -p dla-cli --test py_site_columns --test alloc_test --test harness_capture_python --test summarize_python --test gate_flow --test queue --test skill_text`
   (if a named test file does not exist, find the one that covers the same and say so), the
   Python agent tests, and any test file you added.

## Report (final message; read by the orchestrator)

Two sentences of result first. Then: files changed; how a target's site reaches the agent and
what a line count costs; the exact rule for `of: project` and `of: tests`; one real gate output
with the holder item (first 30 lines); each test with its mutation; existing tests changed and
why; which other runtimes pass through the changed alloc-test path and what they do now;
anything in this brief that was wrong; what you did not verify. Counts only.
