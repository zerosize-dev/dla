# native-split-freelist

Category `archetype-free`, cause `UNCLASSIFIED` (no pair). Runtime native: C11,
`make`, system compiler.

## Mechanism (never shown to the model)

`src/pool.c` is a buffer pool for the relay's messages. It takes 64 KiB chunks
from `malloc` (line 67), cuts blocks off a bump pointer and keeps released blocks
on one list, newest first (line 41). A request looks at the first 16 entries of
that list (`PROBE_LIMIT`, line 48), takes the first block that is large enough
and, when the block is larger than needed, cuts the rest off as a block of its
own (line 88). A request that nothing in the window fits is served from the
chunk. Released blocks are never merged with their neighbours.

The workload mixes status frames (40 to 200 bytes, 90% of messages) with bulk
records (1.5 to 3 KiB, 10%). A status frame takes whatever block is at the
front of the list, so every released bulk block is cut up by the next few
status frames and the pieces are too small for any later bulk record. Bulk
records are served from fresh chunk memory again and again, and the pieces pile
up on the list behind a window that never looks past its first 16 entries. The
amount of data buffered at any moment is flat (about 2000 messages, 0.7 MB
including headers); the number of chunks grows by one for about every 360
messages.

Expected diagnosis: the growth is chunk memory, one `malloc` site
(`pool.c:67`, `start_chunk`) whose block count climbs linearly while the bytes
in use stay flat. Every chunk is still referenced (valgrind: nothing lost), so
it is not a missing release; the pool's choice of blocks keeps stranding free
memory.

Cause `UNCLASSIFIED`: no shipped native row covers a pool that holds free
memory it cannot reuse. `L-HANDLE` (a release obligation not met) does not
apply, and `N-ALLOC` (allocator retention) is ruled out by the trim check below:
the free memory is not the allocator's, it is the pool's, and `malloc_trim` does
not return it. Holder `native-handle/global-ref` is what the census and the
allocation-stack evidence give for a `malloc`ed chunk.

## Commands and measurements

Build and test:

```
$ cd project && make && make test
all tests passed          (7 tests)
```

Workload, peak resident size (`VmHWM` sampled from `/proc/<pid>/status` by a
wrapper kept outside the repository); chunk counts and bytes in use are read
by a small program outside the repository that links the project sources:

| command | in flight | chunks | bytes in use | peak RSS |
|---|---|---|---|---|
| `./msgrelay --iterations 50000` | 2000 | 141 | 775 KB | 10468 kB |
| `./msgrelay --iterations 100000` | 2000 | 278 | 749 KB | 19232 kB |
| `./msgrelay --iterations 200000` | 2000 | 551 | 740 KB | 36728 kB |
| `./msgrelay --iterations 400000` | 2000 | 1109 | 781 KB | 72432 kB |
| `./msgrelay --iterations 800000` | 2000 | 2220 | 756 KB | 143596 kB |
| `./msgrelay --iterations 1600000` | 2000 | 4442 | 740 KB | 285840 kB |

Growth is linear in the tick count: 800000 to 1600000 adds 142 MB, about 178
bytes per tick; 400000 to 800000 adds 71 MB (178 bytes per tick). The default
200000-tick run takes 0.27 s; the 1600000-tick run takes 2.1 s.

Control run (same source with released blocks kept on one list per 16-byte size
class and the smallest sufficient block taken first, measured in a scratch copy):
peak RSS 2596 kB at 50000 ticks, 2736 kB at 100000, 2936 kB at 400000, 3072 kB
at 1600000. Flat, so the growth comes from how the pool picks blocks and not from
the workload.

Trim check (scratch copy that calls `malloc_trim(0)` after the 400000-tick run):
`VmRSS` 72680 kB before and 72680 kB after. The memory is not allocator
retention.

Valgrind (`valgrind --leak-check=full ./msgrelay --iterations N`):

```
N=50000:   definitely lost: 0 bytes in 0 blocks; still reachable: 9,256,576 bytes in 142 blocks
N=200000:  definitely lost: 0 bytes in 0 blocks; still reachable: 36,126,336 bytes in 552 blocks
```

The reachable blocks are the chunks, each held by the pool's chunk list.

## Cleanup

`make clean` removes `msgrelay`, `test_pool` and the object files; none are
tracked.
