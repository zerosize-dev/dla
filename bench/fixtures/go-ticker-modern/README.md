# go-ticker-modern

Category `decoy-control` (partner `go-ticker-legacy`), cause `N-WARM`.
`schema_version = 2`: the fixture names its toolchain, `go = "1.23.12"`.

## Mechanism

The code is the same as the partner's. `awaitConfirm` creates a `time.NewTicker` before its
first check (`wait.go:11`) and returns when the check succeeds, without stopping the ticker;
with the simulated warehouse it returns before the ticker has fired. What differs is the Go
version. `go.mod` names `go 1.23` and the toolchain is Go 1.23.12, where a Ticker that
nothing refers to is collected (`asynctimerchan=0`), so the tickers created per order do not
stay in the runtime's timer heap. Memory stays flat after a short warm-up; a reader who sees
a `NewTicker` without `Stop` in a per-request function will suspect a leak, and the
measurements show there is none on this toolchain.

Expected: no confirmed leak; the growth ruled out with the flat heap after a forced
collection as the discriminator, and the Go version as the reason.

## Distinguishing edit (trap vs control)

One line of `go.mod` (`go 1.23` here, `go 1.22` in `go-ticker-legacy`) and the `[toolchain]`
table of `expected.toml` (`1.23.12` here). The project is otherwise identical and the prompt
is the same.

## Measurements

Toolchain: go1.23.12 linux/amd64 (the fixture's own), standard library only. Checks and
workload:

```
cd project && go vet ./... && go test ./... && go run ./cmd/reservd -n 200000
# orders=200000 reserved=200000 short=0 checksum=b328b520   (0.3 s wall)
```

Unit tests: 6 (`go test ./...`, all pass).

External harness (kept outside the repository): the same `cmd/measure` as the partner's.

| orders | goroutines | heap alloc after GC | heap objects |
|---|---|---|---|
| 1      | 1 | 85 KB  | 289 |
| 100000 | 1 | 99 KB  | 359 |
| 200000 | 1 | 100 KB | 378 |
| 300000 | 1 | 100 KB | 398 |
| 400000 | 1 | 101 KB | 416 |

From 100000 to 400000 orders the live heap moves by 2 KB (0.007 bytes per order) and the
object count by 57, a plateau. Max RSS of `reservd -n N`: 13.2 MB at 50000, 13.1 MB at 200000
and 13.0 MB at 400000. The checksums of both halves agree at every `-n`, since the code is
the same.
