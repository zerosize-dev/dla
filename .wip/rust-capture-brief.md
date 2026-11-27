# Builder S3: unexplained heap growth of a Rust harness is not left unresolved before its allocation stacks are read

Repository: /home/user/dla, branch main. The working tree holds the uncommitted, verified work of
the previous builder (S2: a stair reading shared in crates/dla-stats/src/stair.rs, the trend gate
rescue in classify.rs, a `gate_note` field, tests, docs). Keep it: do not revert or rewrite it;
build on it. Do NOT commit, push or switch branches. Do not touch harness/ templates' behaviour
unless this brief says so, bench/fixtures/, any fixture README or answer key, or
skills/detect-leak-anywhere/SKILL.md. Read each file just before you edit it. Nothing else runs on
this machine. The repository must not name the project of an evidence run or its APIs: say "a Rust
library", "a `Vec` field", nothing more.

## What a real run showed

An unattended run on a Rust library built a driver (H4) for the library's documented leak, a `Vec`
field (finding F41, its writer lines known) that grows while the consumer does not drain. The
driver ran the leak's writer. `dla trend S5` passed `os.rss` (234 KB per unit), and the residual
row it wrote for the heap read 237 KB per unit, all of it unattributed, over a threshold of 150 KB:
open item O1, "unexplained growth ... add a hypothesis or record it unresolved". The trend's
`next` said `dla harness run H4 --mode fresh --capture alloc   # allocation stacks name the site
of the growth of os.rss`. The model started that command, the agent's tool moved it to the
background after 120 s, and the model went on: it ran `dla ledger resolve O1 --as unresolved
--reason inconclusive --remedy "<a guess that the driver's set-up caused it>"` with 161 of 167
budget minutes left, never ran `dla summarize`, and the run ended COMPLETE with no leak; F41 was
closed "its code ran and showed no growth". Evidence: /root/dla-real/r7-fastrace-keep/
(ledger.jsonl lines 97-117, trend/T5.json; read it, do not copy it into the repository).

For the JVM, Node, Go and Python that resolve is refused (`capture-not-summarized`) until the
harness has a summarized capture, the final gate lists a `residual_capture` item, and
`dla harness gen --next` is refused with `capture-first` (crates/dla-core/src/gate.rs
`uncaptured_residuals`, `supports_capture`, `UncapturedResidual::fix`; crates/dla-cli/src/
ledger_cmd.rs `refuse_uncaptured_residual`; harness_cmd.rs ~264; trend_cmd.rs `capture_entry`;
docs/VERIFY.md row "A heap residual above its threshold could be left unresolved with no
summarized capture"). `supports_capture` leaves out Rust, though `dla harness run --capture alloc`
writes allocation stacks for Rust in both modes (crates/dla-workload/src/harness/run.rs
`rust_alloc_points` for a driver: four short processes with the allocator shim; command mode:
the shim dumps of its points), and `dla summarize alloc-stack-diff` reads them
(harness/rust/README.md "Allocation stacks"). The test
`residual_capture::a_runtime_without_capture_is_not_held_to_it` pins Rust as a runtime without
capture: that premise is false (the Rust capture is older than the rule).

## The change

1. **Rust is held to the capture rule.** `supports_capture` (or whatever decides it) includes
   Rust. The fix text is per runtime: for Rust `dla harness run <H> --mode <mode> --capture alloc,
   then the dla summarize commands it prints` (a Rust run takes `alloc` only; `alloc,heap` makes
   it print a limitation). Make `UncapturedResidual` carry what it needs for that and check every
   place that prints its fix (the gate item, the resolve refusal, the `capture-first` refusal, the
   trend's `next` and `_attention`).
2. **Native (C and C++).** Find out whether a native harness has a route that ends in an
   allocation-stack record that `measured_holders::Scan::summarized` counts for the harness
   (command mode: shim dumps? `--tool valgrind|heaptrack` then `dla summarize alloc-stack-diff`?).
   Include native in the rule only if that route works end to end today (show it with a test);
   otherwise leave it out and say why in your report. Do not build new capture machinery.
3. **A capture that cannot give stacks does not hold the run forever.** Find what the existing
   runtimes do when the capture ran and wrote nothing usable. For Rust, `rust_alloc_points`
   records limitations (no shim in the agent directory, every allocation-stack run failed, the
   crate sets its own `#[global_allocator]`). A capture run of the harness that recorded that it
   could not write stacks (the shim missing, or no allocation-stack process completed) releases
   the item: the resolve is allowed and the gate item is gone, and the refusal or item text says
   what to do otherwise. Use what the workload evidence of `dla harness run` already records
   (its `limitations`, `capture`, `gaps`); add a field only if nothing there can tell, with
   `#[serde(default)]` on read. A crate with its own global allocator still gets stacks of the
   system allocator: that alone does not release it.
4. **What the summary leads to.** Check what `dla summarize alloc-stack-diff` on a Rust driver's
   `out/p<k>-alloc/shim-dump.jsonl` files records, and whether a growing allocation site at a
   writer line of a finding (the push into the `Vec` field) then gives that finding the lead and
   route the other runtimes get from an allocation stack at a finding's growing line (look for the
   rule of the earlier round "allocation stack for a finding at a growing line" in crates/dla-core:
   grown.rs `site_column_add`, `is_alloc_site_column`, `finding_is_at_site`, and the gate). Report
   the chain as it is for Rust. If a step is missing, say exactly which; build it only if it is
   small and of the same kind as the other runtimes' step, otherwise stop at the report.

## Tests (each must fail with its change taken out: check it and report)

- residual_capture.rs: a Rust scene is held to it (gate item with the `--capture alloc` fix, resolve
  refused with `capture-not-summarized`); an allocation-stack summary computed from the Rust
  harness's `out/p<k>-alloc/` files clears it; a Rust capture run that recorded it could not
  write stacks releases it. Replace the test that pins Rust as a runtime without capture with one
  for a runtime that really has none (say which), keeping what it proves.
- Native: per point 2.
- Point 4: one test at the level where the chain is decided, if you built a step.
- The dynamic bench policy already takes `residual_capture` items (bench/runner/src/dynamic/
  policy.rs `take_capture`): check that it takes the Rust fix text as printed.

## Checks (after your last edit)

`df -h /` first (about 3.4 GB free; a full dla-cli test build takes 4-5 GB: if under 6 GB run
`cargo clean -p dla-cli -p dla-core -p dla-workload -p dla-bench` first, the approved way to free
the build directory; never delete build directories by hand). Then `cargo test -p dla-stats`,
`cargo test -p dla-core`, `cargo test -p dla-cli --no-fail-fast` (whole suite),
`cargo test -p dla-workload --no-fail-fast`, `cargo test -p dla-report`,
`cargo test -p dla-bench --lib`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo fmt --all --check`, `./target/debug/dla schema --check`. Then the whole dynamic set from
the repository root: `./target/debug/dla-bench dynamic --dla-bin ./target/debug/dla --parallel 2
--json <scratch file>` (expected 63 pass and 1 expected failure, `rust-tree-parent-links`; say
which Rust or native fixtures now take a capture, how long it added, and whether a decoy now
gets a lead); delete its logs after. Docs: one row at the end of docs/VERIFY.md (re-read the end
first), the VERIFY row above where its runtime list is now false, and
skills/detect-leak-anywhere/references/rust.md (and native.md if native is included) where they
describe the residual or the capture-first rule.

## Rules

Shared `target/` only. If a build or the dynamic run fails for lack of space, stop and report it.
No `rm -rf "$VAR/..."`. Never kill processes by name. Never skip or weaken a test. No project
names, attribution lines or model names in the repository. Never use a foreground sleep; wait on
long commands with run_in_background. Scratch:
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u162/.

## Report (final message)

Files changed; the rule as implemented; native included or not and why; the release rule for a
capture that wrote no stacks; the Rust chain after the summary (point 4) and anything missing;
which existing test expectations changed and why; tests with their mutation checks; every command
with pass/fail counts, run after the last edit; the dynamic result; anything you are unsure of.
