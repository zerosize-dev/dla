# Native (C, C++, FFI and Rust on the system allocator)

Worker name and runtime id: `native`. Facts from `harness/native/README.md` and `docs/VERIFY.md`.

## Harness
- Command mode: `libdlashim.so` (compiled with `cc` by `dla`, `-fno-optimize-sibling-calls`
  required) is preloaded into the measured process only (`LD_PRELOAD`; macOS `DYLD_INSERT_LIBRARIES`,
  written but not run on Linux). It interposes malloc, free, calloc, realloc, posix_memalign and
  relatives, and writes one `exit` record: `native.live_bytes`, `native.live_blocks`,
  `native.peak_bytes`, the `os.*` set, and the largest 1024 allocation stacks (`raw_sites`), which
  `dla` symbolizes with `addr2line` and maps to the first project frame.
- `DLA_SHIM_ONLY=<basename>` limits measuring to one executable; `DLA_SHIM_SAMPLE=k` samples one
  allocation in `k` (totals stay exact).
- C driver (`dla_harness.c`): glibc `mallinfo2` and `malloc_trim`. Purges: `trim`, `jemalloc`
  (untested), `none`.
- The shim does not load for statically linked executables, Go, or runtimes with a private
  allocator (jemalloc or tcmalloc linked statically, JVM arenas, pymalloc below 512 bytes); musl is
  untested. Windows has no shim: operating-system columns only, recorded as a gap.

## Attach backends
| Backend | Tier | Notes |
|---|---|---|
| `dla attach heaptrack --pid <P> --op record [--duration 30s]` | 3 | allocation stacks, not contents; refused while `kernel.yama.ptrace_scope` > 0 even for root; DLA stops it through a gdb shim, never by killing the wrapper |
| `dla sample --pid <P>` | 2 | `os.*` columns only (rss, fds, threads) |

## Artifacts
- heaptrack output (`.gz` or `.zst`) and its print summary land under `attach/P<n>/`.
  `dla summarize` reads HPROF only: other formats are M5. Record the files; do not summarize them
  by hand.

## Blind spots
- Command mode reads live bytes at exit: memory freed before exit shows only in the peak columns.
- The shim slows allocation-heavy programs a lot; shorten the run with a smaller `--step` in
  `dla harness gen` if it times out.
- Allocation stacks group by allocation site, not by owner; the holder must be read from code.
- Memory a program maps or allocates outside the C allocator (mmap, custom arenas) shows only in
  the `os.*` columns.
- JNI and FFI: the native side is measured only when the process runs through the shim.
