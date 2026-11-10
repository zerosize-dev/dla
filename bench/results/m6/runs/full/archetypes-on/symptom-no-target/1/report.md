# DLA report

Run: `20261110-204959-75b8`

Status: COMPLETE

Entry: symptom

Scope: repository; excluded: none; out of scope: none

Findings: 0 enumerated, 0 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 0

Residual: not measured

Coverage: not measured

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

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | unresolved | — | env_unavailable: Name the repository, pid/port of the running process, or heap dump files to diagnose |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | unresolved | — | env_unavailable: No target named and no source here; name the repository, pid/port or dump files |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | — |
| `intake.runtimes` | Which language runtimes and processes are involved? | unresolved | — | env_unavailable: No target named and no source here; name the repository, pid/port or dump files |
| `intake.build` | Does the project build locally? | unresolved | — | env_unavailable: No target named and no source here; name the repository, pid/port or dump files |
| `intake.workload` | What can exercise the suspected path? | unresolved | — | env_unavailable: No target named and no source here; name the repository, pid/port or dump files |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo b61698079ca4a074bd9de9f64590212187783a38 | — |
| E2 | run-config | static | dla 0.1.0 | repo b61698079ca4a074bd9de9f64590212187783a38 | — |
