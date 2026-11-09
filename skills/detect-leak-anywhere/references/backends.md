# Backends: which `dla summarize` operation reads which capture

`dla summarize <op> <files...>` turns a captured file into bounded evidence. `dla summarize --list`
prints the adapters of the installed release with their tool and whether it is available here; trust
it over this table. `--backend auto` (default) picks by the format of the inputs, DLA's own readers
first; `--backend <name>` forces one.

## Operations and adapters

| Op | Runtime | Backend | Reads | Tool needed | Row when the tool is missing |
|---|---|---|---|---|---|
| `heap-graph` | JVM | `dla-hprof` | `.hprof`, `.hprof.gz`, 3 or more of one process | none | none |
| `heap-graph` | Node | `node-heapsnapshot` | `.heapsnapshot`, 3 or more of one process (driver mode) | none | none |
| `heap-graph` | native | `valgrind-vgdb` | the `vgdb-*.txt` answers of `harness run --tool valgrind` (one chain per point) | none to read; valgrind and vgdb to capture | `valgrind` (harness) |
| `type-histogram-diff` | JVM | `dla-hprof-histogram` | HPROF dumps | none | none |
| `type-histogram-diff` | JVM | `jcmd-class-histogram` | `GC.class_histogram` text (`attach` or `--capture histogram`) | none to read (`jcmd` to attach) | none for reading; `jcmd` (attach) |
| `type-histogram-diff` | Node | `node-heapsnapshot-types` | `.heapsnapshot` of any points (class histogram) | none | none |
| `type-histogram-diff` | Go | `go-pprof-heap-functions` | `go-<n>.heap.pb.gz` (rows are functions, not types) | none | none |
| `type-histogram-diff` | Python | `py-types` | `py-<n>.types.json` | none | none |
| `type-histogram-diff` | native | `shim-stack-groups` | shim `dump.jsonl` of command-mode points | `addr2line` for project sites | `addr2line` |
| `type-histogram-diff` | any | `harness-types` | harness dump files with a type histogram | none | none |
| `alloc-stack-diff` | JVM | `jfr-old-object-sample` | a `.jfr` recording | `jfr` (JDK 11+) | `jfr-old-object-sample` |
| `alloc-stack-diff` | JVM | `jfr-old-object-text` | `jfr print` text of `jdk.OldObjectSample` | none | none |
| `alloc-stack-diff` | Node | `node-heapprofile` | `alloc-<n>.heapprofile` (sampling heap profile) | none | none |
| `alloc-stack-diff` | Go | `go-pprof-heap` | `go-<n>.heap.pb.gz` (heap profile, stacks) | none | none |
| `alloc-stack-diff` | Python | `py-tracemalloc` | `py-<n>.alloc.json` | none | none |
| `alloc-stack-diff` | Python | `py-memray` | `py-<n>.memray.json` (needs memray in the target's interpreter to capture) | none to read | none |
| `alloc-stack-diff` | native | `valgrind-memcheck` | memcheck XML (`valgrind.xml`) | none to read; valgrind to capture | `valgrind` (harness) |
| `alloc-stack-diff` | native | `heaptrack` | `heaptrack_print` text, a `-d` diff, `-F` folded files; raw `heaptrack.*.gz`/`.zst` | `heaptrack_print` for raw recordings only | `heaptrack_print` (raw), `heaptrack` (harness) |
| `alloc-stack-diff` | native | `dhat` | valgrind DHAT or dhat-rs JSON (live at exit) | none | none |
| `alloc-stack-diff` | native | `lsan` | LeakSanitizer text | none | none |
| `alloc-stack-diff` | any | `dla-alloc-json` | DLA's own allocation JSON | none | none |
| `non-heap` | JVM | `jvm-thread-dump` | `threads.json` (`Thread.dump_to_file -format=json`) | none | none |
| `non-heap` | JVM | `jvm-classloader-stats` | `VM.classloader_stats` text | none | none |
| `non-heap` | JVM | `jvm-nmt` | `nmt-summary.txt` or `nmt-diff.txt` (not mixed) | none | none |
| `non-heap` | Node | `node-resources` | `resources.json` (active resources by creation site) | none | none |
| `non-heap` | Go | `go-goroutine` | `go-<n>.goroutine.txt` (debug=2) or `go-<n>.goroutine.pb.gz`, not mixed | none | none |
| `non-heap` | Go | `go-goroutineleak` | `go-<n>.goroutineleak.txt` | none to read; Go 1.27 to capture | `go/goroutineleak` (probe) |
| `non-heap` | Python | `py-resources` | `py-<n>.nonheap.json` (threads, tasks, ResourceWarnings) | none | none |
| `non-heap` | any | `proc-fd` | `ls -l /proc/<pid>/fd` listings, harness dumps with `fd.*` | none | none |
| `non-heap` | any | `dla-nonheap-json` | DLA's own non-heap JSON | none | none |

Not read by any adapter (no coverage row either): Go `goref` and `viewcore` heap graphs, async-profiler,
MAT, bytehound, memlab. For those keep using the harness columns and `dla attach`. Tools DLA has no
driver for at tier 3 (bcc `memleak`, GPU memory, Valgrind on a JVM) get a `blocked` coverage row in
`dla probe`: a blind spot to state in the report.

## Missing tools

- `--backend auto` with a tool-backed adapter and no tool: exit 0, `backend: null`, `evidence: []`,
  a `coverage_gap` list and a `missing` coverage row. No evidence was made, so nothing is cited and
  nothing is refuted.
- `--backend <name>` without its tool: exit 2, refusal `tool-missing`, the `missing` row written first.
  Install the tool (the refusal prints a hint such as `apt-get install valgrind`), do not retry in a loop.
- `dla harness run H1 --tool valgrind|heaptrack` without the tool refuses with `tool-missing` and leaves
  a `missing` row. Record the candidate `unresolved` with reason `tool_missing`.
- A missing row is a coverage gap, never evidence for or against a leak.

## Capture, then summarize

| Runtime | Capture (stage 4) | Summarize (stage 6) |
|---|---|---|
| JVM | `dla harness run H1 --capture all` (or `heap,histogram,jfr,nonheap`) | one command per file kind, as printed: `type-histogram-diff` on `histogram-*.txt`, `alloc-stack-diff` on the `.jfr`, `non-heap` on `threads.json`, then on `classloader-stats.txt`, then on `nmt-diff.txt` |
| JVM driver mode | the same flag; all `heap-*.hprof` are one process | `heap-graph` on all dumps of the label |
| JVM process or artifact | `dla attach jcmd --pid <P> --op histogram\|thread-dump\|nmt\|jfr\|heap-dump` | the same ops on `attach/P<n>/` files |
| Node | `dla harness run H1 --capture alloc,nonheap` (add `heap` for V8 snapshots) | `alloc-stack-diff` on `alloc-*.heapprofile`; `non-heap` on `resources.json`; `type-histogram-diff` on the snapshots |
| Node driver mode | `--capture heap,alloc,nonheap`; all `heap-*.heapsnapshot` of one pid directory are one process | `heap-graph` on all snapshots (3 or more) |
| Go | `dla harness run H1 --capture alloc,nonheap` | `alloc-stack-diff` on `go-*.heap.pb.gz`; `non-heap` on `go-*.goroutine.txt`, then on `go-*.goroutineleak.txt` |
| Python | `dla harness run H1 --capture histogram,alloc,nonheap` | `type-histogram-diff` on `py-*.types.json`; `alloc-stack-diff` on `py-*.alloc.json`, then on `py-*.memray.json`; `non-heap` on `py-*.nonheap.json` |
| Rust, native (program started directly) | `dla harness run H1 --tool valgrind` | `alloc-stack-diff` on the `valgrind.xml` files; `heap-graph` on the `vgdb-*.txt` files |
| Rust, native | `dla harness run H1 --tool heaptrack` | `alloc-stack-diff` on `heaptrack.txt` files, a second command on `heaptrack-diff.txt` |
| Native with the shim | the normal command-mode harness; it prints no summarize command for native dumps | run by hand: `type-histogram-diff` and `non-heap` on `harness/H1/out/p<k>-<series>/dump.jsonl` (shim stack groups; fd counts) |
| JVM with the shim (JNI) | the normal command-mode harness | `type-histogram-diff` on `shim-dump.jsonl`, as printed |

Run exactly the commands `harness run` prints under `summarize`; each one uses one op and one input
format. Mixed formats in one call are refused as `mixed-formats` and the refusal names the fix.

## What each result proves

- A passing growth column that carries a `tie` (rule `holder`, `site` or `type`) ties to a finding:
  growth confirms only the holder it measures. Cite the `growth-series` evidence in `dla ledger confirm`.
- A column that passes but is untied (no `tie`, or a type match listed under `ambiguous`) is reported, and confirms
  nothing. `needed` says what is missing. `--finding F` never forces a tie; it only narrows to columns
  that already tie to `F`.
- JFR is a sample of old objects, not a census. Counts are small and noisy, so a short run or a small
  leak often fails the gate; use a larger `--step` before concluding anything.
- `valgrind-memcheck` and `heaptrack` tie by allocation site (first project frame). A definitely-lost
  block has no holder, so attribution is by site only; `valgrind-vgdb` gives a holder (a static data
  symbol) when a root points at the block.
- Node: `node-heapsnapshot` names holders from a snapshot series of one process (3 or more): a module
  variable or global property is `static/global`, a timer callback or event listener table is
  `registration-table`, a closure variable `closure/continuation`, an object property `container-field`.
  It ties by holder identity only and prints `needed` with a `dla ledger annotate F --holder ...` command
  when the finding's holder differs from what the snapshots show. Strings are one `(string)` row and are
  never named after their text. A snapshot is tier 4 data (memory contents); a class histogram
  (`node-heapsnapshot-types`) ties to nothing. `node-heapprofile` is a sample (builtin frames are charged
  to the nearest JavaScript frame; a `<script>.map` next to the script maps frames back to the source,
  `source_maps.frames_mapped`); a profile frame is the function, so a site tie by line happens only when
  the function starts on the finding's line. `node-resources` groups are `Type@file:line` and tie by site.
- Go: `go-goroutine` groups goroutines by creation site (`created by` in the debug=2 text) and ties by
  site; the protobuf form has no creation site and never ties. `go-goroutineleak` is a second signal
  (`should-be-dead` evidence): it lists goroutines the runtime proved blocked forever, never writes a
  growth column and never refutes. An empty profile proves nothing, because goroutines parked on a channel
  reachable from a global are not reported. `go-pprof-heap` ties by allocation site; `go-pprof-heap-functions`
  rows are functions and never tie. Heap profiles reflect the last completed collection.
- Python: `py-tracemalloc` and `py-memray` tie by the first project frame (they name no holder);
  `py-types` ties only by type identity and never for built-in containers; `py-resources` groups tie by
  the line that created the thread or task; whole-process counts are metrics. memray is read through a
  helper run in the target's interpreter; without memray there, the capture list carries an error and a
  coverage gap.
- Thread dumps, class loader groups and NMT categories are counts: they report growth and never tie.
  `proc-fd` counts descriptors by kind and never ties.
- Cross-runtime: `dla correlate H1` (below) is the only way a native site gets a managed finding.
- An empty or flat result never refutes a leak. Only a plateau discriminator (`dla alloc-test`,
  `dump-plateau`) favors non-leak.

## Correlate: native growth under a foreign-function boundary

`dla correlate <H> [--series <S>]` is for a harness whose manifest lists an FFI boundary and whose series
holds the allocator shim's `native.live_bytes@<file:line>` columns (a JVM project that loads a JNI library
today). A tie never crosses runtimes, so those columns would tie to nothing in the managed worker. For
each native site under a boundary method the command compares per-iteration slopes (the `dla trend`
gate): the site must pass, the process footprint (`os.rss_anon`, else `os.rss`) must pass, and the site
slope must be between 0.2 and 1.25 of the footprint slope. With a match it writes one correlation evidence
(both slopes, the boundary, the native stack) and adds a candidate to the worker it runs in: origin
`correlation`, site the native site, holder `native-handle/global-ref`, region off-heap, owner the managed
method. The shim column then ties to that finding by site; confirmation goes through the normal ledger
rules. Run it in the managed worker (`dla --run <id> --worker jvm correlate H1`) after the harness run. A
harness with no boundary, or no series with shim columns, is refused and the refusal names the fix. PyO3,
N-API and cgo boundaries are detected and reported by `dla probe`, but the shim measures JNI only.

## Known leaks

`dla probe` checks `data/known-leaks.toml` (documented behavior of a runtime or library that looks like a
leak or hides one, each entry with a public source) against the project and writes `known_leaks` verdicts
(`true`, `false` or `maybe`, with a reason) into the probe evidence. They are context: cite them when
choosing what to test, but a known-leak entry never becomes a finding and never confirms anything.

## Tier 3: kernel level

`dla attach bpftrace --pid <P> --op record [--duration 30s]` (consent tier 3, Linux, root) counts bytes
mapped per symbolized user stack from `mmap`, `munmap` and `brk`, with a time cap and an output cap. It
needs the `bpftrace` program, root, tracefs and kernel BTF; every missing prerequisite is named in the
refusal. No `dla summarize` adapter reads its output: cite the attach evidence. It sees growth no runtime
counter owns (arenas, mapped files, fragmentation); it does not say who holds the memory.
