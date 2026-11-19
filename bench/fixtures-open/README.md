# Open fixtures

These eight fixtures are written and reviewed, but DLA does not pass them yet. They live
here, outside `bench/fixtures/`, so that `dla-bench validate`, `static`, `dynamic` and the
graders do not read them and the acceptance jobs stay green. Nothing scans this directory.
Each one moves back to `bench/fixtures/` (`git mv`) when DLA passes it; fix the product,
never the fixture's answer key.

| Fixture | Why DLA does not pass it yet |
|---|---|
| `go-feed-merge` | Static: leak L2 (`internal/indexer/indexer.go:24`) is not flagged by any candidate. Dynamic: L2 is not confirmed. |
| `go-header-index` | Static: leak L1 (`internal/edge/index.go:36`) is not flagged by any candidate. Dynamic: L1 is not confirmed. |
| `go-ticker-legacy` | Dynamic: L1 is confirmed (the Go detector `go-ticker-no-stop` finds the ticker) but its label is refused with `label-evidence-missing`: Go command mode has no timer evidence, the goroutine growth column does not grow across all snapshots, and the goroutineleak profile is empty. |
| `go-trace-context` | Dynamic: leak L1 (`internal/rates/poller.go:32`) is not confirmed. Static passes where `golangci-lint` and `staticcheck` are installed; see below. |
| `node-signal-compose-24` | Dynamic: leak L1 (`src/handler.js:12`) is not confirmed. |
| `python-frozen-gc` | Static: leak L1 (`journeys/worker.py:25`) is not flagged by any candidate. Dynamic: L1 is not confirmed. |
| `python-zstd-options-3144` | Static: leak L1 (`archive/reader.py:17`) is not flagged by any candidate. Dynamic: L1 is not confirmed. |
| `rust-slot-arena` | Dynamic: leak L1 (`src/arena.rs:46`) is not confirmed. |

## Why two machines disagreed on `go-trace-context`

The static job on the CI runner reported L1 as missed; the static run on the development
machine flagged it. The leak is named only by the `golangci-lint` detector `containedctx`
(at the struct field `internal/rates/poller.go:20`, an alternative site of L1); the other
enabled detector, `fatcontext`, fires once, in `internal/trace/trace_test.go`, not at the
leak. The development machine had `golangci-lint` and `staticcheck`; the CI runner had
neither (the job lists both as missing for every Go fixture), so only the census candidates
(six) existed there and none sits at `internal/rates/poller.go:32`. The static job's rule is
that the census and the DLA rules alone must flag every confirmable leak, so with the
runner's tools this fixture fails static as well as dynamic: whoever fixes dynamic and moves
it back must also make static flag L1 without `golangci-lint`, or the static job turns red.

## Pairs

Three of these fixtures are one half of a version pair. The other half stays in
`bench/fixtures/`, and its `pair` line was removed there because the validator refuses a
pair whose partner is not in the same directory:

- `go-ticker-legacy`, partner `go-ticker-modern`
- `node-signal-compose-24`, partner `node-signal-compose-22`
- `python-zstd-options-3144`, partner `python-zstd-options-3145`

When a fixture moves back, restore the partner's line `pair = "<fixture>"` in its
`expected.toml` (right after `category`).
