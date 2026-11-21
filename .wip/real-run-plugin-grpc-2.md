# Real plugin run grpc-2 (2026-11-21 08:27 to 08:50 UTC), grpc-go v1.84.0

Binary = working tree at 08:24 (ranking, trigger step, Go driver coverage; before most of the
report fixes). 23 minutes of a 180-minute budget, then the model stopped: report INCOMPLETE
(248 open), 12 harness runs, 27 series. Run directory kept: /root/dla-real/grpc-2/project/.dla/
(cache deleted). Documented leak: health/server.go:49 `Server.updates` (F204), trigger = Watch
streams with distinct service names (outer map key never removed).

What happened
- Ranking worked: batch 1 (H1, `--next`) had 11 targets, among them F203 `Server.statusMap`
  (rank 6) and its sibling F204 `Server.updates`.
- The Go driver of a `--next` batch was generated in `./benchmark/benchmain` (the package of the
  discovered workload, a `main` package) for 11 targets in 7 packages. From there a unit cannot
  read unexported state of the other packages and no probe exists. H1 and H2 (same place) did
  not execute health's code. F204 was then SETTLED by the two-drivers rule and closed
  `suspected_static_only / not_exercised` ("targeted in H1 and H2, its code did not run") ten
  minutes into the run. The documented leak was never exercised.
- The model then made per-package harnesses by hand (`harness gen --driver --probe F203 --probe
  F288 --package health`). Note in the output: "its holder health.server.Server.statusMap
  (container-field) is not a package variable, so it is recorded as a target without a probe":
  Go has probes for package variables only; probes.json was empty; `dlaProbes` returned nil.
- H5 unit: `dlaHS.SetServingStatus("svc-"+key, 1)`: one new map entry per unit. alloc-test read
  PLATEAU on go.heap_live (main slope 78 B/unit, extra slope 16.6 B/unit, rise 51 kB vs noise
  20 kB): a Go map grows in steps, between two doublings only the key strings are allocated.
  F203 was REFUTED although the map grows by one entry per unit. A size reading (len of the
  map) would have shown it.
- Trigger step on Go: F194 (the grpclog component cache) confirmed; two entry-path harnesses
  (H7, H9) "grow heap but have no probe to tie it"; the report says "Trigger: not found ... two
  workloads through the public API did not grow it". Wrong reading: the verdicts were not
  `constant` (measured flat), they were unmeasured. Only a measured flat holder is "did not grow".
- `dla ledger close-rest` on 744 findings ran longer than 120 s (moved to the background).
- The model tried to stop four times ("each batch needs a hand-written driver for 1 to 2
  holders", "I can't finish"); the stop hook sent it back three times, the unchanged ledger let
  the last one through with 157 of 180 minutes left and 99 ranked candidates not targeted.
- Final report lines: "Queue: 40 of 128 ranked candidates targeted in 9 harnesses: grew 1,
  executed 10, not reached 18, not read yet 11; 88 not targeted"; "Coverage: go 3520 iterations,
  entry points not enumerated (none listed for this runtime)".

Causes to fix
1. Go batches must be one package each, with the driver generated inside that package.
2. Go needs size probes for struct fields (most real holders are fields), filled with almost no
   work by the model (track the instance, the harness reads the field sizes by reflection).
3. alloc-test must not read a step-growing container as a plateau when a size probe of the
   finding keeps rising.
4. Trigger `not_found` only from two entry-path runs that MEASURED the holder flat (`constant`);
   unmeasured runs are "not checked", said as such, without blocking forever.
5. close-rest speed on a ledger of several hundred findings.
