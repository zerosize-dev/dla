---
name: detect-leak-anywhere
description: Diagnose memory leaks in any runtime. Use when someone reports growing memory, a rising RSS, an out-of-memory kill or OOMKilled pod, a heap that never shrinks, a slow leak, file descriptor or thread growth, or asks to find, prove or rule out a leak in JVM/Kotlin, Go, Rust, Node.js/TypeScript, Python or native/FFI code, in a repository, a running process or captured heap dumps. Diagnosis only; it never edits project source.
hooks:
  PreToolUse:
    - matcher: "Write|Edit|NotebookEdit"
      hooks:
        - type: command
          command: "\"${CLAUDE_PLUGIN_ROOT}/bin/dla\" gate write-hook"
          timeout: 10
          onFailure: block
    - matcher: "Read|Grep|Glob"
      hooks:
        - type: command
          command: "\"${CLAUDE_PLUGIN_ROOT}/bin/dla\" gate read-hook"
          timeout: 10
          onFailure: block
    - matcher: "Bash"
      hooks:
        - type: command
          command: "\"${CLAUDE_PLUGIN_ROOT}/bin/dla\" gate bash-hook"
          timeout: 10
          onFailure: block
  Stop:
    - hooks:
        - type: command
          command: "\"${CLAUDE_PLUGIN_ROOT}/bin/dla\" gate stop-hook"
          timeout: 10
---

# Detect Leak Anywhere (DLA)

DLA diagnoses memory leaks. It measures, attributes and reports, and it never edits project source. A run is a directory `.dla/<run-id>/` that only the `dla` binary writes. Whether something counts as a finding, a label or a strong finding is settled by the ledger rules inside `dla`, never by how persuasive an explanation sounds. Work at high effort: the job is verification.

## Non-negotiables (MUST)

1. Never edit project source, and never write under `.dla/` by hand, except the harness driver file `dla harness gen --driver` tells you to edit (`.dla/<run>/harness/H<n>/driver/`). State moves only through `dla` commands. The ledger, slices, state, coverage manifest and report carry integrity seals, so a hand edit is detected and the report says INCOMPLETE; the write hook also refuses every other file under `.dla/` (trend, series, gate, bundle, intervene, harness output, evidence), which no seal covers.
2. Ask the consent question once for the whole run (tiers in `references/safety-and-effects.md`). A command above the consented tier is refused; never work around a refusal by running the tool yourself.
3. An archetype row or a detector hit only opens a hypothesis. Only `dla ledger test`, then `confirm`, makes a finding. `label` and `strong` need the evidence the ledger names, and DLA never picks a label for you.
4. Each growth series gets a non-leak sibling (`N-*`) and every hypothesis states what would refute it.
5. The report comes from `dla report` alone; show it exactly as printed. `dla run status` and `dla gate check` are the gates: their `missing` and `blocking` lists name each open item and the fixing command. Never present an INCOMPLETE run as finished.

## Invoking the binary

While the plugin is enabled, `dla` is on the Bash tool's PATH (Claude Code 2.1.91 or later); confirm with `command -v dla`, else use `${CLAUDE_PLUGIN_ROOT}/bin/dla`. The Bash tool gets no plugin environment variables, so this skill passes values as arguments. If `dla` says no binary was found, restart the session or follow the path it prints. `dla --help` and `dla <cmd> --help` give the exact flags of the installed release; never guess a flag. When a command named in these files does not exist, record that item `unresolved` with reason `tool_missing` and a remedy naming the command. Do not invent a stand-in.

Each command prints one small JSON object and a pointer to the full record. A refusal names what is missing and the command that fixes it: run that command.

## Stage checklist

Copy this into your notes. Details for each stage are in `references/stages.md`. Run `dla gate check --stage <s>` at the end of a stage (stages are cumulative; it lists what is missing and records the pass).

```
- [ ] 1. Intake checklist and consent            gate: intake
- [ ] 2. Probe                                   gate: probe
- [ ] 3. Static sweep and holder census          gate: sweep
- [ ] 4. Classify growth (harness, trend)        gate: classify
        the pass unlocks references/<lang>-archetypes.md and known-leaks.md; read them now, not before
- [ ] 5. Confirm; then label each confirmed leak with its evidence
        dla ledger label F --suggest, then --as <ID>; nothing fits: --as UNCLASSIFIED --note   gate: confirm
- [ ] 6. Attribute; explain each confirmed leak  gate: attribute
        dla ledger explain F ...; strong comes last, only when time allows: a second independent
        class from a dump's retention path or `dla intervene`, then dla ledger strong F --evidence E
- [ ] 7. Rule out non-leaks                      gate: rule_out
- [ ] 8. Verify every confirmed finding, in every path (not only with workers):
        dla bundle F, one dla:verifier subagent, dla ledger verify --label
        with workers: dla ledger merge first
        dla gate check --final      (passes, or every item has a reason and a next step)
- [ ] 9. dla report, then revisit what is unresolved
```

Rules that stay in force at every stage:

- Intake: items `intake.target`, `intake.scope`, `intake.symptom`, `intake.runtimes`, `intake.build`, `intake.workload`, `intake.consent`, `intake.budget` are answered with `dla ledger resolve <item> --status answered --evidence E1`, or `--status unresolved --reason <r> --remedy "<text>"`. `dla run init` may report a `consent` object from an unattended runner: ask nothing and cite its evidence. Otherwise `dla consent --tiers <list> --ceiling ${user_config.privilege_ceiling}`; a second consent is refused.
- Measure first and explain afterwards. Harness, trend, alloc-test and the plateau verdict decide leak versus non-leak; a runtime-heap sample alone is only a lead.
- Confirmation needs passing growth tied to the finding (its probe, a column at its site, or a column listing it as a candidate), a holder, a refuted `N-*` sibling and a `file:line` (artifact runs: an owner and no site). Growth of another holder or of the whole process (`os.rss`) does not confirm a particular holder: `growth-not-tied`. When `confirm` prints a warning that the tied column observed another holder, run the `ledger annotate --holder --evidence` command it prints.
- Attribute with `dla summarize <heap-graph|type-histogram-diff|alloc-stack-diff|non-heap> <files...>` on the captures `dla harness run` (`--capture`, `--tool`) or `dla attach` wrote; `references/backends.md` says which op reads which file and which tool it needs. Cite the evidence it returns. A column confirms only the holder it is tied to (the output's `tie`: holder, site or type); `--finding` never forces a tie. An empty or failed-gate result never refutes a leak, and a missing tool is a coverage row, not a finding. Native growth under a JNI boundary reaches a managed finding only through `dla correlate <H>`; `dla probe` known-leak verdicts are context, never findings.
- A candidate that could not be exercised is never confirmed or refuted: `dla ledger resolve <F> --as suspected_static_only|unresolved --reason <r> --remedy "<text>"`. Reasons: `not_exercised`, `no_harness`, `env_unavailable`, `build_fails`, `privilege_declined`, `tool_missing`, `time_budget`.
- A `time-budget` refusal whose fix names a smaller command (fewer `--iterations`, a smaller `--step`, fewer `--extra`): run that command. When the deadline has passed or nothing smaller fits, resolve the open items as `unresolved` with reason `time_budget` and a remedy, then report. A result marked `partial run` is inconclusive: it never confirms or refutes.
- A refused command is fixed with the command the refusal prints, never retried in a loop.
- If a run is interrupted, `dla run abort --reason "<why>"` stops what DLA started and undoes what it attached.

## When nothing fits

UNCLASSIFIED is a result, not a failure; use it only when no archetype fits the mechanism. When the best fit is refused for missing evidence, run the command the refusal names once, then label again. If it repeats, or no archetype fits, record `dla ledger label F --as UNCLASSIFIED --note "<holder and mechanism in your own words>"`. A finding with a holder and mechanism and no ID is a success, reported like a labelled one. Never force a label: it counts as a mistake. `dla ledger label F --as <ID>` refuses without the required evidence, a holder the archetype fits, and a refuted look-alike sibling; it lists what is missing, or the runtime's rows when the ID has none (choose one, or UNCLASSIFIED; `--note` only to keep the ID, and it must say why none of the runtime's rows fits). `dla ledger label F --suggest` lists each archetype of the finding's side with its summary, what is met and what is missing, then UNCLASSIFIED: compare the summaries with your mechanism. Findings of one leak: `dla ledger duplicate F --of G` merges F into the labelled G. `dla gate check --final` needs a label decision for each confirmed finding (an ID, or UNCLASSIFIED with a note).

When you annotate a site or writer, take `file:line` from `grep -n` or `cat -n` output, never from counting an unnumbered listing.

## Before you stop

- `dla gate check --final` passes, or every open item has a reason and a next step (`dla ledger resolve ... --as unresolved --reason <r> --remedy "<text>"`). Every confirmed leak has an explanation and a verification record.
- Run a harness `dla harness gen` wrote (`dla harness run`) before closing any finding of its runtime as `no_harness`, `not_exercised` or `time_budget`. If it cannot run, record the blocker on `intake.workload`; if only the user can unblock it (services, credentials, data), ask the user before closing, not after the report.
- If the report's `Measured:` line says nothing was measured, say first and plainly that the run measured nothing, so it neither shows nor rules out a leak, and name what would let it measure (the harness and what it needs).
- If `dla report` shows INCOMPLETE, say so plainly and list what is still open. Never write "done" over it.
- A Stop hook may block your stop while `gate check --final` fails. Do not rely on it: it blocks the first two stop attempts, lets the third go if nothing changed in the ledger, and stamps the run INCOMPLETE, and older Claude Code versions do not run it at all.
- Never edit `.dla` files to make a check pass. The write hook refuses them (the harness driver excepted), the integrity seals on the protected files detect an edit that gets past it, the report names it, and `dla run reseal --reason` is for a change made outside `dla` that you disclose.
- DLA does not call a leak confirmed because a pattern looks familiar. When it suggests a repair it names only the class of repair, for example deregistering the listener on dispose.

## Router: pick the entry path before anything else

The target is what the request names (a path, a pid or port, dump files). Otherwise it is the working directory when that holds source code (`git ls-files`, or the files there): you were started inside the project, so a request that only describes a service or symptom ("the gateway's memory climbs") means this repository. That is not a guess; take the repository path. Ask once ("Which repository, running process or heap dump files?") only when the working directory holds no source and the request names nothing, or the request names a target that is not here. When nobody can answer (an unattended run, or no reply), create the run with `dla run init --entry symptom`, record `dla ledger resolve intake.target --status unresolved --reason env_unavailable --remedy "<name the repository, pid or dump files>"`, run `dla report` and stop. Never invent a target that is neither named nor in the working directory.

| Path | When | Start with | Stages |
|---|---|---|---|
| repository | a source tree in the working directory, or a path/repo given | `dla run init --entry repository [--scope module\|service --scope-path <dir> --exclude <p>]` | all (stages.md 1-9) |
| process | a pid, process name or port of a running local process | `dla run init --entry process` | 1 (consent tier 2+), 2, 3 (`dla census` and `dla sweep` for the runtime; the gates require both), 4 with `dla sample --pid` / `dla attach`, 5-9 |
| artifact | heap dump files, no live process | `dla run init --entry artifact --artifact <path>...` | 1, 6 (`dla summarize heap-graph` once over ALL dumps), `dla gate check --stage classify` (it unlocks `references/<lang>-archetypes.md` and `known-leaks.md`), 5, 8, 9 |
| symptom | only "memory grows", nothing named and no source here | ask once, then re-route; else stop as above | 1, then the path the answer picks |

Add `--budget ${user_config.time_budget} --session-id ${CLAUDE_SESSION_ID}` to `dla run init`. Per-path rules (process consent tiers, artifact series rules, scope) are in `references/stages.md`; failures and the INCOMPLETE stamp in `references/recovery.md`; hooks and integrity in `references/safety-and-effects.md`; per-runtime notes in `references/<jvm|go|rust|node|python|native>.md`; the summarize backends and `dla intervene` in `references/backends.md`; the records and report lines in `references/evidence-and-report.md`.

## Dispatch: one runtime in the core, several in workers

Run stages 1 and 2 (intake, consent, probe, `dla install`) in the core first.

- **One runtime** in scope: the core runs stages 3 to 8 itself. No dispatch.
- **Two or more runtimes** (repository path): `dla run dispatch --workers jvm,rust` registers one worker per runtime and prints, for each, the facts to hand over: run id, worker name, deadline, reference path, scope and the command prefix `dla --run <id> --worker <name>`.
  1. Start one `dla:worker` subagent per runtime IN ONE MESSAGE: one Agent call per worker, each with `run_in_background: false`, so they run in parallel and the turn waits for all of them. Each prompt holds exactly the printed facts for that worker and nothing else. Workers label, explain and (time allowing) strengthen their own findings.
  2. The core writes no findings and runs no stage commands while workers run. Never end your turn while a worker runs: an unattended session has no later turn and stops background agents when it ends. If an Agent call returns at once ("launched in the background"), wait in this turn: `sleep 60; dla run status` until every worker shows `done` or its deadline has passed.
  3. `dla ledger merge`. It copies the slices, dedupes equal (site, holder) findings, opens items for contradictions, runtimes without a worker and workers that did not finish (the run is then INCOMPLETE). Safe to repeat; killed or stalled workers: `references/recovery.md`.
  4. Then step 8 of the checklist for every confirmed finding, as in a single-runtime run.

## Verification: every confirmed finding, in every path

For each confirmed finding `F` (core, after the merge when there are workers): `dla bundle F` prints a directory and a manifest sha256. Start one `dla:verifier` subagent (`run_in_background: false`) given only that directory, the run id, `F` and the sha. It names the site, holder and mechanism from the evidence and code excerpts alone, picks an ID or UNCLASSIFIED, and runs `dla ledger verify`. A disagreement (site, holder or ID) opens an open item to resolve; the report shows `Verified: no` for a confirmed finding nobody verified. Verification does not block the report, but a run that skips it says so. If the `dla:verifier` agent does not exist, write "not independently verified: no verifier agent" in your answer.
