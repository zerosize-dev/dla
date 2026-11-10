# DLA report

Run: `20261110-164425-c6fa`

Status: COMPLETE

Entry: artifact (`dumps/rfq-1c9e.hprof.gz`)

Scope: repository; excluded: none; out of scope: none

Findings: 2 enumerated, 2 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 2

Residual: not measured

Coverage: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: retention
- Holder: static/global
- Owner: `quotes.RfqDesk.OPEN`
- Region: heap
- Locus: app
- Repair class: remove-entry
- Sibling hypotheses: F1.n1
- Reason: not_exercised
- Remedy: Take 2+ more heap dumps of the same process after more traffic and rerun dla summarize heap-graph over all dumps; then test/confirm F1
- Supporting evidence: E3

### F1.n1: Hypothesis OPEN is a bounded working set of in-flight RFQs (warm-up fill / steady state), not unbounded growth (UNCLASSIFIED)

- State: unresolved(not_exercised)
- Evidence classes: retention
- Sibling hypothesis of: F1
- Hypothesis: OPEN is a bounded working set of in-flight RFQs (warm-up fill / steady state), not unbounded growth
- Refuted if: later dumps show OPEN entry count still rising with completed/expired RFQs retained
- Reason: not_exercised
- Remedy: Same: needs a dump series to refute
- Supporting evidence: E3

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
| E1 | run-config | static | dla 0.1.0 | repo e6a62ee783058f619348066ebe1c2f0b65016eea | — |
| E2 | run-config | static | dla 0.1.0 | repo e6a62ee783058f619348066ebe1c2f0b65016eea | — |
| E3 | retention-path | dump | dla-hprof 0.1.0 | artifact dumps/rfq-1c9e.hprof.gz (70b81e368f2a) | summaries/rfq-1c9e-70b81e36.json |
