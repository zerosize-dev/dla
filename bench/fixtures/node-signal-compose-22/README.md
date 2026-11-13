# node-signal-compose-22

Category `decoy-control` (control half of a version pair, partner
`node-signal-compose-24`). Cause `N-WARM`. Expected answer: no confirmed leak; the
heap after a forced collection is flat.

The source, the tests and the prompt are the same as in `node-signal-compose-24`.
Only the Node release differs: `[toolchain] node = "22.12.0"` here, `.nvmrc` and
`engines.node` in `package.json` pin the same release. The partner runs on 24.13.1,
where each `AbortSignal.any([...])` leaves a few bookkeeping records on the long-lived
source signal.

## Mechanism

Each request still builds `AbortSignal.any([lifecycle.signal, deadline.signal])`
(`src/handler.js:12`) from the process-wide controller's signal and a per-request
deadline, and releases its timer when it finishes. On Node 22.12.0 the dependants of
the long-lived signal do not accumulate, so nothing outlives a request.

## Measurements

Node v22.12.0 (the toolchain named in `expected.toml`). Harness outside `project/`
(`measure.js`): the same as for the partner. Command:
`node --expose-gc measure.js project 5000 20000 80000 160000`.

| requests | heap after GC (KB) | bytes per request |
|---|---|---|
| 5000   | 226  | 46 |
| 20000  | 418  | 21 |
| 80000  | -239 | -3 |
| 160000 | -261 | -2 |

The heap after a collection stays within a few hundred KB of the start from 5000 to
160000 requests, and the slope of the second half is zero. The partner grows by
171 bytes per request on 24.13.1 under the same command.

Workload: `node load.js --iterations 20000` runs in about one second. Unit tests:
`node --test` (7 pass).

## Minimal edit from the trap

The Node release (`24.13.1` to `22.12.0`) in `[toolchain]`, `.nvmrc` and
`engines.node`; no source line differs.

## Dynamic result

`dla-bench dynamic` (pinned binaries, `[toolchain] node = "22.12.0"`), archetypes on
and off: pass. The probe on the process-wide signal is flat, the plateau test refutes
the candidate and nothing is confirmed.
