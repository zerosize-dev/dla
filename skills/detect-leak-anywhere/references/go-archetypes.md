# Go archetype rows

A row match opens a hypothesis and carries no verdict.

Read this only after `dla gate check --stage classify` passes. A row says where to look and what a
confirmed case looks like; the finding is confirmed by the ledger rules, then labelled with
`dla ledger label F --as <ID>`. A finding that fits no row stays UNCLASSIFIED. Runtime facts:
`references/go.md`; commands: `references/backends.md`.

| Row | Find candidates | Confirm | Fingerprint | Rule out |
|---|---|---|---|---|
| L-WORKER | `dla sweep --lang go` rules `dla/go-goroutine-no-stop-path`, `dla/go-ticker-no-stop`, `dla/go-time-tick`, `staticcheck/SA1015` [G2]; `dla census --lang go` | `dla harness gen --lang go --driver`, `dla harness run H1 --capture alloc,nonheap`, `dla trend S1` on `go.goroutines` and `go.goroutines@file:line`; `dla summarize non-heap` on `go-*.goroutine.txt`, then on `go-*.goroutineleak.txt` (a second signal that never confirms alone) | Many goroutines created at one `go` statement, parked in the same wait, and the count rises with the units; the `created by` group ties to the finding at that statement. A timer or ticker alone leaks only before Go 1.23 or with `asynctimerchan=1` [G1]; `dla probe` writes that verdict | `N-WARM`: goroutines end once their work is closed and the count levels off; `dla alloc-test H1 --finding F` (plateau test on `go.goroutines`). `N-SCALE`: the count follows live load, quiet under repeated keys; `dla trend S1 --against S2 --finding F` |
| UNCLASSIFIED | Any growth with a holder and no row above | State the holder and the mechanism, then `dla ledger label F --as UNCLASSIFIED --note "<why nothing fits>"` | The holder is named by `dla census --lang go` or the goroutine groups, and the mechanism is not the row above | The same tests as every row: `N-WARM` and `N-HWM` by `dla alloc-test H1 --finding F`, `N-ALLOC` by its `free-os-memory` purge |

## Sources

- [G1] https://go.dev/doc/go1.23
- [G2] https://staticcheck.dev/docs/checks/ (SA1015)
