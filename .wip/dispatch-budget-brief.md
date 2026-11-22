# Brief U: the automatic budget of a run with workers, and gate items that name the driver while the test loop is pending

Repository: /home/user/dla (Rust workspace, branch `main`, a dirty working tree with unfinished
changes on top of commit 29cb6f6; all of it is to stay). Scratch directory for your files:
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u83 (create it).

Two items. Do item 1 first, then item 2. Each has its own tests.

## Item 1: the `auto` budget is never sized in a dispatched run

### What happened (a real unattended run on a project with two runtimes, python and native)

`dla run init --budget auto` starts the budget at 60 minutes and sizes it later with
`clamp(20 + 15 * R + 3 * C, 60, 180)` minutes (`crates/dla-core/src/state.rs`, module `auto`;
R runtimes with a candidate, C candidates). The run dispatched two workers. The python worker
had 186 ranked candidates, so the formula gives the cap, 180 minutes. The run ended with:

    "budget": {"seconds": 3600, "auto": true, "settled": true, "settled_workers": null,
               "estimate": {"runtimes": 0, "candidates": 0, "minutes": 20, "total_minutes": 60}}
    workers: python share_secs 1527, native share_secs 1527   (25 minutes each)

and the report said "budget 1h (auto: 0 runtimes, 0 candidates)" and "175 findings were closed
for lack of time".

### Why

1. A worker's check sizes the budget only when it PASSES at `sweep` or later
   (`crates/dla-cli/src/gate_cmd.rs`, the block that starts with the comment "The first passing
   check at `sweep` or a later stage sizes an `auto` budget"). With the measurement queue on, a
   worker's gate at `sweep` lists its candidates that have no disposition, so it does not pass
   until the worker's work is done. The worker never sized the budget.
2. The core sizes the budget from its own view as soon as sweep and census have run
   (`settle_auto_budget` in `crates/dla-cli/src/scope.rs`, called for every core command from
   `crates/dla-cli/src/main.rs`; and the `ready` path of the core's own gate check). In a
   dispatched run the coverage manifest already lists the workers' sweep and census, but the
   core's own ledger has no candidate, so the estimate is 0 runtimes and 0 candidates, and
   `settle_budget(estimate, None)` makes it final (`settled = true`). After that no worker can
   raise it (`crates/dla-core/src/state.rs`, `settle_budget`).

### The change

a. In a run that has registered workers (`state.workers` is not empty), the core does not size
   the budget from its own view: neither `settle_auto_budget` nor the `ready` path of the core's
   gate check, nor a passing core gate at `sweep` or later. Sizing belongs to the workers. (A run
   without workers behaves exactly as today.)
b. A worker sizes the budget when sweep and census have run for ITS runtime, whether its gate
   passes or not: the same idea as the core's `ready` rule. It must not depend on the worker
   running `dla gate check`: the first command of the worker after its sweep and census
   (`dla ledger list`, `dla run status`, `dla harness gen`, a gate check, ...) sizes it, through
   the same place that calls `settle_auto_budget` for the core. Use
   `timebudget::estimate_across` (every worker's slice plus the main ledger), as the worker's gate
   path does today, and `settle_budget(estimate, Some(worker))`: raise-only, once per worker,
   final once every registered worker has sized it. The delta moves the run deadline and every
   unfinished worker's deadline (that is what `settle_budget` does today; do not change it).
c. Whatever prints "sized" lines today (the gate's `budget_sized` output, `timebudget::settled_line`)
   keeps working for the worker path; the worker's first sizing command should say so in its
   output if the core's does (follow what exists; do not invent a new output shape).
d. `skills/detect-leak-anywhere/references/stages.md` states the sizing rule ("the first passing
   gate check at sweep or later ...; in a dispatched run each worker's first passing check ...").
   Correct it to the new rule in the same place, and `docs/VERIFY.md` gets ONE new row at the end
   of its table in the style of the last rows: left column "The automatic budget of a run with
   workers stayed at one hour", state `corrected`, the facts above (a run with two workers, 186
   candidates in one of them, estimate 0 runtimes and 0 candidates, 25 minutes per worker, 175
   findings closed for lack of time), the change, and the tests. Do NOT touch
   `skills/detect-leak-anywhere/SKILL.md` (size limit).

### Tests for item 1 (each seen failing under a mutation you apply and revert)

Find how the existing tests build a dispatched run (search `crates/dla-cli/tests` for
`settled_workers`, `share_secs`, `--worker`, `dispatch`) and reuse their helpers.

- A dispatched run with two workers; one worker's slice gets enough candidates for the formula to
  exceed 60 minutes (C of 14 or more with R = 1). After that worker's sweep and census, its first
  command that is NOT a gate check sizes the budget: `budget.estimate.candidates` counts the
  candidates across slices, `budget.seconds` is above 3600, `settled_workers` holds the worker,
  `settled` is false while the other worker has not sized, and both unfinished workers' deadlines
  moved by the same delta.
- The same with a FAILING `dla gate check --stage sweep` of the worker as its first command.
- A core command after the workers' census (for example `dla run status` without `--worker`) does
  not settle the budget: `settled` stays false and the estimate is not 0 runtimes, 0 candidates.
- A second sizing by the same worker changes nothing; the second worker's sizing may only raise.
- A run without workers: unchanged (the existing tests pin it; say which).
- Unit tests of `settle_budget` if you change it (you should not need to).

## Item 2: gate items name the driver batch while the Python test loop is the queue's first step

### What happened

An open Python queue of a project with test files starts with the project's own tests in a loop
(`RuntimeQueue.py_loop` in `crates/dla-core/src/queue.rs`; `dla run status` prints
`dla harness gen --lang python --pytest`; the queue's gate item, `view::queue_item` in
`crates/dla-core/src/view.rs`, has a fix that starts with the loop and then names the driver
batch). In the same real run the python worker's `dla gate check --stage sweep` printed, with the
loop still pending:

    finding python/F1 is candidate and has no disposition (+90 waiting on it: ...)
      Fix: dla harness gen --lang python --driver, fill the DLA-HOOK bodies of the harness it prints ...
    finding python/F2 is candidate and has no disposition (+185 waiting on it: ...)
      Fix: dla harness gen --lang python --driver --next

The worker ran the second fix. Nothing in that output named the loop.

### The change

While `py_loop` is `Some` for the python queue of the view (and worker) the gate is checking, a
gate item of a python finding whose fix is a harness-generation step (the queue's batch command,
or the "make a driver" step of `harness_not_generated` / `make_driver_step`) gets the loop first:
its fix is the same text as the queue item's fix (`view::queue_item(q).fix`), or, for the "make
a driver" step, that text followed by the existing sentence about recording the blocker. Find the
places in `crates/dla-core/src/gate.rs` that build those fixes (look for `command_for`,
`batch_command`, `make_driver_step`, `harness_not_generated`) and apply the rule in one helper.
When `py_loop` is `None` nothing changes. Other runtimes are unchanged. `RuntimeQueue::command_for`
itself stays as it is (the candidate's own command is the driver's; other callers rely on it).

### Tests for item 2 (each seen failing under a mutation)

In `crates/dla-cli/tests/pytest_loop.rs` (the plain `project(p, true)` fixture has test files;
read the helpers): after probe, sweep, census, `dla gate check --stage sweep` lists the candidate
items and every python item whose fix generates a harness starts with
`dla harness gen --lang python --pytest`; after a refused generation of the loop (see the existing
test `without_pytest_the_loop_is_refused_with_the_install_text_and_the_queue_goes_on`) the same
items name the driver batch as today; the same project without test files names the driver batch.
If the fixture has no candidate that takes the "make a driver" path, cover that path with a unit
test next to the helper.

## Rules

- No commit, stash, checkout, reset, restore or branch change. Leave the tree dirty.
- Use the shared `target/` directory only (plain `cargo` from /home/user/dla, `-j 2`).
- No `rm -rf` on variable paths. Do not kill processes by name. Do not run `dla consent`. Do not
  start any model, agent or verifier. If an action is denied, stop and report it.
- Keep to this brief. If you find another defect, report it; do not fix it.
- Plain technical English in texts and comments, full sentences, no em dashes. Never name a real
  third-party project, library or its APIs in the repository (say "a project with two runtimes").
  No model names, no attribution lines.
- If an existing test or golden file changes, say which and why; do not weaken a test. A test that
  pinned the old sizing rule for dispatched runs may change to the new rule; say so.
- Mutations by a small script that asserts the old text occurs exactly once. Before you finish,
  `grep -rIl "MUT_" crates harness skills docs bench agents hooks data` must print nothing.

## Checks before you report

1. `cargo fmt --all --check` clean.
2. `cargo clippy -j 2 --workspace --all-targets -- -D warnings` clean.
3. `cargo build -j 2 -p dla-cli` and `./target/debug/dla schema --check` clean.
4. `cargo test -j 2 --no-fail-fast -p dla-cli` (the whole dla-cli package: its tests cover gates,
   budgets and workers) and `cargo test -j 2 -p dla-core --lib` pass. Report the counts.
5. Do NOT run the whole workspace suite; the orchestrator runs it.

## Report (final message; read by the orchestrator)

Two sentences of result first. Then per item: files changed; each test with its mutation; existing
tests or golden files changed and why; anything in this brief that was wrong; what you did not
verify. Counts only, no command output.
