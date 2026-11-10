# DLA report

Run: `20261110-191546-a47a`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 11 enumerated, 11 dispositioned; confirmed 0, strong 0, refuted 4, suspected 0, unresolved 7

Residual: heap 0 bytes/n unattributed of -0.13513513513513514 total (threshold 0.41386864864864864); heap 0.015315315315315317 bytes/n unattributed of 0.015315315315315317 total (threshold 0.6422691891891892); heap 0 bytes/n unattributed of -0.7675675675675676 total (threshold 4.442457297297297); heap 0.02882882882882883 bytes/n unattributed of 0.02882882882882883 total (threshold 0.42131027027027024); heap 0 bytes/n unattributed of -0.22117117117117116 total (threshold 2.7011369189189187); heap 0.032432432432432434 bytes/n unattributed of 0.032432432432432434 total (threshold 0.3171275675675675); heap 0 bytes/n unattributed of -0.8620928620928621 total (threshold 84.76066325155921); heap 0.24463320463320465 bytes/n unattributed of 0.24463320463320465 total (threshold 0.3914129420850106); heap 0 bytes/n unattributed of -0.3231231231231231 total (threshold 3.030737153153132); heap 1.9636636636636635 bytes/n unattributed of 1.9636636636636635 total (threshold 1.2602100000001333) ABOVE THRESHOLD, see O1; heap 0.2855162855162855 bytes/n unattributed of 0.2855162855162855 total (threshold 0.6716188274428034); heap 0.9024453024453024 bytes/n unattributed of 0.9024453024453024 total (threshold 1.2644214671814098); heap 0.218018018018018 bytes/n unattributed of 0.218018018018018 total (threshold 0.6844002162162162); heap 0.4168168168168168 bytes/n unattributed of 0.4168168168168168 total (threshold 0.6102129729731062); heap 0 bytes/n unattributed of -0.17135135135135135 total (threshold 0.25172143783791245); heap 0.03963963963963964 bytes/n unattributed of 0.03963963963963964 total (threshold 0.3514735135135135); heap 0 bytes/n unattributed of -1.0072072072072071 total (threshold 84.41817545945946); non-heap 6.021921921921922 bytes/n unattributed of 6.021921921921922 total (threshold 29.31769945945919); non-heap 0 bytes/n unattributed of -7.702822822822823 total (threshold 4.41986529729727); non-heap 0 bytes/n unattributed of -19.765765765765764 total (threshold 80.0961961081081); non-heap 0 bytes/n unattributed of -6.36962676962677 total (threshold 26.215933436293476); non-heap 0 bytes/n unattributed of -17.27850707850708 total (threshold 54.279417081080986); non-heap 6.890090090090091 bytes/n unattributed of 6.890090090090091 total (threshold 8.401018378378378); non-heap 0 bytes/n unattributed of -20.484684684684684 total (threshold 494.81895210810814); non-heap 5.796156156156156 bytes/n unattributed of 5.796156156156156 total (threshold 18.31833394594596); non-heap 0 bytes/n unattributed of -16.27063063063063 total (threshold 96.73700536216208); non-heap 15.155315315315317 bytes/n unattributed of 15.155315315315317 total (threshold 7.0340497297297295) ABOVE THRESHOLD, see O2; non-heap 5.255805805805806 bytes/n unattributed of 5.255805805805806 total (threshold 7.893019576576536); non-heap 16.85045045045045 bytes/n unattributed of 16.85045045045045 total (threshold 37.20352864864865); non-heap 3.896525096525097 bytes/n unattributed of 3.896525096525097 total (threshold 16.720751351351385); non-heap 0 bytes/n unattributed of -6.395675675675675 total (threshold 36.15071091891881); non-heap 1.7054697554697555 bytes/n unattributed of 1.7054697554697555 total (threshold 17.133446513513388); non-heap 0 bytes/n unattributed of -7.643243243243243 total (threshold 15.404156756756754); non-heap 0 bytes/n unattributed of -37.064864864864866 total (threshold 470.6894367567567)

Coverage: jvm 8880 iterations, 15 of 15 entry points exercised

## Findings

### F5: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: byte[] count growth in a short alloc-test series; no holder or site; heap_used plateaus and heap-graph over a longer series would attribute it
- Supporting evidence: E18

### F6: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: String/CHM node count growth in a short alloc-test series; alloc-test inconclusive (leak verdict came only from os.vm_hwm); rerun with --extra 16
- Supporting evidence: E24

### F7: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: JDK-type histogram count growth in a short alloc-test series; no holder or site, heap_used plateaus; rerun a longer series with heap-graph to attribute
- Supporting evidence: E26

### F8: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: JDK-type histogram count growth in a short alloc-test series; no holder or site, heap_used plateaus; rerun a longer series with heap-graph to attribute
- Supporting evidence: E34

### F9: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: JDK-internal type count (ConcurrentHashMap$Node/WeakReferenceKey, class-loading/instrument machinery) drifting in a short alloc-test series; no project holder or site; heap_used plateaus (N-OBSERVER/N-WARM suspected)
- Supporting evidence: E56

### F10: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: JDK-internal type count (ConcurrentHashMap$Node/WeakReferenceKey, class-loading/instrument machinery) drifting in a short alloc-test series; no project holder or site; heap_used plateaus (N-OBSERVER/N-WARM suspected)
- Supporting evidence: E62

### F11: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: JDK-internal type count (ConcurrentHashMap$Node/WeakReferenceKey, class-loading/instrument machinery) drifting in a short alloc-test series; no project holder or site; heap_used plateaus (N-OBSERVER/N-WARM suspected)
- Supporting evidence: E65

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (non-leak) E13
  - Contradicting evidence: E13
  - Originally supported by: E6
- **F2** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (non-leak) E14
  - Contradicting evidence: E14
  - Originally supported by: E6
- **F3** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (non-leak) E15
  - Contradicting evidence: E15
  - Originally supported by: E6
- **F4** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (non-leak) E16
  - Contradicting evidence: E16
  - Originally supported by: E6

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
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: points 1 to 9 come from S1; the extra points from S4
- E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E13: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: points 1 to 9 come from S1; the extra points from S6
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: points 1 to 9 come from S1; the extra points from S8
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: points 1 to 9 come from S1; the extra points from S10
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
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E26: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E27: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E27: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E27: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E27: limitation: points 1 to 9 come from S1; the extra points from S2
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E28: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E28: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E28: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E28: limitation: points 1 to 9 come from S1; the extra points from S2
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E29: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E29: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E29: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E29: limitation: points 1 to 9 come from S1; the extra points from S4
- E29: limitation: gate thresholds are provisional (not calibrated)
- E30: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E30: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E30: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E30: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E30: limitation: points 1 to 9 come from S1; the extra points from S4
- E30: limitation: gate thresholds are provisional (not calibrated)
- E31: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E31: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E31: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E31: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E31: limitation: points 1 to 9 come from S1; the extra points from S6
- E31: limitation: gate thresholds are provisional (not calibrated)
- E32: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E32: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E32: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E32: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E32: limitation: points 1 to 9 come from S1; the extra points from S6
- E32: limitation: gate thresholds are provisional (not calibrated)
- E33: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E33: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E33: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E33: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E33: limitation: points 1 to 9 come from S1; the extra points from S8
- E33: limitation: gate thresholds are provisional (not calibrated)
- E34: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E34: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E34: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E34: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E34: limitation: points 1 to 9 come from S1; the extra points from S8
- E34: limitation: gate thresholds are provisional (not calibrated)
- E35: partial coverage (2 of 3 examined)
- E35: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E35: limitation: threshold is provisional (not calibrated)
- E36: partial coverage (2 of 3 examined)
- E36: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E36: limitation: threshold is provisional (not calibrated)
- E37: partial coverage (2 of 3 examined)
- E37: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E37: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E37: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E37: limitation: threshold is provisional (not calibrated)
- E38: partial coverage (2 of 3 examined)
- E38: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E38: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E38: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E38: limitation: threshold is provisional (not calibrated)
- E39: partial coverage (2 of 3 examined)
- E39: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E39: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E39: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E39: limitation: threshold is provisional (not calibrated)
- E40: partial coverage (2 of 3 examined)
- E40: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E40: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E40: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E40: limitation: threshold is provisional (not calibrated)
- E41: partial coverage (2 of 3 examined)
- E41: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E41: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E41: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E41: limitation: threshold is provisional (not calibrated)
- E42: partial coverage (2 of 3 examined)
- E42: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E42: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E42: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E42: limitation: threshold is provisional (not calibrated)
- E43: partial coverage (2 of 3 examined)
- E43: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E43: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E43: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E43: limitation: threshold is provisional (not calibrated)
- E44: partial coverage (2 of 3 examined)
- E44: limitation: off-heap: no gap series for S10; run dla gap S10 first (node.external is used when present)
- E44: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E44: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E44: limitation: threshold is provisional (not calibrated)
- E45: partial coverage (2 of 3 examined)
- E45: limitation: off-heap: no gap series for S11; run dla gap S11 first (node.external is used when present)
- E45: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E45: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E45: limitation: threshold is provisional (not calibrated)
- E46: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E46: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E46: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E46: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E46: limitation: points 1 to 9 come from S1; the extra points from S12
- E46: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E46: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E46: limitation: gate and plateau thresholds are provisional (not calibrated)
- E47: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E47: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E47: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E47: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E47: limitation: points 1 to 9 come from S1; the extra points from S14
- E47: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E47: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E47: limitation: gate and plateau thresholds are provisional (not calibrated)
- E48: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E48: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E48: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E48: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E48: limitation: points 1 to 9 come from S1; the extra points from S16
- E48: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E48: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E48: limitation: gate and plateau thresholds are provisional (not calibrated)
- E49: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E49: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E49: limitation: gate thresholds are provisional (not calibrated)
- E50: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E50: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E50: limitation: gate thresholds are provisional (not calibrated)
- E51: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E51: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E51: limitation: gate thresholds are provisional (not calibrated)
- E52: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E52: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E52: limitation: gate thresholds are provisional (not calibrated)
- E53: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E53: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E53: limitation: gate thresholds are provisional (not calibrated)
- E54: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E54: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E54: limitation: gate thresholds are provisional (not calibrated)
- E55: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E55: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E55: limitation: gate thresholds are provisional (not calibrated)
- E56: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E56: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E56: limitation: gate thresholds are provisional (not calibrated)
- E57: partial coverage (2 of 3 examined)
- E57: limitation: off-heap: no gap series for S12; run dla gap S12 first (node.external is used when present)
- E57: limitation: threshold is provisional (not calibrated)
- E58: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E58: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E58: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E58: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E58: limitation: points 1 to 9 come from S1; the extra points from S12
- E58: limitation: gate thresholds are provisional (not calibrated)
- E59: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E59: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E59: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E59: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E59: limitation: points 1 to 9 come from S1; the extra points from S12
- E59: limitation: gate thresholds are provisional (not calibrated)
- E60: partial coverage (2 of 3 examined)
- E60: limitation: off-heap: no gap series for S13; run dla gap S13 first (node.external is used when present)
- E60: limitation: threshold is provisional (not calibrated)
- E61: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E61: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E61: limitation: gate thresholds are provisional (not calibrated)
- E62: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E62: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E62: limitation: gate thresholds are provisional (not calibrated)
- E63: partial coverage (2 of 3 examined)
- E63: limitation: off-heap: no gap series for S14; run dla gap S14 first (node.external is used when present)
- E63: limitation: threshold is provisional (not calibrated)
- E64: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E64: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E64: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E64: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E64: limitation: points 1 to 9 come from S1; the extra points from S14
- E64: limitation: gate thresholds are provisional (not calibrated)
- E65: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E65: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E65: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E65: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E65: limitation: points 1 to 9 come from S1; the extra points from S14
- E65: limitation: gate thresholds are provisional (not calibrated)
- E66: partial coverage (2 of 3 examined)
- E66: limitation: off-heap: no gap series for S15; run dla gap S15 first (node.external is used when present)
- E66: limitation: threshold is provisional (not calibrated)
- E67: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E67: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E67: limitation: gate thresholds are provisional (not calibrated)
- E68: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E68: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E68: limitation: gate thresholds are provisional (not calibrated)
- E69: partial coverage (2 of 3 examined)
- E69: limitation: off-heap: no gap series for S16; run dla gap S16 first (node.external is used when present)
- E69: limitation: threshold is provisional (not calibrated)
- E70: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E70: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E70: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E70: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E70: limitation: points 1 to 9 come from S1; the extra points from S16
- E70: limitation: gate thresholds are provisional (not calibrated)
- E71: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E71: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E71: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E71: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E71: limitation: points 1 to 9 come from S1; the extra points from S16
- E71: limitation: gate thresholds are provisional (not calibrated)
- E72: partial coverage (2 of 3 examined)
- E72: limitation: off-heap: no gap series for S17; run dla gap S17 first (node.external is used when present)
- E72: limitation: threshold is provisional (not calibrated)

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
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (25f2aea2dab7) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (9ced1cf61900) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (9ced1cf61900) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (34f7aa84fcdb) | alloc/A1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S5.json (0620cf0b98b6) | alloc/A2.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S7.json (4e7fd3cd7ca5) | alloc/A3.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S9.json (58f8f6633273) | alloc/A4.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S11.json (d349368fbceb) | alloc/A5.json |
| E17 | growth-series | test | dla 0.1.0 | artifact series/S2.json (1871ee9db59c) | trend/T7.json |
| E18 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (1871ee9db59c) | trend/T7.json |
| E19 | growth-series | test | dla 0.1.0 | artifact series/S4.json (57a56076da7e) | trend/T8.json |
| E20 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (57a56076da7e) | trend/T8.json |
| E21 | growth-series | test | dla 0.1.0 | artifact series/S6.json (5a53b04e98a5) | trend/T9.json |
| E22 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (5a53b04e98a5) | trend/T9.json |
| E23 | growth-series | test | dla 0.1.0 | artifact series/S8.json (e20596a9ad4f) | trend/T10.json |
| E24 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (e20596a9ad4f) | trend/T10.json |
| E25 | growth-series | test | dla 0.1.0 | artifact series/S10.json (bed616ba3990) | trend/T11.json |
| E26 | unit-count-series | test | dla 0.1.0 | artifact series/S10.json (bed616ba3990) | trend/T11.json |
| E27 | growth-series | test | dla 0.1.0 | artifact series/S3.json (34f7aa84fcdb) | trend/T12.json |
| E28 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (34f7aa84fcdb) | trend/T12.json |
| E29 | growth-series | test | dla 0.1.0 | artifact series/S5.json (0620cf0b98b6) | trend/T13.json |
| E30 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (0620cf0b98b6) | trend/T13.json |
| E31 | growth-series | test | dla 0.1.0 | artifact series/S7.json (4e7fd3cd7ca5) | trend/T14.json |
| E32 | unit-count-series | test | dla 0.1.0 | artifact series/S7.json (4e7fd3cd7ca5) | trend/T14.json |
| E33 | growth-series | test | dla 0.1.0 | artifact series/S9.json (58f8f6633273) | trend/T15.json |
| E34 | unit-count-series | test | dla 0.1.0 | artifact series/S9.json (58f8f6633273) | trend/T15.json |
| E35 | residual | test | dla 0.1.0 | artifact series/S1.json (9ced1cf61900) | gap/residual-S1-E35.json |
| E36 | residual | test | dla 0.1.0 | artifact series/S2.json (1871ee9db59c) | gap/residual-S2-E36.json |
| E37 | residual | test | dla 0.1.0 | artifact series/S3.json (34f7aa84fcdb) | gap/residual-S3-E37.json |
| E38 | residual | test | dla 0.1.0 | artifact series/S4.json (57a56076da7e) | gap/residual-S4-E38.json |
| E39 | residual | test | dla 0.1.0 | artifact series/S5.json (0620cf0b98b6) | gap/residual-S5-E39.json |
| E40 | residual | test | dla 0.1.0 | artifact series/S6.json (5a53b04e98a5) | gap/residual-S6-E40.json |
| E41 | residual | test | dla 0.1.0 | artifact series/S7.json (4e7fd3cd7ca5) | gap/residual-S7-E41.json |
| E42 | residual | test | dla 0.1.0 | artifact series/S8.json (e20596a9ad4f) | gap/residual-S8-E42.json |
| E43 | residual | test | dla 0.1.0 | artifact series/S9.json (58f8f6633273) | gap/residual-S9-E43.json |
| E44 | residual | test | dla 0.1.0 | artifact series/S10.json (bed616ba3990) | gap/residual-S10-E44.json |
| E45 | residual | test | dla 0.1.0 | artifact series/S11.json (d349368fbceb) | gap/residual-S11-E45.json |
| E46 | discriminator | test | dla 0.1.0 | artifact series/S13.json (e863b1d87edd) | alloc/A6.json |
| E47 | discriminator | test | dla 0.1.0 | artifact series/S15.json (7ddcab6a84f9) | alloc/A7.json |
| E48 | discriminator | test | dla 0.1.0 | artifact series/S17.json (516c451791d8) | alloc/A8.json |
| E49 | growth-series | test | dla 0.1.0 | artifact series/S12.json (e5c09909d4bf) | trend/T19.json |
| E50 | unit-count-series | test | dla 0.1.0 | artifact series/S12.json (e5c09909d4bf) | trend/T19.json |
| E51 | growth-series | test | dla 0.1.0 | artifact series/S14.json (ec3d33047a27) | trend/T20.json |
| E52 | unit-count-series | test | dla 0.1.0 | artifact series/S14.json (ec3d33047a27) | trend/T20.json |
| E53 | growth-series | test | dla 0.1.0 | artifact series/S16.json (dd16cf27ea62) | trend/T21.json |
| E54 | unit-count-series | test | dla 0.1.0 | artifact series/S16.json (dd16cf27ea62) | trend/T21.json |
| E55 | growth-series | test | dla 0.1.0 | artifact series/S12.json (e5c09909d4bf) | trend/T22.json |
| E56 | unit-count-series | test | dla 0.1.0 | artifact series/S12.json (e5c09909d4bf) | trend/T22.json |
| E57 | residual | test | dla 0.1.0 | artifact series/S12.json (e5c09909d4bf) | gap/residual-S12-E57.json |
| E58 | growth-series | test | dla 0.1.0 | artifact series/S13.json (e863b1d87edd) | trend/T23.json |
| E59 | unit-count-series | test | dla 0.1.0 | artifact series/S13.json (e863b1d87edd) | trend/T23.json |
| E60 | residual | test | dla 0.1.0 | artifact series/S13.json (e863b1d87edd) | gap/residual-S13-E60.json |
| E61 | growth-series | test | dla 0.1.0 | artifact series/S14.json (ec3d33047a27) | trend/T24.json |
| E62 | unit-count-series | test | dla 0.1.0 | artifact series/S14.json (ec3d33047a27) | trend/T24.json |
| E63 | residual | test | dla 0.1.0 | artifact series/S14.json (ec3d33047a27) | gap/residual-S14-E63.json |
| E64 | growth-series | test | dla 0.1.0 | artifact series/S15.json (7ddcab6a84f9) | trend/T25.json |
| E65 | unit-count-series | test | dla 0.1.0 | artifact series/S15.json (7ddcab6a84f9) | trend/T25.json |
| E66 | residual | test | dla 0.1.0 | artifact series/S15.json (7ddcab6a84f9) | gap/residual-S15-E66.json |
| E67 | growth-series | test | dla 0.1.0 | artifact series/S16.json (dd16cf27ea62) | trend/T26.json |
| E68 | unit-count-series | test | dla 0.1.0 | artifact series/S16.json (dd16cf27ea62) | trend/T26.json |
| E69 | residual | test | dla 0.1.0 | artifact series/S16.json (dd16cf27ea62) | gap/residual-S16-E69.json |
| E70 | growth-series | test | dla 0.1.0 | artifact series/S17.json (516c451791d8) | trend/T27.json |
| E71 | unit-count-series | test | dla 0.1.0 | artifact series/S17.json (516c451791d8) | trend/T27.json |
| E72 | residual | test | dla 0.1.0 | artifact series/S17.json (516c451791d8) | gap/residual-S17-E72.json |
