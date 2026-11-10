# DLA report

Run: `20261110-191258-5ed6`

Status: INCOMPLETE (17 open)

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 14 enumerated, 14 dispositioned; confirmed 0, strong 0, refuted 7, suspected 0, unresolved 7

Residual: heap 0 bytes/n unattributed of -1.0846846846846847 total (threshold 1.4551232432432433); heap 0 bytes/n unattributed of -0.3572072072072072 total (threshold 0.6021225945945945); heap 0.8598198198198199 bytes/n unattributed of 0.8598198198198199 total (threshold 0.7682043243241911) ABOVE THRESHOLD, see O1; non-heap 44.95207207207207 bytes/n unattributed of 44.95207207207207 total (threshold 97.91983394594604); non-heap 0 bytes/n unattributed of -3.4905814905814907 total (threshold 25.11521971498774); non-heap 8.066666666666666 bytes/n unattributed of 8.066666666666666 total (threshold 27.21057567567568)

Coverage: jvm 8880 iterations, 14 of 14 entry points exercised

## Open items

- **T2:os.rss**: column os.rss of T2 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T2:os.rss_anon**: column os.rss_anon of T2 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T2:jvm.classes_loaded**: column jvm.classes_loaded of T2 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T2:jvm.nonheap_committed**: column jvm.nonheap_committed of T2 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T2:jvm.class:java.lang.Class.count**: column jvm.class:java.lang.Class.count of T2 (stair) is not flat and is tied to no finding. Fix: `dla trend S5`
- **T3:os.vm_hwm**: column os.vm_hwm of T3 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:os.vm_hwm**: column os.vm_hwm of T7 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S15 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:os.rss**: column os.rss of T8 (spike) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S17 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:os.rss_anon**: column os.rss_anon of T8 (spike) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S17 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:os.vm_hwm**: column os.vm_hwm of T8 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S17 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:jvm.class:java.lang.Class.count**: column jvm.class:java.lang.Class.count of T8 (stair) is not flat and is tied to no finding. Fix: `dla trend S17`
- **T11:os.rss_file**: column os.rss_file of T11 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T14:os.rss**: column os.rss of T14 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T14:os.rss_anon**: column os.rss_anon of T14 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T16:jvm.class:java.lang.Class.count**: column jvm.class:java.lang.Class.count of T16 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E42 # column jvm.class:java.lang.Class.count`
- **T18:jvm.class:byte[].count**: column jvm.class:byte[].count of T18 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E46 # column jvm.class:byte[].count`
- **T18:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T18 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E46 # column jvm.class:java.lang.String.count`

## Findings

### F1.n1: Hypothesis N-HEADROOM: bounded steady-state holder, plateau under extra cycles (UNCLASSIFIED)

- State: unresolved(not_exercised)
- Evidence classes: discriminator
- Sibling hypothesis of: F1
- Hypothesis: N-HEADROOM: bounded steady-state holder, plateau under extra cycles
- Refuted if: growth continues over the extra cycles
- Reason: not_exercised
- Remedy: Non-leak sibling: plateau under extra cycles (alloc-test) favours it; no further action unless production growth persists
- Supporting evidence: E21

### F2.n1: Hypothesis N-WARM: bounded steady-state holder, plateau under extra cycles (UNCLASSIFIED)

- State: unresolved(not_exercised)
- Evidence classes: discriminator
- Sibling hypothesis of: F2
- Hypothesis: N-WARM: bounded steady-state holder, plateau under extra cycles
- Refuted if: growth continues over the extra cycles
- Reason: not_exercised
- Remedy: Non-leak sibling: plateau under extra cycles (alloc-test) favours it; no further action unless production growth persists
- Supporting evidence: E22

### F3.n1: Hypothesis N-WARM: bounded steady-state holder, plateau under extra cycles (UNCLASSIFIED)

- State: unresolved(not_exercised)
- Evidence classes: discriminator
- Sibling hypothesis of: F3
- Hypothesis: N-WARM: bounded steady-state holder, plateau under extra cycles
- Refuted if: growth continues over the extra cycles
- Reason: not_exercised
- Remedy: Non-leak sibling: plateau under extra cycles (alloc-test) favours it; no further action unless production growth persists
- Supporting evidence: E23

### F4.n1: Hypothesis N-WARM: bounded steady-state holder, plateau under extra cycles (UNCLASSIFIED)

- State: unresolved(not_exercised)
- Evidence classes: discriminator
- Sibling hypothesis of: F4
- Hypothesis: N-WARM: bounded steady-state holder, plateau under extra cycles
- Refuted if: growth continues over the extra cycles
- Reason: not_exercised
- Remedy: Non-leak sibling: plateau under extra cycles (alloc-test) favours it; no further action unless production growth persists
- Supporting evidence: E24

### F5.n1: Hypothesis N-WARM: bounded steady-state holder, plateau under extra cycles (UNCLASSIFIED)

- State: unresolved(not_exercised)
- Evidence classes: discriminator
- Sibling hypothesis of: F5
- Hypothesis: N-WARM: bounded steady-state holder, plateau under extra cycles
- Refuted if: growth continues over the extra cycles
- Reason: not_exercised
- Remedy: Non-leak sibling: plateau under extra cycles (alloc-test) favours it; no further action unless production growth persists
- Supporting evidence: E25

### F6.n1: Hypothesis N-WARM: RenderContext list is truncated per render; bounded warm-up capacity only (UNCLASSIFIED)

- State: unresolved(not_exercised)
- Evidence classes: discriminator
- Sibling hypothesis of: F6
- Hypothesis: N-WARM: RenderContext list is truncated per render; bounded warm-up capacity only
- Refuted if: growth continues over the extra cycles
- Reason: not_exercised
- Remedy: Non-leak sibling: plateau under extra cycles (alloc-test) favours it; no further action unless production growth persists
- Supporting evidence: E19

### F7.n1: Hypothesis N-WARM: bounded steady-state holder, plateau under extra cycles (UNCLASSIFIED)

- State: unresolved(not_exercised)
- Evidence classes: discriminator
- Sibling hypothesis of: F7
- Hypothesis: N-WARM: bounded steady-state holder, plateau under extra cycles
- Refuted if: growth continues over the extra cycles
- Reason: not_exercised
- Remedy: Non-leak sibling: plateau under extra cycles (alloc-test) favours it; no further action unless production growth persists
- Supporting evidence: E26

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E21
  - Ruled out by: plateau+purge (non-leak) E21
  - Contradicting evidence: E21
  - Originally supported by: E6
- **F2** (UNCLASSIFIED): discriminator E22
  - Ruled out by: plateau+purge (non-leak) E22
  - Contradicting evidence: E22
  - Originally supported by: E6
- **F3** (UNCLASSIFIED): discriminator E23
  - Ruled out by: plateau+purge (non-leak) E23
  - Contradicting evidence: E23
  - Originally supported by: E6
- **F4** (UNCLASSIFIED): discriminator E24
  - Ruled out by: plateau+purge (non-leak) E24
  - Contradicting evidence: E24
  - Originally supported by: E6
- **F5** (UNCLASSIFIED): discriminator E25
  - Ruled out by: plateau+purge (non-leak) E25
  - Contradicting evidence: E25
  - Originally supported by: E6
- **F6** (UNCLASSIFIED): discriminator E19
  - Ruled out by: plateau+purge (non-leak) E19
  - Contradicting evidence: E19
  - Originally supported by: E6
- **F7** (UNCLASSIFIED): discriminator E26
  - Ruled out by: plateau+purge (non-leak) E26
  - Contradicting evidence: E26
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
- E12: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E12: limitation: the files carry no time, so they are ordered as given
- E13: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- E13: limitation: the files carry no time, so they are ordered as given
- E13: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E14: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E15: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E16: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E17: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E18: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E18: limitation: series: per-point; one process per point at n = 1110, 4440, 6660, 9990 (harness captures; checked: HPROF version, identifier size)
- E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E19: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E19: limitation: points 1 to 9 come from S1; the extra points from S4
- E19: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E19: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E19: limitation: gate and plateau thresholds are provisional (not calibrated)
- E20: partial coverage (2 of 3 examined)
- E20: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E20: limitation: threshold is provisional (not calibrated)
- E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: points 1 to 9 come from S1; the extra points from S6
- E21: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E21: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E21: limitation: gate and plateau thresholds are provisional (not calibrated)
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: points 1 to 9 come from S1; the extra points from S8
- E22: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E22: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E22: limitation: gate and plateau thresholds are provisional (not calibrated)
- E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E23: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E23: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E23: limitation: points 1 to 9 come from S1; the extra points from S10
- E23: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E23: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E23: limitation: gate and plateau thresholds are provisional (not calibrated)
- E24: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E24: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E24: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E24: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E24: limitation: points 1 to 9 come from S1; the extra points from S12
- E24: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E24: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E24: limitation: gate and plateau thresholds are provisional (not calibrated)
- E25: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E25: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E25: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E25: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E25: limitation: points 1 to 9 come from S1; the extra points from S14
- E25: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E25: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E25: limitation: gate and plateau thresholds are provisional (not calibrated)
- E26: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E26: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E26: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E26: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E26: limitation: points 1 to 9 come from S1; the extra points from S16
- E26: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E26: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E26: limitation: gate and plateau thresholds are provisional (not calibrated)
- E27: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E27: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E28: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: limitation: series: per-point; x is the iteration count n of each point (1110, 4440, 6660, 9990); every point is a separate process and heap dump
- E29: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E29: limitation: holders absent from a dump's 200 largest have no figure at that point
- E29: limitation: gate thresholds are provisional (not calibrated)
- E30: limitation: series: per-point; x is the iteration count n of each point (1110, 4440, 6660, 9990); every point is a separate process and heap dump
- E30: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E30: limitation: holders absent from a dump's 200 largest have no figure at that point
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
- E34: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E34: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E34: limitation: gate thresholds are provisional (not calibrated)
- E35: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E35: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E35: limitation: gate thresholds are provisional (not calibrated)
- E36: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E36: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E36: limitation: gate thresholds are provisional (not calibrated)
- E37: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E37: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E37: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E37: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E37: limitation: points 1 to 9 come from S1; the extra points from S8
- E37: limitation: gate thresholds are provisional (not calibrated)
- E38: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E38: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E38: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E38: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E38: limitation: points 1 to 9 come from S1; the extra points from S8
- E38: limitation: gate thresholds are provisional (not calibrated)
- E39: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E39: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E39: limitation: gate thresholds are provisional (not calibrated)
- E40: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E40: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E40: limitation: gate thresholds are provisional (not calibrated)
- E41: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E41: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E41: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E41: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E41: limitation: points 1 to 9 come from S1; the extra points from S10
- E41: limitation: gate thresholds are provisional (not calibrated)
- E42: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E42: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E42: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E42: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E42: limitation: points 1 to 9 come from S1; the extra points from S10
- E42: limitation: gate thresholds are provisional (not calibrated)
- E43: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E43: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E43: limitation: gate thresholds are provisional (not calibrated)
- E44: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E44: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E44: limitation: gate thresholds are provisional (not calibrated)
- E45: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E45: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E45: limitation: gate thresholds are provisional (not calibrated)
- E46: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E46: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E46: limitation: gate thresholds are provisional (not calibrated)
- E47: partial coverage (2 of 3 examined)
- E47: limitation: off-heap: no gap series for S11; run dla gap S11 first (node.external is used when present)
- E47: limitation: threshold is provisional (not calibrated)
- E48: partial coverage (2 of 3 examined)
- E48: limitation: off-heap: no gap series for S14; run dla gap S14 first (node.external is used when present)
- E48: limitation: threshold is provisional (not calibrated)
- E49: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E49: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E49: limitation: gate thresholds are provisional (not calibrated)
- E50: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E50: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E50: limitation: gate thresholds are provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo da12a68e42b7ad806ec1746a9d7bc1189ef42f58 | — |
| E2 | run-config | static | dla 0.1.0 | repo da12a68e42b7ad806ec1746a9d7bc1189ef42f58 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo da12a68e42b7ad806ec1746a9d7bc1189ef42f58 | probe/probe.json |
| E4 | static-hit | static | spotbugs unknown | repo da12a68e42b7ad806ec1746a9d7bc1189ef42f58 | sweep/jvm-spotbugs.json |
| E5 | static-hit | static | dla 0.1.0 | repo da12a68e42b7ad806ec1746a9d7bc1189ef42f58 | sweep/jvm-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo da12a68e42b7ad806ec1746a9d7bc1189ef42f58 | census/jvm.json |
| E7 | workload | static | dla 0.1.0 | repo da12a68e42b7ad806ec1746a9d7bc1189ef42f58 | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo da12a68e42b7ad806ec1746a9d7bc1189ef42f58 | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (bb4fb3b94c67) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (edee2060e883) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (edee2060e883) | trend/T1.json |
| E12 | histogram-diff | live | jcmd-class-histogram 0.1.0 | artifact histograms/H1.json (a275892b82b4) | histograms/H1.json |
| E13 | growth-series | live | jcmd-class-histogram 0.1.0 | artifact series/S2.json (2766fb9a8199) | series/S2.json |
| E14 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-191258-5ed6/harness/H1/out/p1-native/capture/136/heap-1.hprof (503f5e249b63) | summaries/heap-1-503f5e24.json |
| E15 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-191258-5ed6/harness/H1/out/p4-native/capture/455/heap-1.hprof (b68b9c9de51f) | summaries/heap-1-b68b9c9d.json |
| E16 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-191258-5ed6/harness/H1/out/p6-native/capture/668/heap-1.hprof (ac7dc040c77e) | summaries/heap-1-ac7dc040.json |
| E17 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-191258-5ed6/harness/H1/out/p9-native/capture/987/heap-1.hprof (58735ef568ee) | summaries/heap-1-58735ef5.json |
| E18 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S3.json (20f78f94a45f) | series/S3.json |
| E19 | discriminator | test | dla 0.1.0 | artifact series/S5.json (494f47eb925f) | alloc/A1.json |
| E20 | residual | test | dla 0.1.0 | artifact series/S1.json (edee2060e883) | gap/residual-S1-E20.json |
| E21 | discriminator | test | dla 0.1.0 | artifact series/S7.json (09e52f94ea0c) | alloc/A2.json |
| E22 | discriminator | test | dla 0.1.0 | artifact series/S9.json (4d944c3c515d) | alloc/A3.json |
| E23 | discriminator | test | dla 0.1.0 | artifact series/S11.json (eba06b3f58a0) | alloc/A4.json |
| E24 | discriminator | test | dla 0.1.0 | artifact series/S13.json (feadef0ad5c2) | alloc/A5.json |
| E25 | discriminator | test | dla 0.1.0 | artifact series/S15.json (0906f938af2d) | alloc/A6.json |
| E26 | discriminator | test | dla 0.1.0 | artifact series/S17.json (71512ab6551c) | alloc/A7.json |
| E27 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (2766fb9a8199) | trend/T9.json |
| E28 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (2766fb9a8199) | trend/T9.json |
| E29 | growth-series | dump | dla 0.1.0 | artifact series/S3.json (20f78f94a45f) | trend/T10.json |
| E30 | unit-count-series | dump | dla 0.1.0 | artifact series/S3.json (20f78f94a45f) | trend/T10.json |
| E31 | growth-series | test | dla 0.1.0 | artifact series/S4.json (680791a3a33b) | trend/T11.json |
| E32 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (680791a3a33b) | trend/T11.json |
| E33 | growth-series | test | dla 0.1.0 | artifact series/S6.json (3179011d6cab) | trend/T12.json |
| E34 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (3179011d6cab) | trend/T12.json |
| E35 | growth-series | test | dla 0.1.0 | artifact series/S8.json (a3f5010ca043) | trend/T13.json |
| E36 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (a3f5010ca043) | trend/T13.json |
| E37 | growth-series | test | dla 0.1.0 | artifact series/S9.json (4d944c3c515d) | trend/T14.json |
| E38 | unit-count-series | test | dla 0.1.0 | artifact series/S9.json (4d944c3c515d) | trend/T14.json |
| E39 | growth-series | test | dla 0.1.0 | artifact series/S10.json (d6e284a81142) | trend/T15.json |
| E40 | unit-count-series | test | dla 0.1.0 | artifact series/S10.json (d6e284a81142) | trend/T15.json |
| E41 | growth-series | test | dla 0.1.0 | artifact series/S11.json (eba06b3f58a0) | trend/T16.json |
| E42 | unit-count-series | test | dla 0.1.0 | artifact series/S11.json (eba06b3f58a0) | trend/T16.json |
| E43 | growth-series | test | dla 0.1.0 | artifact series/S12.json (ec67e1bbc58e) | trend/T17.json |
| E44 | unit-count-series | test | dla 0.1.0 | artifact series/S12.json (ec67e1bbc58e) | trend/T17.json |
| E45 | growth-series | test | dla 0.1.0 | artifact series/S14.json (abe77dd75fba) | trend/T18.json |
| E46 | unit-count-series | test | dla 0.1.0 | artifact series/S14.json (abe77dd75fba) | trend/T18.json |
| E47 | residual | test | dla 0.1.0 | artifact series/S11.json (eba06b3f58a0) | gap/residual-S11-E47.json |
| E48 | residual | test | dla 0.1.0 | artifact series/S14.json (abe77dd75fba) | gap/residual-S14-E48.json |
| E49 | growth-series | test | dla 0.1.0 | artifact series/S16.json (1af36a185612) | trend/T19.json |
| E50 | unit-count-series | test | dla 0.1.0 | artifact series/S16.json (1af36a185612) | trend/T19.json |
