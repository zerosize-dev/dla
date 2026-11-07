# go-subscription-workers

Category `archetype-leak` (trap half of a pair, partner `go-subscription-workers-closed`).
Cause `L-WORKER`, holder `blocked-stack`, region `non-heap`.

## Mechanism

`Hub.Open` creates a `Subscription`, and `newSubscription` starts one worker
goroutine for it (`subscription.go:33`). The worker sits in a `select` on its
event, flush and `done` channels (`subscription.go:39-47`) and only returns when
`done` is closed, which only `Subscription.Close` does. `Handler.Serve` opens a
subscription for every request, pushes the items, flushes, and returns without
calling `Close`. Nothing else holds the subscription (the hub keeps no table of
them), so each worker is referenced only by its own blocked stack: one goroutine,
its stack, the 4 KB render scratch buffer and the channels stay alive per request.
`Close` exists and is exercised by the unit tests, but no production caller uses it.

Expected site `internal/notify/subscription.go:33`; alternatives: the `select`
(`subscription.go:39-47`) and the `Open` call in `handler.go:18`.

## Distinguishing edit (trap vs control)

The control adds `defer sub.Close()` after the error check in `Handler.Serve`: one
added line, 11 lines of `diff -ru` output (pasted in the partner's README).

## Measurements

Toolchain: go1.24.7 linux/amd64, standard library only. Checks:

```
cd project && go vet ./... && go test ./... && go run ./cmd/notifyd -n 8000
# requests=8000 delivered=23905 ... (0.15 s wall)
```

The measurement harness is kept outside the repository: it copies `project/`, adds
a `cmd/measure` main that replays requests through `workload.Build()` and, at four
checkpoints (and at start), runs `runtime.GC()`, then prints `runtime.NumGoroutine()`
and `runtime.ReadMemStats` (`HeapAlloc`, `StackInuse`, `Sys`).

| requests | goroutines | heap alloc after GC | stack in use | max RSS (`notifyd -n N`) |
|---|---|---|---|---|
| 0     | 1     | 84 KB      | 320 KB    | |
| 2000  | 2001  | 10713 KB   | 4512 KB   | 18.5 MB |
| 8000  | 8001  | 42592 KB   | 17024 KB  | 66.6 MB |
| 32000 | 32001 | 170132 KB  | 66688 KB  | 249.5 MB |

Slopes, taken between the 8000 and 32000 request checkpoints of the 32000-request
run: 1.000 goroutines per request, 5.31 KB of live heap per request, 2.08 KB of stack
per request, 7.6 KB of RSS per request. Growth is linear in the iteration count at
every checkpoint (the 2000 and 8000 request runs agree with it); there is no plateau.
The workload finishes in 0.5 s at 32000 requests.
