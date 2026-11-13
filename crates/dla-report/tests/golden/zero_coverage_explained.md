# DLA report

Run: `20260101-000000-abcd`

Status: INCOMPLETE (13 open)

Measured: 1 harness run, 0 series, 0 captures

Coverage warning: this run exercised 0 of 225 entry points (jvm), so findings outside them rest on reading the code and are hypotheses, not measured leaks. Cause: 750 iterations ran and none called an entry point, so the driver did not reach the code under test; 37 project classes loaded. To measure: dla harness gen --lang jvm --driver, fill the DLA-HOOK bodies of the harness it prints so the driver calls the entry points, then dla harness run <the harness it prints>.

Time: budget 1h, 0s used.

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 2 enumerated, 2 dispositioned; leak hypotheses: confirmed 0, strong 0, refuted 1, suspected 0, unresolved 0; non-leak explanations: confirmed 1, refuted 0, suspected 0, unresolved 0; no leak confirmed

Residual: not measured

Coverage: jvm 750 iterations, 0 of 225 entry points exercised, 37 project classes loaded

## Open items

- **intake.target**: checklist item intake.target is unanswered: Is the target a repository, a local process, an artifact, or only a symptom? Fix: `dla ledger resolve intake.target --status answered --evidence E1`
- **intake.scope**: checklist item intake.scope is unanswered: Does the investigation cover the whole repository, one module or one service, and what is excluded? Fix: `dla ledger resolve intake.scope --status answered --evidence E1`
- **intake.symptom**: checklist item intake.symptom is unanswered: What growth was observed, where, and over what time? Fix: `dla ledger resolve intake.symptom --status answered --evidence E1`
- **intake.runtimes**: checklist item intake.runtimes is unanswered: Which language runtimes and processes are involved? Fix: `dla ledger resolve intake.runtimes --status answered --evidence E1`
- **intake.build**: checklist item intake.build is unanswered: Does the project build locally? Fix: `dla ledger resolve intake.build --status answered --evidence E1`
- **intake.workload**: checklist item intake.workload is unanswered: What can exercise the suspected path? Fix: `dla ledger resolve intake.workload --status answered --evidence E1`
- **intake.consent**: checklist item intake.consent is unanswered: Which intrusive steps has the user allowed, and are raw values permitted in evidence? Fix: `dla ledger resolve intake.consent --status answered --evidence E1`
- **intake.budget**: checklist item intake.budget is unanswered: How much wall-clock time may the investigation use? Fix: `dla ledger resolve intake.budget --status answered --evidence E1`
- **20260101-000000-abcd**: no consent is recorded for this run. Fix: `dla consent --tiers 1,2`
- **20260101-000000-abcd**: no probe evidence is recorded. Fix: `dla probe`
- **jvm/census**: no census has run for runtime jvm. Fix: `dla census --lang jvm`
- **jvm/sweep**: no sweep has run for runtime jvm. Fix: `dla sweep --lang jvm`
- **jvm**: jvm: 750 iteration(s) ran and 0 of 225 entry points were exercised, 37 project classes loaded: the driver did not call the code under test, so the run says nothing about the entry points. Fix: `dla harness gen --lang jvm --driver, fill the DLA-HOOK bodies of the harness it prints so the driver calls the entry points, then dla harness run <the harness it prints>. If the driver cannot reach them, record why: dla ledger resolve intake.workload --status unresolved --reason not_exercised --remedy "<why no entry point can be called>"`

## Findings

No leak hypotheses to report; the confirmed findings are non-leak explanations, listed below.

## Explained, not leaks

1 non-leak explanation held: the growth covered has a benign cause (a warm-up, a bounded cache, load that scales with traffic, a high-water mark), so each is an explanation, not a leak and not a defect.

### F1.n1: Hypothesis N-WARM: jit (UNCLASSIFIED)

- State: confirmed
- Origin: `hypothesis`
- Explained by N-WARM (discriminator E2)
- Evidence classes: discriminator, static
- Sibling hypothesis of: F1
- Hypothesis: N-WARM: jit
- Refuted if: x
- Supporting evidence: E1, E2

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E2
  - Ruled out by: discriminator (non-leak) E2
  - Contradicting evidence: E2
  - Originally supported by: E1

## Blind spots and unknowns

### Coverage gaps

None recorded.

### Runtime blind spots

None recorded.

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
| E1 | static-hit | static | t 0 | repo abc | — |
| E2 | discriminator | test | t 0 | repo abc | — |
