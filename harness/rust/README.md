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

### Features and coverage

`dla harness gen --lang rust --driver --features a,b [--no-default-features]` adds them to the `cargo build` of the
driver (the command in `harness.json` keeps them). A target whose module the crate declares behind
`#[cfg(feature = "x")]`, or inside a `cfg_x! { .. }` block whose `macro_rules!` the crate defines with that attribute, gets
the line "behind feature x: generate with --features x" in the block; the reader is a line scan of the `mod` declarations
up to `lib.rs` (not seen: `#[path]`, modules declared in other macros or inline, a gate on the item, `cfg_attr`, a feature
another feature enables; `any(a, b)` names `a`).

`dla harness run` then builds the driver once more with `RUSTFLAGS=-C instrument-coverage` into
`harness/H<n>/cov-build/`, runs it for one round of `step` units (no warm-up, no captures, no sampler) with
`LLVM_PROFILE_FILE` in `cov/`, merges the `.profraw` with `llvm-profdata` and exports the functions that ran with
`llvm-cov export` into `cov/p1-<mode>-cover.rust.json` (file, demangled name, start line; dependencies and the standard
library left out). The tools are the toolchain's (`<sysroot>/lib/rustlib/<host>/bin`, component `llvm-tools`), else
those on PATH when their LLVM major version is the one rustc reports. The measured points are not instrumented.

### Allocation stacks (`--capture alloc`)

`dla harness gen --driver` also builds the allocator shim (`agent/libdlashim.so`, needs `cc`; a missing compiler is not a
refusal). `dla harness run H<n> --mode fresh --capture alloc` runs the driver four more times, 1 to 4 rounds of `step` units
without warm-up, with the shim preloaded and narrowed to `dla_harness` (the build is the measured one: `line-tables-only`,
no frame pointers needed, the shim unwinds with `backtrace`), and leaves `out/p<k>-alloc/dump.jsonl` and the shim's record split out into
`shim-dump.jsonl`. `dla summarize alloc-stack-diff` reads those files: the first frame inside the project (the driver's own
file is skipped), demangled by `addr2line -C`. The measured points are not slowed; the extra processes are (about ten times for a
unit that only allocates). Command mode has the shim in every point already: `--capture alloc` copies the shim dumps to
`out/p<k>-native/shim-dump.jsonl`.

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

## Rust version

The driver holds to Rust 1.70 (the newest thing in it is `Result::is_ok_and`; a let chain, an inline `const` block, `is_none_or` and `LazyLock` are left out because they need 1.79 to 1.88). 1.70 is the lowest that needs no workaround: older code would need hand-written replacements for `is_ok_and` and the `let ... else` already used (1.65). The `extern` block is written `unsafe extern` only when the crate's edition is 2024 or later (`edition.workspace = true` is resolved from the workspace root), because an older compiler (before 1.82) rejects the marker and the 2024 edition requires it. The harness is built with the toolchain the project pins; a build that fails says which channel built it. It was checked with 1.80.0, the oldest toolchain installed where it was made; 1.70 itself was not run.

## Features that gate code inside functions

A library can compile to a no-op unless a cargo feature is on, with `#[cfg(feature = "x")]` inside its functions and on its items and no gated `mod`. `dla harness gen` counts those sites per feature that the `default` list of `Cargo.toml` does not turn on (directly or through another feature), per file of the crate, and writes into the target block and the `_attention` of its output the features that the driver build leaves off, with the counts and the exact regenerate command. `harness/H<n>/features.json` keeps the same for `dla trend`, the queue and `dla ledger close-rest`: a target that was not executed gets the command first, and the queue does not settle it as not reached while a feature with gated sites was on in none of the harnesses that ran for it.

## Tool captures

Command mode (the crate's binary started directly) also takes `dla harness run <H> --tool valgrind|heaptrack`;
the files and the `dla summarize` commands are in `harness/native/README.md`. Driver mode does not
(its counting allocator is the measurement). Rust data symbols come out of valgrind in v0 mangling and
`dla` demangles them.
