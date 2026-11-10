# DLA report

Run: `20261110-190734-75ac`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 8 enumerated, 8 dispositioned; confirmed 0, strong 0, refuted 2, suspected 0, unresolved 6

Residual: heap 15.993333333333332 bytes/n unattributed of 15.993333333333332 total (threshold 17.87457218181827); heap 15.731818181818182 bytes/n unattributed of 15.731818181818182 total (threshold 6.791559545454545) ABOVE THRESHOLD, see O8; heap 14.622943722943724 bytes/n unattributed of 14.622943722943724 total (threshold 4.322517090909031) ABOVE THRESHOLD, see O8; heap 8.781818181818181 bytes/n unattributed of 8.781818181818181 total (threshold 22.920610909090907); heap 13.023140495867768 bytes/n unattributed of 13.023140495867768 total (threshold 9.144333719008364) ABOVE THRESHOLD, see O8; heap 34.29512987012987 bytes/n unattributed of 34.29512987012987 total (threshold 19.732470779221067) ABOVE THRESHOLD, see O8; heap 14.120227272727274 bytes/n unattributed of 14.120227272727274 total (threshold 15.142097059090942); heap 25.775757575757577 bytes/n unattributed of 25.775757575757577 total (threshold 6.443939393939393) ABOVE THRESHOLD, see O8; heap 17.824242424242424 bytes/n unattributed of 17.824242424242424 total (threshold 5.77764727272722) ABOVE THRESHOLD, see O6; heap 20.14590909090909 bytes/n unattributed of 20.14590909090909 total (threshold 15.89178722727266) ABOVE THRESHOLD, see O8; heap 15.5993006993007 bytes/n unattributed of 15.5993006993007 total (threshold 6.7523617342656985) ABOVE THRESHOLD, see O8; heap 28.38939393939394 bytes/n unattributed of 28.38939393939394 total (threshold 21.658956818181704) ABOVE THRESHOLD, see O6; heap 17.778030303030302 bytes/n unattributed of 17.778030303030302 total (threshold 9.7814535) ABOVE THRESHOLD, see O6; heap 18.815151515151516 bytes/n unattributed of 18.815151515151516 total (threshold 4.703787878787879) ABOVE THRESHOLD, see O8; heap 15.835757575757576 bytes/n unattributed of 15.835757575757576 total (threshold 4.291003818181766) ABOVE THRESHOLD, see O8; non-heap 174.0872727272727 bytes/n unattributed of 174.0872727272727 total (threshold 162.18989345454384) ABOVE THRESHOLD, see O1; non-heap 0 bytes/n unattributed of -18.71818181818182 total (threshold 44.30759727272727); non-heap 21.314545454545453 bytes/n unattributed of 21.314545454545453 total (threshold 40.061469381818426); non-heap 24.77792207792208 bytes/n unattributed of 24.77792207792208 total (threshold 67.95190402597518); non-heap 34.894380165289256 bytes/n unattributed of 34.894380165289256 total (threshold 36.709690621487624); non-heap 0 bytes/n unattributed of -3.3969696969696974 total (threshold 54.96980181818272); non-heap 27.4375 bytes/n unattributed of 27.4375 total (threshold 29.921395159090903); non-heap 116.39272727272729 bytes/n unattributed of 116.39272727272729 total (threshold 92.009385818181) ABOVE THRESHOLD, see O9; non-heap 79.60136363636363 bytes/n unattributed of 79.60136363636363 total (threshold 62.37621676363624) ABOVE THRESHOLD, see O7; non-heap 0 bytes/n unattributed of -11.292424242424245 total (threshold 32.36063318181817); non-heap 18.796681096681095 bytes/n unattributed of 18.796681096681095 total (threshold 43.058148424242354); non-heap 20.61038961038961 bytes/n unattributed of 20.61038961038961 total (threshold 86.69579142857161); non-heap 45.64772727272727 bytes/n unattributed of 45.64772727272727 total (threshold 62.891555045454545); non-heap 0 bytes/n unattributed of -9.923484848484849 total (threshold 59.358875454545895); non-heap 28.765252525252528 bytes/n unattributed of 28.765252525252528 total (threshold 37.28484412121198)

Coverage: jvm 3520 iterations, 19 of 19 entry points exercised

## Findings

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `ledger.LedgerExtension.ROUNDING`
- Region: heap
- Site: `ext/ledger/LedgerExtension.java:11`
- Reason: not_exercised
- Remedy: static ROUNDING table lives in the extension classloader; class count stays flat so it is collected; confirm with a same-process heap-graph series
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: stair
- Locus: runtime
- Hypothesis: JDK/JIT warm-up in class histogram; extension types absent from every histogram
- Refuted if: extension or listener types appear or grow across histograms
- Reason: not_exercised
- Remedy: heap_used/os.rss creep (~100KB heap, ~100MB rss over 7000 reloads) is process-wide with no holder; take 3+ post-GC heap dumps from one process via a harness driver, and an NMT baseline/diff series, to attribute
- Supporting evidence: E15, E16

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: stair
- Locus: runtime
- Hypothesis: JDK type-count wobble between alternating capture points; not an extension type
- Refuted if: extension or listener types grow
- Reason: not_exercised
- Remedy: JDK-internal String/byte[]/int[] count step between capture points (no extension type in histograms, classloader count flat); a same-process heap-graph series would close it
- Supporting evidence: E31

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: stair
- Locus: runtime
- Hypothesis: JDK type-count wobble between alternating capture points; not an extension type
- Refuted if: extension or listener types grow
- Reason: not_exercised
- Remedy: JDK-internal String/byte[]/int[] count step between capture points (no extension type in histograms, classloader count flat); a same-process heap-graph series would close it
- Supporting evidence: E33

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: stair
- Locus: runtime
- Hypothesis: JDK type-count wobble between alternating capture points; not an extension type
- Refuted if: extension or listener types grow
- Reason: not_exercised
- Remedy: JDK-internal String/byte[]/int[] count step between capture points (no extension type in histograms, classloader count flat); a same-process heap-graph series would close it
- Supporting evidence: E37

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: stair
- Locus: runtime
- Hypothesis: JDK type-count wobble between alternating capture points; not an extension type
- Refuted if: extension or listener types grow
- Reason: not_exercised
- Remedy: JDK-internal String/byte[] count step; same-process heap-graph series would close it
- Supporting evidence: E41

## Ruled out

- **F2** (UNCLASSIFIED): discriminator E24
  - Ruled out by: plateau+purge (non-leak) E24
  - Contradicting evidence: E24
  - Originally supported by: E6
- **F3** (UNCLASSIFIED): discriminator E19
  - Ruled out by: plateau+purge (non-leak) E19
  - Contradicting evidence: E19
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
- E12: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E12: limitation: the files carry no time, so they are ordered as given
- E13: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E13: limitation: the files carry no time, so they are ordered as given
- E13: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E14: partial coverage (2 of 3 examined)
- E14: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E14: limitation: threshold is provisional (not calibrated)
- E15: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E15: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E16: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E17: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: gap.unexplained is negative at 9 point(s): the accounted figure exceeds os.rss (reserved but untouched memory)
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E18: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E18: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: points 1 to 9 come from S1; the extra points from S4
- E18: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E18: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E18: limitation: gate and plateau thresholds are provisional (not calibrated)
- E19: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: points 1 to 9 come from S1; the extra points from S6
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
- E22: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: points 1 to 9 come from S1; the extra points from S8
- E22: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E22: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E22: limitation: gate and plateau thresholds are provisional (not calibrated)
- E23: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E23: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E23: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E23: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E23: limitation: points 1 to 9 come from S1; the extra points from S10
- E23: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E23: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E23: limitation: column probe:ledger.LedgerExtension.ROUNDING has 0 of 8 main-window readings (missing values); its plateau test used what exists
- E23: limitation: column probe:ledger.LedgerExtension.ROUNDING.bytes has 0 of 8 main-window readings (missing values); its plateau test used what exists
- E23: limitation: column probe:ledger.LedgerExtension.ROUNDING.size has 0 of 8 main-window readings (missing values); its plateau test used what exists
- E23: limitation: gate and plateau thresholds are provisional (not calibrated)
- E24: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E24: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E24: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E24: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E24: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E24: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E24: limitation: points 1 to 9 come from S1; the extra points from S12
- E24: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E24: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E24: limitation: gate and plateau thresholds are provisional (not calibrated)
- E25: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E25: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E25: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E25: limitation: cgroup.*: /proc/44/cgroup unreadable (No such file or directory (os error 2))
- E25: limitation: os.fds: /proc/44/fd unreadable (No such file or directory (os error 2))
- E25: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E25: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E25: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E25: limitation: points 1 to 9 come from S1; the extra points from S14
- E25: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E25: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E25: limitation: gate and plateau thresholds are provisional (not calibrated)
- E26: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E26: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E26: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E26: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E26: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E26: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E26: limitation: points 1 to 9 come from S1; the extra points from S16
- E26: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E26: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E26: limitation: gate and plateau thresholds are provisional (not calibrated)
- E27: limitation: loaders and classes are whole-runtime counts: they never tie to a finding
- E28: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E28: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E28: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E29: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E29: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E29: limitation: gate thresholds are provisional (not calibrated)
- E30: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E30: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E30: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E30: limitation: gate thresholds are provisional (not calibrated)
- E31: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E31: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E31: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E31: limitation: gate thresholds are provisional (not calibrated)
- E32: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E32: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E32: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E32: limitation: gate thresholds are provisional (not calibrated)
- E33: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E33: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E33: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E33: limitation: gate thresholds are provisional (not calibrated)
- E34: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E34: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E34: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E34: limitation: gate thresholds are provisional (not calibrated)
- E35: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E35: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E35: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E35: limitation: gate thresholds are provisional (not calibrated)
- E36: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E36: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E36: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E36: limitation: gate thresholds are provisional (not calibrated)
- E37: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E37: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E37: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E37: limitation: gate thresholds are provisional (not calibrated)
- E38: limitation: cgroup.*: /proc/44/cgroup unreadable (No such file or directory (os error 2))
- E38: limitation: os.fds: /proc/44/fd unreadable (No such file or directory (os error 2))
- E38: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E38: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E38: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E38: limitation: gate thresholds are provisional (not calibrated)
- E39: limitation: cgroup.*: /proc/44/cgroup unreadable (No such file or directory (os error 2))
- E39: limitation: os.fds: /proc/44/fd unreadable (No such file or directory (os error 2))
- E39: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E39: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E39: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E39: limitation: gate thresholds are provisional (not calibrated)
- E40: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E40: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E40: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E40: limitation: gate thresholds are provisional (not calibrated)
- E41: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E41: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E41: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E41: limitation: gate thresholds are provisional (not calibrated)
- E42: partial coverage (2 of 3 examined)
- E42: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E42: limitation: threshold is provisional (not calibrated)
- E43: partial coverage (2 of 3 examined)
- E43: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E43: limitation: threshold is provisional (not calibrated)
- E44: partial coverage (2 of 3 examined)
- E44: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E44: limitation: threshold is provisional (not calibrated)
- E45: partial coverage (2 of 3 examined)
- E45: limitation: off-heap: no gap series for S10; run dla gap S10 first (node.external is used when present)
- E45: limitation: threshold is provisional (not calibrated)
- E46: partial coverage (2 of 3 examined)
- E46: limitation: off-heap: no gap series for S12; run dla gap S12 first (node.external is used when present)
- E46: limitation: O2 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E46: limitation: threshold is provisional (not calibrated)
- E47: partial coverage (2 of 3 examined)
- E47: limitation: off-heap: no gap series for S14; run dla gap S14 first (node.external is used when present)
- E47: limitation: threshold is provisional (not calibrated)
- E48: partial coverage (2 of 3 examined)
- E48: limitation: off-heap: no gap series for S16; run dla gap S16 first (node.external is used when present)
- E48: limitation: threshold is provisional (not calibrated)
- E49: partial coverage (2 of 3 examined)
- E49: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E49: limitation: O3 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O3 --as unresolved --reason time_budget --remedy "<text>"
- E49: limitation: threshold is provisional (not calibrated)
- E50: partial coverage (2 of 3 examined)
- E50: limitation: off-heap: no gap series for S13; run dla gap S13 first (node.external is used when present)
- E50: limitation: O3 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O3 --as unresolved --reason time_budget --remedy "<text>"
- E50: limitation: threshold is provisional (not calibrated)
- E51: partial coverage (2 of 3 examined)
- E51: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E51: limitation: threshold is provisional (not calibrated)
- E52: partial coverage (2 of 3 examined)
- E52: limitation: off-heap: no gap series for S16; run dla gap S16 first (node.external is used when present)
- E52: limitation: threshold is provisional (not calibrated)
- E53: partial coverage (2 of 3 examined)
- E53: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E53: limitation: threshold is provisional (not calibrated)
- E54: partial coverage (2 of 3 examined)
- E54: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E54: limitation: threshold is provisional (not calibrated)
- E55: partial coverage (2 of 3 examined)
- E55: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E55: limitation: threshold is provisional (not calibrated)
- E56: partial coverage (2 of 3 examined)
- E56: limitation: off-heap: no gap series for S11; run dla gap S11 first (node.external is used when present)
- E56: limitation: threshold is provisional (not calibrated)
- E57: partial coverage (2 of 3 examined)
- E57: limitation: off-heap: no gap series for S13; run dla gap S13 first (node.external is used when present)
- E57: limitation: threshold is provisional (not calibrated)
- E58: partial coverage (2 of 3 examined)
- E58: limitation: off-heap: no gap series for S15; run dla gap S15 first (node.external is used when present)
- E58: limitation: O6 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O6 --as unresolved --reason time_budget --remedy "<text>"
- E58: limitation: threshold is provisional (not calibrated)
- E59: partial coverage (2 of 3 examined)
- E59: limitation: off-heap: no gap series for S17; run dla gap S17 first (node.external is used when present)
- E59: limitation: threshold is provisional (not calibrated)
- E60: partial coverage (2 of 3 examined)
- E60: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E60: limitation: threshold is provisional (not calibrated)
- E61: partial coverage (2 of 3 examined)
- E61: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E61: limitation: threshold is provisional (not calibrated)
- E62: partial coverage (2 of 3 examined)
- E62: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E62: limitation: threshold is provisional (not calibrated)
- E63: partial coverage (2 of 3 examined)
- E63: limitation: off-heap: no gap series for S10; run dla gap S10 first (node.external is used when present)
- E63: limitation: threshold is provisional (not calibrated)
- E64: partial coverage (2 of 3 examined)
- E64: limitation: off-heap: no gap series for S12; run dla gap S12 first (node.external is used when present)
- E64: limitation: O8 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O8 --as unresolved --reason time_budget --remedy "<text>"
- E64: limitation: threshold is provisional (not calibrated)
- E65: partial coverage (2 of 3 examined)
- E65: limitation: off-heap: no gap series for S14; run dla gap S14 first (node.external is used when present)
- E65: limitation: threshold is provisional (not calibrated)
- E66: partial coverage (2 of 3 examined)
- E66: limitation: off-heap: no gap series for S16; run dla gap S16 first (node.external is used when present)
- E66: limitation: threshold is provisional (not calibrated)
- E67: partial coverage (2 of 3 examined)
- E67: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E67: limitation: O9 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O9 --as unresolved --reason time_budget --remedy "<text>"
- E67: limitation: threshold is provisional (not calibrated)
- E68: partial coverage (2 of 3 examined)
- E68: limitation: off-heap: no gap series for S11; run dla gap S11 first (node.external is used when present)
- E68: limitation: O9 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O9 --as unresolved --reason time_budget --remedy "<text>"
- E68: limitation: threshold is provisional (not calibrated)
- E69: partial coverage (2 of 3 examined)
- E69: limitation: off-heap: no gap series for S13; run dla gap S13 first (node.external is used when present)
- E69: limitation: O9 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O9 --as unresolved --reason time_budget --remedy "<text>"
- E69: limitation: threshold is provisional (not calibrated)
- E70: partial coverage (2 of 3 examined)
- E70: limitation: off-heap: no gap series for S15; run dla gap S15 first (node.external is used when present)
- E70: limitation: O8 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O8 --as unresolved --reason time_budget --remedy "<text>"
- E70: limitation: O9 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O9 --as unresolved --reason time_budget --remedy "<text>"
- E70: limitation: threshold is provisional (not calibrated)

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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (821fbd8f5fcd) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (658f9888bc93) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (658f9888bc93) | trend/T1.json |
| E12 | histogram-diff | live | jcmd-class-histogram 0.1.0 | artifact histograms/H1.json (44ff4a3493e0) | histograms/H1.json |
| E13 | growth-series | live | jcmd-class-histogram 0.1.0 | artifact series/S2.json (547753f1348d) | series/S2.json |
| E14 | residual | test | dla 0.1.0 | artifact series/S1.json (658f9888bc93) | gap/residual-S1-E14.json |
| E15 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (547753f1348d) | trend/T2.json |
| E16 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (547753f1348d) | trend/T2.json |
| E17 | non-heap | test | dla 0.1.0 | artifact series/S3.json (11ed9ef290a3) | trend/T3.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S5.json (a3354d885e41) | alloc/A1.json |
| E19 | discriminator | test | dla 0.1.0 | artifact series/S7.json (e37c0422a605) | alloc/A2.json |
| E20 | growth-series | test | dla 0.1.0 | artifact series/S1.json (658f9888bc93) | trend/T6.json |
| E21 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (658f9888bc93) | trend/T6.json |
| E22 | discriminator | test | dla 0.1.0 | artifact series/S9.json (7ae5e7d8262d) | alloc/A3.json |
| E23 | discriminator | test | dla 0.1.0 | artifact series/S11.json (a5c70ee955bd) | alloc/A4.json |
| E24 | discriminator | test | dla 0.1.0 | artifact series/S13.json (862c041790fa) | alloc/A5.json |
| E25 | discriminator | test | dla 0.1.0 | artifact series/S15.json (d79800f8fbec) | alloc/A6.json |
| E26 | discriminator | test | dla 0.1.0 | artifact series/S17.json (f79378fc9a1d) | alloc/A7.json |
| E27 | non-heap | live | jvm-classloader-stats 0.1.0 | artifact nonheap/N1.json (aedf3bd11db0) | nonheap/N1.json |
| E28 | growth-series | test | dla 0.1.0 | artifact series/S4.json (b19c00887474) | trend/T12.json |
| E29 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (b19c00887474) | trend/T12.json |
| E30 | growth-series | test | dla 0.1.0 | artifact series/S6.json (19f0db7875a3) | trend/T13.json |
| E31 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (19f0db7875a3) | trend/T13.json |
| E32 | growth-series | test | dla 0.1.0 | artifact series/S8.json (db17ec8f38c3) | trend/T14.json |
| E33 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (db17ec8f38c3) | trend/T14.json |
| E34 | growth-series | test | dla 0.1.0 | artifact series/S10.json (b718bacd1e72) | trend/T15.json |
| E35 | unit-count-series | test | dla 0.1.0 | artifact series/S10.json (b718bacd1e72) | trend/T15.json |
| E36 | growth-series | test | dla 0.1.0 | artifact series/S12.json (42f218ab5faa) | trend/T16.json |
| E37 | unit-count-series | test | dla 0.1.0 | artifact series/S12.json (42f218ab5faa) | trend/T16.json |
| E38 | growth-series | test | dla 0.1.0 | artifact series/S14.json (99c77823a5e5) | trend/T17.json |
| E39 | unit-count-series | test | dla 0.1.0 | artifact series/S14.json (99c77823a5e5) | trend/T17.json |
| E40 | growth-series | test | dla 0.1.0 | artifact series/S16.json (2fd61af4bf30) | trend/T18.json |
| E41 | unit-count-series | test | dla 0.1.0 | artifact series/S16.json (2fd61af4bf30) | trend/T18.json |
| E42 | residual | test | dla 0.1.0 | artifact series/S5.json (a3354d885e41) | gap/residual-S5-E42.json |
| E43 | residual | test | dla 0.1.0 | artifact series/S4.json (b19c00887474) | gap/residual-S4-E43.json |
| E44 | residual | test | dla 0.1.0 | artifact series/S8.json (db17ec8f38c3) | gap/residual-S8-E44.json |
| E45 | residual | test | dla 0.1.0 | artifact series/S10.json (b718bacd1e72) | gap/residual-S10-E45.json |
| E46 | residual | test | dla 0.1.0 | artifact series/S12.json (42f218ab5faa) | gap/residual-S12-E46.json |
| E47 | residual | test | dla 0.1.0 | artifact series/S14.json (99c77823a5e5) | gap/residual-S14-E47.json |
| E48 | residual | test | dla 0.1.0 | artifact series/S16.json (2fd61af4bf30) | gap/residual-S16-E48.json |
| E49 | residual | test | dla 0.1.0 | artifact series/S7.json (e37c0422a605) | gap/residual-S7-E49.json |
| E50 | residual | test | dla 0.1.0 | artifact series/S13.json (862c041790fa) | gap/residual-S13-E50.json |
| E51 | residual | test | dla 0.1.0 | artifact series/S5.json (a3354d885e41) | gap/residual-S5-E51.json |
| E52 | residual | test | dla 0.1.0 | artifact series/S16.json (2fd61af4bf30) | gap/residual-S16-E52.json |
| E53 | residual | test | dla 0.1.0 | artifact series/S6.json (19f0db7875a3) | gap/residual-S6-E53.json |
| E54 | residual | test | dla 0.1.0 | artifact series/S7.json (e37c0422a605) | gap/residual-S7-E54.json |
| E55 | residual | test | dla 0.1.0 | artifact series/S9.json (7ae5e7d8262d) | gap/residual-S9-E55.json |
| E56 | residual | test | dla 0.1.0 | artifact series/S11.json (a5c70ee955bd) | gap/residual-S11-E56.json |
| E57 | residual | test | dla 0.1.0 | artifact series/S13.json (862c041790fa) | gap/residual-S13-E57.json |
| E58 | residual | test | dla 0.1.0 | artifact series/S15.json (d79800f8fbec) | gap/residual-S15-E58.json |
| E59 | residual | test | dla 0.1.0 | artifact series/S17.json (f79378fc9a1d) | gap/residual-S17-E59.json |
| E60 | residual | test | dla 0.1.0 | artifact series/S5.json (a3354d885e41) | gap/residual-S5-E60.json |
| E61 | residual | test | dla 0.1.0 | artifact series/S4.json (b19c00887474) | gap/residual-S4-E61.json |
| E62 | residual | test | dla 0.1.0 | artifact series/S8.json (db17ec8f38c3) | gap/residual-S8-E62.json |
| E63 | residual | test | dla 0.1.0 | artifact series/S10.json (b718bacd1e72) | gap/residual-S10-E63.json |
| E64 | residual | test | dla 0.1.0 | artifact series/S12.json (42f218ab5faa) | gap/residual-S12-E64.json |
| E65 | residual | test | dla 0.1.0 | artifact series/S14.json (99c77823a5e5) | gap/residual-S14-E65.json |
| E66 | residual | test | dla 0.1.0 | artifact series/S16.json (2fd61af4bf30) | gap/residual-S16-E66.json |
| E67 | residual | test | dla 0.1.0 | artifact series/S9.json (7ae5e7d8262d) | gap/residual-S9-E67.json |
| E68 | residual | test | dla 0.1.0 | artifact series/S11.json (a5c70ee955bd) | gap/residual-S11-E68.json |
| E69 | residual | test | dla 0.1.0 | artifact series/S13.json (862c041790fa) | gap/residual-S13-E69.json |
| E70 | residual | test | dla 0.1.0 | artifact series/S15.json (d79800f8fbec) | gap/residual-S15-E70.json |
