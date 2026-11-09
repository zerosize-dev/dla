# Benchmark fixtures

A fixture is a small program with a known answer. The benchmark runner copies
the fixture's `project/` into an empty working directory, hands the model the
task prompt, and grades what the model recorded against `expected.toml`.

```
bench/fixtures/<id>/
  project/        what the model sees; nothing else is copied (source code,
                  or heap dumps, or notes: see Entry paths)
  expected.toml   the answer key (never shown to the model)
  README.md       for humans: what the fixture tests and the measurements
                  that prove the expected answer (never shown to the model)
```

## Entry paths

`entry` in `expected.toml` says how the diagnosis starts. The default is
`repository`; the other two have no source tree, so the source-size rules and
the site checks below do not apply to them. `project/` is always present: the
runner copies it and the directory name stays the same for every entry path.

| `entry` | `project/` holds | The prompt | Checks instead of the source rules |
|---|---|---|---|
| `repository` (default) | the source code and its tests | the operator's report | all the rules below |
| `artifact` | `dumps/*.hprof.gz`, a series of heap dumps of one process in capture order, and no source | the report plus the sentence that heap dumps of the process are in `dumps/` and no source is available | each dump is a gzip file (starts with `1f 8b`) that inflates to a file starting with `JAVA PROFILE 1.0.`; the header timestamps of the dumps are distinct; at least one dump; no source file |
| `symptom` | only non-source notes (for example a `NOTES.txt` of the operator's remarks) | a report that does not name the target | no source file |

An artifact fixture names its leak by `owner` instead of `site` (below). The
dumps are gzip-wrapped because a raw dump is 5 to 8 MB; keep a series to
three or four dumps. Class and field names inside a dump follow the same "No
hints" rule as source. A symptom fixture is graded like an `impossible` one: a
correct run ends with no claim and says what it is missing.

Static (`dla-bench static`) and dynamic (`dla-bench dynamic`) skip fixtures
whose entry is not `repository` and report them as skipped with the reason.

## Runtimes

`runtime` is the primary runtime of a fixture. A multi-runtime fixture (a
Kotlin front end calling a Rust library through JNI, say) also lists
`runtimes = ["jvm", "rust"]`: at least two, no duplicates, containing `runtime`,
and `project/` must hold source files for each (`.java`/`.kt` for `jvm`, `.rs`
for `rust`, and so on). `dla-bench static` runs every listed runtime and fails
a fixture whose probe does not detect one of them. A `[[leak]]` or
`[[non_leak]]` may carry its own `runtime`, one of the fixture's runtimes; it
is for readers and reports and is not used for grading.

## Rules for `project/`

- Original code written for DLA. No third-party source.
- Builds and runs offline with the toolchain alone: JDK (`javac`/`java`, no
  Maven or Gradle dependencies unless the fixture is meant to fail to build),
  Go standard library, Rust standard library (`Cargo.toml` with no
  dependencies and an empty `[workspace]` table), Node.js built-ins and
  `node --test`, Python standard library and `unittest`, C with `make` and
  the system compiler.
- A workload the model can find and run: a `main` (or `load` script) that
  drives the code through many requests or iterations and accepts an
  iteration count, plus unit tests. One full workload run finishes in under
  30 seconds on a laptop.
- One prompt shape for every category: an operator's report that memory
  (resident memory, heap, threads or handles) grows or is higher than
  expected, ending by asking what is holding the memory. A prompt never asks
  "whether" something leaks and never hints at the category or mechanism.
- No hints. No comment, identifier, log line, commit or file name may say or
  imply "leak", "never removed", "bounded", "TODO", "fix" or anything else
  that gives the answer away. Name things after the domain (orders, sessions,
  subscriptions), not after the bug.
- Realistic shape (`repository` entry): 3 to 10 source files, 150 to 600
  lines in total, with ordinary code around the relevant site so the site is
  not the only thing in the project. `.kt` files count as source.
- Kotlin is allowed. Its `build.sh` (at the root of `project/`) compiles with
  `kotlinc` when it is on `PATH` and otherwise with the embeddable compiler
  from the offline Maven cache (`~/.m2`): `java -cp <compiler jars>
  org.jetbrains.kotlin.cli.jvm.K2JVMCompiler -no-stdlib -cp <stdlib jar> -d out
  *.kt`. The validator requires a `build.sh` that mentions `kotlinc` next to any
  `.kt` file. A fixture that mixes runtimes keeps one build entry point.
- No build outputs committed (`target/`, `*.class`, `__pycache__/`,
  `node_modules/`, binaries).

## `expected.toml`

```toml
schema_version = 1
id = "go-subscription-workers"     # same as the directory name
runtime = "go"                     # jvm | go | rust | node | python | native
# runtimes = ["jvm", "rust"]       # optional: a multi-runtime fixture (see Runtimes)
# entry = "artifact"               # optional: repository (default) | artifact | symptom
category = "archetype-leak"        # see Categories
pair = "go-subscription-workers-closed"   # trap/control partner, or omitted
dynamic_confirmation = true        # false when no harness can run here

[task]
prompt = """
The notification service's memory and thread count climb steadily under
sustained traffic until the pod is OOM-killed after about a day. Find out why.
"""

[[leak]]
id = "L1"
site = "internal/notify/subscriber.go:57"   # project-relative path:line (optional with owner)
# owner = "notify.Hub.subscribers"          # package.Class.field; required without a site
# runtime = "go"                            # optional, multi-runtime fixtures
lines = [55, 61]                            # inclusive range that counts as the site
alt_sites = ["internal/notify/hub.go:22-24"] # other places a correct answer may name
holder = "blocked-stack"
region = "non-heap"
cause = "L-WORKER"                          # archetype ID, or "UNCLASSIFIED"
share = 1.0                                 # share of total growth (multi-leak)
confirmable = true                          # default true; false: only static evidence is obtainable
dump_backed = false                         # default false; true: see below

[[non_leak]]
id = "N1"
site = "internal/notify/cache.go:30"        # optional; owner is the alternative
cause = "N-WARM"
discriminator = "entry count stops at 1000 after warm-up; slope of the second half is zero"
```

Field rules:

- `holder` is one of `static/global`, `registration-table`, `container-field`,
  `thread-local`, `blocked-stack`, `closure/continuation`,
  `native-handle/global-ref`, `cycle-only`.
- `region` is one of `heap`, `off-heap`, `non-heap`.
- `cause` is one of the ten `L-*` IDs, the seven `N-*` IDs or `UNCLASSIFIED`.
- A `[[leak]]` has a `site` or an `owner`, or both. A leak without a `site`
  is matched by `owner` (see `GRADING.md`): the owning field or container as
  `package.Class.field`, with no spaces, slashes or colons; a finding's owner
  matches when it is equal or ends with `.` and this value. `lines` and
  `alt_sites` need a `site`. Artifact fixtures use `owner` because they have no
  source to point at; the leak's `holder`, `region` and `cause` stay as for any
  leak.
- `site` must exist: the file is in `project/` and the line is inside the file.
  (A `site` of an `artifact` or `symptom` fixture is not located.)
  `lines` defaults to `[line, line]` and must contain `line`. Point `site` at
  the line that retains the memory or allocates it without release, and keep
  the range to that statement.
- `dump_backed` (optional, default false) marks a leak whose confirmation must
  rest on a heap dump or an allocation profile: the chosen confirmed finding
  cites evidence of kind `retention-path`, `histogram-diff` or `alloc-stack`
  with authority `dump` or `live`. Only `dla-bench dynamic` checks it (and only
  when its dump check is switched on); both graders accept the field and ignore
  it, and the grading metrics do not change.
- `alt_sites` (optional) lists other locations a correct diagnosis may cite
  for the same leak, as `path:line` or `path:start-end`; typically the holder
  declaration when `site` is the line that adds to it, or the reverse. Each
  must exist like `site`. Do not list unrelated lines.
- `share` values of one fixture sum to 1.0 (within 0.01) when there is more
  than one leak.
- `pair` is reciprocal: the partner names this fixture back, and both use the
  identical `task.prompt`.
- A fixture of category `archetype-leak`, `archetype-free` or `multi-leak` has
  at least one `[[leak]]`; `decoy-control` and `non-leak` have none and at
  least one `[[non_leak]]`; `impossible` has `dynamic_confirmation = false`
  and every leak `confirmable = false`.

## Categories

| Category | What it is | A correct run ends with |
|---|---|---|
| `archetype-leak` | A leak that matches one taxonomy row; the trap half of a pair | The leak confirmed at the right site and holder |
| `decoy-control` | One minimal edit away from its trap; memory is bounded | No confirmed leak; the growth ruled out with its discriminator |
| `archetype-free` | A real leak whose mechanism fits no taxonomy row | The leak confirmed and labelled `UNCLASSIFIED` |
| `non-leak` | Growth that is not a leak (`N-*`) | No confirmed leak; the growth ruled out with its discriminator |
| `impossible` | The leak pattern is in the code, but nothing can run (no build, missing tool) | `suspected_static_only` or `unresolved` with a reason; never confirmed |
| `multi-leak` | Two independent leaks in different subsystems or runtimes | Both confirmed |

The category says what a correct run ends with whatever the entry path: an
artifact fixture can be an `archetype-leak` (the series grows), a
`decoy-control` (the series plateaus) or `impossible` (a single dump: at most
suspected, never claimed). Besides the other metrics of `GRADING.md`,
`source_overclaim_rate` counts the artifact-entry runs that claim a leak at a
source site although no source was given.

## The first twenty

| ID | Runtime | Category | Pair | Mechanism (for authors; never in the code) |
|---|---|---|---|---|
| `jvm-order-cache` | jvm | archetype-leak | `jvm-order-cache-bounded` | Static map keyed by request-derived order IDs, entries added and never removed (`L-COLLECT`, holder `static/global`) |
| `jvm-order-cache-bounded` | jvm | decoy-control | `jvm-order-cache` | Same code with an access-ordered `LinkedHashMap` evicting past a fixed size (`N-WARM`) |
| `jvm-audit-chain` | jvm | archetype-free | | Each request object keeps a field pointing at the previous request object, and a static field holds the newest, so the whole chain stays reachable (`UNCLASSIFIED`, holder `static/global`) |
| `jvm-billing-private-repo` | jvm | impossible | | Maven project whose only dependency comes from an unreachable private repository; a static map grows per request in code that cannot be built here |
| `go-subscription-workers` | go | archetype-leak | `go-subscription-workers-closed` | A goroutine per subscription waits in `select` for a stop signal that callers never send (`L-WORKER`, holder `blocked-stack`) |
| `go-subscription-workers-closed` | go | decoy-control | `go-subscription-workers` | Same code where the handler closes the subscription on return |
| `go-buffer-pool` | go | non-leak | | A buffer pool and a map grow to the peak working set and then stay flat (`N-HWM`) |
| `python-report-memo` | python | archetype-leak | `python-report-memo-bounded` | `functools.lru_cache(maxsize=None)` on a method of per-request objects keeps every instance (`L-COLLECT`, holder `static/global`) |
| `python-report-memo-bounded` | python | decoy-control | `python-report-memo` | `lru_cache(maxsize=128)` on a method of one long-lived object |
| `python-graph-churn` | python | non-leak | | Reference cycles per request with raised GC thresholds; memory climbs and falls back after collection (`N-HEADROOM`) |
| `python-ingest-two-leaks` | python | multi-leak | | A module-level registry of handlers appended on every batch (about 70% of growth, `L-REGISTER`) plus, in another module, a parser that keeps each parsed record through a `previous` field chained from a module global (about 30%, `UNCLASSIFIED`) |
| `node-socket-heartbeat` | node | archetype-leak | `node-socket-heartbeat-cleared` | `setInterval` per connection that is never cleared (`L-REGISTER`, holder `registration-table`) |
| `node-socket-heartbeat-cleared` | node | decoy-control | `node-socket-heartbeat` | Same code with `clearInterval` on close |
| `node-retry-scheduler` | node | archetype-free | | Every attempt object keeps a field pointing at the attempt scheduled before it, and the module's singleton scheduler keeps the newest attempt, so the whole attempt history stays reachable through the objects' own fields; no collection, timer, listener or captured scope grows (`UNCLASSIFIED`, holder `static/global`) |
| `rust-tree-parent-links` | rust | archetype-leak | `rust-tree-parent-links-weak` | Parent and child hold each other through `Rc`, one tree per request (`L-CYCLE`, holder `cycle-only`) |
| `rust-tree-parent-links-weak` | rust | decoy-control | `rust-tree-parent-links` | Same code with a `Weak` back-edge |
| `rust-batch-buffers` | rust | non-leak | | `Vec` and `HashMap` capacity retained after `clear()`; grows to the largest batch and stays flat (`N-HWM`) |
| `native-frame-decoder` | native | archetype-leak | | A decode buffer is `malloc`ed and not freed on an early-return error path hit by a fixed share of inputs (`L-HANDLE`, holder `native-handle/global-ref`, region `off-heap`) |
| `native-glyph-cache` | native | non-leak | | A fixed-capacity cache fills and then evicts; memory rises to a plateau (`N-WARM`) |
| `native-vendor-codec` | native | impossible | | Needs a vendor header and library that are not installed. Frames the library hands to a callback are kept by the application and never handed back with the library's release call; whether that retains memory depends on the library's ownership contract, which only the missing library and its manual define, so no stand-in can settle it |

Six runtimes with three or four fixtures each; six archetype leaks, five
decoy controls, two archetype-free leaks plus one inside the multi-leak case,
four non-leaks and two impossible cases.

## The next five (M4)

| ID | Entry | Runtimes | Category | What it tests |
|---|---|---|---|---|
| `jni-order-codec` | repository | jvm, rust | multi-leak | A Kotlin heap leak and a Rust native leak reached through JNI; both runtimes must be diagnosed and merged into one report |
| `jvm-rfq-dump-series` | artifact | jvm | archetype-leak | Three dumps of one process in which one holder's retained size and count grow |
| `jvm-rfq-dump-series-capped` | artifact | jvm | decoy-control | Three dumps of the same service where the holder plateaus after a warm-up fill |
| `jvm-rfq-single-dump` | artifact | jvm | impossible | One dump: it can end suspected with a request for another dump, never confirmed |
| `symptom-no-target` | symptom | jvm | impossible | A report that names no target and a project that holds only notes: a correct run claims nothing |

## README.md for each fixture

State the category, the mechanism and the minimal edit that distinguishes a
trap from its control, then paste the measurements that prove the answer:
the workload command, two or more iteration counts, and the numbers
(object counts, RSS, entry counts, goroutine counts) showing growth for a
leak and a plateau for a bounded case. For an impossible case, paste the
failing build command and its error.
