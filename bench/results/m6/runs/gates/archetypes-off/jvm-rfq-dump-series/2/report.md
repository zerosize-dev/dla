# DLA report

Run: `20261110-191835-9390`

Status: COMPLETE

Entry: artifact (`dumps/rfq-1c9e.hprof.gz`, `dumps/rfq-7f3a.hprof.gz`, `dumps/rfq-b04d.hprof.gz`)

Scope: repository; excluded: none; out of scope: none

Findings: 2 enumerated, 2 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 0

Residual: not measured

Coverage: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Static collection quotes.RfqDesk.OPEN retains quotes.RfqDesk$Rfq entries (with their byte[]/String payloads) that are added per request and never removed; retained size 788KB->1.58MB->2.37MB and dominated count 13.5k->27k->40.5k, linear across 3 dumps)
- Growth: `holder:static:quotes.RfqDesk.OPEN` 792288 bytes/dump (steady, gate pass, E6)
- Evidence classes: growth, retention
- Holder: static/global
- Owner: `quotes.RfqDesk.OPEN`
- Region: heap
- Shape: steady
- Verified: no
- Sibling hypotheses: F1.n1
- Supporting evidence: E5, E6
- Explanation (observed): Holder: static field quotes.RfqDesk.OPEN retains quotes.RfqDesk$Rfq objects; added per request, no remover found in the dumps, so count grows linearly (13.5k, 27k, 40.5k). No source, so no site.
- Explanation structure: adds: ; removal: not-found; evidence: E5

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E7
  - Ruled out by: dump-plateau (leak) E7
  - Hypothesis: N-WARM: OPEN is a warm-up fill that plateaus; not a leak
  - Refuted if: retained size and count keep growing strictly across all three dumps
  - Contradicting evidence: E7
  - Originally supported by: E6

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
| E3 | retention-path | dump | dla-hprof 0.1.0 | artifact dumps/rfq-7f3a.hprof.gz (bf1e8bc2efeb) | summaries/rfq-7f3a-bf1e8bc2.json |
| E4 | retention-path | dump | dla-hprof 0.1.0 | artifact dumps/rfq-1c9e.hprof.gz (70b81e368f2a) | summaries/rfq-1c9e-70b81e36.json |
| E5 | retention-path | dump | dla-hprof 0.1.0 | artifact dumps/rfq-b04d.hprof.gz (400a2c3bae52) | summaries/rfq-b04d-400a2c3b.json |
| E6 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S1.json (5773c796276c) | series/S1.json |
| E7 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S1.json (5773c796276c) | series/S1.json |
