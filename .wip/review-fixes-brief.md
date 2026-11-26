# Builder Q2: five defects an independent review found in the abort and release rules

Repository: /home/user/dla. Work in the working tree. Do NOT commit, push or switch branches.
The tree holds uncommitted work of two earlier builders (the abort rule: crates/dla-core/src/abort_rule.rs,
crates/dla-cli/src/run/abort.rs, gate_hooks.rs; the release rules: crates/dla-core/src/release.rs and
release/tests.rs, crates/dla-cli/src/ledger_release.rs, machine.rs, explain.rs, record.rs, gate.rs, the
report). Build on it; do not revert it. Do not touch harness/, bench/fixtures/, any fixture README or
answer key, or skills/detect-leak-anywhere/SKILL.md. Read each file just before you edit it.

A long `dla-bench dynamic` run of mine is going on this machine from copied binaries. Building in
`target/` is fine. Do NOT run `dla-bench` while `pgrep -af dla-bench` shows one; wait for it to finish
(poll with a background until-loop, not a foreground sleep).

## The five defects (each confirmed against the code)

1. **Record order is not time order in worker and merged ledgers.**
   - `release.rs` `seq_of_evidence` / `seq_of_finding` search `ledger.lines()`. In a worker slice
     (`Ledger::load_slice_from`, crates/dla-core/src/ledger.rs ~1944) `lines` holds only the slice's own
     lines, its seqs restart at 1, and main-ledger records are only in `index`. So in
     `drivers_disagree`'s `settled` the `?` makes a pair of main-ledger drivers unsettleable from a worker
     (`dla ledger confirm F --worker W` stays refused after the printed route was followed), and a slice
     seq is compared with main seqs.
   - `ledger.evidence()` iterates a BTreeMap ordered by `EvidenceId` (unprefixed `E<n>` first, then
     `<worker>/E<n>` by worker name), but `bound_alloc_tests` and its callers (`drivers_of`,
     `drivers_all`, `release_route`, and the callers in crates/dla-core/src/grown.rs) treat it as oldest
     first. After a merge, a main `E10` recorded after the merge sorts before an older `W/E1`, so "the
     newest alloc-test of a harness" can be the older one.
   - `grown.rs` `evidence_seq` (used by `explained_since`) has the first problem too.
   Fix it once, generally: give `Ledger` a chronological order key for records that is valid in a plain
   ledger, a merged one and a worker slice (main records by their main seq; slice records after all
   main records, e.g. `base_len + slice seq`; first confirm from the code that the main ledger cannot
   grow while a slice is open: the `SLICE_STATE` refusal suggests so; if it can, say how you order
   then), and an iterator over evidence in that order. Use them in release.rs and grown.rs, and find
   every other caller of `ledger.evidence()` or `bound_alloc_tests` that takes the last or first as
   newest or oldest (grep `newest`, `oldest first`, `.last()`, `.rev()`), fix those where order decides
   something, and list each one in your report with whether it changed.

2. **`release-untested` is cleared by any decided `N-RELEASE` sibling, whatever release it names.**
   `release::release_untested` (~line 527) returns None when `release_refuted(root)` or
   `release_confirmed(root)`, which never compare the sibling's `remover` with the explanation's. A
   sibling naming B:20, refuted where B ran, clears an explanation that relies on A:10 that no run
   called. Require the sibling that clears it to name the same remover line (same line number and the
   same file by the module's `same_file`). Keep `grows_only_while_skipped` (the report's "counted
   apart") as it is unless you find it wrong, and say why.

3. **A missing coverage record on a runtime that counts lines is accepted with the note.**
   `release.rs` ~186: for Rust, Go and Node, any `Err` of `counted_in_coverage` (including "the harness
   run wrote no coverage record") becomes `RemoverRun::NotShown`, which `check_release_decision`
   accepts with "remover run not shown". A Rust harness never run with `dla harness run` (so no
   `executed.json`) therefore confirms `N-RELEASE` with no sign that the release ran. The note is for a
   runtime or environment that cannot count the line (JVM, native, or a coverage record that itself
   says the tool could not count: keep that). An absent record, or one from a run before the sibling
   named the line, must be refused (`release-not-run`) with the command that writes the record
   (find which commands write `executed.json`: `dla harness run`, and check whether `dla alloc-test`
   does). Check the Python path (`newest_line_runs`, `lines-*.json`) the same way.

4. **The report subtracts a finding twice when it is both disputed by the verifier and released.**
   crates/dla-report/src/lib.rs `findings_line` (~line 370-385): with confirmed=2, A disputed and
   released, B a plain confirmed leak, the line prints confirmed 0, disputed 1, released 1: B is hidden
   and A is counted twice. A finding goes in one bucket. Decide which (disputed is the verifier's
   word; say what you chose and why), and make every count of the `Findings:` line add up. Check the
   strong-state counts the same way.

5. **The fix printed for a failed `--blocker` check does not run as printed.**
   crates/dla-cli/src/run/abort.rs ~342 prints `dla run abort --blocker {kind} {fix}`, where the fix
   strings from `abort_rule::check_blocker` are not command tails (`--blocker user|tool|...` repeats
   the flag; `--code only with --blocker refusal` makes `only` the value and `with` a stray
   positional). Make every fix `check_blocker` can return a full command that runs as printed apart
   from `<placeholders>` and one `a|b|c` choice, and make abort.rs print it unchanged.

Also three wording slips in skills/detect-leak-anywhere/references/:
- evidence-and-report.md ~213 (`drivers-disagree` row): the settling sibling must be added after the
  older of the two alloc-tests, not after both.
- evidence-and-report.md ~214 (`release-untested` row): the hold also ends when an `N-RELEASE` sibling
  naming that remover is confirmed (the finding is then counted apart), not only refuted.
- recovery.md ~105: the aborted run's Stop hook blocks the first two stop attempts and allows the
  third; say it the way the hook does (read crates/dla-cli/src/gate_hooks.rs `stop_aborted` and
  STOP_LIMIT).

## Tests (each must fail with its fix taken out: check it and report)

- Worker scene: drivers E_a (leak) and E_b (flat) recorded in the main ledger on two harnesses made
  for F; a worker slice adds a non-leak sibling and refutes it with its own alloc-test on a third
  harness made for F that favors leak; `dla ledger confirm F --worker W` passes (it is refused before
  the fix). Use the existing worker test helpers (find them: crates/dla-cli/tests, `--worker`).
- Merged order: a merged ledger where a worker alloc-test on harness H is older (earlier line) than a
  main `E<n>` on H recorded after the merge; the newest for H is the main one (a dla-core unit test is
  enough, plus one CLI-level check of a decision that depends on it if there is one).
- `release-untested` with a sibling naming another remover stays held; with the same remover, clears.
- Rust/Go/Node harness with no `executed.json`: N-RELEASE confirm refused `release-not-run` with the
  command that writes it; with a record counting the line > 0: passes; JVM: accepted with the note.
- Report: the disputed + released + plain scenario above prints every count right.
- Abort: the two bad `--blocker` cases print a fix with exactly one `--blocker` that, run as printed
  with its placeholders filled, is accepted by the argument parser.

## Checks to run

`cargo test -p dla-core --lib`, `cargo test -p dla-core --tests`, `cargo test -p dla-cli --bin dla`,
the dla-cli test files you touch plus `release_hazard`, `abort_verify`, `gate_flow`, `grown_chain`,
`grown_dump`, `gate_hooks`, `skill_text`, the worker/merge tests (find them), `cargo test -p dla-report`,
`cargo test -p dla-bench --lib`, then `cargo clippy --workspace --all-targets -- -D warnings` and
`cargo fmt --all --check`. Then, once no other dla-bench is running, the whole dynamic set from the
repository root: `./target/debug/dla-bench dynamic --dla-bin ./target/debug/dla --parallel 2 --json
<scratch file>` (expected 63 pass and 1 expected failure, `rust-tree-parent-links`).

## Rules

Shared `target/` only. Disk is tight: before building check `df -h /`. Under 3 GB free, delete stale
hash dirs older than 120 minutes under the literal paths /home/user/dla/target/debug/build/dla-cli,
/home/user/dla/target/debug/build/dla-core, /home/user/dla/target/debug/build/dla-workload and
/home/user/dla/target/debug/build/dla-bench with `find <literal path> -mindepth 1 -maxdepth 1 -type d
-mmin +120 -exec rm -rf {} +`, never while a cargo test or a dla-bench run is going. No
`rm -rf "$VAR/..."`. Never kill processes by name. Never skip or weaken a test. No project names,
attribution lines or model names in the repository. Update the existing docs/VERIFY.md rows of these
rules where they now say something false, and add one row at the end for the record-order cause
(re-read the file just before you write). Scratch:
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u153/.

## Report (final message)

Files changed; for each defect the fix as implemented (the order key and where it is used, every
caller you checked for item 1 and whether it changed, the remover match, which records are refused
vs accepted with the note per runtime, the report bucket chosen, the printed abort fixes verbatim);
tests with what each proves and its mutation check; every command with pass/fail counts; the dynamic
result; existing tests whose expectations changed and why; anything you are unsure of.
