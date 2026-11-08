# DLA report

Run: `20261108-204325-ac98`

Status: INCOMPLETE (7 open)

Entry: symptom

Scope: repository; excluded: none; out of scope: none

Findings: 0 enumerated, 0 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 0

Residual: not measured

Coverage: not measured

## Open items

- **intake.scope**: checklist item intake.scope is unanswered: Does the investigation cover the whole repository, one module or one service, and what is excluded? Fix: `dla ledger resolve intake.scope --status answered --evidence E1`
- **intake.symptom**: checklist item intake.symptom is unanswered: What growth was observed, where, and over what time? Fix: `dla ledger resolve intake.symptom --status answered --evidence E1`
- **intake.runtimes**: checklist item intake.runtimes is unanswered: Which language runtimes and processes are involved? Fix: `dla ledger resolve intake.runtimes --status answered --evidence E1`
- **intake.build**: checklist item intake.build is unanswered: Does the project build locally? Fix: `dla ledger resolve intake.build --status answered --evidence E1`
- **intake.workload**: checklist item intake.workload is unanswered: What can exercise the suspected path? Fix: `dla ledger resolve intake.workload --status answered --evidence E1`
- **intake.consent**: checklist item intake.consent is unanswered: Which intrusive steps has the user allowed, and are raw values permitted in evidence? Fix: `dla ledger resolve intake.consent --status answered --evidence E1`
- **intake.budget**: checklist item intake.budget is unanswered: How much wall-clock time may the investigation use? Fix: `dla ledger resolve intake.budget --status answered --evidence E1`

## Findings

No findings to report.

## Ruled out

None.

## Blind spots and unknowns

### Coverage gaps

No coverage manifest recorded.

### Runtime blind spots

No coverage manifest recorded.

### Entry points not exercisable

No coverage manifest recorded.

### Evidence limitations and unknowns

None recorded.

## Checklist

1 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | unresolved | — | env_unavailable: Name the repository, pid/port of the running process, or heap dump files; the working directory holds only a dashboard excerpt (memory-excerpt.txt), no source. |
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
| E1 | run-config | static | dla 0.1.0 | repo b61698079ca4a074bd9de9f64590212187783a38 | — |
| E2 | run-config | static | dla 0.1.0 | repo b61698079ca4a074bd9de9f64590212187783a38 | — |
