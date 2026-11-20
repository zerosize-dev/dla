# go-ticker-legacy

Category `archetype-leak` (trap half of a version pair, partner `go-ticker-modern`).
Cause `L-REGISTER` through the Go ticker row of `data/known-leaks.toml`
(`go-timer-unstopped-before-1.23`), holder `registration-table`, region `heap`.
`schema_version = 2`: the fixture names its toolchain, `go = "1.22.12"`.

## Mechanism

`Service.Reserve` takes stock, places a warehouse hold and calls `awaitConfirm`
(`service.go:46`), which polls the hold every `interval` (30 s in the workload). The
function creates its `time.NewTicker` before the first check (`wait.go:11`) and returns as
soon as the check succeeds. The simulated warehouse confirms at once, so the function
returns on the first check, before the ticker has ever fired, and nothing stops the ticker.

Before Go 1.23, a Ticker that is not stopped is never collected, even when nothing refers to
it: it stays in the runtime's timer heap. Go 1.23 changed this for programs whose `go.mod`
names `go 1.23` or later (`asynctimerchan=0`): an unreferenced Ticker is collected. This
fixture's `go.mod` says `go 1.22` and its toolchain is Go 1.22.12, so each order adds one
Ticker (struct, channel and timer entry) that stays live: 0.24 KB and 3.3 heap objects per
order. Go 1.27 removed the `asynctimerchan` setting, so the old behaviour can only be had on
a toolchain older than 1.27; that is why the pair pins its own toolchains.

Expected site `internal/reserve/wait.go:11`; alternatives: the function body
(`wait.go:10-25`) and the call in `service.go:46`. The repair is `defer tick.Stop()` after
the `NewTicker` call, or raising the `go` line (and the toolchain) to 1.23.

## Distinguishing edit (trap vs control)

The project is identical in both halves, byte for byte, apart from one line of `go.mod`
(`go 1.22` here, `go 1.23` in the partner) and the `[toolchain]` table of `expected.toml`
(`1.22.12` here, `1.23.12` in the partner). The prompt is the same.

```
$ diff -r go-ticker-legacy/project go-ticker-modern/project
diff -r go-ticker-legacy/project/go.mod go-ticker-modern/project/go.mod
3c3
< go 1.22
---
> go 1.23
```

## Measurements

Toolchain: go1.22.12 linux/amd64 (the fixture's own), standard library only. Checks and
workload:

```
cd project && go vet ./... && go test ./... && go run ./cmd/reservd -n 200000
# orders=200000 reserved=200000 short=0 checksum=b328b520   (0.4 s wall)
```

Unit tests: 6 (`go test ./...`, all pass).

External harness (kept outside the repository): copies `project/`, adds a `cmd/measure` main
that replays the same orders and, at the listed counts, runs `runtime.GC()` and prints
`runtime.NumGoroutine()` and `runtime.ReadMemStats` (`HeapAlloc`, `HeapObjects`).

| orders | goroutines | heap alloc after GC | heap objects |
|---|---|---|---|
| 1      | 1 | 80 KB   | 267     |
| 100000 | 1 | 25.4 MB | 370789  |
| 200000 | 1 | 48.5 MB | 711761  |
| 300000 | 1 | 68.8 MB | 1018353 |
| 400000 | 1 | 93.0 MB | 1374190 |

Slope between 100000 and 400000 orders: 0.236 KB of live heap and 3.3 objects per order,
about the same in every interval; no plateau. Max RSS of `reservd -n N`: 23.6 MB at 50000,
67.4 MB at 200000, 139.3 MB at 400000. The partner's table, taken the same way on go1.23.12
with `go 1.23`, is flat at about 100 KB.
