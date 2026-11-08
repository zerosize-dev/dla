# DLA: Detect Leak Anywhere

DLA is a Claude Code plugin that diagnoses memory leaks in JVM, Go, Rust, Node.js/TypeScript,
Python and native code. It is built around one idea: a diagnosis must come with an explanation
that rests on recorded evidence, and a diagnosis the evidence does not support must say so.

- **Diagnosis with explanation.** Every finding cites the measurements that support it, names the
  holder that keeps the memory alive, and states whether each claim was observed or inferred.
  Anything DLA could not establish is reported as unresolved, with the reason and the step that
  would resolve it. It is never silently dropped.
- **Never edits project source.** DLA reads your code and writes only under `.dla/` in the project.
  It can describe the repair class a leak needs; it does not apply one.
- **Local only.** DLA works with code, builds and artifacts on your machine. It never connects to
  deployed environments, and the `dla` binary contains no network code.

A small Rust binary, `dla`, holds the rules: the append-only evidence ledger, the finding state
machine, the report renderer and the checks that refuse an unsupported conclusion. The plugin's
skill and agents drive the investigation; the binary decides what may be recorded.

For the limits of what a run can establish, read [What DLA cannot see](docs/what-dla-cannot-see.md).

## Status

Pre-release, milestone M3 (the dynamic core, on top of the M2 static sweep, the M1 benchmark and
the M0 foundation). What exists today: the data contracts and JSON Schemas, the ledger and its
state machine with the M3 rules (a finding is confirmed only from a passing growth series and a
refuted non-leak explanation, and refuted only by a discriminating test), the probe, the static
sweep, the DLA-owned rules, the holder census, workload discovery, analyzer installation, the loop
harness for JVM, Go, Rust, Node.js, Python and native code, process sampling, trend statistics,
the memory-gap split, the allocator tests, the plugin skeleton, the release pipeline and the
benchmark in [bench/](bench/README.md). The evidence-gate commands (`dla gate check`), attribution
(`dla summarize`), `strong` findings, the verifier bundle and worker merging arrive in later
milestones. Expect breaking changes. The statistical thresholds are provisional until they are
calibrated on new fixtures (M7). Open questions and unverified details are tracked in
[docs/VERIFY.md](docs/VERIFY.md).

Commands:

| Command | What it does |
|---|---|
| `dla run init` | Creates a run under `.dla/` (flags include `--budget` and `--session-id`) |
| `dla run status` | Shows run id, stage, status, counts and the first blocking items |
| `dla probe [--build]` | Records runtimes, versions, tools on the PATH, privilege facts, test runners and library flags; `--build` also runs each runtime's build and needs tier-1 consent |
| `dla install <backend>` | Installs one static analyzer (see below); on failure prints the exact command and leaves the backend `missing` in coverage |
| `dla sweep --lang <l>` | Runs the available static backends for one runtime and adds each rule hit as a candidate |
| `dla census --lang <l>` | Adds every process-lifetime holder of one runtime as a candidate, with heuristic writers and removers |
| `dla workload discover` | Lists the tests, entry points and dev servers a loop can run, ranked by entry points; unrunnable ones are recorded with a reason |
| `dla harness gen --lang <l>` | Copies the project into a scratch worktree under the run directory and writes a loop harness for a workload (command mode: the workload's own command at growing iteration counts; `--driver`: a loop test with hooks to fill in). Runs nothing and needs no consent |
| `dla harness run <H>` | Builds the harness once, measures every point with a per-runtime agent, shim or driver, and writes one series per key mode (`--mode fresh\|repeated\|both`). Tier 1 |
| `dla trend <series>` | Mann-Kendall, Theil-Sen and half-split statistics, the slope gate (counters: every delta at least 1) and a shape for every column; `--against` compares fresh and repeated keys |
| `dla sample --pid <P>` | Samples a running process (memory, threads, descriptors, cgroup figures and runtime counters) into a time series. Tier 1 for a process DLA started, tier 2 for any other |
| `dla gap [<series>]` | Splits process memory into runtime-accounted and unexplained; with `--residual`, totals each region's growth, subtracts what confirmed findings explain and opens an item for the rest |
| `dla alloc-test <H>` | Re-runs a harness with extra points and a purge (trim, free-os-memory, gc, ...) and records whether the result favors a leak or a non-leak explanation. Tier 1 |
| `dla ledger add` | Adds a finding, which always starts as `candidate` |
| `dla ledger annotate` | Records the writers and removers the model found for a holder (`--writer file:line`, `--remover file:line`, `--note`) and the holder, owner, region or site when the finding has none; changes no state |
| `dla ledger test\|confirm\|refute` | Moves a finding under test, confirms it, or refutes it, each citing evidence (`--evidence E1,E2`); the ledger refuses the move unless its rules are met |
| `dla ledger resolve` | Dispositions a finding or open item, or answers a checklist item |
| `dla consent` | Records the run's one consent: `--tiers` (privilege tiers 1 to 4), `--raw-values`, `--ceiling`, `--given-by user\|runner`. A second consent in a run is refused |
| `dla report` | Renders the report, writes `report.md` and stores its hash in `state.json` |
| `dla schema` | Writes or checks the JSON Schema files |

### Dynamic measurement

The dynamic commands run project code, so they sit behind the privilege tiers. One consent per run
covers a set of tiers, and tiers escalate by intrusiveness:

| Tier | Covers | Used by |
|---|---|---|
| 1 | Build and run project code in a scratch copy under the run directory (builds, tests, harness loops, preload agents, allocator shims, in-process probes), and install analyzers | `dla probe --build`, `dla install`, `dla harness run`, `dla alloc-test`, `dla sample` of a process DLA started |
| 2 | Observe a running local process DLA did not start (`/proc` sampling, jcmd attach, Node inspector, Python remote exec, a Go pprof endpoint) | `dla sample --pid` of another process |
| 3 | Debugger or kernel level (ptrace attach, eBPF, perf, valgrind) | later adapters |
| 4 | Capture memory contents (heap dumps, core dumps); outputs may hold secrets | later adapters |

Seeing raw values in summaries is a separate consent (`--raw-values`). The details, including the
effects statement `dla sample --pid` prints before it starts, are in
[safety-and-effects.md](skills/detect-leak-anywhere/references/safety-and-effects.md).

`dla harness gen` never edits project source: it writes a scratch copy of the project and the
harness (manifest, probes, agent files, generated driver) under `.dla/<run>/harness/H<n>/`, and
`dla harness run` writes the logs, raw agent output and series there and under
`.dla/<run>/series/` and `trend/`. The scratch copy can be large; delete the run directory when you
are done. The harness templates for each runtime are in [harness/](harness/README.md); the columns,
the slope gate and the ledger rules are in
[evidence-and-report.md](skills/detect-leak-anywhere/references/evidence-and-report.md). A worked
example on a small Python project:

```
dla run init && dla probe && dla sweep --lang python && dla census --lang python
dla workload discover
dla harness gen --lang python            # H1, command mode
dla harness run H1                       # S1
dla trend S1                             # passed columns and their candidates
dla ledger add --sibling-of F4 --origin hypothesis --hypothesis N-WARM \
    --refute-if "growth continues over the extra cycles" --evidence E12
dla ledger test F4 --evidence E10        # the harness run evidence
dla alloc-test H1 --finding F4           # plateau and purge tests: favors leak
dla ledger refute F4.n1 --evidence E13   # the non-leak sibling loses
dla ledger annotate F4 --holder static/global
dla ledger confirm F4 --evidence E12     # the trend evidence that passed the gate
```

### Static analyzers

`dla sweep` drives these backends, listed per runtime in `data/detectors/<lang>.toml`. Each rule is
marked verified or not; unverified ones are tracked in [docs/VERIFY.md](docs/VERIFY.md). The `dla`
backend is built into the binary: Rust rules over tree-sitter syntax trees, with no external tool.

| Runtime | Backends |
|---|---|
| JVM | Error Prone, SpotBugs, detekt, CodeQL, Sonar |
| Go | staticcheck, golangci-lint, go vet |
| Rust | RAPx (`cargo +nightly rapx check -m`) |
| Node.js/TypeScript | ESLint with rxjs-x and typescript-eslint |
| Python | ruff, pylint, CodeQL |

`dla install <backend>` has one recipe per backend. Self-contained analyzers go into the plugin data
directory (`<data>/tools`), language tools go through the project's own package manager, and nothing
is ever written into project source. Run it for the backends `dla probe` reports as missing, after
tier-1 consent. GitHub release downloads and Docker are not assumed: a recipe that cannot run in
your environment prints the exact manual command and the backend stays `missing` in
`coverage.json`, with the install hint.

CodeQL and Sonar notes: CodeQL needs its query bundle, a large download that must be extracted
locally; when that is not possible the backend stays a recorded gap. Sonar needs a running server,
so its rules (`S5164`, `S2095`) are listed but unverified, and the backend is `missing` unless you
provide one.

Headless consent: an unattended runner can give consent through the environment instead of a
question to the user. If `DLA_CONSENT_TIERS` is set (`1-4`, or a list such as `1,2,3`) when
`dla run init` runs, the run starts with that consent, recorded as given by the runner;
`DLA_CONSENT_RAW_VALUES` set to a true value also allows raw values. An invalid
`DLA_CONSENT_TIERS` makes `dla run init` fail.

## How a diagnosis runs

The skill first picks an entry path from the request: a repository, a running local process,
captured heap dumps, or only a symptom (it asks once for the target and, when nobody can answer,
records the target as unresolved and stops rather than guessing). `dla run init --entry
repository|process|artifact|symptom` starts the run. When the probe finds two or more runtimes,
`dla run dispatch --workers jvm,rust` gives each runtime a share of the time budget and the skill
starts one `dla:worker` subagent per runtime in one turn; each worker writes its own ledger slice
(`dla --worker <name> ...`). The core waits, runs `dla ledger merge` (duplicates, contradictions
and unfinished workers become open items, so a killed worker leaves the run INCOMPLETE), then has a
blind `dla:verifier` subagent re-derive site, holder and mechanism from `dla bundle <finding>` for
each confirmed finding, and renders `dla report`. With one runtime the core does everything itself.
A heap-dump-only run needs a series of at least three dumps of one process to confirm anything,
and never claims a source line. The stage detail, failure recovery and per-runtime notes are in
[skills/detect-leak-anywhere/references/](skills/detect-leak-anywhere/references/).

## Install

From the Claude Code prompt:

```
/plugin marketplace add zerosize-dev/dla
/plugin install dla@zerosize
```

Or from a shell:

```
claude plugin marketplace add zerosize-dev/dla
claude plugin install dla@zerosize
```

Then start a session and run `/dla:detect-leak-anywhere`, or describe a memory problem and let
the skill trigger.

The plugin has three optional settings, asked for at install time or set later with
`/plugin configure`: value redaction (`shapes` by default), the highest privilege tier DLA may
offer, and the default time budget for a run (60 minutes). Claude Code does not export these to the
Bash tool, so the skill passes them to `dla` as arguments (for example `--budget`). The binary has
a default only for the time budget: 60 minutes, overridable with `--budget` or the
`DLA_DEFAULT_BUDGET` environment variable. The other two settings reach `dla` only through the
skill.

On Windows, install Git for Windows: the plugin's hooks run in Git Bash, and so does the `bin/dla`
shim.

### Local development

```
cargo build -p dla-cli
claude --plugin-dir .
```

With `--plugin-dir`, the `bin/dla` shim falls back to `target/release/dla` and then
`target/debug/dla` when no downloaded binary is present, so a local build is picked up with no
further setup. `claude plugin validate . --strict` checks the manifests, skill, agents and hooks.

## How the binary is fetched

The plugin ships a `SessionStart` hook (`hooks/fetch-dla.sh`). On startup and resume it checks
whether `dla-<version>-<target>` exists in the plugin data directory (`${CLAUDE_PLUGIN_DATA}/bin`).
If it does not, the hook downloads the archive for your platform from the GitHub release for this
plugin version, verifies its SHA-256 against the published checksum, and installs it. This happens
once per plugin version. On any failure the hook leaves the session running and prints the exact
path where the binary belongs.

Supported targets:

| Target | Platform |
|---|---|
| `aarch64-apple-darwin` | macOS on Apple silicon |
| `x86_64-apple-darwin` | macOS on Intel |
| `x86_64-unknown-linux-musl` | Linux x86-64 (static) |
| `aarch64-unknown-linux-musl` | Linux ARM64 (static) |
| `x86_64-pc-windows-msvc` | Windows (hooks run in Git Bash) |

Each release `v<version>` publishes `dla-<target>.tar.gz` (the binary `dla`, or `dla.exe`, at the
archive root), `dla-<target>.tar.gz.sha256` and `SHA256SUMS` at
`https://github.com/zerosize-dev/dla/releases/download/v<version>/`.

### Placing the binary by hand

If you are offline, behind a proxy, or on an unsupported platform:

1. Download or build the binary (see [Building from source](#building-from-source)).
2. Copy it to `<plugin data dir>/bin/dla-<version>-<target>` (add `.exe` on Windows) and make it
   executable. The default data directory is `~/.claude/plugins/data/dla-zerosize/` for the
   marketplace install; the hook's failure message prints the exact path.
3. Alternatively, set `DLA_BIN` to the full path of any `dla` binary. It takes precedence over
   everything else.

The shim (`bin/dla`) looks for the binary in this order: `$DLA_BIN`, the installed binary in the
data directory, then `target/release/dla` and `target/debug/dla` inside the plugin directory. If
none exists it prints where to put one and exits with status 127.

For offline mirrors and tests, `DLA_RELEASE_BASE_URL` replaces the download base. It accepts an
https URL, a `file://` URL or a plain directory path.

## Minimum versions

| Component | Minimum | Why |
|---|---|---|
| Claude Code | 2.1.94 | Plugin `bin/` on the `PATH` (2.1.91), plugin options (2.1.83), `CLAUDE_PLUGIN_DATA` (2.1.78), and skill hooks in plugins actually running (2.1.94). Use 2.1.143 or later for the documented Stop-hook continuation cap |
| Node.js | 20 LTS (18 works) | Needed only for memlab-based Node heap analysis. memlab's browser driver requires Node 18; 20 or 22 is recommended |
| JDK | 21 | Needed only for Eclipse MAT heap analysis on JVM targets (MAT 1.17 requires Java 21) |
| Rust | 1.89 stable | Only for building from source (edition 2024) |

Node and a JDK are not needed to install or run the plugin. They are needed when a run uses the
corresponding analysis tool, and DLA reports the tool as missing, with an install hint, when it
cannot find it.

## Runs, `.dla/` and `.gitignore`

Each diagnosis is a run. `dla run init` creates `.dla/<run-id>/` in the project, containing:

- `state.json`: schema version, stage, status, budget, tracked processes, the consent once given and, once a report exists, its hash;
- `ledger.jsonl`: the append-only evidence ledger, one JSON record per line;
- `report.md`: written by `dla report`, which is the only report writer.

It also writes `.dla/current` pointing at the active run. `dla` appends `.dla/` to the project's
`.gitignore` the first time, creating the file if needed and never adding a duplicate (`.dla`,
`.dla/`, `/.dla` and `/.dla/` all count as already ignored). Run data can contain paths and
excerpts from your project, so keep it out of version control unless you mean to share it. The
files carry a `schema_version`; `dla` refuses a run written by a newer schema and says plainly when
it cannot migrate an older one.

## Building from source

```
cargo build --release -p dla-cli      # produces target/release/dla
cargo test --workspace
cargo run -p dla-cli -- schema --check   # JSON Schemas under schemas/ are up to date
```

CI runs `cargo fmt --check`, `cargo clippy -D warnings` and the tests on Linux, macOS and Windows,
validates the plugin with `claude plugin validate . --strict`, and exercises the shim and
downloader on all three systems (`tests/shim/run.sh`).

## Releasing

Tag `v<version>` where the version matches the workspace version in `Cargo.toml`, `plugin.json`
and `marketplace.json`; the release workflow refuses to run if they disagree. It builds the five
targets, packages them with checksums and publishes a GitHub release.

The release workflow can also be started by hand (`workflow_dispatch`). That is a dry run: it
builds and verifies the five targets but publishes nothing.

## Layout

```
.claude-plugin/   plugin and marketplace manifests
skills/           the detect-leak-anywhere skill and its references
agents/           worker and verifier subagents
hooks/            SessionStart hook and downloader
bin/dla           shim that locates and runs the binary
crates/           Rust workspace (dla-core, dla-report, dla-cli, ...)
schemas/          generated JSON Schemas
docs/             blind spots and verification backlog
harness/          templates for the per-runtime agents, shims and loop drivers
bench/            benchmark: fixtures with answer keys, the dla-bench runner and graders
                  (see bench/README.md)
tests/shim/       tests for the shim and downloader
```

## License

Dual-licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option
(`MIT OR Apache-2.0`).
