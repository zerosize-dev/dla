# DLA report

Run: `20261110-175409-3b57`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 23 enumerated, 23 dispositioned; confirmed 0, strong 0, refuted 7, suspected 0, unresolved 16

Residual: heap 15.334545454545454 bytes/n unattributed of 15.334545454545454 total (threshold 23.912990181818117); heap 27.41969696969697 bytes/n unattributed of 27.41969696969697 total (threshold 15.548526818181818) ABOVE THRESHOLD, see O1; heap 16.84628099173554 bytes/n unattributed of 16.84628099173554 total (threshold 10.363741586776989) ABOVE THRESHOLD, see O1; heap 25.35757575757576 bytes/n unattributed of 25.35757575757576 total (threshold 7.160765454545566) ABOVE THRESHOLD, see O1; heap 17.4125 bytes/n unattributed of 17.4125 total (threshold 5.6429777727272725) ABOVE THRESHOLD, see O1; heap 0 bytes/n unattributed of -1.7230769230769232 total (threshold 18.866966265734145); heap 2.526641414141414 bytes/n unattributed of 2.526641414141414 total (threshold 17.415776477272704); heap 0 bytes/n unattributed of -4.797424242424242 total (threshold 14.973069427272767); heap 1.2754545454545454 bytes/n unattributed of 1.2754545454545454 total (threshold 14.98416353359682); heap 0.16075757575757577 bytes/n unattributed of 0.16075757575757577 total (threshold 18.86491689090889); heap 1.5084848484848483 bytes/n unattributed of 1.5084848484848483 total (threshold 13.570653865612536); heap 12.559090909090909 bytes/n unattributed of 12.559090909090909 total (threshold 12.069711818181815) ABOVE THRESHOLD, see O1; heap 10.516528925619834 bytes/n unattributed of 10.516528925619834 total (threshold 11.437830148760415); heap 18.92121212121212 bytes/n unattributed of 18.92121212121212 total (threshold 4.7303030303030305) ABOVE THRESHOLD, see O1; heap 13.826013986013988 bytes/n unattributed of 13.826013986013988 total (threshold 7.014750830769259) ABOVE THRESHOLD, see O1; heap 16.009090909090908 bytes/n unattributed of 16.009090909090908 total (threshold 4.002272727272727) ABOVE THRESHOLD, see O1; heap 13.76 bytes/n unattributed of 13.76 total (threshold 7.176862254545674) ABOVE THRESHOLD, see O1; heap 21.37878787878788 bytes/n unattributed of 21.37878787878788 total (threshold 5.34469696969697) ABOVE THRESHOLD, see O1; heap 16.464297520661155 bytes/n unattributed of 16.464297520661155 total (threshold 4.881405361983491) ABOVE THRESHOLD, see O1; non-heap 107.46666666666668 bytes/n unattributed of 107.46666666666668 total (threshold 153.91120909091043); non-heap 44.83636363636363 bytes/n unattributed of 44.83636363636363 total (threshold 30.926650909090903) ABOVE THRESHOLD, see O2; non-heap 39.68863636363636 bytes/n unattributed of 39.68863636363636 total (threshold 29.685021545454543) ABOVE THRESHOLD, see O2; non-heap 17.184545454545454 bytes/n unattributed of 17.184545454545454 total (threshold 68.07877772727272); non-heap 33.80725436179982 bytes/n unattributed of 33.80725436179982 total (threshold 48.67942155922903); non-heap 0.3081818181818182 bytes/n unattributed of 0.3081818181818182 total (threshold 38.00025103636351); non-heap 20.756998556998557 bytes/n unattributed of 20.756998556998557 total (threshold 39.49241217391288); non-heap 23.140454545454546 bytes/n unattributed of 23.140454545454546 total (threshold 62.52353329090871); non-heap 31.125757575757575 bytes/n unattributed of 31.125757575757575 total (threshold 49.23784920948616); non-heap 26.39517906336088 bytes/n unattributed of 26.39517906336088 total (threshold 35.769603776859014); non-heap 24.22417355371901 bytes/n unattributed of 24.22417355371901 total (threshold 55.262376729249006); non-heap 51.42515151515151 bytes/n unattributed of 51.42515151515151 total (threshold 57.661876090908905); non-heap 53.09848484848485 bytes/n unattributed of 53.09848484848485 total (threshold 36.50408300000021) ABOVE THRESHOLD, see O2; non-heap 49.86168831168831 bytes/n unattributed of 49.86168831168831 total (threshold 59.43548103896142); non-heap 49.09242424242424 bytes/n unattributed of 49.09242424242424 total (threshold 35.39774890909091) ABOVE THRESHOLD, see O2; non-heap 10.102597402597404 bytes/n unattributed of 10.102597402597404 total (threshold 184.22515285714323); non-heap 19.03666666666667 bytes/n unattributed of 19.03666666666667 total (threshold 90.92902610909157); non-heap 10.94060606060606 bytes/n unattributed of 10.94060606060606 total (threshold 162.54340690909171); non-heap 30.117027417027415 bytes/n unattributed of 30.117027417027415 total (threshold 83.2714377575759)

Coverage: jvm 7040 iterations, 19 of 19 entry points exercised

## Findings

### F8: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: Same JDK-internal class-count column already showed plateau in alloc-test E12/E15/E35/E36 (warm-up, favours non-leak); re-run dla alloc-test on this trend's own harness series to refute formally
- Supporting evidence: E43

### F9: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: Same JDK-internal class-count column already showed plateau in alloc-test E12/E15/E35/E36 (warm-up, favours non-leak); re-run dla alloc-test on this trend's own harness series to refute formally
- Supporting evidence: E43

### F10: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: Same JDK-internal class-count column already showed plateau in alloc-test E12/E15/E35/E36 (warm-up, favours non-leak); re-run dla alloc-test on this trend's own harness series to refute formally
- Supporting evidence: E46

### F11: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: Same JDK-internal class-count column already showed plateau in alloc-test E12/E15/E35/E36 (warm-up, favours non-leak); re-run dla alloc-test on this trend's own harness series to refute formally
- Supporting evidence: E46

### F12: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: Same JDK-internal class-count column already showed plateau in alloc-test E12/E15/E35/E36 (warm-up, favours non-leak); re-run dla alloc-test on this trend's own harness series to refute formally
- Supporting evidence: E58

### F13: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: Same JDK-internal class-count column already showed plateau in alloc-test E12/E15/E35/E36 (warm-up, favours non-leak); re-run dla alloc-test on this trend's own harness series to refute formally
- Supporting evidence: E46

### F14: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: Same JDK-internal class-count column already showed plateau in alloc-test E12/E15/E35/E36 (warm-up, favours non-leak); re-run dla alloc-test on this trend's own harness series to refute formally
- Supporting evidence: E85

### F15: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: Same JDK-internal class-count column already showed plateau in alloc-test E12/E15/E35/E36 (warm-up, favours non-leak); re-run dla alloc-test on this trend's own harness series to refute formally
- Supporting evidence: E88

### F16: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: Same JDK-internal class-count column already showed plateau in alloc-test E12/E15/E35/E36 (warm-up, favours non-leak); re-run dla alloc-test on this trend's own harness series to refute formally
- Supporting evidence: E88

### F17: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: Same JDK-internal class-count column already showed plateau in alloc-test E12/E15/E35/E36 (warm-up, favours non-leak); re-run dla alloc-test on this trend's own harness series to refute formally
- Supporting evidence: E91

### F18: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: Same JDK-internal class-count column already showed plateau in alloc-test E12/E15/E35/E36 (warm-up, favours non-leak); re-run dla alloc-test on this trend's own harness series to refute formally
- Supporting evidence: E85

### F19: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: Same JDK-internal class-count column already showed plateau in alloc-test E12/E15/E35/E36 (warm-up, favours non-leak); re-run dla alloc-test on this trend's own harness series to refute formally
- Supporting evidence: E94

### F20: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: Same JDK-internal class-count column already showed plateau in alloc-test E12/E15/E35/E36 (warm-up, favours non-leak); re-run dla alloc-test on this trend's own harness series to refute formally
- Supporting evidence: E94

### F21: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: Same JDK-internal class-count column already showed plateau in alloc-test E12/E15/E35/E36 (warm-up, favours non-leak); re-run dla alloc-test on this trend's own harness series to refute formally
- Supporting evidence: E94

### F22: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: Same JDK-internal class-count column already showed plateau in alloc-test E12/E15/E35/E36 (warm-up, favours non-leak); re-run dla alloc-test on this trend's own harness series to refute formally
- Supporting evidence: E97

### F23: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: Same JDK-internal class-count column already showed plateau in alloc-test E12/E15/E35/E36 (warm-up, favours non-leak); re-run dla alloc-test on this trend's own harness series to refute formally
- Supporting evidence: E94

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E19
  - Ruled out by: plateau+purge (non-leak) E19
  - Contradicting evidence: E19
  - Originally supported by: E6
- **F2** (UNCLASSIFIED): discriminator E22
  - Ruled out by: plateau+purge (non-leak) E22
  - Contradicting evidence: E22
  - Originally supported by: E6
- **F3** (UNCLASSIFIED): discriminator E18
  - Ruled out by: plateau+purge (non-leak) E18
  - Contradicting evidence: E18
  - Originally supported by: E6
- **F4** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (non-leak) E12
  - Contradicting evidence: E12
  - Originally supported by: E11
- **F5** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (non-leak) E15
  - Contradicting evidence: E15
  - Originally supported by: E14
- **F6** (UNCLASSIFIED): discriminator E35
  - Ruled out by: plateau+purge (non-leak) E35
  - Contradicting evidence: E35
  - Originally supported by: E24
- **F7** (UNCLASSIFIED): discriminator E36
  - Ruled out by: plateau+purge (non-leak) E36
  - Contradicting evidence: E36
  - Originally supported by: E34

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
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E17: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E18: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E18: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: points 1 to 9 come from S1; the extra points from S6
- E18: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E18: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E18: limitation: gate and plateau thresholds are provisional (not calibrated)
- E19: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: points 1 to 9 come from S1; the extra points from S8
- E19: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E19: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E19: limitation: gate and plateau thresholds are provisional (not calibrated)
- E20: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: points 1 to 9 come from S1; the extra points from S10
- E20: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E20: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E20: limitation: gate and plateau thresholds are provisional (not calibrated)
- E21: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: points 1 to 9 come from S1; the extra points from S12
- E21: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E21: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E21: limitation: gate and plateau thresholds are provisional (not calibrated)
- E22: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: points 1 to 9 come from S1; the extra points from S14
- E22: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E22: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E22: limitation: gate and plateau thresholds are provisional (not calibrated)
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
- E33: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E33: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E33: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E33: limitation: points 1 to 9 come from S1; the extra points from S14
- E33: limitation: gate thresholds are provisional (not calibrated)
- E34: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E34: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E34: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E34: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E34: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E34: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E34: limitation: points 1 to 9 come from S1; the extra points from S14
- E34: limitation: gate thresholds are provisional (not calibrated)
- E35: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E35: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E35: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E35: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E35: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E35: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E35: limitation: points 1 to 9 come from S1; the extra points from S16
- E35: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E35: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E35: limitation: gate and plateau thresholds are provisional (not calibrated)
- E36: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E36: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E36: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E36: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E36: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E36: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E36: limitation: points 1 to 9 come from S1; the extra points from S18
- E36: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E36: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E36: limitation: gate and plateau thresholds are provisional (not calibrated)
- E37: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E37: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E37: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E37: limitation: gate thresholds are provisional (not calibrated)
- E38: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E38: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E38: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E38: limitation: gate thresholds are provisional (not calibrated)
- E39: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E39: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E39: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E39: limitation: gate thresholds are provisional (not calibrated)
- E40: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E40: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E40: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E40: limitation: gate thresholds are provisional (not calibrated)
- E41: partial coverage (2 of 3 examined)
- E41: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E41: limitation: threshold is provisional (not calibrated)
- E42: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E42: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E42: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E42: limitation: gate thresholds are provisional (not calibrated)
- E43: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E43: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E43: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E43: limitation: gate thresholds are provisional (not calibrated)
- E44: partial coverage (2 of 3 examined)
- E44: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E44: limitation: threshold is provisional (not calibrated)
- E45: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E45: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E45: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E45: limitation: gate thresholds are provisional (not calibrated)
- E46: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E46: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E46: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E46: limitation: gate thresholds are provisional (not calibrated)
- E47: partial coverage (2 of 3 examined)
- E47: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E47: limitation: threshold is provisional (not calibrated)
- E48: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E48: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E48: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E48: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E48: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E48: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E48: limitation: points 1 to 9 come from S1; the extra points from S2
- E48: limitation: gate thresholds are provisional (not calibrated)
- E49: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E49: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E49: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E49: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E49: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E49: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E49: limitation: points 1 to 9 come from S1; the extra points from S2
- E49: limitation: gate thresholds are provisional (not calibrated)
- E50: partial coverage (2 of 3 examined)
- E50: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E50: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E50: limitation: threshold is provisional (not calibrated)
- E51: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E51: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E51: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E51: limitation: gate thresholds are provisional (not calibrated)
- E52: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E52: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E52: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E52: limitation: gate thresholds are provisional (not calibrated)
- E53: partial coverage (2 of 3 examined)
- E53: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E53: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E53: limitation: threshold is provisional (not calibrated)
- E54: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E54: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E54: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E54: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E54: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E54: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E54: limitation: points 1 to 9 come from S1; the extra points from S4
- E54: limitation: gate thresholds are provisional (not calibrated)
- E55: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E55: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E55: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E55: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E55: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E55: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E55: limitation: points 1 to 9 come from S1; the extra points from S4
- E55: limitation: gate thresholds are provisional (not calibrated)
- E56: partial coverage (2 of 3 examined)
- E56: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E56: limitation: threshold is provisional (not calibrated)
- E57: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E57: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E57: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E57: limitation: gate thresholds are provisional (not calibrated)
- E58: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E58: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E58: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E58: limitation: gate thresholds are provisional (not calibrated)
- E59: partial coverage (2 of 3 examined)
- E59: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E59: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E59: limitation: threshold is provisional (not calibrated)
- E60: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E60: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E60: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E60: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E60: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E60: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E60: limitation: points 1 to 9 come from S1; the extra points from S6
- E60: limitation: gate thresholds are provisional (not calibrated)
- E61: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E61: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E61: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E61: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E61: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E61: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E61: limitation: points 1 to 9 come from S1; the extra points from S6
- E61: limitation: gate thresholds are provisional (not calibrated)
- E62: partial coverage (2 of 3 examined)
- E62: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E62: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E62: limitation: threshold is provisional (not calibrated)
- E63: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E63: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E63: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E63: limitation: gate thresholds are provisional (not calibrated)
- E64: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E64: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E64: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E64: limitation: gate thresholds are provisional (not calibrated)
- E65: partial coverage (2 of 3 examined)
- E65: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E65: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E65: limitation: threshold is provisional (not calibrated)
- E66: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E66: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E66: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E66: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E66: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E66: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E66: limitation: points 1 to 9 come from S1; the extra points from S8
- E66: limitation: gate thresholds are provisional (not calibrated)
- E67: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E67: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E67: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E67: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E67: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E67: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E67: limitation: points 1 to 9 come from S1; the extra points from S8
- E67: limitation: gate thresholds are provisional (not calibrated)
- E68: partial coverage (2 of 3 examined)
- E68: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E68: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E68: limitation: threshold is provisional (not calibrated)
- E69: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E69: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E69: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E69: limitation: gate thresholds are provisional (not calibrated)
- E70: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E70: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E70: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E70: limitation: gate thresholds are provisional (not calibrated)
- E71: partial coverage (2 of 3 examined)
- E71: limitation: off-heap: no gap series for S10; run dla gap S10 first (node.external is used when present)
- E71: limitation: threshold is provisional (not calibrated)
- E72: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E72: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E72: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E72: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E72: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E72: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E72: limitation: points 1 to 9 come from S1; the extra points from S10
- E72: limitation: gate thresholds are provisional (not calibrated)
- E73: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E73: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E73: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E73: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E73: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E73: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E73: limitation: points 1 to 9 come from S1; the extra points from S10
- E73: limitation: gate thresholds are provisional (not calibrated)
- E74: partial coverage (2 of 3 examined)
- E74: limitation: off-heap: no gap series for S11; run dla gap S11 first (node.external is used when present)
- E74: limitation: threshold is provisional (not calibrated)
- E75: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E75: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E75: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E75: limitation: gate thresholds are provisional (not calibrated)
- E76: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E76: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E76: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E76: limitation: gate thresholds are provisional (not calibrated)
- E77: partial coverage (2 of 3 examined)
- E77: limitation: off-heap: no gap series for S12; run dla gap S12 first (node.external is used when present)
- E77: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E77: limitation: threshold is provisional (not calibrated)
- E78: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E78: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E78: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E78: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E78: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E78: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E78: limitation: points 1 to 9 come from S1; the extra points from S12
- E78: limitation: gate thresholds are provisional (not calibrated)
- E79: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E79: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E79: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E79: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E79: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E79: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E79: limitation: points 1 to 9 come from S1; the extra points from S12
- E79: limitation: gate thresholds are provisional (not calibrated)
- E80: partial coverage (2 of 3 examined)
- E80: limitation: off-heap: no gap series for S13; run dla gap S13 first (node.external is used when present)
- E80: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E80: limitation: threshold is provisional (not calibrated)
- E81: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E81: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E81: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E81: limitation: gate thresholds are provisional (not calibrated)
- E82: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E82: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E82: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E82: limitation: gate thresholds are provisional (not calibrated)
- E83: partial coverage (2 of 3 examined)
- E83: limitation: off-heap: no gap series for S14; run dla gap S14 first (node.external is used when present)
- E83: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E83: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E83: limitation: threshold is provisional (not calibrated)
- E84: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E84: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E84: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E84: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E84: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E84: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E84: limitation: points 1 to 9 come from S1; the extra points from S14
- E84: limitation: gate thresholds are provisional (not calibrated)
- E85: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E85: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E85: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E85: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E85: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E85: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E85: limitation: points 1 to 9 come from S1; the extra points from S14
- E85: limitation: gate thresholds are provisional (not calibrated)
- E86: partial coverage (2 of 3 examined)
- E86: limitation: off-heap: no gap series for S15; run dla gap S15 first (node.external is used when present)
- E86: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E86: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E86: limitation: threshold is provisional (not calibrated)
- E87: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E87: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E87: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E87: limitation: gate thresholds are provisional (not calibrated)
- E88: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E88: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E88: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E88: limitation: gate thresholds are provisional (not calibrated)
- E89: partial coverage (2 of 3 examined)
- E89: limitation: off-heap: no gap series for S16; run dla gap S16 first (node.external is used when present)
- E89: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E89: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E89: limitation: threshold is provisional (not calibrated)
- E90: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E90: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E90: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E90: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E90: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E90: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E90: limitation: points 1 to 9 come from S1; the extra points from S16
- E90: limitation: gate thresholds are provisional (not calibrated)
- E91: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E91: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E91: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E91: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E91: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E91: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E91: limitation: points 1 to 9 come from S1; the extra points from S16
- E91: limitation: gate thresholds are provisional (not calibrated)
- E92: partial coverage (2 of 3 examined)
- E92: limitation: off-heap: no gap series for S17; run dla gap S17 first (node.external is used when present)
- E92: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E92: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E92: limitation: threshold is provisional (not calibrated)
- E93: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E93: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E93: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E93: limitation: gate thresholds are provisional (not calibrated)
- E94: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E94: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E94: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E94: limitation: gate thresholds are provisional (not calibrated)
- E95: partial coverage (2 of 3 examined)
- E95: limitation: off-heap: no gap series for S18; run dla gap S18 first (node.external is used when present)
- E95: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E95: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E95: limitation: threshold is provisional (not calibrated)
- E96: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E96: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E96: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E96: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E96: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E96: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E96: limitation: points 1 to 9 come from S1; the extra points from S18
- E96: limitation: gate thresholds are provisional (not calibrated)
- E97: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E97: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E97: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E97: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E97: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E97: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E97: limitation: points 1 to 9 come from S1; the extra points from S18
- E97: limitation: gate thresholds are provisional (not calibrated)
- E98: partial coverage (2 of 3 examined)
- E98: limitation: off-heap: no gap series for S19; run dla gap S19 first (node.external is used when present)
- E98: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E98: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E98: limitation: threshold is provisional (not calibrated)

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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (9eee2200767a) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (2eb41075f69c) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (2eb41075f69c) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (f0e571ef1513) | alloc/A1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S2.json (43d37eaec2fd) | trend/T3.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (43d37eaec2fd) | trend/T3.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S5.json (509a43e9412c) | alloc/A2.json |
| E16 | growth-series | test | dla 0.1.0 | artifact series/S4.json (a9e7b6d02c6b) | trend/T5.json |
| E17 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (a9e7b6d02c6b) | trend/T5.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S7.json (92a0598c5b9a) | alloc/A3.json |
| E19 | discriminator | test | dla 0.1.0 | artifact series/S9.json (91f811bf601c) | alloc/A4.json |
| E20 | discriminator | test | dla 0.1.0 | artifact series/S11.json (459da6ca401b) | alloc/A5.json |
| E21 | discriminator | test | dla 0.1.0 | artifact series/S13.json (8e9e638e5b95) | alloc/A6.json |
| E22 | discriminator | test | dla 0.1.0 | artifact series/S15.json (7f349c3abe5c) | alloc/A7.json |
| E23 | growth-series | test | dla 0.1.0 | artifact series/S6.json (1c4e59eadca1) | trend/T11.json |
| E24 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (1c4e59eadca1) | trend/T11.json |
| E25 | growth-series | test | dla 0.1.0 | artifact series/S8.json (5d9e11105c5b) | trend/T12.json |
| E26 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (5d9e11105c5b) | trend/T12.json |
| E27 | growth-series | test | dla 0.1.0 | artifact series/S10.json (ecbe4d8af5bf) | trend/T13.json |
| E28 | unit-count-series | test | dla 0.1.0 | artifact series/S10.json (ecbe4d8af5bf) | trend/T13.json |
| E29 | growth-series | test | dla 0.1.0 | artifact series/S12.json (3c059f351552) | trend/T14.json |
| E30 | unit-count-series | test | dla 0.1.0 | artifact series/S12.json (3c059f351552) | trend/T14.json |
| E31 | growth-series | test | dla 0.1.0 | artifact series/S14.json (588df0b43284) | trend/T15.json |
| E32 | unit-count-series | test | dla 0.1.0 | artifact series/S14.json (588df0b43284) | trend/T15.json |
| E33 | growth-series | test | dla 0.1.0 | artifact series/S15.json (7f349c3abe5c) | trend/T16.json |
| E34 | unit-count-series | test | dla 0.1.0 | artifact series/S15.json (7f349c3abe5c) | trend/T16.json |
| E35 | discriminator | test | dla 0.1.0 | artifact series/S17.json (7255092a89ab) | alloc/A8.json |
| E36 | discriminator | test | dla 0.1.0 | artifact series/S19.json (f5f1992ced70) | alloc/A9.json |
| E37 | growth-series | test | dla 0.1.0 | artifact series/S16.json (a450fb60ec8a) | trend/T19.json |
| E38 | unit-count-series | test | dla 0.1.0 | artifact series/S16.json (a450fb60ec8a) | trend/T19.json |
| E39 | growth-series | test | dla 0.1.0 | artifact series/S18.json (947e93790408) | trend/T20.json |
| E40 | unit-count-series | test | dla 0.1.0 | artifact series/S18.json (947e93790408) | trend/T20.json |
| E41 | residual | test | dla 0.1.0 | artifact series/S1.json (2eb41075f69c) | gap/residual-S1-E41.json |
| E42 | growth-series | test | dla 0.1.0 | artifact series/S1.json (2eb41075f69c) | trend/T21.json |
| E43 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (2eb41075f69c) | trend/T21.json |
| E44 | residual | test | dla 0.1.0 | artifact series/S1.json (2eb41075f69c) | gap/residual-S1-E44.json |
| E45 | growth-series | test | dla 0.1.0 | artifact series/S2.json (43d37eaec2fd) | trend/T22.json |
| E46 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (43d37eaec2fd) | trend/T22.json |
| E47 | residual | test | dla 0.1.0 | artifact series/S2.json (43d37eaec2fd) | gap/residual-S2-E47.json |
| E48 | growth-series | test | dla 0.1.0 | artifact series/S3.json (f0e571ef1513) | trend/T23.json |
| E49 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (f0e571ef1513) | trend/T23.json |
| E50 | residual | test | dla 0.1.0 | artifact series/S3.json (f0e571ef1513) | gap/residual-S3-E50.json |
| E51 | growth-series | test | dla 0.1.0 | artifact series/S4.json (a9e7b6d02c6b) | trend/T24.json |
| E52 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (a9e7b6d02c6b) | trend/T24.json |
| E53 | residual | test | dla 0.1.0 | artifact series/S4.json (a9e7b6d02c6b) | gap/residual-S4-E53.json |
| E54 | growth-series | test | dla 0.1.0 | artifact series/S5.json (509a43e9412c) | trend/T25.json |
| E55 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (509a43e9412c) | trend/T25.json |
| E56 | residual | test | dla 0.1.0 | artifact series/S5.json (509a43e9412c) | gap/residual-S5-E56.json |
| E57 | growth-series | test | dla 0.1.0 | artifact series/S6.json (1c4e59eadca1) | trend/T26.json |
| E58 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (1c4e59eadca1) | trend/T26.json |
| E59 | residual | test | dla 0.1.0 | artifact series/S6.json (1c4e59eadca1) | gap/residual-S6-E59.json |
| E60 | growth-series | test | dla 0.1.0 | artifact series/S7.json (92a0598c5b9a) | trend/T27.json |
| E61 | unit-count-series | test | dla 0.1.0 | artifact series/S7.json (92a0598c5b9a) | trend/T27.json |
| E62 | residual | test | dla 0.1.0 | artifact series/S7.json (92a0598c5b9a) | gap/residual-S7-E62.json |
| E63 | growth-series | test | dla 0.1.0 | artifact series/S8.json (5d9e11105c5b) | trend/T28.json |
| E64 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (5d9e11105c5b) | trend/T28.json |
| E65 | residual | test | dla 0.1.0 | artifact series/S8.json (5d9e11105c5b) | gap/residual-S8-E65.json |
| E66 | growth-series | test | dla 0.1.0 | artifact series/S9.json (91f811bf601c) | trend/T29.json |
| E67 | unit-count-series | test | dla 0.1.0 | artifact series/S9.json (91f811bf601c) | trend/T29.json |
| E68 | residual | test | dla 0.1.0 | artifact series/S9.json (91f811bf601c) | gap/residual-S9-E68.json |
| E69 | growth-series | test | dla 0.1.0 | artifact series/S10.json (ecbe4d8af5bf) | trend/T30.json |
| E70 | unit-count-series | test | dla 0.1.0 | artifact series/S10.json (ecbe4d8af5bf) | trend/T30.json |
| E71 | residual | test | dla 0.1.0 | artifact series/S10.json (ecbe4d8af5bf) | gap/residual-S10-E71.json |
| E72 | growth-series | test | dla 0.1.0 | artifact series/S11.json (459da6ca401b) | trend/T31.json |
| E73 | unit-count-series | test | dla 0.1.0 | artifact series/S11.json (459da6ca401b) | trend/T31.json |
| E74 | residual | test | dla 0.1.0 | artifact series/S11.json (459da6ca401b) | gap/residual-S11-E74.json |
| E75 | growth-series | test | dla 0.1.0 | artifact series/S12.json (3c059f351552) | trend/T32.json |
| E76 | unit-count-series | test | dla 0.1.0 | artifact series/S12.json (3c059f351552) | trend/T32.json |
| E77 | residual | test | dla 0.1.0 | artifact series/S12.json (3c059f351552) | gap/residual-S12-E77.json |
| E78 | growth-series | test | dla 0.1.0 | artifact series/S13.json (8e9e638e5b95) | trend/T33.json |
| E79 | unit-count-series | test | dla 0.1.0 | artifact series/S13.json (8e9e638e5b95) | trend/T33.json |
| E80 | residual | test | dla 0.1.0 | artifact series/S13.json (8e9e638e5b95) | gap/residual-S13-E80.json |
| E81 | growth-series | test | dla 0.1.0 | artifact series/S14.json (588df0b43284) | trend/T34.json |
| E82 | unit-count-series | test | dla 0.1.0 | artifact series/S14.json (588df0b43284) | trend/T34.json |
| E83 | residual | test | dla 0.1.0 | artifact series/S14.json (588df0b43284) | gap/residual-S14-E83.json |
| E84 | growth-series | test | dla 0.1.0 | artifact series/S15.json (7f349c3abe5c) | trend/T35.json |
| E85 | unit-count-series | test | dla 0.1.0 | artifact series/S15.json (7f349c3abe5c) | trend/T35.json |
| E86 | residual | test | dla 0.1.0 | artifact series/S15.json (7f349c3abe5c) | gap/residual-S15-E86.json |
| E87 | growth-series | test | dla 0.1.0 | artifact series/S16.json (a450fb60ec8a) | trend/T36.json |
| E88 | unit-count-series | test | dla 0.1.0 | artifact series/S16.json (a450fb60ec8a) | trend/T36.json |
| E89 | residual | test | dla 0.1.0 | artifact series/S16.json (a450fb60ec8a) | gap/residual-S16-E89.json |
| E90 | growth-series | test | dla 0.1.0 | artifact series/S17.json (7255092a89ab) | trend/T37.json |
| E91 | unit-count-series | test | dla 0.1.0 | artifact series/S17.json (7255092a89ab) | trend/T37.json |
| E92 | residual | test | dla 0.1.0 | artifact series/S17.json (7255092a89ab) | gap/residual-S17-E92.json |
| E93 | growth-series | test | dla 0.1.0 | artifact series/S18.json (947e93790408) | trend/T38.json |
| E94 | unit-count-series | test | dla 0.1.0 | artifact series/S18.json (947e93790408) | trend/T38.json |
| E95 | residual | test | dla 0.1.0 | artifact series/S18.json (947e93790408) | gap/residual-S18-E95.json |
| E96 | growth-series | test | dla 0.1.0 | artifact series/S19.json (f5f1992ced70) | trend/T39.json |
| E97 | unit-count-series | test | dla 0.1.0 | artifact series/S19.json (f5f1992ced70) | trend/T39.json |
| E98 | residual | test | dla 0.1.0 | artifact series/S19.json (f5f1992ced70) | gap/residual-S19-E98.json |
