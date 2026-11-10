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
- Command mode (bin-only crates, or no driver): the allocator shim in `harness/native/` is preloaded
  into the measured process only (`LD_PRELOAD`). It sees the C allocator, so it covers Rust on the
  system allocator but not a custom one (jemalloc, mimalloc) or a statically linked binary.
- Command mode reads live bytes at exit: a program that drops its buffers before exiting shows
  little in `native.live_bytes`; read `native.peak_bytes` or use driver mode.
- Purges: `trim` (glibc `malloc_trim`).

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
