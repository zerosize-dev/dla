# DLA report

Run: `20261110-180505-7183`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 13 enumerated, 13 dispositioned; confirmed 0, strong 0, refuted 7, suspected 0, unresolved 6

Residual: heap 0 bytes/n unattributed of -0.06661518661518662 total (threshold 182.90652203861003); heap 0.03783783783783784 bytes/n unattributed of 0.03783783783783784 total (threshold 0.3554805405405405); heap 0 bytes/n unattributed of -0.003153153153153153 total (threshold 0.30887499999999996); heap 0.012012012012012012 bytes/n unattributed of 0.012012012012012012 total (threshold 0.8136172972973861); heap 0 bytes/n unattributed of -0.026126126126126126 total (threshold 0.4807096756756756); heap 1.36 bytes/n unattributed of 1.36 total (threshold 1.730577729729836); heap 0.4425880425880426 bytes/n unattributed of 0.4425880425880426 total (threshold 1.1768516953317063); heap 1.0954954954954954 bytes/n unattributed of 1.0954954954954954 total (threshold 1.3738378378378375); heap 0.013384813384813383 bytes/n unattributed of 0.013384813384813383 total (threshold 0.31949362162162165); heap 0.05525525525525526 bytes/n unattributed of 0.05525525525525526 total (threshold 0.5938032432433321); heap 0 bytes/n unattributed of -0.003153153153153153 total (threshold 0.32543737837837833); heap 0.36996996996997 bytes/n unattributed of 0.36996996996997 total (threshold 0.44039135135130686); heap 0.017889317889317892 bytes/n unattributed of 0.017889317889317892 total (threshold 0.3093997297296764); heap 1.1315315315315315 bytes/n unattributed of 1.1315315315315315 total (threshold 2.184402162162162); heap 0.019939939939939946 bytes/n unattributed of 0.019939939939939946 total (threshold 0.34528106666669567); non-heap 14.931274131274131 bytes/n unattributed of 14.931274131274131 total (threshold 1088.5977006949806); non-heap 0 bytes/n unattributed of -4.1402402402402405 total (threshold 11.695748648648692); non-heap 0 bytes/n unattributed of -2.5 total (threshold 12.823154324324324); non-heap 0 bytes/n unattributed of -7.36036036036036 total (threshold 38.68326648648649); non-heap 2.4856713856713855 bytes/n unattributed of 2.4856713856713855 total (threshold 27.931306270270216); non-heap 15.832432432432432 bytes/n unattributed of 15.832432432432432 total (threshold 72.60389513513512); non-heap 9.625989625989623 bytes/n unattributed of 9.625989625989623 total (threshold 45.63270376412775); non-heap 9.1003003003003 bytes/n unattributed of 9.1003003003003 total (threshold 15.526657297297296); non-heap 4.3147147147147145 bytes/n unattributed of 4.3147147147147145 total (threshold 9.903946090090049); non-heap 0 bytes/n unattributed of -3.1375375375375376 total (threshold 18.20945729729721); non-heap 0.6550836550836551 bytes/n unattributed of 0.6550836550836551 total (threshold 16.68594745945944); non-heap 12.252852852852854 bytes/n unattributed of 12.252852852852854 total (threshold 18.22014270270266); non-heap 4.642664092664093 bytes/n unattributed of 4.642664092664093 total (threshold 9.738697567567549); non-heap 6.345945945945946 bytes/n unattributed of 6.345945945945946 total (threshold 24.031858378378377); non-heap 5.07027027027027 bytes/n unattributed of 5.07027027027027 total (threshold 18.509259243243243)

Coverage: jvm 8880 iterations, 15 of 15 entry points exercised

## Findings

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: JDK-internal class-count step in the harness class histogram, not a project holder; plateaued under alloc-test on sibling columns (F5-F7). Rerun with a longer window or a heap-dump series to rule out.
- Supporting evidence: E42

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: JDK-internal class-count step in the harness class histogram, not a project holder; plateaued under alloc-test on sibling columns (F5-F7). Rerun with a longer window or a heap-dump series to rule out.
- Supporting evidence: E48

### F10: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: JDK-internal class-count step in the harness class histogram, not a project holder; plateaued under alloc-test on sibling columns (F5-F7). Rerun with a longer window or a heap-dump series to rule out.
- Supporting evidence: E54

### F11: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: JDK-internal class-count step in the harness class histogram, not a project holder; plateaued under alloc-test on sibling columns (F5-F7). Rerun with a longer window or a heap-dump series to rule out.
- Supporting evidence: E66

### F12: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: JDK-internal class-count step in the harness class histogram, not a project holder; plateaued under alloc-test on sibling columns (F5-F7). Rerun with a longer window or a heap-dump series to rule out.
- Supporting evidence: E75

### F13: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: JDK-internal class-count step in the harness class histogram, not a project holder; plateaued under alloc-test on sibling columns (F5-F7). Rerun with a longer window or a heap-dump series to rule out.
- Supporting evidence: E78

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (non-leak) E14
  - Contradicting evidence: E14
  - Originally supported by: E6
- **F2** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (non-leak) E15
  - Contradicting evidence: E15
  - Originally supported by: E6
- **F3** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (non-leak) E13
  - Contradicting evidence: E13
  - Originally supported by: E6
- **F4** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (non-leak) E12
  - Contradicting evidence: E12
  - Originally supported by: E6
- **F5** (UNCLASSIFIED): discriminator E33
  - Ruled out by: plateau+purge (non-leak) E33
  - Contradicting evidence: E33
  - Originally supported by: E17
- **F6** (UNCLASSIFIED): discriminator E34
  - Ruled out by: plateau+purge (non-leak) E34
  - Contradicting evidence: E34
  - Originally supported by: E19
- **F7** (UNCLASSIFIED): discriminator E35
  - Ruled out by: plateau+purge (non-leak) E35
  - Contradicting evidence: E35
  - Originally supported by: E21

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
- E16: limitation: gate thresholds are provisional (not calibrated)
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
- E24: partial coverage (2 of 3 examined)
- E24: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E24: limitation: threshold is provisional (not calibrated)
- E25: partial coverage (2 of 3 examined)
- E25: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E25: limitation: threshold is provisional (not calibrated)
- E26: partial coverage (2 of 3 examined)
- E26: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E26: limitation: threshold is provisional (not calibrated)
- E27: partial coverage (2 of 3 examined)
- E27: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E27: limitation: threshold is provisional (not calibrated)
- E28: partial coverage (2 of 3 examined)
- E28: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E28: limitation: threshold is provisional (not calibrated)
- E29: partial coverage (2 of 3 examined)
- E29: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E29: limitation: threshold is provisional (not calibrated)
- E30: partial coverage (2 of 3 examined)
- E30: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E30: limitation: threshold is provisional (not calibrated)
- E31: partial coverage (2 of 3 examined)
- E31: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E31: limitation: threshold is provisional (not calibrated)
- E32: partial coverage (2 of 3 examined)
- E32: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E32: limitation: threshold is provisional (not calibrated)
- E33: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E33: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E33: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E33: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E33: limitation: points 1 to 9 come from S1; the extra points from S10
- E33: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E33: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E33: limitation: gate and plateau thresholds are provisional (not calibrated)
- E34: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E34: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E34: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E34: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E34: limitation: points 1 to 9 come from S1; the extra points from S12
- E34: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E34: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E34: limitation: gate and plateau thresholds are provisional (not calibrated)
- E35: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E35: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E35: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E35: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E35: limitation: points 1 to 9 come from S1; the extra points from S14
- E35: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E35: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E35: limitation: gate and plateau thresholds are provisional (not calibrated)
- E36: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E36: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E36: limitation: gate thresholds are provisional (not calibrated)
- E37: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E37: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E37: limitation: gate thresholds are provisional (not calibrated)
- E38: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E38: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E38: limitation: gate thresholds are provisional (not calibrated)
- E39: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E39: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E39: limitation: gate thresholds are provisional (not calibrated)
- E40: partial coverage (2 of 3 examined)
- E40: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E40: limitation: threshold is provisional (not calibrated)
- E41: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E41: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E41: limitation: gate thresholds are provisional (not calibrated)
- E42: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E42: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E42: limitation: gate thresholds are provisional (not calibrated)
- E43: partial coverage (2 of 3 examined)
- E43: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E43: limitation: threshold is provisional (not calibrated)
- E44: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E44: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E44: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E44: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E44: limitation: points 1 to 9 come from S1; the extra points from S2
- E44: limitation: gate thresholds are provisional (not calibrated)
- E45: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E45: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E45: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E45: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E45: limitation: points 1 to 9 come from S1; the extra points from S2
- E45: limitation: gate thresholds are provisional (not calibrated)
- E46: partial coverage (2 of 3 examined)
- E46: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E46: limitation: threshold is provisional (not calibrated)
- E47: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E47: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E47: limitation: gate thresholds are provisional (not calibrated)
- E48: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E48: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E48: limitation: gate thresholds are provisional (not calibrated)
- E49: partial coverage (2 of 3 examined)
- E49: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E49: limitation: threshold is provisional (not calibrated)
- E50: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E50: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E50: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E50: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E50: limitation: points 1 to 9 come from S1; the extra points from S4
- E50: limitation: gate thresholds are provisional (not calibrated)
- E51: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E51: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E51: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E51: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E51: limitation: points 1 to 9 come from S1; the extra points from S4
- E51: limitation: gate thresholds are provisional (not calibrated)
- E52: partial coverage (2 of 3 examined)
- E52: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E52: limitation: threshold is provisional (not calibrated)
- E53: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E53: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E53: limitation: gate thresholds are provisional (not calibrated)
- E54: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E54: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E54: limitation: gate thresholds are provisional (not calibrated)
- E55: partial coverage (2 of 3 examined)
- E55: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E55: limitation: threshold is provisional (not calibrated)
- E56: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E56: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E56: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E56: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E56: limitation: points 1 to 9 come from S1; the extra points from S6
- E56: limitation: gate thresholds are provisional (not calibrated)
- E57: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E57: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E57: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E57: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E57: limitation: points 1 to 9 come from S1; the extra points from S6
- E57: limitation: gate thresholds are provisional (not calibrated)
- E58: partial coverage (2 of 3 examined)
- E58: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E58: limitation: threshold is provisional (not calibrated)
- E59: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E59: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E59: limitation: gate thresholds are provisional (not calibrated)
- E60: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E60: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E60: limitation: gate thresholds are provisional (not calibrated)
- E61: partial coverage (2 of 3 examined)
- E61: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E61: limitation: threshold is provisional (not calibrated)
- E62: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E62: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E62: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E62: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E62: limitation: points 1 to 9 come from S1; the extra points from S8
- E62: limitation: gate thresholds are provisional (not calibrated)
- E63: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E63: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E63: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E63: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E63: limitation: points 1 to 9 come from S1; the extra points from S8
- E63: limitation: gate thresholds are provisional (not calibrated)
- E64: partial coverage (2 of 3 examined)
- E64: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E64: limitation: threshold is provisional (not calibrated)
- E65: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E65: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E65: limitation: gate thresholds are provisional (not calibrated)
- E66: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E66: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E66: limitation: gate thresholds are provisional (not calibrated)
- E67: partial coverage (2 of 3 examined)
- E67: limitation: off-heap: no gap series for S10; run dla gap S10 first (node.external is used when present)
- E67: limitation: threshold is provisional (not calibrated)
- E68: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E68: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E68: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E68: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E68: limitation: points 1 to 9 come from S1; the extra points from S10
- E68: limitation: gate thresholds are provisional (not calibrated)
- E69: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E69: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E69: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E69: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E69: limitation: points 1 to 9 come from S1; the extra points from S10
- E69: limitation: gate thresholds are provisional (not calibrated)
- E70: partial coverage (2 of 3 examined)
- E70: limitation: off-heap: no gap series for S11; run dla gap S11 first (node.external is used when present)
- E70: limitation: threshold is provisional (not calibrated)
- E71: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E71: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E71: limitation: gate thresholds are provisional (not calibrated)
- E72: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E72: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E72: limitation: gate thresholds are provisional (not calibrated)
- E73: partial coverage (2 of 3 examined)
- E73: limitation: off-heap: no gap series for S12; run dla gap S12 first (node.external is used when present)
- E73: limitation: threshold is provisional (not calibrated)
- E74: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E74: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E74: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E74: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E74: limitation: points 1 to 9 come from S1; the extra points from S12
- E74: limitation: gate thresholds are provisional (not calibrated)
- E75: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E75: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E75: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E75: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E75: limitation: points 1 to 9 come from S1; the extra points from S12
- E75: limitation: gate thresholds are provisional (not calibrated)
- E76: partial coverage (2 of 3 examined)
- E76: limitation: off-heap: no gap series for S13; run dla gap S13 first (node.external is used when present)
- E76: limitation: threshold is provisional (not calibrated)
- E77: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E77: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E77: limitation: gate thresholds are provisional (not calibrated)
- E78: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E78: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E78: limitation: gate thresholds are provisional (not calibrated)
- E79: partial coverage (2 of 3 examined)
- E79: limitation: off-heap: no gap series for S14; run dla gap S14 first (node.external is used when present)
- E79: limitation: threshold is provisional (not calibrated)
- E80: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E80: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E80: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E80: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E80: limitation: points 1 to 9 come from S1; the extra points from S14
- E80: limitation: gate thresholds are provisional (not calibrated)
- E81: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E81: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E81: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E81: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E81: limitation: points 1 to 9 come from S1; the extra points from S14
- E81: limitation: gate thresholds are provisional (not calibrated)
- E82: partial coverage (2 of 3 examined)
- E82: limitation: off-heap: no gap series for S15; run dla gap S15 first (node.external is used when present)
- E82: limitation: threshold is provisional (not calibrated)

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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (47e76a5ab56c) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (edb6b920beb5) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (edb6b920beb5) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (e3741677d4d8) | alloc/A1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S5.json (b79fb95eb0de) | alloc/A2.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S7.json (c222fcedd4cd) | alloc/A3.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S9.json (ece2f280ebfe) | alloc/A4.json |
| E16 | growth-series | test | dla 0.1.0 | artifact series/S2.json (ce8847f2e6f9) | trend/T6.json |
| E17 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (ce8847f2e6f9) | trend/T6.json |
| E18 | growth-series | test | dla 0.1.0 | artifact series/S4.json (15f3454788ac) | trend/T7.json |
| E19 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (15f3454788ac) | trend/T7.json |
| E20 | growth-series | test | dla 0.1.0 | artifact series/S6.json (f19bc3b411a3) | trend/T8.json |
| E21 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (f19bc3b411a3) | trend/T8.json |
| E22 | growth-series | test | dla 0.1.0 | artifact series/S8.json (23db76a30c07) | trend/T9.json |
| E23 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (23db76a30c07) | trend/T9.json |
| E24 | residual | test | dla 0.1.0 | artifact series/S3.json (e3741677d4d8) | gap/residual-S3-E24.json |
| E25 | residual | test | dla 0.1.0 | artifact series/S1.json (edb6b920beb5) | gap/residual-S1-E25.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S2.json (ce8847f2e6f9) | gap/residual-S2-E26.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S4.json (15f3454788ac) | gap/residual-S4-E27.json |
| E28 | residual | test | dla 0.1.0 | artifact series/S5.json (b79fb95eb0de) | gap/residual-S5-E28.json |
| E29 | residual | test | dla 0.1.0 | artifact series/S6.json (f19bc3b411a3) | gap/residual-S6-E29.json |
| E30 | residual | test | dla 0.1.0 | artifact series/S7.json (c222fcedd4cd) | gap/residual-S7-E30.json |
| E31 | residual | test | dla 0.1.0 | artifact series/S8.json (23db76a30c07) | gap/residual-S8-E31.json |
| E32 | residual | test | dla 0.1.0 | artifact series/S9.json (ece2f280ebfe) | gap/residual-S9-E32.json |
| E33 | discriminator | test | dla 0.1.0 | artifact series/S11.json (eaf1df096f06) | alloc/A5.json |
| E34 | discriminator | test | dla 0.1.0 | artifact series/S13.json (2fdad9622e94) | alloc/A6.json |
| E35 | discriminator | test | dla 0.1.0 | artifact series/S15.json (a16428fafc78) | alloc/A7.json |
| E36 | growth-series | test | dla 0.1.0 | artifact series/S10.json (9e97cb582765) | trend/T13.json |
| E37 | unit-count-series | test | dla 0.1.0 | artifact series/S10.json (9e97cb582765) | trend/T13.json |
| E38 | growth-series | test | dla 0.1.0 | artifact series/S1.json (edb6b920beb5) | trend/T14.json |
| E39 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (edb6b920beb5) | trend/T14.json |
| E40 | residual | test | dla 0.1.0 | artifact series/S1.json (edb6b920beb5) | gap/residual-S1-E40.json |
| E41 | growth-series | test | dla 0.1.0 | artifact series/S2.json (ce8847f2e6f9) | trend/T15.json |
| E42 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (ce8847f2e6f9) | trend/T15.json |
| E43 | residual | test | dla 0.1.0 | artifact series/S2.json (ce8847f2e6f9) | gap/residual-S2-E43.json |
| E44 | growth-series | test | dla 0.1.0 | artifact series/S3.json (e3741677d4d8) | trend/T16.json |
| E45 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (e3741677d4d8) | trend/T16.json |
| E46 | residual | test | dla 0.1.0 | artifact series/S3.json (e3741677d4d8) | gap/residual-S3-E46.json |
| E47 | growth-series | test | dla 0.1.0 | artifact series/S4.json (15f3454788ac) | trend/T17.json |
| E48 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (15f3454788ac) | trend/T17.json |
| E49 | residual | test | dla 0.1.0 | artifact series/S4.json (15f3454788ac) | gap/residual-S4-E49.json |
| E50 | growth-series | test | dla 0.1.0 | artifact series/S5.json (b79fb95eb0de) | trend/T18.json |
| E51 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (b79fb95eb0de) | trend/T18.json |
| E52 | residual | test | dla 0.1.0 | artifact series/S5.json (b79fb95eb0de) | gap/residual-S5-E52.json |
| E53 | growth-series | test | dla 0.1.0 | artifact series/S6.json (f19bc3b411a3) | trend/T19.json |
| E54 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (f19bc3b411a3) | trend/T19.json |
| E55 | residual | test | dla 0.1.0 | artifact series/S6.json (f19bc3b411a3) | gap/residual-S6-E55.json |
| E56 | growth-series | test | dla 0.1.0 | artifact series/S7.json (c222fcedd4cd) | trend/T20.json |
| E57 | unit-count-series | test | dla 0.1.0 | artifact series/S7.json (c222fcedd4cd) | trend/T20.json |
| E58 | residual | test | dla 0.1.0 | artifact series/S7.json (c222fcedd4cd) | gap/residual-S7-E58.json |
| E59 | growth-series | test | dla 0.1.0 | artifact series/S8.json (23db76a30c07) | trend/T21.json |
| E60 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (23db76a30c07) | trend/T21.json |
| E61 | residual | test | dla 0.1.0 | artifact series/S8.json (23db76a30c07) | gap/residual-S8-E61.json |
| E62 | growth-series | test | dla 0.1.0 | artifact series/S9.json (ece2f280ebfe) | trend/T22.json |
| E63 | unit-count-series | test | dla 0.1.0 | artifact series/S9.json (ece2f280ebfe) | trend/T22.json |
| E64 | residual | test | dla 0.1.0 | artifact series/S9.json (ece2f280ebfe) | gap/residual-S9-E64.json |
| E65 | growth-series | test | dla 0.1.0 | artifact series/S10.json (9e97cb582765) | trend/T23.json |
| E66 | unit-count-series | test | dla 0.1.0 | artifact series/S10.json (9e97cb582765) | trend/T23.json |
| E67 | residual | test | dla 0.1.0 | artifact series/S10.json (9e97cb582765) | gap/residual-S10-E67.json |
| E68 | growth-series | test | dla 0.1.0 | artifact series/S11.json (eaf1df096f06) | trend/T24.json |
| E69 | unit-count-series | test | dla 0.1.0 | artifact series/S11.json (eaf1df096f06) | trend/T24.json |
| E70 | residual | test | dla 0.1.0 | artifact series/S11.json (eaf1df096f06) | gap/residual-S11-E70.json |
| E71 | growth-series | test | dla 0.1.0 | artifact series/S12.json (bbe9c19383e9) | trend/T25.json |
| E72 | unit-count-series | test | dla 0.1.0 | artifact series/S12.json (bbe9c19383e9) | trend/T25.json |
| E73 | residual | test | dla 0.1.0 | artifact series/S12.json (bbe9c19383e9) | gap/residual-S12-E73.json |
| E74 | growth-series | test | dla 0.1.0 | artifact series/S13.json (2fdad9622e94) | trend/T26.json |
| E75 | unit-count-series | test | dla 0.1.0 | artifact series/S13.json (2fdad9622e94) | trend/T26.json |
| E76 | residual | test | dla 0.1.0 | artifact series/S13.json (2fdad9622e94) | gap/residual-S13-E76.json |
| E77 | growth-series | test | dla 0.1.0 | artifact series/S14.json (20c3bed5e5a7) | trend/T27.json |
| E78 | unit-count-series | test | dla 0.1.0 | artifact series/S14.json (20c3bed5e5a7) | trend/T27.json |
| E79 | residual | test | dla 0.1.0 | artifact series/S14.json (20c3bed5e5a7) | gap/residual-S14-E79.json |
| E80 | growth-series | test | dla 0.1.0 | artifact series/S15.json (a16428fafc78) | trend/T28.json |
| E81 | unit-count-series | test | dla 0.1.0 | artifact series/S15.json (a16428fafc78) | trend/T28.json |
| E82 | residual | test | dla 0.1.0 | artifact series/S15.json (a16428fafc78) | gap/residual-S15-E82.json |
