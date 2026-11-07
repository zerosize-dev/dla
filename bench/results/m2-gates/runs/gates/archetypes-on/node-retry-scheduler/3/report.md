# DLA report

Run: `20261107-195146-bc49`

Status: INCOMPLETE (8 open)

Findings: 9 enumerated, 1 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 1

Residual: not measured

## Open items

- **F2**: finding F2 is candidate and has no disposition. Fix: `dla ledger resolve F2 --as unresolved --reason time_budget --remedy "<text>"`
- **F3**: finding F3 is candidate and has no disposition. Fix: `dla ledger resolve F3 --as unresolved --reason time_budget --remedy "<text>"`
- **F4**: finding F4 is candidate and has no disposition. Fix: `dla ledger resolve F4 --as unresolved --reason time_budget --remedy "<text>"`
- **F5**: finding F5 is candidate and has no disposition. Fix: `dla ledger resolve F5 --as unresolved --reason time_budget --remedy "<text>"`
- **F6**: finding F6 is candidate and has no disposition. Fix: `dla ledger resolve F6 --as unresolved --reason time_budget --remedy "<text>"`
- **F7**: finding F7 is candidate and has no disposition. Fix: `dla ledger resolve F7 --as unresolved --reason time_budget --remedy "<text>"`
- **F8**: finding F8 is candidate and has no disposition. Fix: `dla ledger resolve F8 --as unresolved --reason time_budget --remedy "<text>"`
- **F9**: finding F9 is candidate and has no disposition. Fix: `dla ledger resolve F9 --as unresolved --reason time_budget --remedy "<text>"`

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Site: `src/scheduler.js:32`
- Reason: tool_missing
- Remedy: installed dla lacks test/confirm/trend/gate/report; rerun with a full release to confirm F1 with N-WARM/N-ALLOC siblings and dla alloc-test
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: container-field
- Owner: `src.pending.PendingList.head`
- Region: heap
- Site: `src/pending.js:7`
- Writers: `src/pending.js:15`, `src/pending.js:24`
- Annotation: heuristic name match over the project's sources
- Supporting evidence: E7

### F3: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: container-field
- Owner: `src.pending.PendingList.tail`
- Region: heap
- Site: `src/pending.js:8`
- Writers: `src/pending.js:16`, `src/pending.js:25`
- Annotation: heuristic name match over the project's sources
- Supporting evidence: E7

### F4: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: container-field
- Owner: `src.scheduler.RetryScheduler.pending`
- Region: heap
- Site: `src/scheduler.js:14`
- Writers: `src/scheduler.js:33`
- Removers: `src/scheduler.js:53`
- Annotation: heuristic name match over the project's sources
- Supporting evidence: E7

### F5: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: container-field
- Owner: `src.scheduler.RetryScheduler.latest`
- Region: heap
- Site: `src/scheduler.js:15`
- Writers: `src/scheduler.js:32`
- Annotation: heuristic name match over the project's sources
- Supporting evidence: E7

### F6: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: container-field
- Owner: `src.scheduler.RetryScheduler.deadLetters`
- Region: heap
- Site: `src/scheduler.js:16`
- Writers: `src/scheduler.js:72`
- Removers: `src/scheduler.js:73`
- Annotation: heuristic name match over the project's sources
- Supporting evidence: E7

### F7: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: container-field
- Owner: `src.scheduler.RetryScheduler.inflight`
- Region: heap
- Site: `src/scheduler.js:17`
- Writers: `src/scheduler.js:22`
- Removers: `src/scheduler.js:66`, `src/scheduler.js:71`
- Annotation: heuristic name match over the project's sources
- Supporting evidence: E7

### F8: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: static/global
- Owner: `src.scheduler.shared`
- Region: heap
- Site: `src/scheduler.js:90`
- Writers: `src/scheduler.js:93`
- Annotation: heuristic name match over the project's sources
- Supporting evidence: E7

### F9: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: container-field
- Owner: `src.scheduler.RetryScheduler.latest`
- Region: heap
- Shape: steady
- Locus: app
- Site: `src/scheduler.js:32`
- Repair class: remove-entry
- Supporting evidence: E5, E7
- Explanation (inferred): enqueue() sets record.previous=this.latest and latest=record; unlink() never clears previous and nothing trims the chain, so every attempt record (with its job payload) stays reachable from scheduler.latest on the process-lifetime shared scheduler. Measured: heap +8.2MB/batch of 2000 jobs, chain length 2902->17614 with pending=0 inflight=0; setting latest=null drops heap 52.7->3.7MB.

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

- E5: partial coverage
- E5: limitation: rxjs-x rules need rxjs to resolve from the project's node_modules for type information
- E7: partial coverage
- E7: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E7: limitation: writers and removers are textual name matches
- E8: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E8: limitation: workloads were discovered by reading files; none was run

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | webhook dispatcher RetryScheduler (src/scheduler.js) |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | whole repo, node |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | heap grows per batch though pending/inflight empty |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | node >=22 |
| `intake.build` | Does the project build locally? | answered | E1 | no build, plain JS |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | node load.js --iterations N |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 raw |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 60m |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | — |
| E2 | run-config | static | dla 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | probe/probe.json |
| E4 | probe | static | dla-probe 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | probe/probe.json |
| E5 | static-hit | static | eslint 10.14.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | sweep/eslint.json |
| E6 | static-hit | static | dla 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | sweep/dla.json |
| E7 | census-holder | static | dla-census 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | census/node.json |
| E8 | workload | static | dla 0.1.0 | repo bf7ddeb22da6fa3bcc203a6dccb1d3271c699fa4 | workload/workloads.json |
