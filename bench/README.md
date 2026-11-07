# DLA benchmark

The benchmark measures DLA against fixtures with known answers. A fixture is
a small program with a leak (or a deliberate non-leak) and an answer key. The
runner hands the fixture to a headless Claude Code session, collects what the
session recorded, and the graders turn that into metrics.

The contracts are `FIXTURES.md` (fixture format) and `GRADING.md` (how runs
are graded). This file explains how to operate the tools.

```
bench/
  FIXTURES.md  GRADING.md        contracts
  fixtures/<id>/                 project/, expected.toml, README.md
  schemas/                       JSON Schemas (generated, committed)
  ablations/prose/SKILL.md       skill text for the "prose" ablation
  runner/                        dla-bench (Rust) and grade.py (Python)
  results/<name>/                one benchmark run (see below)
```

All commands below run from the repository root. `dla-bench` is
`cargo run -p dla-bench --` (add `--release` for speed if you like).

## Validate the fixtures

```
cargo run -q -p dla-bench -- validate [--fixtures bench/fixtures] [--schemas bench/schemas]
```

For every directory under `--fixtures` it parses `expected.toml` into a typed
structure (unknown fields are errors), validates it against
`bench/schemas/expected.schema.json`, applies every rule in `FIXTURES.md`
(id equals the directory name, each `site` and `alt_sites` entry exists in
`project/` with the line inside the file, `lines` contains the site line,
shares sum to 1 for several leaks, pairs are reciprocal with identical
prompts, category rules, no committed build outputs, 3 to 10 source files and
150 to 600 source lines) and scans `project/` for hint words
(`leak*`, `todo`, `fixme`, `fix`, `bug`, `unbounded`, `bounded`,
`never freed|removed|cleared|closed|stopped|released`) with word boundaries,
case-insensitively. File names are scanned too, and `_` counts as a separator,
so `order_leak_count` is caught. It prints one `fixture: problem` line per
problem and exits 1 if there is any. An empty or missing fixtures directory is
valid.

Regenerate and check the schemas with `dla-bench schema` and
`dla-bench schema --check` (CI runs the check).

## Run a baseline

```
cargo run -q --release -p dla-bench -- run --ablation no-skill --runs 3 \
    --out bench/results/baseline-no-skill
```

Per (fixture, run index) the runner:

1. copies the files of `bench/fixtures/<id>/project` that git tracks, plus
   untracked files that are not ignored (every file when the directory is not
   in a git repository), into
   `<work-root>/<results name>/<ablation>/archetypes-<x>/<fixture>/<n>/project`,
   runs `git init` and commits it as `bench <bench@example.invalid>`;
   `expected.toml` and `README.md` are never copied. `--work-root` defaults to
   `$TMPDIR/dla-bench` and must be outside the repository;
2. gives the run a fresh `CLAUDE_CONFIG_DIR` inside that directory;
3. builds the ablation (below), then runs
   `claude -p <prompt> --output-format stream-json --verbose --max-turns T
   --permission-mode bypassPermissions --strict-mcp-config
   --no-session-persistence [--model M] [--plugin-dir <variant>]` with the
   copied project as working directory and stdin closed;
4. kills the whole process group after `--timeout` seconds and records
   `timed_out`;
5. collects the results (below). A run directory that already holds
   `run.json` is skipped, so an interrupted baseline continues where it
   stopped; `--runs 5` after `--runs 3` adds runs 4 and 5.

Scheduling is run-major (run 1 of every fixture, then run 2, ...) and
interleaved across runtimes so heavy builds do not pile up. `--parallel P`
(default 2) runs P sessions at a time. `--budget-usd X` stops starting new
runs once the summed cost of finished runs reaches X; runs in flight finish.
One progress line is printed per finished run: fixture, run index, exit code,
minutes, cost and cumulative cost. The working copy of a run that ended
cleanly is removed; keep it with `--keep-work`, failed runs are always kept.

Other flags: `--fixtures a,b|all`, `--fixtures-dir`, `--model`, `--max-turns`
(default 80), `--timeout` (default 1500), `--repo-root` (where the plugin
sources are), `--ablations-dir`, `--dla-bin`, `--claude-bin`.

### Ablations

| Ablation | What the session gets | Prompt |
|---|---|---|
| `no-skill` | no plugin | task + the `findings.json` instructions |
| `prose` | the skill text of `ablations/prose/SKILL.md`, `references/`, no binary, hooks or agents | `/dla:detect-leak-anywhere ` + task + the `findings.json` instructions |
| `gates` | the plugin without `hooks/`, without `agents/verifier.md`, with any `hooks:` block removed from the SKILL.md frontmatter; `DLA_BIN` points at `target/release/dla` | `/dla:detect-leak-anywhere ` + task + an unattended-run note |
| `full` | the plugin as it is | same as `gates` |

`gates` and `full` build `dla` once with `cargo build --release -p dla-cli`
(or use `--dla-bin`). `--archetypes off` removes
`skills/*/references/*-archetypes.md`, `skills/*/references/known-leaks.md`,
`data/archetypes.toml` and `data/known-leaks.toml` from the plugin copy,
blanks every `suggested_archetype = "..."` in `data/detectors/*.toml` and sets
`DLA_ARCHETYPES=off`. Plugin ablations also set `DLA_CONSENT_TIERS=1-4` and
`DLA_CONSENT_RAW_VALUES=1`, so the runner (not the model) gives consent.

The variant of an ablation can be assembled without running anything, for
example to validate it:

```
cargo run -q -p dla-bench -- variant --ablation gates --archetypes off --out /tmp/variant
claude plugin validate /tmp/variant --strict
```

The `full` variant still contains the `SessionStart` hook that downloads the
`dla` binary; the hook does not look at `DLA_BIN`, but the shim in `bin/`
prefers `DLA_BIN`, so runs use the local build.

### Results layout

```
<out>/runs/<ablation>/archetypes-<on|off>/<fixture>/<n>/
  run.json          always (written last: its presence marks a finished run)
  transcript.jsonl  the stream-json output of claude (git-ignored)
  stderr.log        (git-ignored)
  findings.json     when the model wrote one in the project root
  ledger.jsonl state.json report.md coverage.json
                    copied from .dla/<id in .dla/current>/ when dla ran
<out>/metrics.json  written by `grade`
<out>/table.md      written by `table`
```

`run.json` holds `schema_version, fixture, ablation, archetypes, run,
exit_code, timed_out, wall_seconds, usage{input_tokens, output_tokens,
cache_read_input_tokens, cache_creation_input_tokens}, cost_usd, num_turns,
model, started_at, error`. Usage, cost and turns come from the final `result`
event of the stream, the model from the `init` event. `exit_code` is -1 when
the process was killed or could not start; `error` says why. A run that hit
`--max-turns` exits 1 and counts as errored in the usage block, its findings
are still graded.

## Grade, compare, table

```
cargo run -q -p dla-bench -- grade bench/results/baseline-no-skill
python3 bench/runner/grade.py bench/results/baseline-no-skill --out /tmp/metrics.py.json
cargo run -q -p dla-bench -- compare bench/results/baseline-no-skill/metrics.json /tmp/metrics.py.json
cargo run -q -p dla-bench -- table bench/results/baseline-no-skill
```

`grade` reads `--fixtures` (default `bench/fixtures`) and writes
`<results>/metrics.json` (or `--out`); a run directory whose fixture has no
`expected.toml`, or without `run.json`, is an error that lists the
directories. `compare` exits 0 when two JSON files are equal as parsed JSON
(numbers compare by value, key order is ignored) and otherwise prints the
differing paths. `table` renders `metrics.json` into `table.md`: one metric
table per group (value, numerator/denominator, mean and standard deviation
over run indexes), a usage table and a per-fixture table.

The two graders implement `GRADING.md` independently and must agree exactly.
CI grades every sample under `bench/runner/testdata/*/` with both and
compares. `bench/runner/testdata/rust-sample` also holds
`expected-metrics.json`, the golden result that was derived by hand from the
sample runs and is compared by the Rust tests.

## Isolation and cost

- The child gets a fresh `CLAUDE_CONFIG_DIR` per run: none of the user's
  settings, plugins, memory or MCP servers are loaded
  (`--strict-mcp-config` with no `--mcp-config` yields an empty server list).
  The fresh directory holds no login, so authentication has to come from the
  environment: `ANTHROPIC_API_KEY`, `ANTHROPIC_AUTH_TOKEN`,
  `CLAUDE_CODE_OAUTH_TOKEN`, a Bedrock/Vertex setup, or a base URL that
  authenticates by itself (that is how it works in the cloud sandbox). The
  runner does not copy credential files because a refreshed token could
  invalidate the parent's.
- The child environment is the parent's minus everything that would make the
  child act as part of the parent session: every variable starting with
  `CLAUDE` (except the auth ones above), `CCR_*`, `AI_AGENT`,
  `MAX_THINKING_TOKENS`, `GH_TOKEN`, `GITHUB_TOKEN` and similar tokens, and any
  `DLA_*` setting. `CLAUDE_CODE_SESSION_ID` and `CLAUDE_SESSION_ID` are
  therefore never inherited, so `dla` never mistakes the parent for the run's
  session. The runner sets `CLAUDE_CONFIG_DIR`, `DISABLE_AUTOUPDATER=1`, the
  `DLA_*` values of the ablation and, when it runs as root, `IS_SANDBOX=1`,
  which is what lets `--permission-mode bypassPermissions` run as root.
- The child is not sandboxed. It can read other paths on this machine and
  reach whatever the network allows; the prompt only asks it to stay inside
  the checkout. The protection of the answer key is that the working copy
  lives outside the repository and never contains `expected.toml`.
- The cost is real: every run is a full agent session with builds and
  measurements. A trivial two-turn call is about 0.02 USD; a diagnosis run is
  orders of magnitude more. Use `--runs 1 --fixtures <id>` first, set
  `--budget-usd`, and watch the cumulative cost in the progress lines.
- `--max-turns` is accepted by `claude` 2.1.319 although `claude --help` does
  not list it; reaching it ends the run with `subtype: error_max_turns` and
  exit code 1. Close stdin (the runner does) or `claude -p` waits a few
  seconds for piped input.
