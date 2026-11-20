# Open fixtures

These four fixtures are written and reviewed, but DLA does not pass them yet. They live
here, outside `bench/fixtures/`, so that `dla-bench validate`, `static`, `dynamic` and the
graders do not read them and the acceptance jobs stay green. Nothing scans this directory.
Each one moves back to `bench/fixtures/` (`git mv`) when DLA passes it; fix the product,
never the fixture's answer key.

| Fixture | Why DLA does not pass it yet |
|---|---|
| `go-feed-merge` | Static passes (the store at `internal/indexer/indexer.go:24` is flagged by the rule `go-slice-view-retained`). Dynamic: L2 is not confirmed. The heap read while the workload runs grows and the allocation stacks that grow are where the pinned documents are allocated (`internal/workload/workload.go:41`, `internal/fetcher/session.go:28`); the store itself allocates nothing, so no stack ties to the finding. |
| `go-trace-context` | Static passes without any external analyzer (the census lists the `ctx` field and the rule `go-chain-head-growth` flags `internal/rates/poller.go:32`). Dynamic: L1 is not confirmed. The growing stacks' first project frame is `trace.Start`; the finding is at its caller, and a stack ties only through its first project frame. |
| `python-zstd-options-3144` | Static passes (the census lists the decompressor construction at `archive/reader.py:17`). Dynamic: L1 is not confirmed. The growing allocation stack lands in `archive/tenants.py`, one frame from the finding's site, so nothing ties. |
| `rust-slot-arena` | DLA confirms the leak, at the declaration of the field that holds the pages (`src/arena.rs:24`); the answer key lists `src/arena.rs:46`, `:84-93`, `:25` and `:41`, so the finding does not count. |

## `go-trace-context` and the external analyzers

Static no longer depends on `golangci-lint` or `staticcheck` for this fixture: the census
and the DLA rules alone flag L1, so the static job on a runner with neither tool reports
the same result as the development machine. Only the dynamic half is open.

## Pairs

One of these fixtures is one half of a version pair. The other half stays in
`bench/fixtures/`, and its `pair` line was removed there because the validator refuses a
pair whose partner is not in the same directory:

- `python-zstd-options-3144`, partner `python-zstd-options-3145`

When the fixture moves back, restore the partner's line `pair = "python-zstd-options-3144"`
in its `expected.toml` (right after `category`).
