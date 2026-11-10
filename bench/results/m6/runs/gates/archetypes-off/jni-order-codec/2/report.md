# DLA report

Run: `20261110-184414-f144`

Status: INCOMPLETE (58 open)

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 23 enumerated, 6 dispositioned; confirmed 2, strong 0, refuted 2, suspected 0, unresolved 2

Residual: not measured

Coverage: jvm 17760 iterations, 0 of 17 entry points exercised

## Open items

- **F2**: finding F2 is candidate and has no disposition. Fix: `dla ledger resolve F2 --as unresolved --reason time_budget --remedy "<text>"`
- **F4**: finding F4 is candidate and has no disposition. Fix: `dla ledger resolve F4 --as unresolved --reason time_budget --remedy "<text>"`
- **F5**: finding F5 is candidate and has no disposition. Fix: `dla ledger resolve F5 --as unresolved --reason time_budget --remedy "<text>"`
- **F6**: finding F6 is candidate and has no disposition. Fix: `dla ledger resolve F6 --as unresolved --reason time_budget --remedy "<text>"`
- **F7**: finding F7 is candidate and has no disposition. Fix: `dla ledger resolve F7 --as unresolved --reason time_budget --remedy "<text>"`
- **F8**: finding F8 is candidate and has no disposition. Fix: `dla ledger resolve F8 --as unresolved --reason time_budget --remedy "<text>"`
- **F9**: finding F9 is candidate and has no disposition. Fix: `dla ledger resolve F9 --as unresolved --reason time_budget --remedy "<text>"`
- **F10**: finding F10 is candidate and has no disposition. Fix: `dla ledger resolve F10 --as unresolved --reason time_budget --remedy "<text>"`
- **F11**: finding F11 is candidate and has no disposition. Fix: `dla ledger resolve F11 --as unresolved --reason time_budget --remedy "<text>"`
- **F12**: finding F12 is candidate and has no disposition. Fix: `dla ledger resolve F12 --as unresolved --reason time_budget --remedy "<text>"`
- **F13**: finding F13 is candidate and has no disposition. Fix: `dla ledger resolve F13 --as unresolved --reason time_budget --remedy "<text>"`
- **F14**: finding F14 is candidate and has no disposition. Fix: `dla ledger resolve F14 --as unresolved --reason time_budget --remedy "<text>"`
- **F15**: finding F15 is candidate and has no disposition. Fix: `dla ledger resolve F15 --as unresolved --reason time_budget --remedy "<text>"`
- **F16**: finding F16 is candidate and has no disposition. Fix: `dla ledger resolve F16 --as unresolved --reason time_budget --remedy "<text>"`
- **F17**: finding F17 is candidate and has no disposition. Fix: `dla ledger resolve F17 --as unresolved --reason time_budget --remedy "<text>"`
- **F18**: finding F18 is candidate and has no disposition. Fix: `dla ledger resolve F18 --as unresolved --reason time_budget --remedy "<text>"`
- **F19**: finding F19 is candidate and has no disposition. Fix: `dla ledger resolve F19 --as unresolved --reason time_budget --remedy "<text>"`
- **T1:os.rss**: column os.rss of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:os.rss_anon**: column os.rss_anon of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:os.vm_hwm**: column os.vm_hwm of T1 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:jvm.heap_committed**: column jvm.heap_committed of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:jvm.heap_used**: column jvm.heap_used of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:native.live_blocks**: column native.live_blocks of T1 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:native.live_bytes**: column native.live_bytes of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:native.peak_bytes**: column native.peak_bytes of T1 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:jvm.heap_used.pre_gc**: column jvm.heap_used.pre_gc of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:os.rss**: column os.rss of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:os.rss_anon**: column os.rss_anon of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:os.vm_data**: column os.vm_data of T4 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:os.vm_hwm**: column os.vm_hwm of T4 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:jvm.heap_committed**: column jvm.heap_committed of T4 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:jvm.heap_used**: column jvm.heap_used of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:jvm.nmt_committed**: column jvm.nmt_committed of T4 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:native.live_bytes**: column native.live_bytes of T4 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:native.peak_bytes**: column native.peak_bytes of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:jvm.heap_committed.pre_gc**: column jvm.heap_committed.pre_gc of T4 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.rss**: column os.rss of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.rss_anon**: column os.rss_anon of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.vm_data**: column os.vm_data of T5 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.vm_hwm**: column os.vm_hwm of T5 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:jvm.heap_used**: column jvm.heap_used of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:native.live_blocks**: column native.live_blocks of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:native.live_bytes**: column native.live_bytes of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:native.peak_bytes**: column native.peak_bytes of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:os.rss**: column os.rss of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:os.rss_anon**: column os.rss_anon of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:os.vm_hwm**: column os.vm_hwm of T6 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:jvm.heap_used**: column jvm.heap_used of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:native.live_blocks**: column native.live_blocks of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:native.live_bytes**: column native.live_bytes of T6 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:native.peak_bytes**: column native.peak_bytes of T6 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:os.rss**: column os.rss of T7 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:os.rss_anon**: column os.rss_anon of T7 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:jvm.heap_used**: column jvm.heap_used of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:native.live_blocks**: column native.live_blocks of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:native.live_bytes**: column native.live_bytes of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:native.peak_bytes**: column native.peak_bytes of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:jvm.heap_used.pre_gc**: column jvm.heap_used.pre_gc of T7 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`

## Findings

### F3: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Audit.trail (static ArrayList) gets a 1536-byte copy per inline frame via Audit.record and is never trimmed: unbounded growth, ~1.4KB/order.)
- Growth: `probe:shop.codec.Audit.trail` 2.625 count/n (steady, gate pass, E14)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `shop.codec.Audit.trail`
- Region: heap
- Shape: steady
- Site: `kotlin/shop/Audit.kt:9`
- Writers: `kotlin/shop/Audit.kt:12`
- Annotation: heuristic name match over the project's sources
- Annotation: Audit.trail only ever added to; no remover
- Verified: no
- Sibling hypotheses: F3.n1
- Supporting evidence: E7, E12, E14
- Explanation (observed): Audit.trail (Audit.kt:9) is a static ArrayList; Audit.record (Audit.kt:12) appends a 1536-byte Entry per inline frame and nothing removes entries, so it is never trimmed.
- Explanation structure: adds: kotlin/shop/Audit.kt:12; removal: never; evidence: E14

### F20: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Native frame table FRAMES (lib.rs) holds a Frame per handle from Native.open; OrderEncoder.encode returns early on the bulk lane (size > 16384, wholesale orders, every 8th) without calling Native.close, so those handles' Vec<u8> are never removed (~24.6KB each).)
- Growth: `native.live_blocks@native/src/lib.rs:40` 0.125 count/n (steady, gate pass, E14)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `native.src.lib.FRAMES`
- Region: heap
- Shape: steady
- Site: `native/src/lib.rs:19`
- Writers: `native/src/lib.rs:40`
- Removers: `native/src/lib.rs:73`
- Annotation: FRAMES entry inserted in open(); Native.close is the only remover and Encoder.encode skips it for frames > 16384 bytes (Encoder.kt:14-18)
- Verified: no
- Sibling hypotheses: F20.n1
- Supporting evidence: E9, E12, E14
- Explanation (observed): FRAMES (lib.rs:19) gets a Frame per Native.open (lib.rs:40); the remover Native.close (lib.rs:73) runs only when OrderEncoder.encode sees a frame of 16384 bytes or less, so wholesale frames stay.
- Explanation structure: adds: native/src/lib.rs:40; removal: conditional at native/src/lib.rs:73; evidence: E14

### F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `kotlin/shop/Audit.kt:9`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F3 (same site Audit.kt:9, holder Audit.trail); no separate test needed
- Supporting evidence: E6

### F21: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `shop.codec.Audit.trail`
- Region: heap
- Shape: steady
- Site: `kotlin/shop/Audit.kt:9`
- Reason: not_exercised
- Remedy: Duplicate of confirmed F3 (same site Audit.kt:9, holder Audit.trail); no separate test needed
- Supporting evidence: E18, E20, E22, E24

### F2: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.Entry.sample`
- Region: heap
- Site: `kotlin/shop/Audit.kt:3`
- Supporting evidence: E7

### F4: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.OrderEncoder.Companion`
- Region: heap
- Site: `kotlin/shop/Encoder.kt:6`
- Supporting evidence: E7

### F5: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.FramePacker.MAGIC`
- Region: heap
- Site: `kotlin/shop/FramePacker.kt:9`
- Supporting evidence: E7

### F6: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.Native.INSTANCE`
- Region: heap
- Site: `kotlin/shop/Native.kt:5`
- Supporting evidence: E7

### F7: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.<clinit>`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:6`
- Supporting evidence: E7

### F8: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.open`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:9`
- Supporting evidence: E7

### F9: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.checksum`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:10`
- Supporting evidence: E7

### F10: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.size`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:11`
- Supporting evidence: E7

### F11: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.lines`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:12`
- Supporting evidence: E7

### F12: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.close`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:13`
- Supporting evidence: E7

### F13: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.live`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:14`
- Supporting evidence: E7

### F14: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.liveBytes`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:15`
- Supporting evidence: E7

### F15: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.OrderGenerator.Companion`
- Region: heap
- Site: `kotlin/shop/Order.kt:12`
- Supporting evidence: E7

### F16: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.OrderGenerator.skus`
- Region: heap
- Site: `kotlin/shop/Order.kt:14`
- Supporting evidence: E7

### F17: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.Order.lines`
- Region: heap
- Site: `kotlin/shop/Order.kt:18`
- Supporting evidence: E7

### F18: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.Stats.Companion`
- Region: heap
- Site: `kotlin/shop/Stats.kt:6`
- Supporting evidence: E7

### F19: **UNCLASSIFIED**

- State: candidate
- Evidence classes: static
- Holder: container-field
- Owner: `native.src.frame.Frame.bytes`
- Region: heap
- Site: `native/src/frame.rs:11`
- Supporting evidence: E9

## Ruled out

- **F3.n1** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (leak) E16
  - Hypothesis: N-WARM: growth is warm-up fill / cache high-water mark that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E16
  - Originally supported by: E14
- **F20.n1** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (leak) E15
  - Hypothesis: N-WARM: growth is warm-up fill / cache high-water mark that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E15
  - Originally supported by: E14

## Blind spots and unknowns

### Coverage gaps

- jvm/codeql: missing (codeql is not on PATH or in the tools directory). Install hint: dla install codeql
- jvm/errorprone: missing (Error Prone jars not found in the tools directory (or DLA_ERRORPRONE_HOME)). Install hint: dla install errorprone
- jvm/sonar: missing (sonar-scanner is not installed and Sonar rules need a SonarQube or SonarCloud server). Install hint: install sonar-scanner, run a SonarQube or SonarCloud server and set SONAR_HOST_URL and SONAR_TOKEN (dla install sonar prints the steps)
- jvm/valgrind: blocked (Valgrind is not run on the JVM: the virtual machine's own allocations drown the project's; use the allocator shim, jcmd NMT or `dla attach bpftrace`)
- rust/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- rust/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)
- rust/rapx: blocked (cannot copy the project: Permission denied (os error 13))

### Runtime blind spots

None recorded.

### Entry points not exercisable

None recorded.

### Evidence limitations and unknowns

- E4: partial coverage
- E4: limitation: no auxiliary classpath: library calls are not modelled
- E7: partial coverage
- E7: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E7: limitation: writers and removers are textual name matches
- E9: partial coverage
- E9: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E9: limitation: writers and removers are textual name matches
- E10: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E10: limitation: workloads were discovered by reading files; none was run
- E11: limitation: harness written; nothing was run (dla harness run measures it)
- E11: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E11: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- E11: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E12: partial coverage (9 of 9 examined)
- E12: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E12: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E12: limitation: 29436338 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: 20305298 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: 32303746 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: 31976002 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: 20551099 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E13: limitation: 29436338 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: 20305298 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: 32303746 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: 31976002 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: 20551099 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: 31699009 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: 25078443 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: 29013374 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: 31251751 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E14: limitation: 29436338 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: 20305298 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: 32303746 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: 31976002 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: 20551099 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: 31699009 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: 25078443 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: 29013374 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: 31251751 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: partial coverage (9 of 64 examined)
- E15: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E15: limitation: 29436338 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: 20305298 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: 32303746 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: 31976002 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: 20551099 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: 31699009 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: 25078443 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: 29013374 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: 31251751 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- E15: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E15: limitation: 22438627 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: 31784635 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: 20465148 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: 30446094 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: 29753705 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: 24122467 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: 25084571 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E15: limitation: 30613997 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E15: limitation: points 1 to 9 come from S1; the extra points from S2
- E15: limitation: 9 columns beyond the 64 the series allows were dropped
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- E15: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E16: limitation: 29436338 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 20305298 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 32303746 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 31976002 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 20551099 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 31699009 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 25078443 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 29013374 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 31251751 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- E16: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E16: limitation: 34754406 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 31807826 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 21038152 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 30028433 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 22393772 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 21527980 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 21239080 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E16: limitation: 31631563 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E16: limitation: points 1 to 9 come from S1; the extra points from S4
- E16: limitation: 9 columns beyond the 64 the series allows were dropped
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- E16: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E17: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E17: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E17: limitation: 22438627 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 31784635 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 20465148 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 30446094 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 29753705 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 24122467 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 25084571 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E17: limitation: 30613997 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E18: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E18: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E18: limitation: 22438627 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 31784635 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 20465148 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 30446094 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 29753705 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 24122467 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 25084571 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E18: limitation: 30613997 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E19: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E19: limitation: 29436338 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 20305298 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 32303746 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 31976002 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 20551099 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 31699009 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 25078443 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 29013374 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 31251751 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- E19: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E19: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E19: limitation: 22438627 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 31784635 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 20465148 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 30446094 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 29753705 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 24122467 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 25084571 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E19: limitation: 30613997 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E19: limitation: points 1 to 9 come from S1; the extra points from S2
- E19: limitation: 9 columns beyond the 64 the series allows were dropped
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E20: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E20: limitation: 29436338 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: 20305298 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: 32303746 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: 31976002 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: 20551099 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: 31699009 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: 25078443 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: 29013374 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: 31251751 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- E20: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E20: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E20: limitation: 22438627 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: 31784635 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: 20465148 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: 30446094 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: 29753705 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: 24122467 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: 25084571 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E20: limitation: 30613997 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E20: limitation: points 1 to 9 come from S1; the extra points from S2
- E20: limitation: 9 columns beyond the 64 the series allows were dropped
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E21: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E21: limitation: 34754406 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: 31807826 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: 21038152 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: 30028433 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: 22393772 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: 21527980 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: 21239080 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E21: limitation: 31631563 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E22: limitation: 34754406 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: 31807826 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: 21038152 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: 30028433 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: 22393772 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: 21527980 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: 21239080 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E22: limitation: 31631563 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E23: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E23: limitation: 29436338 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: 20305298 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: 32303746 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: 31976002 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: 20551099 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: 31699009 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: 25078443 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: 29013374 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: 31251751 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- E23: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E23: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E23: limitation: 34754406 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: 31807826 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: 21038152 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: 30028433 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: 22393772 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: 21527980 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: 21239080 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E23: limitation: 31631563 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E23: limitation: points 1 to 9 come from S1; the extra points from S4
- E23: limitation: 9 columns beyond the 64 the series allows were dropped
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- E24: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E24: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E24: limitation: 29436338 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: 20305298 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: 32303746 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: 31976002 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: 20551099 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: 31699009 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: 25078443 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: 29013374 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: 31251751 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- E24: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E24: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E24: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E24: limitation: 34754406 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: 31807826 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: 21038152 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: 30028433 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: 22393772 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: 21527980 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: 21239080 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E24: limitation: 31631563 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E24: limitation: points 1 to 9 come from S1; the extra points from S4
- E24: limitation: 9 columns beyond the 64 the series allows were dropped
- E24: limitation: gate thresholds are provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | — |
| E2 | run-config | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | probe/probe.json |
| E4 | static-hit | static | spotbugs unknown | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-spotbugs.json |
| E5 | static-hit | static | detekt 2.0.0-alpha.6 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-detekt.json |
| E6 | static-hit | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-dla.json |
| E7 | census-holder | static | dla-census 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | census/jvm.json |
| E8 | static-hit | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/rust-dla.json |
| E9 | census-holder | static | dla-census 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | census/rust.json |
| E10 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | workload/workloads.json |
| E11 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | harness/H1/harness.json |
| E12 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (4d12ec3c36f8) | harness/H1/run-S1.json |
| E13 | growth-series | test | dla 0.1.0 | artifact series/S1.json (f4a8b051db74) | trend/T1.json |
| E14 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (f4a8b051db74) | trend/T1.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S3.json (9dd899cc9ddf) | alloc/A1.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S5.json (3e0c8cd31344) | alloc/A2.json |
| E17 | growth-series | test | dla 0.1.0 | artifact series/S2.json (e1b867bc68f5) | trend/T4.json |
| E18 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (e1b867bc68f5) | trend/T4.json |
| E19 | growth-series | test | dla 0.1.0 | artifact series/S3.json (9dd899cc9ddf) | trend/T5.json |
| E20 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (9dd899cc9ddf) | trend/T5.json |
| E21 | growth-series | test | dla 0.1.0 | artifact series/S4.json (7955c3280d16) | trend/T6.json |
| E22 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (7955c3280d16) | trend/T6.json |
| E23 | growth-series | test | dla 0.1.0 | artifact series/S5.json (3e0c8cd31344) | trend/T7.json |
| E24 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (3e0c8cd31344) | trend/T7.json |
