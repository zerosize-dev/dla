# DLA report

Run: `20261110-163518-a562`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 12 enumerated, 12 dispositioned; confirmed 0, strong 0, refuted 1, suspected 0, unresolved 11

Residual: heap 12.828787878787878 bytes/n unattributed of 12.828787878787878 total (threshold 9.24507) ABOVE THRESHOLD, see O3; heap 15.91404958677686 bytes/n unattributed of 15.91404958677686 total (threshold 3.978512396694214) ABOVE THRESHOLD, see O3; heap 14.781363636363636 bytes/n unattributed of 14.781363636363636 total (threshold 4.4825063181818185) ABOVE THRESHOLD, see O3; heap 0 bytes/n unattributed of -2.56969696969697 total (threshold 16.77494509090925); heap 8.457683982683982 bytes/n unattributed of 8.457683982683982 total (threshold 6.673332450592798) ABOVE THRESHOLD, see O3; heap 5.136363636363637 bytes/n unattributed of 5.136363636363637 total (threshold 18.674983636363635); heap 14.151515151515152 bytes/n unattributed of 14.151515151515152 total (threshold 4.02907781818187) ABOVE THRESHOLD, see O3; non-heap 87.19090909090909 bytes/n unattributed of 87.19090909090909 total (threshold 58.19397545454546) ABOVE THRESHOLD, see O4; non-heap 47.21212121212122 bytes/n unattributed of 47.21212121212122 total (threshold 22.18958000000021) ABOVE THRESHOLD, see O4; non-heap 37.63939393939394 bytes/n unattributed of 37.63939393939394 total (threshold 27.30095581818161) ABOVE THRESHOLD, see O4; non-heap 7.88939393939394 bytes/n unattributed of 7.88939393939394 total (threshold 25.990202636364053); non-heap 26.08751803751804 bytes/n unattributed of 26.08751803751804 total (threshold 32.09436235177836); non-heap 0 bytes/n unattributed of -5.4757575757575765 total (threshold 84.02106000000043); non-heap 36.093333333333334 bytes/n unattributed of 36.093333333333334 total (threshold 50.81885556363619)

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
- Remedy: No holder-tied growth: class count flat (~1756) and listener count constant over 7000+ reloads; sub-noise JVM-internal column. Re-measure on the production extension build with heap dumps across a real update round.
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `ledger.LedgerExtension.start`
- Region: heap
- Site: `ext/ledger/LedgerExtension.java:22`
- Reason: not_exercised
- Remedy: Heap growth ~12 B/reload (jvm.heap_used, whole-process, not tied); classes and listener count flat. Re-measure on the production extension build/JVM flags with heap dumps over a real update round.
- Supporting evidence: E6

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: int[] count moved 946->949 over 7000 reloads: noise-level. Re-measure with production extension and more reloads.
- Supporting evidence: E22

### F5: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: No holder-tied growth: class count flat (~1756) and listener count constant over 7000+ reloads; sub-noise JVM-internal column. Re-measure on the production extension build with heap dumps across a real update round.
- Supporting evidence: E23

### F6: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: No holder-tied growth: class count flat (~1756) and listener count constant over 7000+ reloads; sub-noise JVM-internal column. Re-measure on the production extension build with heap dumps across a real update round.
- Supporting evidence: E24

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: No holder-tied growth: class count flat (~1756) and listener count constant over 7000+ reloads; sub-noise JVM-internal column. Re-measure on the production extension build with heap dumps across a real update round.
- Supporting evidence: E25

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: No holder-tied growth: class count flat (~1756) and listener count constant over 7000+ reloads; sub-noise JVM-internal column. Re-measure on the production extension build with heap dumps across a real update round.
- Supporting evidence: E28

### F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: No holder-tied growth: class count flat (~1756) and listener count constant over 7000+ reloads; sub-noise JVM-internal column. Re-measure on the production extension build with heap dumps across a real update round.
- Supporting evidence: E35

### F10: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: No holder-tied growth: class count flat (~1756) and listener count constant over 7000+ reloads; sub-noise JVM-internal column. Re-measure on the production extension build with heap dumps across a real update round.
- Supporting evidence: E37

### F11: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: No holder-tied growth: class count flat (~1756) and listener count constant over 7000+ reloads; sub-noise JVM-internal column. Re-measure on the production extension build with heap dumps across a real update round.
- Supporting evidence: E39

### F12: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Reason: not_exercised
- Remedy: No holder-tied growth: class count flat (~1756) and listener count constant over 7000+ reloads; sub-noise JVM-internal column. Re-measure on the production extension build with heap dumps across a real update round.
- Supporting evidence: E41

## Ruled out

- **F3** (UNCLASSIFIED): discriminator E30
  - Ruled out by: plateau+purge (non-leak) E30
  - Contradicting evidence: E30
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
- E14: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E15: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E16: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E17: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- E18: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E18: limitation: series: per-point; one process per point at n = 440, 1760, 2640, 3960 (harness captures; checked: HPROF version, identifier size)
- E19: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- E20: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E20: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E20: limitation: points 1 to 9 come from S1; the extra points from S4
- E20: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E20: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E20: limitation: gate and plateau thresholds are provisional (not calibrated)
- E21: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E21: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E21: limitation: points 1 to 9 come from S1; the extra points from S4
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E22: limitation: points 1 to 9 come from S1; the extra points from S4
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E23: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E24: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap dump
- E25: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E25: limitation: holders absent from a dump's 200 largest have no figure at that point
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: series: per-point; x is the iteration count n of each point (440, 1760, 2640, 3960); every point is a separate process and heap dump
- E26: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E26: limitation: holders absent from a dump's 200 largest have no figure at that point
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
- E29: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E29: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E29: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E29: limitation: points 1 to 9 come from S1; the extra points from S6
- E29: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E29: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E29: limitation: gate and plateau thresholds are provisional (not calibrated)
- E30: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E30: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E30: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E30: limitation: /proc/17/status lacks os.rss, os.rss_anon, os.rss_file, os.rss_shmem, os.swap, os.vm_data, os.vm_hwm (kernel thread or old kernel)
- E30: limitation: os.pss, os.pss_anon, os.private_dirty: /proc/17/smaps_rollup unreadable (No such process (os error 3))
- E30: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E30: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E30: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E30: limitation: points 1 to 9 come from S1; the extra points from S8
- E30: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E30: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E30: limitation: gate and plateau thresholds are provisional (not calibrated)
- E31: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E31: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E31: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E31: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E31: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E31: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E31: limitation: points 1 to 9 come from S1; the extra points from S10
- E31: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E31: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E31: limitation: gate and plateau thresholds are provisional (not calibrated)
- E32: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E32: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E32: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E32: limitation: gate thresholds are provisional (not calibrated)
- E33: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E33: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E33: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E33: limitation: gate thresholds are provisional (not calibrated)
- E34: limitation: /proc/17/status lacks os.rss, os.rss_anon, os.rss_file, os.rss_shmem, os.swap, os.vm_data, os.vm_hwm (kernel thread or old kernel)
- E34: limitation: os.pss, os.pss_anon, os.private_dirty: /proc/17/smaps_rollup unreadable (No such process (os error 3))
- E34: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E34: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E34: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E34: limitation: gate thresholds are provisional (not calibrated)
- E35: limitation: /proc/17/status lacks os.rss, os.rss_anon, os.rss_file, os.rss_shmem, os.swap, os.vm_data, os.vm_hwm (kernel thread or old kernel)
- E35: limitation: os.pss, os.pss_anon, os.private_dirty: /proc/17/smaps_rollup unreadable (No such process (os error 3))
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
- E38: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E38: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E38: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E38: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E38: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E38: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E38: limitation: points 1 to 9 come from S1; the extra points from S6
- E38: limitation: gate thresholds are provisional (not calibrated)
- E39: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E39: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E39: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E39: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E39: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E39: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E39: limitation: points 1 to 9 come from S1; the extra points from S6
- E39: limitation: gate thresholds are provisional (not calibrated)
- E40: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E40: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E40: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E40: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E40: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E40: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E40: limitation: points 1 to 9 come from S1; the extra points from S10
- E40: limitation: gate thresholds are provisional (not calibrated)
- E41: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E41: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E41: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E41: limitation: probe ledger.LedgerExtension: class not loaded: ledger.LedgerExtension
- E41: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E41: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E41: limitation: points 1 to 9 come from S1; the extra points from S10
- E41: limitation: gate thresholds are provisional (not calibrated)
- E42: partial coverage (2 of 3 examined)
- E42: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E42: limitation: threshold is provisional (not calibrated)
- E43: partial coverage (2 of 3 examined)
- E43: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E43: limitation: threshold is provisional (not calibrated)
- E44: partial coverage (2 of 3 examined)
- E44: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E44: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E44: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E44: limitation: threshold is provisional (not calibrated)
- E45: partial coverage (2 of 3 examined)
- E45: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E45: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"
- E45: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E45: limitation: threshold is provisional (not calibrated)
- E46: partial coverage (2 of 3 examined)
- E46: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E46: limitation: O2 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"
- E46: limitation: threshold is provisional (not calibrated)
- E47: partial coverage (2 of 3 examined)
- E47: limitation: off-heap: no gap series for S11; run dla gap S11 first (node.external is used when present)
- E47: limitation: threshold is provisional (not calibrated)
- E48: partial coverage (2 of 3 examined)
- E48: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E48: limitation: threshold is provisional (not calibrated)
- E49: partial coverage (2 of 3 examined)
- E49: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E49: limitation: threshold is provisional (not calibrated)
- E50: partial coverage (2 of 3 examined)
- E50: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E50: limitation: O4 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O4 --as unresolved --reason time_budget --remedy "<text>"
- E50: limitation: threshold is provisional (not calibrated)
- E51: partial coverage (2 of 3 examined)
- E51: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E51: limitation: O4 (non-heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O4 --as unresolved --reason time_budget --remedy "<text>"
- E51: limitation: threshold is provisional (not calibrated)
- E52: partial coverage (2 of 3 examined)
- E52: limitation: off-heap: no gap series for S11; run dla gap S11 first (node.external is used when present)
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
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (14457b16a8b5) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (fe0c3b1466b8) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (fe0c3b1466b8) | trend/T1.json |
| E12 | histogram-diff | live | jcmd-class-histogram 0.1.0 | artifact histograms/H1.json (2b1e571a8118) | histograms/H1.json |
| E13 | growth-series | live | jcmd-class-histogram 0.1.0 | artifact series/S2.json (c78fde42aff8) | series/S2.json |
| E14 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-163518-a562/harness/H1/out/p1-native/capture/105/heap-1.hprof (afab35e4ce1e) | summaries/heap-1-afab35e4.json |
| E15 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-163518-a562/harness/H1/out/p4-native/capture/466/heap-1.hprof (0b3269a3c6dd) | summaries/heap-1-0b3269a3.json |
| E16 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-163518-a562/harness/H1/out/p6-native/capture/707/heap-1.hprof (f12bf339a2ac) | summaries/heap-1-f12bf339.json |
| E17 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-163518-a562/harness/H1/out/p9-native/capture/1068/heap-1.hprof (364b6061aa25) | summaries/heap-1-364b6061.json |
| E18 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S3.json (3826e5169d2c) | series/S3.json |
| E19 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S3.json (3826e5169d2c) | series/S3.json |
| E20 | discriminator | test | dla 0.1.0 | artifact series/S5.json (bc612db9fa83) | alloc/A1.json |
| E21 | growth-series | test | dla 0.1.0 | artifact series/S5.json (bc612db9fa83) | trend/T3.json |
| E22 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (bc612db9fa83) | trend/T3.json |
| E23 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (c78fde42aff8) | trend/T4.json |
| E24 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (c78fde42aff8) | trend/T4.json |
| E25 | growth-series | dump | dla 0.1.0 | artifact series/S3.json (3826e5169d2c) | trend/T5.json |
| E26 | unit-count-series | dump | dla 0.1.0 | artifact series/S3.json (3826e5169d2c) | trend/T5.json |
| E27 | growth-series | test | dla 0.1.0 | artifact series/S4.json (cd8b28b31eec) | trend/T6.json |
| E28 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (cd8b28b31eec) | trend/T6.json |
| E29 | discriminator | test | dla 0.1.0 | artifact series/S7.json (ad290b4862b9) | alloc/A2.json |
| E30 | discriminator | test | dla 0.1.0 | artifact series/S9.json (bc2d03a0ff1f) | alloc/A3.json |
| E31 | discriminator | test | dla 0.1.0 | artifact series/S11.json (67b9b4b47bfa) | alloc/A4.json |
| E32 | growth-series | test | dla 0.1.0 | artifact series/S6.json (492ac12064ff) | trend/T10.json |
| E33 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (492ac12064ff) | trend/T10.json |
| E34 | growth-series | test | dla 0.1.0 | artifact series/S8.json (bac17dced6d2) | trend/T11.json |
| E35 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (bac17dced6d2) | trend/T11.json |
| E36 | growth-series | test | dla 0.1.0 | artifact series/S10.json (717a35bf47ad) | trend/T12.json |
| E37 | unit-count-series | test | dla 0.1.0 | artifact series/S10.json (717a35bf47ad) | trend/T12.json |
| E38 | growth-series | test | dla 0.1.0 | artifact series/S7.json (ad290b4862b9) | trend/T13.json |
| E39 | unit-count-series | test | dla 0.1.0 | artifact series/S7.json (ad290b4862b9) | trend/T13.json |
| E40 | growth-series | test | dla 0.1.0 | artifact series/S11.json (67b9b4b47bfa) | trend/T14.json |
| E41 | unit-count-series | test | dla 0.1.0 | artifact series/S11.json (67b9b4b47bfa) | trend/T14.json |
| E42 | residual | test | dla 0.1.0 | artifact series/S1.json (fe0c3b1466b8) | gap/residual-S1-E42.json |
| E43 | residual | test | dla 0.1.0 | artifact series/S5.json (bc612db9fa83) | gap/residual-S5-E43.json |
| E44 | residual | test | dla 0.1.0 | artifact series/S6.json (492ac12064ff) | gap/residual-S6-E44.json |
| E45 | residual | test | dla 0.1.0 | artifact series/S8.json (bac17dced6d2) | gap/residual-S8-E45.json |
| E46 | residual | test | dla 0.1.0 | artifact series/S7.json (ad290b4862b9) | gap/residual-S7-E46.json |
| E47 | residual | test | dla 0.1.0 | artifact series/S11.json (67b9b4b47bfa) | gap/residual-S11-E47.json |
| E48 | residual | test | dla 0.1.0 | artifact series/S1.json (fe0c3b1466b8) | gap/residual-S1-E48.json |
| E49 | residual | test | dla 0.1.0 | artifact series/S5.json (bc612db9fa83) | gap/residual-S5-E49.json |
| E50 | residual | test | dla 0.1.0 | artifact series/S9.json (bc2d03a0ff1f) | gap/residual-S9-E50.json |
| E51 | residual | test | dla 0.1.0 | artifact series/S7.json (ad290b4862b9) | gap/residual-S7-E51.json |
| E52 | residual | test | dla 0.1.0 | artifact series/S11.json (67b9b4b47bfa) | gap/residual-S11-E52.json |
