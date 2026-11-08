# Rust

Worker name and runtime id: `rust`. Facts from `harness/rust/README.md`, `harness/native/README.md`
and `docs/VERIFY.md`.

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

## Artifacts
- heaptrack output (`.gz` or `.zst`) and its `heaptrack_print` summary land under `attach/P<n>/`.
  `dla summarize` reads HPROF only: heaptrack and dhat summaries are M5. Record the files and the
  evidence from the attach; do not summarize them by hand.

## Blind spots
- No live, typed retention view: retention is inferred from allocation-site growth and code reading.
- Leak checkers (Miri, LeakSanitizer) see only memory unreachable from statics; a leak held by a
  static collection, interner, channel or registry is invisible to them.
- Reference-count cycles are confirmed only in tests, under a nightly sanitizer or Miri.
- Allocation profiling groups by allocation site, not by owner.
- Entry points the loop exercised are not measured, and memory a linked C library allocates
  outside the Rust allocator shows only in the operating-system columns.
