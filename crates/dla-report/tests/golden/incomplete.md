# DLA report

Run: `20260101-000000-abcd`

Status: INCOMPLETE (11 open)

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 2 enumerated, 0 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 0

Residual: heap 0.5 MB/h unattributed of 10 total (threshold 1); off-heap 6 MB/h unattributed of 8 total (threshold 1) ABOVE THRESHOLD, see O1

Coverage: jvm no iterations, 1 of 2 entry points exercised

## Open items

- **F1**: finding F1 is candidate and has no disposition. Fix: `dla ledger resolve F1 --as unresolved --reason time_budget --remedy "<text>"`
- **F2**: finding F2 is under_test and has no disposition. Fix: `dla ledger resolve F2 --as unresolved --reason time_budget --remedy "<text>"`
- **O1**: open item O1: Residual heap growth unexplained | pipe. Fix: `dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"`
- **intake.budget**: checklist item intake.budget is unanswered: How much wall-clock time may the investigation use? Fix: `dla ledger resolve intake.budget --status answered --evidence E1`
- **20260101-000000-abcd**: no consent is recorded for this run. Fix: `dla consent --tiers 1,2`
- **20260101-000000-abcd**: no probe evidence is recorded. Fix: `dla probe`
- **jvm/census**: no census has run for runtime jvm. Fix: `dla census --lang jvm`
- **jvm/sweep**: no sweep has run for runtime jvm. Fix: `dla sweep --lang jvm`
- **F2**: finding F2 is under test and has no result. Fix: `dla ledger confirm F2 or dla ledger refute F2 (with --evidence)`
- **F2**: finding F2 has no non-leak sibling hypothesis. Fix: `dla ledger add --sibling-of F2 --origin hypothesis --hypothesis N-WARM --refute-if "<result>" --evidence E..`
- **off-heap**: residual growth in off-heap (6.000 MB/h, threshold 1.000) has no resolved open item. Fix: `dla ledger resolve O1 --by <F> or dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"`

## Findings

### F1: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Site: `src/a.py:1`
- Supporting evidence: E2

### F2: **UNCLASSIFIED**

- State: under_test
- Evidence classes: static
- Supporting evidence: E2

## Ruled out

None.

## Blind spots and unknowns

### Coverage gaps

- jvm/jfr: blocked (privilege declined)
- jvm/mat: missing (no JDK on PATH). Install hint: install a JDK 17 or newer

### Runtime blind spots

- jvm: native memory outside NMT is not visible

### Entry points not exercisable

- jvm: `Worker|batch`: needs a queue

### Evidence limitations and unknowns

None recorded.

## Checklist

7 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | — |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | — |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | — |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | — |
| `intake.build` | Does the project build locally? | answered | E1 | — |
| `intake.workload` | What can exercise the suspected path? | unresolved | — | no_harness: ask the owner for a load script |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E1 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | **MISSING** | — | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo abc123 | — |
| E2 | static-hit | static | dla 0.1.0 | repo abc123 | — |
