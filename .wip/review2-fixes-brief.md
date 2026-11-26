# Builder Q3: five defects a second review found in the record order, release and abort fixes

Repository: /home/user/dla. Work in the working tree. Do NOT commit, push or switch branches.
The tree holds uncommitted work of earlier builders (the abort rule, the release rules, and the
fixes of the first review: the record-order key in crates/dla-core/src/ledger.rs
(`Index.clock`, `evidence_order`, `finding_order`, `confirmed_order`, `evidence_in_order`),
`release::same_remover`, `RemoverRun::NoRecord`, the report's one-bucket rule, whole-command
abort fixes). Build on it; do not revert it. Do not touch harness/, bench/fixtures/, any fixture
README or answer key, or skills/detect-leak-anywhere/SKILL.md. Read each file just before you
edit it. Nothing else runs on this machine now; you may run dla-bench at the end (see Checks).

## The five defects (each confirmed by two independent readers of the code)

1. **The order key is the merge position, not the time, for main records written while a slice
   was open.** The core can write to the main ledger while a worker slice is open
   (crates/dla-core/tests/record_order.rs does it). `Index::commit` keys every record by its
   replay position (`clock`). `append_merged_held` (ledger.rs ~3165) appends a slice's lines after
   all main lines, keeping each line's own `ts`. So a main alloc-test E17 recorded at t2, while the
   worker recorded `w/E1` at t1 < t2, sorts as OLDER than `w/E1` after the merge, and
   `evidence_in_order`, `bound_alloc_tests` and their callers (`drivers_of`, `drivers_disagree`,
   `decided_by_alloc_test`, `route_to_confirm`, `explained_since`, ...) take the older worker
   record as the newest. Two workers' slices merged one after the other have the same problem
   between them.
   Fix: key records by time with the replay position as the tie-break: `(ts, clock)`, where `ts`
   is the line's own `LedgerLine.ts` (a merged line keeps its worker's `ts`). Carry `ts` into
   `commit` (find every caller of `commit`/`replay_lines`/append that reaches it). Give the key a
   small public type with `Ord` (e.g. `RecordOrder { ts, at }`) and make `evidence_order`,
   `finding_order`, `confirmed_order` return it; keep the callers' comparisons working on it.
   Check the worker view: main lines are cut at `base_seq`, so they are all older than the slice's
   own lines by `ts` as well; confirm nothing there needs the old `base_len + slice seq` rule, or
   keep it as the tie-break if `ts` can tie. Before you change the key, grep the tests for ledger
   text or records built with fixed or decreasing timestamps (`ts`, `2026-`, `DateTime`,
   `with_ts`, `Pending`) and say in your report whether any relies on replay order against `ts`
   order; do not change a test's expectation to fit without saying why. Update the doc comments
   (ledger.rs `evidence_order` says "valid ... in a merged one", which is only true for records
   made after the merge).

2. **Two gate helpers still order by `lines()` seq, which is slice-local in a worker view.**
   crates/dla-core/src/gate.rs `generated_at` (~2364) and `blocker_recorded` (~2379) search
   `view.ledger.lines()` and compare `l.seq`. Under `dla gate --worker W` (gate_cmd.rs builds
   `RunView::load_for(run_dir, worker)`) `lines()` holds only the slice's lines, so a harness
   generated in the main ledger (`dla harness gen` writes there) is not seen and the worker gate
   leaves out the `harness_not_run` item core mode lists (path: `worker_missing` ->
   `unrun_harness_items` -> `harness_not_run` -> `unrun_harness` -> `generated_at`).
   `blocker_recorded` compares slice seqs with main seqs. Fix both on the index and the key of
   item 1 (`evidence_in_order` + `evidence_order` for the workload record; for checklist answers
   give the index the same key, or iterate whatever the index keeps of them in key order). Then
   grep once more for any other `lines()` search that compares `seq` or takes the first or last
   match as oldest or newest (`.lines().iter()`, `l.seq`) and fix those that decide something in a
   worker view or after a merge; list each in your report with whether it changed.

3. **The release hold and the report disagree on a confirmed sibling that names another
   remover.** `release_untested` (crates/dla-core/src/release.rs ~538) is cleared only by a
   refuted or confirmed `N-RELEASE` sibling that names the same remover (`same_remover`).
   `grows_only_while_skipped` (~754) and the report's "Grows only while callers skip the release"
   block (crates/dla-report/src/lib.rs ~1288, `release_confirmed`) count a finding apart on ANY
   confirmed `N-RELEASE` sibling. So with explanation remover A:10 and a confirmed sibling naming
   B:20, the gate lists `release-untested` for A while the report counts the finding apart from the
   leaks. Decide it this way: a CONFIRMED `N-RELEASE` sibling of the root, whatever line it names,
   says the family's growth was measured flat with a release called, so the finding grows only while
   callers skip it; it is counted apart, and it also clears `release-untested` (nothing is left to
   ask: the finding is not reported as a leak). A REFUTED sibling clears the hold only when it names
   the same remover (a release that did not stop the growth says nothing about another one). Make
   `release_untested`, its refusal and route text, the gate item text, the docs
   (skills/detect-leak-anywhere/references/evidence-and-report.md `release-untested` row,
   stages.md) and the module doc of release.rs say exactly that, and keep the existing
   "same remover" test for the refuted case.

4. **A Python run whose agent could not count lines is refused for good.** When the Python agent
   cannot count (harness/python/sitecustomize.py `_start_lines` returns early with a note when
   another tool already traces the process, or settrace fails), the records carry that note in
   `limitations` and no `line_counts`, so `write_line_runs` (crates/dla-workload/src/harness/run.rs
   ~941) returns at `sites.is_empty()` and writes no `lines-<S>.json`. `remover_run` then says
   `NoRecord`, `check_release_decision` refuses `release-not-run`, and the printed rerun writes no
   file again. Fix in Rust only (do not edit harness/): when the records of the run say the agent
   could not count lines (find the exact note texts in sitecustomize.py and match them by a shared
   constant or a stable prefix, not by copying a sentence that may change; if there is no stable
   marker, say so and choose the most robust test you can), write `lines-<S>.json` with a field
   that says counting was unavailable and why (`#[serde(default)]` so old files still load), and
   make `remover_run` return `NotShown(<why>)` for such a file. A missing file stays `NoRecord`.
   Find every caller of `write_line_runs` and `LineRuns::load` and check none of them misreads the
   new field (e.g. treating "unavailable" as "ran 0 times").

5. **The `budget-left` refusal and the Stop hook offer `--blocker refusal` without `--code`.**
   crates/dla-cli/src/run/abort.rs (~366) prints `dla run abort --blocker
   <user|tool|build|environment|privilege|refusal> --reason "<what blocks the run>"`; choosing
   `refusal` is refused (`--code` is required). crates/dla-cli/src/gate_hooks.rs `held_note`
   (~880) prints `--blocker <kind>` with the same gap. Make both print commands that run as printed
   for every choice: e.g. `dla run abort --blocker <user|tool|build|environment|privilege> --reason
   "<what blocks the run>"` and, for a refusal, `dla run abort --blocker refusal --code <refusal
   code> --reason "<...>"` (keep the hook note short). Update the tests that pin these strings
   (gate_hooks.rs tests ~2169, ~2318 and any other) and add a test that fills every choice of the
   printed text, parses it with the real clap `Cli` (see the existing test
   `every_fix_of_a_failed_blocker_check_runs_as_printed` in abort.rs) and runs `check_blocker` on
   it.

## Tests (each must fail with its fix taken out: check it and report)

- Merge order: a main alloc-test written while a slice is open, newer by `ts` than a worker
  alloc-test on the same harness, stays the newest after the merge; and a decision that depends on
  it (e.g. `drivers_disagree` or the newest verdict of a harness) reads the right one. Also two
  workers merged in the order opposite to their records' times.
- Worker gate: a harness generated in the main ledger is listed by `dla gate --worker W` as core
  mode lists it (CLI test; find the worker gate tests).
- Release: a confirmed sibling naming another remover clears `release-untested` and counts the
  finding apart (report and gate agree); a refuted sibling naming another remover keeps the hold.
- Python: a run whose records carry the agent's "could not count" note: N-RELEASE confirm accepted
  with the note "remover run not shown"; no lines file at all: still refused `release-not-run`.
- Abort and hook: every choice of the printed commands parses and passes `check_blocker`.

## Checks to run

`cargo test -p dla-core --lib`, `cargo test -p dla-core --tests`, `cargo test -p dla-cli --bin
dla`, the dla-cli test files you touch plus `release_hazard`, `abort_verify`, `gate_flow`,
`grown_chain`, `grown_dump`, `gate_hooks`, `skill_text`, `ledger_m4`, `ledger_m6`, `run_m4`,
`py_site_columns`, `cargo test -p dla-workload` (the lines change), `cargo test -p dla-report`,
`cargo test -p dla-bench --lib`, then `cargo clippy --workspace --all-targets -- -D warnings` and
`cargo fmt --all --check`. Then the whole dynamic set from the repository root:
`./target/debug/dla-bench dynamic --dla-bin ./target/debug/dla --parallel 2 --json <scratch file>`
(expected 63 pass and 1 expected failure, `rust-tree-parent-links`). Check `pgrep -af dla-bench`
first; if one runs, wait for it with a background until-loop.

## Rules

Shared `target/` only. Disk is tight (about 2.3 GB free): check `df -h /` before building. Do not
delete build directories; if a build fails for lack of space, stop and report it (do not work
around it). No `rm -rf "$VAR/..."`. Never kill processes by name. Never skip or weaken a test. No
project names, attribution lines or model names in the repository. Update the docs/VERIFY.md
record-order row (the last one) and the release rows where they now say something false (re-read
the file just before you write). Scratch:
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u155/.

## Report (final message)

Files changed; for each defect the fix as implemented (the key type and every place it is used;
whether any test relied on replay order against `ts` order; the `lines()` searches you checked and
whether each changed; the release rule as now written and its texts; the Python marker you match
and why it is stable; the printed abort and hook commands verbatim); tests with what each proves
and its mutation check; every command with pass/fail counts; the dynamic result; existing tests
whose expectations changed and why; anything you are unsure of.
