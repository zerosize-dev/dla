# Real plugin runs axios-1 and tokio-1 (2026-11-21 10:12 UTC), combined build (snapshot 3b0cc2d)

Kept: scratch u36/axios-1/ and u36/tokio-1/ (report.md, ledger.jsonl, transcript.jsonl, the
tokio driver). Both ended in under two minutes, 19 turns each, report "COMPLETE", no leak.

## axios 1.15.1 (Node, `"type": "module"`): measured nothing
- Batch 1 generated (10 targets, all with probes). `dla harness run H1` failed at once:
  `.dla/<run>/harness/H1/agent/dla-preload.js:14 const fs = require('fs') ... ReferenceError:
  require is not defined in ES module scope`. The harness agent and driver are CommonJS files
  named `.js`, and the scratch copy lives under the project's package.json, which says
  `"type": "module"`, so Node loads them as ES modules.
- The model may not edit `.dla` agent files, so it recorded the blocker on `intake.workload`
  (`build_fails`), which closes the queue, closed all 54 candidates `not_exercised` and
  reported "This run measured nothing" (honest, but nothing was tested). 71 seconds.
- No Node fixture has `"type": "module"`, so the bench never saw this.
- Documented leak: listeners added to a keep-alive socket per request and never removed
  (`lib/adapters/http.js`, registrations around lines 903 to 925); the Node agent counts live
  listener registrations by site, which is the right measure once the harness loads.

## tokio-util 0.7.1 (Rust): one driver, every target settled, done in 100 seconds
- 14 findings, 7 ranked. One driver (H1) for all 7; `dla trend`: Rust drivers have no
  coverage data, so every target read `unknown`, and `unknown` settles a target after one
  driver run. Queue empty, everything closed, report COMPLETE with 59 of 60 minutes unused.
- The driver's unit registered waiters and cancelled a token; `native.live_bytes` stayed flat.
  The documented leak needs a grandchild token (`parent.child_token().child_token()`) and
  `parent.cancel()`: 72 bytes per unit. Nothing told the model to derive objects from derived
  objects or to vary the order of cancel and drop.
- Three candidates in `src/task/spawn_pinned.rs` need the crate feature `rt`; the harness
  build has no way to pass features, so they were closed `unresolved(env_unavailable)`.

## Causes to fix
1. Node harness in an ES-module package (agent and driver must load whatever the package type
   is; hooks must be able to import ES modules; probes of module-level bindings of ES modules).
2. A verdict `unknown` (no coverage data) must not settle a target after one driver run.
3. Rust driver runs need coverage data (executed / not_executed) and a way to enable features.
4. The driver block should ask for the life of the objects under test (create, derive also
   from derived objects, use, cancel or close, drop, in more than one order), for every runtime.
