# Brief W: a Python registration table fed by the driver's own new keys is not "reproduced through the public API"

Repository: /home/user/dla (Rust workspace, branch `main`, clean working tree at the start).
Scratch directory: /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u85.

## What happened (a real unattended run on a Python library)

The run confirmed eleven findings. One is the documented leak. Ten are module-level tables such
as an extension table: a dict at module level, one public function `register_x(key, value)` that
inserts, and plugin modules that call a plural wrapper at import time:

    # pkg/core.py
    EXTENSION = {}
    def register_extension(id, extension):
        EXTENSION[extension.lower()] = id
    def register_extensions(id, extensions):
        for extension in extensions:
            register_extension(id, extension)

    # pkg/plugin_a.py (module top level)
    from . import core
    core.register_extensions("A", [".a", ".aa"])

The agent's driver called `register_extensions` with a new extension string per unit, the dict
grew by one per unit, the warm-up sibling was refuted, and the finding was confirmed with the
report line "Trigger: reproduced through the public API in H6 (varies: the file extension string
passed to register_extensions, a different one per unit)". That growth is by design: the table
grows only when a caller registers a new key, and the project itself registers constants at
start-up.

The tool already has the right words for this case, built for another runtime:
`Trigger::Direct` in `crates/dla-core/src/trigger.rs`, with the report line "grown only by
calling the registration function itself (<writer>) in <H> (varies: ...); a leak only if run-time
input reaches that call (the census reads its writers as start-up registrations)". It needs two
facts that were missing here:

1. The census fact `startup` on the writer site (`SiteFact` in `crates/dla-core/src/census.rs`:
   "The site runs only at start-up: it is in an initialiser, or every caller of its function (up
   to three calls up) is one"). The run's fact for the writer was
   `{"site": ".../core.py:N", "kind": "insert", "depth": 0, "function": "register_extension", "param": true, "exported": true}`:
   no `startup`. The only project callers are the wrapper (one call up) and, two calls up, module
   top-level statements in other files, written as `core.register_extensions(...)` or
   `Image.register_extensions(...)` (a call through a module alias).
2. "The unit calls the writer itself" (`direct` in `trigger.rs`, set near line 414): the unit
   called the wrapper `register_extensions`, not the writer `register_extension`.

## First: reproduce and report what is true today

Build the small package above as a test fixture (two plugin modules, one of them using
`from pkg import core as c` and `c.register_extensions(...)`, the other `from . import core`).
Run `dla census --lang python` on it and read the writer site's facts. Then find in the Python
census code (crates/dla-census) how `startup` and `init` are computed for Python and why it is not
set here (the qualified call in another file, the exported function, the intermediate wrapper, or
something else). Write the cause in your report in two sentences before you change anything.
Also read how `startup` is decided for Go for an exported registration function whose callers are
all `init` functions, and mirror that decision (the Go rule is the model, not a new rule).

## The change

a. Python census: the writer site is `startup` when every call site of its function that the
   census can find in the project is at module top level, directly or through up to three calls
   (the existing bound), ALSO when the call is qualified by a module name or alias
   (`core.f(...)`, `pkg.core.f(...)`, `c.f(...)`) and lies in another file of the project. A call
   from inside a function that is not itself start-up-only (a handler, a method called at run
   time) makes it not start-up. A function with no call site found in the project is NOT
   start-up (nothing is known). Method calls on objects (`self.table.f(...)`) are out of scope:
   keep today's behaviour for them. Be conservative: when a name could be another function of
   the same name in an unrelated module that the file does not import, do not count it.
b. The chain. When `startup` came through wrappers, the site fact records the functions on the
   way (a new optional field, for example `startup_via: ["register_extensions"]`, empty for a
   direct start-up call). Regenerate the schema files the way the repository does
   (`dla schema` has a check mode; find the write mode) if a schema changes.
c. Trigger: the unit of an entry-path harness "calls the registration function itself" when it
   calls the writer function OR a function in `startup_via` of a start-up writer site. Then
   `Trigger::Direct` applies as it does today, and its line names the function the unit called.
   Nothing else about `Direct` changes: read what `Direct` does downstream today (the report
   line, the counts "confirmed and strong by trigger", any gate or confirm rule) and state it in
   your report; do not change it.
d. Texts: `skills/detect-leak-anywhere/references/python.md` gets one sentence where registries or
   the trigger are described: a table that the project fills at import time through a
   registration function, and that a driver grows by calling that function (or its wrapper) with
   new keys, is reported as grown by the registration function itself and is a leak only if
   run-time input reaches that call. `docs/VERIFY.md` gets one new row at the end in the style of
   the last rows (left column: "A Python registration table grown by the driver's own keys was
   reported as reproduced through the public API", state `corrected`, the facts above without
   naming the library or its functions: say "an extension table" and "its registration function").
e. A second, separate text item. In the same real run the agent wrapped the run of the Python
   test loop in `timeout 560` and lost nine minutes: the loop needs up to its own time limit
   (1800 s). Add one sentence to the pytest-loop bullet of `references/python.md` and one to the
   place in `references/stages.md` that tells the agent to give a harness run a tool timeout of
   600000 ms: the Python test loop runs up to its time limit of 1800 s, so start
   `dla harness run <H>` for it in the background (or let the tool move it there) and wait for it;
   never put it under the shell's `timeout`. Do NOT touch `skills/detect-leak-anywhere/SKILL.md`
   (size limit). Pin both sentences in `crates/dla-cli/tests/skill_text.rs`.

## Tests (each seen failing under a mutation you apply and revert)

- Census (where the Python census facts are tested today): the fixture above gives
  `startup: true` and `startup_via: ["register_extensions"]` on the writer site; the same fixture
  with one more caller inside a run-time function (`def handle(name): core.register_extension("x", name)`)
  gives no `startup`; a function with no caller gives no `startup`; an unrelated module that
  defines its own `register_extensions` and is not imported by the calling file does not count.
- Trigger (CLI or unit, following the existing tests of `Trigger::Direct`; search the tests for
  "grown only by calling the registration function"): an entry-path harness whose unit calls the
  wrapper gives the `Direct` line naming the wrapper; with the run-time caller present the trigger
  is `reproduced through the public API` as today.
- skill_text pins for item e.

Mutations by a small script that asserts the old text occurs exactly once; see each test fail;
revert. Before you finish, `grep -rIl "MUT_" crates harness skills docs bench agents hooks data`
must print nothing.

## Rules

- No commit, stash, checkout, reset, restore or branch change. Leave the tree dirty.
- Shared `target/` only (`cargo` from /home/user/dla, `-j 2`). No `rm -rf` on variable paths. Do not
  kill processes by name. Do not run `dla consent`. Do not start any model, agent or verifier.
- Run each cargo command on its own, in the foreground, piped through `tail` or `grep`. Do not chain
  a background run to other commands. If an action is denied, stop and report it.
- Keep to this brief. If you find another defect, report it; do not fix it.
- Plain technical English, full sentences, no em dashes. Never name a real third-party project,
  library or its APIs in the repository (the fixture uses its own made-up names; do not use
  `register_extension`, `EXTENSION` or `Image` in the repository: choose other names, for example
  `register_codec`, `CODECS`). No model names, no attribution lines.
- If an existing test or golden file changes, say which and why; do not weaken a test. Static
  acceptance fixtures (`bench/fixtures`) and their answer keys must not be edited; if the change
  makes a fixture's expected output differ, stop and report it.

## Checks before you report

1. `cargo fmt --all --check` clean.
2. `cargo clippy -j 2 --workspace --all-targets -- -D warnings` clean.
3. `cargo build -j 2 -p dla-cli` then `./target/debug/dla schema --check` clean.
4. `cargo test -j 2 --no-fail-fast -p dla-census`, `cargo test -j 2 -p dla-core --lib`,
   `cargo test -j 2 --no-fail-fast -p dla-cli --test skill_text --test trigger --test queue` pass.
5. `cargo build -j 2 -p dla-bench` then
   `./target/debug/dla-bench static --dla-bin ./target/debug/dla --json <your scratch>/static.json`:
   report the pass count and any fixture whose result changed (compare with a run of the same
   command before your change if you can; if not, say so).
6. Do NOT run the whole workspace suite; the orchestrator runs it.

## Report (final message; read by the orchestrator)

Two sentences of result first. Then: the cause you found (two sentences); files changed; each
test with its mutation; what `Direct` does downstream today; existing tests, golden files or
fixtures affected and why; anything in this brief that was wrong; what you did not verify. Counts
only, no command output.
