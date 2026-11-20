# X2: Python module probes never import; one site is one finding

State: all four items done and measured; tree not committed. Files and commands below.

## Causes confirmed on the code and on the measured trees
- Hang: `_find_module` in `harness/python/sitecustomize.py` ended in `importlib.import_module(name)`. The aiohttp run log
  (`u19/aiohttp-run-log.md`) names `examples/server_simple.py` (`web.run_app` at import). Reproduced here on a copy of
  aiohttp 3.10.10 (Python 3.13 venv, deps + editable, `AIOHTTP_NO_EXTENSIONS=1`) with the pre-change binary: driver gen
  with 108 automatic probes, hooks filled, `harness run` killed at the 150 s limit with the bare note "did not finish in 150 s".
- Double finding: `CandidateWriter::add` (crates/dla-core/src/toolctx.rs) keyed duplicates on `(origin, site)`; two origins
  at one site were never compared. In `u19/aiohttp/.dla/*/ledger.jsonl`: F8 (`detector:dla/python-unbounded-cache`) and F102
  (`census`) both at `aiohttp/web_app.py:82`; F9 and F117 at `web_log.py:61` (plus later growth-series findings F923.. at
  the same site, which I did not touch). Census and sweep both write through that one writer.
- `--probe F8` dropped: the census kind of a module-level cached function is `cache-decorator`; `targets_for` made a `PyAttr`
  only when the owner has a dot, so there was no target of its own and nothing said so (run log line 18).
- Other runtimes: Node reads `Module._cache[...]`, never requires (no change). JVM agent with Instrumentation reads only
  loaded classes; its fallback without Instrumentation does `Class.forName(name, false, ..)` and reading a static field then
  initialises the class. That fallback is not reachable in a harness run (the agent is always on JAVA_TOOL_OPTIONS); not
  changed, reported. Go/Rust/native probes are compiled in.

## Item 1: probes never import (done)
`_find_module` returns None unless the module is in `sys.modules` (or is the running `__main__`). A not-loaded module gives a
probe record with `error` and no values (no column) and one limitation line (up to five names). Fixed my own bug found by
an adversarial input: a spec without a module made the limitation line raise (name not converted to str).
Files: harness/python/sitecustomize.py, harness/python/README.md, references/python.md, references/stages.md.

## Item 2: phase named at a timeout (done)
Agent writes `<dump>.phase` (`mark_phase`): start-up, `hook setup`, `hook unit (round r of n ..)` (driver template
dla_harness.py), checkpoint stages and `checkpoint: probe P3 (pymodule m)`. `run.rs` clears it before each process and appends
"; it was killed in: <line>" (or "the agent recorded no phase ...") to the three timeout notes (point, driver, detector pass).
aiohttp driver with a setup that never returns, 6 s limit: "...killed (log ..); it was killed in: hook setup (the project
imports and the objects under test) (at 32 ms)". Node/JVM/Go/Rust/native agents write no phase (message says so).

## Item 3: one site, one finding (done)
- New: `Added` outcome and `CandidateWriter::add_outcome`; a census entry joins a rule/analyzer finding at its site and the
  reverse, never two of one side (two rules at one site stay two findings: `real_tools` expects linters on one line reported
  separately; my first version merged them and failed 5 real_tools tests, then narrowed). The join is one
  `finding_annotated` line (fields added to the record: `also_origin`, `also_supports`) carrying holder/owner/region the
  finding lacks (a different value goes in the note, not a refusal), the other origin's evidence, its message and archetype.
  `Finding.also_origins` is filled at replay. No new record kind, no ledger version change; schemas regenerated (X1 had
  already run `dla schema --out`; `dla schema --check` ok).
- Site equivalence: `dla_census::anchor` (Python decorator stack .. `def`/`class` line is one site) via `SiteAnchor`;
  files of other languages: exact line only (Java annotations not recognised).
- Worker slices record only on their own findings (without the check: `foreign-item` refusal, test failed).
- Consumers: census/sweep use `add_outcome` (`joined_sites` in their output; a joined hit counts in the backend's
  `duplicates` so gate accounting adds up); report shows `Also raised by` and ranks as the better of the two tiers;
  probes find the census finding through `also_origins`.
- `--probe F<n>`: module-level cached function is a `PyGlobal`; merged finding refused naming the finding to use; finding no
  probe can read refused with reason (Go command mode: use `--driver`); module-only probing said in the notes.
- aiohttp after the static stage: web_app.py:82 -> F8 only, web_log.py:61 -> F9 only; findings 189 -> 187 (180/178 census,
  7 ruff, 2 dla). Two `finding_annotated` lines with `also_origin: census`.

## Measurements (aiohttp copy, python only; machine shared with another package's runs)
| step | before | after |
|---|---|---|
| sweep python | 11.7 s | 12.9 s |
| census python | 26.1 s | 26.4 s |
| harness gen --driver, automatic probes | 0.2 s, 108 probes | 0.5 s, 112 probes (4 cached functions now have a probe of their own) |
| harness run H1 --mode fresh | killed at 150 s limit, no point | 13.8 s, exit 0, 5 points, series S1 |
| not-loaded modules | imported (hang) | 12 (examples, docs conf): no column, one limitation line |
| findings at the two sites | 2 + 2 | 1 + 1 |
Python-only candidates are about 190 in both (X1's scope work is what removes vendored/generated code).

## Bench
- `dla-bench static` on 64 fixtures, both modes: all pass with the final binary too (rerun after the last code change: 64/64 in both modes, 321 -> 299 candidates). Against the unchanged build run in the same
  environment the only differences are census candidates that joined a rule finding: 321 -> 299 candidates in total, 17
  fixtures, leak sites/flags/confirmability identical, `enumerated` identical. (Against `u17/static-*.json` there are also
  differences from tools present now: ruff, pylint, rapx, eslint plugin.)
- `dla-bench dynamic --parallel 1 --only python-`: 9/9 pass, 1 confirmed, 10 refuted, 4 claims, 8 rule-outs; per-fixture
  outcome equal to `u17/dynamic.json` (passed-column counts vary a little run to run). Other agents unchanged, so no other
  runtime fixtures were run.

## Tests (new) and what I saw fail
Failed with the change removed: harness_run `a_python_module_probe_never_imports_a_module_the_workload_did_not_load` (40 s timeout),
`a_python_point_killed_at_its_timeout_says_what_it_was_doing`, `a_point_of_another_runtime_that_times_out_says_it_knows_no_phase`;
harness_capture_python `a_probe_never_imports_and_says_so_for_shapes_a_project_really_has` (without the str() fix);
toolctx unit tests: `a_rule_hit_and_a_census_entry_at_one_site_are_one_finding_in_either_order`, `a_second_run_of_a_stage_does_not_join_again`,
`lines_of_one_declaration_are_one_site_and_other_lines_are_not`, `a_different_owner_at_a_joined_site_is_said_in_the_note_not_refused`,
`two_rules_at_one_site_stay_two_findings_and_a_census_entry_joins_the_first`, `a_worker_never_records_on_a_finding_of_the_core` (without the worker check);
dla-cli `one_site`: all four (join removed: the finding test and the merged-probe test; probe changes removed: the three probe tests).
Judged by reading (new functions, pass on first run, not reverted): anchor unit tests (3), `a_phase_note_keeps_the_first_line_and_bounds_it`,
`a_growth_series_finding_is_never_joined_to_a_static_site`, the report test `a_site_raised_by_a_rule_and_the_census_shows_both_origins_and_ranks_as_the_better`.

## Commands and results
- `cargo test -j 2 --no-fail-fast` dla-core, dla-census, dla-backends, dla-workload, dla-report: all pass; `real_tools`
  failed twice (go tests) while the disk was full (ENOSPC in the sweep; one `Blocked`), pass alone: 8/8 after freeing space.
- dla-cli: bins 264 pass; all 40 integration test targets pass (577 tests), incl. `skill_text` 13.
- `cargo clippy --workspace --all-targets -j 2 -- -D warnings`: clean (after one `type_complexity` fix in my code);
  Windows (`x86_64-pc-windows-msvc`) clippy for dla-core and dla-report: clean (dla-probe not touched by me).
- `cargo fmt --all --check` clean; `dla schema --check` ok (20); `python3 bench/runner/test_grade.py` 190 ok.

## Not done / unverified
- Java/Kotlin decorators and annotations are not site-equivalent (exact line only); growth-series findings at an already-found
  site (F923.. on aiohttp) are still separate findings (out of this item's wording).
- JVM Instrumentation-less fallback still initialises a class when reading its field (unreachable in harness runs).
- Node/JVM/Go agents record no phase. No full-run (skill) rerun of aiohttp; only the static stage and a driver run with my own hooks.
- Disk was at 0.1 to 1 GB all session; two bench runs and two real_tools tests failed with ENOSPC and were rerun.
