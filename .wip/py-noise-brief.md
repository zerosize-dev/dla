# Brief N: two sources of false "growing line" items in Python runs

Repository: /home/user/dla (Rust workspace, branch `main`, clean working tree at the start).
Scratch directory: /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u88.
The `target/` directory is empty: your first cargo command rebuilds everything (15 to 20 minutes).

## What happened (a real unattended run on a Python library)

A Python driver run counts, at every checkpoint, the live traced blocks per project source line
(columns `py.blocks@<file:line>`, written by `harness/python/sitecustomize.py`, see
`_blocks_checkpoint` and the `checkpoint` function near line 1482). The final gate lists every
such column that is not flat and is tied to no finding, with the command that adds a finding at
the line (`classify` in `crates/dla-core/src/gate.rs`, item kind `trend_column`).

The run ended with 189 open items. 186 of them were of that kind, and they came from two causes
that are not leaks of the project.

### Cause 1: the interpreter's own attribute cache (175 of the 186 items)

Four lines accounted for 175 items, repeated over about fifty trends each. All four build an
attribute name at run time and look it up:

    return getattr(self, "chunk_" + cid.decode("ascii"))(pos, length)
    v = getattr(self, "_get" + id)()
    decoder = getattr(core, decoder_name + "_decoder")
    encoder = getattr(core, encoder_name + "_encoder")

CPython keeps a reference to the name string of an attribute lookup in its type attribute cache
(4096 slots, the slot chosen by the address of the string object). A name built at run time is a
new string object each time, so the cache keeps up to 4096 of them alive. At a checkpoint the
line therefore shows a few dozen to a few hundred live blocks, the number wanders and creeps
up, and `dla trend` reads it as `stair`, `plateau` or sometimes `steady`.

Measured on Python 3.12 with a ten-line script (a class with two methods, a loop that calls
`getattr(a, "chunk_" + s)()`, two collections, then the live blocks at that line): 15, 30, 169,
206 after four rounds of 2000 calls; 0 right after `sys._clear_type_cache()`; 663 after the
next round. So the blocks are held by the cache and by nothing else.

The interpreter has a call for exactly this: `sys._clear_internal_caches()` (Python 3.13 and
later; its documentation says it is for releasing unnecessary references and memory blocks when
hunting for leaks) and before that `sys._clear_type_cache()` (deprecated from 3.13 in favour of
the first).

### Cause 2: a line that writes into the holder of an existing finding (2 items here, 10 and 24 in two earlier runs)

The census records, for a finding whose holder is a container, the lines that write into it
(`Annotation.facts.writer_sites`, each a `SiteFact` with a `site`; `crates/dla-core/src/census.rs`).
A column `py.blocks@<file:line>` ties to a finding only when the finding's own `site` is that
line (`tied_findings` in gate.rs near line 764, and the same comparison in
`crates/dla-core/src/grown.rs` near line 241 and `crates/dla-core/src/trigger.rs` near line 236).
The finding's `site` is the line that defines the container. So when a unit registers a new key
per call, the writer line of the table grows, its column is "tied to no finding", and the gate
asks for a NEW finding at the writer line, although the census finding of that table exists.
The new finding has no census facts, so the rules that depend on them (for example the refusal
`registration-only`) do not apply to it.

## The change

### N1. Empty the interpreter's caches before a checkpoint is counted

In `harness/python/sitecustomize.py`, where a checkpoint collects twice (`gc.collect()` twice,
inside `if not tick:`), call a new small helper right after the two collections and before any
figure is read: it calls `sys._clear_internal_caches()` when the interpreter has it, otherwise
`sys._clear_type_cache()` when it has that, and never raises. If the interpreter has neither,
add one limitation line to the record, once per process (use the mechanism the file already has
for limitations), saying that the interpreter's attribute cache could not be emptied, so lines
that build attribute names at run time can show live blocks that are not held by the program.
Check that the Python test loop (`harness/python/dla_pytest_loop.py`, it calls
`dla.checkpoint`) takes the same path, and that tick records are not changed.
Do not call the deprecated function on an interpreter that has the new one (a
DeprecationWarning must not appear).

Texts: `skills/detect-leak-anywhere/references/python.md`, the bullet "What the dump measures
(after `gc.collect()` twice)": add that the interpreter's internal caches are emptied too, with
the two function names, and why (one clause). `docs/VERIFY.md`: one new row at the end in the
style of the last rows (left column: "Attribute names built at run time read as growing lines";
state `corrected`; the facts above without naming the project or its functions; the change; the
tests). Do NOT touch `skills/detect-leak-anywhere/SKILL.md` (size limit).

### N2. A site column at a writer line of a finding's holder is at that finding

One predicate, in one place (suggested: `crates/dla-core/src/grown.rs`, public in the crate):
a leak-side finding is "at" a site when its own `site` is that site (trimmed, as today) or the
census annotation of the finding lists that site among `writer_sites` (and the plain `writers`
list if the annotation has one). Use it in the three places named above instead of the bare
comparison of `f.site`. Then find every other place that matches an allocation-site column to
findings by the finding's site (search `is_alloc_site_column`, `column_site(` and
`site_column_add` in `crates/`, including `crates/dla-cli/src/trend/`, and
`crates/dla-cli/src/alloc/pick.rs`) and for each one either use the predicate or say in your
report why it must stay as it is. The intended results:

- the gate no longer asks for a new finding at a writer line of an existing census finding; the
  column is tied to that finding (and waits on it while it has no disposition);
- such a finding counts as grown by that column, so the rules that stop a grown finding from
  being closed unmeasured apply to it (this matters: without it a finding closed
  `not_exercised` would hide a line that grows);
- `dla trend` does not print the command that adds a finding at a line where a finding already
  "is" by this rule; it leads with that finding as it does today for a finding whose site is
  the line.

A sibling (non-leak) finding never ties this way, as today.

`docs/VERIFY.md`: one new row (left column: "A growing writer line of a known holder asked for a
second finding"; state `corrected`).

## Tests (each seen failing under a mutation you apply and revert)

N1:
- `crates/dla-workload/tests/harness_python_blocks.rs` (read its helpers `fixture`, `project`,
  `run_driver`, `block_columns`, `line_of`): a project whose unit calls a method through a name
  built at run time (`getattr(obj, "m_" + suffix)()`, the suffix alternating between two or three
  values) and keeps nothing. Assert that no `py.blocks@` column exists for that line, or that
  its value is 0 at every checkpoint (choose what the agent's column rules make exact, and say
  which). Mutation: the helper's call removed; the test must then fail on the interpreter of
  this machine (state its version). Make the unit do enough lookups per iteration that the
  mutated run shows a non-zero value reliably (in the measurement above 2000 calls per round
  gave 15 to 663 live blocks).
- a unit test or a second assertion that a site that really keeps one object per call still has
  its column with the same values as before (the existing first test of that file may already
  pin this: say so).

N2:
- `crates/dla-cli/tests/py_site_columns.rs` (read `prepared`, `synthetic`, `five_sites` and the
  existing tests): a run with a census-style finding whose annotation has a writer site at line
  L and whose own site is another line, and a trend with a passed `py.blocks@...:L` column.
  Assert: the final gate has no `trend_column` item that asks to add a finding at L while the
  finding is dispositioned; while it is open, the item (if any) names the existing finding and
  not an add command; `dla trend` does not print the add command for L; the finding cannot be
  closed `not_exercised` (the refusal that already exists for a finding at the column's own
  site). If the helpers cannot write an annotation with `writer_sites`, find how the unit tests
  in `crates/dla-core/src/gate.rs` or `trigger.rs` build one and do the same.
- unit tests next to the predicate: own site, writer site, a sibling, a site that is neither.
- Existing tests that pinned "a finding at another line does not tie it"
  (`the_final_gate_lists_the_untied_column_with_the_add_command_and_a_finding_at_another_line_does_not_tie_it`)
  must still pass unchanged: a finding at another line that is not a writer line still does not
  tie. If any existing test changes, say which and why; do not weaken a test.

Mutations by a small script that asserts the old text occurs exactly once. Before you finish,
`grep -rIl "MUT_" crates harness skills docs bench agents hooks data` must print nothing.

## Rules

- No commit, stash, checkout, reset, restore or branch change. Leave the tree dirty.
- Shared `target/` only (`cargo` from /home/user/dla, `-j 2`). No `rm -rf` on variable paths. Do not
  kill processes by name. Do not run `dla consent`. Do not start any model, agent or verifier.
- Run each cargo command on its own, in the foreground, piped through `tail` or `grep`. If an action
  is denied, stop and report it.
- Keep to this brief. If you find another defect, report it; do not fix it.
- Plain technical English, full sentences, no em dashes. Never name a real third-party project,
  library or its APIs in the repository (the four code lines above are from a real library: do not
  copy them; write your own example). No model names, no attribution lines.
- Do not edit anything under `bench/fixtures`. If a golden file or a schema file changes, say which
  and why (`./target/debug/dla schema --check` must stay clean; regenerate with
  `dla schema --out schemas` only if a schema really changes).
- Do not run the whole workspace suite. A pytest for the pytest-dependent tests is at
  /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u72/pillow/.venv/bin
  (put it first on PATH for those runs only).

## Checks before you report

1. `cargo fmt --all --check` clean.
2. `cargo clippy -j 2 --workspace --all-targets -- -D warnings` clean.
3. `cargo build -j 2 -p dla-cli` then `./target/debug/dla schema --check` clean.
4. `cargo test -j 2 -p dla-core --lib`,
   `cargo test -j 2 --no-fail-fast -p dla-workload --test harness_python_blocks --test harness_pytest_loop --test harness_capture_python`
   (two `gc.freeze` tests of harness_capture_python are known to fail under the interpreter of
   this machine before your change: report the counts before and after, by name),
   `cargo test -j 2 --no-fail-fast -p dla-cli --test py_site_columns --test gate_harness --test gate_flow --test trigger --test queue --test keep_going --test pytest_loop --test skill_text`.

## Report (final message; read by the orchestrator)

Two sentences of result first. Then: files changed; each test with its mutation; every place
that matches a site column to findings and what you did there; existing tests changed and why;
anything in this brief that was wrong; what you did not verify. Counts only.
