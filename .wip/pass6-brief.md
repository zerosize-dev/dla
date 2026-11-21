# Pass 6: speed of the measurement loop on a large ledger

Same agent as passes 3 to 5, same working rules (cargo -j 2, shared target/, no commits, no
stash/reset/checkout of files, never restore a file from a copy, never kill by name, literal
paths in rm, every behaviour change with a test SEEN failing or, for a pure speed change, an
equality test plus measured timings, VERIFY rows at the end of `## M7`, nothing in the
repository names a real project, a model, a plan or an agent). Scratch `<sp>/u42/` (exists);
report with the Bash tool to `<sp>/u42/report.md`.
`<sp>` = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad.

Three other builders edit the tree at the same time: Go harness (`harness/go/`, Go parts of
`crates/dla-workload/src/harness/`, `crates/dla-core/src/queue.rs` batch functions,
`crates/dla-core/src/view.rs`, `crates/dla-cli/src/harness_cmd.rs`, the alloc-test code),
Node harness (`harness/node/`, Node parts of dla-workload, dla-bench node policy), Rust
harness (Rust parts of dla-workload, `Index::standing` in queue.rs, the shared driver-block
text). Do not change behaviour in their areas. If a hot spot is inside a function one of them
is editing, make the smallest exact-replacement change and say so in your report, or leave it
and report the measurement. If the build is broken by someone's half-finished edit, wait a
minute and retry.

## Evidence
A real run on a Java library: 602 findings, 389 ranked, 17 series. Its project copy with the
run directory is `<sp>/u36/spring-4/project` (run dir `.dla/<run>/`). Work on a COPY:
`cp -r <sp>/u36/spring-4/project <sp>/u42/proj` and run every command there with your binary
and `DLA_PLUGIN_ROOT=/home/user/dla` (the copy's ledger changes when you run `trend`; make a
fresh copy when you need the original state; delete copies you no longer need, literal paths;
keep the disk above 2 GB free).
Measured by me on that directory with the committed build: `dla trend S8` 62 s wall, 38 s
CPU. Its output and every stored trend report carry verdicts for about 320 findings, about 300
of them `not_loaded`. In the run the command kept hitting the caller's 120 s foreground limit
(the machine was loaded) and the model spent a large part of its turns waiting.

## Job
1. Time each command of the measurement loop on the copy (debug build is fine, say so):
   `dla run status`, `dla ledger list`, `dla ledger list --all`, `dla harness gen --lang jvm
   --driver --next` (it builds nothing heavy; if it starts a build, time only until it
   returns and say what it did), `dla trend S<n>` for two series, `dla gate check --stage
   classify`, `dla gate check --final`, `dla gap S<n> --residual`, `dla report`. A table of
   wall and CPU seconds.
2. For every command above 5 s: find where the time goes (a sampling profile with `perf` if
   present, else timing prints in a scratch build that you remove again, or
   `cargo flamegraph` is NOT installed: do not install tools) and fix the cause without
   changing any output: typical causes are reloading and replaying the whole run per finding
   or per column, recomputing the ranking inside a loop, quadratic matching of columns
   against findings, re-reading coverage or class lists per finding, re-parsing JSON files
   per item. `dla trend` is the priority: target under 5 s for one series on that ledger.
3. For each command you speed up: an equality check that its output (and what it writes to
   the ledger and the run directory, apart from timestamps and evidence ids that depend on
   order only if they depended on it before) is identical before and after on the copy, and
   the existing tests of that command unchanged and passing. Where a small synthetic test can
   pin the complexity without being flaky (for example a counter of run loads exposed only
   to tests), add it; otherwise report timings and add no time bound.
4. Do not change what verdicts are computed or stored. If you conclude that computing a
   verdict for every finding of the ledger in every trend is the cost and cannot be made
   cheap, stop and report that with numbers; I decide.

## Acceptance
`cargo fmt --all --check`; `cargo clippy --workspace --all-targets -j 2 -- -D warnings` (if it
fails only in files of the other builders, name them and run `--no-deps -p` on your crates);
`cargo test -j 2 --no-fail-fast -p dla-core -p dla-report -p dla-cli`;
`target/debug/dla schema --check`; `python3 bench/runner/test_grade.py`. Report: the timing
table before and after, per command what the cause was and what changed (files, functions),
the equality checks, anything not done and why, anything unverified.
