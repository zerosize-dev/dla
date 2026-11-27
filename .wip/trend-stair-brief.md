# Builder S2: growth in doubling steps passes the trend gate when its flat end is too short to say it stopped

Repository: /home/user/dla, branch main, clean tree. Work in the working tree. Do NOT commit, push
or switch branches. Do not touch harness/, bench/fixtures/, any fixture README or answer key, or
skills/detect-leak-anywhere/SKILL.md. Read each file just before you edit it. Nothing else runs on
this machine. The repository must not name the project of an evidence run or its APIs: say "a Rust
library" and "a `Vec` field", nothing more.

## What a real run showed (the second time the same leak was lost to the same shape)

The previous round fixed `dla alloc-test`: crates/dla-cli/src/alloc/judge.rs `plateau()` now reads
the steps of a gauge (`stair()`, `rise_xs`) whatever the fitted rise, and "stopped" waits for the
expected next gap (2 x (last gap + one point spacing) when the gaps grow, else the longest gap).
Read that code and its tests first: this brief asks for the same reading one stage earlier.

The next unattended run on the Rust library built a driver (H4) that ran the leak's writer, and the
allocator shim's `native.live_bytes` went from 128 MB to 1.009 GB (warm-up removed, x in units):

    440 128537961 | 880 254367081 | 1320 506025321 | 1760 506025321 | 2200 1009341801
    2640 1009341801 | 3080 1009341801 | 3520 1009341801 | 3960 1009341977

The level doubles at each step (a `Vec` doubling its capacity); the steps come further apart; after
the last one the column is flat for 1760 units, which is less than the 2 x (880 + 440) = 2640 the
next step is expected after. `os.rss` grew steadily the whole time (110 MB to 935 MB: the pages of
the reserved capacity are touched as elements are written). `dla trend` failed the column:
`gate_failed: ["sen lower bound <= 0", "decaying half-split"]`, shape `plateau`
(crates/dla-stats/src/classify.rs: `classify_fit` returns Plateau before its Stair test, and
`gauge_gate` is a pure slope gate). With no heap column passed, the finding at the field was closed
"its code ran and showed no growth"; whole-process columns tie to no finding. The evidence is at
/root/dla-real/r7-fastrace-keep/ (series/S5.json, S6.json, S7.json; trend/T5.json); read it, do not
copy it into the repository.

A doubling container always ends a finite window in a flat stretch about as long as all the growth
before it, so a slope gate over that window can always read its second half as flat. That is true
for a Rust `Vec` or `HashMap`, a Go slice or map, a Java `ArrayList` or `HashMap`, a Python list or
dict: every runtime's byte columns.

## The change

1. **One stair reading, shared.** Move the step reading (`rise_xs`, the step threshold of
   `STEP_SCATTERS` scatters around the Theil-Sen trend, the gaps, the growing-gap expectation and
   its `points_needed`) out of crates/dla-cli/src/alloc/judge.rs into crates/dla-stats (a small
   public module or functions in classify.rs), and make judge.rs call it. Behavior of `dla
   alloc-test` must not change (its tests are the check).

2. **The trend gate of a gauge reads the steps too.** In crates/dla-stats/src/classify.rs, a gauge
   series that grew in at least two separate steps (the shared reading; a ramp or a level is not
   one), whose steps do not shrink (each step's rise at least the one before, within one scatter:
   a doubling's steps double), whose total rise clears the existing noise tolerance (`tol`), and
   whose flat stretch after the last step is no longer than the expected next gap, is shape
   `Stair` and passes the gauge gate, whatever the Sen lower bound and the half split say. A
   series whose flat end is longer than the expected gap keeps today's verdict (a bounded
   container that filled and stopped). Put the reason in the column record so the trend output can
   say it (e.g. a field or a `gate_note`: "grew in steps that come further apart; its flat end is
   shorter than the next step is expected to be"), without breaking the record's schema for old
   readers (`#[serde(default)]` where it is read back; check schemas/ and the schema test).
   Counters keep their every-delta gate.

3. **What follows the pass.** A passed heap column ties to the finding as any passed column does
   (crates/dla-core: grown.rs `grown_unmeasured`, the trend `targets`, gate.rs ~709 which already
   accepts `Stair`). Check that the finding then gets the growth lead and the alloc-test route, and
   that the alloc-test (with the previous round's stair reading) is what decides it: a container
   that really stopped reads `Stopped` there once its window is long enough, so it is refuted
   there and not at the trend.

## Tests (each must fail with its change taken out: check it and report)

- dla-stats: the series above passes the gauge gate with shape Stair (fails today). The same
  series with its flat end extended past the expected gap (add flat points to 6600 and beyond)
  keeps today's verdict (plateau, gate fail). A bounded fill (a ramp up then flat: no separate
  steps) keeps today's verdict. Steps that shrink (a big first step, smaller later ones: a warm-up
  then small growth) keep today's verdict. A two-step series under the noise tolerance keeps it.
- dla-cli trend (find the trend tests: crates/dla-cli/src/trend/analysis.rs and the trend CLI
  tests): `dla trend` on a series file with the column above reports it passed, shape `stair`,
  with the note, and its finding target is not `executed` with no growth.
- judge.rs: all its existing stair tests pass unchanged after the move.
- One grown/gate test (dla-core or a dla-cli test): a finding whose only heap column is such a
  stair gets the growth lead (the close "its code ran and showed no growth" is refused while the
  lead stands, or whichever rule applies: find it and say).

## Checks (after your last edit)

Check `df -h /` first (about 4 GB free; a full dla-cli test build takes 4-5 GB: if it is under 6 GB,
run `cargo clean -p dla-cli -p dla-core -p dla-workload -p dla-bench` before the full suite, which
is the approved way to free the build directory; never delete build directories by hand). Then
`cargo test -p dla-stats`, `cargo test -p dla-core`, `cargo test -p dla-cli --no-fail-fast` (the
whole suite), `cargo test -p dla-workload --no-fail-fast`, `cargo test -p dla-report`,
`cargo test -p dla-bench --lib`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo fmt --all --check`. Then the whole dynamic set from the repository root:
`./target/debug/dla-bench dynamic --dla-bin ./target/debug/dla --parallel 2 --json <scratch file>`
(expected 63 pass and 1 expected failure, `rust-tree-parent-links`; decoy fixtures must stay
non-leak: if one now gets a lead it must be refuted by its alloc-test, say which). Delete the
dynamic logs after. Docs: one row at the end of docs/VERIFY.md, and the sentence of
skills/detect-leak-anywhere/references/ that describes the trend gate's shapes (grep `stair`,
`plateau`, `half`) where it now says something false.

## Rules

Shared `target/` only. If a build or the dynamic run fails for lack of space, stop and report it.
No `rm -rf "$VAR/..."`. Never kill processes by name. Never skip or weaken a test. No project names,
attribution lines or model names in the repository. Never use a foreground sleep; wait on long
commands with run_in_background. Scratch:
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u161/.

## Report (final message)

Files changed; the rule as implemented (exact conditions and constants, where the shared reading
lives); what the real series above now reads at the trend (shape, gate, note) and with its flat
end extended; what follows for its finding; whether any existing test's expectation changed and
why; tests with what each proves and its mutation check; every command with pass/fail counts, run
after the last edit; the dynamic result with any decoy that now gets a lead; anything you are
unsure of.
