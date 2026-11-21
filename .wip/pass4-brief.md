# Pass 4: report and gate defects seen in a real run (aiohttp 3.10.10, queue build)

You are the same agent that did pass 3 (close reasons, env reuse, `enclosing`). Same working
rules as your pass-3 brief (`wip-notes/pass3-brief.md`): cargo `-j 2`, shared `target/` only,
never kill by name, no `rm -rf` with a variable in the path, no commits, write your report
with the Bash tool (`cat > <sp>/u37/report.md <<'EOF'`), every behaviour change gets a test
you have SEEN fail without it, docs/VERIFY.md rows for what changes behaviour, no plan names
or model names in the repository. Scratch: `<sp>/u37/`. Other agents may still be editing the
tree (census/rank code, trigger code): do not touch `crates/dla-census/`,
`crates/dla-core/src/rank.rs`, or the trigger code (`--entry`, `VARIES.txt`, `Trigger:` lines,
gate item kind `trigger`); if a file you need has their edits, edit only your hunks with exact
replacements and never restore a file from a copy.

Evidence: the report and ledger of the run are in `<sp>/u31/aiohttp-2/` (report.md,
ledger.jsonl). The run had 197 findings, 90 ranked, 15 driver harnesses, 48 series.

## Defects (each: reproduce with a small test first, then fix)

1. `Queue:` line counts only findings still in the candidate universe. The report said
   "89 of 90 ranked candidates targeted in 15 harnesses: grew 1, executed 65, not reached 23;
   1 not targeted" while two targeted findings were CONFIRMED (they had left the candidate
   state, so they fell out of "grew" and out of the 90). Code: `ReportFacts` in
   `crates/dla-core/src/queue.rs`. Wanted: the line counts every finding that was ranked when
   the queue was built, whatever its state now; a finding that grew and was later confirmed or
   refuted still counts in "targeted" and in "grew". State the rule in the doc comment.

2. `Residual:` header line: one clause per series (48 clauses, several kB on one line) with
   unrounded floats ("heap 1304.199053030303 bytes/iteration unattributed of
   1304.199053030303 total (threshold 326.04976325757576) ABOVE THRESHOLD, see O4; ..."). Wanted: numbers
   rounded like the rest of the report (find the existing number formatter; no new one), and
   the header bounded: the series above threshold named (with their O-ids), the rest counted
   ("44 series below threshold"). The per-series detail stays wherever the body of the report
   already has it; if it is nowhere else, keep it in a body section, not in the header.

3. `Coverage: python 3520 iterations, 0 of 0 entry points exercised` for hook-driver runs:
   "0 of 0" reads as nothing covered. Find why a driver run of Python has no discovered entry
   points (the run used `dla harness gen --lang python --driver --next`, 15 times) and decide
   from the code what is true: if the entry points were never discovered, the line must say so
   in words ("entry points not enumerated" or the tool's existing phrase), not "0 of 0"; if
   discovery should have run, fix that. Do not invent coverage.

4. The final gate asked for a residual row per series: 176 gate items, which the agent
   cleared with a shell loop over `dla gap S<n> --residual` and canned `inconclusive` closes.
   That is busywork with no information. Read how the final gate derives its residual items
   and what the plan of record in docs says a residual must cover. Wanted: one residual
   obligation per harness mode that has a growth signal (a series whose trend passed the gate,
   or is the leading series of a confirmed/strong finding), not one per flat series; flat
   series are covered by a single line the tool computes itself. If you find the per-series
   rule is deliberate and documented, do NOT change it silently: write what you found in your
   report and add a VERIFY row proposal instead; I decide.

5. Verifier site disagreement that is only declaration line vs writer line: the finding's
   site was `aiohttp/web_log.py:61` (the holder's declaration), the verifier answered line 75
   (the writer of the same holder) and the comparison counted a site disagreement. Find the
   comparison (verifier result intake, `dla ledger verify` or the bundle check) and make a
   site that is the holder's declaration, one of its writers or removers, or the same function
   count as the same site. Use the finding's own recorded holder/writer facts, no new parsing.

6. Your pass-3 note: "a series with no growth column writes no growth record, so its verdicts
   never reach the ledger". Write the test that shows it (a driver harness with declared
   targets whose series is flat: after `dla trend`, are the target verdicts `executed` /
   `constant` readable by `dla ledger list` and by `close-rest`?). If they are lost, fix it so
   a flat series still records the verdicts of its declared targets.

## Acceptance (run what the other agents' state allows; say exactly what you could not run)
- `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings` (if it
  fails only in files you did not touch, name them and run `--no-deps -p` on your crates).
- `cargo test -j 2 -p dla-core -p dla-report -p dla-cli` (name every failing test and whose
  code it is).
- `dla schema --check`, `python3 bench/runner/test_grade.py`, the `skill_text` test.
- Rebuild the report for the kept ledger if that is possible offline (if `dla report` needs
  the run directory, say so and show the lines from your tests instead).
- Report: what changed (files, functions), the test for each defect and that you saw it fail,
  anything you decided not to change and why, anything unverified.
