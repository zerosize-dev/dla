---
name: detect-leak-anywhere
description: Diagnose memory leaks in any runtime. Use when someone reports growing memory, a rising RSS, an out-of-memory kill or OOMKilled pod, a heap that never shrinks, a slow leak, file descriptor or thread growth, or asks to find, prove or rule out a leak in JVM/Kotlin, Go, Rust, Node.js/TypeScript, Python or native/FFI code. Diagnosis only; it never edits project source.
---

# Detect Leak Anywhere (DLA)

DLA diagnoses memory leaks. It measures, attributes and reports; it never edits project source. A run is a directory `.dla/<run-id>/` that only the `dla` binary writes. What counts as a finding is decided by the ledger rules in `dla`, not by how convincing a story sounds.

Run this skill at high effort (`/effort high` or above): the work is verification, and shortcuts show up as INCOMPLETE reports.

## Invoking the binary

The plugin's `bin/` directory is on the Bash tool's PATH while the plugin is enabled (Claude Code 2.1.91 or later), so call `dla` directly. Check with `command -v dla`. If that finds nothing, call it by its absolute path: `${CLAUDE_PLUGIN_ROOT}/bin/dla`. The binary is downloaded by a SessionStart hook; if it reports that no binary was found, restart the session or follow the path it prints. `dla --help` lists the commands the installed release has. If a command named below is missing, record that stage item as `unresolved` with reason `tool_missing` and a remedy naming the command; do not improvise a substitute.

Every `dla` command prints one small JSON object and a pointer to the full record. A refusal names what is missing and the command that fixes it: run that command.

## Non-negotiables (MUST)

1. Run the stages in order. `dla gate check --stage <stage>` must pass before the next one.
2. Change hypothesis state only with `dla ledger ...`. Files under the run directory are written by `dla` alone.
3. An archetype match opens a hypothesis. Only `dla ledger confirm` can turn it into a finding.
4. For every growth series, record at least two hypotheses, one of them a non-leak (`N-*`), each with the result that would refute it.
5. Produce the report only with `dla report`, and show its output unchanged.

## Stage checklist

Copy this into your notes and tick it off as you go.

```
- [ ] 1. Intake checklist
- [ ] 2. Probe
- [ ] 3. Static sweep
- [ ] 4. Classify growth
- [ ] 5. Confirm
- [ ] 6. Attribute and explain
- [ ] 7. Rule out non-leaks
- [ ] 8. Gate check
- [ ] 9. Report and revisit
```

1. **Intake checklist.** Ask the user what you cannot read from the repository, then record answers.
   `dla run init --budget 60m` creates the run. Answer each item with evidence:
   `dla ledger resolve intake.scope --status answered --evidence E1 --note "<text>"`.
   Items: `intake.target`, `intake.scope`, `intake.symptom`, `intake.runtimes`, `intake.build`, `intake.workload`, `intake.consent`, `intake.budget`. Use `--status unresolved --reason <r> --remedy "<text>"` when the user cannot answer.
2. **Probe.** `dla probe` records versions, tools and what each runtime can measure here. Missing tools become coverage gaps, not silent skips.
3. **Static sweep.** `dla sweep` runs every available detector and counts what it found. Then do a holder census yourself, without any list of known patterns: enumerate every process-lifetime holder (statics, singletons, module globals, registries, spawned units, acquired handles) with its writers and removers. Record each candidate:
   `dla ledger add --origin <detector:backend/rule|census> --evidence E4 --site file:line`.
4. **Classify growth.** Measure before explaining. `dla sample`, then `dla trend` (region and shape per series) and `dla gap --residual` (growth nothing explains yet). Then `dla gate check --stage classify`. Pattern references for each runtime are read after this gate passes, and not before.
5. **Confirm.** For each growth series, add a non-leak sibling:
   `dla ledger add --sibling-of F3 --origin hypothesis --hypothesis N-WARM --refute-if "<result that would refute it>" --evidence E7`.
   Run the harness (`dla workload`), then `dla ledger confirm F3 --evidence E9,E10`. Confirmed needs a growth series that passes the slope gate, `file:line`, a holder, and one refuted non-leak sibling.
6. **Attribute and explain.** Collect a retention path or allocation stack, then `dla ledger label F3 --archetype <ID>` (the label is earned by evidence kinds; otherwise it stays UNCLASSIFIED) and `dla ledger explain F3 --text "<lifecycle story>" --tag inferred`. Name the missing pairing (what should have released, removed or stopped), not a weak-reference workaround. For `strong`, add an independent second evidence class (retention or intervention in a scratch copy), then `dla ledger strong F3`. For each confirmed finding run `dla bundle F3` and give the directory to the `dla:verifier` agent; it sees no label.
7. **Rule out non-leaks.** Test the siblings: allocator retention (`N-ALLOC`), deferred collection (`N-HEADROOM`), warm-up caches (`N-WARM`), pool high-water marks (`N-HWM`), workload scaling (`N-SCALE`), intended retention (`N-INTENT`), and the instrument itself (`N-OBSERVER`). Record each result: `dla ledger refute F3.n1 --evidence E12` (needs a discriminator evidence record).
8. **Gate check.** `dla gate check --final`. Work every open item it lists. If something cannot be done, record it:
   `dla ledger resolve F3 --as unresolved --reason time_budget --remedy "<what to do next>"`.
   Reasons: `not_exercised`, `no_harness`, `env_unavailable`, `build_fails`, `privilege_declined`, `tool_missing`, `time_budget`.
9. **Report and revisit.** `dla report` renders `report.md` from the ledger and prints it; show it unchanged. Then revisit every `unresolved` and `suspected_static_only` finding while budget remains: `dla run status` lists them.

## Splitting by runtime

For a polyglot target, start one `dla:worker` per runtime. A worker works one runtime through stages 2 to 8 and writes only with `dla ledger --worker <name> ...`; you merge and report.

## When nothing fits

UNCLASSIFIED is a correct answer. State the holder and the mechanism in your own words. A forced archetype label counts as an error in DLA's evaluation; an evidenced UNCLASSIFIED counts as a success. Report it as a novel mechanism, with the same weight as any labelled finding.

## Before you stop

Run `dla gate check --final`. Work every open item it lists, or record it as `unresolved` with a reason and a remedy. The gate output decides when the run is done. If `dla report` says INCOMPLETE, say so in your answer and list the open items; do not describe an INCOMPLETE run as finished.

## What DLA will not do

Edit project source, write into the run directory by hand, or call a leak confirmed because a pattern looks familiar. Fix suggestions are limited to naming the repair class (for example: deregister the listener on dispose).
