# Native (C, C++, FFI and Rust on the system allocator)

Worker name and runtime id: `native`. Facts from `harness/native/README.md` and `docs/VERIFY.md`.

Archetype rows (what to look for, how to confirm, how to rule out): read `references/native-archetypes.md` only after `dla gate check --stage classify` passes.

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

## Artifacts and summarize ops
Capture with the harness, then read the files (`references/backends.md` has every adapter and row):

| Capture | Command | What it proves |
|---|---|---|
| `dla harness run H1 --tool valgrind` | `alloc-stack-diff` on the `valgrind.xml` files | growth of leaked bytes by allocation stack; ties by site (first project frame) |
| the same run, `vgdb-1-leak.txt`, `vgdb-2-blocks.txt`, `vgdb-3-who.txt` | `heap-graph` on all vgdb files of every point | who points at the leaked block: a `holder:static:<symbol>` column that ties by holder to the finding whose owner ends with the symbol's module path and name. A stack or segment root ties nothing |
| `dla harness run H1 --tool heaptrack` | `alloc-stack-diff` on `heaptrack.txt` files, then on `heaptrack-diff.txt` | allocation-stack growth; ties by site. Raw `heaptrack.*.gz`/`.zst` need `heaptrack_print` |
| shim `harness/H1/out/p<k>-<series>/dump.jsonl` (a plain native run prints no command for it; pass the paths by hand) | `type-histogram-diff`, `non-heap` | stack groups (ties by site, needs `addr2line`), descriptor counts (never tie) |
| `dhat`, LeakSanitizer reports | `alloc-stack-diff` | live-at-exit or leak records by stack; tie by site |

- Memcheck XML from separate runs is a run-length series (runs of different length), not dumps of
  one process. Heaptrack text is rounded (about 1%); folded `-F` files are exact bytes without counts.
- A definitely-lost block has no holder (nothing points at it), so attribution is by allocation site,
  not by a vgdb holder. Two findings at one site (a detector and a census finding) are
  both tied: `candidates` lists every one, `tie` names the `--finding` one if it is among them, else the
  first by id. `ambiguous` holds only type matches that hit several findings.
- The vgdb chain runs only when the harness starts the program directly. With a wrapper
  (make, sh) there is no chain, and a limitation line says so. Valgrind plus vgdb are Linux.
- Stacks match across points by their first 8 resolved frames; without `addr2line` they stay as
  module offsets and nothing ties (coverage row `addr2line` missing).
- Census owners are file-path based: a C static is `src.<file>.<name>`, a Rust static
  `src.<module>.<NAME>`. A symbol in the root module ties only when exactly one finding ends with it.

## Blind spots
- Command mode reads live bytes at exit: memory freed before exit shows only in the peak columns.
- The shim slows allocation-heavy programs a lot; shorten the run with a smaller `--step` in
  `dla harness gen` if it times out.
- Allocation stacks group by allocation site, not by owner; the holder must be read from code.
- Memory a program maps or allocates outside the C allocator (mmap, custom arenas) shows only in
  the `os.*` columns.
- JNI and FFI: the native side is measured only when the process runs through the shim.
