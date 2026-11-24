# Brief X: unexplained heap growth leads the run, and a Node heap summary shows who really keeps the objects

Repository: /home/user/dla (Rust workspace, branch `main`). The working tree holds uncommitted
changes of earlier builders. Leave them as they are; where you must edit one of those files, add
your own lines only. You are the only builder in the tree.
Scratch directory: /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u123
(create it; write nothing else outside the repository).

The machine has 4 cores. Use `cargo ... -j 3`, one cargo command at a time. Never touch
/root/dla-real.

## What happened (a real unattended run on a Node library written in TypeScript)

1. The FIRST harness measured the documented leak: `node.heap_used` rose by about 15 KB per
   unit, steadily, and eight live-listener columns grew at lines of the sources. `dla trend S1`
   printed, as `next`, the missed targets, then the three commands that test the first tied
   finding. Nothing said that the heap itself grows and that nothing explains it.
2. The gate does have the right item: after `dla gap S1 --residual` it lists, first,
   `residual_capture` ("1 heap residual row of S1 over the threshold and H1 has no summarized
   capture", fix `dla harness run H1 --mode fresh --capture alloc,heap`, then the summaries).
   But the residual row is written only by `dla gap --residual`, a command the model never ran
   (1,412 gate items waited on it). So the capture item never appeared, the stop hook kept
   naming the queue first, the model worked through 79 queue harnesses and stopped with
   nothing confirmed and no capture taken.
3. Replayed by hand on that library (one mocked request per unit): the capture run takes 44 s
   and its summaries pass their gates. `dla summarize heap-graph` names six holders with
   5,940 instances each: `MockHttpSocket.<symbol dla.listeners>`, two closure variables of
   the request handler, and stream objects of the runtime. Every one of their paths is
   `(GC roots) -> (Strong root list) -> .weak_refs_keep_during_job -> <object>`. That root is
   V8's list of the objects a `WeakRef` was read from in the current job: the tool's own
   agent reads its `WeakRef`s when it counts live listeners, just before the snapshot. So
   the summary shows the agent's own keep-alive as the path, hides who really keeps the
   objects, and names the agent's own record (`<symbol dla.listeners>`) as the top holder.
   45 of the 200 holders of the summary have such a path; all six growing ones do.
4. The model also wrote in its report that a per-site listener column "counts calls to a
   line, not live listeners". That is wrong (the agent counts the listeners that are still
   attached to objects that are still alive, after a forced collection), and the reference
   text invites it: it says such a column "counts what one line made".

## The change

### 1. `dla trend` writes the residual rows of its series

`dla trend <S>` ends by writing the residual rows of that series, as `dla gap <S> --residual`
does (find the function behind `dla gap --residual` in `crates/dla-cli/src` and call it; do
not copy it). When that is not possible for a series (say when and why), nothing changes.
`dla gap --residual` and `--all` keep working and find the rows already written. The output of
`dla trend` gains, when a heap row is over its threshold and no summarized capture of that
harness exists: a first `next` entry and an `_attention` entry with the capture command (keep
`_attention` entries short: they are cut at 160 characters when the output is over its bound).

### 2. The capture comes before the next batch

While a `residual_capture` item is open (`crates/dla-core/src/gate.rs`,
`uncaptured_residuals`): `dla harness gen --next` is refused with a new code,
`capture-first`, whose fix is the capture command and "then the `dla summarize` commands it
prints". Model it on the existing refusal `measured-holder-first`
(`crates/dla-cli/src/harness_cmd.rs`, `crates/dla-core/src/error.rs`); a harness generated
with `--probe` or `--entry` for a named finding is not refused. Check the order of the first
blocking items in `dla gate check --stage classify`, `dla gate check --final`, `dla run
status` and the stop hook's text (`hooks/`): `residual_capture` comes before the queue item in
all four. One capture per run is enough to lift the refusal for every harness whose rows are
over the threshold only if you can show that their growth is the same growth; if you cannot,
keep it per harness and say so.

### 3. A Node heap summary does not walk through the agent

In the heap snapshot reader (`crates/dla-parsers/src/node/snapshot.rs`, `graph.rs`,
`holders.rs`):

- The edge `weak_refs_keep_during_job` of the strong root list is not a retaining edge: skip
  it when paths and retained sizes are computed, as weak edges are skipped. An object that
  nothing else reaches is then unreachable and leaves the summary; say in `notes` how many
  objects that was.
- Nothing of the agent is a holder: the statics of the preload script (`listenerRefs`,
  `mapsByFile` and the like: every script under the run directory's `agent/`), and a field
  whose name is one of the agent's own symbols (`<symbol dla.listeners>` and the others the
  preload defines: list them from `harness/node/dla-preload.cjs`). What such a field keeps is
  counted for the object that owns it, under that object's real holder.
- Then look at what the summary says on the real case. A scratch project with the filled
  driver is at
  /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u122/proj
  (run `20261124-195747-5c30`, harness H1; copy the directory to your scratch directory
  first and work on the copy; the environment is `DLA_CONSENT_TIERS=1-4
  DLA_CONSENT_RAW_VALUES=1 DLA_PLUGIN_ROOT=/home/user/dla`; `dla harness run H1 --mode fresh
  --capture alloc,heap` takes under a minute and writes about 600 MB: delete the capture
  directory of your copy when you are done). Report the first eight holders and their paths
  before and after your change, and whether the growing objects now have a path that names
  who keeps them. The earlier output is in
  /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u122/keep.
- If the snapshot is taken in the same job in which the agent read its `WeakRef`s, and moving
  the snapshot before that reading is a small change in `harness/node/dla-preload.cjs`, do
  that as well, and say which of the two changes the test below needs.

### 4. Texts

- The sentence on per-site count columns in
  `skills/detect-leak-anywhere/references/evidence-and-report.md` and `references/stages.md`
  ("counts what one line made"): say what each counts. For `node.listeners@<file:line>`: the
  listeners registered at that line that are still attached to objects that are still alive,
  read after a forced collection; a rising value means those objects are kept. Check the
  other per-site columns the sentence names in their agents and describe each truthfully.
- `references/node.md` and `references/stages.md`: the three changes above in a few
  sentences. Pinned in `crates/dla-cli/tests/skill_text.rs`. `docs/VERIFY.md`: rows at the
  end, state `corrected`, "a Node library written in TypeScript that the tool had not been
  run on". Do NOT touch `skills/detect-leak-anywhere/SKILL.md`.

## Tests (each seen failing under a mutation you apply and revert)

- Part 1: a series with heap growth and no tied finding: after `dla trend` the residual row
  exists and the first item of `dla gate check --stage classify` is `residual_capture`; a
  series under the threshold gets a row and no item. Runtimes: at least Node and Python (use
  the scenes of `crates/dla-cli/tests/residual_capture.rs`).
- Part 2: `dla harness gen --next` is refused with `capture-first` while the item is open and
  works after the summaries; `--probe` is not refused.
- Part 3: reader tests on a small hand-made snapshot: an object reached only through
  `weak_refs_keep_during_job` is not in the summary; an object reached through it AND through
  a module-level `Map` is listed under the `Map` with that path; an agent static and an
  agent symbol field are not holders. One CLI test with a real Node run, guarded like the
  existing Node tests: a small module whose `register()` keeps objects in a module-level
  array and attaches a listener to each; the heap-graph summary names the array, and no path
  contains `weak_refs_keep_during_job`.
- Existing tests that change: say which and why. Do not weaken a test. Nothing under
  `bench/fixtures` is edited. Run `dla-bench dynamic --only <name>` (one name per call) for
  three Node fixtures (two leaks, one non-leak) and two Python fixtures and report each
  result; if one changes, stop and report it.

## Rules

- No `git commit`, `git stash`, `git checkout` (in any form), `git reset`, `git restore` or
  branch change. Leave the tree dirty.
- Shared `target/` only (`cargo` from /home/user/dla). No `rm -rf` on variable paths. Do not
  kill processes by name. Do not run `dla consent`. Do not start any model, agent or verifier.
  No package installs.
- Run each cargo command on its own, in the foreground, piped through `tail` or `grep`. If an
  action is denied, stop that action and report it; do not ask anyone to do it for you and do
  not try another way to the same end.
- Keep to this brief. If you find another defect, report it; do not fix it.
- Plain technical English, full sentences, no em dashes. Never name a real third-party project,
  library or its APIs in the repository (the names above are for you; in tests and texts use
  neutral names). No model names, no attribution lines.
- Mutations by a small script that asserts the old text occurs exactly once. Before you finish,
  `grep -rIl "MUT_" crates harness skills docs bench agents hooks data` must print nothing.
- If the schema of a record changes, update `schemas/` and run `./target/debug/dla schema
  --check`.

## Checks before you report

`cargo fmt --all --check`; `cargo clippy -j 3 --workspace --all-targets -- -D warnings`;
`cargo build -j 3 -p dla-cli -p dla-bench` and `./target/debug/dla schema --check`;
`cargo test -j 3 -p dla-parsers`, `cargo test -j 3 -p dla-core --lib`,
`cargo test -j 3 -p dla-cli --bin dla`, the Node test files of dla-cli (list them),
`--test gate_flow --test queue --test skill_text --test holder_first --test residual_capture`,
and the tests you added.

## Report (final message; read by the orchestrator)

Two sentences of result first. Then per part: files changed, what works, one real output
(first lines); the holders and paths of the real case before and after; each test with its
mutation; the five fixture results; existing tests changed and why; anything in this brief
that was wrong; what you did not verify. Counts only.
