# DLA report

Run: `20261110-202354-56c3`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 14 enumerated, 14 dispositioned; confirmed 2, strong 0, refuted 2, suspected 0, unresolved 10

Residual: heap 2681.55909090909 bytes/n unattributed of 2681.55909090909 total (threshold 670.3897727272727) ABOVE THRESHOLD, see O1; heap 2680.013125 bytes/n unattributed of 2680.013125 total (threshold 670.00328125) ABOVE THRESHOLD, see O1; heap 2679.212923881674 bytes/n unattributed of 2679.212923881674 total (threshold 669.8032309704186) ABOVE THRESHOLD, see O1; heap 2679.8552272727275 bytes/n unattributed of 2679.8552272727275 total (threshold 669.9638068181819) ABOVE THRESHOLD, see O1; heap 2679.1618957431456 bytes/n unattributed of 2679.1618957431456 total (threshold 669.7904739357864) ABOVE THRESHOLD, see O1

Coverage: python 3520 iterations, 8 of 11 entry points exercised

## Findings

### F3: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (module global parser._newest holds the newest Record; each Record.previous links to the prior one, so the whole history of Records is one unbounded linked chain reachable from the global; nothing truncates previous)
- Growth: `probe:ingest.parser._newest` 15.97 count/n (steady, gate pass, E12)
- Evidence classes: context, discriminator, growth, static
- Holder: static/global
- Owner: `ingest.parser._newest`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:3`
- Writers: `ingest/parser.py:29`
- Annotation: heuristic name match over the project's sources
- Annotation: module global _newest is the head of a linked list: every Record.previous points to the prior Record, so no Record is ever freed
- Verified: DISAGREES (site: finding `ingest/parser.py:3` vs verifier `ingest/parser.py:28`; label: finding UNCLASSIFIED vs verifier L-STALE; bundle 4ed87ee8dd56); finding mechanism: parser._newest (static/global holder, ingest/parser.py:3) is set to each new Record at ingest/parser.py:29 and Record.previous links to the prior one, forming an unbounded chain; previous is never cleared, so all Records stay alive.; verifier mechanism: Module global _newest holds latest Record; each new Record sets record.previous = _newest, forming an unbounded backward linked chain of all parsed records that is never truncated (previous only needed for delta), so every Record stays reachable.
- Sibling hypotheses: F3.n1
- Supporting evidence: E7, E10, E12, E13
- Explanation (inferred): parser._newest (static/global holder, ingest/parser.py:3) is set to each new Record at ingest/parser.py:29 and Record.previous links to the prior one, forming an unbounded chain; previous is never cleared, so all Records stay alive.
- Explanation structure: adds: ingest/parser.py:29; removal: never

### F6: **L-REGISTER**

- State: confirmed
- Label: L-REGISTER (evidence: E12 unit-count-series)
- Growth: `probe:ingest.registry._LISTENERS` 7.974 count/n (steady, gate pass, E12)
- Evidence classes: context, discriminator, growth, static
- Holder: registration-table
- Owner: `ingest.registry._LISTENERS`
- Region: heap
- Shape: steady
- Site: `ingest/registry.py:3`
- Writers: `ingest/registry.py:7`, `ingest/pipeline.py:28`
- Removers: `ingest/registry.py:24`
- Annotation: heuristic name match over the project's sources
- Annotation: each batch subscribes a new CheckpointWatcher to the checkpoint topic; it is never unsubscribed (only reset() clears, test-only)
- Verified: DISAGREES (site: finding `ingest/registry.py:3` vs verifier `ingest/pipeline.py:28`; bundle 9c33a7727d8e); finding mechanism: ingest/pipeline.py:28 subscribes a new CheckpointWatcher into registry._LISTENERS (registration-table holder, ingest/registry.py:3) on every batch; the remover at ingest/registry.py:24 runs only when a test calls reset, so in production listeners grow per batch.; verifier mechanism: process_batch creates a CheckpointWatcher per batch and subscribes it to the module-level registry._LISTENERS['checkpoint'] list; nothing ever unsubscribes (no unsubscribe API, only test reset), so the list and each watcher with its marks dict grows per batch forever
- Sibling hypotheses: F6.n1
- Supporting evidence: E7, E10, E12, E14
- Explanation (inferred): ingest/pipeline.py:28 subscribes a new CheckpointWatcher into registry._LISTENERS (registration-table holder, ingest/registry.py:3) on every batch; the remover at ingest/registry.py:24 runs only when a test calls reset, so in production listeners grow per batch.
- Explanation structure: adds: ingest/pipeline.py:28; removal: conditional at ingest/registry.py:24

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth, static
- Site: `ingest/parser.py:29`
- Reason: not_exercised
- Remedy: duplicate of confirmed F3 (chain head detector hit at parser.py:29)
- Supporting evidence: E6, E12

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth, static
- Site: `ingest/registry.py:3`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: duplicate of confirmed F6 (same registry._LISTENERS container, static detector hit)
- Supporting evidence: E6, E12

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth, static
- Holder: container-field
- Owner: `ingest.pipeline.Pipeline.per_sensor`
- Region: heap
- Site: `ingest/pipeline.py:15`
- Writers: `ingest/pipeline.py:25`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: per_sensor is bounded by 64 sensor keys; its probe did not grow
- Supporting evidence: E7, E12

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth, static
- Holder: registration-table
- Owner: `ingest.pipeline.Pipeline.process_batch`
- Region: heap
- Site: `ingest/pipeline.py:28`
- Reason: not_exercised
- Remedy: census row for process_batch; the registration is the confirmed F6
- Supporting evidence: E7, E12

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `ingest.models.Record.previous`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:29`
- Reason: not_exercised
- Remedy: duplicate of confirmed F3 (same Record.previous chain, instance-count view)
- Supporting evidence: E12

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `ingest.watchers.CheckpointWatcher`
- Region: heap
- Shape: steady
- Site: `ingest/pipeline.py:28`
- Reason: not_exercised
- Remedy: duplicate of confirmed F6 (CheckpointWatcher instances held by registry._LISTENERS)
- Supporting evidence: E12

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: discriminator, growth
- Holder: static/global
- Owner: `ingest.models.Record.previous`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:29`
- Reason: not_exercised
- Remedy: duplicate of confirmed F3 (same Record.previous chain, instance-count view)
- Supporting evidence: E12, E13, E14, E15, E16, E17, E18

### F10: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: discriminator, growth
- Holder: registration-table
- Owner: `ingest.watchers.CheckpointWatcher`
- Region: heap
- Shape: steady
- Site: `ingest/pipeline.py:28`
- Reason: not_exercised
- Remedy: duplicate of confirmed F6 (CheckpointWatcher instances held by registry._LISTENERS)
- Supporting evidence: E12, E13, E14, E15, E16, E17, E18

### F11: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `ingest.models.Record.previous`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:29`
- Reason: not_exercised
- Remedy: duplicate of confirmed F3 (Record chain, instance-count view)
- Supporting evidence: E20, E22

### F12: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `ingest.watchers.CheckpointWatcher`
- Region: heap
- Shape: steady
- Site: `ingest/pipeline.py:28`
- Reason: not_exercised
- Remedy: duplicate of confirmed F6 (CheckpointWatcher instances in registry._LISTENERS)
- Supporting evidence: E20, E22

## Ruled out

- **F3.n1** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (leak) E13
  - Hypothesis: N-WARM: growth is a warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E13
  - Originally supported by: E12
- **F6.n1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (leak) E14
  - Hypothesis: N-WARM: growth is a warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E14
  - Originally supported by: E12

## Blind spots and unknowns

### Coverage gaps

- python/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- python/codeql: missing (codeql is not on PATH or in the tools directory). Install hint: dla install codeql
- python/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)
- python/memray: missing (memray is not installed in the target's python interpreter, so --capture alloc has the tracemalloc report only). Install hint: pip install memray in the interpreter the workload runs under, then dla harness run <H> --capture alloc

### Runtime blind spots

None recorded.

### Entry points not exercisable

None recorded.

### Evidence limitations and unknowns

- E7: partial coverage
- E7: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E7: limitation: writers and removers are textual name matches
- E8: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E8: limitation: workloads were discovered by reading files; none was run
- E9: limitation: harness written; nothing was run (dla harness run measures it)
- E9: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E10: partial coverage (9 of 9 examined)
- E10: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E10: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E10: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E10: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E10: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E11: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E11: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E11: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E11: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E11: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E12: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E12: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E12: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E12: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E13: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E13: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E13: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E13: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E13: limitation: points 1 to 9 come from S1; the extra points from S2
- E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- E14: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E14: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E14: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E14: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E14: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E14: limitation: points 1 to 9 come from S1; the extra points from S4
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E19: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E19: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E19: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E19: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E19: limitation: points 1 to 9 come from S1; the extra points from S2
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E20: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E20: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E20: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E20: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E20: limitation: points 1 to 9 come from S1; the extra points from S2
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E21: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E21: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E21: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E21: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E21: limitation: points 1 to 9 come from S1; the extra points from S4
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E22: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E22: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E22: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E22: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E22: limitation: points 1 to 9 come from S1; the extra points from S4
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: partial coverage (1 of 3 examined)
- E23: limitation: F3: growth measured as a count; bytes not attributed
- E23: limitation: F6: growth measured as a count; bytes not attributed
- E23: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E23: limitation: threshold is provisional (not calibrated)
- E24: partial coverage (1 of 3 examined)
- E24: limitation: F3: growth evidence is from S1, not S2; not subtracted
- E24: limitation: F6: growth evidence is from S1, not S2; not subtracted
- E24: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E24: limitation: threshold is provisional (not calibrated)
- E25: partial coverage (1 of 3 examined)
- E25: limitation: F3: growth evidence is from S1, not S3; not subtracted
- E25: limitation: F6: growth evidence is from S1, not S3; not subtracted
- E25: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E25: limitation: threshold is provisional (not calibrated)
- E26: partial coverage (1 of 3 examined)
- E26: limitation: F3: growth evidence is from S1, not S4; not subtracted
- E26: limitation: F6: growth evidence is from S1, not S4; not subtracted
- E26: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E26: limitation: threshold is provisional (not calibrated)
- E27: partial coverage (1 of 3 examined)
- E27: limitation: F3: growth evidence is from S1, not S5; not subtracted
- E27: limitation: F6: growth evidence is from S1, not S5; not subtracted
- E27: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E27: limitation: threshold is provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | — |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | — |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | — |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | — |
| `intake.build` | Does the project build locally? | answered | E1 | — |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | — |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E1 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | — |
| E2 | run-config | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | probe/probe.json |
| E4 | static-hit | static | ruff 0.16.9 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | sweep/python-ruff.json |
| E5 | static-hit | static | pylint 4.1.3 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | sweep/python-pylint.json |
| E6 | static-hit | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | sweep/python-dla.json |
| E7 | census-holder | static | dla-census 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | census/python.json |
| E8 | workload | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | workload/workloads.json |
| E9 | workload | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | harness/H1/harness.json |
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (676a83684ec1) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (4afd01b10de1) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (4afd01b10de1) | trend/T1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S3.json (3e98abb0b6f9) | alloc/A1.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S5.json (3d2c1e9effc7) | alloc/A2.json |
| E15 | growth-series | test | dla 0.1.0 | artifact series/S2.json (88387ea6bf21) | trend/T4.json |
| E16 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (88387ea6bf21) | trend/T4.json |
| E17 | growth-series | test | dla 0.1.0 | artifact series/S4.json (06c1e5bba323) | trend/T5.json |
| E18 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (06c1e5bba323) | trend/T5.json |
| E19 | growth-series | test | dla 0.1.0 | artifact series/S3.json (3e98abb0b6f9) | trend/T6.json |
| E20 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (3e98abb0b6f9) | trend/T6.json |
| E21 | growth-series | test | dla 0.1.0 | artifact series/S5.json (3d2c1e9effc7) | trend/T7.json |
| E22 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (3d2c1e9effc7) | trend/T7.json |
| E23 | residual | test | dla 0.1.0 | artifact series/S1.json (4afd01b10de1) | gap/residual-S1-E23.json |
| E24 | residual | test | dla 0.1.0 | artifact series/S2.json (88387ea6bf21) | gap/residual-S2-E24.json |
| E25 | residual | test | dla 0.1.0 | artifact series/S3.json (3e98abb0b6f9) | gap/residual-S3-E25.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S4.json (06c1e5bba323) | gap/residual-S4-E26.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S5.json (3d2c1e9effc7) | gap/residual-S5-E27.json |
