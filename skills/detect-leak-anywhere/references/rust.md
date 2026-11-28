# Rust

Worker name and runtime id: `rust`. Facts from `harness/rust/README.md`, `harness/native/README.md`
and `docs/VERIFY.md`.

Archetype rows (what to look for, how to confirm, how to rule out): read `references/rust-archetypes.md` only after `dla gate check --stage classify` passes.

## Harness
- Driver mode (`dla harness gen --lang rust --driver`): `src/bin/dla_harness.rs` in the scratch copy,
  only for crates with a library target. It installs a counting `#[global_allocator]` over `System`
  and writes a `checkpoint` per round with `native.live_bytes`, `native.live_blocks`,
  `native.peak_bytes` and the Linux `os.*` set. Do not preload the shim into it. Fill the hooks
  (`state`, `setup`, `unit`; `tokio` only for tokio projects, unverified).
- Features: `dla harness gen --lang rust --driver --features a,b` (and `--no-default-features`) passes them
  to the driver build; the build command in `harness.json` carries them and `dla harness run` reuses it. A target in a
  module the crate declares behind `#[cfg(feature = "x")]` (or inside a `cfg_x! { .. }` macro block the crate defines with that
  attribute) says "behind feature x: generate with
  --features x" in its block (and `gen` prints it as `feature_note`). That reader is a line scan of the `mod`
  declarations up to `lib.rs`: it does not see `#[path]`, a module declared in another macro or inline, a gate on the item
  itself, `cfg_attr`, or a feature another feature turns on.
- Code behind a feature inside functions: a crate whose type is a no-op unless its feature is on gates no `mod`, so a driver built without the feature never runs the code of the target. The block of the driver and the `_attention` of `dla harness gen` name each feature that the `default` list does not turn on, with its gated sites in the target's file and in the crate (`cfg(feature)`, `not(feature)`, `cfg_attr` and `cfg!`; `test` conditions are skipped) and the exact `dla harness gen --lang rust --driver --workload <W> --features <list> --probe <F>` command, which repeats the `--workload`, `--no-default-features` and features of the generating call so that it runs as printed; `dla harness gen` prints it whole under `regenerate`, and its `_attention` line (one per feature) only says that the feature is off. In a workspace whose root has no package, driver mode needs `--workload rust:test:<member>/Cargo.toml`, and the refusal names the member crates that have a library target. A target that reads not executed leads with that command, and the queue keeps it open, whatever the number of drivers that missed it, until a harness that targeted it was built with the feature.
- Toolchain: the driver is built with the toolchain the project pins (`rust-toolchain.toml`), and the driver holds to Rust 1.70 and newer, writing an `unsafe extern` block only for a crate of edition 2024 or later; a failed build names the pinned channel.
- Coverage: after the measured process `dla harness run` builds the driver again with
  `-C instrument-coverage` into `harness/H<n>/cov-build/`, runs one round (no warm-up, no captures) and reads
  the functions that ran with the `llvm-profdata` and `llvm-cov` of the toolchain that builds it
  (`rustup component add llvm-tools`; the tools on PATH only when their LLVM version is rustc's). Targets without a probe
  read `executed` or `not_executed`; a build, run or tool that fails is a limitation line and the targets read `unknown`.
  The same export gives, per writer line of a target, how often it ran: the count of the innermost code region that covers the line, written beside the functions in `cov/p1-<mode>-cover.rust.json` and `executed-S<n>.json` (`lines`; a line outside every region is unknown, never 0). A function that ran whose writer lines all have a count of 0 reads `writer_not_executed` (a writer behind a full buffer, a state that the set-up call removes), and in fresh-key mode so does one whose writer lines ran fewer than one time per ten units of the coverage round; the target keeps its queue place and its focus harness shows the writer line with the lines above it that decide whether it runs. For a guard that fills when nothing drains it (a full buffer, a backlog), run the set-up with the consumer absent; a slowed consumer is a different state, not a substitute.
- Command mode (bin-only crates, or no driver): the allocator shim in `harness/native/` is preloaded
  into the measured process only (`LD_PRELOAD`). It sees the C allocator, so it covers Rust on the
  system allocator but not a custom one (jemalloc, mimalloc) or a statically linked binary.
- Command mode reads live bytes at exit: a program that drops its buffers before exiting shows
  little in `native.live_bytes`; read `native.peak_bytes` or use driver mode.
- Allocation stacks: `dla harness run H1 --mode fresh --capture alloc` (a Rust holder has no size probe, so this is what
  names the site of a growing column). Driver mode runs the same binary four more times (1 to 4 rounds of `step` units, no warm-up)
  with the allocator shim preloaded and narrowed to it, into `out/p<k>-alloc/`; command mode uses the shim dumps of its
  points. Measured points are untouched (the shim slows an allocation-heavy unit about ten times, in those extra processes only). Then run the
  `dla summarize alloc-stack-diff` command it prints. A crate with its own `#[global_allocator]`, or a build without
  debug line tables, shows fewer project frames (limitation lines say so).
- An unexplained heap residual: `dla trend` writes the residual rows of its series. While a heap row is over its
  threshold (and its unattributed growth over the series is 64 KiB or more) and the harness has no summarized capture, the
  first `next` entry of `dla trend` is `dla harness run <H> --mode <mode> --capture alloc, then the `dla summarize`
  commands it prints` (once the capture was taken, it is the `dla summarize alloc-stack-diff` command of that capture), the final gate lists a `residual_capture` item, `dla ledger resolve <O> --as unresolved` is refused
  (`capture-not-summarized`) and `dla harness gen --next` is refused (`capture-first`). A Rust run takes `alloc` only
  (`alloc,heap` prints a limitation). The item goes when `dla summarize alloc-stack-diff` has read the `out/p<k>-alloc/shim-dump.jsonl`
  files of the harness, or when the capture run itself recorded it could not write stacks (no shim in the agent
  directory, fewer than three allocation-stack processes that left stacks, no measured process that wrote a dump: a limitation that starts
  with `--capture alloc`); a crate's own `#[global_allocator]` does not release it. The four allocation-stack processes run
  1, 2, 4 and 8 steps of work (each its own process), so a buffer whose capacity doubles has been resized between
  any two of them. A `Vec` field that grows as one buffer (no block per push) gives the summary one stack with a
  constant block count and a capacity that doubles: the dump-series rule passes a constant count of blocks whose bytes
  rise at every pair (`"resized": true` in the passed column), so it ties to the finding at the line of the push like a
  field that keeps one block per push (a `Box`, a `String`), and the finding has the growth lead (`dla alloc-test` decides
  it; closing it as no growth is refused). A buffer cleared with its capacity kept, or one that stood still between two
  points, does not pass (the `needed` text says which). `dla summarize alloc-stack-diff` of a capture with one flat
  pair of a constant-count stack says to take points whose work differs by a factor of two or more (only when they differ
  by less than that). A summary marked `blind` (its totals stayed flat while the process's own live bytes rose: state the exit freed, such as a thread-local) is no evidence of no growth: a generated driver hands the shim a snapshot after its last unit, and `dla alloc-test` reads the process's own series.
- Purges: `trim` (glibc `malloc_trim`).

A `thread_local!` holder, and a container of a module or type that owns one, is kept per thread (the census records the holder and its owner, not the writers of a Rust holder): the driver block says so for such a target, and the units must run on a new thread per unit (`std::thread::spawn`, joined) and on a small pool of threads that live across units, not only on the caller's thread (a first pass on one thread leaves such a target pending for its second look).

## Attach backends
| Backend | Tier | Notes |
|---|---|---|
| `dla attach heaptrack --pid <P> --op record [--duration 30s]` | 3 | allocation stacks, not contents; refused while `kernel.yama.ptrace_scope` > 0; DLA stops it with its own gdb call and never kills the wrapper |
| `dla sample --pid <P> --counters tokio` | 2 | task counts only if the process exposes them |

## Artifacts and summarize ops
For a bin crate started directly, `dla harness run H1 --tool valgrind` (memcheck XML and the vgdb
chain) and `--tool heaptrack` write files that `dla summarize alloc-stack-diff` and `heap-graph`
read; a `dhat-rs` JSON file goes to `alloc-stack-diff` too. Commands, rows and what each proves:
`references/backends.md` and `references/native.md`.

- valgrind prints Rust data symbols in v0 mangling (`_RNvNtCs..._15outline_service8registry8REGISTRY`);
  `dla` demangles them and drops the crate name. A static `REGISTRY` in `src/registry.rs` ties by
  holder to the finding whose owner is `src.registry.REGISTRY`.
- Rust allocator and library frames (`alloc::`, `with_capacity`, `grow_one`) are skipped when the
  site is chosen, so the site is the first project frame. They are never learned as writers.
- A leak held by a static collection, interner or registry is the vgdb `holder:` case; a leak whose
  blocks are unreachable is definitely lost and attributes by site only. Its holder kind is
  `native-handle/global-ref` (L-OWNERSHIP, a raw pointer nobody frees) or `cycle-only` (L-CYCLE,
  an `Rc` cycle); record it with `ledger annotate F --holder <kind> --evidence E..`, as `ledger confirm` and
  `ledger label` need it.
- A Rust `cdylib` that a JVM project loads through JNI is measured from the JVM worker with the shim, and
  `dla correlate H1` there ties its native site to a managed finding (`references/backends.md`, Correlate).
  Record the Rust-side census finding of the same site as the cause's location; do not confirm it from
  the JVM worker's column.
- Driver-mode runs use the counting allocator and have no valgrind or heaptrack capture.

## Blind spots
- No live, typed retention view: retention is inferred from allocation-site growth and code reading.
- Leak checkers (Miri, LeakSanitizer) see only memory unreachable from statics; a leak held by a
  static collection, interner, channel or registry is invisible to them.
- Reference-count cycles are confirmed only in tests, under a nightly sanitizer or Miri.
- Allocation profiling groups by allocation site, not by owner.
- Entry points the loop exercised are not measured, and memory a linked C library allocates
  outside the Rust allocator shows only in the operating-system columns.
