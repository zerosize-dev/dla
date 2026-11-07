# DLA report

Run: `20261107-195308-bd45`

Status: INCOMPLETE (14 open)

Findings: 16 enumerated, 2 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 2

Residual: not measured

## Open items

- **F2**: finding F2 is candidate and has no disposition. Fix: `dla ledger resolve F2 --as unresolved --reason time_budget --remedy "<text>"`
- **F3**: finding F3 is candidate and has no disposition. Fix: `dla ledger resolve F3 --as unresolved --reason time_budget --remedy "<text>"`
- **F4**: finding F4 is candidate and has no disposition. Fix: `dla ledger resolve F4 --as unresolved --reason time_budget --remedy "<text>"`
- **F5**: finding F5 is candidate and has no disposition. Fix: `dla ledger resolve F5 --as unresolved --reason time_budget --remedy "<text>"`
- **F6**: finding F6 is candidate and has no disposition. Fix: `dla ledger resolve F6 --as unresolved --reason time_budget --remedy "<text>"`
- **F7.n1**: finding F7.n1 is candidate and has no disposition. Fix: `dla ledger resolve F7.n1 --as unresolved --reason time_budget --remedy "<text>"`
- **F7.n2**: finding F7.n2 is candidate and has no disposition. Fix: `dla ledger resolve F7.n2 --as unresolved --reason time_budget --remedy "<text>"`
- **F8**: finding F8 is candidate and has no disposition. Fix: `dla ledger resolve F8 --as unresolved --reason time_budget --remedy "<text>"`
- **F9**: finding F9 is candidate and has no disposition. Fix: `dla ledger resolve F9 --as unresolved --reason time_budget --remedy "<text>"`
- **F10**: finding F10 is candidate and has no disposition. Fix: `dla ledger resolve F10 --as unresolved --reason time_budget --remedy "<text>"`
- **F11**: finding F11 is candidate and has no disposition. Fix: `dla ledger resolve F11 --as unresolved --reason time_budget --remedy "<text>"`
- **F12**: finding F12 is candidate and has no disposition. Fix: `dla ledger resolve F12 --as unresolved --reason time_budget --remedy "<text>"`
- **F13**: finding F13 is candidate and has no disposition. Fix: `dla ledger resolve F13 --as unresolved --reason time_budget --remedy "<text>"`
- **F14**: finding F14 is candidate and has no disposition. Fix: `dla ledger resolve F14 --as unresolved --reason time_budget --remedy "<text>"`

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Site: `src/heartbeat.js:15`
- Writers: `src/heartbeat.js:15`
- Annotation: same site as F7; no clearInterval/ clearTimeout in repo
- Suggested archetype (unconfirmed): L-WORKER
- Reason: tool_missing
- Remedy: duplicate site of F7; fold into F7 confirmation
- Supporting evidence: E6

### F7: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `src.heartbeat.startHeartbeat`
- Region: non-heap
- Site: `src/heartbeat.js:15`
- Writers: `src/heartbeat.js:15`, `src/gateway.js:26`
- Annotation: setInterval created per session in accept(); timer handle stored on session.heartbeat; no clearInterval anywhere in src (grep). Closure captures session -> socket, recent, topics. Session.close() and Gateway.release() never clear it
- Sibling hypotheses: F7.n1, F7.n2
- Reason: tool_missing
- Remedy: installed dla lacks test/confirm/refute/gate/harness/sample/trend/summarize; once available run: ledger test F7; confirm with the scratch-copy control (clearInterval in Session.close -> heap plateau ~4MB vs +8.2MB/wave) as evidence
- Supporting evidence: E7

### F2: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.metrics`
- Region: heap
- Site: `src/gateway.js:13`
- Supporting evidence: E7

### F3: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.router`
- Region: heap
- Site: `src/gateway.js:14`
- Supporting evidence: E7

### F4: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.sessions`
- Region: heap
- Site: `src/gateway.js:15`
- Writers: `src/gateway.js:22`
- Removers: `src/gateway.js:32`
- Annotation: heuristic name match over the project's sources
- Annotation: sessions Map entry deleted on release; verified, not the leak
- Supporting evidence: E7

### F5: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: registration-table
- Owner: `src.gateway.Gateway.accept`
- Region: heap
- Site: `src/gateway.js:24`
- Writers: `src/gateway.js:25`, `src/session.js:12`, `src/session.js:54`
- Removers: `load.js:50`, `src/gateway.js:31`, `src/session.js:66`
- Annotation: heuristic name match over the project's sources
- Supporting evidence: E7

### F6: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: registration-table
- Owner: `src.gateway.Gateway.accept`
- Region: heap
- Site: `src/gateway.js:25`
- Writers: `src/gateway.js:24`, `src/session.js:12`, `src/session.js:54`
- Removers: `load.js:50`, `src/gateway.js:31`, `src/session.js:66`
- Annotation: heuristic name match over the project's sources
- Supporting evidence: E7

### F7.n1: Hypothesis N-HEADROOM (UNCLASSIFIED)

- State: candidate
- Evidence classes: static
- Sibling hypothesis of: F7
- Hypothesis: N-HEADROOM
- Refuted if: heapUsed after forced double gc still rises per wave (observed: 3.5->12.1->...->44.7 MB, +8.2MB/wave)
- Supporting evidence: E7

### F7.n2: Hypothesis N-WARM (UNCLASSIFIED)

- State: candidate
- Evidence classes: static
- Sibling hypothesis of: F7
- Hypothesis: N-WARM
- Refuted if: growth stops after initial waves (observed: linear over 5 waves, not a plateau)
- Supporting evidence: E7

### F8: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: container-field
- Owner: `src.loopback.LoopbackSocket.written`
- Region: heap
- Site: `src/loopback.js:11`
- Writers: `src/loopback.js:17`
- Annotation: heuristic name match over the project's sources
- Supporting evidence: E7

### F9: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: container-field
- Owner: `src.metrics.Metrics.counters`
- Region: heap
- Site: `src/metrics.js:7`
- Writers: `src/metrics.js:12`
- Annotation: heuristic name match over the project's sources
- Supporting evidence: E7

### F10: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: container-field
- Owner: `src.router.Router.topics`
- Region: heap
- Site: `src/router.js:9`
- Writers: `src/router.js:16`, `src/router.js:19`
- Removers: `src/router.js:26`, `src/router.js:27`
- Annotation: heuristic name match over the project's sources
- Supporting evidence: E7

### F11: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: registration-table
- Owner: `src.router.Router.dispatch`
- Region: heap
- Site: `src/router.js:51`
- Writers: `src/session.js:38`
- Removers: `src/gateway.js:25`, `src/router.js:54`, `src/session.js:33`
- Annotation: heuristic name match over the project's sources
- Supporting evidence: E7

### F12: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.decoder`
- Region: heap
- Site: `src/session.js:15`
- Writers: `src/session.js:30`
- Annotation: heuristic name match over the project's sources
- Supporting evidence: E7

### F13: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.topics`
- Region: heap
- Site: `src/session.js:19`
- Supporting evidence: E7

### F14: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.recent`
- Region: heap
- Site: `src/session.js:20`
- Writers: `src/session.js:44`
- Removers: `src/session.js:45`
- Annotation: heuristic name match over the project's sources
- Supporting evidence: E7

## Ruled out

None.

## Blind spots and unknowns

### Coverage gaps

None recorded.

### Runtime blind spots

None recorded.

### Entry points not exercisable

- node: `src/loopback.js::LoopbackSocket.destroy`: no_harness
- node: `src/loopback.js::LoopbackSocket.write`: no_harness
- node: `src/metrics.js::Metrics.observe`: no_harness
- node: `src/router.js::Router.publish`: no_harness
- node: `src/router.js::Router.subscribe`: no_harness
- node: `src/router.js::Router.unsubscribe`: no_harness
- node: `src/session.js::Session.describe`: no_harness
- node: `src/session.js::Session.sendRaw`: no_harness

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
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | edge gateway Node service in src/ |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | whole repo, src/ + load.js |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | RSS grows per connect/disconnect wave, restart every 2 days |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | node >=22 |
| `intake.build` | Does the project build locally? | answered | E1 | plain JS, no build; npm test |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | load.js --iterations N |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 raw values |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 60m |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | — |
| E2 | run-config | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | probe/probe.json |
| E4 | probe | static | dla-probe 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | probe/probe.json |
| E5 | static-hit | static | eslint 10.14.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | sweep/eslint.json |
| E6 | static-hit | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | sweep/dla.json |
| E7 | census-holder | static | dla-census 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | census/node.json |
| E8 | workload | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | workload/workloads.json |
