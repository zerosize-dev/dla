# DLA report

Run: `20261110-180923-a3f2`

Status: INCOMPLETE (34 open)

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 23 enumerated, 23 dispositioned; confirmed 1, strong 1, refuted 2, suspected 19, unresolved 0

Residual: heap 1454.7726726726728 bytes/n unattributed of 1454.7726726726728 total (threshold 363.6931681681682) ABOVE THRESHOLD, see O1; heap 1441.770313170313 bytes/n unattributed of 1441.770313170313 total (threshold 360.4425782925783) ABOVE THRESHOLD, see O1; heap 1440.0901115401116 bytes/n unattributed of 1440.0901115401116 total (threshold 360.0225278850279) ABOVE THRESHOLD, see O1; non-heap 18.75987987987988 bytes/n unattributed of 18.75987987987988 total (threshold 79.66191070270246); non-heap 0 bytes/n unattributed of -3.6046449046449047 total (threshold 7.774759950859895); non-heap 0 bytes/n unattributed of -5.984298584298584 total (threshold 5.369301729729712)

Coverage: jvm 17760 iterations, 0 of 17 entry points exercised; rust no iterations, 0 of 10 entry points exercised

## Open items

- **T3:jvm.class:byte[].count**: column jvm.class:byte[].count of T3 (steady) is not flat and is tied to no finding. Fix: `dla trend S5`
- **T3:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T3 (steady) is not flat and is tied to no finding. Fix: `dla trend S5`
- **T3:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T3 (steady) is not flat and is tied to no finding. Fix: `dla trend S5`
- **T4:jvm.class:byte[].count**: column jvm.class:byte[].count of T4 (steady) is not flat and is tied to no finding. Fix: `dla trend S7`
- **T4:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T4 (steady) is not flat and is tied to no finding. Fix: `dla trend S7`
- **T4:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T4 (steady) is not flat and is tied to no finding. Fix: `dla trend S7`
- **T6:type:byte[]**: column type:byte[] of T6 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E6 # column type:byte[]`
- **T6:type:byte[].count**: column type:byte[].count of T6 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E7 # column type:byte[].count`
- **T6:type:java.lang.String**: column type:java.lang.String of T6 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E6 # column type:java.lang.String`
- **T6:type:java.lang.String.count**: column type:java.lang.String.count of T6 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E7 # column type:java.lang.String.count`
- **T6:type:jdk.internal.vm.FillerElement[]**: column type:jdk.internal.vm.FillerElement[] of T6 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E6 # column type:jdk.internal.vm.FillerElement[]`
- **T6:type:jdk.internal.vm.FillerElement[].count**: column type:jdk.internal.vm.FillerElement[].count of T6 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E7 # column type:jdk.internal.vm.FillerElement[].count`
- **T6:type:java.lang.Object[]**: column type:java.lang.Object[] of T6 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E6 # column type:java.lang.Object[]`
- **T6:type:java.lang.Long**: column type:java.lang.Long of T6 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E6 # column type:java.lang.Long`
- **T6:type:java.lang.Long.count**: column type:java.lang.Long.count of T6 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E7 # column type:java.lang.Long.count`
- **T7:os.rss**: column os.rss of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:os.rss_anon**: column os.rss_anon of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:jvm.heap_used**: column jvm.heap_used of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:native.live_bytes**: column native.live_bytes of T7 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:jvm.class:byte[].count**: column jvm.class:byte[].count of T7 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E9 # column jvm.class:byte[].count`
- **T7:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E9 # column jvm.class:java.lang.String.count`
- **T7:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E9 # column jvm.class:shop.codec.Entry.count`
- **T8:os.rss**: column os.rss of T8 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:os.rss_anon**: column os.rss_anon of T8 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:os.vm_data**: column os.vm_data of T8 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:os.vm_hwm**: column os.vm_hwm of T8 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:jvm.heap_committed**: column jvm.heap_committed of T8 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:jvm.heap_used**: column jvm.heap_used of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:native.live_blocks**: column native.live_blocks of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:native.live_bytes**: column native.live_bytes of T8 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:native.peak_bytes**: column native.peak_bytes of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:jvm.class:byte[].count**: column jvm.class:byte[].count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E11 # column jvm.class:byte[].count`
- **T8:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E11 # column jvm.class:java.lang.String.count`
- **T8:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E11 # column jvm.class:shop.codec.Entry.count`

## Workers

- **jvm** (jvm): status done; 77 lines merged; slice done; torn tail: no
- **rust** (rust): status done; 12 lines merged; slice done; torn tail: no

## Findings

### jvm/F3: **UNCLASSIFIED**

- State: strong
- Label: UNCLASSIFIED (Static object Audit.trail (Audit.kt:9) accumulates a 1536-byte sample per inline frame via Audit.record (Audit.kt:12); never removed)
- Growth: `holder:static:shop.codec.Audit.trail` 8501451 bytes/n (steady, gate pass, jvm/E14)
- Evidence classes: context, growth, retention, static
- Strong: second class jvm/E10 retention-path that names the holder, independent of the growth record jvm/E20 (dla)
- Holder: static/global
- Owner: `shop.codec.Audit.trail`
- Region: heap
- Shape: steady
- Site: `kotlin/shop/Audit.kt:9`
- Writers: `kotlin/shop/Audit.kt:12`, `kotlin/shop/Encoder.kt:17`
- Annotation: heuristic name match over the project's sources
- Annotation: OrderEncoder.encode calls Audit.record for every inline frame; trail is never trimmed
- Verified: no
- Sibling hypotheses: jvm/F3.n1
- Supporting evidence: jvm/E4, jvm/E7, jvm/E14, jvm/E20, jvm/E10, jvm/E13
- Explanation (inferred): Audit.trail (kotlin/shop/Audit.kt:9) is a static object list; Audit.record at Audit.kt:12 adds a 1536-byte Entry sample per inline frame and nothing removes entries, the removal never happens, so the static holder retains them forever.
- Explanation structure: adds: kotlin/shop/Audit.kt:12; removal: never

### jvm/F19: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Native frame handle from Native.open (lib.rs:40) is not closed on the bulk branch (frames over 16384 bytes) in OrderEncoder.encode, so native memory grows)
- Growth: `native.live_blocks@native/src/lib.rs:40` 0.125 count/n (steady, gate pass, jvm/E20)
- Evidence classes: allocation, context, growth
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.open`
- Region: off-heap
- Shape: steady
- Site: `native/src/lib.rs:40`
- Writers: `kotlin/shop/Native.kt:9`, `kotlin/shop/Encoder.kt:9`
- Annotation: native allocation under shop.codec.Native.open (Java_shop_codec_Native_open); found by dla correlate
- Annotation: Native.open allocates a frame; the bulk branch at Encoder.kt:12-16 returns without Native.close
- Verified: no
- Sibling hypotheses: jvm/F19.n1
- Supporting evidence: jvm/E18, jvm/E7, jvm/E20
- Explanation (inferred): Native.open at Encoder.kt:9 allocates a native frame at native/src/lib.rs:40 held as a native handle; the bulk branch (size over 16384) returns at Encoder.kt:15 without Native.close, and found no remover for those handles; not found anywhere.
- Explanation structure: adds: kotlin/shop/Encoder.kt:9; removal: not-found

### jvm/F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Site: `kotlin/shop/Audit.kt:9`
- Reason: not_exercised
- Remedy: Same retention as confirmed jvm/F3 (Audit.trail holds Entry samples); no separate growth tie. Fix F3: bound or drop the trail
- Supporting evidence: jvm/E3

### jvm/F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.Entry.sample`
- Region: heap
- Site: `kotlin/shop/Audit.kt:3`
- Reason: not_exercised
- Remedy: Same retention as confirmed jvm/F3 (Audit.trail holds Entry samples); no separate growth tie. Fix F3: bound or drop the trail
- Supporting evidence: jvm/E4

### jvm/F4: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.OrderEncoder.Companion`
- Region: heap
- Site: `kotlin/shop/Encoder.kt:6`
- Reason: not_exercised
- Remedy: No probe or column grew for this holder; static census only (constant, binding declaration or bounded structure). Re-test with a longer run if code changes
- Supporting evidence: jvm/E4

### jvm/F5: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.FramePacker.MAGIC`
- Region: heap
- Site: `kotlin/shop/FramePacker.kt:9`
- Reason: not_exercised
- Remedy: No probe or column grew for this holder; static census only (constant, binding declaration or bounded structure). Re-test with a longer run if code changes
- Supporting evidence: jvm/E4

### jvm/F6: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.Native.INSTANCE`
- Region: heap
- Site: `kotlin/shop/Native.kt:5`
- Reason: not_exercised
- Remedy: No probe or column grew for this holder; static census only (constant, binding declaration or bounded structure). Re-test with a longer run if code changes
- Supporting evidence: jvm/E4

### jvm/F7: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.<clinit>`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:6`
- Reason: not_exercised
- Remedy: No probe or column grew for this holder; static census only (constant, binding declaration or bounded structure). Re-test with a longer run if code changes
- Supporting evidence: jvm/E4

### jvm/F8: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.open`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:9`
- Reason: not_exercised
- Remedy: No probe or column grew for this holder; static census only (constant, binding declaration or bounded structure). Re-test with a longer run if code changes
- Supporting evidence: jvm/E4

### jvm/F9: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.checksum`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:10`
- Reason: not_exercised
- Remedy: No probe or column grew for this holder; static census only (constant, binding declaration or bounded structure). Re-test with a longer run if code changes
- Supporting evidence: jvm/E4

### jvm/F10: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.size`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:11`
- Reason: not_exercised
- Remedy: No probe or column grew for this holder; static census only (constant, binding declaration or bounded structure). Re-test with a longer run if code changes
- Supporting evidence: jvm/E4

### jvm/F11: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.lines`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:12`
- Reason: not_exercised
- Remedy: No probe or column grew for this holder; static census only (constant, binding declaration or bounded structure). Re-test with a longer run if code changes
- Supporting evidence: jvm/E4

### jvm/F12: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.close`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:13`
- Reason: not_exercised
- Remedy: No probe or column grew for this holder; static census only (constant, binding declaration or bounded structure). Re-test with a longer run if code changes
- Supporting evidence: jvm/E4

### jvm/F13: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.live`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:14`
- Reason: not_exercised
- Remedy: No probe or column grew for this holder; static census only (constant, binding declaration or bounded structure). Re-test with a longer run if code changes
- Supporting evidence: jvm/E4

### jvm/F14: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.liveBytes`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:15`
- Reason: not_exercised
- Remedy: No probe or column grew for this holder; static census only (constant, binding declaration or bounded structure). Re-test with a longer run if code changes
- Supporting evidence: jvm/E4

### jvm/F15: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.OrderGenerator.Companion`
- Region: heap
- Site: `kotlin/shop/Order.kt:12`
- Reason: not_exercised
- Remedy: No probe or column grew for this holder; static census only (constant, binding declaration or bounded structure). Re-test with a longer run if code changes
- Supporting evidence: jvm/E4

### jvm/F16: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.OrderGenerator.skus`
- Region: heap
- Site: `kotlin/shop/Order.kt:14`
- Reason: not_exercised
- Remedy: No probe or column grew for this holder; static census only (constant, binding declaration or bounded structure). Re-test with a longer run if code changes
- Supporting evidence: jvm/E4

### jvm/F17: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.Order.lines`
- Region: heap
- Site: `kotlin/shop/Order.kt:18`
- Reason: not_exercised
- Remedy: No probe or column grew for this holder; static census only (constant, binding declaration or bounded structure). Re-test with a longer run if code changes
- Supporting evidence: jvm/E4

### jvm/F18: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.Stats.Companion`
- Region: heap
- Site: `kotlin/shop/Stats.kt:6`
- Reason: not_exercised
- Remedy: No probe or column grew for this holder; static census only (constant, binding declaration or bounded structure). Re-test with a longer run if code changes
- Supporting evidence: jvm/E4

### rust/F1: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `native.src.frame.Frame.bytes`
- Region: heap
- Site: `native/src/frame.rs:11`
- Writers: `native/src/lib.rs:44`
- Annotation: Frame.bytes is owned by an entry of FRAMES; freed only when close() removes the entry
- Reason: no_harness
- Remedy: Frame.bytes is owned by the FRAMES entries (rust/F2); same JVM-side shim measurement and correlate H1 applies.
- Supporting evidence: rust/E3

### rust/F2: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: static/global
- Owner: `native.src.lib.FRAMES`
- Region: heap
- Site: `native/src/lib.rs:19`
- Writers: `native/src/lib.rs:44`
- Removers: `native/src/lib.rs:68`
- Annotation: FRAMES static map: open() inserts per call; close() removes. Kotlin OrderEncoder.encode returns before Native.close for frames over 16384 bytes (bulk lane), so those handles are never removed.
- Reason: no_harness
- Remedy: Crate is a cdylib with no bin or rlib target, so no Rust harness runs. Measure from the JVM worker with the allocator shim (dla harness run, then dla correlate H1) over orders with more than 255 lines (frame over 16384 bytes, which skips Native.close); cause site native/src/lib.rs:44 (FRAMES insert in Java_shop_codec_Native_open).
- Supporting evidence: rust/E3

## Ruled out

- **jvm/F3.n1** (UNCLASSIFIED): discriminator jvm/E21
  - Ruled out by: plateau+purge (leak) jvm/E21
  - Hypothesis: N-WARM: Audit.trail fills to a cap then plateaus
  - Refuted if: alloc-test shows trail size and retained bytes plateau over extra cycles
  - Contradicting evidence: jvm/E21
  - Originally supported by: jvm/E9
- **jvm/F19.n1** (UNCLASSIFIED): discriminator jvm/E22
  - Ruled out by: plateau+purge (leak) jvm/E22
  - Hypothesis: N-HWM: native live bytes are a bounded pool high-water mark
  - Refuted if: alloc-test shows native.live_bytes plateaus after purge
  - Contradicting evidence: jvm/E22
  - Originally supported by: jvm/E18

## Blind spots and unknowns

### Coverage gaps

- jvm/codeql: missing (codeql is not on PATH or in the tools directory). Install hint: dla install codeql
- jvm/errorprone: missing (Error Prone jars not found in the tools directory (or DLA_ERRORPRONE_HOME)). Install hint: dla install errorprone
- jvm/sonar: missing (sonar-scanner is not installed and Sonar rules need a SonarQube or SonarCloud server). Install hint: install sonar-scanner, run a SonarQube or SonarCloud server and set SONAR_HOST_URL and SONAR_TOKEN (dla install sonar prints the steps)
- jvm/valgrind: blocked (Valgrind is not run on the JVM: the virtual machine's own allocations drown the project's; use the allocator shim, jcmd NMT or `dla attach bpftrace`)
- rust/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- rust/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)

### Runtime blind spots

None recorded.

### Entry points not exercisable

- rust: `native/src/lib.rs::Java_shop_codec_Native_checksum`: no_harness
- rust: `native/src/lib.rs::Java_shop_codec_Native_close`: no_harness
- rust: `native/src/lib.rs::Java_shop_codec_Native_lines`: no_harness
- rust: `native/src/lib.rs::Java_shop_codec_Native_live`: no_harness
- rust: `native/src/lib.rs::Java_shop_codec_Native_liveBytes`: no_harness
- rust: `native/src/lib.rs::Java_shop_codec_Native_open`: no_harness
- rust: `native/src/lib.rs::Java_shop_codec_Native_size`: no_harness

### Evidence limitations and unknowns

- E4: limitation: series: per-point; x is the iteration count n of each point (2220, 8880, 13320, 19980); every point is a separate process and heap dump
- E4: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E4: limitation: holders absent from a dump's 200 largest have no figure at that point
- E4: limitation: gate thresholds are provisional (not calibrated)
- E5: limitation: series: per-point; x is the iteration count n of each point (2220, 8880, 13320, 19980); every point is a separate process and heap dump
- E5: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E5: limitation: holders absent from a dump's 200 largest have no figure at that point
- E5: limitation: gate thresholds are provisional (not calibrated)
- E6: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E6: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E6: limitation: gate thresholds are provisional (not calibrated)
- E7: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E7: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E7: limitation: gate thresholds are provisional (not calibrated)
- E8: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E8: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E8: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E8: limitation: 29226278 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: 20615103 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: 21010077 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: 29821674 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: 31988357 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: 29589451 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: 27156157 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E8: limitation: 21096996 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E8: limitation: gate thresholds are provisional (not calibrated)
- E9: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E9: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E9: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E9: limitation: 29226278 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: 20615103 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: 21010077 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: 29821674 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: 31988357 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: 29589451 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: 27156157 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E9: limitation: 21096996 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E9: limitation: gate thresholds are provisional (not calibrated)
- E10: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E10: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E10: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E10: limitation: 25008486 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: 20576002 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: 33282358 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: 21107597 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: 19713034 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: 21566894 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: 32346065 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E10: limitation: 32066421 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E11: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E11: limitation: 25008486 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: 20576002 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: 33282358 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: 21107597 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: 19713034 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: 21566894 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: 32346065 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E11: limitation: 32066421 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: partial coverage (2 of 3 examined)
- E12: limitation: jvm/F3: growth measured as a count; bytes not attributed
- E12: limitation: jvm/F19: growth measured as a count; bytes not attributed
- E12: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E12: limitation: threshold is provisional (not calibrated)
- E13: partial coverage (2 of 3 examined)
- E13: limitation: jvm/F3: growth evidence is from S2, not S5; not subtracted
- E13: limitation: jvm/F19: growth evidence is from S1, not S5; not subtracted
- E13: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E13: limitation: threshold is provisional (not calibrated)
- E14: partial coverage (2 of 3 examined)
- E14: limitation: jvm/F3: growth evidence is from S2, not S7; not subtracted
- E14: limitation: jvm/F19: growth evidence is from S1, not S7; not subtracted
- E14: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E14: limitation: threshold is provisional (not calibrated)
- jvm/E1: partial coverage
- jvm/E1: limitation: no auxiliary classpath: library calls are not modelled
- jvm/E4: partial coverage
- jvm/E4: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- jvm/E4: limitation: writers and removers are textual name matches
- jvm/E5: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- jvm/E5: limitation: workloads were discovered by reading files; none was run
- jvm/E6: limitation: harness written; nothing was run (dla harness run measures it)
- jvm/E6: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E6: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E6: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E7: partial coverage (9 of 9 examined)
- jvm/E7: limitation: command-mode heap dumps come from separate processes: dla summarize heap-graph compares dumps of one process only, so use a driver (dla harness gen --driver) for a dump series
- jvm/E7: limitation: --capture alloc is not written by jvm harnesses
- jvm/E7: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E7: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E7: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E7: limitation: 40516824 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E7: limitation: 32037273 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E7: limitation: 31273349 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E8: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E8: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E8: limitation: 40516824 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 32037273 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 31273349 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E8: limitation: 45768735 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 22894189 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 43472331 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 19929618 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 32004898 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 53820321 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E8: limitation: gate thresholds are provisional (not calibrated)
- jvm/E9: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E9: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E9: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E9: limitation: 40516824 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 32037273 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 31273349 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E9: limitation: 45768735 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 22894189 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 43472331 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 19929618 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 32004898 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 53820321 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E9: limitation: gate thresholds are provisional (not calibrated)
- jvm/E10: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- jvm/E11: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- jvm/E12: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- jvm/E13: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- jvm/E14: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- jvm/E14: limitation: series: per-point; one process per point at n = 2220, 8880, 13320, 19980 (harness captures; checked: HPROF version, identifier size)
- jvm/E15: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- jvm/E16: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- jvm/E16: limitation: the files carry no time, so they are ordered as given
- jvm/E17: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- jvm/E17: limitation: the files carry no time, so they are ordered as given
- jvm/E17: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- jvm/E18: partial coverage (1 of 1 examined)
- jvm/E18: limitation: the stack is the project frame of the largest allocation stack at the site, not every frame
- jvm/E18: limitation: slopes are per unit of the series axis; the thresholds are provisional
- jvm/E19: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E19: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E19: limitation: 40516824 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 32037273 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 31273349 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E19: limitation: 45768735 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 22894189 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 43472331 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 19929618 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 32004898 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 53820321 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E19: limitation: gate thresholds are provisional (not calibrated)
- jvm/E20: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E20: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E20: limitation: 40516824 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 32037273 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 31273349 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E20: limitation: 45768735 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 22894189 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 43472331 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 19929618 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 32004898 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 53820321 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E20: limitation: gate thresholds are provisional (not calibrated)
- jvm/E21: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E21: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E21: limitation: 40516824 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 32037273 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 31273349 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E21: limitation: 45768735 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 22894189 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 43472331 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 19929618 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 32004898 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 53820321 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E21: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E21: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E21: limitation: 29226278 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 20615103 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 21010077 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 29821674 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 31988357 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 29589451 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 27156157 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E21: limitation: 21096996 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E21: limitation: points 1 to 9 come from S1; the extra points from S4
- jvm/E21: limitation: 9 columns beyond the 64 the series allows were dropped
- jvm/E21: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E21: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E21: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E21: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E22: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E22: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E22: limitation: 40516824 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 32037273 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 31273349 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E22: limitation: 45768735 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 22894189 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 43472331 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 19929618 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 32004898 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 53820321 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E22: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E22: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E22: limitation: 25008486 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 20576002 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 33282358 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 21107597 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 19713034 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 21566894 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 32346065 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E22: limitation: 32066421 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E22: limitation: points 1 to 9 come from S1; the extra points from S6
- jvm/E22: limitation: 8 columns beyond the 64 the series allows were dropped
- jvm/E22: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E22: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E22: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E22: limitation: gate and plateau thresholds are provisional (not calibrated)
- rust/E3: partial coverage
- rust/E3: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- rust/E3: limitation: writers and removers are textual name matches
- rust/E4: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- rust/E4: limitation: workloads were discovered by reading files; none was run

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
| E4 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (a1949527a37a) | trend/T5.json |
| E5 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (a1949527a37a) | trend/T5.json |
| E6 | growth-series | dump | dla 0.1.0 | artifact series/S3.json (bf2fa9826714) | trend/T6.json |
| E7 | unit-count-series | dump | dla 0.1.0 | artifact series/S3.json (bf2fa9826714) | trend/T6.json |
| E8 | growth-series | test | dla 0.1.0 | artifact series/S4.json (44d5813ca524) | trend/T7.json |
| E9 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (44d5813ca524) | trend/T7.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S6.json (cd36451ee3b6) | trend/T8.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (cd36451ee3b6) | trend/T8.json |
| E12 | residual | test | dla 0.1.0 | artifact series/S1.json (8ec48e9c187c) | gap/residual-S1-E12.json |
| E13 | residual | test | dla 0.1.0 | artifact series/S5.json (f850a42ef3d1) | gap/residual-S5-E13.json |
| E14 | residual | test | dla 0.1.0 | artifact series/S7.json (3c823a354529) | gap/residual-S7-E14.json |
| jvm/E1 | static-hit | static | spotbugs unknown | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-spotbugs.json |
| jvm/E2 | static-hit | static | detekt 2.0.0-alpha.6 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-detekt.json |
| jvm/E3 | static-hit | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-dla.json |
| jvm/E4 | census-holder | static | dla-census 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | census/jvm.json |
| jvm/E5 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | workload/jvm.json |
| jvm/E6 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | harness/H1/harness.json |
| jvm/E7 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (f53c1cfd9541) | harness/H1/run-S1.json |
| jvm/E8 | growth-series | test | dla 0.1.0 | artifact series/S1.json (8ec48e9c187c) | trend/T1.json |
| jvm/E9 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (8ec48e9c187c) | trend/T1.json |
| jvm/E10 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-180923-a3f2/harness/H1/out/p1-native/capture/9/heap-1.hprof (4ae97b74cd73) | summaries/heap-1-4ae97b74.json |
| jvm/E11 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-180923-a3f2/harness/H1/out/p4-native/capture/388/heap-1.hprof (bdea182ddeae) | summaries/heap-1-bdea182d.json |
| jvm/E12 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-180923-a3f2/harness/H1/out/p6-native/capture/619/heap-1.hprof (6847ca1adbe7) | summaries/heap-1-6847ca1a.json |
| jvm/E13 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-180923-a3f2/harness/H1/out/p9-native/capture/961/heap-1.hprof (88bc4b97870d) | summaries/heap-1-88bc4b97.json |
| jvm/E14 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S2.json (a1949527a37a) | series/S2.json |
| jvm/E15 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S2.json (a1949527a37a) | series/S2.json |
| jvm/E16 | histogram-diff | live | jcmd-class-histogram 0.1.0 | artifact histograms/H1.json (8cb8f746c499) | histograms/H1.json |
| jvm/E17 | growth-series | live | jcmd-class-histogram 0.1.0 | artifact series/S3.json (bf2fa9826714) | series/S3.json |
| jvm/E18 | alloc-stack | test | dla-correlate 0.1.0 | artifact series/S1.json (8ec48e9c187c) | correlate/C1.json |
| jvm/E19 | growth-series | test | dla 0.1.0 | artifact series/S1.json (8ec48e9c187c) | trend/T2.json |
| jvm/E20 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (8ec48e9c187c) | trend/T2.json |
| jvm/E21 | discriminator | test | dla 0.1.0 | artifact series/S5.json (f850a42ef3d1) | alloc/A1.json |
| jvm/E22 | discriminator | test | dla 0.1.0 | artifact series/S7.json (3c823a354529) | alloc/A2.json |
| rust/E1 | static-hit | static | rapx 0.7.71 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/rust-rapx.json |
| rust/E2 | static-hit | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/rust-dla.json |
| rust/E3 | census-holder | static | dla-census 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | census/rust.json |
| rust/E4 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | workload/rust.json |
