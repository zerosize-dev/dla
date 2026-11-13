# jvm-pool-warmup

Category `non-leak`. Cause `N-WARM`: a buffer pool grows to its configured size
while load builds up, then stays flat.

## Mechanism

`BufferPool` owns at most `SIZE = 200` buffers of 16 KB. `acquire` hands out an
idle one, creates a pooled one while it owns fewer than 200, and otherwise
returns a plain array that the pool does not take back (`BufferPool.acquire`;
the pooled allocation is line 39). Every completed job gives its leases back.
The driver lets the number of jobs in flight rise from 4 to 48 over the first
4400 jobs, like a morning rush; the pool reaches 200 buffers at about job 3800 and
the heap with it. After that the heap in use is the pool (3.2 MB) plus the
buffers of the jobs in flight, and the arrays above the pool are garbage after
each job. The `unpooled` counter in the driver output keeps rising, but it only
counts those short-lived arrays. A run that reports a confirmed growth here is a
false alarm.

## Measurements

Toolchain: OpenJDK 21.0.12.

```
cd project && ./build.sh run 8000        # compiles, runs the unit tests, runs 8000 jobs
```

Single run of 8000 jobs (checkpoints at 2000, 4000, 6000, 8000): heap in use
3507, 5060, 5430, 5446 KB; the pool owns 126, 200, 200, 200 buffers. Separate
runs, last checkpoint:

| jobs | pool owns | heap in use after GC |
|---|---|---|
| 500   | 45 (still filling)  | 2183 KB |
| 1000  | 74 (still filling)  | 2649 KB |
| 2000  | 126 (still filling) | 3485 KB |
| 4000  | 200                 | 5060 KB |
| 8000  | 200                 | 5446 KB |
| 16000 | 200                 | 5431 KB |
| 32000 | 200                 | 5414 KB |

The heap rises by about 3.3 MB while the pool fills (the first 4000 jobs), then
the slope is zero: 8000 and 32000 jobs differ by 32 KB. Unit tests: 6 in
`test/exports/ExporterTest.java`, all pass.
