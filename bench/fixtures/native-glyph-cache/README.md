# native-glyph-cache

Category `non-leak`, look-alike `N-WARM` (no pair). Runtime native: C11, `make`,
system compiler.

## Mechanism (never shown to the model)

`src/cache.c` is a glyph bitmap cache with a fixed entry capacity (default 2048,
`--capacity`). `cache_lookup` rasterizes a missing glyph (`calloc` of the entry at
line 118, bitmap allocated by `raster_glyph`), then evicts from the
least-recently-used end while the count is at capacity (line 127,
`evict_oldest`, which unlinks the entry from its hash bucket and the recency
list and frees both the bitmap and the entry). Memory therefore rises while the
cache fills, then stays flat: a warm-up, not a leak. `cache_destroy` releases
everything.

Expected answer: no confirmed leak; the growth is ruled out with its
discriminator (entry count plateaus at capacity, evictions keep pace with
misses, RSS slope in the second half is zero).

## Commands and measurements

```
$ cd project && make && make test
all tests passed
```

Peak resident size (`VmHWM` sampled from `/proc/<pid>/status` by a wrapper kept
outside the repository), default capacity 2048:

| command | entries | evictions | peak RSS |
|---|---|---|---|
| `./textrun --iterations 100` | 1978/2048 | 0 | 2544 kB |
| `./textrun --iterations 300` | 2048/2048 | 3130 | 2640 kB |
| `./textrun --iterations 500` | 2048/2048 | 6440 | 2732 kB |
| `./textrun --iterations 2000` | 2048/2048 | 30937 | 2804 kB |
| `./textrun --iterations 8000` | 2048/2048 | 129692 | 2896 kB |
| `./textrun --iterations 32000` | 2048/2048 | 524415 | 2912 kB |

The cache fills within about 100 runs; between 2000 and 32000 runs (16 times the
work) peak RSS moves by 112 kB, with no trend (allocator noise). The cached
bytes hover around 700-800 kB throughout.

The plateau follows the configured capacity:

| capacity | 2000 runs | 8000 runs | 32000 runs |
|---|---|---|---|
| 512 | 2024 kB | 2076 kB | 2096 kB |
| 2048 | 2804 kB | 2892 kB | 2916 kB |
| 8192 | 5436 kB | 5876 kB | 5972 kB |

Progress lines (`TEXTRUN_PROGRESS=1 ./textrun --iterations 3000`, written to
stderr every 500 runs) show `entries=2048` and bytes between 732176 and 790980
at every checkpoint while evictions grow by about 8000 per 500 runs.

Valgrind, `valgrind --leak-check=full ./textrun --iterations 2000`:

```
total heap usage: 65,973 allocs, 65,973 frees, 13,970,100 bytes allocated
All heap blocks were freed -- no leaks are possible
```

## Cleanup

`make clean` removes `textrun`, `test_cache` and the object files; none are
tracked.
