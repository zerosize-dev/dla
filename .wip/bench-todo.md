# Planned benchmark work (after the current round is released)

Why: three real runs on Kotlin Spring Boot services failed in ways the benchmark never showed
(it was green): the model stopped with 55 of 60 minutes left and asked whether to continue; a
generated harness was never filled or run; "0 of 225 entry points exercised" after 750
iterations; a finding closed `time_budget` at 10 of 60 minutes; non-leak explanations counted
as confirmed; a class path assembled by hand.

Rules: no model benchmark runs (no `dla-bench run`, no API calls) and no benchmark in CI;
mechanical tests, graders and `dla-bench dynamic`/`static` only. The Rust grader
(bench/runner/src/grade.rs, metrics_ext.rs) and the Python grader (bench/runner/grade.py) stay
in exact agreement: every new metric in both, golden tests in both. Do not change the `Status:`
line or any existing metric's meaning; new metrics are additive. Do not edit bench/fixtures/**
or EXPECTED_FAILURES.

1. Per-run metrics in an informational block (not the release ratios): `budget_used_fraction`,
   `exercised_fraction`, `time_budget_closures_early` (a `time_budget` closure while more than
   max(10 min, 20%) of the budget remained, from the ledger timestamps and the run's budget),
   `harness_ran`. GRADING.md section; README line: read the transcript tail of any run under 50%
   budget used.
2. Trace graders for the eval cases: no question to continue in the final assistant turn (with
   a "the run reached the report" precondition); on a JVM case, no hand-built `java -cp` /
   `-classpath` path join in Bash calls.
3. A bad and a good trimmed transcript/ledger pair per real failure under
   bench/runner/testdata/, and bench/runner/tests/failure_fixtures.rs: each new grader or metric
   fails its bad twin and passes the good one. GRADING.md: a new grader or gate merges only with
   such a pair.
4. A worst-run gate on the shipped arm (no early `time_budget` closure; budget used at least 0.8
   unless every open item is unresolved with a blocker only the user can clear) and min/max
   columns in table.rs. Gate unit test with one bad run among three.
5. `explanation_as_leak` (a confirmed finding whose label is N-*, or counted as a leak in the
   report's Findings line), pinned at 0 on non-leak and decoy-control fixtures, in both graders.
6. README paragraph: compare a change against a snapshot of the previous skills/ tree.
7. `dla-bench commands`: from transcript.jsonl, Bash commands normalised to their first two
   tokens, top 20 by run and fixture, non-`dla` marked, harness fill and run counts; report-only.
8. If disk allows: one many-entry-point Kotlin eval case under bench/evals (scaffold.sh, 40-60
   handlers, stub annotations, no network), one real leak, explain-only non-leaks, unattended
   prompt, with the graders above.
9. The `jni-order-codec` flake: on CI run 43247003121 (attempt 1) both native-shim summaries
   (`type-histogram-diff via shim-stack-groups`, `alloc-stack-diff via shim-alloc-stacks`) failed
   their gate and the L-HANDLE label had no evidence; the re-run passed and local runs pass. Find
   why the native stacks can miss (run it several times, under CPU load; compare shim dumps of a
   passing and a failing run) and fix the cause; do not loosen a gate. If it cannot be
   reproduced, record the evidence in VERIFY as open.
