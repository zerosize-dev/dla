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

## Research facts that are not verified by running them

These came from documentation or a changelog and could not be exercised in the build environment.
The remaining research facts were checked by running the Claude Code CLI.

| Item | Status | Evidence | Resolution |
|---|---|---|---|
| Plugin agent frontmatter: `permissionMode`, `hooks`, `mcpServers` and `initialPrompt` are ignored for plugin agents, and the other listed fields are honoured | unverified | Read in the Claude Code plugin and sub-agent docs. A run confirmed only that hooks in a plugin agent do not fire | Agents carry no hooks. Recheck the supported field list when raising the minimum Claude Code version |
| Hooks on Windows: shell-form hooks run in Git Bash when installed and in PowerShell otherwise; exec form needs a real executable | unverified | Read in the hooks docs; the build environment is Linux | Test on a Windows runner with and without Git Bash. If Git Bash is absent, `bash "..."` fails as a non-blocking error; consider a Node-based hook or two entries with `shell` set |
| Minimum Claude Code versions per feature (plugin options 2.1.83, plugin data 2.1.78, plugin `bin/` 2.1.91, plugin skill hooks 2.1.94, exec-form hook args 2.1.139, Stop-hook continuation cap 2.1.143, `${user_config.*}` shell-form rejection 2.1.207, `options` enum 2.1.271, validating both manifests 2.1.289) | unverified | Mapped from changelog lines, not tested on old versions | README states 2.1.94 as the floor. Install the floor version in CI once and run the plugin checks against it |
| Minimum Node for memlab: 18 in practice (its browser driver requires 18; the core package declares 12.6) | unverified | Package `engines` fields; no memlab run | README recommends 20 or 22. Run a memlab scenario on the floor version before M-series work that depends on it |
| Minimum JDK for Eclipse MAT: 21 for the 1.17 stand-alone release | unverified | MAT download page and manifest; no MAT run | Run headless `ParseHeapDump.sh` on a JDK 21 install before relying on it |
| `claude plugin eval` case format and commands (needed for the evaluation suite) | unverified | Read in the plugin evals docs; needs model credentials, so not run | Create one case with `claude plugin eval init --bare` and run it when the suite starts |
| `onFailure: "block"` has no effect on Stop-type hooks, so Stop hooks always fail open | unverified | Docs statement; fail-open behaviour for crashes and non-zero exits was verified | Recheck on the version raised to in the future |
| Defaults for `userConfig` options are not exported to hooks, and the Bash tool receives no plugin variables | verified | Empirical hook environment dump and Bash tool `env` listing | The `dla` binary and the shim apply their own defaults and locate the data directory themselves |

## Items the plugin work flagged for verification

| Item | Status | Evidence | Resolution |
|---|---|---|---|
| Stage commands in the skill and agent bodies use provisional flag shapes: `dla probe`, `sweep`, `sample`, `trend`, `gap --residual`, `workload`, `gate check --stage/--final`, `ledger confirm/strong/refute/label/explain`, `bundle`, `ledger verify`, `ledger --worker` | unverified | None of these commands exist in M0 | Reconcile each with the real CLI in the milestone that implements it (M3 to M6). The skill tells the model to record `unresolved(tool_missing)` when a command is absent |
| The name `dla ledger refute` is a placeholder; the plan only says the refute command arrives in M3 | unverified | Chosen by the plugin author | Confirm or rename in M3 and update the skill and agents |
| Windows (Git Bash and PowerShell) and macOS behaviour of `bin/dla` and `hooks/fetch-dla.sh`, including bash 3.2 | unverified | Only Linux was tested locally; CI runs `tests/shim/run.sh` on all three systems | Confirm the CI shim job passes on macOS and Windows runners, then close |
| The real `dla` binary has not been run through the shim and downloader | unverified | The shim test used a fake payload because no binary existed | `tests/shim/run.sh` uses `target/debug/dla` automatically when it exists; confirm in CI |
| The worker agent's preload of the `dla:detect-leak-anywhere` skill resolves at runtime | unverified | `claude plugin validate` and the headless listing pass; no session was run | Run one agent invocation and check that the skill text is present |
| Downloaded binaries of older plugin versions are never pruned from the data directory | unverified | By design in M0 | Decide on a pruning policy before the first public release |
| Release asset layout the downloader expects: `dla-<target>.tar.gz` with the binary at the archive root, a `.sha256` file whose first field is the hex digest, `SHA256SUMS` | unverified | Defined by the release workflow and tested only against a locally built fake release | Push a pre-release tag and run the downloader against the real GitHub release |
| Plugin directory submission: no README link or icon is checked by default validation but is an error under the directory profile | unverified | Advisory notes from `claude plugin validate` | README now exists; add an icon and run the validator with the directory profile before any directory submission |

## Open design questions

| Item | Status | Evidence | Resolution |
|---|---|---|---|
| Privilege tiers 1 to 4 are not yet defined (to be written in `safety-and-effects.md` in M3). The `privilege_ceiling` setting defaults to 4 until they are | unverified | The setting's description names tiers but no document defines them | Define the tiers in M3, then revisit the default (a lower default may be safer) and the setting's description |
| Source of the session ID for `dla run init`: `CLAUDE_SESSION_ID` or `CLAUDE_CODE_SESSION_ID` | unverified | The CLI reads both, in that order, and falls back to `unknown`. The Bash tool receives neither plugin variables nor, as far as tested, a documented session variable | Find which variable, if either, Claude Code sets in the Bash tool environment, or pass the ID explicitly via the skill (the skill body can embed the session ID by substitution) |

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
