# DLA harness templates

Source files that the `dla` binary embeds (`include_str!`) and writes into a run's harness
directory (`.dla/<run>/harness/H<n>/`). Each language has its own directory with a README that
lists its placeholders and hooks:

| Directory | Contents |
|---|---|
| `jvm/` | `DlaAgent.java` (premain agent), `DlaDump.java` (dump and probe code), `DlaHarness.java` (driver), `MANIFEST.MF` |
| `python/` | `sitecustomize.py` (agent), `dla_harness.py` (driver), `test_dla_harness.py` (pytest-memray variant) |
| `node/` | `dla-preload.js` (agent), `dla-harness.js` (driver) |
| `go/` | `zz_dla_harness_test.go.tmpl` (command mode), `zz_dla_driver_test.go.tmpl` (driver), `zz_dla_goleak_test.go.tmpl` (optional goleak check) |
| `native/` | `dla_alloc_shim.c` (allocator shim for Rust and native programs), `dla_harness.c` (C driver) |
| `rust/` | `dla_harness.rs` (driver for crates with a library target) |

Each directory has a README that lists the files, placeholders and hooks of its language.

## Environment contract (every agent, shim and driver)

The measured process writes JSON lines to the file named by `DLA_HARNESS_OUT`
(`<harness dir>/out/<pid>.jsonl`). A driver writes one `checkpoint` line per round; an agent
writes one `exit` line at exit and may write periodic `tick` lines. One record:

```json
{"dla_harness":1,"phase":"checkpoint|exit|tick","pid":123,"argv":["..."],"exe":"...","runtime":"jvm",
 "x":2000,"t_ms":1234,"gc_forced":true,
 "values":{"jvm.heap_used":123456},"values_pre_gc":{"jvm.heap_used":456789},
 "purge":{"kind":"trim","before":{"os.rss":1},"after":{"os.rss":1}},
 "groups":{}, "raw_sites":{},
 "probes":[{"id":"P1","name":"ORDERS_BY_ID","path":"shop.OrderCache.ORDERS_BY_ID","size":1750,"reachable":5400,"bytes":null,"error":null}],
 "types":[{"name":"shop.Order","count":1750,"bytes":84000}],
 "coverage":{"kind":"classes|functions","items":["shop.OrderCache"]},
 "detectors":[], "limitations":["..."]}
```

`values` are read after a forced collection when the runtime has one; `values_pre_gc` holds the
values read just before it. `purge` is present only when `DLA_PURGE` is set, at the exit record
or the last checkpoint. Absent optional sections are simply left out.

| Variable | Meaning |
|---|---|
| `DLA_HARNESS_OUT` | file the records are appended to; when unset the agent does nothing |
| `DLA_HARNESS_PROBES` | path of `probes.json`: a JSON array of `ProbeSpec` (`{"id","finding","target":{"kind",...},"site"}`); an object with a `probes` array is also accepted. A runtime reads only its own probe kinds. Kind spelling is compared ignoring case, `-` and `_` (`JvmClass`, `jvm-class`) |
| `DLA_PURGE` | `none`, `trim`, `gc`, `free-os-memory`, `trim-native-heap`, `jemalloc`, `empty-cache`. Handled per runtime: JVM `trim-native-heap` and `gc`; Python `trim` and `gc`; Node `gc`; Go `free-os-memory` and `gc` (agent and driver); the allocator shim `trim` and `jemalloc` (the latter untested); the Rust and C drivers `trim`. An unsupported kind becomes a limitation. `empty-cache` has no handler yet |
| `DLA_PROJECT` | scratch project root; used to select project classes/functions for coverage |
| `DLA_TICK_MS` | optional period of light `tick` records (no forced collection, no probes) |
| `DLA_ITER` | iteration count `n` of a command-mode run; copied into `x` of the exit record |
| `DLA_CAPTURE` | JVM only: comma list of raw captures to write next to the records: `histogram` (full class histogram text), `heap` (HPROF dump), `jfr` (flight recording with old-object samples and paths to GC roots), `nonheap` (thread dump as JSON, class loader statistics, native memory summary and its change since a baseline). Unset: none |
| `DLA_CAPTURE_DIR` | directory the captures go to (`<dir>/<pid>/<file>`); required with `DLA_CAPTURE` |
| `DLA_CAPTURE_DUMPS` | JVM driver: how many of the rounds take a heap dump (default 4: the end of the warm-up, the last round and evenly spaced rounds between) |

Driver-only variables (read by `DlaHarness` and `dla_harness.py`; the defaults are baked into the
file at generation time, so the variables are optional overrides):

| Variable | Meaning |
|---|---|
| `DLA_KEY_MODE` | `fresh` (key = i) or `repeated` (key = i % repeat) |
| `DLA_STEP` | units of work per round |
| `DLA_WARMUP` | number of leading warm-up rounds (they also write checkpoints; the series marks them) |
| `DLA_ROUNDS` | measured rounds after the warm-up (`dla harness run` sets it to `--iterations` plus `--extra`); the driver runs `DLA_WARMUP + DLA_ROUNDS` rounds in total |
| `DLA_REPEAT` | size of the repeated key space |

Captures (`DLA_CAPTURE`) are listed in a `captures` array of the record that triggered them:
`{"kind":"heap","file":"heap-2.hprof","role":"heap dump (HPROF)","bytes":123,"ms":45}` (or
`"error"` when the capture failed). A driver takes the histogram at every checkpoint, heap dumps at
the selected rounds, and the flight recording and non-heap reports at the last round. A
command-mode process takes whatever `DLA_CAPTURE` names at exit (`dla harness run` names the heavy
kinds for a few points only). The flight recording is written by the JVM itself when it exits, so
it is not in the list; `dla harness run` lists the file by its size. The files are listed in the
harness manifest (`captures`) and in `harness/H<n>/captures.json` with the `dla summarize`
commands that read them.

`x` of a driver checkpoint is the number of units completed. When a driver wrote checkpoints, the
agent in the same process writes no `exit` record.

## Placeholder syntax

Generation-time substitution is plain text replacement of `@@DLA_NAME@@` (upper-case name between
double at-signs), done by `dla harness gen` when it writes a template. Every placeholder is
listed in the README of its language. An unreplaced placeholder is a generation bug; a template
containing placeholders is not valid source until generated. Where a placeholder is a package
name that may be empty (`@@DLA_PACKAGE@@` for the default Java package), `dla` removes the whole
`package ...;` line.

## Hook syntax

Driver templates contain hooks the model fills in. A hook is a line comment

```
// DLA-HOOK <name>: <instruction>        (Java, Go, Rust, C, Node)
# DLA-HOOK <name>: <instruction>         (Python)
```

directly followed by a stub body that fails loudly (throws / raises) until it is replaced. The
model replaces the stub body, keeps the comment, and does not touch the rest of the file. The hook
names per language are listed in the language README.
