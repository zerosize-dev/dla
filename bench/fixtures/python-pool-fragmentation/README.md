# python-pool-fragmentation

Category: `non-leak`. Look-alike cause: `N-ALLOC`. No pair.

## Mechanism (for authors)

`Importer.import_batch` parses a batch of 60 to 300 statement lines into row dicts (each
memo is a 14 KB string, so a large batch holds several MB at its peak), totals them by
account, and returns. The rows, the totals and the input lines are all garbage when the
call returns: nothing keeps them. The importer keeps only the last 64 batch summaries in
a `deque(maxlen=64)`, and each summary holds a copy of one memo (14 KB) that was
allocated in the middle of the burst. Those copies sit among the row strings in the C
allocator's heap (strings over 512 bytes bypass pymalloc), so after the burst the free
space in that heap is spread between live blocks and the allocator does not give it back
to the system. Resident memory reaches the size of the largest burst and stays there; the
Python heap itself is back to its base after every batch. A harness sees a rise in
resident memory during the first batches and a flat line after that, and `malloc_trim(0)`
returns the free pages.

Nothing is retained: the deque is bounded, traced memory is flat, and the rise stops at
the largest batch the generator can produce (300 rows, reached in the first 40 batches).

## Measurements

Harness outside `project/` (drives `statements.load.batches` through `Importer`,
`gc.collect()` before each reading, `tracemalloc` started after the imports; `rss_kb` is
the resident pages from `/proc/self/statm`; `malloc_trim(0)` called through `ctypes` at the
end):

```
python3 -B harness.py <project dir> 300
n=300 i=37  traced_kb=716  rss_kb=30460 recent=37
n=300 i=74  traced_kb=1232 rss_kb=33168 recent=64
n=300 i=148 traced_kb=1229 rss_kb=33564 recent=64
n=300 i=222 traced_kb=1229 rss_kb=33568 recent=64
n=300 i=296 traced_kb=1231 rss_kb=33888 recent=64
n=300 rss before malloc_trim=33888 after=21688 (start 18048)
python3 -B harness.py <project dir> 1000
n=1000 i=125  traced_kb=1231 rss_kb=33200 recent=64
n=1000 i=500  traced_kb=1233 rss_kb=34228 recent=64
n=1000 i=875  traced_kb=1233 rss_kb=34492 recent=64
n=1000 i=1000 traced_kb=1233 rss_kb=34112 recent=64
n=1000 rss before malloc_trim=34112 after=21812 (start 17992)
```

Traced memory is 1.2 MB after every batch from the 74th on, at 300 and at 1000 (the 64
summaries; 716 KB at batch 37 while the deque fills). Resident memory goes from 18 MB at
the start to 33 MB in the first 74 batches and then stays within 1 MB (33.2 MB at batch
125, 34.1 MB at batch 1000), so the slope of the second half is zero. `malloc_trim(0)`
returns 12 MB. Without `tracemalloc` the plateau is 30.8 MB at 500, 30.9 MB at 2000 and
31.0 MB at 4000 batches, and the trim returns 7 to 9 MB.

Python 3.13.18, glibc. Workload: `python3 -m statements.load --iterations 300` takes about
1 s (about 3 ms per batch).
Tests: `python3 -m unittest discover -s tests` from `project/` (8 tests, pass).
Size: 7 files, 195 lines.
