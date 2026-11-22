# Brief X: allocation-site count columns for Python driver runs

Repository: /home/user/dla (Rust workspace: crates dla-core, dla-workload, dla-cli and others;
plugin files under agents/, skills/, harness/, data/). You are a builder. The working tree holds
uncommitted work from an earlier brief (W1, the workload blocker per runtime); build on it, do not
revert it.

Scratchpad (sp): /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad
Your scratch directory: <sp>/u76 (create it). You cannot write a report file with the Write tool:
give the report as your final message (you may also write it with a shell redirect to
<sp>/u76/report.md).

## Hard rules

- No `git commit`, `git stash`, `git checkout`, `git reset`, `git restore` in /home/user/dla.
- Never kill a process by name. No `rm -rf "$VAR/..."`: only literal paths, only under <sp>/u76.
- Use the shared `target/` directory only (`cargo ... -j 2`). Disk is tight (about 5 GB free):
  run `bash <sp>/wip-notes/prune-superseded.sh` before a whole-suite run.
- Nothing in the repository may name a model, an AI assistant, or any third-party skill collection.
  No attribution lines. Plain technical English, short sentences. Do not name the project of the
  evidence run in the repository (say "a Python library with C extension modules").
- A marker mutation is an exact text replacement that carries a unique marker such as
  `/*MUT_X1*/` (`#MUT_X1` in Python); apply it with a script that asserts the old text occurs once
  (see <sp>/u66/mut.py for the pattern), see the test fail, reverse it with the exact reverse
  replacement. At the end `grep -rIl "MUT_" crates harness skills docs bench agents hooks data`
  must print nothing.
- Do not run `dla consent`, and do not start any model, verifier or agent. A scratch run is
  initialised with `DLA_CONSENT_TIERS=1-4 DLA_CONSENT_RAW_VALUES=1` in the environment of
  `dla run init`.
- If something you are told here is wrong, say so in the report; do not widen the task.
- If a permission check denies a command, do not work around it and do not ask the orchestrator to
  run it for you: report it and go on with the rest.

## What was measured (the reason for this brief)

A Python library with C extension modules has a documented leak: every call of one public
function leaks one Python object, because a C function of the extension takes a reference and
never releases it. Nothing holds the objects: no container, no global. tracemalloc sees them at
the Python line that calls into C (`src/PIL/ImageFont.py:580`), one live block more per call. No
static candidate can name this leak.

Driven by hand, the tool reaches it only over a long route (hand measurements and outputs:
<sp>/u74/4.md, read it; code map of the route: <sp>/u74/1.md): the whole-process column
`py.traced` passes, the residual row opens an item, the item asks for a second run with
`--capture alloc,histogram` (16 minutes for a unit that runs the project's tests), then
`dla summarize alloc-stack-diff` lists the line third after two lines of the test runner, with
`"tied":[]`. In a real unattended run the agent stopped at the open item ("inconclusive").

The tool already has the right shape for this in other runtimes: per-site count columns
(`node.listeners@<file:line>`, `node.timers@<site>`, `go.goroutines@<site>`), written by the agent
as `groups` in each record, built into series columns `<base>@<site>` (crates/dla-workload/src/
harness/build.rs), tied only to a finding at exactly that site
(crates/dla-cli/src/trend/candidates.rs `is_per_site_count`), and an untied passing one prints the
command that adds a finding there (crates/dla-cli/src/trend_cmd.rs, the `node.listeners@{site}`
text). A passing growth column that points at a finding also stops the finding from being closed
without an alloc-test (crates/dla-core/src/grown.rs), and `dla alloc-test` binds to a finding's
"site group" column.

## The change

Python driver runs get the same thing for allocations: `py.blocks@<file:line>`, the number of
live traced blocks allocated at one line of the project.

1. Agent (harness/python/sitecustomize.py), driver mode only (records of phase `checkpoint`):
   at each checkpoint, after the collections the record already forces, count the live traced
   blocks per site. The site of a block is the first frame of its traceback, from the innermost
   outward, that lies inside the project and is neither a test file nor a file of the harness
   (the driver, the agent). Use the agent's existing notion of "inside the project" and of
   project-relative paths (see how `_capture_tracemalloc` names frames); for "test file" mirror the
   rule the Rust side uses for workload discovery (crates/dla-workload: `FileInfo.is_test`), and
   say in the report what you mirrored.
   - Depth: tracemalloc runs with 1 frame today unless a capture asks for 10. With 1 frame only
     the innermost frame is known, so an allocation made inside a library function called by
     project code has no project site. Measure the cost on the real tree below (unit time for
     depth 1, 2, 4 and 8 in a driver run without captures) and choose the default depth for driver
     mode: the deepest of those whose unit time is at most 1.25 times the depth-1 time.
     `DLA_TRACEMALLOC_FRAMES` still overrides, and a capture keeps its 10. Report the numbers.
   - Columns: a site gets a column when its count at the last checkpoint is greater than at the
     first checkpoint after warm-up. Keep at most 12 sites, the largest growth first, and write a
     limitation line when more grew ("N more allocation sites grew and have no column"). Sites
     that did not grow get no column. A kept site has a value at EVERY checkpoint of the process,
     the early ones included: the agent learns only at the end which sites grew, so it has to
     keep the per-checkpoint counts in memory and hand the series over when the last checkpoint is
     written (for example a table in the last record that the series builder turns into columns;
     choose the mechanism, keep records bounded, and say what you chose). Do not emit a column
     whose early points are missing or zero-filled.
   - Bound the cost: report the time the snapshot and the grouping take per checkpoint on the
     real tree. If tracemalloc is not tracing, or the record is reduced, write the limitation the
     agent writes for comparable gaps and no columns.
   - Command mode (one process per point) and other runtimes: unchanged.
2. Series (crates/dla-workload/src/harness/build.rs): the columns are named
   `py.blocks@<file:line>`, carry their site (as the group columns do), unit count, region heap.
3. Trend (crates/dla-cli/src/trend_cmd.rs and trend/): `py.blocks@<site>` is a per-site count
   column (the existing rule already says so by its `@`; check it, and check the tie: only a
   finding whose own site is that line). For a passing column that ties to no finding, `next`
   prints the add command, generalised from the listener text:
   `dla ledger add --origin growth-series --evidence <E> --site <site> --region heap --holder <kind>   # no finding there; py.blocks@<site> then ties to it. Holder: read the line: a container of the project that keeps the objects (static/global, container-field, registration-table), or native-handle/global-ref when the line calls into an extension module or a C function and nothing in the program holds the result`
   Keep the listener wording for listener and timer columns. Keep `next` inside its bounds; site
   columns come before whole-process hints.
4. Check, and fix where it does not hold, with a test each:
   - the final gate (`dla gate check --final`, `dla run status`) names a passing
     `py.blocks@<site>` column that ties to no dispositioned finding, as it does for a listener
     site column, and its fix is the add command above;
   - `dla ledger resolve` of a finding at that site is refused while its column passed and no
     alloc-test is bound to it (the `growth-not-measured` rule of grown.rs), and
     `dla ledger close-rest` leaves it;
   - `dla alloc-test <H> --finding <F>` for a finding at that site decides by the
     `py.blocks@<site>` column, not by `py.traced` (the whole process: a unit that runs a test
     runner grows on its own).
   - `dla ledger confirm` accepts the trend's evidence for a finding at that site (the column ties
     by site), given a holder and a refuted non-leak sibling as for any finding.
5. Archetype table (data/archetypes.toml): `L-OWNERSHIP` ("A buffer or object is handed across a
   boundary and neither side takes ownership, so neither frees it") lists only region `off-heap`.
   A managed object kept alive by a reference that native code never released (a CPython
   reference count, a JNI global reference) lies in the managed heap. Add `heap` to its regions.
   Find the tests and generated texts that pin the table (the archetypes block in
   agents/verifier.md is generated from it: regenerate it the way the repository does, do not
   edit it by hand) and keep them green.
6. Texts:
   - skills/detect-leak-anywhere/references/python.md: a short paragraph on the columns (what
     they count, driver runs only, tie by own site), the route for a growing site that no finding
     names (the add command from the trend, then sibling, test, alloc-test, confirm as for any
     finding), and how to choose the holder (as in the comment above); in "Blind spots", correct
     "Reference-count errors in C extensions show only as native growth": a leaked Python object
     shows as a growing `py.blocks@` site at the line that calls into the extension; memory the
     extension takes with malloc does not.
   - One sentence in references/stages.md and in references/evidence-and-report.md where per-site
     count columns are described, so they name the Python ones too.
   - harness/python/README.md: the column and the depth.
   - Update the pins in crates/dla-cli/tests/skill_text.rs where a pinned text changes.
   - docs/VERIFY.md: rows in the style of the neighbouring ones: (a) `corrected`: a Python leak
     without a holder was reachable only through a second capture run; what is verified and what
     is not; (b) the plan deviation of step 5 (L-OWNERSHIP also fits the heap), with the reason.

## Tests

- Real Python driver (crates/dla-workload/tests: find the tests that run a real Python driver
  harness; add next to them): a small project with one function that keeps one object per call
  (append to a module list) at a known line, one function that allocates and frees, and a test
  file whose own code keeps objects. Driver units call both functions. Assert: the series has
  `py.blocks@<file>:<line>` for the keeping line, rising by the expected amount per unit, with a
  value at every point; no column for the line that frees; no column for the test file; the
  12-site bound and its limitation line (a project with more than 12 growing lines).
- An allocation made inside a library function called from a project line is counted at the
  project line when the depth allows it (this is the depth rule's test).
- Trend and gate (crates/dla-cli/tests; the listener-site tests are the model: grep
  `node.listeners@`): the add command for an untied passing column; the tie after `ledger add`;
  the final-gate item; the refusal to resolve; alloc-test deciding by the site column; confirm.
- Each new rule seen failing with a marker mutation, then reversed (list rule, mutation, failing
  test).

## Checks you run

- `cargo fmt --all`, `cargo clippy -j 2 --workspace --all-targets -- -D warnings`,
  `./target/debug/dla schema --check`, and the Python agent's own tests if the repository has
  any for sitecustomize.py (find how CI runs them).
- The whole suite once at the end (prune first):
  `cargo test -j 2 --no-fail-fast --workspace > <sp>/u76/test-all.log 2>&1`; count the
  `^test result:` lines and the failures from the log (about 17 minutes).
- Real tree, no model. The library is at <sp>/u72/pillow (scratch; a git checkout with `.venv`,
  Python 3.12, the library installed editable from this tree, pytest installed; earlier runs are
  under its `.dla/`). Environment:
  `export DLA_PLUGIN_ROOT=/home/user/dla DLA_BIN=/home/user/dla/target/debug/dla PATH=<sp>/u72/pillow/.venv/bin:$PATH`
  and use `/home/user/dla/target/debug/dla` from that directory. Start a NEW run
  (`DLA_CONSENT_TIERS=1-4 DLA_CONSENT_RAW_VALUES=1 dla run init --entry repository --budget auto`,
  answer the intake items with `dla ledger resolve intake.<item> --status answered --evidence E1`
  for target scope symptom runtimes build workload consent budget, then `dla probe`,
  `dla sweep --lang python`, `dla census --lang python`).
  1. `dla harness gen --lang python --driver --next`; fill the hooks so one unit is ONE direct
     call: `ImageDraw.Draw(im).text((5, 5), "hello world", font=font, fill=(255, 0, 0, 255))` with
     `im = Image.new("RGBA", (400, 100))` and `font = ImageFont.truetype("Tests/fonts/FreeMono.ttf", 24)`
     built once in setup. `dla harness run <H> --mode both`, `dla trend <S>`.
     Expected: `py.blocks@src/PIL/ImageFont.py:580` passes with a slope of about 1 per unit and
     `next` prints the add command for it. Show the outputs.
  2. Follow the route by hand and show each command with the head of its output: the add command
     with `--holder native-handle/global-ref` (the line calls `self.font.render(...)`, a function
     of the extension module, and nothing in the program holds the result: this is the documented
     choice of step 6, not a way around a refusal), a non-leak sibling
     (`dla ledger add --sibling-of <F> --origin hypothesis --hypothesis "N-WARM: ..." --refute-if "..." --evidence <E>`),
     `dla ledger test`, `dla alloc-test <H> --finding <F>` (show `decided_by`), the refute of the
     sibling, `dla ledger confirm`, `dla ledger label <F> --suggest`. Stop at the first refusal you
     cannot satisfy with a real measurement and report it.
  3. A second harness whose unit runs two of the project's test files in the same process:
     `pytest.main(["-q", "-x", "-p", "no:cacheprovider", "Tests/test_imagedraw.py", "Tests/test_imagefont.py"])`
     with stdout redirected; generate it with `--step 2` (a unit takes about 3 s). `dla harness run
     <H> --mode fresh`, `dla trend <S>`. Expected: the same column passes (about 145 blocks per
     unit) although the test runner's own lines grow too; show which `py.blocks@` columns exist,
     which passed, and the limitation line if more than 12 sites grew. Lines of the test runner
     are outside the project and must have no column.
  4. The depth measurement of step 1 of "The change" on harness 3's unit.
  If a figure disagrees with what is expected here, report what you see; do not tune anything to
  match.

## Report

Lead with the result. What changed (files), the mechanism you chose for the columns, the depth
numbers, the tests and the mutation each was seen failing with, the real-tree outputs, the
whole-suite counts, what is not verified.
