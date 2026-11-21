# Go

Worker name and runtime id: `go`. Facts from `harness/go/README.md` and `docs/VERIFY.md`.

Archetype rows (what to look for, how to confirm, how to rule out): read `references/go-archetypes.md` only after `dla gate check --stage classify` passes.

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
- Command mode also measures while the workload runs. The record above is written after `main`
  returned, when state owned by locals of the workload (a request loop filling an index, a slice
  built in `run()`) is already dead: that series reads flat and the heap profile is empty although
  the heap grew by megabytes. A sampler goroutine forces one collection per sample (the first at 2 ms,
  each next one 15 % later than the last ended, never more than a quarter of the elapsed time spent
  in samples) and the exit record adds `go.heap_live.in_run` and `go.heap_objects.in_run`: the lowest
  after-collection reading among the samples of the last third of the run (a floor, not a maximum: the
  maximum of noisy readings rises with their number). Read it as a lower bound of the live heap while the
  workload ran; on the leaks measured it was 65 to 87 % of the true growth. The base `go.*` columns keep
  their meaning (read after `main`, after two collections), and driver mode has no in-run column.
  `dla alloc-test --finding` on a heap finding takes `go.heap_live.in_run` as its heap column only when
  the latest trend passed it through the gate and `go.heap_live` did not pass or rose by less than a
  tenth of the in-run rise (an exit column that passes on a few hundred bytes of its own says nothing
  about state that is gone at exit); otherwise `go.heap_live` decides. The in-run
  columns have no purge reading and no pre-GC twin, and `dla gap --residual` still reads `go.heap_live`.
  With `--capture alloc` the heap profile of a point is the one of the last sample (role "taken in the
  run at <ms> ms"), in place of the exit profile; goroutine captures stay at exit. Limitation lines say
  when it did not work: no sample ended before exit (a workload shorter than 2 ms, or `DLA_INRUN=off`), the
  reading is from the first half of the run (the time budget allowed few collections of a heap this
  size), or no profile was taken in the run (the exit profile is then written, without state owned by
  locals). The cost measured was +2 to +3 % of the run time on runs of 8 to 100 seconds and up to +16 %
  on a one-second run of a pointer-heavy heap. A workload that is never idle at the sampler's instants
  and frees nothing between batches reads its working set as the floor: flat in `n`, so no trend.
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

## Static rules
`dla sweep --lang go` runs five DLA rules of its own beside the Go linters; the candidate of each rule
is an open question for the dynamic stages, not a verdict.
- `dla/go-goroutine-no-stop-path`, `dla/go-time-tick`, `dla/go-ticker-no-stop`: see `references/go-archetypes.md`, row L-WORKER.
- `dla/go-slice-view-retained` (no archetype, prior 0.3): a slice parameter is stored without a copy into
  state that accumulates (`x.f[k] = p`, `x.f = append(x.f, p)`, or an insert call on a field whose argument is
  `p` or a literal holding `p`), and a call site in the project passes a view of a larger buffer (a
  sub-slice, or the result of `bytes.Cut`, `bytes.SplitN`, `bytes.Fields` and the like, also through a
  struct field set from one). The hit is at the store line and its message names the parameter, the
  container, the feeding call and the origin of the view. Each stored entry then keeps its whole source
  buffer alive, which a heap profile blames on the allocation site, not on the store. The rule is quiet for
  `bytes.Clone`, `slices.Clone`, `append([]byte(nil), p...)`, `string(p)`, a copy into a fresh slice, a
  `string` parameter and a pool `Put` fed only by buffers from `Get` or `make`. The candidate carries its
  holder (`container-field`, or `static/global` for a package variable), its owner and region `heap`.
  A call feeds a method only when its receiver can be the method's type, read from the source (a parameter,
  `var x T`, `x := T{...}`, `new(T)`, a constructor, a field of a known struct); a call on another type with
  a method of the same name does not count. Where the type cannot be read (an interface value, a call
  result) the name decides and the hit's message says the receiver's type was not resolved: check that call
  before you trust the hit. A plain function is fed by a call in its package or through its package qualifier.
- `dla/go-chain-head-growth` (no archetype, prior 0.3): a field of a receiver (or a package variable) is
  reassigned from a call that takes the field's own value, `p.ctx = trace.Start(p.ctx, ...)`, the field's type
  can hold what it replaces (an interface such as `context.Context`, a func, or a pointer to a
  type that points to itself; an `error` wrapped around itself is left out, it is a one-off on a failure
  path), and nothing in the project resets it (`= nil`, `context.Background()`) or
  steps it back (`p.ctx = p.ctx.Parent()`). The hit is at the assignment. `dla census --lang go` lists
  such a field as `container-field` (region `heap`) with the assignment as its writer, so a heap
  allocation stack that ends in the wrapping call ties to the finding.

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
- Heap profiles blame the allocation site, not the holder (a subslice pinning a large buffer): the
  stacks that tie to the store of `dla/go-slice-view-retained` carry only the small stored objects, and the
  buffers they pin are allocated elsewhere.
- Maps and pools keep peak capacity: a flat entry count with high memory is a high-water mark,
  not necessarily a leak.
- Memory held by cgo or by the runtime outside the Go heap is not in a heap profile.
- No linter covers the commonest goroutine leaks; only running the code finds them.
