# DLA report

Run: `20261110-164303-c2bc`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 8 enumerated, 8 dispositioned; confirmed 0, strong 0, refuted 5, suspected 0, unresolved 3

Residual: heap 41.64890604890605 bytes/n unattributed of 41.64890604890605 total (threshold 92.1948227027027); heap 0.4804804804804804 bytes/n unattributed of 0.4804804804804804 total (threshold 0.7426356756756312); heap 2.2324324324324323 bytes/n unattributed of 2.2324324324324323 total (threshold 3.066444216216216); heap 0.8981981981981981 bytes/n unattributed of 0.8981981981981981 total (threshold 0.8858391891891891) ABOVE THRESHOLD, see O2; heap 1.7225225225225227 bytes/n unattributed of 1.7225225225225227 total (threshold 3.524046702702702); heap 1.014054054054054 bytes/n unattributed of 1.014054054054054 total (threshold 0.9582518918918916) ABOVE THRESHOLD, see O2; heap 2.321921921921922 bytes/n unattributed of 2.321921921921922 total (threshold 3.3195992792793); heap 0.7027027027027027 bytes/n unattributed of 0.7027027027027027 total (threshold 2.4007816216216216); heap 1.2108108108108109 bytes/n unattributed of 1.2108108108108109 total (threshold 3.225923891891892); heap 1.0630630630630629 bytes/n unattributed of 1.0630630630630629 total (threshold 1.31316); heap 0.032432432432432434 bytes/n unattributed of 0.032432432432432434 total (threshold 0.4295532972972973); heap 2.1585585585585587 bytes/n unattributed of 2.1585585585585587 total (threshold 4.06656); heap 1.0263063063063065 bytes/n unattributed of 1.0263063063063065 total (threshold 84.18389794594594); heap 2.486246246246246 bytes/n unattributed of 2.486246246246246 total (threshold 5.1563187027027375); heap 1.0858858858858855 bytes/n unattributed of 1.0858858858858855 total (threshold 84.78406154954958); non-heap 303.9320463320463 bytes/n unattributed of 303.9320463320463 total (threshold 677.1715394594593); non-heap 10.483843843843845 bytes/n unattributed of 10.483843843843845 total (threshold 9.574199567567636) ABOVE THRESHOLD, see O3; non-heap 62.406306306306305 bytes/n unattributed of 62.406306306306305 total (threshold 95.48625194594594); non-heap 16.55885885885886 bytes/n unattributed of 16.55885885885886 total (threshold 43.14308756756783); non-heap 50.538738738738736 bytes/n unattributed of 50.538738738738736 total (threshold 147.28335394594595); non-heap 0 bytes/n unattributed of -11.153153153153152 total (threshold 38.33351027027027); non-heap 42.104504504504504 bytes/n unattributed of 42.104504504504504 total (threshold 73.8425625945946); non-heap 18.16966966966967 bytes/n unattributed of 18.16966966966967 total (threshold 97.13691810810784); non-heap 38.58944658944659 bytes/n unattributed of 38.58944658944659 total (threshold 96.61575654054036); non-heap 31.804204204204204 bytes/n unattributed of 31.804204204204204 total (threshold 29.236414054054052) ABOVE THRESHOLD, see O3; non-heap 4.942030492030492 bytes/n unattributed of 4.942030492030492 total (threshold 11.920083451143492); non-heap 5.7405405405405405 bytes/n unattributed of 5.7405405405405405 total (threshold 53.83040108108108); non-heap 20.748648648648647 bytes/n unattributed of 20.748648648648647 total (threshold 506.3431618378377); non-heap 50.25405405405405 bytes/n unattributed of 50.25405405405405 total (threshold 197.09879027027023); non-heap 26.12972972972973 bytes/n unattributed of 26.12972972972973 total (threshold 504.6342983783784)

Coverage: jvm 8880 iterations, 15 of 15 entry points exercised

## Findings

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: JDK-internal class-count growth (harness/capture artefact, a few classes); the alloc-test only tested os.rss, which is whole-process. Re-run with a longer in-process driver and no heap dumps to see if class counts keep growing.
- Supporting evidence: E11

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: JDK-internal class-count growth (harness/capture artefact, a few classes); the alloc-test only tested os.rss, which is whole-process. Re-run with a longer in-process driver and no heap dumps to see if class counts keep growing.
- Supporting evidence: E20

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: String/byte[] instance counts rise a little across harness points while ORDERS_BY_ID probes plateau at 1000; take a longer in-process driver series (no dumps) to see if they plateau.
- Supporting evidence: E29

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (non-leak) E14
  - Contradicting evidence: E14
  - Originally supported by: E6
- **F2** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (non-leak) E15
  - Contradicting evidence: E15
  - Originally supported by: E6
- **F3** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (non-leak) E16
  - Contradicting evidence: E16
  - Originally supported by: E6
- **F4** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (non-leak) E12
  - Contradicting evidence: E12
  - Originally supported by: E6
- **F7** (UNCLASSIFIED): discriminator E27
  - Ruled out by: plateau+purge (non-leak) E27
  - Contradicting evidence: E27
  - Originally supported by: E22

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
- E7: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E7: limitation: workloads were discovered by reading files; none was run
- E8: limitation: harness written; nothing was run (dla harness run measures it)
- E8: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E8: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E9: partial coverage (9 of 9 examined)
- E9: limitation: command-mode heap dumps come from separate processes: dla summarize heap-graph compares dumps of one process only, so use a driver (dla harness gen --driver) for a dump series
- E9: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E9: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E10: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E10: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E11: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: points 1 to 9 come from S1; the extra points from S2
- E12: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E12: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E12: limitation: gate and plateau thresholds are provisional (not calibrated)
- E13: partial coverage (2 of 3 examined)
- E13: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E13: limitation: threshold is provisional (not calibrated)
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: points 1 to 9 come from S1; the extra points from S4
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: points 1 to 9 come from S1; the extra points from S6
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: points 1 to 9 come from S1; the extra points from S8
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E23: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E24: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E25: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E25: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E25: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E25: limitation: points 1 to 9 come from S1; the extra points from S10
- E25: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E25: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E25: limitation: gate and plateau thresholds are provisional (not calibrated)
- E26: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E26: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E26: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E26: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E26: limitation: points 1 to 9 come from S1; the extra points from S12
- E26: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E26: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E26: limitation: gate and plateau thresholds are provisional (not calibrated)
- E27: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E27: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E27: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E27: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E27: limitation: points 1 to 9 come from S1; the extra points from S14
- E27: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E27: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E27: limitation: gate and plateau thresholds are provisional (not calibrated)
- E28: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E28: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E29: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E29: limitation: gate thresholds are provisional (not calibrated)
- E30: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E30: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E30: limitation: gate thresholds are provisional (not calibrated)
- E31: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E31: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E31: limitation: gate thresholds are provisional (not calibrated)
- E32: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E32: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E32: limitation: gate thresholds are provisional (not calibrated)
- E33: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E33: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E33: limitation: gate thresholds are provisional (not calibrated)
- E34: partial coverage (2 of 3 examined)
- E34: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E34: limitation: threshold is provisional (not calibrated)
- E35: partial coverage (2 of 3 examined)
- E35: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E35: limitation: threshold is provisional (not calibrated)
- E36: partial coverage (2 of 3 examined)
- E36: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E36: limitation: threshold is provisional (not calibrated)
- E37: partial coverage (2 of 3 examined)
- E37: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E37: limitation: threshold is provisional (not calibrated)
- E38: partial coverage (2 of 3 examined)
- E38: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E38: limitation: O1 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E38: limitation: threshold is provisional (not calibrated)
- E39: partial coverage (2 of 3 examined)
- E39: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E39: limitation: O1 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E39: limitation: threshold is provisional (not calibrated)
- E40: partial coverage (2 of 3 examined)
- E40: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E40: limitation: O1 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E40: limitation: threshold is provisional (not calibrated)
- E41: partial coverage (2 of 3 examined)
- E41: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E41: limitation: O1 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E41: limitation: threshold is provisional (not calibrated)
- E42: partial coverage (2 of 3 examined)
- E42: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E42: limitation: O1 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E42: limitation: threshold is provisional (not calibrated)
- E43: partial coverage (2 of 3 examined)
- E43: limitation: off-heap: no gap series for S10; run dla gap S10 first (node.external is used when present)
- E43: limitation: threshold is provisional (not calibrated)
- E44: partial coverage (2 of 3 examined)
- E44: limitation: off-heap: no gap series for S11; run dla gap S11 first (node.external is used when present)
- E44: limitation: O1 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E44: limitation: threshold is provisional (not calibrated)
- E45: partial coverage (2 of 3 examined)
- E45: limitation: off-heap: no gap series for S12; run dla gap S12 first (node.external is used when present)
- E45: limitation: O1 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E45: limitation: threshold is provisional (not calibrated)
- E46: partial coverage (2 of 3 examined)
- E46: limitation: off-heap: no gap series for S13; run dla gap S13 first (node.external is used when present)
- E46: limitation: O2 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E46: limitation: O1 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E46: limitation: threshold is provisional (not calibrated)
- E47: partial coverage (2 of 3 examined)
- E47: limitation: off-heap: no gap series for S14; run dla gap S14 first (node.external is used when present)
- E47: limitation: O1 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E47: limitation: threshold is provisional (not calibrated)
- E48: partial coverage (2 of 3 examined)
- E48: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E48: limitation: threshold is provisional (not calibrated)
- E49: partial coverage (2 of 3 examined)
- E49: limitation: off-heap: no gap series for S10; run dla gap S10 first (node.external is used when present)
- E49: limitation: threshold is provisional (not calibrated)
- E50: partial coverage (2 of 3 examined)
- E50: limitation: off-heap: no gap series for S15; run dla gap S15 first (node.external is used when present)
- E50: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | — |
| E2 | run-config | static | dla 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | probe/probe.json |
| E4 | static-hit | static | spotbugs unknown | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | sweep/jvm-spotbugs.json |
| E5 | static-hit | static | dla 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | sweep/jvm-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | census/jvm.json |
| E7 | workload | static | dla 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (0d06c7515494) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (bf4ef4e48284) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (bf4ef4e48284) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (270929a1d1f7) | alloc/A1.json |
| E13 | residual | test | dla 0.1.0 | artifact series/S1.json (bf4ef4e48284) | gap/residual-S1-E13.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S5.json (10b152b859c6) | alloc/A2.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S7.json (7424b49e83c6) | alloc/A3.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S9.json (faf0d6e04a67) | alloc/A4.json |
| E17 | growth-series | test | dla 0.1.0 | artifact series/S2.json (be88e9e1f1cc) | trend/T6.json |
| E18 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (be88e9e1f1cc) | trend/T6.json |
| E19 | growth-series | test | dla 0.1.0 | artifact series/S4.json (caf2f62ccb88) | trend/T7.json |
| E20 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (caf2f62ccb88) | trend/T7.json |
| E21 | growth-series | test | dla 0.1.0 | artifact series/S6.json (7efb41229ea8) | trend/T8.json |
| E22 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (7efb41229ea8) | trend/T8.json |
| E23 | growth-series | test | dla 0.1.0 | artifact series/S8.json (cefdfee6d224) | trend/T9.json |
| E24 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (cefdfee6d224) | trend/T9.json |
| E25 | discriminator | test | dla 0.1.0 | artifact series/S11.json (aa9c6f123882) | alloc/A5.json |
| E26 | discriminator | test | dla 0.1.0 | artifact series/S13.json (f98b4886bfa5) | alloc/A6.json |
| E27 | discriminator | test | dla 0.1.0 | artifact series/S15.json (e4d3361acbe6) | alloc/A7.json |
| E28 | growth-series | test | dla 0.1.0 | artifact series/S10.json (5c2ce3d8463e) | trend/T13.json |
| E29 | unit-count-series | test | dla 0.1.0 | artifact series/S10.json (5c2ce3d8463e) | trend/T13.json |
| E30 | growth-series | test | dla 0.1.0 | artifact series/S12.json (f5ada80f56bb) | trend/T14.json |
| E31 | unit-count-series | test | dla 0.1.0 | artifact series/S12.json (f5ada80f56bb) | trend/T14.json |
| E32 | growth-series | test | dla 0.1.0 | artifact series/S14.json (afa0c7df365a) | trend/T15.json |
| E33 | unit-count-series | test | dla 0.1.0 | artifact series/S14.json (afa0c7df365a) | trend/T15.json |
| E34 | residual | test | dla 0.1.0 | artifact series/S1.json (bf4ef4e48284) | gap/residual-S1-E34.json |
| E35 | residual | test | dla 0.1.0 | artifact series/S2.json (be88e9e1f1cc) | gap/residual-S2-E35.json |
| E36 | residual | test | dla 0.1.0 | artifact series/S3.json (270929a1d1f7) | gap/residual-S3-E36.json |
| E37 | residual | test | dla 0.1.0 | artifact series/S4.json (caf2f62ccb88) | gap/residual-S4-E37.json |
| E38 | residual | test | dla 0.1.0 | artifact series/S5.json (10b152b859c6) | gap/residual-S5-E38.json |
| E39 | residual | test | dla 0.1.0 | artifact series/S6.json (7efb41229ea8) | gap/residual-S6-E39.json |
| E40 | residual | test | dla 0.1.0 | artifact series/S7.json (7424b49e83c6) | gap/residual-S7-E40.json |
| E41 | residual | test | dla 0.1.0 | artifact series/S8.json (cefdfee6d224) | gap/residual-S8-E41.json |
| E42 | residual | test | dla 0.1.0 | artifact series/S9.json (faf0d6e04a67) | gap/residual-S9-E42.json |
| E43 | residual | test | dla 0.1.0 | artifact series/S10.json (5c2ce3d8463e) | gap/residual-S10-E43.json |
| E44 | residual | test | dla 0.1.0 | artifact series/S11.json (aa9c6f123882) | gap/residual-S11-E44.json |
| E45 | residual | test | dla 0.1.0 | artifact series/S12.json (f5ada80f56bb) | gap/residual-S12-E45.json |
| E46 | residual | test | dla 0.1.0 | artifact series/S13.json (f98b4886bfa5) | gap/residual-S13-E46.json |
| E47 | residual | test | dla 0.1.0 | artifact series/S14.json (afa0c7df365a) | gap/residual-S14-E47.json |
| E48 | residual | test | dla 0.1.0 | artifact series/S4.json (caf2f62ccb88) | gap/residual-S4-E48.json |
| E49 | residual | test | dla 0.1.0 | artifact series/S10.json (5c2ce3d8463e) | gap/residual-S10-E49.json |
| E50 | residual | test | dla 0.1.0 | artifact series/S15.json (e4d3361acbe6) | gap/residual-S15-E50.json |
