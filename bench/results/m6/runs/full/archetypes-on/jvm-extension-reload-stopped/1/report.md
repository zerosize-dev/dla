# DLA report

Run: `20261110-203618-cc0b`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 7 enumerated, 7 dispositioned; confirmed 0, strong 0, refuted 5, suspected 0, unresolved 2

Residual: heap 14.002272727272729 bytes/n unattributed of 14.002272727272729 total (threshold 31.13604409090909); heap 29.6 bytes/n unattributed of 29.6 total (threshold 24.647743636363636) ABOVE THRESHOLD, see O2; heap 15.368181818181816 bytes/n unattributed of 15.368181818181816 total (threshold 10.803436636363635) ABOVE THRESHOLD, see O2; heap 0 bytes/n unattributed of -7.115785123966942 total (threshold 26.692620123967018); heap 0.203030303030303 bytes/n unattributed of 0.203030303030303 total (threshold 18.495874505928818); heap 19.077272727272728 bytes/n unattributed of 19.077272727272728 total (threshold 4.769318181818182) ABOVE THRESHOLD, see O3; heap 14.930027548209369 bytes/n unattributed of 14.930027548209369 total (threshold 7.184361024793402) ABOVE THRESHOLD, see O6; heap 17.263636363636365 bytes/n unattributed of 17.263636363636365 total (threshold 4.315909090909091) ABOVE THRESHOLD, see O2; heap 15.255578512396694 bytes/n unattributed of 15.255578512396694 total (threshold 6.591535450413038) ABOVE THRESHOLD, see O2; heap 24.409090909090907 bytes/n unattributed of 24.409090909090907 total (threshold 14.992551818181818) ABOVE THRESHOLD, see O2; heap 14.246590909090909 bytes/n unattributed of 14.246590909090909 total (threshold 11.630660045454546) ABOVE THRESHOLD, see O2; heap 18.752987012987013 bytes/n unattributed of 18.752987012987013 total (threshold 4.688246753246753) ABOVE THRESHOLD, see O2; heap 14.9 bytes/n unattributed of 14.9 total (threshold 9.247380545454543) ABOVE THRESHOLD, see O2; heap 16.93409090909091 bytes/n unattributed of 16.93409090909091 total (threshold 4.3315506818181815) ABOVE THRESHOLD, see O2; heap 14.993939393939392 bytes/n unattributed of 14.993939393939392 total (threshold 6.051703636363636) ABOVE THRESHOLD, see O2; non-heap 207.9884848484849 bytes/n unattributed of 207.9884848484849 total (threshold 71.01422945454554) ABOVE THRESHOLD, see O1; non-heap 12.238636363636363 bytes/n unattributed of 12.238636363636363 total (threshold 68.20657977272727); non-heap 33.102777777777774 bytes/n unattributed of 33.102777777777774 total (threshold 54.14256771212086); non-heap 6.042424242424243 bytes/n unattributed of 6.042424242424243 total (threshold 40.87752836363615); non-heap 18.37909090909091 bytes/n unattributed of 18.37909090909091 total (threshold 34.56758081027668); non-heap 0 bytes/n unattributed of -21.17194805194805 total (threshold 22.681139376623417); non-heap 23.872348484848487 bytes/n unattributed of 23.872348484848487 total (threshold 65.36081411363615); non-heap 0 bytes/n unattributed of -14.877272727272729 total (threshold 86.88084136363636); non-heap 32.85867768595041 bytes/n unattributed of 32.85867768595041 total (threshold 76.21372690909091); non-heap 0 bytes/n unattributed of -37.81590909090909 total (threshold 61.3240425); non-heap 23.418181818181814 bytes/n unattributed of 23.418181818181814 total (threshold 71.1270610909091); non-heap 54.391666666666666 bytes/n unattributed of 54.391666666666666 total (threshold 38.20270295454545) ABOVE THRESHOLD, see O1; non-heap 57.27426210153483 bytes/n unattributed of 57.27426210153483 total (threshold 43.20588719008243) ABOVE THRESHOLD, see O1; non-heap 0 bytes/n unattributed of -1.2606060606060605 total (threshold 90.82946727272682); non-heap 33.34015151515152 bytes/n unattributed of 33.34015151515152 total (threshold 77.79954731818202)

Coverage: jvm 3520 iterations, 19 of 19 entry points exercised

## Findings

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: stair
- Hypothesis: Class/int[] counts step with warm-up class loading, not a leak
- Reason: not_exercised
- Remedy: Class/int[] count steps in a short warm-up harness series; jvm.heap_used plateaus (E45). A longer soak would settle it.
- Supporting evidence: E44

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: stair
- Hypothesis: byte[]/String counts step with warm-up class loading, not a leak
- Reason: not_exercised
- Remedy: String/byte[] counts step in a short warm-up harness; jvm.heap_used plateaus (E45). Longer soak would settle it.
- Supporting evidence: E47

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E19
  - Ruled out by: plateau+purge (non-leak) E19
  - Contradicting evidence: E19
  - Originally supported by: E6
- **F2** (UNCLASSIFIED): discriminator E17
  - Ruled out by: plateau+purge (non-leak) E17
  - Contradicting evidence: E17
  - Originally supported by: E6
- **F3** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (non-leak) E15
  - Contradicting evidence: E15
  - Originally supported by: E6
- **F4** (UNCLASSIFIED): discriminator E18
  - Ruled out by: plateau+purge (non-leak) E18
  - Hypothesis: x
  - Contradicting evidence: E18
  - Originally supported by: E11
- **F5** (UNCLASSIFIED): discriminator E45
  - Ruled out by: plateau+purge (non-leak) E45
  - Hypothesis: String/byte[] counts step with class loading during warm-up, not a leak
  - Contradicting evidence: E45
  - Originally supported by: E23

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
- E9: limitation: --capture alloc is not written by jvm harnesses
- E9: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E9: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E9: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E10: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E10: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E10: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E11: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: points 1 to 9 come from S1; the extra points from S2
- E12: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E12: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E12: limitation: gate and plateau thresholds are provisional (not calibrated)
- E13: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: points 1 to 9 come from S1; the extra points from S4
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: points 1 to 9 come from S1; the extra points from S6
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E16: limitation: column probe:ledger.LedgerExtension.ROUNDING has 0 of 8 main-window readings (missing values); its plateau test used what exists
- E16: limitation: column probe:ledger.LedgerExtension.ROUNDING.bytes has 0 of 8 main-window readings (missing values); its plateau test used what exists
- E16: limitation: column probe:ledger.LedgerExtension.ROUNDING.size has 0 of 8 main-window readings (missing values); its plateau test used what exists
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E17: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E17: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: points 1 to 9 come from S1; the extra points from S8
- E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E17: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E17: limitation: gate and plateau thresholds are provisional (not calibrated)
- E18: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E18: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E18: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: points 1 to 9 come from S1; the extra points from S10
- E18: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E18: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E18: limitation: gate and plateau thresholds are provisional (not calibrated)
- E19: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: points 1 to 9 come from S1; the extra points from S12
- E19: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E19: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E19: limitation: gate and plateau thresholds are provisional (not calibrated)
- E20: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E23: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E24: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E24: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E25: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E25: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E26: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E26: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E27: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E27: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E28: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E28: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E29: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E29: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E29: limitation: gate thresholds are provisional (not calibrated)
- E30: partial coverage (2 of 3 examined)
- E30: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E30: limitation: threshold is provisional (not calibrated)
- E31: partial coverage (2 of 3 examined)
- E31: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E31: limitation: O1 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E31: limitation: threshold is provisional (not calibrated)
- E32: partial coverage (2 of 3 examined)
- E32: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E32: limitation: O1 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E32: limitation: threshold is provisional (not calibrated)
- E33: partial coverage (2 of 3 examined)
- E33: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E33: limitation: O1 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E33: limitation: threshold is provisional (not calibrated)
- E34: partial coverage (2 of 3 examined)
- E34: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E34: limitation: O1 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E34: limitation: threshold is provisional (not calibrated)
- E35: partial coverage (2 of 3 examined)
- E35: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E35: limitation: threshold is provisional (not calibrated)
- E36: partial coverage (2 of 3 examined)
- E36: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E36: limitation: threshold is provisional (not calibrated)
- E37: partial coverage (2 of 3 examined)
- E37: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E37: limitation: O1 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E37: limitation: threshold is provisional (not calibrated)
- E38: partial coverage (2 of 3 examined)
- E38: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E38: limitation: O1 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E38: limitation: threshold is provisional (not calibrated)
- E39: partial coverage (2 of 3 examined)
- E39: limitation: off-heap: no gap series for S10; run dla gap S10 first (node.external is used when present)
- E39: limitation: O1 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E39: limitation: threshold is provisional (not calibrated)
- E40: partial coverage (2 of 3 examined)
- E40: limitation: off-heap: no gap series for S11; run dla gap S11 first (node.external is used when present)
- E40: limitation: O1 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E40: limitation: threshold is provisional (not calibrated)
- E41: partial coverage (2 of 3 examined)
- E41: limitation: off-heap: no gap series for S12; run dla gap S12 first (node.external is used when present)
- E41: limitation: O2 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E41: limitation: O1 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E41: limitation: threshold is provisional (not calibrated)
- E42: partial coverage (2 of 3 examined)
- E42: limitation: off-heap: no gap series for S13; run dla gap S13 first (node.external is used when present)
- E42: limitation: O2 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E42: limitation: O1 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E42: limitation: threshold is provisional (not calibrated)
- E43: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E43: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E43: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E43: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E43: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E43: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E43: limitation: points 1 to 9 come from S1; the extra points from S12
- E43: limitation: gate thresholds are provisional (not calibrated)
- E44: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E44: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E44: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E44: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E44: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E44: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E44: limitation: points 1 to 9 come from S1; the extra points from S12
- E44: limitation: gate thresholds are provisional (not calibrated)
- E45: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E45: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E45: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E45: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E45: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E45: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E45: limitation: points 1 to 9 come from S1; the extra points from S14
- E45: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E45: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E45: limitation: gate and plateau thresholds are provisional (not calibrated)
- E46: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E46: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E46: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E46: limitation: gate thresholds are provisional (not calibrated)
- E47: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E47: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E47: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E47: limitation: gate thresholds are provisional (not calibrated)
- E48: partial coverage (2 of 3 examined)
- E48: limitation: off-heap: no gap series for S15; run dla gap S15 first (node.external is used when present)
- E48: limitation: threshold is provisional (not calibrated)
- E49: partial coverage (2 of 3 examined)
- E49: limitation: off-heap: no gap series for S14; run dla gap S14 first (node.external is used when present)
- E49: limitation: threshold is provisional (not calibrated)
- E50: partial coverage (2 of 3 examined)
- E50: limitation: off-heap: no gap series for S15; run dla gap S15 first (node.external is used when present)
- E50: limitation: threshold is provisional (not calibrated)
- E51: partial coverage (2 of 3 examined)
- E51: limitation: off-heap: no gap series for S15; run dla gap S15 first (node.external is used when present)
- E51: limitation: threshold is provisional (not calibrated)
- E52: partial coverage (2 of 3 examined)
- E52: limitation: off-heap: no gap series for S15; run dla gap S15 first (node.external is used when present)
- E52: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | — |
| E2 | run-config | static | dla 0.1.0 | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | probe/probe.json |
| E4 | static-hit | static | spotbugs unknown | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | sweep/jvm-spotbugs.json |
| E5 | static-hit | static | dla 0.1.0 | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | sweep/jvm-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | census/jvm.json |
| E7 | workload | static | dla 0.1.0 | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (0cd1325929e6) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (c982f37ea0af) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (c982f37ea0af) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (dcc081fb64c8) | alloc/A1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S2.json (cffd81d192c4) | trend/T3.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (cffd81d192c4) | trend/T3.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S5.json (6fc755c316f5) | alloc/A2.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S7.json (7d1270184205) | alloc/A3.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S9.json (0d6ce1cb2f26) | alloc/A4.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S11.json (9eb7dd35bfb6) | alloc/A5.json |
| E19 | discriminator | test | dla 0.1.0 | artifact series/S13.json (e45bb1ce3c33) | alloc/A6.json |
| E20 | growth-series | test | dla 0.1.0 | artifact series/S4.json (a71509fb9170) | trend/T9.json |
| E21 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (a71509fb9170) | trend/T9.json |
| E22 | growth-series | test | dla 0.1.0 | artifact series/S6.json (6157242a0c8a) | trend/T10.json |
| E23 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (6157242a0c8a) | trend/T10.json |
| E24 | growth-series | test | dla 0.1.0 | artifact series/S8.json (c29e4a6946c3) | trend/T11.json |
| E25 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (c29e4a6946c3) | trend/T11.json |
| E26 | growth-series | test | dla 0.1.0 | artifact series/S10.json (34039aaaefbd) | trend/T12.json |
| E27 | unit-count-series | test | dla 0.1.0 | artifact series/S10.json (34039aaaefbd) | trend/T12.json |
| E28 | growth-series | test | dla 0.1.0 | artifact series/S12.json (977538d76c19) | trend/T13.json |
| E29 | unit-count-series | test | dla 0.1.0 | artifact series/S12.json (977538d76c19) | trend/T13.json |
| E30 | residual | test | dla 0.1.0 | artifact series/S1.json (c982f37ea0af) | gap/residual-S1-E30.json |
| E31 | residual | test | dla 0.1.0 | artifact series/S2.json (cffd81d192c4) | gap/residual-S2-E31.json |
| E32 | residual | test | dla 0.1.0 | artifact series/S3.json (dcc081fb64c8) | gap/residual-S3-E32.json |
| E33 | residual | test | dla 0.1.0 | artifact series/S4.json (a71509fb9170) | gap/residual-S4-E33.json |
| E34 | residual | test | dla 0.1.0 | artifact series/S5.json (6fc755c316f5) | gap/residual-S5-E34.json |
| E35 | residual | test | dla 0.1.0 | artifact series/S6.json (6157242a0c8a) | gap/residual-S6-E35.json |
| E36 | residual | test | dla 0.1.0 | artifact series/S7.json (7d1270184205) | gap/residual-S7-E36.json |
| E37 | residual | test | dla 0.1.0 | artifact series/S8.json (c29e4a6946c3) | gap/residual-S8-E37.json |
| E38 | residual | test | dla 0.1.0 | artifact series/S9.json (0d6ce1cb2f26) | gap/residual-S9-E38.json |
| E39 | residual | test | dla 0.1.0 | artifact series/S10.json (34039aaaefbd) | gap/residual-S10-E39.json |
| E40 | residual | test | dla 0.1.0 | artifact series/S11.json (9eb7dd35bfb6) | gap/residual-S11-E40.json |
| E41 | residual | test | dla 0.1.0 | artifact series/S12.json (977538d76c19) | gap/residual-S12-E41.json |
| E42 | residual | test | dla 0.1.0 | artifact series/S13.json (e45bb1ce3c33) | gap/residual-S13-E42.json |
| E43 | growth-series | test | dla 0.1.0 | artifact series/S13.json (e45bb1ce3c33) | trend/T14.json |
| E44 | unit-count-series | test | dla 0.1.0 | artifact series/S13.json (e45bb1ce3c33) | trend/T14.json |
| E45 | discriminator | test | dla 0.1.0 | artifact series/S15.json (c6c36b9a6dd4) | alloc/A7.json |
| E46 | growth-series | test | dla 0.1.0 | artifact series/S14.json (b3ab1f013d44) | trend/T16.json |
| E47 | unit-count-series | test | dla 0.1.0 | artifact series/S14.json (b3ab1f013d44) | trend/T16.json |
| E48 | residual | test | dla 0.1.0 | artifact series/S15.json (c6c36b9a6dd4) | gap/residual-S15-E48.json |
| E49 | residual | test | dla 0.1.0 | artifact series/S14.json (b3ab1f013d44) | gap/residual-S14-E49.json |
| E50 | residual | test | dla 0.1.0 | artifact series/S15.json (c6c36b9a6dd4) | gap/residual-S15-E50.json |
| E51 | residual | test | dla 0.1.0 | artifact series/S15.json (c6c36b9a6dd4) | gap/residual-S15-E51.json |
| E52 | residual | test | dla 0.1.0 | artifact series/S15.json (c6c36b9a6dd4) | gap/residual-S15-E52.json |
