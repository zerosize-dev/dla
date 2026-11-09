# DLA report

Run: `20261109-205707-d8f8`

Status: COMPLETE

Entry: artifact (`dumps/rfq-1c9e.hprof.gz`, `dumps/rfq-7f3a.hprof.gz`, `dumps/rfq-b04d.hprof.gz`)

Scope: repository; excluded: none; out of scope: none

Findings: 2 enumerated, 2 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 0

Residual: not measured

Coverage: not measured

## Findings

### F1: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `holder:static:quotes.RfqDesk.OPEN` 792288 bytes/dump (steady, gate pass, E7)
- Evidence classes: growth, retention
- Holder: static/global
- Owner: `quotes.RfqDesk.OPEN`
- Region: heap
- Shape: steady
- Sibling hypotheses: F1.n1
- Supporting evidence: E6, E7

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E8
  - Ruled out by: dump-plateau (leak) E8
  - Hypothesis: N-WARM: OPEN is a warm-up fill that plateaus
  - Refuted if: retained size and count keep growing strictly across all three dumps
  - Contradicting evidence: E8
  - Originally supported by: E7

## Blind spots and unknowns

### Coverage gaps

None recorded.

### Runtime blind spots

None recorded.

### Entry points not exercisable

None recorded.

### Evidence limitations and unknowns

- E4: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E5: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E6: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E7: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E7: limitation: the dumps are assumed to come from one process (checked: HPROF version, identifier size, timestamps, class set)
- E8: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | — |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | — |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | — |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | — |
| `intake.build` | Does the project build locally? | answered | E1 | — |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | — |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E1 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 736af8540afbcb1367081b0737b7c74e196ce349 | — |
| E2 | run-config | static | dla 0.1.0 | repo 736af8540afbcb1367081b0737b7c74e196ce349 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 736af8540afbcb1367081b0737b7c74e196ce349 | probe/probe.json |
| E4 | retention-path | dump | dla-hprof 0.1.0 | artifact dumps/rfq-7f3a.hprof.gz (bf1e8bc2efeb) | summaries/rfq-7f3a-bf1e8bc2.json |
| E5 | retention-path | dump | dla-hprof 0.1.0 | artifact dumps/rfq-1c9e.hprof.gz (70b81e368f2a) | summaries/rfq-1c9e-70b81e36.json |
| E6 | retention-path | dump | dla-hprof 0.1.0 | artifact dumps/rfq-b04d.hprof.gz (400a2c3bae52) | summaries/rfq-b04d-400a2c3b.json |
| E7 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S1.json (5773c796276c) | series/S1.json |
| E8 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S1.json (5773c796276c) | series/S1.json |
