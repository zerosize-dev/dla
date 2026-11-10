# DLA report

Run: `20261110-203123-edde`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 11 enumerated, 11 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 9

Residual: heap 13919.929090909092 bytes/iteration unattributed of 13919.929090909092 total (threshold 3479.982272727273) ABOVE THRESHOLD, see O1; heap 13938.00909090909 bytes/iteration unattributed of 13938.00909090909 total (threshold 3484.5022727272726) ABOVE THRESHOLD, see O1; heap 13919.888986013986 bytes/iteration unattributed of 13919.888986013986 total (threshold 3479.9722465034965) ABOVE THRESHOLD, see O1; heap 13964.820606060606 bytes/iteration unattributed of 13964.820606060606 total (threshold 3491.2051515151516) ABOVE THRESHOLD, see O1; heap 13960.493939393938 bytes/iteration unattributed of 13960.493939393938 total (threshold 3490.1234848484846) ABOVE THRESHOLD, see O1; non-heap 6956.480303030303 bytes/iteration unattributed of 6956.480303030303 total (threshold 1739.1200757575757) ABOVE THRESHOLD, see O2; non-heap 6956.581515151514 bytes/iteration unattributed of 6956.581515151514 total (threshold 1739.1453787878786) ABOVE THRESHOLD, see O2; non-heap 8305.68686868687 bytes/iteration unattributed of 8305.68686868687 total (threshold 2076.4217171717173) ABOVE THRESHOLD, see O2; non-heap 6977.469393939395 bytes/iteration unattributed of 6977.469393939395 total (threshold 1744.367348484849) ABOVE THRESHOLD, see O2; non-heap 7017.107575757575 bytes/iteration unattributed of 7017.107575757575 total (threshold 1754.276893939394) ABOVE THRESHOLD, see O2

Coverage: jvm 3520 iterations, 0 of 0 entry points exercised

## Findings

### F3: **L-REGISTER**

- State: confirmed
- Label: L-REGISTER (evidence: E12 unit-count-series)
- Growth: `probe:reports.EventBus.LISTENERS` 1 count/iteration (steady, gate pass, E12)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `reports.EventBus.LISTENERS`
- Region: heap
- Shape: steady
- Site: `src/reports/EventBus.java:8`
- Writers: `src/reports/EventBus.java:13`, `src/reports/EventBus.java:15`, `src/reports/EventBus.java:14`
- Removers: `src/reports/EventBus.java:17`, `src/reports/EventBus.java:19`, `src/reports/EventBus.java:18`
- Annotation: heuristic name match over the project's sources
- Annotation: subscribe adds; unsubscribe only called from LedgerExtension.stop, which reload() never calls
- Verified: DISAGREES (site: finding `src/reports/EventBus.java:8` vs verifier `src/reports/EventBus.java:13`; holder: finding static/global vs verifier registration-table; bundle 74d1859d9c39); finding mechanism: The static list EventBus.java:8 (LISTENERS) gets a Tally via subscribe (EventBus.java:13) on each ExtensionManager.reload; unsubscribe (EventBus.java:17) runs only when stop() is called, which reload() never does, so old Tallys and loaders stay reachable.; verifier mechanism: ExtensionManager.reload() drops the old Extension without calling stop(), so its Tally listener stays in static EventBus.LISTENERS (added via subscribe in LedgerExtension.start). Each reload adds a listener, which also pins the old URLClassLoader and its classes.
- Sibling hypotheses: F3.n1
- Supporting evidence: E6, E8, E12, E10
- Explanation (observed): The static list EventBus.java:8 (LISTENERS) gets a Tally via subscribe (EventBus.java:13) on each ExtensionManager.reload; unsubscribe (EventBus.java:17) runs only when stop() is called, which reload() never does, so old Tallys and loaders stay reachable.
- Explanation structure: adds: src/reports/EventBus.java:13; removal: conditional at src/reports/EventBus.java:17; evidence: E19

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `ledger.LedgerExtension.ROUNDING`
- Region: heap
- Site: `ext/ledger/LedgerExtension.java:11`
- Reason: not_exercised
- Remedy: ROUNDING is a fixed 1024-long table per loaded LedgerExtension class; it only multiplies because F3 pins each loader, not a separate holder
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `ledger.LedgerExtension.start`
- Region: heap
- Site: `ext/ledger/LedgerExtension.java:22`
- Writers: `ext/ledger/LedgerExtension.java:22`
- Annotation: start() subscribes tally to EventBus
- Reason: not_exercised
- Remedy: LedgerExtension.start() is the registering side of F3; same mechanism, recorded there
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: retention
- Holder: static/global
- Owner: `dla.harness.DlaDump.FIELDS`
- Region: heap
- Reason: not_exercised
- Remedy: dla.harness.DlaDump is DLA's own instrument (observer), not project code; no action
- Supporting evidence: E18

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: retention
- Holder: static/global
- Owner: `dla.harness.DlaDump.SHALLOW`
- Region: heap
- Reason: not_exercised
- Remedy: dla.harness.DlaDump is DLA's own instrument (observer), not project code; no action
- Supporting evidence: E18

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Class/object-count growth is the footprint of loaders and classes pinned by F3 (EventBus.LISTENERS holding each reloaded Tally); re-measure after the old extension is stopped on reload; not a separate holder
- Supporting evidence: E22

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Class/object-count growth is the footprint of loaders and classes pinned by F3 (EventBus.LISTENERS holding each reloaded Tally); re-measure after the old extension is stopped on reload; not a separate holder
- Supporting evidence: E24

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: discriminator
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Class/object-count growth is the footprint of loaders and classes pinned by F3 (EventBus.LISTENERS holding each reloaded Tally); re-measure after the old extension is stopped on reload; not a separate holder
- Supporting evidence: E13

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Class/object-count growth is the footprint of loaders and classes pinned by F3 (EventBus.LISTENERS holding each reloaded Tally); re-measure after the old extension is stopped on reload; not a separate holder
- Supporting evidence: E12

### F10: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Class/object-count growth is the footprint of loaders and classes pinned by F3 (EventBus.LISTENERS holding each reloaded Tally); re-measure after the old extension is stopped on reload; not a separate holder
- Supporting evidence: E28

## Ruled out

- **F3.n1** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (leak) E13
  - Hypothesis: N-WARM: listener list fills during warm-up then plateaus
  - Refuted if: listener count keeps growing over extra cycles
  - Contradicting evidence: E13
  - Originally supported by: E10

## Blind spots and unknowns

### Coverage gaps

- jvm/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- jvm/codeql: missing (codeql is not on PATH or in the tools directory). Install hint: dla install codeql
- jvm/errorprone: missing (Error Prone jars not found in the tools directory (or DLA_ERRORPRONE_HOME)). Install hint: dla install errorprone
- jvm/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)
- jvm/sonar: missing (sonar-scanner is not installed and Sonar rules need a SonarQube or SonarCloud server). Install hint: install sonar-scanner, run a SonarQube or SonarCloud server and set SONAR_HOST_URL and SONAR_TOKEN (dla install sonar prints the steps)
- jvm/valgrind: blocked (Valgrind is not run on the JVM: the virtual machine's own allocations drown the project's; use the allocator shim, jcmd NMT or `dla attach bpftrace`)

### Runtime blind spots

None recorded.

### Entry points not exercisable

None recorded.

### Evidence limitations and unknowns

- E4: partial coverage
- E4: limitation: no auxiliary classpath: library calls are not modelled
- E6: partial coverage
- E6: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E6: limitation: writers and removers are textual name matches
- E7: limitation: harness written; nothing was run (dla harness run measures it)
- E7: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E7: limitation: junit-platform-console-standalone not used: the plain driver main runs the loop (JUnit variant is generated only when the jar is on the classpath)
- E8: partial coverage (9 of 9 examined)
- E8: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E8: limitation: no entry points are listed for this runtime: run dla workload discover before the harness to see which ones ran
- E9: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E9: limitation: gate thresholds are provisional (not calibrated)
- E10: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E13: limitation: junit-platform-console-standalone not used: the plain driver main runs the loop (JUnit variant is generated only when the jar is on the classpath)
- E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- E14: partial coverage (9 of 9 examined)
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: no entry points are listed for this runtime: run dla workload discover before the harness to see which ones ran
- E15: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E16: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E17: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E18: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E19: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E19: limitation: the dumps are assumed to come from one process (checked: HPROF version, identifier size, timestamps, class set)
- E20: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: x is the dump number in header-timestamp order; t_ms is the header timestamp difference from the first dump
- E25: limitation: each point comes from a separate heap dump of one process; retained bytes are approximate (see the dump summaries)
- E25: limitation: holders absent from a dump's 200 largest have no figure at that point
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: x is the dump number in header-timestamp order; t_ms is the header timestamp difference from the first dump
- E26: limitation: each point comes from a separate heap dump of one process; retained bytes are approximate (see the dump summaries)
- E26: limitation: holders absent from a dump's 200 largest have no figure at that point
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: limitation: tested in a scratch copy; the project was not changed
- E29: limitation: gate thresholds are provisional (not calibrated); the 0.2 and 0.5 slope fractions are placeholders
- E29: limitation: exercised is null: the harness records entry points, not lines, so whether the edited line ran is not checked
- E29: limitation: every measured process of mode fresh exited with a non-zero code (p1-fresh: 1): the harness itself failed, so no series was written
- E30: partial coverage (2 of 3 examined)
- E30: limitation: F3: growth measured as a count; bytes not attributed
- E30: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E30: limitation: threshold is provisional (not calibrated)
- E31: partial coverage (2 of 3 examined)
- E31: limitation: F3: growth measured as a count; bytes not attributed
- E31: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E31: limitation: threshold is provisional (not calibrated)
- E32: partial coverage (2 of 3 examined)
- E32: limitation: F3: growth evidence is from S2, not S3; not subtracted
- E32: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E32: limitation: threshold is provisional (not calibrated)
- E33: partial coverage (2 of 3 examined)
- E33: limitation: F3: growth evidence is from S2, not S4; not subtracted
- E33: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E33: limitation: threshold is provisional (not calibrated)
- E34: partial coverage (2 of 3 examined)
- E34: limitation: F3: growth evidence is from S2, not S5; not subtracted
- E34: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E34: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | — |
| E2 | run-config | static | dla 0.1.0 | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | probe/probe.json |
| E4 | static-hit | static | spotbugs unknown | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | sweep/jvm-spotbugs.json |
| E5 | static-hit | static | dla 0.1.0 | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | sweep/jvm-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | census/jvm.json |
| E7 | workload | static | dla 0.1.0 | repo c12af1ceaa05db1d2eab2a666cf93fcb55cafa75 | harness/H1/harness.json |
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (44f654888581) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (50493d01a5ef) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (50493d01a5ef) | trend/T1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S2.json (99a9b7d0b88c) | trend/T2.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (99a9b7d0b88c) | trend/T2.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S3.json (baf446db60b1) | alloc/A1.json |
| E14 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (69d294d15d7a) | harness/H1/run-S4.json |
| E15 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-203123-edde/harness/H1/out/p1-fresh/capture/9/heap-1.hprof (927d43ae6c8c) | summaries/heap-1-927d43ae.json |
| E16 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-203123-edde/harness/H1/out/p1-fresh/capture/9/heap-2.hprof (b320726b76ef) | summaries/heap-2-b320726b.json |
| E17 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-203123-edde/harness/H1/out/p1-fresh/capture/9/heap-3.hprof (77cacaf411dc) | summaries/heap-3-77cacaf4.json |
| E18 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-203123-edde/harness/H1/out/p1-fresh/capture/9/heap-4.hprof (5502e0d30fed) | summaries/heap-4-5502e0d3.json |
| E19 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S6.json (e3dd5bc8153a) | series/S6.json |
| E20 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S6.json (e3dd5bc8153a) | series/S6.json |
| E21 | growth-series | test | dla 0.1.0 | artifact series/S4.json (24c4c89479e2) | trend/T4.json |
| E22 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (24c4c89479e2) | trend/T4.json |
| E23 | growth-series | test | dla 0.1.0 | artifact series/S5.json (859b759c14e5) | trend/T5.json |
| E24 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (859b759c14e5) | trend/T5.json |
| E25 | growth-series | dump | dla 0.1.0 | artifact series/S6.json (e3dd5bc8153a) | trend/T6.json |
| E26 | unit-count-series | dump | dla 0.1.0 | artifact series/S6.json (e3dd5bc8153a) | trend/T6.json |
| E27 | growth-series | test | dla 0.1.0 | artifact series/S3.json (baf446db60b1) | trend/T7.json |
| E28 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (baf446db60b1) | trend/T7.json |
| E29 | intervention | test | dla 0.1.0 | artifact intervene/I1/patch.diff (85e68bf6f129) | intervene/I1 |
| E30 | residual | test | dla 0.1.0 | artifact series/S1.json (50493d01a5ef) | gap/residual-S1-E30.json |
| E31 | residual | test | dla 0.1.0 | artifact series/S2.json (99a9b7d0b88c) | gap/residual-S2-E31.json |
| E32 | residual | test | dla 0.1.0 | artifact series/S3.json (baf446db60b1) | gap/residual-S3-E32.json |
| E33 | residual | test | dla 0.1.0 | artifact series/S4.json (24c4c89479e2) | gap/residual-S4-E33.json |
| E34 | residual | test | dla 0.1.0 | artifact series/S5.json (859b759c14e5) | gap/residual-S5-E34.json |
