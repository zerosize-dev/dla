# Targets with no coverage data, Rust driver coverage and features, the life of the objects

You are a builder on the `dla` tool (Rust workspace at /home/user/dla). `<sp>` below is
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad; your scratch
directory is `<sp>/u41/` (exists). Read first `<sp>/wip-notes/real-run-plugin-axios-1-tokio-1.md`
(section tokio-util), then the code named below.

## Working rules (binding)
- No git commit, push, stash, reset, checkout of files. Other builders edit the same tree at
  the same time: one in the Go harness code (`harness/go/`, Go parts of
  `crates/dla-workload/src/harness/{gen,probes,targets,run}.rs`, `crates/dla-core/src/queue.rs`
  functions `with_siblings`, `batch_for`, `declarable`, `crates/dla-core/src/view.rs`,
  `crates/dla-cli/src/harness_cmd.rs`, the alloc-test code), one in the Node harness code
  (`harness/node/`, Node parts of the same files). Touch only what your jobs need; edit with
  exact replacements; re-read a file right before editing it; never restore a file from an
  older copy. In `queue.rs` you change only `Index::standing` and its tests.
- cargo always with `-j 2`; only the shared `target/`. 4 CPUs, other work runs: rerun a failed
  timing test alone before calling it a failure. If the build is broken by someone else's
  half-finished edit, wait a minute and retry; do not fix their code.
- Never kill processes by name. No `rm -rf` with a variable in the path: literal paths only.
- Every behaviour change gets a test you have SEEN fail without the change (revert the hunk by
  exact replacement, run, put it back by the reverse replacement, run again).
- Nothing in the repository names a real open-source project, a model, a plan or an agent.
- docs/VERIFY.md: one row per behaviour change at the end of the `## M7` table, 4 cells, tests
  named, like the rows there.
- Report with the Bash tool: `cat > <sp>/u41/report.md <<'EOF' ... EOF`; final message = short
  summary + that path. Keep the disk above 2 GB free (`df -h /`); set `CARGO_TARGET_DIR` and
  `CARGO_HOME` for runs on the real tree to directories under `<sp>/u41/` and delete build
  output you no longer need (literal paths).

## Job 1: `unknown` does not settle a target after one driver run
`crates/dla-core/src/queue.rs`, `Index::standing`: a target is `Settled(Why::Executed)` when a
hook-driver run gave it verdict `Executed` OR `Unknown`. `Unknown` means the run had no
coverage data: nothing says the target's code ran. In a real run on a Rust library every
target read `unknown`, the queue was empty after one driver and the run ended in 100 seconds.
Wanted: only `Executed` settles as executed. A run with verdict `unknown` counts as one of the
two different drivers of the existing two-drivers rule, nothing more. The texts that explain
a target's standing (`dla ledger list`, the queue item of `run status`, the `target-pending`
refusal, `close-rest`'s record, the report's `Targeted:` line and the skill reference
`references/stages.md`) say for `unknown`: "the run has no coverage data, so whether its code
ran is not known: run a second, different driver, or one with a size reading". Grep for the
places that print standing reasons (`crates/dla-cli/src/ledger_queue.rs`, `crates/dla-core/src/view.rs`
is another builder's file: if the text lives there, change only that string by exact
replacement). Tests in the queue unit tests and `crates/dla-cli/tests/queue.rs`.

## Job 2: coverage data for a Rust driver run
Find how the Rust driver harness is built and run (`crates/dla-workload/src/harness/commands.rs`,
`gen.rs`, the template under `harness/`), and how the Go driver got its coverage point
(`go_cover_point` in `crates/dla-workload/src/harness/run.rs`: one more short process after the
measured one, instrumented, its profile turned into `executed-<S>.json` / `executed.json`
with kind `functions`, read by `dla trend` for the verdicts `executed` / `not_executed` and by
the entry-point coverage). Do the same for Rust: after the measured process, build the driver
once more with `RUSTFLAGS=-C instrument-coverage` into a separate target directory under the
harness directory, run one round (no warm-up, no captures), merge the `.profraw` with
`llvm-profdata` and export the functions that ran with `llvm-cov` (use the tools of the
toolchain that built it: `$(rustc --print sysroot)/lib/rustlib/<host>/bin/`, else the ones on
PATH; if none exists or the versions do not match, a limitation line and verdicts stay
`unknown`: never a refusal). Map functions to census sites the way the Go path does (file and
function name; demangled names). The measured points stay uninstrumented. Its failure is a
limitation, never a refusal. Tests: a small Rust crate in a temp dir (look at the existing
Rust harness tests for how fixtures are made and how they skip when cargo is missing): a
driver whose unit calls one function and not another gives `executed` for the first and
`not_executed` for the second.

## Job 3: crate features for the Rust harness build
`dla harness gen --lang rust --driver --features a,b` (and `--no-default-features` if the
build needs it): passed to the cargo build of the driver, recorded in `harness.json`, kept on
reruns. When a target's site lies in a module that the crate gates with
`#[cfg(feature = "x")]` on its `mod` declaration (read `src/lib.rs` and the parent module
files with the existing tree-sitter helpers or a plain line scan; keep it simple and say what
it does not see), the target block says "behind feature x: generate with --features x", and
`harness gen --next` without the feature prints the same as a note. Tests for the option and
the note.

## Job 4: the driver block asks for the life of the objects
The generated block above the hooks (all runtimes; find the shared text in
`crates/dla-workload/src/harness/targets.rs` or `gen.rs`) gets one short rule, in the list of
what a unit must do: "a unit covers the life of the objects it touches: create, derive
(also from derived objects), use, cancel or close, drop or release, and not always in the
same order (vary it by key)". Keep it one or two lines. Add the same sentence, once, to the
skill reference that explains how to fill a driver (`skills/detect-leak-anywhere/references/stages.md`
or the per-language reference: where the hook rules are). SKILL.md itself has word limits
checked by the `skill_text` test: do not grow it. Update the golden texts of tests that
assert the block.

## Check on a real tree (tool check)
Copy `<sp>/u18/tokio/proj` to `<sp>/u41/tokio` (cp -r; a crate source tree, dependencies come
from crates.io: if the registry is not reachable, say so and stop this part). With your binary
and `DLA_PLUGIN_ROOT=/home/user/dla`, start the run the way the unattended benchmark runner
does (environment `DLA_CONSENT_TIERS=1-4 DLA_CONSENT_RAW_VALUES=1` on `dla run init`; never run
`dla consent`; if the permission system refuses a command, report it, do not look for another
way), then `probe`, `sweep --lang rust`, `census --lang rust`, `workload discover`,
`ledger list`, `harness gen --lang rust --driver --next`. Fill the hooks twice:
(a) a unit that registers a waiter on a token and cancels it; `harness run --mode both`,
`trend`: targets read `executed` or `not_executed` from the coverage data (not `unknown`),
`run status` still shows the queue open for the ones not executed;
(b) a second harness whose unit creates a parent token, a child and a grandchild
(`parent.child_token().child_token()`), cancels the parent and drops all three: show the
`native.live_bytes` column and what `dla trend` says (the documented leak is 72 bytes per
unit; you know the cause, that is fine, this checks the tool), what `next` proposes, and
whether the growth can be tied to a finding (`dla alloc-test`, `--capture alloc` if the Rust
harness has allocation stacks, `dla ledger add --origin growth-series`). Record exact outputs.
Also generate once with `--features rt` and show that the `spawn_pinned` targets build.

## Acceptance (say exactly what you could not run)
`cargo fmt --all --check`; `cargo clippy --workspace --all-targets -j 2 -- -D warnings` (if it
fails only in files you did not touch, name them and run `--no-deps` on your crates);
`cargo test -j 2 --no-fail-fast -p dla-core -p dla-workload -p dla-cli`;
`target/debug/dla schema --check` (regenerate with `dla schema --out schemas` if a schema
changed); `python3 bench/runner/test_grade.py`; `cargo test -j 2 -p dla-cli --test skill_text`;
`dla-bench dynamic --only rust-` with your binary (copy `target/debug/dla` and
`target/debug/dla-bench` to `<sp>/u41/bin/`, run from /home/user/dla with
`DLA_PLUGIN_ROOT=/home/user/dla`).

## Report
Per job: what changed (files, functions), tests and that you saw each fail, the real-tree
outputs, what you decided not to do and why, what is unverified.
