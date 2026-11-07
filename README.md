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

Pre-release, milestone M1 (benchmark harness on top of the M0 foundation). What exists today: the
data contracts and JSON Schemas, the ledger and its state machine, `dla run init`, `dla run status`,
`dla ledger add`, `dla ledger resolve`, `dla consent`, `dla report`, `dla schema`, the plugin
skeleton, the release pipeline and the benchmark in [bench/](bench/README.md). Measurement commands
(probing, sweeps, sampling, trend analysis) arrive in later milestones, so the investigation stages
in the skill are not yet backed by working commands. Expect breaking changes. Open questions and
unverified details are tracked in [docs/VERIFY.md](docs/VERIFY.md).

Commands:

| Command | What it does |
|---|---|
| `dla run init` | Creates a run under `.dla/` (flags include `--budget` and `--session-id`) |
| `dla run status` | Shows run id, stage, status, counts and the first blocking items |
| `dla ledger add` | Adds a finding, which always starts as `candidate` |
| `dla ledger resolve` | Dispositions a finding or open item, or answers a checklist item |
| `dla consent` | Records the run's one consent: `--tiers` (privilege tiers 1 to 4), `--raw-values`, `--ceiling`, `--given-by user\|runner`. A second consent in a run is refused |
| `dla report` | Renders the report, writes `report.md` and stores its hash in `state.json` |
| `dla schema` | Writes or checks the JSON Schema files |

Headless consent: an unattended runner can give consent through the environment instead of a
question to the user. If `DLA_CONSENT_TIERS` is set (`1-4`, or a list such as `1,2,3`) when
`dla run init` runs, the run starts with that consent, recorded as given by the runner;
`DLA_CONSENT_RAW_VALUES` set to a true value also allows raw values. An invalid
`DLA_CONSENT_TIERS` makes `dla run init` fail.

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
| Rust | 1.85 stable | Only for building from source (edition 2024) |

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
skills/           the detect-leak-anywhere skill
agents/           worker and verifier subagents
hooks/            SessionStart hook and downloader
bin/dla           shim that locates and runs the binary
crates/           Rust workspace (dla-core, dla-report, dla-cli, ...)
schemas/          generated JSON Schemas
docs/             blind spots and verification backlog
bench/            benchmark: fixtures with answer keys, the dla-bench runner and graders
                  (see bench/README.md)
tests/shim/       tests for the shim and downloader
```

## License

Dual-licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option
(`MIT OR Apache-2.0`).
