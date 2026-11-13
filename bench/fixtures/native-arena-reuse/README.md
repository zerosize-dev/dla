# native-arena-reuse

Category `non-leak`, look-alike `N-ALLOC` (no pair). Runtime native: C11,
`make`, glibc.

## Mechanism (never shown to the model)

Each batch of the settlement run (`src/batch.c`) reads 10500 to 13500 payment
records into memory, one `malloc` of 40 to 570 bytes each (`src/record.c`,
line 7), totals them and frees every record and the pointer array at the end
of the batch. Every 400 records a 24-byte running summary is handed to a ring
of 32 (`src/carry.c`), which frees the summary it replaces.

The summaries are allocated between the records, so a few live blocks sit in
the middle and near the top of the heap while the batch's records are freed
around them. glibc cannot give back the pages below the topmost live block with
its usual trim at the top of the heap, so the freed space stays inside the
process. The next batch allocates from that space, which is why the resident
size reaches a high-water mark in the first batch or two and then does not move.
Memory is not lost: everything is freed or replaced, and the allocator holds the
free space for reuse.

Expected answer: no confirmed leak. The growth is ruled out with its
discriminator: bytes in use (`mallinfo2`) are tiny next to the bytes the
allocator holds free; the resident size is flat from the second batch to the
160th; valgrind reports every block freed; `malloc_trim(0)` returns the memory.
Cause `N-ALLOC` (allocator retention).

## Commands and measurements

Build and test:

```
$ cd project && make && make test
all tests passed          (4 tests)
```

Workload, peak resident size (`VmHWM` sampled from `/proc/<pid>/status` by a
wrapper kept outside the repository); one iteration is one batch of about
12000 records (about 3 MB of live records at the peak of a batch):

| command | records | peak RSS |
|---|---|---|
| `./settlerun --iterations 1` | 11625 | 5000 kB |
| `./settlerun --iterations 2` | 24522 | 5428 kB |
| `./settlerun --iterations 5` | 60047 | 5472 kB |
| `./settlerun --iterations 10` | 118998 | 5500 kB |
| `./settlerun --iterations 40` | 468534 | 5500 kB |
| `./settlerun --iterations 160` | 1897733 | 5532 kB |

The resident size is at its plateau after two batches: from 5 to 160 batches
(32 times the work) it moves by 60 kB. The 160-batch run takes 2.2 s.

Allocator state at exit (`./settlerun --iterations 40 --stats`, written to
stderr):

```
heap: arena=3731456 in_use=77264 free=3654192
```

Scratch copy that calls `malloc_trim(0)` after the report (`VmRSS` before and
after):

| iterations | before | after |
|---|---|---|
| 1 | 5252 kB | 1996 kB |
| 5 | 5372 kB | 1996 kB |
| 40 | 5376 kB | 1996 kB |

Valgrind (`valgrind --leak-check=full ./settlerun --iterations 10`):

```
total heap usage: 119,302 allocs, 119,302 frees, 35,481,161 bytes allocated
All heap blocks were freed -- no leaks are possible
```

## Cleanup

`make clean` removes `settlerun`, `test_batch` and the object files; none are
tracked.
