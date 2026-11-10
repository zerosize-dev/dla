# DLA report

Run: `20260101-000000-abcd`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 3 enumerated, 3 dispositioned; confirmed 1, strong 0, refuted 2, suspected 0, unresolved 0

Residual: not measured

Coverage: go 8000 iterations, 2 of 3 entry points exercised

## Findings

### F1: **UNCLASSIFIED**

- State: confirmed
- Growth: `probe:Registry.listeners` 412 bytes/n (steady, gate pass, E3)
- Evidence classes: context, growth, static
- Holder: registration-table
- Owner: `Bus.listeners`
- Region: heap
- Shape: steady
- Site: `src/bus.go:12`
- Verified: no
- Sibling hypotheses: F1.n1
- Supporting evidence: E6, E2, E3

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E4
  - Ruled out by: plateau+purge (leak) E4
  - Hypothesis: N-WARM: the listener table fills during warm-up
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E4
  - Originally supported by: E6
- **F2** (UNCLASSIFIED): discriminator E5
  - Ruled out by: plateau (non-leak) E5
  - Contradicting evidence: E5
  - Originally supported by: E6

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

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | — |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | — |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | — |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | — |
| `intake.build` | Does the project build locally? | answered | E1 | — |
| `intake.workload` | What can exercise the suspected path? | answered | E2 | — |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E1 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo abc123 | — |
| E2 | workload | test | dla 0.1.0 | repo abc123 | harness/H1/harness.json |
| E3 | growth-series | test | dla 0.1.0 | artifact series/S1.json (0123456789ab) | trend/T1.json |
| E4 | discriminator | test | dla 0.1.0 | repo abc123 | alloc-test/A1.json |
| E5 | discriminator | test | dla 0.1.0 | repo abc123 | alloc-test/A2.json |
| E6 | census-holder | static | scan 1.0 | repo abc123 | — |
