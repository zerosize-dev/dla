# Verification log

This file tracks everything in the project that is not yet checked against a primary source or a
running system: rule IDs and tool flags that came from memory, behaviours that were read in
documentation but never run, open design questions, and corrections to the original plan.

How to use it:

- Add a row when you rely on something you have not verified. Do not state it as fact elsewhere in
  the repository until the row is closed.
- Status is one of `unverified`, `verified` or `corrected`.
- When you verify or correct an item, fill in the evidence (the command run, the page read, the
  version tested) and the resolution (what changed in the repository as a result), then change the
  status. Keep closed rows; they are the history.
- Shipping rule: no detector rule, tool flag or archetype row goes into a runtime reference while
  its row here is `unverified`, unless the reference marks it as unverified too.

## Plan corrections and inconsistencies

| Item | Status | Evidence | Resolution |
|---|---|---|---|
| The plan says the workspace has "eight crates" but its layout lists ten: core, stats, probe, census, workload, sample, backends, parsers, report, cli | corrected | Counting the layout in the plan gives ten names | Ten crates were created. The seven that milestone M0 does not use are library stubs holding only a crate-level doc comment that states their future responsibility |
| The archetype data example uses the holders `thread` and `executor`, which are not in the holder vocabulary (static/global, registration-table, container-field, thread-local, blocked-stack, closure/continuation, native-handle/global-ref, cycle-only) | unverified | The example is inconsistent with the closed holder enum in `dla-core` | Resolve in M6 when the archetype data format is defined: map `thread` and `executor` onto `blocked-stack` or `registration-table` per row, or extend the vocabulary deliberately and update the schema |
| Plan correction (M6): the plan says a `Stop` hook in `worker.md` becomes `SubagentStop`. Plugin agents ignore the `hooks` frontmatter field, so this cannot work for a plugin | corrected | Verified empirically: a plugin agent with `Stop` and `PreToolUse` frontmatter hooks ran Bash and neither marker file was created (run with the Claude Code CLI in print mode against a throwaway plugin). The sub-agent docs say the same for plugin subagents (https://code.claude.com/docs/en/sub-agents) | Decide in M6 between a `SubagentStop` entry in `hooks/hooks.json` (conflicts with the rule that plugin hooks hold only `SessionStart`) and a hook in the SKILL.md frontmatter (documented to fire for plugin skills; the subagent case is unverified). Agents carry no hooks until then |

## Research facts that are not verified by running them

These came from documentation or a changelog and could not be exercised in the build environment.
The remaining research facts were checked by running the Claude Code CLI.

| Item | Status | Evidence | Resolution |
|---|---|---|---|
| Plugin agent frontmatter: `permissionMode`, `hooks`, `mcpServers` and `initialPrompt` are ignored for plugin agents, and the other listed fields are honoured | unverified | Read in the Claude Code plugin and sub-agent docs. The `hooks` part is verified: hooks in a plugin agent do not fire (see the M6 plan correction above). The other fields were not exercised | Agents carry no hooks. Recheck the supported field list when raising the minimum Claude Code version |
| Hooks on Windows: shell-form hooks run in Git Bash when installed and in PowerShell otherwise; exec form needs a real executable | unverified | Read in the hooks docs; the build environment is Linux | The README now says Windows needs Git for Windows. CI runs `tests/shim/run.sh` under Git Bash on a Windows runner. Still untested: a machine without Git Bash, where `bash "..."` fails as a non-blocking error; consider a Node-based hook or two entries with `shell` set |
| Minimum Claude Code versions per feature (plugin options 2.1.83, plugin data 2.1.78, plugin `bin/` 2.1.91, plugin skill hooks 2.1.94, exec-form hook args 2.1.139, Stop-hook continuation cap 2.1.143, `${user_config.*}` shell-form rejection 2.1.207, `options` enum 2.1.271, validating both manifests 2.1.289) | unverified | Mapped from changelog lines, not tested on old versions | README states 2.1.94 as the floor. Install the floor version in CI once and run the plugin checks against it |
| Minimum Node for memlab: 18 in practice (its browser driver requires 18; the core package declares 12.6) | unverified | Package `engines` fields; no memlab run | README recommends 20 or 22. Run a memlab scenario on the floor version before M-series work that depends on it |
| Minimum JDK for Eclipse MAT: 21 for the 1.17 stand-alone release | unverified | MAT download page and manifest; no MAT run | Run headless `ParseHeapDump.sh` on a JDK 21 install before relying on it |
| `claude plugin eval` case format and commands (needed for the evaluation suite) | unverified | Read in the plugin evals docs; needs model credentials, so not run | Create one case with `claude plugin eval init --bare` and run it when the suite starts |
| `onFailure: "block"` has no effect on Stop-type hooks, so Stop hooks always fail open | unverified | Docs statement; fail-open behaviour for crashes and non-zero exits was verified | Recheck on the version raised to in the future |
| Defaults for `userConfig` options are not exported to hooks, and the Bash tool receives no plugin variables | verified | Empirical hook environment dump and Bash tool `env` listing | The shim locates the data directory itself. Only the time budget has a binary default (60m, `DLA_DEFAULT_BUDGET`); the other settings reach `dla` through the skill (next row) |
| The Bash tool receives no `CLAUDE_PLUGIN_*` or `CLAUDE_PLUGIN_OPTION_*` variables, so `userConfig` values and the session id reach `dla` through text substitution in SKILL.md (`${user_config.time_budget}`, `${CLAUDE_SESSION_ID}`) | verified | Run in a session with the plugin loaded: `env \| grep ^CLAUDE_PLUGIN` printed nothing in the Bash tool, while the skill body rendered `Max: 256` for a `userConfig` number and the absolute `CLAUDE_PLUGIN_DATA` path. Non-sensitive values only | SKILL.md stage 1 runs `dla run init --budget ${user_config.time_budget} --session-id ${CLAUDE_SESSION_ID}` |

## Items the plugin work flagged for verification

| Item | Status | Evidence | Resolution |
|---|---|---|---|
| Stage commands in the skill and agent bodies follow the planned command surface, but their flag shapes are provisional: `dla probe`, `sweep`, `sample`, `trend`, `gap --residual`, `workload`, `gate check --stage/--final`, `ledger confirm/strong/refute/label/explain`, `bundle`, `ledger verify`, `ledger --worker` | unverified | None of these commands exist in M0 | Reconcile each with the real CLI in the milestone that implements it (M3 to M6). The skill tells the model to record `unresolved(tool_missing)` when a command is absent |
| The name `dla ledger refute` is a placeholder; the plan only says the refute command arrives in M3 | unverified | Chosen by the plugin author | Confirm or rename in M3 and update the skill and agents |
| Windows (Git Bash and PowerShell) and macOS behaviour of `bin/dla` and `hooks/fetch-dla.sh`, including bash 3.2 | unverified | Only Linux was tested locally; CI runs `tests/shim/run.sh` on all three systems. On Windows the symlinked-shim case is skipped because Git Bash `ln -s` copies files, and the colon-in-path data directory case is skipped because Windows cannot create such a directory. The hooks require Git Bash | Confirm the CI shim job passes on macOS and Windows runners, then close |
| The real `dla` binary has not been run through the shim and downloader | unverified | On Linux `tests/shim/run.sh` now installs the real `target/debug/dla` through the downloader and runs it through the shim; macOS and Windows runs are not yet confirmed | `tests/shim/run.sh` uses `target/debug/dla` (`dla.exe` on Windows) automatically when it exists and matches the host; confirm in CI |
| The worker agent's preload of the `dla:detect-leak-anywhere` skill resolves at runtime | unverified | `claude plugin validate` and the headless listing pass; no session was run | Run one agent invocation and check that the skill text is present |
| Downloaded binaries of older plugin versions are never pruned from the data directory | unverified | By design in M0 | Decide on a pruning policy before the first public release |
| Release asset layout the downloader expects: `dla-<target>.tar.gz` with the binary at the archive root, a `.sha256` file whose first field is the hex digest, `SHA256SUMS` | unverified | Defined by the release workflow and tested only against a locally built fake release | Push a pre-release tag and run the downloader against the real GitHub release |
| Plugin directory submission: no README link or icon is checked by default validation but is an error under the directory profile | unverified | Advisory notes from `claude plugin validate` | README now exists; add an icon and run the validator with the directory profile before any directory submission |

## Open design questions

| Item | Status | Evidence | Resolution |
|---|---|---|---|
| Privilege tiers 1 to 4 are not yet defined (to be written in `safety-and-effects.md` in M3). The `privilege_ceiling` setting defaults to 4 until they are | unverified | The setting's description defers to that document and defines no tier itself | Define the tiers in M3, then revisit the default (a lower default may be safer) and the setting's description |
| Source of the session ID for `dla run init`: `CLAUDE_SESSION_ID` or `CLAUDE_CODE_SESSION_ID` | verified | The Bash tool receives no such variable, but the skill body substitutes `${CLAUDE_SESSION_ID}` (observed: a skill body containing `${CLAUDE_SESSION_ID}` rendered the session ID, while `env \| grep SESSION` in the Bash tool printed nothing). The CLI still reads both variables and falls back to `unknown` | Resolved: SKILL.md passes `--session-id ${CLAUDE_SESSION_ID}` explicitly |
| Checklist answers accept any evidence ID (M6) | unverified | `dla ledger resolve intake.* --status answered --evidence E<n>` is accepted with any evidence ID, whatever kind of evidence it names | Bind each intake item to the evidence kinds that may answer it and refuse the rest |
| No file locking for concurrent ledger writers (M4) | unverified | Ledger appends from two processes at once can interleave | Per-worker ledger slices plus `dla ledger merge` (M4) remove the concurrent writers |

## Benchmark and evaluation (M1)

| Item | Status | Evidence | Resolution |
|---|---|---|---|
| Runs without dla (the `no-skill` and `prose` ablations) have no ledger, so graders read `findings.json` instead | verified | `bench/GRADING.md` ("`findings.json` (runs without dla)"); both graders ran on the fixture results in `bench/runner/testdata/` and produced the same metrics | Grading reads `ledger.jsonl` when it exists and `findings.json` otherwise. |
| Grading contract clarified after the two graders (`dla-bench grade` and `bench/runner/grade.py`) disagreed on the same results directory | corrected | The two graders produced different `metrics.json` on the same results; the contract rule is in `bench/GRADING.md` | The contract was fixed first, then both graders were changed to match; CI runs both on `bench/runner/testdata/` and compares them |
| Benchmark isolation: each child runs under Claude Code's Bash sandbox with Bash allowed inside it (`autoAllowBashIfSandboxed`), no network, writes confined to the working copy, `cache` and `tmp`; the file tools and the sandbox are denied the repository, the answer keys, the results directory and the parent configuration. Sibling working copies under the work root remain readable but hold no answers | verified | Settings in `bench/runner/src/run.rs` (`sandbox_settings`); `bench/README.md`, section "Isolation and cost"; the runner's `--probe-prompt` option exists to check the sandbox with a custom prompt | Documented, including the known gaps (parallel runs can read each other's working copies; the sandbox does not confine plugin hooks) |
| Plugin eval constraints: a plugin directory containing hard-linked files is refused, so cargo's `target/` must live outside the plugin directory | verified | `claude plugin eval` refused the repository while `target/` held hard links; it ran after the build moved to another directory | CI builds with `CARGO_TARGET_DIR` under the runner temp directory and copies only the binary to `target/debug/dla` |
| Plugin eval constraints: running as root, the eval's working directory was read-only here | unverified | Observed in this build environment (root user); not seen on CI's non-root runner | Run the eval job on the CI runner and note the result here |
| Plugin eval constraints: the plugin's `bin/` was not on `PATH` inside the eval | unverified | Observed in this environment's eval run; the README's 2.1.91 floor assumes `bin/` is on `PATH` in normal sessions | The skill and fixtures call `dla` through the shim; recheck on a normal session and on CI |
| Plugin eval constraints: the CI eval job is informational until `dla ledger confirm` exists (M3), needs the `ANTHROPIC_API_KEY` repository secret (it skips with a notice when absent) and takes an optional `DLA_BENCH_MODEL` repository variable for the model | unverified | `.github/workflows/ci.yml`, job `plugin-eval`; never run with a real key in CI | Run the job once with the secret set; add a score threshold when the confirm step exists |
| Privilege tier definitions (1 to 4) are still open; the benchmark's headless consent grants `1-4` | unverified | Same open item as under "Open design questions"; `bench/runner/src/run.rs` sets `DLA_CONSENT_TIERS=1-4` for plugin ablations | Define the tiers in M3 and revisit what the benchmark grants |
| Fixture caveat: the sawtooth period of `python-graph-churn` and the 70/30 split of `python-ingest-two-leaks` depend on CPython object sizes | unverified | Object sizes differ between CPython versions and platforms | Re-measure on the Python version the benchmark runs under, and loosen the key if the numbers move |
| Fixture caveat: the JVM fixture drivers print heap-after-GC checkpoints, which are the growth signal the driver gives a run | unverified | The checkpoints depend on the JVM's collector and flags | Check that the growth is still visible when the JDK or collector changes |

## Verification backlog

Detector rule IDs, tool flags and fingerprints that came from memory or secondary sources.
None of these may appear in a runtime reference as established fact until its row is closed.

### JVM

| Item | Status | Evidence | Resolution |
|---|---|---|---|
| SpotBugs `OS_OPEN_STREAM*` rule IDs | unverified | Not checked against the SpotBugs bug descriptions | Read the bug-descriptions page for the pinned version |
| SpotBugs `ODR_OPEN_DATABASE_RESOURCE*` rule IDs | unverified | As above | As above |
| SpotBugs `SIC_*` (inner class that could be static) rule IDs | unverified | As above | As above |
| Error Prone `ClassCanBeStatic` | unverified | Not checked for current name and default severity | Read the Error Prone bug-pattern page |
| Error Prone `ThreadLocalUsage` | unverified | As above | As above |
| PMD `CloseResource` | unverified | Not checked for current rule name and category | Read the PMD rule page |
| Sonar `S2694` | unverified | Not checked | Read the Sonar rule page |
| Sonar Kotlin rule for `GlobalScope` use | unverified | The rule ID is not known | Search the Sonar Kotlin rule list |
| Kotlin compiled continuation field names (`L$0`, `label`, `completion`) | unverified | Seen in older compiler output only | Run `javap -p` on a build with Kotlin 2.x and record the names |
| Whether async-profiler native-memory stacks include Java frames | unverified | Not tested | Run the profiler in native-memory mode on a sample app and inspect the output |
| Retention-path fingerprints marked uncertain in the Java and Kotlin references (for example continuation classes in a path) | unverified | Marked with a question mark in the source notes | Capture real paths from a leaking sample and replace the guesses |

### Python

| Item | Status | Evidence | Resolution |
|---|---|---|---|
| pylint `W1516` | unverified | Not checked for name and message | Read the pylint message reference |
| pylint `W1517` | unverified | As above | As above |
| ruff `SIM115` | unverified | Not checked for current behaviour | Read the ruff rule page |

### Rust

| Item | Status | Evidence | Resolution |
|---|---|---|---|
| clippy `mem_forget` (lint group and default level) | unverified | Not checked | Read the clippy lint list for the pinned version |
| clippy `let_underscore_future` | unverified | As above | As above |
| Retention-path fingerprints marked uncertain in the Rust reference | unverified | Marked with a question mark in the source notes | Capture real output from a leaking sample |

### Go

| Item | Status | Evidence | Resolution |
|---|---|---|---|
| gocritic `deferInLoop` | unverified | Not checked for name and availability | Read the gocritic checker list |
| `goref` against Go 1.27 binaries | unverified | Documented support ends at 1.26 | Run it on a binary built with 1.27 |
| Retention-path fingerprints marked uncertain in the Go reference | unverified | Marked with a question mark in the source notes | Capture real output from a leaking sample |

### Native

| Item | Status | Evidence | Resolution |
|---|---|---|---|
| Clang Static Analyzer leak checker IDs (including `unix.Malloc` and `cplusplus.NewDeleteLeaks`) | unverified | Not checked against the checker list | Read the analyzer checker documentation |
| CodeQL C++ leak query IDs (including `cpp/memory-never-freed`) | unverified | Not checked | Read the CodeQL query help |
| cppcheck leak rule IDs (including `memleak`) | unverified | Not checked | Read the cppcheck error list |
| Infer Pulse leak issue names | unverified | Not checked | Read the Infer issue-type list |

### Node.js

| Item | Status | Evidence | Resolution |
|---|---|---|---|
| Retention-path fingerprints marked uncertain in the Node reference | unverified | Marked with a question mark in the source notes | Capture real memlab output from a leaking sample |
