# Brief S: allocation-site columns are shown largest growth first

Repository: /home/user/dla (Rust workspace, branch `main`, a dirty working tree with unfinished
changes on top of commit 29cb6f6; all of it is to stay). Scratch directory for your files:
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u81 (create it).

## The defect

A Python driver run writes `py.blocks@<file:line>` columns (live traced blocks per project line;
`dla_core::grown::is_alloc_site_column` recognises them). A loop over a project's whole test
suite gives up to twelve such columns that pass the growth gate, all with the same Mann-Kendall
p value (every step rises).

Measured on a real library: twelve passing site columns with Sen slopes from 24 to 337 blocks
per run. The line of the real leak had the second largest slope (299). `dla trend` printed the
passing columns in NAME order and the `dla ledger add ... --site ...` command for the first
THREE in name order: slopes 56, 24 and 101. The leak line was tenth by name, so the agent never
saw a command for it. The final gate listed the twelve untied columns in name order too.

Cause: `passing` in `crates/dla-cli/src/trend/summary.rs` sorts by (tied to something, p value,
name). The suggestions in `crates/dla-cli/src/trend_cmd.rs` (`fn suggestions`, the
`None if unbound < 3 && site_add.is_some()` arm) walk that order. The final gate
(`crates/dla-core/src/gate.rs`, the loop that ends in `out.add("trend_column", ...)` with the
`site_column_add` fix) walks the columns in the order of the trend file, which is name order.

## The change

1. `summary::passing`: among allocation-site columns that are equal on the keys before the name
   (tied or not, p value), the one with the larger Sen slope comes first; the name decides only
   after that. Columns that are not allocation-site columns keep exactly today's order relative
   to each other and to the site columns (do not compare slopes of different units). A missing
   slope sorts last.
2. `dla trend`: the `passed` list of the output and the add commands for untied site columns
   follow that order, so the three printed add commands are for the three largest growths,
   largest first. No change to the cap of three or to the command text.
3. Final gate (and the stage gates that use the same loop): the items for untied allocation-site
   columns of one trend are added largest Sen slope first. Items for other columns keep their
   order and their place relative to the site columns as a group. The column's slope is in the
   trend file (`sen.slope`); look at how `ColumnTrend` exposes it.
4. Texts, one short clause each, where the site columns and the add command are described:
   `skills/detect-leak-anywhere/references/python.md` and
   `skills/detect-leak-anywhere/references/stages.md`: say that the trend and the final gate name
   the lines largest growth first, so the first commands are for the lines that grow most. Do NOT
   touch `skills/detect-leak-anywhere/SKILL.md` (size limit) and do NOT touch `docs/VERIFY.md`
   (the orchestrator edits it).

## Tests (each seen failing under a mutation you apply and revert)

- Unit test next to `passing`: five passing site columns with the same p value whose name order
  is the reverse of their slope order, plus one non-site passing column; the site columns come out
  largest slope first and the non-site column keeps the place today's rule gives it.
- CLI test in `crates/dla-cli/tests/py_site_columns.rs` (read its helpers first; it builds series
  by hand): a series with five untied passing site columns, name order opposite to slope order:
  `dla trend` prints add commands for exactly the three largest, largest first, and `passed`
  starts with the largest.
- Gate test (same file or a unit test in gate.rs, whichever has the helpers): the final gate lists
  the untied site columns largest slope first.

Mutations by a small script that asserts the old text occurs exactly once; see each test fail;
revert. Before you finish, `grep -rIl "MUT_" crates harness skills docs bench agents hooks data`
must print nothing.

## Rules

- No commit, stash, checkout, reset, restore or branch change. Leave the tree dirty.
- Use the shared `target/` directory only (plain `cargo` from /home/user/dla, `-j 2`).
- No `rm -rf` on variable paths. Do not kill processes by name. Do not run `dla consent`. Do not
  start any model, agent or verifier. If an action is denied, stop and report it.
- Keep to this brief. If you find another defect, report it; do not fix it.
- Plain technical English in texts and comments, full sentences, no em dashes. Never name a real
  third-party project, library or its APIs in the repository. No model names, no attribution lines.
- If an existing test or golden file changes, say which and why; do not weaken a test.

## Checks before you report

1. `cargo fmt --all --check` clean.
2. `cargo clippy -j 2 --workspace --all-targets -- -D warnings` clean.
3. `cargo build -j 2 -p dla-cli` and `./target/debug/dla schema --check` clean.
4. `cargo test -j 2 -p dla-cli --test py_site_columns --test skill_text --test keep_going --test gate_flow`,
   `cargo test -j 2 -p dla-cli --bins` (or the target that holds the `trend::summary` unit tests) and
   `cargo test -j 2 -p dla-core --lib gate` pass.
5. Do NOT run the whole workspace suite; the orchestrator runs it.

## Report (final message; read by the orchestrator)

Two sentences of result first. Then: files changed; each test with its mutation; existing tests or
golden files changed and why; anything in this brief that was wrong; what you did not verify.
Counts only, no command output.
