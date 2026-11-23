# Brief I: a finding whose alloc-test favors leak is confirmed, not closed as inconclusive

Repository: /home/user/dla (Rust workspace, branch `main`, clean working tree at the start).
Scratch directory: /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u99
(create it). `target/` is empty: your first cargo command rebuilds everything (15 to 20 minutes).

## What happened (a real unattended run on a Python library)

Everything up to the last step worked. The loop over the project's tests measured the documented
leak line. The run added finding F572 at the line with an N-WARM sibling F572.n1, made a probe
driver H18 for it (targets `[F572]`), and its series is a textbook leak: the column
`py.blocks@<file:580>` is 440, 880, 1320, ... 3960 at nine checkpoints of 440 units each, in both
key modes. `dla alloc-test H18 --finding F572` wrote discriminator E186: favors leak, decided by
that column.

Then the agent, working from a script of its own and not from the `next` list of the alloc-test:

1. `dla ledger confirm F572 --evidence E186` was refused with `growth-gate-failed` (E186 is the
   discriminator, not growth evidence). The fix said `dla trend S1 then dla ledger confirm F572
   --evidence <trend evidence id>`: a placeholder, and S1 is the test loop, not the driver that
   was just measured.
2. `dla ledger confirm F572 --evidence E258` (a trend evidence) was refused with
   `nonleak-refutation-required`. The fix said `dla alloc-test <harness> --finding F572 then
   dla ledger refute F572.n1 --evidence <discriminator id>`: placeholders again, although the
   alloc-test had been run and its discriminator E186 refutes the sibling.
3. `dla ledger resolve F572 --as unresolved --reason inconclusive --remedy "alloc-test leaned
   leak but ... no refuted N-WARM sibling ..." --evidence E186` was ACCEPTED, with 107 of 180
   budget minutes left. `dla ledger close-rest` closed the sibling. The report ended COMPLETE,
   "no leak confirmed".

Two commands would have confirmed it: `dla ledger refute F572.n1 --evidence E186` and
`dla ledger confirm F572 --evidence <the trend evidence of H18's series>`. The tool knows both
ids: `next_commands` in `crates/dla-cli/src/alloc_cmd.rs` prints exactly these lines after an
alloc-test.

A copy of that run's state (ledger, trends, series, harness manifests; no sources and no harness
work directories) is at
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u98/project/.dla/20261123-170841-90a0
Never write into it; make copies. In the full ledger the transition of F572 to `unresolved` is
line 2000: cut a copy to its first 1999 lines to have F572 `under_test` with E186 written, and
`dla run reseal --reason "..."` in the copy. The copied run's deadline has passed, so a command
that checks the deadline answers `time-budget` there; for those, test with fixtures instead.

## The change

I1. One function (in `dla-core` if the data allows, else in the CLI next to `next_commands`)
    answers, for a leak-side finding: is there a discriminator bound to it (an alloc-test on a
    harness that is about it, `dla_core::grown::harness_about`) that favors leak, newer than
    any discriminator bound to it that favors non-leak, and what are the ledger commands that
    confirm the finding from the present state: the `refute` of each open N-* sibling that this
    discriminator rules out, the `test` if the finding is still a candidate, and the `confirm`
    with the growth evidence the confirm rule will accept (the evidence `next_commands` would
    print: reuse that code, do not write a second chooser). Call the result "the route to
    confirm".
I2. `dla ledger resolve <F> --as unresolved` (every reason, `time_budget` included when budget
    remains by the rule that exists for it) and `dla ledger close-rest` do not close a finding
    that has a route to confirm. New refusal code (suggested `leak-favored`; add it in
    `crates/dla-core/src/error.rs` and to the refusal table in
    `skills/detect-leak-anywhere/references/evidence-and-report.md`). Message: `<F>` cannot be
    closed unresolved: the alloc-test `<E>` on `<H>` favors leak (decided by `<column>`) and
    nothing measured since says otherwise. Fix: the route to confirm, as exact commands joined
    by "then", with real ids. `close-rest` leaves the finding and gives the same commands in
    its `left` entry. The gate and `dla run status` print the same route as the fix of the
    finding (find where the fix of an open or under-test finding is chosen:
    `crates/dla-core/src/view.rs` near the `grown` branch, and `gate.rs`).
    A finding can still be refuted by newer evidence, and a non-leak-favoring newer alloc-test
    lifts the rule.
I3. The two confirm refusals name real ids when they can
    (`crates/dla-core/src/machine.rs`, `GROWTH_GATE_FAILED` near line 1173 and
    `NONLEAK_REFUTATION_REQUIRED` near line 1235; if the core cannot see what is needed, add the
    ids in the CLI where the refusal is turned into output, as other refusals do):
    - `nonleak-refutation-required`: when a discriminator bound to the finding already rules out
      an open sibling, the fix is `dla ledger refute <sibling> --evidence <that discriminator>`
      for each such sibling, then the confirm command with the growth evidence; the generic
      text stays for the case with no such discriminator.
    - `growth-gate-failed` (and whichever refusal a cited discriminator or other non-growth
      evidence gets): say that `<E>` is a discriminator and not growth evidence, and name the
      growth evidence of the finding that passed (prefer the series of the harness of the
      newest leak-favoring alloc-test), with the complete confirm command.
I4. Texts: `skills/detect-leak-anywhere/references/stages.md` and `evidence-and-report.md` where
    the alloc-test and its `next` are described: one sentence, after an alloc-test that favors
    leak run the commands of `next` in order; a finding with a leak-favoring alloc-test is
    confirmed or refuted, never closed unresolved. `agents/worker.md`: the same sentence where
    the alloc-test is named, if it is named. `docs/VERIFY.md`: one new row at the end in the
    style of the last rows (left column: "A finding whose alloc-test favored leak was closed as
    inconclusive"; state `corrected`; no project name). Do NOT touch
    `skills/detect-leak-anywhere/SKILL.md` (size limit). Pins in
    `crates/dla-cli/tests/skill_text.rs`.

## Tests (each seen failing under a mutation you apply and revert)

- `crates/dla-cli/tests/py_site_columns.rs` (read `prepared`, `leak_at_the_line` and the test
  `a_probe_driver_whose_unit_keeps_the_object_is_a_leak_by_the_site_column`): after an alloc-test
  that favors leak and BEFORE any ledger command of `next`:
  `dla ledger resolve <F> --as unresolved --reason inconclusive --remedy x` is refused with the
  new code and the fix holds the exact refute and confirm commands; running exactly the
  commands of the fix, in order, confirms the finding; `dla ledger close-rest` leaves it with
  the same commands; the final gate's fix for the finding is that route; `not_exercised` is
  refused too.
- The two confirm refusals: citing the discriminator gives a fix that names the growth evidence
  and the full command (run it: it then asks for the sibling's refutation with the exact
  command; run that; confirm succeeds).
- A finding whose newest alloc-test favors non-leak, or is inconclusive, can be closed as today
  (pin both).
- A leak-favoring alloc-test followed by a newer non-leak one: the rule does not apply.
- Unit tests of the function of I1 where the code allows.
- Existing tests that close a finding unresolved after a leak-favoring alloc-test: list each,
  say what it meant to test, and change the fixture, not the assertion, unless the assertion
  was the behaviour this brief removes (then say so). The bench tool's dynamic flow
  (`bench/runner/src/dynamic_cmd.rs` and `ops.rs`): read whether it ever resolves a finding
  unresolved after a leak-favoring alloc-test bound to it ("else unresolved" in its steps 3 and
  4), say exactly where, and run the five fixtures below.

Mutations by a small script that asserts the old text occurs exactly once. Before you finish,
`grep -rIl "MUT_" crates harness skills docs bench agents hooks data` must print nothing.

## Rules

- No commit, stash, checkout (in any form, `-p` included), reset, restore or branch change.
  Leave the tree dirty.
- Shared `target/` only (`cargo` from /home/user/dla, `-j 3`). No `rm -rf` on variable paths. Do not
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
2. `cargo clippy -j 3 --workspace --all-targets -- -D warnings` clean.
3. `cargo build -j 3 -p dla-cli -p dla-bench` then `./target/debug/dla schema --check` clean.
4. `cargo test -j 3 -p dla-core --lib`, `cargo test -j 3 -p dla-cli --bin dla`,
   `cargo test -j 3 --no-fail-fast -p dla-cli --test py_site_columns --test alloc_test --test alloc_test_jvm --test trigger --test gate_flow --test gate_harness --test keep_going --test queue --test ledger_m6 --test reasons --test time_budget --test skill_text`,
   and `cargo test -j 3 -p dla-bench`.
5. The bench's dynamic flow on five fixtures (links in a scratch fixtures directory, as
   `/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u97/fx5` does:
   go-buffer-pool, go-gc-tuned-batch, go-header-index, go-subscription-workers,
   go-subscription-workers-closed) plus two Python and two JVM fixtures of your choice from
   `bench/fixtures` (one leak and one non-leak each):
   `./target/debug/dla-bench dynamic --fixtures <dir> --dla-bin ./target/debug/dla --parallel 3 --json <scratch>/dyn9.json`.
   All must pass; quote the totals line.
6. Replay on a COPY of the real state cut to 1999 lines and resealed, working directory the
   copy's `project` directory: `dla gate check --final` (or `dla run status`) prints for F572 the
   route to confirm with real ids; quote it. Commands that check the deadline answer
   `time-budget` there; say which did.

## Report (final message; read by the orchestrator)

Two sentences of result first. Then: files changed; the function of I1 as implemented; each
test with its mutation; existing tests changed and why; the bench answer and the totals line;
the replay output; anything in this brief that was wrong; what you did not verify.
