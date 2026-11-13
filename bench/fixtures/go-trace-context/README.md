# go-trace-context

Category `archetype-free`, cause `UNCLASSIFIED`, holder `container-field`, region
`heap`. No partner.

## Mechanism

`Poller.Lookup` starts its span with `trace.Start(p.ctx, "lookup", ...)` and stores
the result back in the poller's `ctx` field (`poller.go:32`). `trace.Start` wraps the
context it is given with `context.WithValue` (`trace.go:35`) and sets the new span's
`Parent` to the span already in that context. Because every lookup derives its context
from the previous lookup's, the contexts form one chain that begins at
`context.Background()` and gains one value context, one span with its attributes and
the span's attribute slice and strings per lookup. The poller is long-lived and its field points
at the newest link, so the whole chain stays reachable through the contexts' own parent
pointers and the spans' `Parent` fields. No collection, goroutine, timer, listener or
captured scope grows; the quote cache holds at most eight pairs. The mechanism fits no
taxonomy row (a chain of contexts, not a registry or cache). Expected site
`internal/rates/poller.go:32`; alternatives: the `ctx` field (`poller.go:20`) and the
`WithValue` call (`trace.go:35`). The repair is to start each lookup from a context that
does not carry the previous one (`context.Background()` or a per-request context).

## Measurements

Toolchain: go1.24.7 linux/amd64, standard library only. Checks and workload:

```
cd project && go vet ./... && go test ./... && go run ./cmd/ratesd -n 100000
# lookups=100000 fresh=13805 cached=86195 checksum=5b7c2152   (0.10 s wall)
```

External harness (kept outside the repository): copies `project/`, adds a
`cmd/measure` main that replays the same lookups through `workload.NewPoller` and, at
the listed counts, runs `runtime.GC()` and prints `runtime.NumGoroutine()` and
`runtime.ReadMemStats` (`HeapAlloc`, `HeapObjects`, `StackInuse`).

| lookups | goroutines | heap alloc after GC | heap objects | stack in use |
|---|---|---|---|---|
| 1      | 1 | 86 KB   | 276     | 352 KB |
| 100000 | 1 | 30.6 MB | 599768  | 352 KB |
| 200000 | 1 | 61.1 MB | 1199766 | 352 KB |
| 300000 | 1 | 91.6 MB | 1799766 | 320 KB |
| 400000 | 1 | 122.2 MB | 2399772 | 384 KB |

Slope between 100000 and 400000 lookups: 0.320 KB of live heap and 6.0 objects per
lookup, the same in each interval; no plateau. Max RSS of `ratesd -n N`: 12.8 MB at
20000, 36.2 MB at 100000, 71.3 MB at 200000. With the edit of the repair (the lookup
starts from `context.Background()`) the heap after GC is 92 KB at 200000 and 93 KB at 400000
lookups, 291 and 299 objects. The workload takes 0.10 s at 100000 lookups and 0.21 s at 200000.
