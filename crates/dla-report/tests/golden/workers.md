# DLA report

Run: `20260101-000000-abcd`

Status: INCOMPLETE (22 open)

Entry: repository

Scope: module `services/api`; excluded: `services/api/generated`, `vendor`; out of scope: jvm 12 files, rust 1 file

Findings: 3 enumerated, 0 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 0

Residual: not measured

Coverage: not measured

## Open items

- **jvm/F1**: finding jvm/F1 is candidate and has no disposition. Fix: `dla ledger resolve jvm/F1 --as unresolved --reason time_budget --remedy "<text>"`
- **jvm/F1.n1**: finding jvm/F1.n1 is candidate and has no disposition. Fix: `dla ledger resolve jvm/F1.n1 --as unresolved --reason time_budget --remedy "<text>"`
- **rust/F2**: finding rust/F2 is candidate and has no disposition. Fix: `dla ledger resolve rust/F2 --as unresolved --reason time_budget --remedy "<text>"`
- **O1**: open item O1: worker rust did not finish (slice ends at line 3). Fix: `dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"`
- **O2**: open item O2: verifier disagrees with rust/F2 (site: finding native/pool.rs:7 vs verifier none; holder: finding native-handle/global-ref vs verifier thread-local); bundle aaaaaaaaaaaa. Fix: `dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"`
- **intake.target**: checklist item intake.target is unanswered: Is the target a repository, a local process, an artifact, or only a symptom? Fix: `dla ledger resolve intake.target --status answered --evidence E1`
- **intake.scope**: checklist item intake.scope is unanswered: Does the investigation cover the whole repository, one module or one service, and what is excluded? Fix: `dla ledger resolve intake.scope --status answered --evidence E1`
- **intake.symptom**: checklist item intake.symptom is unanswered: What growth was observed, where, and over what time? Fix: `dla ledger resolve intake.symptom --status answered --evidence E1`
- **intake.runtimes**: checklist item intake.runtimes is unanswered: Which language runtimes and processes are involved? Fix: `dla ledger resolve intake.runtimes --status answered --evidence E1`
- **intake.build**: checklist item intake.build is unanswered: Does the project build locally? Fix: `dla ledger resolve intake.build --status answered --evidence E1`
- **intake.workload**: checklist item intake.workload is unanswered: What can exercise the suspected path? Fix: `dla ledger resolve intake.workload --status answered --evidence E1`
- **intake.consent**: checklist item intake.consent is unanswered: Which intrusive steps has the user allowed, and are raw values permitted in evidence? Fix: `dla ledger resolve intake.consent --status answered --evidence E1`
- **intake.budget**: checklist item intake.budget is unanswered: How much wall-clock time may the investigation use? Fix: `dla ledger resolve intake.budget --status answered --evidence E1`
- **20260101-000000-abcd**: run is incomplete: workers did not finish: rust (slice ends at line 3). Fix: `let the unfinished workers end (dla --worker <name> ledger done), then dla ledger merge`
- **20260101-000000-abcd**: no consent is recorded for this run. Fix: `dla consent --tiers 1,2`
- **20260101-000000-abcd**: no probe evidence is recorded. Fix: `dla probe`
- **jvm/census**: no census has run for runtime jvm. Fix: `dla census --lang jvm`
- **jvm/sweep**: no sweep has run for runtime jvm. Fix: `dla sweep --lang jvm`
- **rust/census**: no census has run for runtime rust. Fix: `dla census --lang rust`
- **rust/sweep**: no sweep has run for runtime rust. Fix: `dla sweep --lang rust`
- **rust**: slice of worker rust has 4 line(s) and 3 merged. Fix: `dla ledger merge`
- **rust**: slice of worker rust ends in a torn line (the worker was killed mid-write). Fix: `dla ledger merge`

## Workers

- **jvm** (jvm): status done; 4 lines merged; slice done; torn tail: no
- **rust** (rust): status killed; 3 of 4 lines merged; slice not done; torn tail: yes, final line dropped

### Unmerged slices

- **rust**: 4 lines in ledger.rust.jsonl, 3 merged. Fix: `dla ledger merge`

## Findings

### jvm/F1: **UNCLASSIFIED**

- State: candidate
- Evidence classes: context
- Holder: static/global
- Owner: `Registry.listeners`
- Site: `src/Registry.java:42`
- Verified: agrees (verifier site `src/Registry.java:44`, holder static/global, label UNCLASSIFIED; bundle aaaaaaaaaaaa)
- Merged duplicates: rust/F1
- Sibling hypotheses: jvm/F1.n1
- Supporting evidence: E1

### jvm/F1.n1: Hypothesis N-WARM (UNCLASSIFIED)

- State: candidate
- Evidence classes: context
- Sibling hypothesis of: jvm/F1
- Hypothesis: N-WARM
- Refuted if: growth continues
- Supporting evidence: E1

### rust/F2: **UNCLASSIFIED**

- State: candidate
- Evidence classes: context
- Holder: native-handle/global-ref
- Site: `native/pool.rs:7`
- Verified: DISAGREES (site: finding `native/pool.rs:7` vs verifier none; holder: finding native-handle/global-ref vs verifier thread-local; bundle aaaaaaaaaaaa); finding mechanism: not stated; verifier mechanism: a thread local
- Supporting evidence: E1

## Merged duplicates

- **rust/F1** merged into **jvm/F1** (a duplicate of the same leak; its evidence E1 is kept on jvm/F1)

## Ruled out

None.

## Blind spots and unknowns

### Coverage gaps

None recorded.

### Runtime blind spots

- go: no worker covered this runtime

### Entry points not exercisable

None recorded.

### Evidence limitations and unknowns

None recorded.

## Checklist

0 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | **MISSING** | — | — |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | **MISSING** | — | — |
| `intake.symptom` | What growth was observed, where, and over what time? | **MISSING** | — | — |
| `intake.runtimes` | Which language runtimes and processes are involved? | **MISSING** | — | — |
| `intake.build` | Does the project build locally? | **MISSING** | — | — |
| `intake.workload` | What can exercise the suspected path? | **MISSING** | — | — |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | **MISSING** | — | — |
| `intake.budget` | How much wall-clock time may the investigation use? | **MISSING** | — | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo /work/project | — |
