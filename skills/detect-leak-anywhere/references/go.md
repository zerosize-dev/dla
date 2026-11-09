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
- Raw captures: `dla harness run H1 --capture alloc,nonheap`. `alloc` writes `go-<n>.heap.pb.gz`
  (`DLA_MEMPROFILE_RATE`, default 4096); `nonheap` writes `go-<n>.goroutine.pb.gz`, `go-<n>.goroutine.txt`
  (debug=2, with the `created by` frame) and `go-<n>.goroutineleak.txt`. Driver mode captures at the
  checkpoints, so 3 or more files come from one process.
- The goroutine leak profile is generally available from Go 1.27 (Go 1.26: `GOEXPERIMENT=goroutineleakprofile`).
  `dla probe` records the toolchain decision (probe field `go`, coverage row `go/goroutineleak`). With an
  older project toolchain and Go 1.27 in the module cache, `harness run` makes one extra pass with
  `GOTOOLCHAIN=go1.27.x` just for the leak profile; with none cached the row is `missing` and the hint is
  `dla install go-toolchain-1.27` (tier 1; it downloads the toolchain into the Go module cache, outside the
  project). Record the candidate `unresolved` (`tool_missing`) if you cannot install it.

## Attach backends
| Backend | Tier | Notes |
|---|---|---|
| `dla sample --pid <P> --counters go-pprof=<url>` | 2 | HTTP GETs to the target's pprof endpoint |
| `dla attach goref --pid <P> --op heap-profile` | 3 | suspends the target for the whole run; written from the goref README and tested with a fake `grf` only: marked unverified in the output |

There is no jcmd-style attach for Go. Prefer the harness.

## Artifacts
- Harness captures: read the heap profiles with `dla summarize alloc-stack-diff` (`go-pprof-heap`, ties
  by allocation site), the goroutine dumps with `non-heap` (`go-goroutine`: the group at the `created by`
  site ties to the finding at that `go` statement) and the leak files with `non-heap` (`go-goroutineleak`).
  The leak evidence is a second signal: it never confirms alone and never refutes.
- goref heap profiles land under `attach/P<n>/`; no `dla summarize` adapter reads them. Record them as
  artifacts.
- cgo: `import "C"` with project-defined C functions is detected as a boundary (probe summary `boundaries`),
  but the allocator shim measures JNI only, so `dla correlate` has nothing to compare for Go.

## Blind spots
- The leaked-goroutine profile misses primitives still reachable from globals and depends on the
  Go version; an empty profile proves nothing. The debug=2 text lists every goroutine (the profiler's own
  included), so count the `(leaked)` marks.
- debug=2 goroutine text carries no pprof label values.
- Known leak: timers before Go 1.23 (see `references/backends.md`, known leaks): `dla probe` reports it when
  the `go` line is older and the source makes timers or tickers.
- Heap profiles blame the allocation site, not the holder (a subslice pinning a large buffer).
- Maps and pools keep peak capacity: a flat entry count with high memory is a high-water mark,
  not necessarily a leak.
- Memory held by cgo or by the runtime outside the Go heap is not in a heap profile.
- No linter covers the commonest goroutine leaks; only running the code finds them.
