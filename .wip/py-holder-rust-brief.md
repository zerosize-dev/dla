# Brief R2: the holder chains of the Python agent become retention evidence

Repository: /home/user/dla (Rust workspace, branch `main`). The working tree holds one finished
agent change that this brief builds on (see "What exists"); all of it stays. Scratch directory:
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u93.
`target/` is empty: your first cargo command rebuilds everything (15 to 20 minutes).

## Why

A Python driver run counts live blocks per project line (`py.blocks@<file:line>`). A line that
grows says where the surviving objects were made, not who keeps them. In a real run on a Python
library, twelve lines grew in the loop over the project's tests; eleven were objects of ordinary
callers, all kept by one holder that no line pointed at (a module-level object that stores an
exception and re-raises that same object, so its traceback grows and keeps the raising frames
and their locals), and one was held from native code. The run added findings at the lines, had
nothing to say about holders, and ended with eleven unexplained growing lines.

## What exists (do not redo; read it)

`harness/python/sitecustomize.py` has a new capture kind `heap` (section "heap" and
`_capture_heap`; `harness/python/README.md` describes it). With `heap` in `DLA_CAPTURE` the agent
writes, at the last dump point of the process, `DLA_CAPTURE_DIR/<pid>/py-1.retain.json`:

    {"dla_pyretain": 1, "frames": N,
     "sites": [{"site", "blocks_first", "blocks_last", "found_tracked", "found_untracked", "kept",
                "note"?, "chains": [{"count", "root": {...},
                                     "steps": [{"type", "via"?, "at"?, "repeat"?}]}]}],
     "limitations": [...]}

Roots: `{"kind":"module-global","module","key"}`, `{"kind":"class-attribute","class","attribute"}`,
`{"kind":"frame","function","at"}`, `{"kind":"no-referrer"}`, `{"kind":"depth-limit"}`,
`{"kind":"no-root"}`. Steps run from the kept object to the holder; a step's `via` says how the
NEXT step reaches this object. A site whose blocks grew and for which no object was found has a
`note` that native code holds them or a reference was lost. The final record's `captures` array
gets `{"kind":"heap","file":"py-1.retain.json",...}`. `DLA_RAW_VALUES=1` lets the agent write
dict keys; without it only identifiers and type names are written. `DLA_HEAP_SECS` bounds the
time (default 60 s). Real example, checked by hand on that library:

    site src/x/cms.py:411  blocks 800 -> 4800  tracked 3600  kept 40
      count 17  root module-global pkg.cms._native
        traceback(__traceback__)@src/x/cms.py:411 x4774 <- ImportError(.ex) <- pkg._util.Deferred(._native)
    site src/x/font.py:580  blocks 200 -> 1200  tracked 0  untracked 0  kept 0
      note: no object was found for this site: the blocks do not belong to an object any Python
      container refers to, so native code holds them or a reference was lost there

Test scripts of that change (Python only) are in scratchpad `u91` (`runcase.py`,
`check_cases.py`, the test project `proj/` with cases A to I); reuse the case modules for your
fixtures.

## The change

1. The capture flag. `--capture heap` is supported for Python
   (`crates/dla-workload/src/harness/capture.rs`, `supported`), and `--capture all` includes it.
   `dla harness run` sets `DLA_RAW_VALUES=1` for the agent when the run's consent allows raw
   values (find how the run records that; the alloc and histogram captures of other runtimes are
   the model for value handling). `harness/<H>/captures.json` lists the file with its kind, and
   the run prints the summarize command for it next to the others
   (`capture::summarize_commands`).
2. The summary. `dla summarize heap-graph <file>` reads a `dla_pyretain` file with a new adapter
   (model: `crates/dla-cli/src/summarize/node_heapsnapshot.rs`, the single-snapshot path; reuse
   its evidence shape for `holders` so that the label rule, the bundle and the report need no
   special case). It writes one `retention-path` record with:
   - one holder per distinct root: for `module-global` the key `<module>.<key>` and holder kind
     static/global; for `class-attribute` `<class>.<attribute>`, static/global; for `frame` the
     function and its line, kind stack (use the kind the other adapters use for a stack root);
     for `no-referrer` and for a site with the native note, kind `native-handle/global-ref` and
     the site as its key;
   - per holder: the sites it holds with their block counts at the first and last dump, the
     number of kept objects, and the chain with the most objects as a path in the frame-exempt
     form the bundle keeps (`FRAME_KEYS` in `crates/dla-cli/src/bundle_cmd.rs`: check which key
     fits; a step with `at` is a frame the bundle may show an excerpt for);
   - ties: a finding whose owner names the holder (the rule the other adapters use, `tie.rs`),
     and a finding at a site the holder holds (`dla_core::grown::finding_is_at_site`);
   - the input's digest in the raw file, in the place the alloc adapter keeps its `captures`
     digests, so that "captured and not summarized" can be told (see 4);
   - `next`: for a holder that names no finding, the command that adds a finding for it (model:
     what the node adapter prints for an unowned holder; the site of the new finding is the
     `at` of the first step that has one closest to the root, else the growing site), and for a
     holder that names a finding, the command that the route of that finding needs next.
3. The gate. A non-flat `py.blocks@<site>` column that is tied to no finding by today's rules is
   tied to the findings that a `retention-path` record names for a holder that holds that site
   (extend `tied_findings` in `crates/dla-core/src/gate.rs`; keep `grown_unmeasured` as it is:
   the lines a holder keeps alive are not growth of their own findings, so they do not create an
   alloc-test obligation for the holder's finding beyond what its own columns create; say in the
   report if you find that this leaves a hole). So one finding at the holder accounts for every
   line whose objects it keeps, and the item of such a column, while the finding has no
   disposition, names the finding and not an add command.
4. Hints.
   - `dla alloc-test` and the `stacks-not-captured` refusal of `dla bundle` (both in the working
     tree; read `stacks_offer`, `stacks_commands`, `stacks_hint`, `stacks_check`): for a runtime
     whose heap capture is this chain capture (Python), the capture command they print is
     `--capture alloc,heap`, and the sentence says the run prints two summarize commands (the
     stacks and the holders). The check for "captured and not summarized" prints every
     summarize command of that capture that has not been run.
   - `dla trend`: when it prints the add command for a growing `py.blocks@` line that no finding
     is at, and the harness has no heap capture yet, it also prints, once per trend and in a key
     the 2 KB output bound never cuts (`KEPT_WHOLE` in `crates/dla-cli/src/out.rs`), that the
     line says where the objects were made and `dla harness run <H> --mode <mode> --capture heap`
     with the summarize command it prints says who holds them; add the finding at the holder
     when one holder keeps several lines.
5. Texts. `skills/detect-leak-anywhere/references/python.md`: the capture list and the route
   (a growing line, then who holds it, then a finding at the holder; the native note means read
   the extension function the line calls). `skills/detect-leak-anywhere/references/stages.md`:
   the Python capture kinds. `harness/python/README.md` if the Rust side changes what it says.
   `docs/VERIFY.md`: one new row at the end in the style of the last rows (left column: "A
   growing line said where objects were made and nothing said who keeps them"; state
   `corrected`; no project name). Do NOT touch `skills/detect-leak-anywhere/SKILL.md` (size
   limit). Pins in `crates/dla-cli/tests/skill_text.rs`.

## Added after a later real run (read this first; it sets the order of work)

A later unattended run confirmed the leak in the C extension with the right explanation, and the
independent verifier still disagreed. Its bundle held the allocation stack and the excerpt of the
growing line (a call into a function of an extension module), and it answered N-OBSERVER: "nothing
in the three Python modules adds to a long-lived container ... the retained blocks are most likely
test or pytest-held results, or tracemalloc attribution ... there is no entry-path run ... I found
no library-side holder". Two facts were missing from what it was given: that no Python object
holds the blocks of the line (this capture measures exactly that), and that the growth was
reproduced by a driver made for the finding, one block per unit, not only by the loop over the
tests.

Order of work: P1 first (items 1, 2 and 6 and the bundle part of 7), then P2 (item 3), then P3
(items 4, 5, 7 trigger part). If you run short of budget, finish and verify P1 and say what is
not done.

6. The bundle and the verifier (P1).
   - `dla bundle` already takes the `retention-path` records that name the finding
     (`naming_records` in `crates/dla-cli/src/bundle_cmd.rs`). Check that the record of item 2
     arrives in the bundle with its chains readable (type names and links are not values; the
     holder key of a module global is an identifier), and that the native note survives the scrub.
   - `index.json` of a bundle gets a `harnesses` list for the harnesses behind the bundled
     evidence: the id, what it is in plain words ("the project's own tests in a loop", "a driver
     made for this finding", "a driver for other targets", "a command of the project"), the
     units per round, and the key modes. No finding id, no site.
   - The verifier's instructions (`agents/verifier.md`; read how it is told to answer): add what
     the new record means. Blocks of a line that no Python container refers to are held from
     native code or lost there: the answer is then in the extension function the line calls, and
     "held by the test runner" or N-OBSERVER does not explain them. A chain that ends at a module
     global or a class attribute names the holder. Keep the text short and in the style of the
     file. Pin it in `crates/dla-cli/tests/skill_text.rs` (or the test file that pins agent
     texts).
7. The trigger (P3) and the bundle's harness list (P1, above). When a test loop and a driver
   made for the finding both grew it, the trigger of the report names the driver and its
   VARIES text, not the loop (`crates/dla-core/src/trigger.rs`; today the report says
   "reproduced through the public API in <loop> (the project's own tests ...)" although a probe
   driver reproduced it). One test.

## Tests (each seen failing under a mutation you apply and revert)

- `crates/dla-workload/tests/harness_capture_python.rs` (read its helpers): a run with
  `--capture heap` writes the file, the `captures` entry and the `captures.json` entry; a run
  with consent for raw values passes `DLA_RAW_VALUES`, one without does not.
- `crates/dla-cli/tests/summarize_python.rs`: the adapter on files you write by hand for each
  root kind and for the native note: holders, kinds, sites, ties (owner and site), `next`.
- One end-to-end CLI test on a real Python harness (guard with `have_python()`), in
  `crates/dla-cli/tests/py_site_columns.rs` or a new file: a project with a module-level object
  that re-raises a stored exception (write your own; do not copy a real library) and a unit that
  makes a local object and touches it in try/except. Run the harness, `dla trend` (assert the
  kept hint), the capture run, the summarize command it prints; assert the record names the
  module global as the holder of the growing line; add the finding with the command `next`
  prints; assert the final gate has no item that asks for a finding at the victim line; bundle
  the finding and assert the bundle holds the retention record with its path.
- The gate rule as unit tests next to `tied_findings` (or CLI tests if the unit tests cannot
  build a view): a retention record ties a victim column to the holder's finding; a record that
  names no finding ties nothing; a sibling never ties.
- Existing tests that pin "heap is not supported for Python" or the Python capture list change
  to the new rule: say which. Do not weaken a test. Do not edit anything under `bench/fixtures`.

Mutations by a small script that asserts the old text occurs exactly once. Before you finish,
`grep -rIl "MUT_" crates harness skills docs bench agents hooks data` must print nothing.

## Rules

- No commit, stash, checkout, reset, restore or branch change. Leave the tree dirty.
- Shared `target/` only (`cargo` from /home/user/dla, `-j 2`). No `rm -rf` on variable paths. Do not
  kill processes by name. Do not run `dla consent`. Do not start any model, agent or verifier.
- Run each cargo command on its own, in the foreground, piped through `tail` or `grep`. If an action
  is denied, stop and report it.
- Keep to this brief. If you find another defect, report it; do not fix it. If the agent's file
  format lacks something the summary needs, say what, and change the agent only for that.
- Plain technical English, full sentences, no em dashes. Never name a real third-party project,
  library or its APIs in the repository. No model names, no attribution lines.
- If a golden file or a schema file changes, say which and why (`./target/debug/dla schema --check`
  must stay clean; regenerate with `dla schema --out schemas` only if a schema really changes).
- Do not run the whole workspace suite. The machine's `python3` is 3.13; a 3.12 with pytest is at
  /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u72/pillow/.venv/bin.

## Checks before you report

1. `cargo fmt --all --check` clean.
2. `cargo clippy -j 2 --workspace --all-targets -- -D warnings` clean.
3. `cargo build -j 2 -p dla-cli` then `./target/debug/dla schema --check` clean.
4. `cargo test -j 2 -p dla-core --lib`, `cargo test -j 2 -p dla-cli --bin dla`,
   `cargo test -j 2 --no-fail-fast -p dla-workload --test harness_capture_python --test harness_python_blocks`,
   `cargo test -j 2 --no-fail-fast -p dla-cli --test summarize_python --test py_site_columns --test harness_capture --test alloc_test --test ledger_m6 --test gate_flow --test gate_harness --test keep_going --test skill_text`
   and any test file you added.

## Report (final message; read by the orchestrator)

Two sentences of result first. Then: files changed; the evidence record as written for the
stored-exception fixture (one real example); each test with its mutation; existing tests changed
and why; anything in this brief that was wrong; what you did not verify. Counts only.
