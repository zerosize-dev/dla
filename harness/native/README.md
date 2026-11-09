# Native allocator shim and C driver

| File | Role |
|---|---|
| `dla_alloc_shim.c` | allocator shim, compiled by `dla` with `cc` into `libdlashim.so` / `libdlashim.dylib` |
| `dla_harness.c` | loop driver template for C projects (glibc `mallinfo2`, `malloc_trim`) |

The shim measures any dynamically linked program that uses the C allocator, which includes Rust
programs on the system allocator (the default), C and C++ programs. It needs no change to the
project: `dla` preloads it into the measured process only (build steps run without it).

## Shim: build and run

```
cc -std=gnu11 -O2 -g -fPIC -shared -fno-optimize-sibling-calls -o libdlashim.so dla_alloc_shim.c -ldl -lpthread     # Linux
cc -std=gnu11 -O2 -g -dynamiclib -o libdlashim.dylib dla_alloc_shim.c                                               # macOS
LD_PRELOAD=<abs>/libdlashim.so DLA_HARNESS_OUT=<harness dir>/out/<pid>.jsonl ./program ...                           # Linux
DYLD_INSERT_LIBRARIES=<abs>/libdlashim.dylib DLA_HARNESS_OUT=... ./program ...                                      # macOS
```

`-fno-optimize-sibling-calls` is required (the call-stack capture skips a fixed two shim frames);
`-g` is optional. The macOS path (`__DATA,__interpose`, `dladdr`) is written but was not built or
run in this repository's Linux sandbox. Windows: no shim (OS columns only, recorded gap).

Interposed: `malloc free calloc realloc reallocarray posix_memalign aligned_alloc memalign valloc
pvalloc` (macOS: the first six that exist there). The real functions are found with
`dlsym(RTLD_NEXT)`; allocations made while `dlsym` itself runs come from a 128 KiB static arena
(freed pointers from it are ignored, `realloc` of one copies out of it). A thread-local guard sends
nested calls (from `backtrace()`, the dump) straight to the real allocator.

### Environment

| Variable | Meaning |
|---|---|
| `DLA_HARNESS_OUT` | file the exit record is appended to; unset means the shim is inert |
| `DLA_PURGE` | `trim` (glibc `malloc_trim(0)`; macOS `malloc_zone_pressure_relief`), `jemalloc` (`mallctl("arena.4096.purge")` when the process has jemalloc; untested), `none`; anything else becomes a limitation |
| `DLA_ITER` | copied into `x` of the record |
| `DLA_SHIM_ONLY=<basename>` | only a process whose executable (or `argv[0]`) basename matches is measured; others (a `sh -c` wrapper, `make`) stay inert |
| `DLA_SHIM_SAMPLE=k` | capture the call stack of one allocation in `k` (per thread); `native.*` totals stay exact, `raw_sites` cover the sampled allocations (the rest is `shim.unsampled_*`) |
| `DLA_SHIM_SIGTERM=0` | do not install the SIGTERM dump handler (it is installed only when SIGTERM has the default disposition) |
| `DLA_SHIM_DISABLE=1` | stay inert (a driver in the same process writes the records) |

### The record

One JSON line, `phase:"exit"`, `runtime:"native"`, written by the destructor (and on SIGTERM):

- `values`: `native.live_bytes`, `native.live_blocks`, `native.peak_bytes` (requested sizes of live
  blocks, peak of live bytes) and `os.rss`, `os.rss_anon`, `os.rss_file`, `os.rss_shmem`,
  `os.swap`, `os.vm_data`, `os.vm_hwm`, `os.threads`, `os.fds` (bytes; unreadable keys left out), and
  the descriptors by kind as `fd.socket`, `fd.pipe`, `fd.file`, `fd.deleted`, `fd.dev`, `fd.eventfd`,
  `fd.eventpoll`, `fd.timerfd` (always present, 0 when none) plus `fd.memfd`, `fd.inotify`,
  `fd.other` and so on when a descriptor of that kind is open (the same classification as the
  sampler's `fd.<kind>` columns; Linux only, not in a SIGTERM dump). The values are read before
  any purge.
- `purge` (only when `DLA_PURGE` is set): `{kind, before:{os.*}, after:{os.*}, detail}`.
- `raw_sites`: `{ "<stack key>": {live_bytes, live_blocks, allocs, frames:[{a, m, o}]} }`, the
  largest 1024 stacks by live bytes. `frames` are the first 8 return addresses outside the shim
  (innermost first; the stack is captured 12 deep and the two shim frames are dropped). `a` is the
  runtime address, `m` an index into `modules`, `o` the module-relative offset of `a - 1` (the call
  instruction) so that
  `addr2line -e <modules[m].path> -f -C -i <o>` yields the file and line. `dla` picks the first
  frame whose file is inside the project after symbolizing; frames in libc, libstdc++ and Rust std
  come first and are skipped there.
- `modules`: `[{path, bias}]` from `dl_iterate_phdr` (`bias` is the load bias, 0 for a non-PIE
  executable). `o = a - 1 - bias`, which is what addr2line expects for PIE executables and shared
  objects. The vdso (`linux-vdso.so.1`) is listed but has no file.
- `shim`: bookkeeping (`sites_used`, `sites_live`, `overflow_*` when more than 4096 distinct stacks
  were seen, `unsampled_*`, `total_allocs`, `untracked_allocs`, `stale_replaced`, `unknown_frees`).
- `limitations`: strings for everything that makes the numbers incomplete.

### Design choices

- Hash of pointer to (size, site) instead of a per-block header: a header would change the
  alignment guarantees of `posix_memalign`/`aligned_alloc`, break `free()` of blocks that came from
  before the shim resolved the real allocator, and break `malloc_usable_size`.
- The table is sharded 64 ways by pointer hash, each shard with its own spin lock, in `mmap`ed
  memory (never `malloc`). A block is removed from the table BEFORE it is passed to the real `free`
  or `realloc`, so another thread given the same address cannot collide with a stale entry.
- The 4096-entry stack table has lock-free lookup, a lock only to add a stack, and atomic
  counters; the exit dump therefore needs no lock and is safe to run from the SIGTERM handler
  (an `alarm(3)` default-action backstop covers a loader deadlock).

### Limits

- Fork: children do not write a record (the dump runs only in the process that loaded the shim);
  a child that execs another program is measured as that program (its own pid line, same file).
- `_exit()`, `SIGKILL`, `abort()` and crashes leave no record. A program that frees everything
  before `exit` shows the retained memory only in `native.peak_bytes` and in `os.*`.
- Statically linked executables, Go, and runtimes with a private allocator (statically linked
  jemalloc/tcmalloc, JVM arenas, Python's `pymalloc` below 512 bytes) are not covered; musl is not
  tested (`backtrace` and `malloc_trim` may be missing).
- Stacks are the first 8 frames after the shim; deep Rust or C++ call chains may not reach a
  project frame within 8 (the site then stays unattributed).
- Cost: a few hundred nanoseconds per allocation, up to about 1 microsecond with call stacks and
  many threads contending (an 8-thread allocation microbenchmark ran about 50 times slower);
  `DLA_SHIM_SAMPLE` removes the unwinding cost, not the counting. The shim adds a few MB of RSS
  (static tables, dump buffer).
- Sizes are the requested sizes, not `malloc_usable_size`; allocator headroom shows up in `os.*`,
  not in `native.*`.

## C driver `dla_harness.c`

Build it with the project's sources (without the project's `main`) and run it from the scratch
project root; see the header comment. One `checkpoint` record per round; values are
`native.live_bytes` (`uordblks + hblkhd`), `native.free_bytes`, `native.arena_bytes` (from
`mallinfo2`, glibc 2.33+) and the `os.*` set; `DLA_PURGE=trim` calls `malloc_trim(0)` after the
last round and records `purge.before/after`. Env overrides: `DLA_KEY_MODE`, `DLA_STEP`,
`DLA_WARMUP`, `DLA_ROUNDS`, `DLA_REPEAT`.

### Placeholders

| Placeholder | File | Meaning |
|---|---|---|
| `@@DLA_STEP@@` | `dla_harness.c` | units per round (default for `DLA_STEP`) |
| `@@DLA_WARMUP@@` | `dla_harness.c` | warm-up rounds |
| `@@DLA_ROUNDS@@` | `dla_harness.c` | measured rounds |
| `@@DLA_REPEAT@@` | `dla_harness.c` | repeated key space size |

`dla_alloc_shim.c` has no placeholders.

### Hooks

| Hook | Signature | Fill with |
|---|---|---|
| `DLA-HOOK state` | file-scope declarations | variables `setup()` fills and `unit()` uses |
| `DLA-HOOK setup` | `static void setup(void)` | construct the objects under test once |
| `DLA-HOOK unit` | `static void unit(unsigned long i, unsigned long key)` | one unit of the workload for `key` |

The stubs print a message and `abort()` until replaced.

## Tool captures (`dla harness run <H> --tool valgrind|heaptrack`)

For a program the harness starts directly, `--tool` runs three of the points under the tool instead of
measuring a series (Linux; a missing tool refuses with `tool-missing` and leaves a `missing` coverage row).

| `--tool` | Files per point under `out/<label>/` | Summarize command printed |
|---|---|---|
| `valgrind` | `valgrind.xml`, `vgdb-1-leak.txt`, `vgdb-2-blocks.txt`, `vgdb-3-who.txt` | `alloc-stack-diff` on the XML files; `heap-graph` on the vgdb files of every point |
| `heaptrack` | `heaptrack.txt` (`heaptrack_print`), `heaptrack-diff.txt` (first to last) | `alloc-stack-diff` on the texts; a separate command for the diff |

Valgrind runs with `--vgdb=yes --vgdb-stop-at=exit`, and the chain keeps one `vgdb --pid` relay open
(a standalone `vgdb` call lets the process go on, so one stop allows one call). It walks `who_points_at`
from the biggest leaked block, at most 12 queries. A wrapper command (make, sh) gets no chain.

## Inside a JVM (JNI)

When `dla probe` finds a JNI boundary (JVM sources that call `System.loadLibrary` and a Rust
cdylib or C file that exports `Java_` functions), `dla harness gen --lang jvm` preloads this shim
into the JVM with `DLA_SHIM_ONLY=java`, builds the cdylib with
`CARGO_PROFILE_RELEASE_DEBUG=line-tables-only` and sets `-Djava.library.path`. The JVM agent writes
the `exit` record of the `jvm` runtime and the shim writes its own after it, in the same process;
`dla harness run` merges the shim's `native.*` and `fd.*` values, stacks and modules into the JVM
record. The stack of a native allocation has only the frames of the library (the JVM's JIT frames
have no module), so the site is the library's line (`native/src/lib.rs:40`), and the function name
that `addr2line` prints (`Java_shop_codec_Native_open`) names the managed method.
