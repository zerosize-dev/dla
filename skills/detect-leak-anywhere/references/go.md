# Go

Worker name and runtime id: `go`. Facts from `harness/go/README.md` and `docs/VERIFY.md`.

## Harness
- Command mode: `zz_dla_harness_test.go` is written next to a `package main` and runs the
  program's `main` through `go test -run '^TestDLAHarness$'` with the workload's flags passed in
  `DLA_ARGS`. `os.Exit(0)` is recovered and counted as a normal end; a non-zero exit leaves only
  tick records. Pass a larger `-timeout` for long runs (the default is 10 minutes).
- Driver mode (`dla harness gen --lang go --driver [--package <dir>]`): a test inside the package
  under test, so hooks reach unexported identifiers and package variables. Probes of package
  variables work only here. Fill `dlaSetup`, `dlaUnit` and, optionally, `dlaProbes`. An optional
  goleak check runs only when `go.uber.org/goleak` resolves offline.
- What the dump measures (after two `runtime.GC()`): `go.heap_live`, `go.heap_objects`, `go.total`,
  `go.released`, `go.stacks`, `go.goroutines`, the `os.*` set, and the goroutine profile grouped by
  `created by` file:line (`go.goroutines@file:line`). Coverage comes from `go test -coverprofile`.
- Purges: `free-os-memory` and `gc`.

## Attach backends
| Backend | Tier | Notes |
|---|---|---|
| `dla sample --pid <P> --counters go-pprof=<url>` | 2 | HTTP GETs to the target's pprof endpoint |
| `dla attach goref --pid <P> --op heap-profile` | 3 | suspends the target for the whole run; written from the goref README and tested with a fake `grf` only: marked unverified in the output |

There is no jcmd-style attach for Go. Prefer the harness.

## Artifacts
- goref heap profiles and pprof files land under `attach/P<n>/`, but
  no `dla summarize` adapter reads them in this release (Go profiles and the `goroutineleak` profile are
  cut 2; see `references/backends.md`). Record them as artifacts and say the summarizer is not available yet.

## Blind spots
- The leaked-goroutine profile misses primitives still reachable from globals and depends on the
  Go version.
- Heap profiles blame the allocation site, not the holder (a subslice pinning a large buffer).
- Maps and pools keep peak capacity: a flat entry count with high memory is a high-water mark,
  not necessarily a leak.
- Memory held by cgo or by the runtime outside the Go heap is not in a heap profile.
- No linter covers the commonest goroutine leaks; only running the code finds them.
