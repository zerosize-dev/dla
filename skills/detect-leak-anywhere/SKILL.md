---
name: detect-leak-anywhere
description: Diagnose memory leaks in any runtime. Use when someone reports growing memory, a rising RSS, an out-of-memory kill or OOMKilled pod, a heap that never shrinks, a slow leak, file descriptor or thread growth, or asks to find, prove or rule out a leak in JVM/Kotlin, Go, Rust, Node.js/TypeScript, Python or native/FFI code. Diagnosis only; it never edits project source.
---

# Detect Leak Anywhere (DLA)

DLA diagnoses memory leaks. It measures, attributes and reports, and it never edits project source. A run is a directory `.dla/<run-id>/` that only the `dla` binary writes. Whether something counts as a finding is settled by the ledger rules inside `dla`, never by how persuasive an explanation sounds.

Work at high effort (`/effort high` or above). The job is verification, and cutting corners shows up as an INCOMPLETE report.

## Invoking the binary

While the plugin is enabled, its `bin/` directory is on the Bash tool's PATH (Claude Code 2.1.91 or later), so call `dla` directly and confirm with `command -v dla`. The Bash tool receives no plugin environment variables, so this skill passes the values it needs as arguments. If `dla` is not found, use the absolute path `${CLAUDE_PLUGIN_ROOT}/bin/dla`. A SessionStart hook downloads the binary; when `dla` says none was found, restart the session or follow the path it prints. `dla --help` lists the commands the installed release has. When a command named below does not exist yet, record that stage item as `unresolved` with reason `tool_missing` and a remedy that names the command. Do not invent a stand-in.

Each `dla` command prints one small JSON object plus a pointer to the full record. A refusal names what is missing and the command that fixes it, so run that command.

## Command surface

```
dla run init | run status | run abort
dla consent --tiers <list of 1-4> [--raw-values] [--ceiling N]
dla probe [--build]
dla sweep --lang <l>
dla census --lang <l>
dla workload discover
dla harness gen --lang <l> [--workload <id>] [--driver] [--iter-arg "<..{n}..>"] | harness run <H> [--mode fresh|repeated|both]
dla sample --pid <P> [--duration 60s] [--counters auto|none|<list>] [--tree]
dla trend <series> [--column <name>] [--against <series> --finding <F>]
dla gap [<series>] [--residual]
dla alloc-test <H> [--finding <F>] [--column <name>] [--purge <kind>] [--extra <k>]
dla summarize <heap-graph|type-histogram-diff|alloc-stack-diff|non-heap> [--backend auto|<name>] <artifact>
dla install <backend>
dla ledger add|annotate|test|confirm|refute|strong|label|explain|resolve|verify
dla ledger merge
dla bundle <finding>
dla gate check --stage <s> | --final
dla report
```

## Non-negotiables (MUST)

1. Go through the stages in order. `dla gate check --stage <stage>` has to pass before the next stage starts.
2. Hypothesis state moves only through `dla ledger ...`. Nothing under the run directory is edited by hand.
3. Matching an archetype merely opens a hypothesis. Only `dla ledger confirm` produces a finding, and it requires the finding to be `under_test` first (`dla ledger test F3`).
4. Each growth series gets at least two hypotheses, and one of them is a non-leak explanation (`N-*`). Every hypothesis states the outcome that would refute it.
5. The report comes from `dla report` alone. Show its output exactly as printed.

## Stage checklist

Copy this into your notes and tick items off.

```
- [ ] 1. Intake checklist
- [ ] 2. Probe
- [ ] 3. Static sweep and holder census
- [ ] 4. Classify growth
- [ ] 5. Confirm
- [ ] 6. Attribute and explain
- [ ] 7. Rule out non-leaks
- [ ] 8. Final gate
- [ ] 9. Report and revisit
```

Every stage ends with `dla gate check --stage <stage>`, using the stage name that `dla` prints.

1. **Intake checklist.** Ask the user for what the repository cannot tell you.
   Create the run with `dla run init --budget ${user_config.time_budget} --session-id ${CLAUDE_SESSION_ID}` (Claude Code fills in both values before you read this).
   Ask the consent question once, as a single question for the whole run covering privilege tiers 1 to 4 (what each allows is in `references/safety-and-effects.md`), offering only tiers up to the configured privilege ceiling. If `dla run init` reports a `consent` object, an unattended runner has already answered: ask nothing and cite its evidence for `intake.consent`. Otherwise record the answer with `dla consent --tiers <list> --ceiling ${user_config.privilege_ceiling}`, adding `--raw-values` only if the user explicitly agrees to see raw values. `dla consent` refuses a second consent in a run; never ask a second consent question.
   Answer each checklist item with evidence: `dla ledger resolve intake.scope --status answered --evidence E1 --note "<text>"`.
   Items: `intake.target`, `intake.scope`, `intake.symptom`, `intake.runtimes`, `intake.build`, `intake.workload`, `intake.consent`, `intake.budget`. If the user cannot answer one, use `--status unresolved --reason <r> --remedy "<text>"`.
2. **Probe.** `dla probe` records runtimes, versions, tools on the PATH, privilege facts, test runners and the library flags a later harness will switch on. Every backend the runtimes could use but the machine lacks is recorded in coverage as `missing`, with an install hint. After tier-1 consent, run `dla probe --build` to learn whether the project builds (`--build-timeout <secs>`, default 300); a failed build is a coverage gap (`build_fails`), not a reason to stop. For each missing backend that probe reports, run `dla install <backend>` once tier-1 consent exists. If the install fails, the backend stays a recorded gap and the command prints the exact manual command; never retry in a loop and never install into project source.
3. **Static sweep and holder census.** For every runtime that probe found (`jvm`, `go`, `rust`, `node`, `python`, `native`), run both, in this order:
   `dla sweep --lang <l>` runs the static backends that are available and adds one candidate per rule hit (origin `detector:<backend>/<rule>`). A backend that is absent or fails shows in coverage as `missing` or `blocked`; do not hand-run it as a substitute. A detector may carry a suggested archetype; it is only a tag and opens nothing.
   `dla census --lang <l>` adds one candidate per process-lifetime holder (statics, singletons, module globals, registries, spawned units, acquired handles; origin `census`). Then `dla workload discover` lists the tests, entry points and dev servers a loop could run, ranked by the entry points they exercise; entry points with no runnable path come back as not exercisable, each with a reason.
   The census output lists heuristic writers and removers for each holder, matched by name. Treat them as leads. For every census holder, open the source and check them: find the code that adds to the holder and the code that removes from it, including ones the name match missed or wrongly paired. Record what you find with `dla ledger annotate F<n> --writer file:line --remover file:line --note "<text>"` (both flags repeat; at least one of writer, remover or note is required). Annotating changes no state. A holder with writers and no remover is the lead to carry into stage 4.
4. **Classify growth.** Measure first and explain afterwards. These commands build and run project code in a scratch copy under the run directory and need tier-1 consent (`references/safety-and-effects.md`).
   `dla harness gen --lang <l> [--workload <id>]` copies the project into `.dla/<run>/harness/H1/work/` (project source is never touched), finds the workload's iteration-count argument, derives probes from the census candidates and writes the manifest. It runs nothing. If it refuses with `iteration-arg-unknown`, pass `--iter-arg "--iterations {n}"` (or the workload's own spelling). For a library with no main, or to compare fresh keys (every unit uses a new key) with repeated keys (a few keys reused), add `--driver [--package <dir>]`: it prints the file to edit and its hooks. Replace the body of each `DLA-HOOK` (keep the comment, leave the rest alone) and derive ids from `key`, so that a repeated key reproduces an earlier request. Go probes of package variables work only in driver mode.
   `dla harness run H1` builds once and measures every point. In driver mode add `--mode both` to get one series per key mode. It prints the series ids (`S1`, ...) and the next command. A harness run or an alloc-test can take several minutes: give that Bash call a timeout of 600000 ms rather than the default.
   `dla trend S1` gives the shape and the slope gate for every column. Read the `passed` columns (the full report is `trend/T<n>.json`) and, for each, its `candidates`: the findings tied to it by finding id or site. A passed column with no candidate is growth nothing explains yet. `dla trend S1 --against S2 --finding F3` compares a fresh series with a repeated one and records whether the growth follows new keys.
   `dla sample --pid <P>` samples a process that is already running (tier 2 unless DLA started it). Its runtime heap columns are not read after a collection, so treat them as a lead and confirm with the harness. `dla gap S1` splits whole-process memory into runtime-accounted and unexplained; `dla gap S1 --residual` totals each region's growth, subtracts what confirmed findings explain and opens an open item for the rest. Runtime pattern references are read only once this stage's gate has passed. Column names, the gate and the residual rule are in `references/evidence-and-report.md`.
5. **Confirm.** For each passed column tied to a candidate `F3`:
   - Add a non-leak sibling: `dla ledger add --sibling-of F3 --origin hypothesis --hypothesis N-WARM --refute-if "<result that would refute it>" --evidence E7`. The hypothesis must start with one of `N-ALLOC`, `N-HEADROOM`, `N-WARM`, `N-HWM`, `N-SCALE`, `N-INTENT`, `N-OBSERVER`.
   - `dla ledger test F3 --evidence E9`, citing the harness's `workload` evidence (from `harness gen` or `harness run`). A leak-side finding needs its sibling first.
   - `dla alloc-test H1 --finding F3` re-runs the harness with extra points and a purge and records a `discriminator` whose `favors` is `leak`, `non-leak` or `inconclusive`.
   - Refute the loser with that evidence: `dla ledger refute F3.n1 --evidence E13` when it favors leak, `dla ledger refute F3 --evidence E13` when it favors non-leak. Refuting needs a discriminator that opposes the finding.
   - `dla ledger annotate F3 --holder <holder>` when the finding has no holder (`--owner`, `--region` and `--site` fill other gaps), then `dla ledger confirm F3 --evidence E11`, citing the `dla trend` evidence that passed. Confirmation needs a growth series that passes the slope gate, a `file:line`, a holder, and one non-leak sibling that has been refuted. A refusal names what is missing.
6. **Attribute and explain.** Gather a retention path or an allocation stack with `dla summarize <heap-graph|type-histogram-diff|alloc-stack-diff|non-heap> [--backend auto|<name>] <artifact>`; if the backend is missing, `dla install <backend>` fetches it. Then `dla ledger label F3 --archetype <ID>` (the label is earned from the kinds of evidence present; otherwise the finding stays UNCLASSIFIED) and `dla ledger explain F3 --text "<lifecycle story>" --tag inferred`. Name the missing pairing, meaning what should have released, removed or stopped, and not a weak-reference workaround. For `strong`, add a second, independent class of evidence (retention, or an intervention in a scratch copy), then `dla ledger strong F3`. For each confirmed finding run `dla bundle F3` and hand the directory to the `dla:verifier` agent, which never sees the label.
7. **Rule out non-leaks.** Test the sibling explanations: allocator retention (`N-ALLOC`, checked with `dla alloc-test`, which trims or purges and looks for a plateau), deferred collection (`N-HEADROOM`), warm-up caches (`N-WARM`), pool high-water marks (`N-HWM`), workload scaling (`N-SCALE`), intended retention (`N-INTENT`) and the instrument itself (`N-OBSERVER`).
   Growth that a discriminator shows to be non-leak ends as refuted: `dla alloc-test H1 --finding F3`, then `dla ledger refute F3 --evidence E12`. A candidate the harness exercised whose column did not grow gets the same test (`dla alloc-test H1 --finding F5`), and is refuted when the plateau verdict favors non-leak. When no column tied to any candidate grew, one unbound `dla alloc-test H1` judges the whole workload; if it favors non-leak, refute each exercised candidate with that evidence. A candidate the harness could not exercise is not confirmed or refuted: `dla ledger resolve F6 --as suspected_static_only --reason not_exercised --remedy "<text>" --evidence E4` (reason `not_exercised`, `no_harness` or `build_fails`), or `--as unresolved` when no static evidence exists. A failed build or a run without usable points (`harness-failed`) confirms nothing.
8. **Final gate.** `dla gate check --final`. Work through every open item it lists. When something cannot be done, record that: `dla ledger resolve F3 --as unresolved --reason time_budget --remedy "<what to do next>"`.
   Reasons: `not_exercised`, `no_harness`, `env_unavailable`, `build_fails`, `privilege_declined`, `tool_missing`, `time_budget`.
9. **Report and revisit.** `dla report` writes `report.md` from the ledger and prints it; show it exactly as printed. While budget remains, return to the findings that are `unresolved` or `suspected_static_only`. `dla run status` lists them under `revisit` (at most 10 entries, with `revisit_total` giving the full count); each entry carries `id`, `state`, `reason` and `remedy`. If you decide to stop early, `dla run abort` closes the run.

## Splitting by runtime

A polyglot target gets one `dla:worker` per runtime. Each worker takes one runtime through stages 2 to 8 and writes only with `dla ledger --worker <name> ...`. You combine the slices with `dla ledger merge` and then produce the report.

## When nothing fits

UNCLASSIFIED is a legitimate outcome. Describe the holder and the mechanism in plain words of your own. Forcing a pattern label onto a leak it does not fit is counted as a mistake in DLA's evaluation, whereas an UNCLASSIFIED result that is backed by evidence is counted as a success. Report such a leak as a novel mechanism, giving it the same prominence as any labelled finding.

## Before you stop

Run `dla gate check --final` and deal with each open item it lists, either by finishing it or by marking it `unresolved`, giving the reason and the next step. The gate result, not your own judgement, says when the run is done. If `dla report` shows INCOMPLETE, say so in your answer and list what is still open. Never present an INCOMPLETE run as finished.

## What DLA will not do

DLA does not edit project source, does not write into the run directory by hand, and does not call a leak confirmed because a pattern looks familiar. When it suggests a repair it names only the class of repair, for example deregistering the listener on dispose.
