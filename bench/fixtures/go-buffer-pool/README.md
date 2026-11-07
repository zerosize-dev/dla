# go-buffer-pool

Category `non-leak`, cause `N-HWM` (high-water mark), no partner.

## Mechanism

`render.BufferPool` hands out byte buffers and takes them back with
`p.free = append(p.free, buf)` (`pool.go:37`) with no cap on the idle list, and
`Renderer.RenderBatch` indexes each rendered document in a map keyed by invoice ID
(`batch.go:40`) and empties the map with `delete` after the batch. Both structures
grow with the number of invoices in flight, not with the number of batches: they
rise to the size of the largest batch the workload ever sends (400 invoices) within
the first ten batches, keep that size (an idle buffer keeps its capacity, and a Go
map keeps its table after `delete`), and stay flat afterwards. A reader who only sees
an append that is never trimmed will suspect it; the measurements show a plateau.

Expected: no confirmed leak; both growth sources ruled out with the plateau as the
discriminator.

## Measurements

Toolchain: go1.24.7. Checks and workload:

```
cd project && go vet ./... && go test ./... && go run ./cmd/invoiced -n 2000
# batches=2000 invoices=395669 ...  (3.0 s wall; -n 8000 takes 12.7 s)
```

External harness (kept outside the repository): copies `project/`, adds a
`cmd/measure` main that renders batches through `workload.NewRenderer()` and, at the
listed batches, runs `runtime.GC()` and prints `pool.Stats()`, `runtime.NumGoroutine()`
and `runtime.ReadMemStats`. One run of 8000 batches:

| batch | buffers created | idle in pool | goroutines | heap alloc after GC |
|---|---|---|---|---|
| 1    | 79  | 79  | 1 | 226 KB  |
| 3    | 164 | 164 | 1 | 465 KB  |
| 10   | 400 | 400 | 1 | 1121 KB |
| 30   | 400 | 400 | 1 | 1284 KB |
| 100  | 400 | 400 | 1 | 1334 KB |
| 300  | 400 | 400 | 1 | 1339 KB |
| 2000 | 400 | 400 | 1 | 1381 KB |
| 4000 | 400 | 400 | 1 | 1387 KB |
| 8000 | 400 | 400 | 1 | 1389 KB |

The 2000-batch run ends at 400 buffers and 1381 KB, the same as the 8000-batch run
at batch 2000. From batch 300 to batch 8000 the live heap moves by 50 KB over 7700
batches (0.0065 KB per batch, a plateau; the first 10 batches add 900 KB). Max RSS
of `invoiced -n N`: 13.0 MB at 500 batches, 13.1 MB at 2000, 13.1 MB at 8000.
