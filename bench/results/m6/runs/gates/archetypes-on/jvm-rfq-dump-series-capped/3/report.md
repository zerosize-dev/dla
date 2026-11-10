# DLA report

Run: `20261110-180625-75a4`

Status: COMPLETE

Entry: artifact (`dumps`)

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

None recorded.

### Runtime blind spots

None recorded.

### Entry points not exercisable

None recorded.

### Evidence limitations and unknowns

- E3: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E4: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E5: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E6: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E6: limitation: the dumps are assumed to come from one process (checked: HPROF version, identifier size, timestamps, class set)
- E7: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | — |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | unresolved | — | env_unavailable: no source available; artifact-only run on dumps/ |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | — |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | — |
| `intake.build` | Does the project build locally? | unresolved | — | env_unavailable: no source available; artifact-only run on dumps/ |
| `intake.workload` | What can exercise the suspected path? | unresolved | — | env_unavailable: no source available; artifact-only run on dumps/ |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 9e92b75647b905f3966a81ba7e96330b1b65e5a7 | — |
| E2 | run-config | static | dla 0.1.0 | repo 9e92b75647b905f3966a81ba7e96330b1b65e5a7 | — |
| E3 | retention-path | dump | dla-hprof 0.1.0 | artifact dumps/rfq-e5d2.hprof.gz (c6ca006b714a) | summaries/rfq-e5d2-c6ca006b.json |
| E4 | retention-path | dump | dla-hprof 0.1.0 | artifact dumps/rfq-3a86.hprof.gz (cd3c459808ba) | summaries/rfq-3a86-cd3c4598.json |
| E5 | retention-path | dump | dla-hprof 0.1.0 | artifact dumps/rfq-90cb.hprof.gz (eb2161cdce49) | summaries/rfq-90cb-eb2161cd.json |
| E6 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S1.json (6d38913f1923) | series/S1.json |
| E7 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S1.json (6d38913f1923) | series/S1.json |
