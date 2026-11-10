# DLA report

Run: `20261110-171048-ac78`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 12 enumerated, 12 dispositioned; confirmed 0, strong 0, refuted 4, suspected 0, unresolved 8

Residual: heap 15.477272727272728 bytes/n unattributed of 15.477272727272728 total (threshold 27.753983181818175); heap 21.595454545454544 bytes/n unattributed of 21.595454545454544 total (threshold 5.398863636363636) ABOVE THRESHOLD, see O5; heap 15.336363636363636 bytes/n unattributed of 15.336363636363636 total (threshold 6.276789272727272) ABOVE THRESHOLD, see O5; heap 15.192121212121211 bytes/n unattributed of 15.192121212121211 total (threshold 4.987504909090999) ABOVE THRESHOLD, see O5; heap 14.345454545454546 bytes/n unattributed of 14.345454545454546 total (threshold 8.121952363636364) ABOVE THRESHOLD, see O5; heap 27.29805194805195 bytes/n unattributed of 27.29805194805195 total (threshold 9.456113571428714) ABOVE THRESHOLD, see O5; heap 18.36753246753247 bytes/n unattributed of 18.36753246753247 total (threshold 11.096779636363614) ABOVE THRESHOLD, see O5; heap 21.29469696969697 bytes/n unattributed of 21.29469696969697 total (threshold 7.277014772727385) ABOVE THRESHOLD, see O5; heap 15.129545454545454 bytes/n unattributed of 15.129545454545454 total (threshold 6.7128084545454545) ABOVE THRESHOLD, see O5; heap 17.25151515151515 bytes/n unattributed of 17.25151515151515 total (threshold 4.3128787878787875) ABOVE THRESHOLD, see O5; heap 15.360714285714286 bytes/n unattributed of 15.360714285714286 total (threshold 4.857007227272593) ABOVE THRESHOLD, see O5; heap 19.88138528138528 bytes/n unattributed of 19.88138528138528 total (threshold 18.18096701298714) ABOVE THRESHOLD, see O5; heap 13.470862470862471 bytes/n unattributed of 13.470862470862471 total (threshold 11.193664559440538) ABOVE THRESHOLD, see O5; non-heap 151.68636363636364 bytes/n unattributed of 151.68636363636364 total (threshold 71.31209727272727) ABOVE THRESHOLD, see O4; non-heap 32.43878787878788 bytes/n unattributed of 32.43878787878788 total (threshold 67.7763850909099); non-heap 61.066161616161615 bytes/n unattributed of 61.066161616161615 total (threshold 63.02614966666652); non-heap 87.98939393939393 bytes/n unattributed of 87.98939393939393 total (threshold 58.93479409090909) ABOVE THRESHOLD, see O4; non-heap 65.89863636363637 bytes/n unattributed of 65.89863636363637 total (threshold 64.00606589999987) ABOVE THRESHOLD, see O4; non-heap 32.717727272727274 bytes/n unattributed of 32.717727272727274 total (threshold 37.194583090909624); non-heap 49.99545454545455 bytes/n unattributed of 49.99545454545455 total (threshold 43.18139890909091) ABOVE THRESHOLD, see O4; non-heap 54.195454545454545 bytes/n unattributed of 54.195454545454545 total (threshold 92.41459772727274); non-heap 61.825 bytes/n unattributed of 61.825 total (threshold 75.12064636363635); non-heap 0 bytes/n unattributed of -14.274545454545454 total (threshold 30.00012218181737); non-heap 30.80378787878788 bytes/n unattributed of 30.80378787878788 total (threshold 54.857323181818394); non-heap 35.86363636363636 bytes/n unattributed of 35.86363636363636 total (threshold 85.57971545454544); non-heap 50.07818181818182 bytes/n unattributed of 50.07818181818182 total (threshold 45.85573974545417) ABOVE THRESHOLD, see O4

Coverage: jvm 3520 iterations, 19 of 19 entry points exercised

## Findings

### F5: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: JDK-internal String/byte[] counts rose with warm-up; no project holder tied, os.rss verdict was noisy between runs. Re-run with a longer series or a JFR old-object capture to attribute.
- Supporting evidence: E22

### F6: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: JDK-internal warm-up type counts (Method/Class/CHM nodes/int[]) with no project holder tied; plateau expected. Re-run a longer series with a JFR old-object capture to attribute if it persists
- Supporting evidence: E30

### F7: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: JDK-internal warm-up type counts (Method/Class/CHM nodes/int[]) with no project holder tied; plateau expected. Re-run a longer series with a JFR old-object capture to attribute if it persists
- Supporting evidence: E42

### F8: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: JDK-internal warm-up type counts (Method/Class/CHM nodes/int[]) with no project holder tied; plateau expected. Re-run a longer series with a JFR old-object capture to attribute if it persists
- Supporting evidence: E43

### F9: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: warm-up/process-level growth without a tied project holder; plateau under alloc-test. Re-run a longer series with JFR old-object capture if it persists
- Supporting evidence: E47

### F10: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: warm-up/process-level growth without a tied project holder; plateau under alloc-test. Re-run a longer series with JFR old-object capture if it persists
- Supporting evidence: E47

### F11: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: warm-up/process-level growth without a tied project holder; plateau under alloc-test. Re-run a longer series with JFR old-object capture if it persists
- Supporting evidence: E64

### F12: **UNCLASSIFIED**

- State: unresolved(time_budget)
- Evidence classes: growth
- Reason: time_budget
- Remedy: warm-up/process-level growth without a tied project holder; plateau under alloc-test. Re-run a longer series with JFR old-object capture if it persists
- Supporting evidence: E64

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
- **F4** (UNCLASSIFIED): discriminator E44
  - Ruled out by: plateau+purge (non-leak) E44
  - Contradicting evidence: E44
  - Originally supported by: E18

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
- E9: limitation: cgroup.*: /proc/21/cgroup unreadable (No such file or directory (os error 2))
- E9: limitation: os.fds: /proc/21/fd unreadable (No such file or directory (os error 2))
- E9: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E9: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E9: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E10: limitation: cgroup.*: /proc/21/cgroup unreadable (No such file or directory (os error 2))
- E10: limitation: os.fds: /proc/21/fd unreadable (No such file or directory (os error 2))
- E10: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E10: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E10: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: cgroup.*: /proc/21/cgroup unreadable (No such file or directory (os error 2))
- E11: limitation: os.fds: /proc/21/fd unreadable (No such file or directory (os error 2))
- E11: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E11: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: partial coverage (2 of 3 examined)
- E12: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E12: limitation: threshold is provisional (not calibrated)
- E13: limitation: cgroup.*: /proc/21/cgroup unreadable (No such file or directory (os error 2))
- E13: limitation: os.fds: /proc/21/fd unreadable (No such file or directory (os error 2))
- E13: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: points 1 to 9 come from S1; the extra points from S2
- E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E13: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- E14: limitation: cgroup.*: /proc/21/cgroup unreadable (No such file or directory (os error 2))
- E14: limitation: os.fds: /proc/21/fd unreadable (No such file or directory (os error 2))
- E14: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: points 1 to 9 come from S1; the extra points from S4
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: limitation: cgroup.*: /proc/21/cgroup unreadable (No such file or directory (os error 2))
- E15: limitation: os.fds: /proc/21/fd unreadable (No such file or directory (os error 2))
- E15: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: points 1 to 9 come from S1; the extra points from S6
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: cgroup.*: /proc/21/cgroup unreadable (No such file or directory (os error 2))
- E16: limitation: os.fds: /proc/21/fd unreadable (No such file or directory (os error 2))
- E16: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: points 1 to 9 come from S1; the extra points from S8
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E17: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E18: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E18: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: cgroup.*: /proc/21/cgroup unreadable (No such file or directory (os error 2))
- E19: limitation: os.fds: /proc/21/fd unreadable (No such file or directory (os error 2))
- E19: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: points 1 to 9 come from S1; the extra points from S2
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: cgroup.*: /proc/21/cgroup unreadable (No such file or directory (os error 2))
- E20: limitation: os.fds: /proc/21/fd unreadable (No such file or directory (os error 2))
- E20: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: points 1 to 9 come from S1; the extra points from S2
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: cgroup.*: /proc/21/cgroup unreadable (No such file or directory (os error 2))
- E23: limitation: os.fds: /proc/21/fd unreadable (No such file or directory (os error 2))
- E23: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E23: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E23: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E23: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E23: limitation: points 1 to 9 come from S1; the extra points from S4
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: cgroup.*: /proc/21/cgroup unreadable (No such file or directory (os error 2))
- E24: limitation: os.fds: /proc/21/fd unreadable (No such file or directory (os error 2))
- E24: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E24: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E24: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E24: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E24: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E24: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E24: limitation: points 1 to 9 come from S1; the extra points from S4
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E25: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E25: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E26: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E26: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: limitation: cgroup.*: /proc/21/cgroup unreadable (No such file or directory (os error 2))
- E27: limitation: os.fds: /proc/21/fd unreadable (No such file or directory (os error 2))
- E27: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E27: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E27: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E27: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E27: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E27: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E27: limitation: points 1 to 9 come from S1; the extra points from S6
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: cgroup.*: /proc/21/cgroup unreadable (No such file or directory (os error 2))
- E28: limitation: os.fds: /proc/21/fd unreadable (No such file or directory (os error 2))
- E28: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E28: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E28: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E28: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E28: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E28: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E28: limitation: points 1 to 9 come from S1; the extra points from S6
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E29: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E29: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E29: limitation: gate thresholds are provisional (not calibrated)
- E30: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E30: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E30: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E30: limitation: gate thresholds are provisional (not calibrated)
- E31: limitation: cgroup.*: /proc/21/cgroup unreadable (No such file or directory (os error 2))
- E31: limitation: os.fds: /proc/21/fd unreadable (No such file or directory (os error 2))
- E31: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E31: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E31: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E31: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E31: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E31: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E31: limitation: points 1 to 9 come from S1; the extra points from S8
- E31: limitation: gate thresholds are provisional (not calibrated)
- E32: limitation: cgroup.*: /proc/21/cgroup unreadable (No such file or directory (os error 2))
- E32: limitation: os.fds: /proc/21/fd unreadable (No such file or directory (os error 2))
- E32: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E32: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E32: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E32: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E32: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E32: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E32: limitation: points 1 to 9 come from S1; the extra points from S8
- E32: limitation: gate thresholds are provisional (not calibrated)
- E33: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E33: limitation: the files carry no time, so they are ordered as given
- E34: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E34: limitation: the files carry no time, so they are ordered as given
- E34: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E35: partial coverage (2 of 3 examined)
- E35: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E35: limitation: threshold is provisional (not calibrated)
- E36: partial coverage (2 of 3 examined)
- E36: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E36: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E36: limitation: threshold is provisional (not calibrated)
- E37: partial coverage (2 of 3 examined)
- E37: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E37: limitation: threshold is provisional (not calibrated)
- E38: partial coverage (2 of 3 examined)
- E38: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E38: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E38: limitation: threshold is provisional (not calibrated)
- E39: partial coverage (2 of 3 examined)
- E39: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E39: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E39: limitation: threshold is provisional (not calibrated)
- E40: partial coverage (2 of 3 examined)
- E40: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E40: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E40: limitation: threshold is provisional (not calibrated)
- E41: partial coverage (2 of 3 examined)
- E41: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E41: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E41: limitation: threshold is provisional (not calibrated)
- E42: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E42: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E42: limitation: gate thresholds are provisional (not calibrated)
- E43: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E43: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E43: limitation: gate thresholds are provisional (not calibrated)
- E44: limitation: cgroup.*: /proc/21/cgroup unreadable (No such file or directory (os error 2))
- E44: limitation: os.fds: /proc/21/fd unreadable (No such file or directory (os error 2))
- E44: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E44: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E44: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E44: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E44: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E44: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E44: limitation: points 1 to 9 come from S1; the extra points from S11
- E44: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E44: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E44: limitation: gate and plateau thresholds are provisional (not calibrated)
- E45: limitation: cgroup.*: /proc/21/cgroup unreadable (No such file or directory (os error 2))
- E45: limitation: os.fds: /proc/21/fd unreadable (No such file or directory (os error 2))
- E45: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E45: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E45: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E45: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E45: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E45: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E45: limitation: points 1 to 9 come from S1; the extra points from S13
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
- E48: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E48: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E48: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E48: limitation: gate thresholds are provisional (not calibrated)
- E49: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E49: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E49: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E49: limitation: gate thresholds are provisional (not calibrated)
- E50: partial coverage (2 of 3 examined)
- E50: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E50: limitation: threshold is provisional (not calibrated)
- E51: partial coverage (2 of 3 examined)
- E51: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E51: limitation: O4 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O4 --as unresolved --reason time_budget --remedy "<text>"
- E51: limitation: threshold is provisional (not calibrated)
- E52: partial coverage (2 of 3 examined)
- E52: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E52: limitation: threshold is provisional (not calibrated)
- E53: partial coverage (2 of 3 examined)
- E53: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E53: limitation: O4 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O4 --as unresolved --reason time_budget --remedy "<text>"
- E53: limitation: threshold is provisional (not calibrated)
- E54: partial coverage (2 of 3 examined)
- E54: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E54: limitation: O4 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O4 --as unresolved --reason time_budget --remedy "<text>"
- E54: limitation: threshold is provisional (not calibrated)
- E55: partial coverage (2 of 3 examined)
- E55: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E55: limitation: O4 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O4 --as unresolved --reason time_budget --remedy "<text>"
- E55: limitation: threshold is provisional (not calibrated)
- E56: partial coverage (2 of 3 examined)
- E56: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E56: limitation: O4 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O4 --as unresolved --reason time_budget --remedy "<text>"
- E56: limitation: threshold is provisional (not calibrated)
- E57: partial coverage (2 of 3 examined)
- E57: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E57: limitation: O4 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O4 --as unresolved --reason time_budget --remedy "<text>"
- E57: limitation: threshold is provisional (not calibrated)
- E58: partial coverage (2 of 3 examined)
- E58: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E58: limitation: threshold is provisional (not calibrated)
- E59: partial coverage (2 of 3 examined)
- E59: limitation: off-heap: no gap series for S11; run dla gap S11 first (node.external is used when present)
- E59: limitation: O4 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O4 --as unresolved --reason time_budget --remedy "<text>"
- E59: limitation: threshold is provisional (not calibrated)
- E60: partial coverage (2 of 3 examined)
- E60: limitation: off-heap: no gap series for S12; run dla gap S12 first (node.external is used when present)
- E60: limitation: O4 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O4 --as unresolved --reason time_budget --remedy "<text>"
- E60: limitation: threshold is provisional (not calibrated)
- E61: partial coverage (2 of 3 examined)
- E61: limitation: off-heap: no gap series for S13; run dla gap S13 first (node.external is used when present)
- E61: limitation: threshold is provisional (not calibrated)
- E62: partial coverage (2 of 3 examined)
- E62: limitation: off-heap: no gap series for S14; run dla gap S14 first (node.external is used when present)
- E62: limitation: threshold is provisional (not calibrated)
- E63: limitation: cgroup.*: /proc/21/cgroup unreadable (No such file or directory (os error 2))
- E63: limitation: os.fds: /proc/21/fd unreadable (No such file or directory (os error 2))
- E63: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E63: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E63: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E63: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E63: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E63: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E63: limitation: points 1 to 9 come from S1; the extra points from S13
- E63: limitation: gate thresholds are provisional (not calibrated)
- E64: limitation: cgroup.*: /proc/21/cgroup unreadable (No such file or directory (os error 2))
- E64: limitation: os.fds: /proc/21/fd unreadable (No such file or directory (os error 2))
- E64: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E64: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E64: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E64: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E64: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E64: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E64: limitation: points 1 to 9 come from S1; the extra points from S13
- E64: limitation: gate thresholds are provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | — |
| E2 | run-config | static | dla 0.1.0 | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | probe/probe.json |
| E4 | static-hit | static | spotbugs unknown | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | sweep/jvm-spotbugs.json |
| E5 | static-hit | static | dla 0.1.0 | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | sweep/jvm-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | census/jvm.json |
| E7 | workload | static | dla 0.1.0 | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo 23c8a8332e0b6c7c0713cd698b13722c2eb021cc | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (4b4fd18c17b9) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (84b368f8116e) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (84b368f8116e) | trend/T1.json |
| E12 | residual | test | dla 0.1.0 | artifact series/S1.json (84b368f8116e) | gap/residual-S1-E12.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S3.json (f2c4f62395a5) | alloc/A1.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S5.json (dbf1e21c1041) | alloc/A2.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S7.json (903b32923804) | alloc/A3.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S9.json (0e6135a18e9b) | alloc/A4.json |
| E17 | growth-series | test | dla 0.1.0 | artifact series/S2.json (0e8126fad23d) | trend/T6.json |
| E18 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (0e8126fad23d) | trend/T6.json |
| E19 | growth-series | test | dla 0.1.0 | artifact series/S3.json (f2c4f62395a5) | trend/T7.json |
| E20 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (f2c4f62395a5) | trend/T7.json |
| E21 | growth-series | test | dla 0.1.0 | artifact series/S4.json (6e8d744ded2f) | trend/T8.json |
| E22 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (6e8d744ded2f) | trend/T8.json |
| E23 | growth-series | test | dla 0.1.0 | artifact series/S5.json (dbf1e21c1041) | trend/T9.json |
| E24 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (dbf1e21c1041) | trend/T9.json |
| E25 | growth-series | test | dla 0.1.0 | artifact series/S6.json (59128b5f7841) | trend/T10.json |
| E26 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (59128b5f7841) | trend/T10.json |
| E27 | growth-series | test | dla 0.1.0 | artifact series/S7.json (903b32923804) | trend/T11.json |
| E28 | unit-count-series | test | dla 0.1.0 | artifact series/S7.json (903b32923804) | trend/T11.json |
| E29 | growth-series | test | dla 0.1.0 | artifact series/S8.json (7a2a93843f40) | trend/T12.json |
| E30 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (7a2a93843f40) | trend/T12.json |
| E31 | growth-series | test | dla 0.1.0 | artifact series/S9.json (0e6135a18e9b) | trend/T13.json |
| E32 | unit-count-series | test | dla 0.1.0 | artifact series/S9.json (0e6135a18e9b) | trend/T13.json |
| E33 | histogram-diff | live | jcmd-class-histogram 0.1.0 | artifact histograms/H1.json (ad4e9fc6f971) | histograms/H1.json |
| E34 | growth-series | live | jcmd-class-histogram 0.1.0 | artifact series/S10.json (57310ff594fe) | series/S10.json |
| E35 | residual | test | dla 0.1.0 | artifact series/S1.json (84b368f8116e) | gap/residual-S1-E35.json |
| E36 | residual | test | dla 0.1.0 | artifact series/S2.json (0e8126fad23d) | gap/residual-S2-E36.json |
| E37 | residual | test | dla 0.1.0 | artifact series/S3.json (f2c4f62395a5) | gap/residual-S3-E37.json |
| E38 | residual | test | dla 0.1.0 | artifact series/S4.json (6e8d744ded2f) | gap/residual-S4-E38.json |
| E39 | residual | test | dla 0.1.0 | artifact series/S5.json (dbf1e21c1041) | gap/residual-S5-E39.json |
| E40 | residual | test | dla 0.1.0 | artifact series/S6.json (59128b5f7841) | gap/residual-S6-E40.json |
| E41 | residual | test | dla 0.1.0 | artifact series/S7.json (903b32923804) | gap/residual-S7-E41.json |
| E42 | growth-series | dump | dla 0.1.0 | artifact series/S10.json (57310ff594fe) | trend/T14.json |
| E43 | unit-count-series | dump | dla 0.1.0 | artifact series/S10.json (57310ff594fe) | trend/T14.json |
| E44 | discriminator | test | dla 0.1.0 | artifact series/S12.json (c6a47f763cae) | alloc/A5.json |
| E45 | discriminator | test | dla 0.1.0 | artifact series/S14.json (2ce6079ea744) | alloc/A6.json |
| E46 | growth-series | test | dla 0.1.0 | artifact series/S11.json (a4abf9407eb3) | trend/T17.json |
| E47 | unit-count-series | test | dla 0.1.0 | artifact series/S11.json (a4abf9407eb3) | trend/T17.json |
| E48 | growth-series | test | dla 0.1.0 | artifact series/S13.json (8363e90eab55) | trend/T18.json |
| E49 | unit-count-series | test | dla 0.1.0 | artifact series/S13.json (8363e90eab55) | trend/T18.json |
| E50 | residual | test | dla 0.1.0 | artifact series/S1.json (84b368f8116e) | gap/residual-S1-E50.json |
| E51 | residual | test | dla 0.1.0 | artifact series/S2.json (0e8126fad23d) | gap/residual-S2-E51.json |
| E52 | residual | test | dla 0.1.0 | artifact series/S3.json (f2c4f62395a5) | gap/residual-S3-E52.json |
| E53 | residual | test | dla 0.1.0 | artifact series/S4.json (6e8d744ded2f) | gap/residual-S4-E53.json |
| E54 | residual | test | dla 0.1.0 | artifact series/S5.json (dbf1e21c1041) | gap/residual-S5-E54.json |
| E55 | residual | test | dla 0.1.0 | artifact series/S6.json (59128b5f7841) | gap/residual-S6-E55.json |
| E56 | residual | test | dla 0.1.0 | artifact series/S7.json (903b32923804) | gap/residual-S7-E56.json |
| E57 | residual | test | dla 0.1.0 | artifact series/S8.json (7a2a93843f40) | gap/residual-S8-E57.json |
| E58 | residual | test | dla 0.1.0 | artifact series/S9.json (0e6135a18e9b) | gap/residual-S9-E58.json |
| E59 | residual | test | dla 0.1.0 | artifact series/S11.json (a4abf9407eb3) | gap/residual-S11-E59.json |
| E60 | residual | test | dla 0.1.0 | artifact series/S12.json (c6a47f763cae) | gap/residual-S12-E60.json |
| E61 | residual | test | dla 0.1.0 | artifact series/S13.json (8363e90eab55) | gap/residual-S13-E61.json |
| E62 | residual | test | dla 0.1.0 | artifact series/S14.json (2ce6079ea744) | gap/residual-S14-E62.json |
| E63 | growth-series | test | dla 0.1.0 | artifact series/S14.json (2ce6079ea744) | trend/T19.json |
| E64 | unit-count-series | test | dla 0.1.0 | artifact series/S14.json (2ce6079ea744) | trend/T19.json |
