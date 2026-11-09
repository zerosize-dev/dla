# Backends: which `dla summarize` operation reads which capture

`dla summarize <op> <files...>` turns a captured file into bounded evidence. `dla summarize --list`
prints the adapters of the installed release with their tool and whether it is available here; trust
it over this table. `--backend auto` (default) picks by the format of the inputs, DLA's own readers
first; `--backend <name>` forces one.

## Operations and adapters

| Op | Runtime | Backend | Reads | Tool needed | Row when the tool is missing |
|---|---|---|---|---|---|
| `heap-graph` | JVM | `dla-hprof` | `.hprof`, `.hprof.gz`, 3 or more of one process | none | none |
| `heap-graph` | native | `valgrind-vgdb` | the `vgdb-*.txt` answers of `harness run --tool valgrind` (one chain per point) | none to read; valgrind and vgdb to capture | `valgrind` (harness) |
| `type-histogram-diff` | JVM | `dla-hprof-histogram` | HPROF dumps | none | none |
| `type-histogram-diff` | JVM | `jcmd-class-histogram` | `GC.class_histogram` text (`attach` or `--capture histogram`) | none to read (`jcmd` to attach) | none for reading; `jcmd` (attach) |
| `type-histogram-diff` | native | `shim-stack-groups` | shim `dump.jsonl` of command-mode points | `addr2line` for project sites | `addr2line` |
| `type-histogram-diff` | any | `harness-types` | harness dump files with a type histogram | none | none |
| `alloc-stack-diff` | JVM | `jfr-old-object-sample` | a `.jfr` recording | `jfr` (JDK 11+) | `jfr-old-object-sample` |
| `alloc-stack-diff` | JVM | `jfr-old-object-text` | `jfr print` text of `jdk.OldObjectSample` | none | none |
| `alloc-stack-diff` | native | `valgrind-memcheck` | memcheck XML (`valgrind.xml`) | none to read; valgrind to capture | `valgrind` (harness) |
| `alloc-stack-diff` | native | `heaptrack` | `heaptrack_print` text, a `-d` diff, `-F` folded files; raw `heaptrack.*.gz`/`.zst` | `heaptrack_print` for raw recordings only | `heaptrack_print` (raw), `heaptrack` (harness) |
| `alloc-stack-diff` | native | `dhat` | valgrind DHAT or dhat-rs JSON (live at exit) | none | none |
| `alloc-stack-diff` | native | `lsan` | LeakSanitizer text | none | none |
| `alloc-stack-diff` | any | `dla-alloc-json` | DLA's own allocation JSON | none | none |
| `non-heap` | JVM | `jvm-thread-dump` | `threads.json` (`Thread.dump_to_file -format=json`) | none | none |
| `non-heap` | JVM | `jvm-classloader-stats` | `VM.classloader_stats` text | none | none |
| `non-heap` | JVM | `jvm-nmt` | `nmt-summary.txt` or `nmt-diff.txt` (not mixed) | none | none |
| `non-heap` | any | `proc-fd` | `ls -l /proc/<pid>/fd` listings, harness dumps with `fd.*` | none | none |
| `non-heap` | any | `dla-nonheap-json` | DLA's own non-heap JSON | none | none |

Cut 2, not available in this release (do not call them; there is no adapter): Go `goroutineleak`
profile (needs a Go 1.27 toolchain; a pass would run with `GOTOOLCHAIN=go1.27.x`) and the Go goroutine
and heap profile adapters, Node V8 heap snapshots, Python memray and tracemalloc, async-profiler, MAT,
bytehound, goref. For those runtimes keep using the harness columns and `dla attach`.

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
- Thread dumps, class loader groups and NMT categories are counts: they report growth and never tie.
  `proc-fd` counts descriptors by kind and never ties.
- An empty or flat result never refutes a leak. Only a plateau discriminator (`dla alloc-test`,
  `dump-plateau`) favors non-leak.
