# Builder G: growth measured over dumps is a lead; the Node agent snapshots before it reads its WeakRefs

Repository: /home/user/dla (Rust workspace, plugin `dla`). Work in the working tree. Do NOT commit,
push, or switch branches. Another builder's finished change is uncommitted in the tree
(`crates/dla-cli/src/alloc/judge.rs`, `crates/dla-cli/src/alloc/pick.rs`,
`crates/dla-cli/src/alloc_cmd.rs`, `docs/VERIFY.md`,
`skills/detect-leak-anywhere/references/evidence-and-report.md`): leave its lines alone. You may
add rows to `docs/VERIFY.md`.

## What a real run showed (the evidence; do not name the project, its types or its files anywhere in the repository)

A run on a real Node library measured its documented leak and then lost it, in two ways.

1. `dla summarize heap-graph` over four heap snapshots of harness H1 wrote a dump-axis growth series
   (`series/S4.json`, `"axis": "dump"`, source `{"kind": "dump", "command": "dla summarize heap-graph"}`).
   It recorded:
   - a `growth-series` evidence record (authority `dump`, gate `pass`) whose `passed` columns
     `holder:field:<module>.<Class>.request` etc. carry `"finding": "F130"`;
   - `discriminator` records (authority `dump`, `"favors": "leak"`, `"finding": "F130"`,
     `"reason": "retained bytes and count grew over every interval"`, `"series": "S4"`).

   The findings F129–F131 were created from it (origin `growth-series`). The summary's `needed`
   said "a refuted non-leak sibling and a holder, then dla ledger confirm F130 --evidence E20".
   Then `dla ledger close-rest` closed all three as `unresolved`, with the remedy
   "tier 5 (not ranked for measurement), not targeted". Later captures tied the same growth to the
   same findings again (S30, S35), but neither the gate nor `run status` ever brought them back.

   Cause: `dla_core::grown::grown_unmeasured` (crates/dla-core/src/grown.rs) reads only
   `view.trends`, the records of `dla trend`. A dump-axis series that a summarizer wrote has no
   trend, so its growth never counts as "grown, undecided". `close-rest`
   (crates/dla-cli/src/ledger_queue.rs), the refusal of `ledger resolve`
   (`refuse_grown_unmeasured` in crates/dla-cli/src/ledger_cmd.rs) and the gate (`grown::items`
   and the open-finding items built from `Grown`) all read that one list. Every dump summarizer
   prints the same confirm route: grep `a refuted non-leak sibling and a holder` (node_heapsnapshot.rs,
   histogram.rs, alloc_stack.rs, vgdb.rs, shim_groups.rs). So this is not specific to Node.

2. The Node agent (`harness/node/dla-preload.cjs`, function `build`) does, in one synchronous job:
   forced gc, `sweepListenerRefs()` (calls `deref()` on every tracked emitter's WeakRef),
   `countListeners()` (`deref()` again, and on listener WeakRefs), `runProbes`, and only then
   `captureNow` → `captureHeap` → `v8.writeHeapSnapshot`. Per the WeakRef spec, a `deref()` that
   returns a target keeps it on the job's kept list, so in the snapshot every live tracked emitter
   has an edge from `(Strong root list)` named `weak_refs_keep_during_job`. That edge keeps
   nothing alive that was not alive anyway: our summarizer skips it (crates/dla-parsers/src/node/holders.rs,
   `JOB_KEEP_EDGE`) and found only 1 object reachable by it alone. But the model wrote its own
   shortest-path script. Every leaked object's shortest path went through that edge, so it called
   the growth "the measuring agent's artifact". The driver (`harness/node/dla-harness.cjs`) already
   awaits `setImmediate` before `dla.checkpoint()`, so at the start of `build` the kept list is
   empty. Only the agent's own reads in `build` fill it.

## Fix A: growth measured over dumps is a lead to decide, like a passed trend column

In `grown_unmeasured`, also walk the run's evidence for growth that a dump series measured. The
outcome must be the same as for a passed trend column. The finding is listed as grown and
undecided, so:

- close-rest leaves it open, with the commands, in the same `left_open` form;
- `ledger resolve --as unresolved|suspected_static_only` refuses it the way it refuses a grown
  trend column;
- the gate lists it as a blocking item while it is open, and also when it is already closed
  `unresolved` or `suspected_static_only`. That covers a run whose older close-rest or resolve
  closed it, as in the evidence above.

Rules to implement (keep them in one place in grown.rs, documented):

- **What counts.** A growth-series record with authority `dump` (use the existing predicate
  `crate::machine::is_passing_growth` if it fits) whose gate passed, with a passed column that names
  a finding (`finding`, else `candidates`, the same lookup the trend path uses). Also count a
  `discriminator` record with authority `dump`, `favors: leak` and a `finding`. Only leak-polarity
  roots count. They must be open, `unresolved` or `suspected_static_only`, and neither merged nor a
  sibling: the same filter as the trend path.
- **What lifts it.** The newest decisive record about the root's family decides: a dump
  discriminator naming it, or an alloc-test bound to it. If that record does not favor leak (a
  newer dump discriminator that favors non-leak or is inconclusive, or a newer alloc-test that
  favors non-leak), the lead goes away. Check how `alloc_tested` and `bound_alloc_tests` already
  do this and reuse them. Do not invent a new ordering: evidence order in the ledger is time order.
- **Harness.** The harness the dump series belongs to. A summarizer-made series has no harness in
  its file. Take it from the capture files its dumps came from (`harness/<H>/...` in the
  `source.artifact.path` of the per-dump records of the same summarize call), with
  `measured_holders::harness_of_path` or whatever the code already has. If no harness is known,
  still list the lead, with commands that do not need one.
- **Commands.** The dump route, not an alloc-test: the summarizer already says what the ledger
  accepts. Use `confirm_commands(ledger, root_f, harness, &discriminator_evidence)`. It prints the
  sibling add and refute when needed, the `test` with the harness's workload evidence when the
  state allows it (an `unresolved` finding can go `under_test`), the annotations it lacks, and the
  `confirm` with the growth evidence. Each printed command must be accepted by the ledger as
  printed, in order, in a real scene. Prove that in a test that runs them through the CLI.
- **Text.** The gate item text for a closed one must not say "no alloc-test decided it" when the
  lead is a dump. Say what decided it: "its holder column X of S4 grew over every dump (a dump
  series of dla summarize heap-graph) and nothing decided it since", or similar plain words. Keep
  the existing text for trend columns.
- **No double report.** A finding that is already in the list through a trend column, or as a
  Python `measured_holders` holder, must not get two items.

Tests (crates/dla-core unit tests in grown.rs, and a CLI test in crates/dla-cli/tests/, near
`close_rest_*` in queue.rs, second_look.rs or py_site_columns.rs). Build the scene the way those
files do, with neutral names (`app.Conn.request`, `src/conn.js`):

1. A finding tied to a passing dump growth with a leak discriminator: close-rest leaves it open,
   and the reason names the dump series.
2. The same finding closed `unresolved` beforehand, by a `ledger resolve` done before the
   evidence or written into the ledger directly: the gate (`dla gate check --final` or
   `dla run status`) lists it, and running the printed commands in order ends with `confirmed`.
3. A newer dump discriminator that favors non-leak, or a newer alloc-test bound to it that
   favors non-leak: no lead, and close-rest closes it.
4. A dump growth of a non-leak sibling, or of a refuted or confirmed root: no lead.
5. A finding with both a trend column and a dump growth: one item.

Mutation check: make the new walk return nothing, see tests 1 and 2 fail, then restore it.

## Fix B: the Node agent takes its captures before it reads any WeakRef in the job

In `build` (harness/node/dla-preload.cjs), take the captures that are due at this record (the
allocation profile first, then the heap snapshot, as now) right after the forced collection, and
before `sweepListenerRefs`, `countListeners`, `runProbes` and anything else that calls `deref()`.
The measured values and their order of gc, sweep, extra gc and reading must stay exactly as they
are. Only the capture moves. `captureHeap` needs `heapUsed` for its preflight: read it at that
moment (`memValues()` or `process.memoryUsage()`). The non-heap resources capture at the final
record may stay where it is if it reads nothing weak; check `captureResources`. The `captures`
array must still land in `rec.captures`, with the same entries and order. Command mode (`exit`
phase) gets the same order. There is no yield there, so say so in a comment and leave it.

Also look for other `deref()` calls that run in `build` before the snapshot (probes walking
WeakRefs, line ~817), and list them in your report.

Test: a Node capture test in crates/dla-workload/tests/harness_capture_node.rs or
crates/dla-cli/tests/harness_capture_node.rs, whichever already runs the agent with
`DLA_CAPTURE=heap` (follow its pattern and its node guard). Its unit adds an EventEmitter with a
listener to a module-level Map once per unit, so the emitters are alive and tracked by the agent.
Parse the written `heap-<n>.heapsnapshot` with the Rust snapshot reader in crates/dla-parsers/src/node
(find the `JOB_KEEP_EDGE` helpers, such as `job_keep_edge_present` and `job_keep_only`, or add a
small one that counts the targets of that edge). Assert that the edge has no target that is an
EventEmitter, or no targets at all. Mutation check: put the capture back after `countListeners`
and see the test fail. Say in the report that you did it.

## Docs

- docs/VERIFY.md: one row per fix, in the style of the rows near the end. Give the claim, how it
  was checked, and what is not covered. Use neutral words: "a real Node library run", never the
  project.
- The skill reference that says which findings close-rest leaves open, or which leads the gate
  lists. Find it with grep in skills/detect-leak-anywhere/references/ (for example "its column
  grew" or "close-rest"). Add the dump case in one sentence. Do not touch SKILL.md: its byte budget
  is pinned in a test.
- No project names, no evidence project types or paths, no attribution lines, no model names
  anywhere in the repository.

## Rules

- Shared `target/` only. Run `cargo test -p dla-core`, `cargo test -p dla-cli` (the test files you
  touched and their neighbours; the whole crate if it fits in time),
  `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all --check`. For
  dla-workload, run the Node capture tests.
- No `rm -rf "$VAR/..."`. Delete only literal paths. Never kill processes by name.
- No benchmarks (`dla-bench dynamic` / `static`).
- Never skip, disable or weaken a test.

## Report (your final message)

List the files changed. Give the rule as implemented (what counts, what lifts it, the harness, the
commands, the texts), the tests added with what each proves, the mutation checks and their
results, the commands run with pass or fail counts, the other `deref()` sites in `build`, and
anything you did not do or are unsure of.
