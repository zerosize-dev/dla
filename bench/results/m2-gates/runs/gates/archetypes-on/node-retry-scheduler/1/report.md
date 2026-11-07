# DLA report

Run: `20261107-194008-ff36`

Status: COMPLETE

Findings: 8 enumerated, 8 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 8

Residual: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Site: `src/scheduler.js:32`
- Reason: tool_missing
- Remedy: installed dla lacks test/confirm/harness commands; not the cause per direct reading (bounded or released holder) unless F5 chain
- Supporting evidence: E5

### F2: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.pending.PendingList.head`
- Region: heap
- Site: `src/pending.js:7`
- Writers: `src/pending.js:15`, `src/pending.js:24`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: installed dla lacks test/confirm/harness commands; not the cause per direct reading (bounded or released holder) unless F5 chain
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.pending.PendingList.tail`
- Region: heap
- Site: `src/pending.js:8`
- Writers: `src/pending.js:16`, `src/pending.js:25`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: installed dla lacks test/confirm/harness commands; not the cause per direct reading (bounded or released holder) unless F5 chain
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.scheduler.RetryScheduler.pending`
- Region: heap
- Site: `src/scheduler.js:14`
- Writers: `src/scheduler.js:33`
- Removers: `src/scheduler.js:53`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: installed dla lacks test/confirm/harness commands; not the cause per direct reading (bounded or released holder) unless F5 chain
- Supporting evidence: E6

### F5: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.scheduler.RetryScheduler.latest`
- Region: heap
- Site: `src/scheduler.js:15`
- Writers: `src/scheduler.js:32`
- Annotation: heuristic name match over the project's sources
- Annotation: enqueue() sets record.previous=this.latest and latest=record; nothing ever clears previous or latest. Chain keeps every record and its job alive. Scratch measurement (outside project): heapUsed after gc 11.7->44.5MB over 5 batches with pending=0 inflight=0, chain length 2902->14510; setting latest=null dropped heap to 3.7MB.
- Reason: tool_missing
- Remedy: dla lacks 'ledger test/confirm' and harness/trend commands in this release; rerun on a release with them. Evidence is manual measurement.
- Supporting evidence: E6

### F6: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.scheduler.RetryScheduler.deadLetters`
- Region: heap
- Site: `src/scheduler.js:16`
- Writers: `src/scheduler.js:72`
- Removers: `src/scheduler.js:73`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: installed dla lacks test/confirm/harness commands; not the cause per direct reading (bounded or released holder) unless F5 chain
- Supporting evidence: E6

### F7: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.scheduler.RetryScheduler.inflight`
- Region: heap
- Site: `src/scheduler.js:17`
- Writers: `src/scheduler.js:22`
- Removers: `src/scheduler.js:66`, `src/scheduler.js:71`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: installed dla lacks test/confirm/harness commands; not the cause per direct reading (bounded or released holder) unless F5 chain
- Supporting evidence: E6

### F8: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: static/global
- Owner: `src.scheduler.shared`
- Region: heap
- Site: `src/scheduler.js:90`
- Writers: `src/scheduler.js:93`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: installed dla lacks test/confirm/harness commands; not the cause per direct reading (bounded or released holder) unless F5 chain
- Supporting evidence: E6

## Ruled out

None.

## Blind spots and unknowns

### Coverage gaps

None recorded.

### Runtime blind spots

None recorded.

### Entry points not exercisable

- node: `src/pending.js::PendingList.[Symbol.iterator]`: no_harness

### Evidence limitations and unknowns

- E4: partial coverage
- E4: limitation: rxjs-x rules need rxjs to resolve from the project's node_modules for type information
- E6: partial coverage
- E6: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E6: limitation: writers and removers are textual name matches
- E7: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E7: limitation: workloads were discovered by reading files; none was run

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | webhook dispatcher RetryScheduler (node) |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | from task text/repo; consent per E2 |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | from task text/repo; consent per E2 |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | from task text/repo; consent per E2 |
| `intake.build` | Does the project build locally? | answered | E1 | from task text/repo; consent per E2 |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | from task text/repo; consent per E2 |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E1 | from task text/repo; consent per E2 |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | from task text/repo; consent per E2 |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | — |
| E2 | run-config | static | dla 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | probe/probe.json |
| E4 | static-hit | static | eslint 10.14.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | sweep/eslint.json |
| E5 | static-hit | static | dla 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | sweep/dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | census/node.json |
| E7 | workload | static | dla 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | workload/workloads.json |
