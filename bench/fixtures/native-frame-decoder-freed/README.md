# native-frame-decoder-freed

Category `decoy-control` (control half of a pair, partner `native-frame-decoder`).
Expected answer: no confirmed leak; the `malloc` per frame in `decode_frame` is
ruled out (`N1`, `N-WARM`: the allocator's working set is reached within the first
frames and stays flat).

## Mechanism

Identical to the partner except that the checksum-mismatch path frees the output
buffer before it returns. Every `malloc` in `decode_frame` is now matched by a
`free` on every path: the payload error path, the checksum error path and the
success path (`frame_release`). About 6% of the frames are still rejected, so the
error path is exercised as often as in the trap, and an allocation per rejected
frame looks as suspicious as in the trap.

Background: `free` returns a block to the allocator, which keeps it for reuse and
may keep the heap pages mapped; resident size therefore rises to the working set
of the largest live buffer and stays there. `malloc_trim` is the call that asks
the allocator to give free heap memory back to the system
(https://man7.org/linux/man-pages/man3/malloc_trim.3.html: "attempts to release
free memory from the heap").

## Distinguishing edit (trap vs control)

`diff -u` of `src/decoder.c`, run from `bench/fixtures`: 4 added lines, 1 changed.

```
@@ -36,8 +36,10 @@
     uint32_t stored = load_u32(wire + FRAME_HEADER_SIZE + enc_len);
-    if (crc32_of(buf, raw_len) != stored)
+    if (crc32_of(buf, raw_len) != stored) {
+        free(buf);
         return FRAME_ERR_CHECKSUM;
+    }
```

## Measurements

```
$ cd project && make && make test
all tests passed
```

Peak resident size (`VmHWM` sampled from `/proc/<pid>/status` by a wrapper outside
the repository), trap and control with the same default workload
(`--damage-every 16`):

| `./framedec --iterations N` | rejected frames | control peak RSS | trap peak RSS |
|---|---|---|---|
| 50     | 3    | 1528 kB | |
| 250    | 15   | 1540 kB | 1748 kB |
| 1000   | 62   | 1540 kB | 2400 kB |
| 4000   | 250  | 1540 kB | 5716 kB |
| 16000  | 1000 | 1536 kB | 18564 kB |
| 32000  | 2000 | 1540 kB | 35560 kB |
| 128000 | 8000 | 1540 kB | |

The control is flat from 50 iterations on (slope of the second half is zero); the
trap grows about 1.06 kB per iteration. The 32000-iteration run takes 3.5 s.

Valgrind, `valgrind --leak-check=full ./framedec --iterations 2000`:

```
total heap usage: 2,002 allocs, 2,002 frees, 36,012,096 bytes allocated
All heap blocks were freed -- no leaks are possible
```

The trap reports 124 definitely-lost blocks for the same command.
