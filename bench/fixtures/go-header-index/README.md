# go-header-index

Category `archetype-free`, cause `UNCLASSIFIED`, holder `container-field`, region
`heap`. No partner.

## Mechanism

`edge.ReadFrame` fills a fresh 16 KB buffer for every client frame. `ParseRequest`
returns the request line and header values as sub-slices of that buffer, and
`Router.Handle` hands the `X-Request-Id` value (about 20 bytes) to `Index.Note`
(`router.go:48`), which stores the slice itself in a list entry
(`index.go:36`) so the diagnostics endpoint can return it without copying. The
index drops its oldest entry once it holds 100000 tokens, so its entry count is
bounded, but a retained 20-byte token keeps the whole 16 KB array it was cut from:
each entry pins 16 KB, about 800 times its own size, and no collection, goroutine,
timer, listener or captured scope holds more than the index's list and map. The mechanism
fits no taxonomy row: the container is capped and the growth comes from what its
elements point into. Expected site `internal/edge/index.go:36`; alternatives: the
sub-slice in `request.go:45` and the call in `router.go:48`. The repair is to copy
the token (`bytes.Clone`) before storing it.

The cap is 100000 entries, so the series is linear in the number of requests for any
replay a person would run here (the cap is reached at about 1.6 GB of live heap, which is
where the "restarted every few hours" in the report comes from). The workload
default is 6000 requests.

## Measurements

Toolchain: go1.24.7 linux/amd64, standard library only. Checks and workload:

```
cd project && go vet ./... && go test ./... && go run ./cmd/edged -n 6000
# requests=6000 routed=4058 denied=1942 checksum=99606b16   (0.8 s wall)
```

External harness (kept outside the repository): copies `project/`, adds a
`cmd/measure` main that replays the same connection through the router and, at the listed
counts, runs `runtime.GC()` and prints the index size, `runtime.NumGoroutine()` and
`runtime.ReadMemStats` (`HeapAlloc`, `StackInuse`).

| requests | index entries | goroutines | heap alloc after GC | stack in use |
|---|---|---|---|---|
| 1     | 1     | 1 | 3.4 MB   | 320 KB |
| 5000  | 5000  | 1 | 82.0 MB  | 320 KB |
| 10000 | 10000 | 1 | 160.7 MB | 320 KB |
| 15000 | 15000 | 1 | 239.3 MB | 384 KB |
| 20000 | 20000 | 1 | 317.9 MB | 384 KB |
| 25000 | 25000 | 1 | 396.5 MB | 384 KB |
| 30000 | 30000 | 1 | 475.1 MB | 384 KB |

Slope between 5000 and 30000 requests: 16.1 KB of live heap per request (the frame size
plus 100 bytes of list and map entry), constant at every checkpoint; there is no plateau
within the cap. Max RSS of `edged -n N`: 63 MB at 3000, 116 MB at 6000, 236 MB at 12000 and 566 MB at
30000 (about 19 KB per request: the frame plus runtime headroom). With the one-line
edit of the repair (`bytes.Clone` in `Note`) the heap after GC is 4.6 MB at 10000
requests and 7.1 MB at 30000 (0.125 KB per request: the index entries only). The workload
finishes in 0.8 s at 6000 requests and 3.7 s at 30000.
