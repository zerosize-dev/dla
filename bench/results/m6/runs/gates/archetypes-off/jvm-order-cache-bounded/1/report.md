# DLA report

Run: `20261110-184101-057b`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 8 enumerated, 8 dispositioned; confirmed 0, strong 0, refuted 4, suspected 0, unresolved 4

Residual: heap 1.6324324324324322 bytes/n unattributed of 1.6324324324324322 total (threshold 4.9194843243243245); heap 0.008168168168168169 bytes/n unattributed of 0.008168168168168169 total (threshold 0.3198498018017811); heap 0.03423423423423423 bytes/n unattributed of 0.03423423423423423 total (threshold 0.3463216216216216); heap 0 bytes/n unattributed of -0.022222222222222223 total (threshold 0.22991430630632703); heap 0.010210210210210211 bytes/n unattributed of 0.010210210210210211 total (threshold 0.4398189189190077); heap 0 bytes/n unattributed of -0.05225225225225225 total (threshold 0.36009816216216217); heap 0.2096096096096096 bytes/n unattributed of 0.2096096096096096 total (threshold 0.5226308108109441); heap 0 bytes/n unattributed of -0.001081081081081081 total (threshold 0.20596118918925135); non-heap 10.21081081081081 bytes/n unattributed of 10.21081081081081 total (threshold 30.31201459459459); non-heap 0 bytes/n unattributed of -0.15927927927927932 total (threshold 25.54314105945947); non-heap 0 bytes/n unattributed of -6.794894894894895 total (threshold 12.694738648648602); non-heap 0 bytes/n unattributed of -9.73101673101673 total (threshold 11.932201405405396); non-heap 0 bytes/n unattributed of -0.25885885885885884 total (threshold 7.1681897297296855); non-heap 0 bytes/n unattributed of -3.6536936936936937 total (threshold 9.733122075675723); non-heap 21.57957957957958 bytes/n unattributed of 21.57957957957958 total (threshold 39.40663027027018); non-heap 0 bytes/n unattributed of -8.01996996996997 total (threshold 21.338353891891828)

Coverage: jvm 8880 iterations, 15 of 15 entry points exercised

## Findings

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: Growth is in JDK-internal class instance counts (ConcurrentHashMap$Node/WeakReferenceKey) of the harness JVM with flat jvm.heap_used after GC; not a project holder. Re-check with a heap-graph series only if heap_used ever rises.
- Supporting evidence: E21

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: Growth is in JDK-internal class instance counts (ConcurrentHashMap$Node/WeakReferenceKey) of the harness JVM with flat jvm.heap_used after GC; not a project holder. Re-check with a heap-graph series only if heap_used ever rises.
- Supporting evidence: E23

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: Growth is in JDK-internal class instance counts (ConcurrentHashMap$Node/WeakReferenceKey) of the harness JVM with flat jvm.heap_used after GC; not a project holder. Re-check with a heap-graph series only if heap_used ever rises.
- Supporting evidence: E29

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: JDK-internal WeakReferenceKey count in the harness JVM; heap_used after GC is flat, no project holder. Re-check with heap-graph series only if heap_used rises.
- Supporting evidence: E38

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (non-leak) E15
  - Contradicting evidence: E15
  - Originally supported by: E6
- **F2** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (non-leak) E16
  - Contradicting evidence: E16
  - Originally supported by: E6
- **F3** (UNCLASSIFIED): discriminator E17
  - Ruled out by: plateau+purge (non-leak) E17
  - Contradicting evidence: E17
  - Originally supported by: E6
- **F4** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (non-leak) E12
  - Contradicting evidence: E12
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
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: cgroup.*: /proc/71/cgroup unreadable (No such file or directory (os error 2))
- E15: limitation: os.fds: /proc/71/fd unreadable (No such file or directory (os error 2))
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: points 1 to 9 come from S1; the extra points from S4
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: points 1 to 9 come from S1; the extra points from S6
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: points 1 to 9 come from S1; the extra points from S8
- E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E17: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E17: limitation: gate and plateau thresholds are provisional (not calibrated)
- E18: partial coverage (2 of 3 examined)
- E18: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E18: limitation: threshold is provisional (not calibrated)
- E19: partial coverage (2 of 3 examined)
- E19: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E19: limitation: threshold is provisional (not calibrated)
- E20: limitation: cgroup.*: /proc/71/cgroup unreadable (No such file or directory (os error 2))
- E20: limitation: os.fds: /proc/71/fd unreadable (No such file or directory (os error 2))
- E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: cgroup.*: /proc/71/cgroup unreadable (No such file or directory (os error 2))
- E21: limitation: os.fds: /proc/71/fd unreadable (No such file or directory (os error 2))
- E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: cgroup.*: /proc/71/cgroup unreadable (No such file or directory (os error 2))
- E22: limitation: os.fds: /proc/71/fd unreadable (No such file or directory (os error 2))
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: points 1 to 9 come from S1; the extra points from S4
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E23: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E23: limitation: cgroup.*: /proc/71/cgroup unreadable (No such file or directory (os error 2))
- E23: limitation: os.fds: /proc/71/fd unreadable (No such file or directory (os error 2))
- E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E23: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E23: limitation: points 1 to 9 come from S1; the extra points from S4
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E24: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E25: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E26: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E26: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E26: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E26: limitation: points 1 to 9 come from S1; the extra points from S6
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E27: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E27: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E27: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E27: limitation: points 1 to 9 come from S1; the extra points from S6
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E28: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E29: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E29: limitation: gate thresholds are provisional (not calibrated)
- E30: partial coverage (2 of 3 examined)
- E30: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E30: limitation: threshold is provisional (not calibrated)
- E31: partial coverage (2 of 3 examined)
- E31: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E31: limitation: threshold is provisional (not calibrated)
- E32: partial coverage (2 of 3 examined)
- E32: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E32: limitation: threshold is provisional (not calibrated)
- E33: partial coverage (2 of 3 examined)
- E33: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E33: limitation: threshold is provisional (not calibrated)
- E34: partial coverage (2 of 3 examined)
- E34: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E34: limitation: threshold is provisional (not calibrated)
- E35: partial coverage (2 of 3 examined)
- E35: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E35: limitation: threshold is provisional (not calibrated)
- E36: partial coverage (2 of 3 examined)
- E36: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E36: limitation: threshold is provisional (not calibrated)
- E37: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E37: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E37: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E37: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E37: limitation: points 1 to 9 come from S1; the extra points from S2
- E37: limitation: gate thresholds are provisional (not calibrated)
- E38: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E38: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E38: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E38: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E38: limitation: points 1 to 9 come from S1; the extra points from S2
- E38: limitation: gate thresholds are provisional (not calibrated)
- E39: partial coverage (2 of 3 examined)
- E39: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E39: limitation: threshold is provisional (not calibrated)

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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (6b9c451ca6b9) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (b287e2ff5a73) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (b287e2ff5a73) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (43cf767981b7) | alloc/A1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S2.json (d8f4686d4b69) | trend/T3.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (d8f4686d4b69) | trend/T3.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S5.json (aa41e167f75c) | alloc/A2.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S7.json (da96f512fb35) | alloc/A3.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S9.json (9a35454b26b4) | alloc/A4.json |
| E18 | residual | test | dla 0.1.0 | artifact series/S3.json (43cf767981b7) | gap/residual-S3-E18.json |
| E19 | residual | test | dla 0.1.0 | artifact series/S2.json (d8f4686d4b69) | gap/residual-S2-E19.json |
| E20 | growth-series | test | dla 0.1.0 | artifact series/S4.json (5f073551d208) | trend/T7.json |
| E21 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (5f073551d208) | trend/T7.json |
| E22 | growth-series | test | dla 0.1.0 | artifact series/S5.json (aa41e167f75c) | trend/T8.json |
| E23 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (aa41e167f75c) | trend/T8.json |
| E24 | growth-series | test | dla 0.1.0 | artifact series/S6.json (24c04a04dc57) | trend/T9.json |
| E25 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (24c04a04dc57) | trend/T9.json |
| E26 | growth-series | test | dla 0.1.0 | artifact series/S7.json (da96f512fb35) | trend/T10.json |
| E27 | unit-count-series | test | dla 0.1.0 | artifact series/S7.json (da96f512fb35) | trend/T10.json |
| E28 | growth-series | test | dla 0.1.0 | artifact series/S8.json (63d7d14a8ff6) | trend/T11.json |
| E29 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (63d7d14a8ff6) | trend/T11.json |
| E30 | residual | test | dla 0.1.0 | artifact series/S3.json (43cf767981b7) | gap/residual-S3-E30.json |
| E31 | residual | test | dla 0.1.0 | artifact series/S5.json (aa41e167f75c) | gap/residual-S5-E31.json |
| E32 | residual | test | dla 0.1.0 | artifact series/S7.json (da96f512fb35) | gap/residual-S7-E32.json |
| E33 | residual | test | dla 0.1.0 | artifact series/S9.json (9a35454b26b4) | gap/residual-S9-E33.json |
| E34 | residual | test | dla 0.1.0 | artifact series/S4.json (5f073551d208) | gap/residual-S4-E34.json |
| E35 | residual | test | dla 0.1.0 | artifact series/S6.json (24c04a04dc57) | gap/residual-S6-E35.json |
| E36 | residual | test | dla 0.1.0 | artifact series/S8.json (63d7d14a8ff6) | gap/residual-S8-E36.json |
| E37 | growth-series | test | dla 0.1.0 | artifact series/S3.json (43cf767981b7) | trend/T12.json |
| E38 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (43cf767981b7) | trend/T12.json |
| E39 | residual | test | dla 0.1.0 | artifact series/S3.json (43cf767981b7) | gap/residual-S3-E39.json |
