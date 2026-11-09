# DLA report

Run: `20261109-205801-4692`

Status: COMPLETE

Entry: artifact (`dumps/rfq-1c9e.hprof.gz`)

Scope: repository; excluded: none; out of scope: none

Findings: 1 enumerated, 1 dispositioned; confirmed 0, strong 0, refuted 0, suspected 1, unresolved 0

Residual: not measured

Coverage: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: retention
- Holder: static/global
- Owner: `quotes.RfqDesk.OPEN`
- Region: heap
- Locus: app
- Hypothesis: quotes.RfqDesk.OPEN (static HashMap of RfqDesk$Rfq) retains ~1.58MB/27k objects; entries may never be removed
- Refuted if: later dumps show OPEN size plateaus (warm-up/high-water mark)
- Reason: not_exercised
- Remedy: provide 2+ more heap dumps of the same process taken after more traffic
- Supporting evidence: E4

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

- E4: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)

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
| E3 | probe | static | dla-probe 0.1.0 | repo e6a62ee783058f619348066ebe1c2f0b65016eea | probe/probe.json |
| E4 | retention-path | dump | dla-hprof 0.1.0 | artifact dumps/rfq-1c9e.hprof.gz (70b81e368f2a) | summaries/rfq-1c9e-70b81e36.json |
