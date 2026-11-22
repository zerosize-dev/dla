# Brief V: Python function coverage when the package runs from the source tree

Repository: /home/user/dla (Rust workspace, a dirty working tree; all of it is to stay). Scratch
directory: /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u84.

## The defect

A real run on a Python library installed in editable mode ended with this at the top of its
report: "Coverage warning: this run exercised 0 of 739 entry points (python) ... 3520 iterations
ran and none called an entry point, so the driver did not reach the code under test". That is
false: the drivers called the library's public API in every unit.

Cause: the harness runs in a scratch copy (`DLA_PROJECT`), but a package installed in editable
mode is imported from the project the copy was made from (`DLA_SOURCE_ROOT`, set by
`dla harness run` for Python). The agent's coverage callback, `on_start` inside
`_start_coverage` in `harness/python/sitecustomize.py`, records only functions whose file lies
under `DLA_PROJECT`, so every function of the package is dropped. The allocation-site columns
already solve the same problem with `_source_rel(filename)` in that file (a path under
`DLA_SOURCE_ROOT`, relative to it, not `.dla`, `.git` or installed packages; the copy keeps the
project's layout, so the relative path names the same file in both).

## The change

In `on_start`: when the file is not under the scratch root, use `_source_rel(filename)`. The
recorded item has the same form as today, `module:qualname@<relative path>:<first line>`, with
the path relative to the source root. Keep the per-file cache (`decided`) and the cap. Nothing
else changes. Check `harness/python/README.md` for the sentence that describes the coverage
section and add the source-root case in one clause.

## Tests (each seen failing under a mutation that you revert)

- `crates/dla-workload/tests/harness_capture_python.rs` drives the agent directly (read its `Run`
  helper: it sets `DLA_PROJECT` and runs `python3` with the agent first on `PYTHONPATH`). Add one
  test: a second directory outside `DLA_PROJECT` holds a module `lib/tool.py` with a function;
  that directory is passed as `DLA_SOURCE_ROOT` and put on `PYTHONPATH`; the script calls the
  function; the dump's `coverage.items` holds `lib.tool:<function>@lib/tool.py:<line>`. Without
  `DLA_SOURCE_ROOT` the item is absent. A file under the source root's `.dla` directory is never
  listed. The test skips when `have_python()` is false, as its neighbours do, and also when the
  interpreter is older than 3.12 if its neighbours that read `coverage` do so (follow them).
- If there is an existing test of `dla harness run` that checks exercised entry points of a Python
  driver (search `crates/dla-workload/tests` and `crates/dla-cli/tests` for `exercised`), add the
  editable-install case there only if it takes a few lines; otherwise say that it is not covered
  end to end.

## Rules

- No commit, stash, checkout, reset, restore or branch change. Leave the tree dirty.
- Shared `target/` only (`cargo` from /home/user/dla, `-j 2`). No `rm -rf` on variable paths. Do not
  kill processes by name. Do not run `dla consent`. Do not start any model or agent. If an action is
  denied, stop and report it.
- Keep to this brief. Plain technical English, full sentences, no em dashes. Never name a real
  third-party project or its APIs in the repository. No model names, no attribution lines.
- Run each cargo command on its own, in the foreground, piped through `tail` or `grep`. Do not chain
  a background run to other commands.
- Two tests of `harness_capture_python` about `gc.freeze` fail under the interpreter at
  /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u72/pillow/.venv/bin
  for a reason that is not yours; use the default `python3` (3.13) for your runs.

## Checks before you report

1. `cargo fmt --all --check` clean.
2. `cargo clippy -j 2 -p dla-workload --all-targets -- -D warnings` clean.
3. `cargo test -j 2 -p dla-workload --test harness_capture_python --test harness_python_blocks` pass.
4. `grep -rIl "MUT_" crates harness skills docs bench agents hooks data` prints nothing.
5. Do NOT run the whole workspace suite.

## Report (final message; read by the orchestrator)

Two sentences of result. Then: files changed; each test with its mutation; anything in this brief
that was wrong; what you did not verify. Counts only.
