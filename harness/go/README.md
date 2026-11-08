# Go agent and drivers

| File | Role |
|---|---|
| `zz_dla_harness_test.go.tmpl` | command mode: written next to a `package main` as `zz_dla_harness_test.go` |
| `zz_dla_driver_test.go.tmpl` | driver mode: written into the package under test as `zz_dla_driver_test.go` |
| `zz_dla_goleak_test.go.tmpl` | optional goleak check, build tag `dlagoleak` (driver mode only) |

Standard library only (the goleak file excepted). Every identifier and every import alias starts with
`dla`/`DLA`, so the files cannot clash with the package they are dropped into. Both main templates
carry the same dump engine between `DLA-ENGINE-BEGIN` and `DLA-ENGINE-END`; check that they agree with

```
diff <(sed -n '/DLA-ENGINE-BEGIN/,/DLA-ENGINE-END/p' zz_dla_harness_test.go.tmpl) \
     <(sed -n '/DLA-ENGINE-BEGIN/,/DLA-ENGINE-END/p' zz_dla_driver_test.go.tmpl)
```

## Command mode

```
cd <scratch project root>
DLA_ARGS='["-n","2000"]' DLA_HARNESS_OUT=... DLA_PROJECT=<scratch project root> \
go test -vet=off -count=1 -run '^TestDLAHarness$' \
  -coverprofile <harness dir>/cov/<n>.out -coverpkg=./... ./cmd/x
```

`TestDLAHarness` sets `os.Args` to the program name plus `DLA_ARGS` (a JSON array of strings; the
workload's own flags), calls `main()`, and writes one `exit` record from a deferred function, so the
record is written even when `main` panics. `main` calling `os.Exit(0)` is a panic under `go test`;
the harness recovers it, notes it in `limitations`, and counts it as a normal end. `os.Exit(n)` with
`n != 0` ends the process at once: only `tick` records exist then (`DLA_TICK_MS`, a limitation to
record). The default `go test` timeout is 10 minutes; pass `-timeout` for long runs. The test binary
prints the workload's stdout; keep it in the harness log.

## What the dump records

- `values` after `runtime.GC()` twice: from `runtime/metrics`, `go.heap_live` (`/gc/heap/live:bytes`),
  `go.heap_objects` (`/memory/classes/heap/objects:bytes`), `go.total` (`/memory/classes/total:bytes`),
  `go.released` (`/memory/classes/heap/released:bytes`), `go.stacks`
  (`/memory/classes/heap/stacks:bytes`), `go.goroutines` (`/sched/goroutines:goroutines`; includes the
  test runner and the workload's main goroutine); and the `os.*` figures from `/proc/self/status`.
  `values_pre_gc`: `go.heap_objects`, `go.total`, `go.goroutines`, `os.rss` before the collections.
- `groups["go.goroutines"]`: the goroutine profile (`pprof.Lookup("goroutine").WriteTo(w, 2)`)
  parsed in-process and counted by the `created by` file:line, relative to `DLA_PROJECT`
  (standard-library sites as `$GOROOT/src/...`, other paths unchanged). Goroutines created by
  `testing.*`, `runtime.*` and the harness ticker are left out; goroutines without a creator (main)
  are not grouped.
- `purge`: `DLA_PURGE=free-os-memory` records `go.released`, `go.total`, `go.heap_objects`, `os.rss`,
  `os.rss_anon` right after the collections and again after `debug.FreeOSMemory()`; `gc` records the
  pre- and post-collection figures; other kinds are a limitation.
- `tick` records every `DLA_TICK_MS` (when set): no forced collection, the metrics, OS values and
  the goroutine groups. The profile stops the world for a moment, so keep the interval above ~100 ms.
- `detectors`: goleak output (driver mode, tag `dlagoleak`).
- Coverage comes from `go test -coverprofile` with `-coverpkg=./...`; the dump carries none.

## Driver mode

```
go test -vet=off -count=1 -run '^TestDLADriver$' [-tags dlagoleak] ./<package dir>
```

One process, one `checkpoint` record per round (`x` = units completed), the last one with the purge
and, with the tag, the goleak report (the ticker is stopped first). Because the file lives in the
package under test, the hooks can use unexported identifiers and package variables; `GoVar` probes are
read by the optional `probes` hook (below), which fills the `probes` list of every checkpoint.

### Placeholders

| Placeholder | Where | Meaning |
|---|---|---|
| `@@DLA_PACKAGE@@` | all three files | package name (`main` in command mode) |
| `@@DLA_STEP@@` | driver | units per round (default for `DLA_STEP`) |
| `@@DLA_WARMUP@@` | driver | warm-up rounds (default for `DLA_WARMUP`) |
| `@@DLA_ROUNDS@@` | driver | measured rounds (default for `DLA_ROUNDS`) |
| `@@DLA_REPEAT@@` | driver | repeated key space size (default for `DLA_REPEAT`) |

Run-time environment of the driver: `DLA_KEY_MODE` (`fresh` uses `key = i`, `repeated` uses
`key = i % repeat`), `DLA_STEP`, `DLA_WARMUP`, `DLA_ROUNDS`, `DLA_REPEAT`.

### Hooks

| Hook | Signature | Fill with |
|---|---|---|
| `DLA-HOOK setup` | `func dlaSetup()` | construct the services/caches/pools; keep them in package-level variables; `dlaT` is the running `*testing.T` |
| `DLA-HOOK unit` | `func dlaUnit(i, key int)` | one unit of the workload for `key`; derive ids and payloads from `key` so a repeated key reproduces an earlier request |

| `DLA-HOOK probes` (optional) | `func dlaProbes() []map[string]any` | one `dlaProbe(id, name, path, size)` per probed package variable (`GoVar`); `size` is its entry count; leave `return nil` when there are none |

The hooks are `dlaSetup`/`dlaUnit` (not `setup`/`unit`) so they cannot collide with identifiers of
the package under test.

### goleak

`zz_dla_goleak_test.go.tmpl` (`//go:build dlagoleak`) sets `dlaGoleakCheck` to `goleak.Find()`.
Phase 2 writes it and passes `-tags dlagoleak` only when `go.uber.org/goleak` resolves offline;
otherwise the driver skips the check and the harness records the gap.
