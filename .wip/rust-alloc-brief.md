# Allocation stacks for Rust harness runs

Follow-up for the builder of the Rust driver coverage (same working rules as your first
brief: cargo -j 2, shared target/, no commits, no stash/reset/checkout of files, never
restore a file from a copy, never kill by name, literal paths in rm, every behaviour change
with a test SEEN failing, VERIFY rows at the end of `## M7`, nothing in the repository names
a real project, a model, a plan or an agent; `CARGO_TARGET_DIR` and `CARGO_HOME` for runs on
the real tree under `<sp>/u41/`; keep the disk above 2 GB free). Append your report to
`<sp>/u41/report2.md` with the Bash tool.
`<sp>` = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad.

Two other builders edit the tree now: the batch and queue code (`crates/dla-core/src/queue.rs`
incl. `Index::standing`, `crates/dla-workload/src/harness/{gen,targets}.rs` batch parts,
`crates/dla-cli/src/{harness_cmd,trend_cmd,ledger_queue}.rs`, `view.rs`) and the statistics
and alloc-test rules (`crates/dla-stats/`, the alloc-test code, the growth gate). Do not
change those; if the tie of an allocation stack to a finding needs a change in code they
own, make the smallest exact-replacement change and name it in your report.

## The gap
Your real-tree check showed it: a Rust unit that leaks 72 bytes per unit makes
`native.live_bytes` pass the gate with slope 72.0, `dla alloc-test` favors leak, and then the
tool cannot say where: "`--capture alloc` writes nothing for Rust, so there are no allocation
stacks". A Rust holder has no size probe, so the allocation stack is the only way to tie the
growth to a site, and without a tie nothing can be confirmed (`growth-not-tied`).

## Wanted
- `dla harness run H --capture alloc` on a Rust harness (driver mode and command mode) writes
  allocation-stack captures the way a native (C/C++) harness does. Find how the native path
  does it (the preload shim `libdlashim.so`, the tests `native_shim_sites_are_symbolized_to_the_leaking_line`
  and the summarize op `alloc-stack-diff` with its native backend; `references/backends.md`
  says which op reads which file) and make the Rust harness use the same mechanism: the
  measured Rust process runs with the shim preloaded, and the driver build carries what the
  shim's unwinder and the symbolizer need (line tables: `-C debuginfo=1` or what the native
  path requires; frame pointers `-C force-frame-pointers=yes` if the shim walks frame
  pointers; say what you chose and measure the slowdown of the measured points, because the
  flags apply to the measured build too unless you build a separate capture binary: prefer
  what the native path does). Rust's default allocator is the system `malloc`, so the shim
  sees the allocations; a crate with `#[global_allocator]` set to something else is detected
  if cheaply possible (a line scan for `global_allocator`) and reported as a limitation.
- `dla summarize alloc-stack-diff <captures>` on those files names the project frames with
  demangled Rust function names and `file:line` relative to the project root (the scratch
  copy's paths mapped back like the native path does), skips allocator and std frames when it
  picks the site of a stack (first frame inside the project, as for native), and ties the
  growth to a finding whose site or writer is in that function, by the existing tie rules.
- `dla trend` of a Rust series whose whole-process column passed proposes the capture run in
  `next` exactly as it does for a native series (check what it prints for native and make
  Rust the same).
- When no finding has that site, the existing way to add one from growth evidence
  (`dla ledger add --origin growth-series --evidence <the alloc-stack evidence>`) must give
  the new finding the site of the top project frame without the caller typing it (check what
  the native path does; if the caller has to pass `--site` there too, keep that and make the
  summarize output print the exact command with the site filled in).
- Tests: a small Rust crate in a temp dir with one function that leaks a boxed value per
  call (for example `Box::leak` or a `Vec` in a static that is never drained) and one that
  does not: `--capture alloc`, `summarize alloc-stack-diff` names the leaking function and
  line and not the other; the tie to a finding at that site; skip when cargo or the shim's
  build tools are missing, the way the existing tests skip.

## Check on the real tree
In `<sp>/u41/` (your tokio-util copy; make a fresh copy if the old run directory is in the
way): the grandchild unit again (`parent.child_token().child_token()`, cancel the parent,
drop all), `harness run --mode fresh --capture alloc`, `summarize alloc-stack-diff`: record
which function and `file:line` it names for the 72 bytes per unit, the `next` it prints, the
finding that `ledger add` creates (its site must come from the tool), and then go on as the
skill's flow would with the commands the tool prints (`ledger test`, the sibling, `alloc-test`,
`ledger confirm`): how far does it get, and what stops it if something does. Record exact
outputs.

## Acceptance
`cargo fmt --all --check` (your files); `cargo clippy --workspace --all-targets -j 2 --
-D warnings`; `cargo test -j 2 --no-fail-fast -p dla-workload -p dla-cli` (and dla-core if you
touched it); `target/debug/dla schema --check`; `python3 bench/runner/test_grade.py`;
`dla-bench dynamic --only rust-` and `--only native-` with your binary (copy to
`<sp>/u41/bin/`, run from /home/user/dla with `DLA_PLUGIN_ROOT=/home/user/dla`).

## Report
What changed (files, functions), the build flags and their measured cost, tests and that you
saw each fail, the real-tree outputs, what you did not do and why, what is unverified.
