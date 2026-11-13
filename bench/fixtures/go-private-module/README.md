# go-private-module

Category `impossible`. `dynamic_confirmation = false`, the leak is `confirmable = false`:
the project cannot be built without a private module, so a run can only suspect the
cause from the source.

## Mechanism (not visible at runtime here)

`Dispatcher.Submit` posts each job through the ledger client and starts a watcher
goroutine for the job's receipt (`dispatcher.go:46`). The watcher waits in a `select` on
the receipt's `Done()` channel and on the dispatcher's context (`dispatcher.go:51-57`);
the context is `context.Background()` from `New` (`dispatcher.go:21`) and nothing ever
cancels it. A watcher therefore ends only when the ledger answers its receipt. Which
receipts the ledger answers, and whether a rejected or expired posting is answered at
all, is decided by `ledgerkit` (`Post`, `Receipt.Done`), which is why the behaviour cannot
be reproduced with a stand-in: whether goroutines pile up depends on the library's
contract. Every receipt that is never answered keeps one goroutine, its stack and the
receipt alive.

Cause `L-WORKER`, holder `blocked-stack`, region `non-heap`. Expected site
`internal/dispatch/dispatcher.go:46` (the `go` statement); alternatives: the `select`
(`dispatcher.go:51-57`) and the context field (`dispatcher.go:21`).

## Failing build

`go.mod` requires `example.invalid/ledgerkit v1.4.2` and replaces it with
`../vendor/ledgerkit`, a vendor checkout that exists on the maintainers' machines and
not next to this project. The module is also not on any proxy a sandbox can reach.

```
$ cd project && go build ./...
internal/dispatch/dispatcher.go:7:2: example.invalid/ledgerkit@v1.4.2: replacement directory ../vendor/ledgerkit does not exist
$ go test ./...
# example.com/dispatchd/cmd/dispatchd
internal/dispatch/dispatcher.go:7:2: example.invalid/ledgerkit@v1.4.2: replacement directory ../vendor/ledgerkit does not exist
FAIL	example.com/dispatchd/cmd/dispatchd [setup failed]
# example.com/dispatchd/internal/dispatch
internal/dispatch/dispatcher.go:7:2: example.invalid/ledgerkit@v1.4.2: replacement directory ../vendor/ledgerkit does not exist
FAIL	example.com/dispatchd/internal/dispatch [setup failed]
FAIL
```

(go1.24.7, exit status 1 for both commands; no files are written. Every package of the
project imports the module through `dispatcher.go`, so even the unit tests cannot run.)

## Source sanity check (not part of the fixture)

To make sure the Go is well-formed, the project was built and its tests run once against
a throw-away stand-in of `ledgerkit` kept outside the repository (one file, about 25 lines:
`Open`, `Client.Post`, `Receipt.Done`, `Entry`, `Result`) placed at `../vendor/ledgerkit`.
`go vet ./...` and `go test ./...` passed. The stand-in answered 7 receipts in 10 and
left the rest open; with it, the goroutine count after replaying 1000, 2000 and 4000
jobs (cumulative) was 249, 755 and 1771, which shows the watcher mechanism but proves
nothing about the real library.
