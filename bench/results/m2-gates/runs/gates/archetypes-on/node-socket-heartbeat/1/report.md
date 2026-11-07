# DLA report

Run: `20261107-194121-6eb6`

Status: COMPLETE

Findings: 14 enumerated, 14 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 14

Residual: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Site: `src/heartbeat.js:15`
- Writers: `src/heartbeat.js:15`, `src/gateway.js:26`
- Annotation: setInterval per session; handle stored in session.heartbeat; no clearInterval anywhere (session.close at src/session.js:62-69 never clears it). Interval closure captures session -> socket, decoder, recent[64], topics. unref() only stops it keeping the loop alive, not from being GC root. beat() returns early when closed but never clears itself. Manual measurement: heap +4MB/wave of 1000 conns after forced GC, 0 open sessions; adding clearInterval in close() in a scratch copy gave plateau ~4MB.
- Suggested archetype (unconfirmed): L-WORKER
- Reason: tool_missing
- Remedy: Leak located manually (src/heartbeat.js:15 setInterval never cleared on Session.close, src/session.js:62). Install a dla release with harness/sample/trend/ledger confirm/gate/report to formally confirm; fix class: clear the interval when the session closes.
- Supporting evidence: E5

### F2: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.metrics`
- Region: heap
- Site: `src/gateway.js:13`
- Reason: tool_missing
- Remedy: Not the growth source by manual check (holder emptied on close / bounded / test-only); formal refute needs ledger refute, absent in this dla release.
- Supporting evidence: E6

### F3: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.router`
- Region: heap
- Site: `src/gateway.js:14`
- Reason: tool_missing
- Remedy: Not the growth source by manual check (holder emptied on close / bounded / test-only); formal refute needs ledger refute, absent in this dla release.
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.sessions`
- Region: heap
- Site: `src/gateway.js:15`
- Writers: `src/gateway.js:22`
- Removers: `src/gateway.js:32`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: Not the growth source by manual check (holder emptied on close / bounded / test-only); formal refute needs ledger refute, absent in this dla release.
- Supporting evidence: E6

### F5: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: registration-table
- Owner: `src.gateway.Gateway.accept`
- Region: heap
- Site: `src/gateway.js:24`
- Writers: `src/gateway.js:25`, `src/session.js:12`, `src/session.js:54`
- Removers: `load.js:50`, `src/gateway.js:31`, `src/session.js:66`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: Not the growth source by manual check (holder emptied on close / bounded / test-only); formal refute needs ledger refute, absent in this dla release.
- Supporting evidence: E6

### F6: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: registration-table
- Owner: `src.gateway.Gateway.accept`
- Region: heap
- Site: `src/gateway.js:25`
- Writers: `src/gateway.js:24`, `src/session.js:12`, `src/session.js:54`
- Removers: `load.js:50`, `src/gateway.js:31`, `src/session.js:66`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: Not the growth source by manual check (holder emptied on close / bounded / test-only); formal refute needs ledger refute, absent in this dla release.
- Supporting evidence: E6

### F7: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `src.heartbeat.startHeartbeat`
- Region: non-heap
- Site: `src/heartbeat.js:15`
- Writers: `src/heartbeat.js:15`, `src/gateway.js:26`
- Annotation: setInterval per session; handle stored in session.heartbeat; no clearInterval anywhere (session.close at src/session.js:62-69 never clears it). Interval closure captures session -> socket, decoder, recent[64], topics. unref() only stops it keeping the loop alive, not from being GC root. beat() returns early when closed but never clears itself. Manual measurement: heap +4MB/wave of 1000 conns after forced GC, 0 open sessions; adding clearInterval in close() in a scratch copy gave plateau ~4MB.
- Reason: tool_missing
- Remedy: Leak located manually (src/heartbeat.js:15 setInterval never cleared on Session.close, src/session.js:62). Install a dla release with harness/sample/trend/ledger confirm/gate/report to formally confirm; fix class: clear the interval when the session closes.
- Supporting evidence: E6

### F8: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.loopback.LoopbackSocket.written`
- Region: heap
- Site: `src/loopback.js:11`
- Writers: `src/loopback.js:17`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: Not the growth source by manual check (holder emptied on close / bounded / test-only); formal refute needs ledger refute, absent in this dla release.
- Supporting evidence: E6

### F9: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.metrics.Metrics.counters`
- Region: heap
- Site: `src/metrics.js:7`
- Writers: `src/metrics.js:12`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: Not the growth source by manual check (holder emptied on close / bounded / test-only); formal refute needs ledger refute, absent in this dla release.
- Supporting evidence: E6

### F10: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.router.Router.topics`
- Region: heap
- Site: `src/router.js:9`
- Writers: `src/router.js:16`, `src/router.js:19`
- Removers: `src/router.js:26`, `src/router.js:27`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: Not the growth source by manual check (holder emptied on close / bounded / test-only); formal refute needs ledger refute, absent in this dla release.
- Supporting evidence: E6

### F11: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: registration-table
- Owner: `src.router.Router.dispatch`
- Region: heap
- Site: `src/router.js:51`
- Writers: `src/session.js:38`
- Removers: `src/gateway.js:25`, `src/router.js:54`, `src/session.js:33`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: Not the growth source by manual check (holder emptied on close / bounded / test-only); formal refute needs ledger refute, absent in this dla release.
- Supporting evidence: E6

### F12: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.decoder`
- Region: heap
- Site: `src/session.js:15`
- Writers: `src/session.js:30`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: Not the growth source by manual check (holder emptied on close / bounded / test-only); formal refute needs ledger refute, absent in this dla release.
- Supporting evidence: E6

### F13: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.topics`
- Region: heap
- Site: `src/session.js:19`
- Reason: tool_missing
- Remedy: Not the growth source by manual check (holder emptied on close / bounded / test-only); formal refute needs ledger refute, absent in this dla release.
- Supporting evidence: E6

### F14: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.recent`
- Region: heap
- Site: `src/session.js:20`
- Writers: `src/session.js:44`
- Removers: `src/session.js:45`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: Not the growth source by manual check (holder emptied on close / bounded / test-only); formal refute needs ledger refute, absent in this dla release.
- Supporting evidence: E6

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
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | edge gateway node project (cwd) |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | src/ in repo |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | RSS grows per connect/disconnect wave, no open sessions |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | node |
| `intake.build` | Does the project build locally? | answered | E1 | plain JS, no build |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | load.js |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 raw |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 60m |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | — |
| E2 | run-config | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | probe/probe.json |
| E4 | static-hit | static | eslint 10.14.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | sweep/eslint.json |
| E5 | static-hit | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | sweep/dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | census/node.json |
| E7 | workload | static | dla 0.1.0 | repo 43ae7425f8abfad179c2be1b5b3e254e09be292e | workload/workloads.json |
