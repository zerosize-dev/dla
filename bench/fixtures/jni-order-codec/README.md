# jni-order-codec

Category `multi-leak`, runtimes `jvm` (primary) and `rust`, entry path
`repository`. Two independent leaks in two runtimes that share one workload: a
Kotlin service calls a Rust `cdylib` through raw JNI function-table slots (no
`jni` crate, no dependencies).

## Mechanism

`OrderEncoder.encode` packs an order into a frame, hands it to the library
(`Native.open` stores a copy in a global table and returns a handle), reads the
checksum and size, and calls `Native.close(handle)` to remove the frame. One
order in eight is a wholesale order whose frame is 24,592 bytes, above the
16,384-byte inline limit; for those, `encode` returns early before `close`.

- **L1, native (Rust side), share 0.68.** The frame inserted in
  `native/src/lib.rs:43` is never removed for wholesale orders: the early
  return in `kotlin/shop/Encoder.kt:12-15` skips `Native.close`. Cause
  `L-HANDLE`, holder `native-handle/global-ref`, region `off-heap`. Per order
  the native table grows by 24,592 / 8 = about 3.0 KB. The Java heap does not
  see it; RSS does.
- **L2, Kotlin heap, share 0.32.** `Audit.trail` (`kotlin/shop/Audit.kt`, a
  list in a Kotlin `object`) receives a 1,536-byte sample per audited
  (non-wholesale) order at line 12 and nothing removes entries. Cause
  `L-COLLECT`, holder `static/global`, region `heap`. About 1.4 KB of heap per
  order including the entry, its id string and the sample.

The call chain does not join the two: the Kotlin handle is a plain `Long`, so
no heap object retains the native frames.

Expected sites: L1 `native/src/lib.rs:43` (alternatives: the early return,
`kotlin/shop/Encoder.kt:12-15`, and the `FRAMES` map that holds the frames,
`native/src/lib.rs:19`); L2 `kotlin/shop/Audit.kt:12` (alternative: the
declaration at line 9).

## Build and run

Toolchain: OpenJDK 21, Rust stable, Kotlin 2.0.21 through `kotlinc` when it is
on `PATH` and otherwise the embeddable compiler jars from `~/.m2` (offline).

```
cd project && ./build.sh            # builds libcodec, compiles Kotlin, runs both test suites
./build.sh run 32000                # also replays 32000 orders (java -Xmx256m)
```

A full build is about 5 seconds; a 32,000-order run takes about 5 seconds in
total with the build.

## Measurements

The driver forces a GC at each quarter of the run and prints the heap in use,
the live frames in the native table (`nativeLive`, `nativeKB` = capacity sum),
the audit list size and VmRSS. Two complete runs gave identical native and
audit numbers and heap values within 30 KB.

`./build.sh run 8000`:

| order | heap after GC | nativeLive | nativeKB | audit | RSS |
|---|---|---|---|---|---|
| 0 | 1834 KB | 1 | 24 | 0 | 52 MB |
| 2000 | 4393 KB | 251 | 6027 | 1750 | 81 MB |
| 4000 | 7212 KB | 501 | 12031 | 3500 | 92 MB |
| 6000 | 10026 KB | 751 | 18035 | 5250 | 114 MB |
| done | | 1000 | | 7000 | |

`./build.sh run 32000`:

| order | heap after GC | nativeLive | nativeKB | audit | RSS |
|---|---|---|---|---|---|
| 0 | 1834 KB | 1 | 24 | 0 | 52 MB |
| 8000 | 12881 KB | 1001 | 24039 | 7000 | 175 MB |
| 16000 | 24225 KB | 2001 | 48055 | 14000 | 240 MB |
| 24000 | 35508 KB | 3001 | 72070 | 21000 | 216 MB |
| done | | 4000 | | 28000 | |

Both counters are linear in the order count: `nativeLive` equals the number of
wholesale orders (one eighth), `audit` equals the number of other orders.
RSS is noisy because the JVM returns and re-touches heap pages; use the two
counters, not RSS, for the shares.

### How the shares were measured

Shares are the fractions of the growth in live bytes held by each side, taken
between the checkpoints at orders 8000 and 24000 of the 32,000-order run:

- native: 72,070 KB - 24,039 KB = 48,031 KB (`Native.liveBytes()`, the capacity
  of every frame in the table)
- heap: 35,508 KB - 12,881 KB = 22,627 KB (used heap after `System.gc()`)

Native 48,031 / 70,658 = 0.680, heap 22,627 / 70,658 = 0.320. The numbers
follow from the frame sizes: 24,592 B x 1/8 = 3,074 B of native per order
against about 1,415 B of heap per order. The same run repeated gave 22,622 KB
and 48,031 KB.

## `dla probe`

On a git copy of `project/` (`dla run init`, then `dla probe`) the probe
reports `runtimes: ["jvm","rust"]` and `test_runners: ["cargo test"]`.
