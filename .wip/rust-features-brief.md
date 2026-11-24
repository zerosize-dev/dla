# Brief R: Rust code that is compiled only with a feature the crate does not turn on by default

Repository: /home/user/dla (Rust workspace, branch `main`). The working tree may hold uncommitted
changes of other builders: leave what you do not need alone.
Scratch directory: /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u116
(create it; write nothing else outside the repository).

## What happened (a real unattended run on a Rust library the tool had never seen)

The library is a tracing crate. Its `Cargo.toml` has `[features] enable = ["<macro crate>/enable"]`
and no `default`. Its public span type compiles to a no-op unless `enable` is on: one source
file has 25 `#[cfg(feature = "enable")]` sites, others 10 and 8, all INSIDE functions and on
items; no `mod` declaration is gated. The documented leak is a vector field of an internal
sender that a collector fills when spans are committed.

The census listed that field as a candidate. The model wrote a driver that creates and drops
spans, and a focus harness for the field. Both ran with the default features, so no span did
anything, the collector never ran, and the target read "not executed" twice. The queue then
closed it as static-only ("missed by two drivers, the focused harness among them") and the run
ended COMPLETE after two minutes.

`dla harness gen --lang rust --driver --features a,b` exists, and a target in a module that the
crate gates behind a feature says so in its block (`crates/dla-workload/src/harness/targets.rs`,
`feature_gate`, `add_feature_gates`; `gen.rs` near lines 2749 and 2873). That covers a gated
`mod` declaration only.

## The change (Rust only)

1. What a crate's features gate. For the crate of a target, read `[features]` of its
   `Cargo.toml` and count, per feature that the `default` list does not turn on (directly or
   through another default feature), the `#[cfg(feature = "x")]`, `#[cfg(not(feature = "x"))]`,
   `#[cfg_attr(feature = "x", ..)]` and `cfg!(feature = "x")` sites in the crate's `src`, per
   file. Reuse the reader that `feature_gate` has (it knows `all`, `any`, `not`; keep its
   treatment of test, target and platform gates).
2. The target block of a Rust driver (the text the model reads in the generated driver) says,
   for a crate that has such sites: which features are off by default, how many gated sites
   each has in the target's file and in the crate, and the exact command that regenerates the
   driver with them on (`dla harness gen --lang rust --driver --features <list> ...` with the
   same targets). When the features of the driver already include them, it says nothing.
   The `harness gen` JSON output carries the same under a key the output bound never cuts
   (`_attention`, see `crates/dla-cli/src/out.rs`) when a feature with gated sites in a
   target's file or in the crate's public entry files is off.
3. A target whose code did not run. The remedy that `dla trend` and the queue give for
   "not executed" / "not reached" names the features with gated sites that the harness did not
   enable, and the `--features` command, before any other advice. The queue does not settle a
   Rust target as not reached after a focus harness while the crate has a feature with gated
   sites that none of the harnesses that targeted it enabled: it stays open with that remedy
   (find where "missed once, no focus harness" and the close-rest rule decide, and add this one
   condition; say where).
4. Texts: `skills/detect-leak-anywhere/references/rust.md` (or the file that holds the Rust
   notes: find it) and `harness/rust/README.md`: three sentences, pinned in
   `crates/dla-cli/tests/skill_text.rs`; `docs/VERIFY.md`: one row at the end, state
   `corrected`, "a Rust library the tool had not been run on". Do NOT touch
   `skills/detect-leak-anywhere/SKILL.md`.

## Tests (each seen failing under a mutation you apply and revert)

- Unit tests next to the reader: a crate with `enable` off by default and gated sites inside
  functions gives the counts per file; a feature in `default` gives nothing; a feature turned
  on through another default feature gives nothing; `not(feature)` and `cfg!` count; a gated
  `mod` still works as today.
- CLI tests with real Rust builds, guarded like the existing Rust tests (find the helper): a
  small crate whose public function fills a static registry only under
  `#[cfg(feature = "enable")]`. `harness gen --driver --next` without features: the block and
  `_attention` name `enable` and the command; the run reads the target not executed and the
  remedy leads with the feature; the queue keeps it open after a focus harness. With
  `--features enable`: the target grows and nothing is said about features.
- Existing tests that change: say which and why. Do not weaken a test. Nothing under
  `bench/fixtures` is edited; if a Rust fixture's output changes (a new `_attention`), list it.

## Rules

- No `git commit`, `git stash`, `git checkout` (in any form), `git reset`, `git restore` or
  branch change. Leave the tree dirty.
- Shared `target/` only (`cargo` from /home/user/dla, `-j 3`). No `rm -rf` on variable paths. Do
  not kill processes by name. Do not run `dla consent`. Do not start any model, agent or
  verifier. No package installs; the small crate has no dependencies.
- Run each cargo command on its own, in the foreground, piped through `tail` or `grep`. If a
  build fails in a file you did not edit, another change is in progress: wait two minutes and
  try again, up to five times, then report it. If an action is denied, stop that action and
  report it; do not ask anyone to do it for you and do not try another way to the same end.
- Keep to this brief. If you find another defect, report it; do not fix it.
- Plain technical English, full sentences, no em dashes. Never name a real third-party project,
  library or its APIs in the repository. No model names, no attribution lines.
- Mutations by a small script that asserts the old text occurs exactly once. Before you finish,
  `grep -rIl "MUT_" crates harness skills docs bench agents hooks data` must print nothing
  (a marker of another change in progress is not yours: report it, do not remove it).

## Checks before you report

`cargo fmt --all --check`; `cargo clippy -j 3 --workspace --all-targets -- -D warnings`;
`cargo build -j 3 -p dla-cli` and `./target/debug/dla schema --check`;
`cargo test -j 3 -p dla-workload --lib`, `cargo test -j 3 -p dla-core --lib`,
`cargo test -j 3 -p dla-cli --bin dla`, every Rust-harness test file of dla-cli and dla-workload
(list them), `--test queue --test gate_flow --test skill_text`, and the tests you added.

## Report (final message; read by the orchestrator)

Two sentences of result first. Then: files changed; the exact rule for "off by default" and
for a gated site; one real target block and one real remedy (first lines); where the queue rule
sits; each test with its mutation; existing tests changed and why; anything in this brief that
was wrong; what you did not verify. Counts only.
