# Real plugin runs grpc-3 and axios-2 (2026-11-21 13:24 UTC), build of the working tree at 13:23

Kept: scratch u36/grpc-3/ (report, ledger, state.json, transcript) and u36/axios-2/. Both
ended early without a leak, for reasons that are not about leak finding.

## grpc-3 (grpc-go v1.84.0): 11 turns, under 3 minutes of model time
- `dla census --lang go` did not return within the caller's 120 s foreground limit and was
  moved to the background (measured by me afterwards on a copy: 3 min 33 s wall, 2 min 8 s of
  CPU on the loaded machine; the census with the new per-site facts on 1 083 Go files).
- While it ran, the model answered three times with text only ("waiting for the census").
  Each was a stop attempt. The stop hook counted three attempts with no ledger change, let
  the third through and stamped the run INCOMPLETE, while a `dla` command of the run was
  still running.
- The session was resumed by the background notification and went on: census, sweep, first
  Go batch = package `encoding` with 2 targets (the per-package batch works), both `grew`.
- `budget_over_floor` returns None for a run stamped Incomplete, so the queue read
  `closed: floor`. `dla ledger close-rest` then closed 728 candidates with reason
  `time_budget` although 45 of 60 minutes were left.
- The auto budget was never settled (`settled: false`, 60 minutes instead of the 180 the same
  repository got before): the chained command that would have run the sweep gate was the one
  that was moved to the background.

## axios-2 (axios 1.15.1): the harness loads now, the rig had no dependencies
- The Node agent and driver load in the ES-module package (the `.cjs` change works).
- The project tree has no node_modules. The model's `npm install` of the three runtime
  dependencies hung in the sandbox of my test rig (the registry is not reachable from there),
  so only `lib/helpers/validator.js` could be loaded; the model recorded the blocker on
  `intake.workload` (env_unavailable) and closed everything. Report line: "Queue: 10 of 31
  ranked candidates targeted in 1 harness: no coverage data 1, missed once, no focus harness
  9; 21 not targeted".
- Rig fix: source copy with the three runtime dependencies installed:
  scratch u18/axios/proj-deps (installed in an empty directory and moved in; `npm install`
  inside the tree fails with an npm error).

## To fix in the tool
1. Census speed on a large repository (2 minutes of CPU).
2. The stop hook must not count an attempt as a stall, nor stamp INCOMPLETE, while a `dla`
   command of the run is still running; it should say which command is running.
3. A run stamped INCOMPLETE that goes on working must not have its queue closed by the stamp;
   `time_budget` must not be given as a reason with time left.
4. The auto budget must settle without depending on one particular gate command being run.
