# DLA report

Run: `20261107-194930-0dc8`

Status: COMPLETE

Findings: 13 enumerated, 13 dispositioned; confirmed 0, strong 0, refuted 0, suspected 0, unresolved 13

Residual: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.metrics`
- Region: heap
- Site: `src/gateway.js:13`
- Reason: tool_missing
- Remedy: Installed dla lacks gate/harness/sample/trend/ledger test|confirm|refute. Manual measurement (6-25 waves x 2000 sessions, forced GC): heapUsed flat ~4MB, 0 sessions, 0 active timers, RSS plateaus ~88MB after warm-up; no growth reproduced for this holder. Re-run with a full dla release and a real net.Socket workload.
- Supporting evidence: E7

### F2: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.gateway.Gateway.router`
- Region: heap
- Site: `src/gateway.js:14`
- Reason: tool_missing
- Remedy: Installed dla lacks gate/harness/sample/trend/ledger test|confirm|refute. Manual measurement (6-25 waves x 2000 sessions, forced GC): heapUsed flat ~4MB, 0 sessions, 0 active timers, RSS plateaus ~88MB after warm-up; no growth reproduced for this holder. Re-run with a full dla release and a real net.Socket workload.
- Supporting evidence: E7

### F3: **UNCLASSIFIED**

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
- Remedy: Installed dla lacks gate/harness/sample/trend/ledger test|confirm|refute. Manual measurement (6-25 waves x 2000 sessions, forced GC): heapUsed flat ~4MB, 0 sessions, 0 active timers, RSS plateaus ~88MB after warm-up; no growth reproduced for this holder. Re-run with a full dla release and a real net.Socket workload.
- Supporting evidence: E7

### F4: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: registration-table
- Owner: `src.gateway.Gateway.accept`
- Region: heap
- Site: `src/gateway.js:24`
- Writers: `src/gateway.js:25`, `src/session.js:12`, `src/session.js:54`
- Removers: `load.js:50`, `src/gateway.js:31`, `src/session.js:67`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: Installed dla lacks gate/harness/sample/trend/ledger test|confirm|refute. Manual measurement (6-25 waves x 2000 sessions, forced GC): heapUsed flat ~4MB, 0 sessions, 0 active timers, RSS plateaus ~88MB after warm-up; no growth reproduced for this holder. Re-run with a full dla release and a real net.Socket workload.
- Supporting evidence: E7

### F5: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: registration-table
- Owner: `src.gateway.Gateway.accept`
- Region: heap
- Site: `src/gateway.js:25`
- Writers: `src/gateway.js:24`, `src/session.js:12`, `src/session.js:54`
- Removers: `load.js:50`, `src/gateway.js:31`, `src/session.js:67`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: Installed dla lacks gate/harness/sample/trend/ledger test|confirm|refute. Manual measurement (6-25 waves x 2000 sessions, forced GC): heapUsed flat ~4MB, 0 sessions, 0 active timers, RSS plateaus ~88MB after warm-up; no growth reproduced for this holder. Re-run with a full dla release and a real net.Socket workload.
- Supporting evidence: E7

### F6: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: blocked-stack
- Owner: `src.heartbeat.startHeartbeat`
- Region: non-heap
- Site: `src/heartbeat.js:15`
- Reason: tool_missing
- Remedy: Installed dla lacks gate/harness/sample/trend/ledger test|confirm|refute. Manual measurement (6-25 waves x 2000 sessions, forced GC): heapUsed flat ~4MB, 0 sessions, 0 active timers, RSS plateaus ~88MB after warm-up; no growth reproduced for this holder. Re-run with a full dla release and a real net.Socket workload.
- Supporting evidence: E7

### F7: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.loopback.LoopbackSocket.written`
- Region: heap
- Site: `src/loopback.js:11`
- Writers: `src/loopback.js:17`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: Installed dla lacks gate/harness/sample/trend/ledger test|confirm|refute. Manual measurement (6-25 waves x 2000 sessions, forced GC): heapUsed flat ~4MB, 0 sessions, 0 active timers, RSS plateaus ~88MB after warm-up; no growth reproduced for this holder. Re-run with a full dla release and a real net.Socket workload.
- Supporting evidence: E7

### F8: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.metrics.Metrics.counters`
- Region: heap
- Site: `src/metrics.js:7`
- Writers: `src/metrics.js:12`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: Installed dla lacks gate/harness/sample/trend/ledger test|confirm|refute. Manual measurement (6-25 waves x 2000 sessions, forced GC): heapUsed flat ~4MB, 0 sessions, 0 active timers, RSS plateaus ~88MB after warm-up; no growth reproduced for this holder. Re-run with a full dla release and a real net.Socket workload.
- Supporting evidence: E7

### F9: **UNCLASSIFIED**

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
- Remedy: Installed dla lacks gate/harness/sample/trend/ledger test|confirm|refute. Manual measurement (6-25 waves x 2000 sessions, forced GC): heapUsed flat ~4MB, 0 sessions, 0 active timers, RSS plateaus ~88MB after warm-up; no growth reproduced for this holder. Re-run with a full dla release and a real net.Socket workload.
- Supporting evidence: E7

### F10: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: registration-table
- Owner: `src.router.Router.dispatch`
- Region: heap
- Site: `src/router.js:51`
- Writers: `src/session.js:38`
- Removers: `src/gateway.js:25`, `src/router.js:54`, `src/session.js:33`, `src/session.js:65`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: Installed dla lacks gate/harness/sample/trend/ledger test|confirm|refute. Manual measurement (6-25 waves x 2000 sessions, forced GC): heapUsed flat ~4MB, 0 sessions, 0 active timers, RSS plateaus ~88MB after warm-up; no growth reproduced for this holder. Re-run with a full dla release and a real net.Socket workload.
- Supporting evidence: E7

### F11: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.decoder`
- Region: heap
- Site: `src/session.js:15`
- Writers: `src/session.js:30`
- Annotation: heuristic name match over the project's sources
- Reason: tool_missing
- Remedy: Installed dla lacks gate/harness/sample/trend/ledger test|confirm|refute. Manual measurement (6-25 waves x 2000 sessions, forced GC): heapUsed flat ~4MB, 0 sessions, 0 active timers, RSS plateaus ~88MB after warm-up; no growth reproduced for this holder. Re-run with a full dla release and a real net.Socket workload.
- Supporting evidence: E7

### F12: **UNCLASSIFIED**

- State: unresolved(tool_missing)
- Evidence classes: static
- Holder: container-field
- Owner: `src.session.Session.topics`
- Region: heap
- Site: `src/session.js:19`
- Reason: tool_missing
- Remedy: Installed dla lacks gate/harness/sample/trend/ledger test|confirm|refute. Manual measurement (6-25 waves x 2000 sessions, forced GC): heapUsed flat ~4MB, 0 sessions, 0 active timers, RSS plateaus ~88MB after warm-up; no growth reproduced for this holder. Re-run with a full dla release and a real net.Socket workload.
- Supporting evidence: E7

### F13: **UNCLASSIFIED**

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
- Remedy: Installed dla lacks gate/harness/sample/trend/ledger test|confirm|refute. Manual measurement (6-25 waves x 2000 sessions, forced GC): heapUsed flat ~4MB, 0 sessions, 0 active timers, RSS plateaus ~88MB after warm-up; no growth reproduced for this holder. Re-run with a full dla release and a real net.Socket workload.
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
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | edge-gateway node project in cwd |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | src/*.js |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | RSS grows per wave of connect/disconnect, no sessions open |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | node>=22 |
| `intake.build` | Does the project build locally? | answered | E1 | plain JS no build |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | load.js runWorkload |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | runner consent tiers 1-4 raw values |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | 60m |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | — |
| E2 | run-config | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | probe/probe.json |
| E4 | probe | static | dla-probe 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | probe/probe.json |
| E5 | static-hit | static | eslint 10.14.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | sweep/eslint.json |
| E6 | static-hit | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | sweep/dla.json |
| E7 | census-holder | static | dla-census 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | census/node.json |
| E8 | workload | static | dla 0.1.0 | repo 707b9e31201c2ebd92364a906fb9199d67de1668 | workload/workloads.json |
