# Brief H2: a series that never showed the growth cannot speak against it

Follow-up to brief H in the same working tree (your edits and the Go census edits of another
builder are in it; leave `crates/dla-census/` alone). Same rules as brief H (no git state changes,
shared `target/`, `cargo -j 3` one command at a time in the foreground, no installs, no model
starts, mutations by script, no `MUT_` left, plain English, no project names).

## The defect you reported, and its cause

A finding with an owner gets a probe of its holder from `dla harness gen --probe F`. A driver
whose unit never feeds the holder reads that probe flat, `dla alloc-test` says `favors: non-leak`
("the probe columns named after its owner"), and `dla ledger refute` accepts it.

Cause, read in the code: `crates/dla-cli/src/alloc/judge.rs`. `plateau()` judges the EXTRA
window only: a rise there within the noise is `Plateau`, whatever the main window did. `combine()`
turns `Plateau` into `Favors::NonLeak` with the support N-WARM and N-HWM. N-WARM means growth
that stops. A column that never grew in this series did not stop growing: the series does not
reproduce the growth at all, and that says nothing about a growth that another series measured.

The same holds for the path you corrected in brief H: "the line ran N times in this series and
none of its blocks stayed" is a measurement of THIS unit. When a retention record of the run
shows an object of the project keeping the blocks of that line, a unit in which they do not stay
does not answer that record.

## The change

1. `dla alloc-test <H> --finding <F>` (every runtime): a column that decides for a bound finding
   can favor non-leak through a plateau only when it GREW in the main window of this series
   (define it with what is there: the trend gate of that column in this series, or the main
   window's Sen interval above zero for a gauge, a net rise of the counter over the main window;
   say in the report which you used and why). When the deciding columns are flat from the start:
   `favors: inconclusive`, a verdict word of its own in the column record (not `plateau`), the
   text "this series does not show the growth of <F>: <column> is flat from its first measured
   point, so the unit of <H> does not feed it", and `next` names the harness and series of the
   growth evidence: "`dla alloc-test <H of that series> --finding <F>`: the growth was measured
   there". A purge that released memory or the pre-GC twin rule keep their meaning only when
   there was growth to release: check what `combine` does for a flat column with a purge and say
   it; a flat column must not come out non-leak by any route.
   An alloc-test with no `--finding` is not changed.
2. The allocation-site path of brief H: "the line ran N times and none of its blocks stayed"
   stays `non-leak` only when no retention record of the run names a holder of the project that
   keeps that line (or that has the finding's owner). When one does: `inconclusive`, the text
   says which holder keeps the line according to which record, and `next` is the alloc-test on
   the harness of that capture.
3. `dla ledger refute`: nothing to change if 1 and 2 hold (it already needs a discriminator that
   favors non-leak). Add no second mechanism.
4. Texts: one sentence in `skills/detect-leak-anywhere/references/stages.md` where the alloc-test
   is described (a series that never showed the growth is not evidence against it; run the
   alloc-test on the harness that measured the growth), pinned in `skill_text.rs`; one row in
   `docs/VERIFY.md` after your two rows (state `corrected`).

## Tests (each seen failing under a mutation you apply and revert)

- `judge.rs` unit tests: flat main and flat extra is the new inconclusive verdict; growth in
  main and flat extra is still `plateau` and non-leak; growth in both is `growth-continues`; a
  flat column with a "released" purge is not non-leak.
- `holder_first.rs`: your scratch scenario as a test (owner finding, `--probe F`, a unit that
  never feeds the holder: inconclusive with the new text, refute refused, `next` names the first
  harness); the same finding on the harness that grew: leak, and the route to `confirmed` works;
  a real warm-up (a unit that fills a bounded cache during the main window and then stops) on
  its own harness: still non-leak with N-WARM; the line-ran-and-released driver with a retention
  record that names a project holder of the line: inconclusive, and without such a record:
  non-leak as now.
- Run every existing test that asserts a non-leak alloc-test verdict (search the tests of
  dla-cli, dla-core and dla-bench for `non-leak`, `N-WARM`, `plateau`) and list the ones whose
  outcome changes with the reason for each. Do not weaken a test: if an existing test relied on
  a column that never grew counting as a plateau, say so and stop at that test instead of
  editing it (leave it failing and report it).

## Checks before you report

fmt, clippy `--workspace --all-targets -D warnings`, `dla schema --check`, and:
`cargo test -j 3 -p dla-cli --bin dla`, `cargo test -j 3 -p dla-core --lib`,
`cargo test -j 3 -p dla-core --test m6_rules`,
`cargo test -j 3 --no-fail-fast -p dla-cli --test holder_first --test alloc_test --test py_site_columns --test gate_flow --test queue --test skill_text --test grown_chain`,
`cargo test -j 3 -p dla-bench --lib` (or whatever unit tests dla-bench has), the Python agent
tests. Do NOT run `dla-bench dynamic`: the orchestrator runs it.

## Report

Two sentences of result first. Then: files changed; the exact definition of "grew in the main
window"; what a flat column with each purge verdict gives now; each test with its mutation; the
list of existing tests whose outcome changed and why (or "none"); anything in this brief that was
wrong; what you did not verify. Counts only.
