# Builder H: the Node heap-graph output shows how each growing holder is kept, and the project line of the callback that keeps it

Repository: /home/user/dla (Rust workspace, plugin `dla`). Work in the working tree. Do NOT commit,
push, or switch branches. Two other changes are in the tree, uncommitted:
- builder F's, finished: `crates/dla-cli/src/alloc/*`, `crates/dla-cli/src/alloc_cmd.rs`,
  `docs/VERIFY.md`, `skills/detect-leak-anywhere/references/evidence-and-report.md`;
- builder G's, in progress now: `crates/dla-core/src/grown.rs`, `crates/dla-cli/src/ledger_queue.rs`,
  `crates/dla-cli/src/ledger_cmd.rs`, `harness/node/dla-preload.cjs`, Node capture tests, `docs/VERIFY.md`.
Do not edit those files, except to add rows at the end of `docs/VERIFY.md`. Re-read the file just
before you edit it, because G may have added rows.

## What a real run showed (do not name the project, its classes or its files anywhere in the repository)

A Node library leaked about 20 KB per request: every request made an object of a project class
(call it `Conn`). Native code kept two HTTP parsers per request, and each parser kept a bound
callback whose `this` was the `Conn`. `dla summarize heap-graph` over four snapshots found the
growth and tied the holders `Conn.request`, `Conn.originalSocket` and `Conn.responseStream`
(counts 22k → 200k, discriminators favor leak). Its per-snapshot summary file
(`summaries/heap-<n>-<sha>.json`, `holders[].path`) even had the right path:

    (GC roots) [10] > (Global handles) .2 > HTTPParser [1] > native_bind .bound_this > Conn .request

But the JSON the command prints, which is what the model reads, has no path. It lists `passed`
holder names, `discriminators`, `dumps` with `top_holder`, `candidates`, `tied` and `needed`. The
model cut the output at 3,500 characters, wrote its own shortest-path script and was misled by
another problem (being fixed elsewhere). It never named the mechanism: a native handle that keeps
a parser, which keeps a bound callback of the project, which keeps the object.

## The change

1. **The path in the command output.** In `dla summarize heap-graph` (crates/dla-cli/src/summarize/node_heapsnapshot.rs,
   `heap_graph`), for each holder that passed (at most 3, largest first), print a compact retention
   path from the root to the holder. Use the `path` the parser already computes
   (crates/dla-parsers/src/node/holders.rs). For example, under a new key `kept_by`:
   `{"field:<module>.Conn.request": "(Global handles) > HTTPParser[1] > bound function (this) > Conn.request"}`.
   Keep it short: drop the synthetic `""` and `(GC roots)` steps, collapse runs of `system / Context`
   and array steps into `...`, cap each path at about 160 characters, and give readable edge names
   (`bound_this` → `bound function (this)`, `[n]` kept). Respect the output and result bounds (the
   evidence result is held to 2048 bytes, crates/dla-cli/src/summarize/emit.rs). If the paths do
   not fit, put them in the printed output only, not in the evidence result, or cut to 1 path. Say
   which you did.

2. **The root, in words.** For each printed path, add what the root is, from the first non-synthetic
   step: a native handle (`(Global handles)`, `(Strong roots)` other than the job-keep list, a
   handle scope), a module variable or global of the project, a finalization registry, or a
   library's global. If the root is a native handle and the path goes through a bound function or a
   closure of a project script before it reaches the holder, add the project line of that function:
   - the bound function's target function (edge `bound_function`) or the closure's own location;
   - the snapshot's `locations` table (`Snapshot::locations()` in crates/dla-parsers/src/node/snapshot.rs);
   - its script id mapped to a project file through the `scripts.json` sidecar the agent writes next
     to the snapshots, and through source maps when the script is a bundle (see how `module_of` and
     crates/dla-parsers/src/node/sourcemap.rs are used today).

   The text, for example: `"root": "a native handle keeps an HTTPParser, which keeps a bound function created at src/conn.ts:119 whose this is the Conn"`.
   This line is what a finding's `--adds` and `ledger explain` need. When no location maps, say so
   and give no line. Never guess one.

3. **`needed`.** When the top holder's root is a native handle, the `needed` text says so in one
   short sentence before what it already says, for example: "native code keeps the objects (a handle
   that is not released); the line that hands them to it is src/conn.ts:119; then ...". Keep the
   confirm command it prints exactly as it is now.

4. **The reference.** In skills/detect-leak-anywhere/references/node.md (or wherever heap-graph
   output is explained for Node; find it with grep), add 1–3 short sentences: `kept_by` and `root`
   are the retention path the summary used. The edge `weak_refs_keep_during_job` is never a holder
   (it holds what a `WeakRef` was read from in the current job, and the summary skips it). Read the
   path before writing your own analysis. Do not touch SKILL.md: its byte budget is pinned by a test.

## Tests

- Parser and summary unit tests with a small hand-built snapshot (the test helpers in holders.rs
  and node_heapsnapshot.rs build snapshots from `(type, name, size, edges)` tuples):
  - a growing project object kept as `(GC roots) > (Global handles) > NativeParser [1] > native_bind .bound_this > Conn .request`,
    with a `locations` row for the bound function's target in a script that `scripts.json` maps to
    `src/conn.js`: the output's `kept_by` has the short path, and `root` names the native handle
    and `src/conn.js:<line>`;
  - the same with a bundled script and a source map: the line maps to the source file;
  - a module-variable root: `root` says module variable, with no native wording and no line;
  - no locations: no line, and the text says it is not known.
- A CLI test in crates/dla-cli/tests/summarize_node.rs (look at what it already does with real or
  hand-built snapshots). If `node` is available there, the best test is a real one: a small script
  run under the agent with `DLA_CAPTURE=heap`, where a module keeps objects through a native-held
  callback. Find a native holder Node offers without network, for example an HTTP parser from
  `http.createServer` and a client on localhost, or a `fs.watch` or timer handle whose callback is
  bound to a project object. The output must name the line of the `bind` call. If you cannot build
  a real scene that keeps objects through a native handle in a few seconds, say so and keep the
  hand-built tests.
- Mutation check: make `root` never add the line, see the test fail, then restore it.

## Rules

- Shared `target/` only. Run `cargo test -p dla-parsers`, `cargo test -p dla-cli --test summarize_node`
  and the unit tests of the files you touched, `cargo clippy --workspace --all-targets -- -D warnings`
  and `cargo fmt --all --check`. If another builder's half-done edit breaks the workspace build,
  wait a few minutes and retry. Do not fix their files.
- `docs/VERIFY.md`: one row in the style of the last rows (the claim, how it was checked, what is
  not covered), with neutral words.
- No `rm -rf "$VAR/..."`. Delete only literal paths. Never kill processes by name. No benchmarks.
  Never skip or weaken a test. No project names, no attribution lines, no model names in the
  repository.

## Report (final message)

List the files changed. Give the output shape (keys and an example), the root words and when a
line is given, the tests with what each proves, the mutation check and its result, the commands
with pass or fail counts, and what you did not do or are unsure of.
