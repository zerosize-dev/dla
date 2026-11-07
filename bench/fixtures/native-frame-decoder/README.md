# native-frame-decoder

Category `archetype-leak` (no pair). Runtime native: C11, `make`, system compiler.

## Mechanism (never shown to the model)

`decode_frame` in `src/decoder.c` allocates the output buffer with `malloc`
(line 29), then validates the payload and the trailer checksum. The payload
error path frees the buffer; the checksum-mismatch path (line 40) returns
without freeing it and without handing it to the caller. The workload damages
the trailer of every 16th frame (`--damage-every 16`, the default), so about
6% of the frames take that path and each one strands a buffer of 4 to 32 KiB
(about 18 KiB on average).

Cause `L-HANDLE` (release obligation not met on one early-return path), holder
`native-handle/global-ref`, region `off-heap` (malloc heap, invisible to any
managed-heap view). `site` is the `malloc` statement; `alt_sites` lists the
early return at line 40.

## Commands and measurements

Build and test:

```
$ cd project && make && make test
all tests passed
```

Workload, peak resident size sampled from `/proc/<pid>/status` (`VmHWM`) by a
small wrapper outside the repository:

| command | rejected frames | peak RSS |
|---|---|---|
| `./framedec --iterations 1000` | 62 | 2596 kB |
| `./framedec --iterations 2000` | 125 | 3744 kB |
| `./framedec --iterations 4000` | 250 | 5844 kB |
| `./framedec --iterations 8000` | 500 | 10100 kB |
| `./framedec --iterations 16000` | 1000 | 18660 kB |
| `./framedec --iterations 32000` | 2000 | 35688 kB |

Growth is linear in the iteration count: about 1.06 kB per iteration
(2000 to 8000: 6356 kB over 6000 iterations; 8000 to 32000: 25588 kB over
24000), i.e. about 17.5 kB per rejected frame. The 32000-iteration run takes
4 s.

Control run with no damaged frames (`--damage-every 0`): peak RSS 1544 kB at
2000 iterations, 1540 kB at 8000, 1540 kB at 32000. Flat, so the growth comes
from the rejected-frame path only.

Valgrind (`valgrind --leak-check=full ./framedec --iterations N`):

```
N=500:   definitely lost: 533,504 bytes in 31 blocks
N=2000:  definitely lost: 2,240,512 bytes in 124 blocks (+25,600 indirect from the next record)
         at malloc  by decode_frame (decoder.c:29)  by main (main.c:47)
```

Block counts match the rejected-frame counts of the workload (31 rejected at
500 iterations; 125 at 2000, where the allocator reports one of the blocks as
reachable only through another lost block, hence 124 direct plus 1 indirect).

## Cleanup

`make clean` removes `framedec`, `test_decoder` and the object files; none are
tracked.
