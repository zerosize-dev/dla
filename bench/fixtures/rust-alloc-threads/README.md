# rust-alloc-threads

Category `non-leak`, cause `N-ALLOC` (the allocator keeps freed memory), no partner.

## Mechanism

`Pool` starts eight worker threads that take export jobs from one queue
(`src/pool.rs`). Each job renders two to six sections of 6 to 126 kB, joins them
into one document of up to 750 kB (`src/render.rs:28`) and drops all of it before
the next job; the worker keeps a receipt of its last 16 jobs. The buffers are
freed every time, but glibc serves each thread from its own malloc arena and keeps
the pages a thread has freed for the next job (the trim threshold rises after the
first large buffer is freed). Resident memory therefore rises to the working set of
eight arenas in the first few hundred jobs and stays there, while the bytes the
program has allocated and not freed stay at about 20 kB (the receipts). Nothing grows
with the number of jobs: the series is a step, then flat.

Expected: no confirmed leak; the resident memory ruled out with the flat live-byte
series (and the arenas' retained free memory) as the discriminator. A claim of a leak
at the receipt ring or at the buffers is a false alarm.

## Measurements

Toolchain: cargo 1.99.1 (glibc 2.39). Checks and workload (`--iterations N` renders N jobs):

```
cd project && cargo test --release     # 6 passed
cargo run --release -- --iterations 8000
# jobs=8000 bytes=2090174394 digest=952410998b3c9141  (6.7 s)
```

External harness (kept outside the repository): a copy of `project/` with a counting
`#[global_allocator]` (live bytes, live blocks), a read of `VmRSS` before the process exits
(the program ends with `process::exit`, so the workers are still running), and then a call
to glibc's `malloc_trim(0)` and a second read:

| jobs | live bytes | live blocks | RSS | RSS after `malloc_trim(0)` |
|---|---|---|---|---|
| 0 (start) | 3 886  | 26  | 2 380 kB  | 2 380 kB |
| 220       | 19 326 | 300 | 11 592 kB | 6 656 kB |
| 660       | 19 670 | 302 | 12 096 kB | 8 028 kB |
| 1980      | 19 718 | 302 | 12 156 kB | 8 776 kB |
| 8000      | 19 694 | 302 | 12 872 kB | 10 108 kB |

Live bytes and live blocks are flat from the first 220 jobs on (each of the eight workers
already holds its 16 receipts: 8 x 16 x 2 blocks, plus a few dozen of the pool's own).
From 220 to 8000 jobs (36 times as many) the resident set rises by 1.3 MB (11%), 0.5 MB of
it before 660 jobs. The trim returns 2.8 to 4.9 MB, the part of the arenas that holds free pages in
the middle of a heap; the rest is the top of each arena, which `malloc_trim` leaves
mapped.
