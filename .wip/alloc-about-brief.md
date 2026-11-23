# Brief H: an alloc-test decides a finding only on a harness that is about it

Repository: /home/user/dla (Rust workspace, branch `main`, clean working tree at the start).
Scratch directory: /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u97
(create it). `target/` is empty: your first cargo command rebuilds everything (15 to 20 minutes).

## What happened (a real unattended run on a Python library)

The loop over the project's tests (harness H1) measured the documented leak line cleanly: the
column `py.blocks@<file:580>` rose by exactly 299 blocks at each of nine rounds and passed the
gate. The run added finding F572 at that line (and ten more at the other growing lines). The
gate's fix for each said: add an N-WARM sibling, `dla ledger test`, generate a probe driver for
the finding, run it, `dla trend`, then `dla alloc-test <H> --finding <F>` "on the harness just
made".

The agent skipped the probe drivers and ran, for all eleven findings in a shell loop,

    dla alloc-test H5 --finding F572

H5 is a batch driver made earlier for three other findings (its targets are F11, F42, F12). Its
unit never touches the line of F572, and its series has no `py.blocks@` column at all. The
alloc-test answered

    basis: "the primary column of the region of F572"   columns: py.traced plateau, os.rss plateau
    favors: non-leak

and `dla ledger refute F572 --evidence <that evidence>` was accepted. The report ended COMPLETE
with "no leak confirmed", and the leak line appears nowhere in it.

Why the tool let it through: `pick` in `crates/dla-cli/src/alloc/pick.rs` looks for columns of
the new series tied to the finding (its probe column, the columns at its site, its owner, the
trend's candidates); `vanished_site_column` covers a site column that passed in the latest trend
OF THE SAME HARNESS and is missing now; when nothing matches, the fallback is the primary
whole-process column of the finding's region. For a harness that has nothing to do with the
finding that fallback always answers, and for a workload that keeps nothing it answers
"non-leak".

A copy of that run's state (ledger, trends, series, harness manifests; no sources and no harness
work directories) is at
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u96/project/.dla/20261123-124107-7a96
Never write into it; make copies (cut the ledger of a copy to its first 1657 lines to have F572
as an open candidate with no sibling; `dla run reseal --reason "..."` clears the tamper item in a
copy).

## The change

H1. `dla alloc-test <H> --finding <F>` runs only when the harness is about the finding. A
    harness is about a finding (use the root finding; a sibling counts through its parent) when
    at least one of these holds:
    a. the finding is among the targets of the harness manifest (the `targets` list, the keys
       or values of `sibling_of`), or the harness is the entry-path or probe harness of the
       finding (read `HarnessManifest` in `crates/dla-core/src/harness.rs` for how those are
       recorded);
    b. a trend of a series of this harness has a column tied to the finding: the column's
       `finding`, its `candidates`, or, for an allocation-site column, the finding is at the
       column's site (`dla_core::grown::finding_is_at_site`);
    c. the finding cites (in `supports` or `contradicts`) growth or unit-count evidence of a
       series of this harness.
    One function for the predicate, in `dla-core` next to the grown rules, with a doc comment.
    When it does not hold, the command is refused BEFORE the harness is run, with a new code
    (suggested `harness-not-about`; add it in `crates/dla-core/src/error.rs` and to the refusal
    table in `skills/detect-leak-anywhere/references/evidence-and-report.md`). Message: `<H>`
    does not target `<F>` and nothing it measured is tied to it, so an alloc-test there says
    nothing about `<F>`. Fix, built from the state: when a harness exists that is about the
    finding, name it (prefer a hook driver; for a test loop say that each run re-runs the tests,
    minutes); and always the driver of its own:
    `dla harness gen --lang <rt> --driver --probe <F>`, fill the unit so that it calls the
    function of the line, `dla harness run <H'>`, `dla trend <S>`, then
    `dla alloc-test <H'> --finding <F>`.
    Without `--finding` the command works as today.
H2. On a harness that IS about the finding, the whole-process fallback is not used for a
    finding that has an allocation-site column of its own somewhere. Extend the existing
    `vanished_site_column` rule: the site column to look for is one that counts as growth
    (`dla_core::grown::counts_as_growth`) in a trend of ANY harness of the run and is tied to the
    finding (as in H1.b), not only one of this harness's latest trend. When that column is not
    in the new series, the answer follows what the trend of the new series says about the
    finding's code (the target verdicts of `dla trend`, see `Verdict` in
    `crates/dla-core/src/trend.rs`, and how the alloc-test already reads coverage): when the
    function of the line ran (`executed`, `constant`, `grew`), the absence is a measurement:
    the line ran and none of its blocks stayed, and the verdict is non-leak with a basis that
    says exactly that; when it did not run or coverage does not know (`not_executed`,
    `not_loaded`, `unknown`, no verdict), the verdict is inconclusive with the basis "the unit
    of `<H>` does not reach `<site>`", and `next` says to make the unit call the function of the
    line. Read what the code does today for these cases first and change only what differs;
    say in your report what it did before.
H3. `dla ledger refute <F> --evidence <E>` and `dla ledger confirm <F.nN> --evidence <E>` with
    a discriminator: check that a discriminator of a harness that is not about the finding
    cannot be cited (the binding rule that exists may already cover it once H1 stops such
    evidence from being written; say what you found and add the check only if a path remains).
H4. Texts: `skills/detect-leak-anywhere/references/python.md` and `stages.md` where the
    alloc-test route of a finding at a growing line is described: one sentence, the alloc-test
    runs on a harness that targets the finding or measured it, never on another one.
    `agents/worker.md` if it names the alloc-test for a line finding. `docs/VERIFY.md`: one new
    row at the end in the style of the last rows (left column: "An alloc-test on an unrelated
    harness refuted the finding at the leak line"; state `corrected`; no project name). Do NOT
    touch `skills/detect-leak-anywhere/SKILL.md` (size limit). Pins in
    `crates/dla-cli/tests/skill_text.rs`.

## Tests (each seen failing under a mutation you apply and revert)

- `crates/dla-cli/tests/py_site_columns.rs` (read `prepared`, `synthetic`, `five_sites`,
  `leak_at_the_line`): two harnesses in one run: one whose series grows the line and a finding
  at the line; a second driver for another finding whose unit does not touch the line.
  `dla alloc-test <second> --finding <line finding>` is refused with the new code before any
  run (assert no new series was written), and the fix names the first harness and the probe
  command; `dla alloc-test <first> --finding <line finding>` works as today.
- H2: a probe driver for the line finding whose unit does not call the function of the line:
  inconclusive with the basis that names the site; one whose unit calls it and keeps nothing:
  non-leak with the basis that the line ran; one whose unit calls it and keeps the object:
  leak by the site column, as today.
- Unit tests of the predicate: target, sibling target, entry or probe harness, tied column
  (finding, candidate, site), cited evidence, none of them.
- A test that an alloc-test without `--finding` is not refused.
- Existing tests that call `dla alloc-test <H> --finding <F>` on a harness that is not about
  the finding: list each, say what it meant to test, and change the fixture so that the harness
  is about the finding (do not change what the test asserts). The bench tool's dynamic flow
  (`crates/dla-bench`, search `alloc-test`): read how it binds a finding to the harness it
  alloc-tests and say whether the new rule can refuse it; if it can, say exactly when.

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
- If a golden file or a schema file changes, say which and why (`./target/debug/dla schema --check`
  must stay clean). Do not edit anything under `bench/fixtures`.
- Do not run the whole workspace suite.

## Checks before you report

1. `cargo fmt --all --check` clean.
2. `cargo clippy -j 2 --workspace --all-targets -- -D warnings` clean.
3. `cargo build -j 2 -p dla-cli` then `./target/debug/dla schema --check` clean.
4. `cargo test -j 2 -p dla-core --lib`, `cargo test -j 2 -p dla-cli --bin dla`,
   `cargo test -j 2 --no-fail-fast -p dla-cli --test py_site_columns --test alloc_test --test trigger --test gate_flow --test gate_harness --test keep_going --test queue --test ledger_m6 --test skill_text`
   and every other test file of `dla-cli` that calls `alloc-test` (search the tests directory
   and list them), and `cargo test -j 2 -p dla-bench` (its unit tests only, no model).
5. Replay on a COPY of the real state (see above; working directory = the copy's `project`
   directory, your built binary): with the ledger cut to 1657 lines and resealed,
   `dla alloc-test H5 --finding F572` is refused with the new code and writes no series;
   quote the message and the fix. With the frozen binary /root/dla-real/bin/dla the same
   command starts the harness (it then fails for the missing work directory): say what it
   printed.

## Report (final message; read by the orchestrator)

Two sentences of result first. Then: files changed; the predicate as implemented; what H2 did
before and does now; H3 finding; each test with its mutation; existing tests changed and why;
the bench flow answer; the replay outputs; anything in this brief that was wrong; what you did
not verify.
