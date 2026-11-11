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
prompts, category rules, no committed build outputs, and for the default
`repository` entry 3 to 10 source files and 150 to 600 source lines) and scans `project/` for hint words
(`leak*`, `todo`, `fixme`, `fix`, `bug`, `unbounded`, `bounded`,
`never freed|removed|cleared|closed|stopped|released`) with word boundaries,
case-insensitively. File names are scanned too, and `_` counts as a separator,
so `order_leak_count` is caught. It prints one `fixture: problem` line per
problem and exits 1 if there is any. An empty or missing fixtures directory is
valid.

The entry paths change the rules (see `FIXTURES.md`): an `artifact` fixture
(`entry = "artifact"`) has no source-size rule and no site check; instead every
`project/dumps/*.hprof.gz` must be a gzip file that inflates to a file starting
with `JAVA PROFILE 1.0.`, with timestamps in the dump headers that differ from
each other. A `symptom` fixture may hold only non-source notes. A fixture with
`runtimes = [...]` needs source files for each listed runtime, and `.kt` sources
need a `build.sh` that tries `kotlinc` first. A `[[leak]]` without `site` needs an
`owner`.

Regenerate and check the schemas with `dla-bench schema` and
`dla-bench schema --check` (CI runs the check).

## Run a baseline

`run` needs Linux or macOS (the child sessions run in the Claude Code sandbox);
on Windows it exits with an error. All other commands work on Windows (`dynamic --without` is untested there).

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
   `${XDG_CACHE_HOME:-$HOME/.cache}/dla-bench` (resolved when the command runs,
   not in `/tmp`, which the children may write to) and must be outside the
   repository (and outside every path the child may not read, see below);
2. gives the run a fresh `CLAUDE_CONFIG_DIR` inside that directory, writes the
   sandbox settings (below) to `<run dir>/settings.json` and creates the
   toolchain cache directories;
3. builds the ablation (below), then runs
   `claude -p <prompt> --output-format stream-json --verbose --max-turns T
   --settings <run dir>/settings.json --strict-mcp-config
   --no-session-persistence [--model M] [--plugin-dir <variant>]` with the
   copied project as working directory and stdin closed. The permission mode
   is the default one; there is no `bypassPermissions`;
4. kills the whole process group after `--timeout` seconds and records
   `timed_out`;
5. collects the results (below). Before an attempt the run directory is
   cleaned of `findings.json`, `ledger.jsonl`, `state.json`, `report.md`,
   `coverage.json`, `transcript.jsonl`, `stderr.log` and `run.json`, so
   nothing of an earlier attempt can be graded as the new answer. A run
   directory that already holds `run.json` is skipped, so an interrupted
   baseline continues where it stopped; `--runs 5` after `--runs 3` adds runs
   4 and 5.

Scheduling is run-major (run 1 of every fixture, then run 2, ...) and
interleaved across runtimes so heavy builds do not pile up. `--parallel P`
(default 2) runs P sessions at a time. `--budget-usd X` stops starting new
runs once the summed `cost_usd` of the runs finished in the results directory
reaches X; runs finished by an earlier invocation count, runs in flight
finish. One progress line is printed per finished run: fixture, run index,
exit code, minutes, cost and the cumulative cost of the results directory. The working copy of a run that ended
cleanly is removed; keep it with `--keep-work`, failed runs are always kept.

### Interrupts, lock and retry

- **Interrupt.** On SIGINT or SIGTERM the runner sends SIGTERM to the process
  group of every live session, waits 2 seconds, sends SIGKILL, releases the
  lock and exits with 130 (SIGINT) or 143 (SIGTERM). An interrupted run gets
  no `run.json`; start the same command again to redo it. Sessions run in
  their own process groups, so a terminal Ctrl-C reaches them only through the
  runner. If the runner itself is killed with SIGKILL it cannot clean up:
  look for `claude` processes before starting again.
- **Lock.** The runner takes `<out>/runner.lock` (containing its pid) before it
  builds or starts anything. A second runner on the same results directory
  refuses to start and names the pid. A lock whose pid is no longer alive is
  stale and is replaced; a lock file that holds no pid is never guessed at:
  remove it by hand once no runner is active.
- **Retry.** `--retry-errored` treats a `run.json` with a nonzero or missing
  `exit_code`, `timed_out` true or a non-null `error` as unfinished and runs
  it again (after cleaning the directory, see above). Without the flag an
  errored run is final. Errored runs that are retried do not count towards
  `--budget-usd`, since their result is replaced.
- **Systematic failure.** After 3 consecutive finished runs with zero turns and
  zero tokens (a lapsed login, an exhausted quota, no network) the runner
  stops starting runs, prints which cause that points to and exits non-zero.
  The three failed runs are on disk as errored runs; fix the cause and run the
  same command with `--retry-errored` to redo them and the rest of the queue.

Other flags: `--fixtures a,b|all`, `--fixtures-dir`, `--model`, `--max-turns`
(default 80), `--timeout` (default 1500), `--child-budget` (default `20m`: the
time budget the child's `dla` runs get, passed as `DLA_DEFAULT_BUDGET`; a
duration such as `90s`, `20m` or `2h`), `--repo-root` (where the plugin
sources are), `--ablations-dir`, `--dla-bin`, `--claude-bin`, `--deny-read
<path>` (repeatable, see below) and `--probe-prompt <file>` (use the file as
the prompt instead of the fixture's task; for checking the sandbox).

### Ablations

| Ablation | What the session gets | Prompt |
|---|---|---|
| `no-skill` | no plugin | task + the `findings.json` instructions |
| `prose` | the skill text of `ablations/prose/SKILL.md`, `references/`, no binary, hooks or agents | `/dla:detect-leak-anywhere ` + task + the `findings.json` instructions |
| `gates` | the plugin without `hooks/`, without `agents/verifier.md`, with any `hooks:` block removed from the SKILL.md frontmatter; `DLA_BIN` points at `target/release/dla`. The gates are `dla` commands the session runs itself; the integrity seals stay (they are written by `dla`) | `/dla:detect-leak-anywhere ` + task + an unattended-run note |
| `full` | the plugin as it is: `hooks/` with the `hooks.json` gates (worker `SubagentStop`, the `if`-filtered `PreToolUse` write and read gates), the SKILL.md frontmatter hooks and `agents/verifier.md` | same as `gates` |

The `findings.json` instructions of `no-skill` and `prose` runs say that `site` stays
empty for a finding without a source line (an artifact task) and that such a finding
carries an optional `owner` (`package.Class.field`); both graders read `owner` from
`findings.json` and match owner-only leaks by it (`GRADING.md`).

`gates` and `full` build `dla` once with `cargo build --release -p dla-cli`
(or use `--dla-bin`) and copy the binary to `<work-root>/<results name>/bin/dla`,
because the repository is unreadable to the child; `DLA_BIN` points at the
copy. The plugin variants are built under the work root as well. `--archetypes off` removes
`skills/*/references/*-archetypes.md`, `skills/*/references/known-leaks.md`,
`data/archetypes.toml` and `data/known-leaks.toml` from the plugin copy,
blanks every `suggested_archetype = "..."` in `data/detectors/*.toml`, removes
every `<!-- archetypes:begin -->` ... `<!-- archetypes:end -->` block from every `.md`
file of the variant (`agents/verifier.md` carries the 17 IDs in one; an unbalanced marker
fails the build of the variant) and sets `DLA_ARCHETYPES=off`. Plugin ablations also set `DLA_CONSENT_TIERS=1-4` and
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

The hooks of `full` run `"${CLAUDE_PLUGIN_ROOT}/bin/dla" gate ...` inside the child's
environment, so the child needs `DLA_BIN` (the runner sets it for `gates` and `full`, to the
copy of the binary next to the working copies) and the shim needs `bash`, `dirname` and
`readlink` on `PATH` (the runner passes `PATH` through). The shim exports `DLA_PLUGIN_ROOT`
and `DLA_DATA_DIR` itself. `tests/run_fake.rs` runs the first hook command of a `full` variant
the way Claude Code does (a shell with `CLAUDE_PLUGIN_ROOT` set) from the environment the
runner builds, and checks that the shim reaches the binary. It does not prove that Claude
Code fires the hooks; `tests/live_hooks.sh` is the live check.

### Results layout

```
<out>/runs/<ablation>/archetypes-<on|off>/<fixture>/<n>/
  run.json          written last: its presence marks a finished run (also a
                    failed one, see --retry-errored); absent after an interrupt
  runner.lock       only while a runner is active
  transcript.jsonl  the stream-json output of claude (git-ignored)
  stderr.log        (git-ignored)
  findings.json     when the model wrote one in the project root
  ledger.jsonl state.json report.md coverage.json
                    copied from .dla/<id in .dla/current>/ when dla ran
  ledger.<worker>.jsonl
                    the slices of the workers of a multi-runtime run
  bundle/<id>/index.json manifest.json excerpt*.txt
                    the index, the manifest and the excerpts of each verification
                    bundle: what the verifier was shown (nothing else of a bundle)
  integrity/<file>.seal
                    the seals `dla` wrote for the ledger, state, coverage and report
<out>/metrics.json  written by `grade`
<out>/table.md      written by `table`
```

`run.json` holds `schema_version, fixture, ablation, archetypes, run,
exit_code, timed_out, wall_seconds, usage{input_tokens, output_tokens,
cache_read_input_tokens, cache_creation_input_tokens}, cost_usd, num_turns,
model, started_at, error` and, for `gates` and `full` runs whose `state.json` was
collected, `incomplete_reason` (why `dla` marked the run incomplete, when it did) and
`stop_blocks` (how many times the stop hook blocked the stop; 0 when none, absent when
there is no `state.json`). Usage, cost and turns come from the final `result`
event of the stream, the model from the `init` event. `exit_code` is -1 when
the process was killed or could not start. `error` says why a run failed: a
spawn failure, a timeout, a missing `result` event, or the `result` event's own
failure, written `result: <subtype>` (for example `result: error_max_turns`)
when `is_error` is true or `subtype` is not `success`. A run that hit
`--max-turns` exits 1 and counts as errored in the usage block, its findings
are still graded.

### Keeping results

Raw runs are working material: a benchmark is run, graded, summarized and then deleted.
`<out>/runs/` is git-ignored. What stays in `bench/results/<name>/` is the record of what the
run proved: `README.md` (the setup, the comparison and the failures, written from the runs),
`metrics.json` and `table.md`. Grade and summarize before deleting the runs: a deleted run
cannot be regraded. The ledgers the schema-replay test needs are kept as test data
(`crates/dla-core/tests/data/old-ledgers`), not as results.

## Grade, compare, table

```
cargo run -q -p dla-bench -- grade bench/results/baseline-no-skill
python3 bench/runner/grade.py bench/results/baseline-no-skill --out /tmp/metrics.py.json
cargo run -q -p dla-bench -- compare bench/results/baseline-no-skill/metrics.json /tmp/metrics.py.json
cargo run -q -p dla-bench -- table bench/results/baseline-no-skill
```

`grade` reads `--fixtures` (default `bench/fixtures`) and writes
`<results>/metrics.json` (or `--out`). A missing `runs/` directory is an
error, and so is a run directory that has a `run.json` but whose fixture has
no `expected.toml`, or a `run.json` that is not a JSON object or not UTF-8.
A run directory without `run.json` is skipped and counted as incomplete in
`usage.incomplete` (it is not an error and needs no `expected.toml`); a group
with only such runs is written with `runs` 0 and null means. Directories that
are not one of the four ablations are ignored. `compare` exits 0 when two JSON files are equal as parsed JSON
(numbers compare by value, key order is ignored) and otherwise prints the
differing paths. `table` renders `metrics.json` into `table.md`: one metric
table per group (value, numerator/denominator, mean and standard deviation
over run indexes), a usage table and a per-fixture table.

`metrics.json` has seventeen ratio metrics (four of them added in M7:
`undiscriminated_refutations`, `uncited_findings`, `uncited_checklist_answers`
and `dropped_rate`, plus the `decoys` and `agreement` blocks; see `GRADING.md`).
`dla-bench gates`, `close-calls` and `calib-extract` read them. `source_overclaim_rate` is the share
of `artifact`-entry runs that claim a leak at a source site although no source
was given; it is null when a results directory holds no such run, which is the
case for every result before M4. A finding may carry an `owner`, a leak without a
`site` is matched by it, ledger ids may carry a worker prefix (`jvm/F3`), and a
merged duplicate counts once; the per-fixture `worker_provenance` (only present
when a chosen finding has a worker) says which worker produced the finding that
found each leak. It is informational and part of no metric. See `GRADING.md`.

The two graders implement `GRADING.md` independently and must agree exactly.
CI grades every sample under `bench/runner/testdata/*/` with both and
compares (`cargo test -p dla-bench --test graders_agree` does the same when
`python3` is installed). `bench/runner/testdata/rust-sample` also holds
`expected-metrics.json`, the golden result that was derived by hand from the
sample runs and is compared by the Rust tests. `testdata/m4-sample` covers the
rules added in M4 (owner matching, worker prefixes, merged findings, artifact
overclaim, worker provenance); its numbers are asserted in `tests/golden.rs`.

`dla-bench static` skips fixtures whose `entry` is `artifact` or `symptom` (they
hold no source to probe) and reports them as skipped with the reason, in the table
and as `skipped` in the JSON; `static` runs every runtime a multi-runtime fixture
lists and fails the fixture when the probe does not detect one of them.
`dla-bench dynamic` runs all three entry paths (below).

## Dynamic acceptance

`dla-bench dynamic` is the deterministic acceptance driver of the dynamic
core (M3). Like `dla-bench static` it runs outside the sandbox with no model:
each fixture is copied to a scratch directory (with `git init`), a run is
created with consent for tiers 1-4 (`DLA_CONSENT_TIERS=1-4`), and a fixed
policy, identical for every fixture, drives `dla`. The policy reads only the
outputs of `dla`; `expected.toml` is read after the run to grade the ledger
with the grader of the benchmark.

```sh
cargo build -p dla-cli -p dla-bench
./target/debug/dla-bench dynamic --dla-bin ./target/debug/dla --parallel 2 \
    --json /tmp/dynamic.json            # logs go to /tmp/dynamic.logs
./target/debug/dla-bench dynamic --only python-report-memo --keep-work
```

Options: `--only <text>` (fixtures whose id contains it), `--archetypes
on|off`, `--parallel <N>` (fixtures at the same time, default 2; a harness run
takes 10 to 90 s), `--json <file>`, `--log-dir <dir>` (default: `<json>.logs`),
`--keep-work` (keep the scratch copies), `--dla-bin`, `--repo-root`,
`--command-timeout <seconds>` (default 900: a `dla` command that runs longer is
killed with its whole process group, the fixture fails with a message naming the
command, and the rest of that fixture is skipped, so one stuck fixture cannot
hold a worker until the job timeout). Each fixture's run is created with
`--budget 2h`.

The policy, per fixture (repository entry, one runtime):

1. `run init`, `probe --build`, `sweep` and `census` per detected runtime,
   `workload discover`.
2. `harness gen --lang L` (default workload; on `workload-required` or
   `iteration-arg-unknown`, or when the harness does not run, the next main or
   script workload with an iteration argument, at most two) and
   `harness run H`. If no harness ran, every candidate becomes
   `suspected_static_only` (`build_fails` or `no_harness`).
3. `trend S`. For every passed column with candidates, the column's first
   candidate (once per run): add an N-WARM sibling, `ledger test`, `alloc-test
   H --finding F`. Favors leak: refute the sibling, annotate the holder if
   missing (census entry at the site, else `blocked-stack` for a goroutine or
   thread column, `registration-table` for a timer, listener or handle
   column, else `static/global`); the `ledger confirm` waits for step 6.
   Favors non-leak: refute the finding. Inconclusive: `unresolved`
   (`time_budget`).
4. Candidates tied only to non-passing columns: sibling, test, alloc-test;
   refuted when it favors non-leak, else unresolved.
5. When no passed column had candidates but a non-container column passed: a
   `growth-series` finding without a site, N-HEADROOM and N-ALLOC siblings, an
   unbound alloc-test, refuted when it favors non-leak.
6. The per-operation steps, for every harness that grew (a passing,
   non-container column), then the held-back confirmations. The steps read
   only `dla` JSON and never name a backend:
   - `dla summarize --list` (backend names grouped by op and runtime; `missing`
     lists the tools absent here).
   - `dla correlate H` for every JVM harness with a foreign-function boundary, before the trends:
     a harness without a foreign-function boundary refuses (logged, not a
     failure); one with a boundary adds a correlation candidate and an
     `alloc-stack` evidence that the shim column then ties to by site (this
     confirms `jni-order-codec` L1).
   - Capture. A node harness runs again as `harness run H --capture
     heap,alloc,nonheap`, a go harness as `--capture alloc,nonheap`, a python
     harness as `--capture histogram,alloc,nonheap` and a JVM harness as
     `--capture all` (the harness drops kinds a runtime does not write and
     names them). A rust or native harness runs again as `harness run H --tool
     valgrind` and `--tool heaptrack` for the tools that have a listed adapter;
     a missing tool refuses with `tool-missing` and leaves its `missing`
     coverage row, which is what a `--without` run looks for. Driver mode is
     not used: its hooks are filled in by reading the code, which the policy
     cannot do. Command-mode heap captures of one harness (3 or more files of
     one format) get a per-point `heap-graph` command, which the policy runs
     with the other printed commands.
   - Every `dla summarize` command the capture runs printed runs once (the
     full list is in `harness/H<n>/captures.json` when the output was cut),
     at most 24 per fixture.
   - Every call is checked, and a violation fails the fixture: stdout is at
     most 2048 bytes; `raw` exists in the run directory with a matching
     sha256; every evidence id is in the ledger; every evidence result is at
     most 2048 bytes; every tied passing column carries a `tie` reason.
     A refusal (for example `tool-missing`) is logged, not a failure; it
     still counts for the size check, and a crash or non-JSON output fails.
   - A finding that a summarize call's passing growth column ties, and that is
     still a candidate (a census finding whose in-process probe saw nothing,
     such as a closure context, or one the summarize step added), is driven
     like a harness-tied finding: `ledger add --sibling-of F` (N-WARM) citing
     the call's growth evidence, `ledger test F` with the workload of the
     harness, `ledger refute` of the sibling with the call's `favors leak`
     discriminator, then `ledger confirm F` citing the growth evidence and the
     call's evidence. Only a finding with a site (an owner on an artifact run)
     that no other driven finding shares is driven. A finding that is not a
     candidate (a refuted one stays refuted) or that has no discriminator
     favoring leak is left alone. This confirms `node-price-board` L1.
   - Each held-back `ledger confirm` cites the trend evidence and, when a
     summarize call's passing growth column is tied (with a reason) to the
     finding, that call's evidence (its histogram, allocation or retention
     evidence and its growth series), so the leak is `dump_backed`. The policy
     runs `dla ledger annotate F --holder H` only when a summarize call's
     `needed` names it (a finding with no holder) and the call's
     `observed_holder` agrees. If the
     ledger refuses that citation the confirmation is retried with the trend
     evidence alone and the refusal is logged.
7. `gap S --residual`; whatever is still a candidate or under test becomes
   `suspected_static_only` (`not_exercised`) when it has static evidence, else
   `unresolved`; then `report`.

8. The M6 steps (`bench/runner/src/dynamic/policy.rs`), after the steps above, on the
   confirmed and strong leak-side findings. They never change which findings are
   confirmed.
   - Label. A finding that matches an expected leak whose `cause` is a row with a shipped
     prose file (`L-CAPTURE`, `L-COLLECT`, `L-CYCLE`, `L-HANDLE`, `L-OWNERSHIP`,
     `L-REGISTER`, `L-TLS`, `L-UNLOAD`, `L-WORKER`; `tests/archetype_lists.rs` keeps the
     list equal to the rows of the `*-archetypes.md` files) gets `ledger label F --as
     <cause>`; a leak whose cause is `UNCLASSIFIED` gets `--as UNCLASSIFIED --note`. A
     refused label of a shipped row fails the fixture, so the table says whether every label
     is reachable. A finding whose holder differs from the expected one is not labelled (that
     is a failure already); a cause without a shipped row (`L-STALE`) is not tried.
     `ledger label F --suggest` runs for every confirmed finding and must say `on` or `off`
     as the run is.
   - Explain. Every confirmed or strong leak-side finding gets `ledger explain` with
     `--adds` the recorded writers (annotations plus the census entry of the owner or site),
     `--removal never` (`conditional` with `--remover` when a remover is recorded) and a
     text built from the site, the holder, the owner, the adding site and the removal. The
     mechanical policy cannot read the code: when no writer is recorded it records the
     finding's own site as the adding site (`ledger annotate F --writer SITE`). A refused
     explain fails the fixture. A finding without a site (an artifact run) cannot get a
     writer and is reported as a gap of the gate.
   - Intervene. On a repository fixture with one runtime, `dla intervene H --finding F
     --stub-writer W...` for the recorded writers (at most three) on the harness the finding
     was confirmed on. The effect (`growth-removed`, `growth-remains`, `inconclusive`,
     `invalid`) or the refusal code is reported; it never fails the fixture. A writer that
     is a declaration, a decorator or a condition is refused (`intervene-not-statement`);
     the policy has no way to find the statement that adds.
   - Strong. `ledger strong F --evidence E` with each eligible record: an intervention that
     removed the growth, and the latest retention records. Reported, never gating.
   - Close-out. `dla gate check --final` must pass. The policy runs the command the gate
     names for each item (answer the intake checklist items, `dla trend` for a series
     without one, `ledger add --origin growth-series` for a trend column tied to no finding
     and `ledger resolve --as unresolved` for that finding and for the residual open item).
     An item that cannot be reached is reported as a gap of the gate with the reason
     (`gate.known_gaps`); anything else still missing fails the fixture and is named.
   - `--archetypes off`: a label of a shipped row must be refused with `archetypes-off`
     (accepted, or refused with another code, is a failure), `UNCLASSIFIED` stays allowed,
     `--suggest` must say `off`; everything else is as with archetypes on.

At most eight alloc-tests run per fixture. Checks (the holder of each confirmed leak
must equal the expected holder, besides the list below): every confirmable leak of a
leak fixture with `dynamic_confirmation = true` is confirmed; decoy controls,
non-leak and impossible fixtures carry no claim; non-leak fixtures have at least
one rule-out; every confirmed finding satisfies the M3 rules when the ledger is
re-read (a test before the confirmation, a refuted N-* sibling, cited growth
evidence with gate pass, site and holder set); the unexplained growth share is
computable when a residual row exists; the summarize checks of step 6; the
`dump_backed` check. The command prints one row per fixture
(runtime, harness ran, series, passed columns, confirmed, refuted, claim,
rule-out, leaks, `ops/ev/B`, result) and totals. `ops/ev/B` is the number of
summarize calls that succeeded, the evidence ids they wrote and the largest
stdout in bytes (`row.json` and the `--json` file list the `<op>:<backend>`
names, the ids, the capture runs and the decisions), then `ok: N fixture(s) pass, M expected failure(s)`; it exits 1
on a failure. The log directory holds, per fixture, every `dla` command with
its JSON output (`NNN-<command>.log`), `row.json` (the row with the policy's
decisions) and the run files (`ledger.jsonl`, `report.md`, `series/`, `trend/`,
`alloc/`, `harness/H*/harness.json`).

`dla-bench run` also copies `series/S*.json`, `trend/T*.json`, `alloc/A*.json`,
`harness/H*/harness.json` and `run-*.json`, every `ledger.<worker>.jsonl`, every
`bundle/<id>/index.json`, `manifest.json` and `excerpt*.txt`, and the seals
`integrity/*.seal` of the child's dla run next to its
`ledger.jsonl` (never `work/`, `agent/`, `out/`, `logs/` or the rest of a bundle). The run directory is
writable by the sandboxed model, so the copy never follows a symbolic link, copies
only regular files with those names, and stops at 6 directory levels, 2000
entries and 64 MiB; whatever it left out is listed in `collect-skipped.txt` next
to the copied files and logged to stderr.

CI runs the command in the `dynamic` job with all four toolchains installed
and, for `jni-order-codec`, the Kotlin compiler 2.0.21 (the official release zip,
pinned with its sha256; the job also copies the compiler's `kotlin-stdlib.jar`
to the `~/.m2` path the fixture's `build.sh` runs its classes with, because the
runner has no Maven cache). The `static` job covers the `--archetypes off` path,
so `dynamic` does not repeat it; it needs no Kotlin, because the failed JVM build
only drops the class-file census and the source census and the DLA rules still
flag both leaks.

### Multi-runtime fixtures

A fixture whose `runtimes` lists two or more runtimes (`jni-order-codec`) runs
the workers. After `run init` and the core `probe --build` the driver calls
`run dispatch --workers <runtimes the probe found>`; then, for each runtime in
turn, it runs the policy above for that runtime only with `--worker <runtime>`
on every command (`sweep`, `census`, `workload discover`, `harness gen/run`,
`trend`, `alloc-test`, `ledger ...`; stage outputs land in
`sweep/<lang>-<backend>.json` and `workload/<runtime>.json`), closes the slice
with `ledger done`, and finally calls `ledger merge` and `report` in the core. A
real session runs the workers at the same time; the driver runs them one after
the other, which exercises the same slices and the same merge. The fixture
fails unless the merge succeeded, every worker's `merged_lines` is above zero,
no worker is unfinished and `coverage.json` has an entry for every runtime. The
JSON row has `workers` (`name`, `harnesses`, `merged_lines`) and `merge`
(`status`, `coverage_runtimes`, `duplicates`, `contradictions`, `unfinished`).
Leak grading reads the merged ledger.

### Artifact and symptom fixtures

`entry = "artifact"` fixtures hold heap dumps and no source. The driver lists
the `.hprof` and `.hprof.gz` files of the project, runs `run init --entry
artifact --artifact <each dump>` and `summarize heap-graph <all dumps>`, and
reads only the output of that command:

- gate pass: for each candidate (the holder, with an owner and no site) an
  N-WARM sibling, `ledger test` with the growth evidence, the sibling refuted
  with the dump-plateau discriminator that favors leak, `ledger confirm` citing
  the growth evidence. The finding names its owner and never a site; the M3
  rules check re-reads that (owner set, no site, growth evidence with authority
  `dump`).
- gate fail because the holder stopped growing (a discriminator that favors
  non-leak, no candidate): no claim. The holder is recorded as a finding with
  its owner and refuted with the discriminator, which the graders count as a
  rule-out.
- fewer than three dumps: no claim. The largest holder of the newest dump is
  recorded and resolved `suspected_static_only` (`not_exercised`) with a remedy
  that asks for at least three dumps.

`entry = "symptom"` fixtures: `run init --entry symptom`, then `ledger resolve
intake.target --status unresolved --reason env_unavailable --remedy "<what is
needed>"`, no stage, `report`. An impossible fixture whose ledger holds no
finding at all passes the no-claim check.

### Holder check

`grade_row` compares the holder of the chosen finding of every confirmable leak
(`LeakOutcome.chosen_holder`, the same one the grounded-attribution metric uses)
with the leak's `holder`. A leak that is confirmed with another holder fails the
fixture: `leak L1 (<site>) is confirmed with holder <h>, expected <h>`. The
artifact path takes the holder of the findings it records itself from the root
kind of the holder key (`static` is `static/global`, `jni-global` is
`native-handle/global-ref`, `thread` is `blocked-stack`, anything else is
`container-field`) and no longer writes `static/global` for all of them.
`expected.toml` has no alternative holders, so the check is exact.

### `--check-dump-backed`

A leak with `dump_backed = true` in `expected.toml` (see FIXTURES.md) must be
confirmed by a finding that cites evidence of kind `retention-path`,
`histogram-diff`, `alloc-stack`, `non-heap` or `should-be-dead` with authority
`dump` or `live`, in its
supports or in the evidence of its confirming transition. The check is **on by
default** (`CHECK_DUMP_BACKED` in `dynamic_cmd.rs`, flag
`--check-dump-backed=false` to turn it off): the policy runs the summarize
steps before it confirms and cites the evidence of a call whose passing growth
column is tied to the chosen finding (step 6 above). Marked: `go-subscription-workers` L1 (goroutine groups tied by creation site), `jvm-audit-chain` L1, `jvm-order-cache` L1,
`jni-order-codec` L2, `rust-tree-parent-links` L1, `jvm-extension-reload` L1,
`rust-meter-callbacks` L1, `jvm-invoice-render` L1, `node-price-board` L1 and
`native-frame-decoder` L1 (the one native fixture with a confirmable leak, which
runs under valgrind;
`native-glyph-cache` and `native-vendor-codec` have no confirmable leak).
The graders ignore the field and the grading metrics are unchanged.

### `--without <tool>[,<tool>...]`

Hides tools from every `dla` call of the run. Linux and macOS are the tested
platforms; on Windows the farm uses hard links or copies and is untested. Each
`PATH` directory that holds a hidden tool (matched on the file name) is replaced
by `<work>/bin-without/<n>/`, a directory of absolute links to its other
executables; the other `PATH` directories stay as they are, in order, so a
virtualenv or any other tool directory behaves as usual (an interpreter in a
virtualenv `bin` gets a small wrapper script, because Python finds its
`pyvenv.cfg` from the path it was started with). The tool-home variables that
belong to a hidden tool are unset for the call: `JAVA_HOME`, `JDK_HOME`,
`JRE_HOME`, `GRAALVM_HOME` for a JDK tool (`jcmd`, `jfr`, `jmap`, `jstack`,
`java`, ...), `GOROOT` for `go`, `ASYNC_PROFILER_HOME` and `ASPROF_HOME` for
`asprof`, `MAT_HOME` for `mat`, `NODE_HOME` for `node`; hiding `valgrind` leaves
them alone.

A run with hidden tools must still complete: a `dla` call that exits with 101 or
prints no JSON is a failure of the fixture (a JSON refusal such as `missing_tool`
is the expected degradation). Every coverage row (`coverage.json`) that depends
on a hidden tool must read `missing`. A row depends on a tool when it is named
like the tool (`jcmd`) or starts with it and a dash (`jcmd-class-histogram`,
`jfr-old-object-sample`). The `harness-jfr` row is not one: the JVM writes the
recording itself, and only reading it needs the `jfr` tool. The number of such rows is `hidden_backend_rows` in the
fixture row. The run fails when a hidden tool matched no row in any fixture,
because then hiding it was never exercised. The per-operation steps write the
rows: `jfr` (JVM capture and summarize), `valgrind` and `heaptrack` (the `--tool` runs of rust and
native harnesses; the harness names the row after the tool, also when only `heaptrack_print` is
missing, so a `--without heaptrack_print` run does not yet see a `heaptrack_print` row: only
`dla summarize` of a raw heaptrack recording writes that one). Each hidden
tool also has a per-fixture expectation: when the policy ran the capture that
needs it (`captures` in the row: `H1:capture-all` for `jfr`, `H1:tool-valgrind`
for `valgrind`, `H1:tool-heaptrack` for `heaptrack` and `heaptrack_print`), the
run must have written a row for the tool, and every such row must read
`missing` (`heaptrack_print` is looked up after `heaptrack`, so it is expected
only when `heaptrack` is not hidden too). A summarize call whose auto-selected
tool is missing succeeds with a `coverage_gap` list, no evidence and no `raw`
file; the checks accept that. A `dump_backed` failure of a run with a hidden-tool
row counts as a leak gap (the evidence of the hidden backend cannot be cited). A leak that stays unconfirmed or
gets another holder is reported as an expected failure of that run (`xfail: a
tool is hidden by --without`) only for a fixture that saw a hidden-tool row;
elsewhere the strict list below applies as usual. Any other failure stays a
failure. Hiding `jstack` or `jmap` alone does not turn the `jcmd` row to
`missing`; name `jcmd` for that. `memray` is not reliably hidden by name: attach
falls back to `python3 -m memray`. Tested with fake tools in
`bench/runner/tests/without.rs`.

### Go 1.27 prefetch (gates benchmark)

The goroutine leak profile needs Go 1.27. Before the first child of `dla-bench
run` starts, the runner makes sure `go1.27.0` is in the host module cache
(`GOTOOLCHAIN=go1.27.0 go version` fetches it when it is absent; a failure is a
warning, and the children then see the coverage row `go/goroutineleak` as
missing). Each child's private module cache gets the unpacked toolchain and its
download-cache entry (small files copied, the archive linked), so
`GOTOOLCHAIN=go1.27.0` switches offline with `GOPROXY=off`. The children run with
`GOTOOLCHAIN=auto` (with the proxy off nothing is downloaded, and a project that asks
for no newer Go builds with the local one); `dla` sets the switch itself for the leak
pass. Until the M5 gates run the children had `GOTOOLCHAIN=local`, which `dla`
respects as a ban on switching, so that run has the leak profile blocked. Checked by hand: with only those links in a fresh module cache,
`GOTOOLCHAIN=go1.27.0 GOPROXY=off go version` prints go1.27.0.

### Expected failures

The policy is mechanical: it cannot read the code to choose which edge of a
reference cycle to weaken. A fixture whose failure follows from that is listed in
`EXPECTED_FAILURES` (`bench/runner/src/dynamic_cmd.rs`): the fixture id, the
exact failure text and the reason. A listed fixture that fails with exactly the
listed text is reported as `xfail: <reason>` in the table and in the JSON
(`xfail`, `expected_failures`) and is not counted as a failure. The run fails when
a listed fixture passes (remove the entry) or fails with any other text or with an
additional failure (a regression must not hide behind the entry).

The last full run (`dla-bench dynamic --parallel 2`, release binaries, M6 after the fix rounds): 32 of 34
fixtures pass and 2 are expected failures: `node-socket-heartbeat` and `rust-tree-parent-links`
(the policy cannot read the code). Since M7 the `node-resources` summary names the observed holder of a timer group tied by the finding's own site, so `node-socket-heartbeat` passes and its entry is removed; `rust-tree-parent-links` is the only expected failure. After the benchmark follow-up (loopback `close` in the two node-socket-heartbeat fixtures, the label decision in the policy) the run was repeated with release binaries, archetypes on and off: 32 pass and the same 2 expected failures each time; `node-socket-heartbeat` still failed with the same holder text then. The four fixtures whose label could not be reached before the fix rounds
(`jvm-extension-reload`, `jvm-invoice-render`, `rust-meter-callbacks`, `jni-order-codec`) pass and their entries are removed. The
`needs_archetypes` flag of an entry stays for a future label entry. `node-retry-scheduler` passes: the policy runs the
`dla ledger annotate F --holder H --evidence E` command that `needed` prints for the finding of
`RetryScheduler.latest`, and for any other tied finding whose set holder differs from the one observed
(`needed` names the first tied finding only), and the ledger accepts it because the growth evidence's passing
column observed `static/global`.
`jvm-audit-chain` is dump-backed through `heap-graph`.

| fixture | failure | reason |
| --- | --- | --- |
| `rust-tree-parent-links` | `leak L1 (src/section.rs:27) is not confirmed` | The policy confirms the `children` edge of the `Rc` cycle (`section.rs:11`); the fixture expects the `parent` back-link (`section.rs:27`, alternatively `:10`, `:16-22`). Picking the edge to weaken needs reading the code. |

`jni-order-codec` meets the milestone's acceptance for multi-runtime runs (two
workers, one merged ledger with both slices, merged coverage): the driver fails
the fixture if that is not so. Its native leak L1 (`native/src/lib.rs:43`) is
confirmed since cut 2: the policy runs `dla correlate` for a JVM harness before
the trends, the correlation adds a candidate at the native site under the managed
method, and the shim column ties to it by site. The Kotlin leak L2 (`Audit.kt:12`)
has been confirmed since M4 (probe columns tie to the finding with that owner).

### M6 dynamic results

`dla-bench dynamic --parallel 2` on the 34 fixtures with release binaries, archetypes on (32 pass, 2
expected failures, 0 failures) and off (33 pass, 1 flaky failure, 2 expected failures, see below). The columns are the
M6 steps of the policy: the label of the matched expected leak, the explanation, `dla intervene
--stub-writer` (informational), `dla ledger strong` (informational) and `dla gate check --final`
(`pass`; `gaps` would mean the gate passes except for items the policy cannot reach, listed in
`gate.known_gaps` of the JSON; there are none). The policy follows every fix the gate prints: `dla trend <S>` and then
the `ledger add --origin growth-series` it prints for a trend column, `dla gap <S> --residual` for a whole-process
column, and `ledger explain F --tag inferred --removal not-found --text ...` for an artifact finding. `x2` is two findings; `unmatched` is a confirmed
finding that matches no expected leak; `not tried (holder)` is a finding whose holder differs from the expected one;
`not tried (worker harness)` is an intervention the policy does not run on a worker's harness. Decoys and impossible
fixtures confirm nothing, so they have no step except the gate.

Archetypes on:

| fixture | result | label | explain | intervene | strong | final gate |
|---|---|---|---|---|---|---|
| `go-buffer-pool` | pass | - | - | - | - | pass |
| `go-subscription-workers` | pass | ok | ok | refused intervene-not-statement | no eligible record | pass |
| `go-subscription-workers-closed` | pass | - | - | - | - | pass |
| `jni-order-codec` | pass | ok x2 | ok x2 | not tried (worker harness) x2 | ok; refused strong-second-class-missing x3 | pass |
| `jvm-audit-chain` | pass | ok | ok | invalid | ok | pass |
| `jvm-billing-private-repo` | pass | - | - | - | - | pass |
| `jvm-extension-reload` | pass | ok | ok | invalid | refused strong-holder-mismatch x3 | pass |
| `jvm-extension-reload-stopped` | pass | - | - | - | - | pass |
| `jvm-invoice-render` | pass | ok | ok | invalid | ok | pass |
| `jvm-invoice-render-trimmed` | pass | - | - | - | - | pass |
| `jvm-order-cache` | pass | ok | ok | invalid | ok | pass |
| `jvm-order-cache-bounded` | pass | - | - | - | - | pass |
| `jvm-rfq-dump-series` | pass | ok | ok | - | refused strong-not-independent x3 | pass |
| `jvm-rfq-dump-series-capped` | pass | - | - | - | - | pass |
| `jvm-rfq-single-dump` | pass | - | - | - | - | pass |
| `native-frame-decoder` | pass | ok | ok | refused intervene-not-statement | ok | pass |
| `native-frame-decoder-freed` | pass | - | - | - | - | pass |
| `native-glyph-cache` | pass | - | - | - | - | pass |
| `native-vendor-codec` | pass | - | - | - | - | pass |
| `node-price-board` | pass | ok | ok | refused intervene-no-control | refused strong-not-independent x3 | pass |
| `node-price-board-flat` | pass | - | - | - | - | pass |
| `node-retry-scheduler` | pass | ok; unmatched | ok x2 | growth-removed; refused intervene-not-statement | ok x2 | pass |
| `node-socket-heartbeat` | xfail | not tried (holder); unmatched x3 | ok x4 | refused intervene-not-statement; growth-removed; refused intervene-no-control x2 | refused strong-second-class-missing x3; ok; refused strong-not-independent x6 | pass |
| `node-socket-heartbeat-cleared` | pass | - | - | - | - | pass |
| `python-graph-churn` | pass | - | - | - | - | pass |
| `python-ingest-two-leaks` | pass | ok x2 | ok x2 | growth-removed x2 | ok x2 | pass |
| `python-report-memo` | pass | ok | ok | refused intervene-not-statement | no eligible record | pass |
| `python-report-memo-bounded` | pass | - | - | - | - | pass |
| `rust-batch-buffers` | pass | - | - | - | - | pass |
| `rust-meter-callbacks` | pass | ok | ok | refused intervene-not-statement | refused strong-second-class-missing x3 | pass |
| `rust-meter-callbacks-reclaimed` | pass | - | - | - | - | pass |
| `rust-tree-parent-links` | xfail | unmatched | ok | refused intervene-not-statement | refused strong-second-class-missing x3 | pass |
| `rust-tree-parent-links-weak` | pass | - | - | - | - | pass |
| `symptom-no-target` | pass | - | - | - | - | pass |

Archetypes off. Every label of a shipped row was refused with `archetypes-off` (12), the three
`UNCLASSIFIED` labels were accepted, and nothing else changed: the same leaks are confirmed with the same holders, the
same explanations are accepted and the final gate passes everywhere. The first full run had one failure,
`go-buffer-pool` (`no rule-out on a non-leak fixture`): the alloc-test on `go.heap_live` was inconclusive (once a
false `leak`) on a non-leak fixture, so the candidate stayed unresolved. Four more runs of that fixture with archetypes off gave 2 failures and 2
passes, and four with archetypes on gave none (nor did the full run), so the flake may depend on the archetypes setting; the row below is from one of the passing runs. See `docs/VERIFY.md`.

| fixture | result | label | explain | intervene | strong | final gate |
|---|---|---|---|---|---|---|
| `go-buffer-pool` | pass | - | - | - | - | pass |
| `go-subscription-workers` | pass | refused archetypes-off (expected) | ok | refused intervene-not-statement | no eligible record | pass |
| `go-subscription-workers-closed` | pass | - | - | - | - | pass |
| `jni-order-codec` | pass | refused archetypes-off (expected) x2 | ok x2 | not tried (worker harness) x2 | ok; refused strong-second-class-missing x3 | pass |
| `jvm-audit-chain` | pass | ok | ok | invalid | ok | pass |
| `jvm-billing-private-repo` | pass | - | - | - | - | pass |
| `jvm-extension-reload` | pass | refused archetypes-off (expected) | ok | invalid | refused strong-second-class-missing x3 | pass |
| `jvm-extension-reload-stopped` | pass | - | - | - | - | pass |
| `jvm-invoice-render` | pass | refused archetypes-off (expected) | ok | invalid | ok | pass |
| `jvm-invoice-render-trimmed` | pass | - | - | - | - | pass |
| `jvm-order-cache` | pass | refused archetypes-off (expected) | ok | invalid | ok | pass |
| `jvm-order-cache-bounded` | pass | - | - | - | - | pass |
| `jvm-rfq-dump-series` | pass | refused archetypes-off (expected) | ok | - | refused strong-not-independent x3 | pass |
| `jvm-rfq-dump-series-capped` | pass | - | - | - | - | pass |
| `jvm-rfq-single-dump` | pass | - | - | - | - | pass |
| `native-frame-decoder` | pass | refused archetypes-off (expected) | ok | refused intervene-not-statement | ok | pass |
| `native-frame-decoder-freed` | pass | - | - | - | - | pass |
| `native-glyph-cache` | pass | - | - | - | - | pass |
| `native-vendor-codec` | pass | - | - | - | - | pass |
| `node-price-board` | pass | refused archetypes-off (expected) | ok | refused intervene-no-control | refused strong-not-independent x3 | pass |
| `node-price-board-flat` | pass | - | - | - | - | pass |
| `node-retry-scheduler` | pass | ok; unmatched | ok x2 | growth-removed; refused intervene-not-statement | ok x2 | pass |
| `node-socket-heartbeat` | xfail | not tried (holder); unmatched x3 | ok x4 | refused intervene-not-statement; growth-removed; refused intervene-no-control x2 | refused strong-second-class-missing x3; ok; refused strong-not-independent x6 | pass |
| `node-socket-heartbeat-cleared` | pass | - | - | - | - | pass |
| `python-graph-churn` | pass | - | - | - | - | pass |
| `python-ingest-two-leaks` | pass | ok; refused archetypes-off (expected) | ok x2 | growth-removed x2 | ok x2 | pass |
| `python-report-memo` | pass | refused archetypes-off (expected) | ok | refused intervene-not-statement | no eligible record | pass |
| `python-report-memo-bounded` | pass | - | - | - | - | pass |
| `rust-batch-buffers` | pass | - | - | - | - | pass |
| `rust-meter-callbacks` | pass | refused archetypes-off (expected) | ok | refused intervene-not-statement | refused strong-second-class-missing x3 | pass |
| `rust-meter-callbacks-reclaimed` | pass | - | - | - | - | pass |
| `rust-tree-parent-links` | xfail | unmatched | ok | refused intervene-not-statement | refused strong-second-class-missing x3 | pass |
| `rust-tree-parent-links-weak` | pass | - | - | - | - | pass |
| `symptom-no-target` | pass | - | - | - | - | pass |

Totals (on / off): labels accepted 15 / 3 (UNCLASSIFIED) with 0 / 0 refused for missing evidence and 0 / 12
refused `archetypes-off`; explanations accepted 21 / 21; interventions `growth-removed` 4 / 4,
`invalid` 4 / 4 (JVM harness exits 1 once the stubbed writer is gone, see `docs/VERIFY.md`),
refused `intervene-not-statement` 7 / 7 and `intervene-no-control` 3 / 3; `strong` reached 10 / 10;
final gate passed 34 / 34, gaps only 0 / 0, missing 0 / 0.

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
  session. The runner sets `CLAUDE_CONFIG_DIR`, `DISABLE_AUTOUPDATER=1`,
  `CLAUDE_CODE_TMPDIR` (a `tmp` directory of the run, so that parallel runs do
  not see each other's temporary files), the toolchain caches, `DLA_DEFAULT_BUDGET`
  (from `--child-budget`, so the budget paths of `dla` run inside the
  timeout) and the other `DLA_*` values of the ablation.
- Every child runs under Claude Code's own Bash sandbox, written to the run's
  `settings.json` (`sandbox_settings` in `bench/runner/src/run.rs`):
  `sandbox.enabled`, `autoAllowBashIfSandboxed`, `allowUnsandboxedCommands:
  false` and `failIfUnavailable: true` (a machine without bubblewrap and
  socat fails the run instead of running unconfined). Bash commands can write
  only in the working copy, the run's `cache` and `tmp` directories and the system
  temporary directory `/tmp` (JVM attach needs it), and
  have no network: no `allowedDomains` are configured, so every connection is
  refused by the sandbox's proxy (this includes package registries; builds
  must work offline, and `GOPROXY=off`, `PIP_NO_INDEX=1`,
  `CARGO_NET_OFFLINE=true` say so to the tools). `WebFetch` and `WebSearch`
  are denied.
- The paths in `sandbox.filesystem.denyRead` are invisible to Bash. The Read,
  Edit, Write, Glob and Grep tools are not covered by the sandbox, so the same
  paths are also in `permissions.deny` (as `Tool(//absolute/path/**)`), and
  Edit and Write are allowed only under the working copy, `cache` and `tmp`
  (not `/tmp`). The `Agent` and `Task` tools are allowed, so a skill can
  dispatch subagents.
  The denied paths are the repository root (answer keys), the fixtures
  directory, the results directory, `~/.claude`, the parent's
  `CLAUDE_CONFIG_DIR` and every `--deny-read` path. Pass the scratch
  directories of your own session with `--deny-read`, but never Claude
  Code's shared temporary directory (`claude-<uid>` under the system temp
  directory) or anything above it: child sessions keep their shell state
  and `TMPDIR` there, and denying it makes every Bash call fail to record
  its state and breaks tools such as valgrind while the run still looks
  healthy. `dla-bench run` refuses such a path; deny the session's own
  subdirectory of it instead.
- Toolchain caches live in `<run dir>/cache`: `GOCACHE`, `GOPATH`,
  `GOMODCACHE`, `GOTMPDIR`, the Maven local repository
  (`MAVEN_OPTS=-Dmaven.repo.local=...`), the npm and pip caches, `CARGO_HOME`
  and `XDG_CACHE_HOME`. `RUSTUP_HOME` keeps pointing at the installed
  toolchains (read only). Nothing else is writable.
- The default permission mode applies. In `-p` mode a call that would need an
  approval is refused, not asked. Notably a Bash command that expands `$?` or
  another variable that cannot be checked before it runs, for example
  `make; echo "exit=$?"`, is refused; the model usually retries the parts as
  separate commands, which costs a turn.
- Writable `/tmp`: a child can read and write everything in `/tmp` that is
  not denied, including other sessions' files under `/tmp/claude-<uid>`. A path
  passed with `--deny-read` that lies under `/tmp` is hidden from Bash (the
  sandbox mounts an empty directory over it, so reads, writes and deletes of its
  files do not reach the real one; verified, see `docs/VERIFY.md`) and is also
  listed in `sandbox.filesystem.denyWrite`. Denying the shared `claude-<uid>`
  directory itself, for reading or writing, breaks every Bash call, so the
  runner refuses it. Pass your own session's scratch directory with
  `--deny-read`. The working copies are no longer in `/tmp` for that reason: a
  child cannot reach another run's working copy.
- Inside the sandbox the PID namespace is separate and `/proc` does not show it:
  the runner's own processes are invisible to the child, and `jcmd -l` lists a
  JVM the child started but `jcmd <pid> <command>` and `jstack <pid>` fail with
  "non existent JVM pid" (a SIGQUIT thread dump with `kill -QUIT` works). Attach
  backends cannot be exercised in a benchmark child.
- Known gaps: the sandbox only confines what the model does, not what the
  plugin's hooks do.
- The cost is real: every run is a full agent session with builds and
  measurements. A trivial two-turn call is about 0.02 USD; a diagnosis run is
  orders of magnitude more. Use `--runs 1 --fixtures <id>` first, set
  `--budget-usd`, and watch the cumulative cost in the progress lines.
- `--max-turns` is accepted by `claude` 2.1.319 although `claude --help` does
  not list it; reaching it ends the run with `subtype: error_max_turns` and
  exit code 1. Close stdin (the runner does) or `claude -p` waits a few
  seconds for piped input.
