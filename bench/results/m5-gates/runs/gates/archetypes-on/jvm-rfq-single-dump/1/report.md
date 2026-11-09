# DLA report

Run: `20261109-201054-9b5a`

Status: COMPLETE

Entry: artifact (`dumps`)

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
- Hypothesis: RfqDesk.OPEN static collection retains ~27k objects (3000 RfqDesk$Rfq) (1.5MB retained); may be leak or warm-up fill
- Refuted if: OPEN count/retained plateau or shrink across later dumps
- Reason: not_exercised
- Remedy: provide 2+ more heap dumps of the same process taken after more traffic, then rerun dla summarize heap-graph over all dumps
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
| `intake.build` | Does the project build locally? | unresolved | — | env_unavailable: source not available; provide source/build to map holder to file:line |
| `intake.workload` | What can exercise the suspected path? | unresolved | — | env_unavailable: no live process or source; provide more dumps of the same process |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo e6a62ee783058f619348066ebe1c2f0b65016eea | — |
| E2 | run-config | static | dla 0.1.0 | repo e6a62ee783058f619348066ebe1c2f0b65016eea | — |
| E3 | retention-path | dump | dla-hprof 0.1.0 | artifact dumps/rfq-1c9e.hprof.gz (70b81e368f2a) | summaries/rfq-1c9e-70b81e36.json |
