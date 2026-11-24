# Brief N: a Node project that is built before it runs (TypeScript)

Repository: /home/user/dla (Rust workspace, branch `main`). The working tree holds uncommitted
changes of two other builders (Go census; Python holders, alloc-test rules, gate). Leave what
you do not need alone; where you must edit a file they changed (`crates/dla-core/src/gate.rs`),
edit only your own lines.
Scratch directory: /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u114
(create it; write nothing else outside the repository).

## What happened (a real unattended run on a Node library the tool had never seen)

The library is written in TypeScript under `src/`. Its `package.json` has
`"main": "./lib/node/index.js"`, a `module`, an `exports` map and `"files": ["lib", ...]`. `lib/`
is build output (bundled chunks such as `lib/node/chunk-AB12.js`, each ending in
`//# sourceMappingURL=chunk-AB12.js.map`; the maps list `../../src/...ts` sources and carry
`sourcesContent`). `.gitignore` lists `lib` and `node_modules`. On the developer's machine both
exist.

1. `dla harness gen --lang node --driver` copies the project into `harness/<H>/work` from the
   git file list (`crates/dla-workload/src/harness/scratch.rs`, `copy_project_for`). `lib/` and
   `node_modules/` are ignored by git, so the copy has neither, and a driver's
   `projectRequire('lib/node/index.js')` fails with "Cannot find module". The model worked
   around it in every one of 86 drivers by copying `lib` into the work copy at run time and
   requiring dependencies by absolute path.
2. Every site the Node agent reported (`node.listeners@lib/node/chunk-AB12.js:1076` and so on)
   and every coverage answer was a position in the built bundle. The candidates of the census
   are lines of `src/*.ts`. Nothing tied: 124 of 172 targets read "not reached", growth of
   several listener counts per request was never attributed, and the run ended with "no leak
   confirmed". With a driver that sends one mocked request per unit, the listener columns at
   the built lines grow by 1 to 4 per unit and the heap by about 29 KB per unit: the leak is
   measured, only at positions that no finding has.
3. The census also listed declaration files of the build output as holders
   (`lib.node.interceptors.X.index.d.SomeClass.field`).
4. The heap residual was above its threshold on 178 of 370 rows. The model resolved the open
   items as unresolved with the remedy "capture alloc profile then summarize". It had run one
   capture on the first, generic harness, had not run the `dla summarize heap-graph` command the
   capture prints, and ran no capture on the harnesses that showed the growth.

## The change, in this order (report after each part what works; stop and report if a part
turns out much larger than it looks)

### 1. The work copy can load the built package

- When the project is copied from the git file list and its `package.json` (the root one, and
  the one of each workspace package the copy holds) names entry points (`main`, `module`,
  `bin`, string values anywhere in `exports`, the directories of `files`) that lie in a
  directory git ignores, that directory is copied too. Record the count in the copy report
  (how the Python build extensions are recorded is the model: `extensions_kept`) and say it in
  the `harness gen` output.
- `node_modules`: when the project has one beside a `package.json` the copy holds, the work
  copy gets a symbolic link at the same relative path to the project's directory (a link, not
  a copy; say what happens on Windows and keep the code compiling there). Check that the
  thinning of older work copies (`thin_files`, same file) leaves the link alone and treats the
  copied build output like any other file.
- Nothing changes for a project without such entry points.

### 2. The census reads sources, not their build output

- Declaration files (`*.d.ts`, `*.d.mts`, `*.d.cts`) hold no holders.
- A JavaScript file that ends with a `sourceMappingURL` comment whose map lists at least one
  source file that exists in the project is build output: the census skips it (its sources are
  read instead). Say how many candidates the blind project's shape loses and gains (build a
  small copy of that shape in your scratch directory: two `.ts` files, a bundled `.js` with a
  map, a `.d.ts`).

### 3. Sites and coverage at the original source position

- `harness/node/dla-preload.cjs` (and the driver agent if it derives sites): a site in a file
  that has a source map is reported at the original position when the map's source resolves to
  a file of the project (relative to the project root, forward slashes): `src/a/b.ts:139`
  instead of `lib/node/chunk-AB12.js:1076`. Node has this built in:
  `process.setSourceMapsEnabled(true)` and `require('node:module').findSourceMap(file)` with
  `findEntry(line, column)`; use it, add no dependency, and fall back to the built position
  when there is no map or the entry is missing. Keep the built position in the record where
  the format has room, and say where.
- Every per-site column and group of the Node agent (listeners, timers, resources, the heap
  profile frames of `--capture alloc`, the holders of the heap snapshot summary where they
  carry a file position) and the coverage answers that `dla trend` reads for targets
  (`executed`, `not reached`) must use the same mapping, so that a target at a `.ts` line is
  "executed" when its function ran in the bundle. Find where each is produced (agent or Rust)
  and map there. If a map must be read on the Rust side, write the small decoder yourself.
- A project of plain JavaScript must give byte-identical records to today's.

### 4. A residual above threshold needs a summarized capture before it can be left

- `dla gate check` (classify and final) and `dla ledger resolve <O> --as unresolved` for the
  open item of a residual above its threshold: while the harness of that series has no
  summarized capture (no retention or allocation-stack evidence whose source lies under that
  harness) and the runtime supports `--capture`, the gate lists one item per harness with the
  fix `dla harness run <H> --mode <mode> --capture alloc,heap` and "then the `dla summarize`
  commands it prints", and the resolve is refused with a new code and the same fix. Read how
  the Python capture item of the other builder works in `gate.rs` (`measured_holder`, the
  capture item) and use one mechanism for both if that is natural; do not break its tests.

## Tests (each seen failing under a mutation you apply and revert)

- `scratch.rs` unit tests: ignored entry-point directory copied, `node_modules` linked, a
  project without entry points unchanged, thinning keeps the link.
- Census unit tests for part 2.
- CLI tests with real Node runs, guarded like the existing Node tests (find the helper): a
  small TypeScript-shaped project (a `src/reg.ts` kept only as source text, a hand-written
  bundle `lib/index.js` with a hand-written or generated source map that maps the registering
  line, `.gitignore` with `lib`): the driver loads `lib/index.js` from the work copy with no
  workaround; a listener or registry column of a unit that leaks is reported at
  `src/reg.ts:<line>`; a candidate at that line ties to it and reads "executed"/"grew"; the
  same project without the map reports the built position.
- Part 4: gate item and refusal, then gone after the capture's summaries.
- Existing tests that change: say which and why. Do not weaken a test. Nothing under
  `bench/fixtures` is edited; if a Node fixture's records change, stop and report it.

## Rules

- No `git commit`, `git stash`, `git checkout` (in any form), `git reset`, `git restore` or
  branch change. Leave the tree dirty.
- Shared `target/` only (`cargo` from /home/user/dla, `-j 3`). No `rm -rf` on variable paths. Do
  not kill processes by name. Do not run `dla consent`. Do not start any model, agent or
  verifier. No package installs (`npm install` included): write the small project by hand.
- Run each cargo command on its own, in the foreground, piped through `tail` or `grep`. If an
  action is denied, stop that action and report it; do not ask anyone to do it for you and do
  not try another way to the same end.
- Keep to this brief. If you find another defect, report it; do not fix it.
- Plain technical English, full sentences, no em dashes. Never name a real third-party project,
  library or its APIs in the repository. No model names, no attribution lines.
- Mutations by a small script that asserts the old text occurs exactly once. Before you finish,
  `grep -rIl "MUT_" crates harness skills docs bench agents hooks data` must print nothing.
- Texts: `skills/detect-leak-anywhere/references/node.md` (or the file that holds the Node
  notes: find it) and `harness/node/README.md`: the four changes in a few sentences, pinned in
  `crates/dla-cli/tests/skill_text.rs`; `docs/VERIFY.md`: rows at the end, state `corrected`,
  "a Node library written in TypeScript that the tool had not been run on". Do NOT touch
  `skills/detect-leak-anywhere/SKILL.md`.

## Checks before you report

`cargo fmt --all --check`; `cargo clippy -j 3 --workspace --all-targets -- -D warnings`;
`cargo build -j 3 -p dla-cli` and `./target/debug/dla schema --check`;
`cargo test -j 3 -p dla-workload --lib`, `cargo test -j 3 -p dla-census`,
`cargo test -j 3 -p dla-core --lib`, `cargo test -j 3 -p dla-cli --bin dla`, every Node test
file of dla-cli and dla-workload (list them), `--test gate_flow --test queue --test skill_text
--test holder_first --test work_thin`, and the tests you added.

## Report (final message; read by the orchestrator)

Two sentences of result first. Then per part: files changed, what works, one real output
(first lines); where each site and each coverage answer is mapped; each test with its
mutation; existing tests changed and why; anything in this brief that was wrong; what you did
not verify. Counts only.
