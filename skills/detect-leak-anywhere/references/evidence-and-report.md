# Evidence and report

How the measurements are named, how `dla trend` judges them, what a discriminator is, and which
ledger rules turn that evidence into a confirmed or refuted finding. All numeric thresholds below
are provisional: they are uncalibrated until benchmark milestone M7, and every trend, alloc-test
and residual result says so (`calibrated: false`, or a limitation "thresholds are provisional").
Do not present a gate verdict as more precise than that.

## Series

A series is one table of measurements: `.dla/<run>/series/S<n>.json`. Rows are points ordered by
`x`; columns are named values. `axis` says what `x` means: `n` (units of work, command mode),
`iteration` (units completed, driver mode) or `seconds` (`dla sample`). The first `warmup` points
are warm-up and are left out of every test. `source.kind` is `harness`, `sample`, `alloc-test` or
`gap`; `source.mode` is `fresh`, `repeated` or `native` (command mode: keys as the workload makes
them). `events` holds things done between points, such as a purge (`purge:trim`). `limitations`
lists what the series could not measure. Only `dla` writes series.

Every column has a unit (`bytes`, `count`, `ratio`, `seconds`), a kind, a region and flags:

- **kind**: `gauge` (a level in bytes) or `counter` (an integer count of units: goroutines, fds,
  entries). Gauges are judged by the statistical gate, counters by the every-delta rule.
- **region**: `heap` (managed heap, allocator live bytes and holder probes), `off-heap` (native
  allocator, direct buffers, `gap.unexplained`), `non-heap` (threads, goroutines, stacks, fds,
  timers, listeners, metaspace) or `process` (whole-process totals such as `os.rss`).
- **after_gc**: the value was read after a forced collection. A column ending `.pre_gc` is the
  same value read just before it. Sampled runtime heap columns (`dla sample`) are never after GC.
- **scope**: `process`, or `container` for `cgroup.*` figures (container-wide; on cgroup v1,
  `cgroup.current` includes page cache and is not comparable with `cgroup.anon`).
- **site**: the `file:line` the column is tied to (probe definition, creation site, allocation
  site); **finding**: the finding it was probed for.
- A series has at most 64 columns. Grouped and per-type columns beyond that are dropped, lowest
  growth first, and the drop is written to `limitations`.

### Column names

| Group | Names |
|---|---|
| OS, every run (Linux) | `os.rss`, `os.rss_anon`, `os.rss_file`, `os.rss_shmem`, `os.swap`, `os.vm_data`, `os.vm_hwm`, `os.pss`, `os.pss_anon`, `os.private_dirty`, `os.fds`, `os.threads` |
| OS, macOS / Windows | macOS `os.rss`, `os.phys_footprint`, `os.threads`, `os.fds`; Windows `os.rss` (working set), `os.private`, `os.pagefile`, `os.handles`, `os.threads` |
| Container | `cgroup.current`, `cgroup.anon`, `cgroup.file` |
| JVM | `jvm.heap_used`, `jvm.heap_committed`, `jvm.nonheap_used`, `jvm.nmt_committed`, `jvm.threads`, `jvm.classes_loaded` |
| Go | `go.heap_live`, `go.heap_objects`, `go.total`, `go.released`, `go.stacks`, `go.goroutines` |
| Node | `node.heap_used`, `node.heap_total`, `node.external`, `node.array_buffers` |
| Python | `py.traced`, `py.gc_objects` |
| Rust, native | `native.live_bytes`, `native.live_blocks`, `native.peak_bytes` |
| Grouped counters | `<base>@<file:line>`, for example `go.goroutines@internal/notify/subscription.go:33`, `node.timers@src/heartbeat.js:15`, `native.live_bytes@src/decoder.c:29` |
| Holder probes | `probe:<target>` (entry or reachable-object count, a counter), `probe:<target>.bytes` (a gauge, when the runtime can estimate bytes) and, for a JVM class loader, `probe:<target>.classes` (the classes it defined) |
| Type counts | `jvm.class:<name>.count`, `py.type:<name>.count` (top types by growth) |
| Gap | `gap.accounted`, `gap.unexplained` |

A column's meaning depends on the mode. In command mode the process exits at each point and the
agent reads its figures at exit, after a forced collection where the runtime has one;
`os.vm_hwm` and the other peaks come from sampling the process while it runs, so a high-water
mark that was freed before exit shows only in the peak columns (`os.vm_hwm`, `native.peak_bytes`)
and not in the live ones. A Rust or native program that drops its memory before exit shows
almost nothing in `native.live_bytes`; use driver mode for per-round live bytes.

## Trend

`dla trend S1` analyses every column over its post-warm-up points and writes `trend/T<n>.json`.
Per column:

| Field | Meaning |
|---|---|
| `n` | post-warm-up points with a value |
| `mk` | Mann-Kendall test: `s`, `var_s`, `z`, `p_increasing` (one-sided), `tau`. Exact p for n up to 10 without ties |
| `sen` | Theil-Sen `slope` and `intercept`, with a 95% confidence interval `lo`, `hi` |
| `halves` | slope of the first and of the second half and their `ratio` (second over first) |
| `every_delta` | counters only: the number of consecutive deltas, whether all were at least 1, their `min` and `max`. The delta from the last warm-up point to the first measured point counts |
| `noise` | robust residual scale around the Sen line (1.4826 times the median absolute deviation) |
| `rise` | `slope` times the x-span of the post-warm-up points |
| `shape` | `flat`, `declining`, `steady`, `stair`, `spike`, `plateau` or `insufficient` |
| `gate`, `gate_failed` | `pass` or `fail`, and the reasons ("n < 8", "p >= 0.05", "decaying half-split", "delta < 1") |
| `candidates` | findings tied to the column: those whose site equals the column's site, else those in the same file within the same enclosing function or class |

Shape is the first of these that applies: `insufficient` (fewer than 3 points), `spike` (a
transient excursion that returns), `flat` (no significant rise and `|rise|` within 3 noise
units), `declining`, `plateau` (rises in the first half, then levels off), `stair` (80% of the
rise in the largest third of the deltas), `steady`.

### The slope gate

A gauge column passes when all of these hold (current provisional thresholds):

- at least 8 post-warm-up points;
- Mann-Kendall `p_increasing` below 0.05;
- the lower end of the Sen interval above 0;
- the second half is not decaying: `halves.ratio` at least 0.5 (when the first half does not rise
  the ratio is undefined and the half passes if the second half rises);
- `rise` above 3 noise units.

A counter column passes under the every-delta rule, taken from CPython's refleak check: at least 8
points and every consecutive delta at least 1, so a single flat step fails it. The counter rule
needs no statistics, but it is strict: one point where the count does not grow, including after a
collection, fails the column.

Top-level `gate` of a `dla trend` evidence record is `pass` when at least one column of that
kind passed. Two evidence records are written, `growth-series` for gauge columns and
`unit-count-series` for counter columns; the authority is `test` for a harness or alloc-test
series and `live` for `dla sample`. The record's `passed` list holds, per column, `column`,
`region`, `site`, `finding`, `candidates`, `slope`, `lo`, `p`, `ratio`, `shape`, `unit`; it is cut
to the most important columns (those bound to a finding or site first) with a `truncated` count.
`dla sample` writes its own `growth-series` evidence without a gate; only `dla trend` sets one.

`dla trend S1 --against S2 --finding F3` compares a fresh-keys series with a repeated-keys series
and writes one `keys` discriminator. Growth in both modes favors `leak` (independent of keys).
Growth only with fresh keys that does not level off also favors `leak` (an unbounded key space).
Growth in neither favors `non-leak` (N-WARM). A plateau-shaped fresh column that is quiet under
repeated keys supports N-WARM and N-SCALE.

## Discriminators

A `discriminator` evidence record is the only kind that can move a finding to `refuted` or
confirm a non-leak. Only DLA tools write them (`dla alloc-test`, `dla trend --against`); there is
no command to add one by hand. Its `result` carries:

| Field | Meaning |
|---|---|
| `test` | `plateau`, `purge`, `gc`, `keys`, or a combination such as `plateau+purge` |
| `favors` | `leak`, `non-leak` or `inconclusive`. This is what the ledger reads |
| `supports` | for `non-leak`, the N-* ids the result supports (`N-WARM`, `N-HWM`, `N-ALLOC`, `N-HEADROOM`, `N-SCALE`); for `leak`, `["leak"]` |
| `finding` | the finding it is bound to, or null |
| `column`, `columns` | the tested columns, each with its own verdict and numbers |

`dla alloc-test H1 [--finding F3]` re-runs the harness with `iterations + extra` points (extra
defaults to the iteration count) and a purge applied at the last point, then tests each chosen
column. The column is `--column`, else the one tied to `--finding`, else every column that passed
the gate in the latest trend of the harness plus `os.rss`.

- **Plateau test.** Main window = the first `iterations` post-warm-up points, extra window = the
  rest. If the extra slope is at least half the main slope and the extra rise exceeds 3 noise
  units (counters: every-delta over the extra window), the growth continues and the test favors
  `leak`. If the extra rise is within 3 noise units (counters: no net increase) the growth
  stopped: favors `non-leak`, supports N-WARM and N-HWM. A cache that fills and stops, and a pool
  that reaches its high-water mark, end up here. Anything else is inconclusive.
- **Purge test** (process and off-heap columns, and gauges). `drop` is the value before the purge
  minus the value after; `growth` is the value before the purge minus the value at the end of
  warm-up. A drop of at least half the growth favors `non-leak` and supports N-ALLOC for a trim,
  free-os-memory, trim-native-heap or jemalloc purge, or N-HEADROOM for a gc purge. A drop of at
  most a fifth of the growth favors `leak`. In between is inconclusive. `--purge auto` picks the
  runtime's own purge (JVM `trim-native-heap`, Go `free-os-memory`, Python `trim` and `gc`, Node
  `gc`, Rust and native `trim`).
- **GC test**, when `.pre_gc` twins exist. A pre-GC series that passes the gate or shows a
  sawtooth while the post-GC series is flat favors `non-leak` and supports N-HEADROOM.
- **Overall verdict.** Bound to a finding it is that finding's column verdict. Unbound, it is
  `non-leak` only when every tested column favors non-leak, `leak` when any favors leak and none
  favors non-leak, else `inconclusive`. The same run also writes a new series (source
  `alloc-test`) and its trend.

An `inconclusive` discriminator moves nothing. Run again with a larger `--extra` or more
iterations (`dla harness run H1 --iterations N`), or record the item `unresolved`.

## Ledger rules for dynamic findings

A finding is leak-side when it is top-level or a sibling whose hypothesis is not an N-* id. It is
non-leak when it is a sibling whose hypothesis begins with `N-ALLOC`, `N-HEADROOM`, `N-WARM`,
`N-HWM`, `N-SCALE`, `N-INTENT` or `N-OBSERVER`. The family of a finding is its root plus all
siblings of the root.

| Move | What the ledger requires | Refusal code |
|---|---|---|
| `ledger test F3 --evidence E` | a `workload` evidence from `harness gen` or `harness run` among the cited or supporting evidence | `harness-required` |
| `ledger test F3` (leak-side) | at least one non-leak sibling in the family | `sibling-required` |
| `ledger confirm F3` (leak-side) | a `growth-series` or `unit-count-series` evidence of authority test, live or dump whose result has `gate: pass` | `growth-gate-failed` |
| | a cited passing growth column tied to the finding (probe, site, owner or `candidates`) | `growth-not-tied` |
| | the finding has a `site` and a `holder` | `site-required`, `holder-required` |
| | some non-leak member of the family is `refuted` | `nonleak-refutation-required` |
| `ledger confirm F3.n1` (non-leak) | a `discriminator` with `favors: non-leak`, and the leak-side root is `refuted` | `discriminator-required`, `discriminator-direction`, `leak-refutation-required` |
| `ledger refute F3` (leak-side) | a discriminator with `favors: non-leak` | `discriminator-direction` (none at all: `discriminator-required`) |
| `ledger refute F3.n1` (non-leak) | a discriminator with `favors: leak` | the same |
| `ledger annotate F3 --holder H` | sets holder, owner, region or site only when the finding has none; a different value is refused | `annotation-conflict` |
| `ledger annotate F3 --holder H --evidence E` (holder already set) | `E` is a heap-graph `growth-series` (dump or live) from `dla-hprof` or `dla-v8-snapshot`; one of its passing columns is tied to `F3` and has `holder_kind` equal to `H`. The record keeps `evidence` and `replaced_holder` | `annotation-conflict` (the message names the failed condition) |

A discriminator that names a finding counts only for findings in that finding's family. Refusals
name the missing item and the command that fixes it. A leak-side confirm needs a cited passing
growth column tied to the finding (its probe, a column at its site, a column naming its owner, or
one listing it under `candidates`); otherwise it is refused with `growth-not-tied`. On confirm the ledger fills `shape` and `region` from
the first passing column cited when the finding has none (a `process` region leaves `region`
unset).

From ledger schema 6 a confirmed finding can also carry a label, a stored explanation and a strong
mark. Each is a record of its own that replays from the ledger alone (a record carries its proof, so
old ledgers replay unchanged):

| Move | What the ledger requires | Refusal code |
|---|---|---|
| `ledger label F3 --as L-X` | F3 confirmed or strong; `L-*` on a leak-side finding, `N-*` on a non-leak sibling whose hypothesis names it; one `requires` alternative of the archetype met by evidence tied to F3; F3's holder among the archetype's holders; each look-alike the archetype disconfirms that exists as a sibling is refuted; an `L-*` ID has a row in the runtime's archetype reference (the runtime is the language of the finding's site, else its worker), or a shipped `known-leaks.toml` row names that archetype for the runtime (the Go ticker is L-REGISTER), unless `--note` says why none of the rows fits | `label-evidence-missing`, `label-polarity`, `label-hypothesis-mismatch`, `label-state`, `label-holder`, `label-disconfirm-open`, `label-runtime-row`, `unknown-archetype`, `archetypes-off` |
| `ledger label F3 --as UNCLASSIFIED --note ".."` | F3 confirmed or strong; a note; always allowed (also with archetypes off) | `empty-field` |
| `ledger duplicate F4 --of F1` | F4 and F1 are distinct leak-side top-level findings (no hypothesis), neither refuted, F4 not merged yet and F1 not a duplicate itself; both in one scope (the same worker or both core, and the same runtime by the language of the site when both sites have a known language) with the same canonical owner (qualified: a bare name matches nothing) or the same `file:line` site. F4's supports then count for F1 | `duplicate-state`, `duplicate-not-same` |
| `ledger strong F3 --evidence E` | F3 confirmed and leak-side; a passing growth record tied to F3; a second record that names F3's holder (retention) or removed the growth (intervention); the two records independent (tool, tool class and source differ); a holder kind the second record names equals F3's holder. A `histogram-diff` never counts | `strong-leak-only`, `strong-second-class-missing`, `strong-growth-unsigned`, `strong-not-independent`, `strong-holder-mismatch` |
| `ledger explain F3 ...` | the text names the site or owner, the holder, an adding site and how removal fails; `--adds` are recorded writers or the site; `--removal` agrees with the recorded removers; 40 to 600 characters | `explain-no-writer`, `explain-remover-recorded` and the other `explain-*` codes. On an artifact run (no source, so no writer can be known) the owner stands in for the site, `--adds` may be left out, `--removal` must be `not-found` and `--tag` stays `inferred` unless a retention path is cited |

The label of a finding is the latest `finding_labeled` record. `dla ledger confirm` also prints a
`warning` (not a refusal) when the passing column tied to the finding carries a `holder_kind` other
than the finding's holder; it names the observed holder and the `ledger annotate F --holder H
--evidence E` command that corrects it.

An `intervention` record (`dla intervene`) holds the control and treated series, the guards and an
`effect`: `growth-removed`, `growth-remains`, `inconclusive` or `invalid`. `growth-remains` and
`inconclusive` never refute a leak. `dla trend` writes a `signal` (tool, tool class, source hash) into
its growth evidence, which is how independence is judged.

## Residual

`dla gap S1 --residual` asks what growth nothing accounts for. Per region (`heap`, `off-heap`,
`non-heap`; whole-process growth has no region of its own and is not a residual row):

- `total` is the Theil-Sen slope of the region's primary column: the runtime heap after GC
  (`jvm.heap_used`, `go.heap_live`, `node.heap_used`, `py.traced`, `native.live_bytes`) for heap,
  `gap.unexplained` for off-heap (run `dla gap S1` first), thread or goroutine stack bytes for
  non-heap.
- `attributed` is the sum, over confirmed findings, of the slope of the passing column their
  growth evidence cites in that region (a column counted once), capped at `total`. A finding whose
  growth was measured as a count contributes no bytes and the limitation says so.
- `residual = max(0, total - attributed)`, against the provisional threshold
  `max(0.25 * total, 3 * noise / x-span)`.

Rows are written to `coverage.json` under `residual` (region, total, attributed, residual,
threshold, unit such as `bytes/n`, open item). A region above the threshold opens an open item
("unexplained growth: 2360 bytes/n in heap (S1)"); resolve it by adding a hypothesis
(`dla ledger add --origin residual --evidence E14 ...`) or by recording it unresolved. A later
`--residual` run that is below the threshold resolves the item it opened; the resolution is
attached to a finding (the residual-origin finding first, else the newest), not to the residual
evidence. `dla gap S1` (without `--residual`) splits process memory into `gap.accounted` (the
runtime's own accounting: NMT or committed memory for the JVM, `node.heap_total + node.external`,
`py.traced`, `go.total - go.released`, `native.live_bytes`) and `gap.unexplained`; when the
unexplained part passes the gate it names the `native` route and `dla alloc-test --purge trim` as
the next steps. Accounting of the runtime is partial: `py.traced` sees only Python allocations
made after tracemalloc started.

## How the report shows it

`dla report` is the only report writer. For dynamic findings it prints:

- confirmed: `Growth: <column> <slope> <unit>/<axis> (<shape>, gate pass, E12)`, for example
  `Growth: probe:reports.builder.ReportBuilder.category_totals 36.73 count/n (steady, gate pass, E12)`;
- refuted: `Ruled out by: <test> (<favors>) E13`, from the discriminator, for example
  `Ruled out by: plateau+purge (leak) E13` for a non-leak sibling refuted because the growth went
  on;
- a `Coverage:` line with iterations and exercised entry points per runtime
  (`Coverage: python 7040 iterations, 10 of 10 entry points exercised`), or "not measured" for
  runtimes whose entry points cannot be measured (Rust, native);
- open items, including residual items, with the command that closes each.

For labelled, explained and verified findings it also prints, per finding:

- `Label: L-COLLECT (evidence: E11 unit-count-series, E14 retention-path)`; a finding with no label shows
  its ID line as UNCLASSIFIED and, when a note was recorded, `Label: UNCLASSIFIED (<note>)`. UNCLASSIFIED has the
  same weight as any ID;
- `Explanation (inferred|observed): <text>` and an `Explanation structure:` line with the adding sites,
  the removal and the remover;
- `Strong: second class E20 retention-path that names the holder, independent of the growth record E11
  (dla-trend)` (or `intervention that removed the growth`);
- `Verified: agrees (verifier site .., holder .., label ..; bundle ..)`, or `Verified: DISAGREES` with
  what differs and both mechanisms, or `Verified: no` for a confirmed leak nobody verified.

## Integrity

`dla` seals each protected file it writes (`ledger*.jsonl`, `state.json`, `coverage.json`, `report.md`)
by storing its length and sha256 in `<run>/integrity/<file>.seal`, under the same lock as the write.
`dla gate check`, `dla report` and `dla run status` verify the seals. A mismatch is a tamper item: the
report still renders, with `Status: INCOMPLETE` and the item named, and `gate check` fails. The hooks
(`references/safety-and-effects.md`) only make an accidental edit less likely; the seals are the
guarantee for those five kinds of file. The seals do not cover the other files `dla` generates for the
gates to read (trend, series, gate, bundle, intervene, harness output, evidence files); only the write hook
refuses edits to them. A `dla` command killed between a write and its seal leaves a `.pending` marker
instead of a tamper: the next `dla` write of that file, or `dla ledger merge`, seals it again when the
difference is the interrupted write itself (the sealed bytes are still the file's start, or the file is
what the writer announced). If a protected file really was changed outside `dla` (a restored backup, a merge conflict you
resolved), disclose it and run `dla run reseal --reason "<what changed and why>"`: it appends an
`integrity_resealed` record to the ledger, seals the files again, and the report lists the reseal under
Integrity. A run created before ledger schema 6 has no `integrity/` directory and is reported as unsealed,
not as tampered. If `report.md` is edited after it was rendered, the next `dla report` replaces it and
says so.

Show the report exactly as printed. If it says INCOMPLETE, say so and list what is open.
