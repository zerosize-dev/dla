# Brief W: a run does not keep one full copy of the project per harness

Repository: /home/user/dla (Rust workspace, branch `main`). Scratch directory:
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u100.

## Why

`dla harness gen` copies the project into `.dla/<run>/harness/<H>/work`
(`scratch::copy_project_for`, called from `crates/dla-workload/src/harness/gen.rs`). A real
unattended run on a Python library made 168 harnesses (one probe driver per batch of
candidates); each kept a copy of about 80 MB, 13 GB in all, and the machine ran out of disk
before the run ended. Almost every file of such a copy is byte for byte the project's file, and
most harnesses are used for a few minutes and never again.

## The change

A harness's `work/` becomes a copy that can be thinned and filled again.

1. Thin (`crates/dla-workload/src/harness/scratch.rs`, a new public function). Walk `work/`
   without following symlinks. A regular file is removed when the project holds a regular file
   at the same relative path with the same length and the same bytes. Everything else stays:
   changed files, new files (the generated driver, build outputs, caches), symlinks and every
   directory. Before the first file is removed, the list of files to remove is written to
   `harness/<H>/work-thinned.json` (write a temporary file, then rename):
   `{"dla_work_thin":1,"files":[{"path","len","digest"}],"bytes":N}`; `digest` is the digest
   function the workspace already uses for evidence inputs (find it; add no dependency). So a
   thin that is cut half way leaves a list whose files are partly still there, which the fill
   handles.
2. Fill (a second function). When `work-thinned.json` exists: for every listed file that is
   missing in `work/`, check that the project's file still has the listed length and digest.
   Check ALL files first. If one or more do not match or are gone, change nothing and refuse
   with a new code `work-copy-stale` (`crates/dla-core/src/error.rs`, and wherever the codes are
   listed for the docs and tests): the message names the count and the first few paths, and the
   fix says how to get a harness on the project as it is now (read how the run's seal and
   `dla run reseal` treat a changed project and make the fix agree with them; say in your report
   how the two interact). If all match, copy them with the function the first copy used (the
   modification time and the permissions of the project's file are kept, so a build tool sees
   the same tree as before and does not rebuild), then delete `work-thinned.json`.
3. A lock per harness, `harness/<H>/work.lock`, with the standard library's file locks
   (`File::lock`, `lock_shared`, `try_lock`; the workspace's minimum Rust is 1.89, where they are
   stable).
   - A user of `work/` (every code path that reads or runs in it: `Paths::new` in
     `crates/dla-workload/src/harness/run.rs` for `dla harness run` and what runs through it;
     the clone source in `crates/dla-cli/src/intervene/clone.rs` and
     `crates/dla-cli/src/intervene_cmd.rs`; FIND THE OTHERS: every place that opens a file under
     a harness's `work/`, for example for an excerpt, a symbol or coverage) does this: take the
     shared lock; if `work-thinned.json` does not exist, go on and hold the shared lock until it
     is done with `work/`; else release it, take the exclusive lock, fill if the file still
     exists, release, and start again. It also sets the modification time of `work.lock` to now
     (the "last used" time).
   - The thinner takes the exclusive lock with `try_lock` and skips the harness when it does
     not get it. It skips a harness that already has `work-thinned.json`.
   - For a clone you may copy the thinned `work/` together with `work-thinned.json` instead of
     filling the source first, if that is simpler and correct; say which you chose.
4. When it runs.
   - At the end of `dla harness gen`: every harness of the run except the four used last (by
     the time of `work.lock`, else of the harness directory) and except the new one is thinned.
     The JSON output of `harness gen` gets `thinned` (the ids) and `thinned_bytes` when
     something was thinned, and nothing otherwise.
   - At the end of `dla report`: every harness of the run is thinned.
   - `DLA_KEEP_WORK=1` in the environment turns thinning off (fill still works).
   - A failure to thin is never a failure of the command: it is skipped, and reported in the
     output key `thin_skipped` with the reason.
5. Texts. `skills/detect-leak-anywhere/references/stages.md`: two sentences where harnesses are
   described (older harnesses keep only the files that differ from the project; using one again
   brings its files back; `work-copy-stale` means the project changed in between).
   `docs/VERIFY.md`: one new row at the end in the style of the last rows (left column: "A run
   kept one full copy of the project per harness"; state `corrected`; no project name). Do NOT
   touch `skills/detect-leak-anywhere/SKILL.md`. Pin the sentences in
   `crates/dla-cli/tests/skill_text.rs`.

Not in scope: build outputs inside `work/` (a compiled harness keeps them; report the sizes you
see for one Rust, one Go and one JVM fixture harness so the next step can be decided), sharing
one copy between harnesses, hard links.

## Tests (each seen failing under a mutation you apply and revert)

- Unit tests next to the functions: thin removes identical files only and keeps a changed file,
  a new file, a symlink and an empty directory; the list holds exactly the removed files; fill
  brings back bytes, modification time and the executable bit; a project file changed after the
  thin gives the refusal and leaves `work/` and the list untouched; a list whose file is still
  present (a cut thin) fills the others and keeps that file; a second thin of a thinned harness
  does nothing.
- CLI tests (a new file under `crates/dla-cli/tests/`; guard with `have_python()` where a real
  run is needed; read `py_site_columns.rs` for helpers): six `dla harness gen` calls on a small
  Python project thin the first one or two (the list exists, a known source file is gone, the
  driver is there, the output names them); `dla harness run` on a thinned harness fills it (file
  back, list gone) and the run's result passes the same assertions as a run of a harness that
  was never thinned; a harness whose `work.lock` the test holds with a shared lock is not
  thinned; a project file changed after the thin makes `dla harness run` refuse with
  `work-copy-stale` and the fix; with `DLA_KEEP_WORK=1` nothing is thinned; `dla report` thins
  all; `dla intervene` (or the clone function) from a thinned harness gives a clone that runs.
- Existing tests that count files in `work/` or read a file of an older harness there: say
  which changed and why. Do not weaken a test. Do not edit anything under `bench/fixtures`.

Mutations by a small script that asserts the old text occurs exactly once. Before you finish,
`grep -rIl "MUT_" crates harness skills docs bench agents hooks data` must print nothing.

## Rules

- No `git commit`, `git stash`, `git checkout` (in any form, `-p` and `-- <file>` included),
  `git reset`, `git restore` or branch change. Leave the tree dirty. Undo your own edits by hand
  or with your mutation script.
- Shared `target/` only (`cargo` from /home/user/dla, `-j 2`). No `rm -rf` on variable paths. Do not
  kill processes by name. Do not run `dla consent`. Do not start any model, agent or verifier.
- Run each cargo command on its own, in the foreground, piped through `tail` or `grep`. If an action
  is denied, stop and report it; do not ask anyone to do it for you.
- Keep to this brief. If you find another defect, report it; do not fix it.
- Plain technical English, full sentences, no em dashes. Never name a real third-party project,
  library or its APIs in the repository. No model names, no attribution lines.
- If a golden file or a schema file changes, say which and why (`./target/debug/dla schema --check`
  must stay clean; regenerate with `dla schema --out schemas` only if a schema really changes).
- Do not run the whole workspace suite. The machine's `python3` is 3.13.

## Checks before you report

1. `cargo fmt --all --check` clean.
2. `cargo clippy -j 2 --workspace --all-targets -- -D warnings` clean.
3. `cargo build -j 2 -p dla-cli` then `./target/debug/dla schema --check` clean.
4. `cargo test -j 2 -p dla-workload --lib`, `cargo test -j 2 -p dla-cli --bin dla`,
   `cargo test -j 2 --no-fail-fast -p dla-workload --test harness_pytest_loop --test harness_python_blocks`,
   `cargo test -j 2 --no-fail-fast -p dla-cli --test py_site_columns --test harness_capture --test alloc_test --test intervene --test gate_flow --test keep_going --test skill_text`
   (if a named test file does not exist, find the one that covers `dla intervene` and say so)
   and any test file you added.

## Report (final message; read by the orchestrator)

Two sentences of result first. Then: files changed; every reader of `work/` you found and how it
is handled; one real example of `work-thinned.json` (first lines) with the bytes freed; each test
with its mutation; existing tests changed and why; the build-output sizes asked for above;
anything in this brief that was wrong; what you did not verify. Counts only.
