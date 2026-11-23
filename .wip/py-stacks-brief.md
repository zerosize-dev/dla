# Brief A: a finding born from a growing allocation line reaches the verifier with its stack

Repository: /home/user/dla (Rust workspace, branch `main`, a dirty working tree that holds one
finished change; all of it is to stay). Scratch directory:
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u90.

## What happened (a real unattended run on a Python library with C extension modules)

The run found the documented leak as a growing column `py.blocks@<file:line>` (the count of live
traced blocks whose innermost project frame is that line). The worker added a finding at the
line, wrote a probe driver that reproduced the growth through the public API, ran
`dla alloc-test <H> --finding <F>` (basis "the columns at its site", favors leak), refuted the
N-WARM sibling and confirmed the finding. Then:

- `dla bundle <F>` wrote a bundle with `"excerpt": false, "frames": []`. Its evidence was two
  trends and the alloc-test discriminator. By design a column name is cut at `@` in a bundle
  and excerpts come only from the frames of allocation and retention evidence
  (`crates/dla-cli/src/bundle_cmd.rs`, module comment). The run had made 0 captures.
- The independent verifier therefore answered `--site none` ("Bundle has no source excerpt or
  frames"), and the report prints `Verified: DISAGREES (site: finding ... vs verifier none ...)`.
- The finding kept the label UNCLASSIFIED and has no stored explanation, although the real
  mechanism is one call away in the stack: the growing line is a callback whose result is
  handed to a function of an extension module.

Facts of the code (surveyed; check each as you go, line numbers may have moved):

- `dla alloc-test` reruns the harness with `RunOptions::default()` (no capture) and builds its
  `next` list in `next_commands()` (`crates/dla-cli/src/alloc_cmd.rs`, near line 1605;
  `MAX_NEXT = 8`). The only capture hint it has is `deciding_capture()` (`--capture nonheap`,
  JVM only).
- `dla harness run <H> --capture alloc` works for Python (`CaptureSet::supported` in
  `crates/dla-workload/src/harness/capture.rs`): the agent keeps 10 frames and writes
  `capture/<pid>/py-<n>.alloc.json` (4 dumps per process by default), and the run prints the
  `dla summarize alloc-stack-diff <files>` command (capture.rs near line 618).
- `dla summarize alloc-stack-diff` (adapter `py-tracemalloc`,
  `crates/dla-cli/src/summarize/alloc_stack.rs`) writes an `alloc-stack` evidence record (class
  Allocation) with raw `alloc/A<n>.json`. It names a finding in `finding`, `candidates` or
  `ties` when a stack's live bytes grow over at least 3 dumps of one process and the stack's
  first project frame is the finding's site (`stack_matches`, `tie::findings_at_site`).
- `dla bundle` takes only the evidence in `supports` and `contradicts` of the finding, its
  siblings and its parent (`evidence_ids()` in bundle_cmd.rs near line 268). `dla summarize`
  does not add its evidence to `supports`, so a stack record reaches a bundle only when a
  transition cites it.
- `dla ledger label` already counts a `retention-path` or `alloc-stack` record that names the
  finding without being cited (`crates/dla-core/src/labels.rs` near line 275:
  `names_holder` / `names_subject`).

## The change

A1. `dla alloc-test`: when the result favors leak, the deciding column is an allocation-site
    column (`dla_core::grown::is_alloc_site_column`), the runtime of the harness supports the
    `alloc` capture, and no `alloc-stack` evidence names the finding yet (the same test the
    label uses), the `next` list starts with two entries before the ledger commands:
    1. `dla harness run <H> --mode <the mode the alloc-test ran> --capture alloc`
    2. a line that says: run the `dla summarize alloc-stack-diff` command that run prints; it
       ties the stacks at the line to `<F>` (write it as a `#` comment line or in the form the
       other non-command hints of `next` use, so that nothing tries to execute it).
    The ledger commands follow unchanged. If the list would exceed `MAX_NEXT`, keep these two
    and the confirm line, and say in your report what was dropped. Add a top-level key that the
    2 KB output bound never cuts (see `KEPT_WHOLE` in `crates/dla-cli/src/out.rs`; reuse an
    existing kept key if one fits, otherwise add `stacks_hint`) with one sentence: the growth
    is a count of blocks allocated at one line; a verifier and an explanation need the stack
    that leads to it; the two commands.
A2. `dla bundle <F>`: besides the cited evidence, take the `alloc-stack` and `retention-path`
    records that name the finding by the rule the label uses (one shared function; do not
    copy the rule). Newest first, at most 2 such records per kind. They get the same scrub and
    the same frame exemption as cited records of their class.
A3. `dla bundle <F>` is refused with a new code `stacks-not-captured` when all of these hold:
    the finding is leak-side and confirmed or strong; the bundle would hold no evidence of
    class Allocation or Retention; a growth evidence of the finding ties it through an
    allocation-site column; the runtime of the harness behind that evidence supports the
    `alloc` capture; and no harness that grew that column has a capture run recorded
    (`harness/<H>/captures.json`, written by `dla harness run --capture`; check what it holds).
    Message: the bundle of `<F>` would give the verifier no stack and no source excerpt; its
    growth is a count of blocks allocated at one line, and the stacks that lead to the line
    were not captured. Fix: the same two commands as A1, with the harness chosen in this order:
    a hook driver (entry-path, probe or batch) whose series grew the column, else the test loop
    that grew it (say that a capture slows a test loop and that a probe driver is the cheaper
    route: `dla harness gen --lang <rt> --driver --probe <F>`); then `dla bundle <F>` again.
    When a capture run exists and no stack named the finding, the bundle is written as today and
    its `index.json` and the command's summary carry one sentence that says so (an allocation
    capture of `<H>` was summarized and no growing stack ends at the finding's line).
    Add the code where the codes live (`crates/dla-core/src/error.rs`) and to the refusal table
    in `skills/detect-leak-anywhere/references/evidence-and-report.md`.
A4. Texts. `skills/detect-leak-anywhere/references/python.md`: where the `py.blocks@` columns
    and the route of a finding added at a line are described, add the step: after
    `dla alloc-test` favors leak, capture the stacks with the two commands, then label, bundle
    and explain; the frame above the growing line shows who receives the object (for a callback
    or a wrapper object, a function of an extension module: read that module's source for the
    reference it keeps). `agents/worker.md`: one sentence at the place where the alloc-test
    route is named, if there is such a place; otherwise leave it. Do NOT touch
    `skills/detect-leak-anywhere/SKILL.md` (size limit). `docs/VERIFY.md`: one new row at the end
    in the style of the last rows (left column: "A finding at a growing line reached the
    verifier without a stack"; state `corrected`; the facts above without naming the project).
    Pins in `crates/dla-cli/tests/skill_text.rs` for the new sentences.

Not in scope: the label rows of a runtime, the `dla ledger verify` record, the report text, the
agent's capture code. If one of them stops the route below from working, report it; do not
change it.

## Tests (each seen failing under a mutation you apply and revert)

- One end-to-end CLI test on a real Python harness (guard with the `have_python()` helper the
  neighbouring tests use). Start from `the_alloc_test_decides_by_the_site_column_and_the_trend_evidence_confirms`
  in `crates/dla-cli/tests/py_site_columns.rs` (read its fixture: a small project with a line that
  keeps one object per call): after the alloc-test, assert that `next` starts with the capture
  command and the summarize line; run the capture command; run the `dla summarize` command that
  run prints (parse it from the run's output as an agent would); assert that the `alloc-stack`
  evidence names the finding (this checks the path normalization between the stack's first
  project frame and the finding's site: if it does not match, find out why and report it; fix it
  only if the fix is small and inside `summarize`); confirm the finding with the printed confirm
  command; then `dla bundle <F>`: assert `excerpt` is true, the frames list holds the growing
  line, and `excerpt.txt` contains the source line. Then a second alloc-test: `next` no longer
  starts with the capture command.
- `dla bundle` refused with `stacks-not-captured` for the same finding when no capture was made,
  with the fix naming the harness; accepted (with the sentence) when `captures.json` exists and
  no stack names the finding (build that state by hand or with a capture run of a harness that
  does not grow the line; say which).
- A finding whose growth is NOT an allocation-site column (the existing census finding of the
  fixtures with a probe column) bundles as today: pin it.
- Unit tests next to `next_commands()` for the three conditions (favors leak and site column:
  the two entries; favors non-leak: none; a stack record that names the finding: none).
- Existing tests that execute or compare the `next` list of an alloc-test
  (`crates/dla-cli/tests/alloc_test.rs`, helper `command_of` / `run_next`; unit tests in
  alloc_cmd.rs): they must keep passing; adapt a helper only so that it skips a comment line,
  and say so.

Mutations by a small script that asserts the old text occurs exactly once. Before you finish,
`grep -rIl "MUT_" crates harness skills docs bench agents hooks data` must print nothing.

## Rules

- No commit, stash, checkout, reset, restore or branch change. Leave the tree dirty.
- Shared `target/` only (`cargo` from /home/user/dla, `-j 2`). No `rm -rf` on variable paths. Do not
  kill processes by name. Do not run `dla consent`. Do not start any model, agent or verifier.
- Run each cargo command on its own, in the foreground, piped through `tail` or `grep`. If an action
  is denied, stop and report it.
- Keep to this brief. If you find another defect, report it; do not fix it.
- Plain technical English, full sentences, no em dashes. Never name a real third-party project,
  library or its APIs in the repository. No model names, no attribution lines.
- Do not edit anything under `bench/fixtures`. If a golden file or a schema file changes, say which
  and why (`./target/debug/dla schema --check` must stay clean).
- Do not run the whole workspace suite.

## Checks before you report

1. `cargo fmt --all --check` clean.
2. `cargo clippy -j 2 --workspace --all-targets -- -D warnings` clean.
3. `cargo build -j 2 -p dla-cli` then `./target/debug/dla schema --check` clean.
4. `cargo test -j 2 -p dla-cli --bin dla` (the unit tests of the CLI crate; if the crate's unit
   tests live elsewhere, run those), and
   `cargo test -j 2 --no-fail-fast -p dla-cli --test py_site_columns --test alloc_test --test harness_capture --test summarize_python --test skill_text --test gate_flow --test keep_going`,
   and the test file of the bundle command (find it: search `dla bundle` under
   `crates/dla-cli/tests`).

## Report (final message; read by the orchestrator)

Two sentences of result first. Then: files changed; each test with its mutation; existing tests
changed and why; what the end-to-end test showed about the tie (did the stack name the finding
at once); anything in this brief that was wrong; what you did not verify. Counts only.
