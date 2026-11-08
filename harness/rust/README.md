# Rust driver

| File | Role |
|---|---|
| `dla_harness.rs` | loop driver template, written to `src/bin/dla_harness.rs` of the scratch copy; only for crates with a library target |

Bin-only crates cannot be driven this way (the driver imports the library); `dla harness gen`
refuses driver mode for them with that reason and command mode (the allocator shim in
`harness/native/`) still works.

## Run

```
cd <scratch project root>
CARGO_PROFILE_RELEASE_DEBUG=line-tables-only cargo build --release --bin dla_harness
DLA_HARNESS_OUT=<harness dir>/out/<pid>.jsonl target/release/dla_harness
```

The driver installs a counting `#[global_allocator]` over `System` (live bytes, live blocks, peak
of live bytes) and writes one `checkpoint` record per round (`x` = units completed): `values` has
`native.live_bytes`, `native.live_blocks`, `native.peak_bytes` and the Linux `os.*` set from
`/proc/self/status` (plus `tokio.alive_tasks` when the tokio hook is filled in). With
`DLA_PURGE=trim` the last record also carries `purge.before/after` around glibc `malloc_trim(0)`
(other targets: a limitation). Env overrides: `DLA_KEY_MODE` (`fresh`|`repeated`), `DLA_STEP`,
`DLA_WARMUP`, `DLA_ROUNDS`, `DLA_REPEAT`. Do not preload `libdlashim.so` into the driver (or set
`DLA_SHIM_DISABLE=1`): it already measures the allocator in-process.

The driver needs no dependencies. Because it counts allocations itself, it measures only what the
Rust allocator sees; C libraries linked into the process allocate outside it (visible in `os.*`).

### dhat-rs variant

`dla harness gen` does not generate this variant yet; it records a gap when `dhat` does not resolve
offline (it does not in the benchmark sandbox) and uses the counting-allocator driver. By hand:
add `dhat` as a dev-dependency, replace the `GLOBAL` allocator with `dhat::Alloc`, create `dhat::Profiler::builder().testing().build()` in `main`, and assert on
`dhat::HeapStats::get()` after the last round. The notes are also in the file header. Untested here.

### Tokio

`DLA-HOOK tokio` returns `None` by default. Fill it only for projects that depend on tokio: keep
the runtime `Handle` in `State` and return `handle.metrics().num_alive_tasks()`. The crate was not
available offline, so the API (stable from tokio 1.39; earlier versions need `--cfg tokio_unstable`)
is unverified.

## Placeholders

| Placeholder | Meaning |
|---|---|
| `@@DLA_CRATE@@` | library target name of the crate under test (underscores), used by the hooks |
| `@@DLA_STEP@@` | units per round (default for `DLA_STEP`), integer literal |
| `@@DLA_WARMUP@@` | warm-up rounds, integer literal |
| `@@DLA_ROUNDS@@` | measured rounds, integer literal |
| `@@DLA_REPEAT@@` | repeated key space size, integer literal |

## Hooks

| Hook | Where | Fill with |
|---|---|---|
| `DLA-HOOK state` | `struct State` | fields `setup()` fills and `unit()` uses |
| `DLA-HOOK setup` | `fn setup() -> State` | build the objects under test with `@@DLA_CRATE@@::...` |
| `DLA-HOOK unit` | `fn unit(state, i, key)` | one unit of work for `key`; derive ids from `key` so a repeated key reproduces an earlier request |
| `DLA-HOOK tokio` | `fn tokio_alive_tasks` | only for tokio projects; see above |

The stubs call `todo!` until replaced. The template contains `unsafe` (the allocator impl and
`malloc_trim`) and is generated into the scratch copy only, never into the dla workspace.
