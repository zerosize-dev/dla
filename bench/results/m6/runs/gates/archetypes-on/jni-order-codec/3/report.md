# DLA report

Run: `20261110-172436-39b7`

Status: INCOMPLETE (33 open)

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 24 enumerated, 24 dispositioned; confirmed 2, strong 0, refuted 2, suspected 19, unresolved 1

Residual: heap 1451.3315315315317 bytes/n unattributed of 1451.3315315315317 total (threshold 362.8328828828829) ABOVE THRESHOLD, see O2; heap 1443.1503003003004 bytes/n unattributed of 1443.1503003003004 total (threshold 360.7875750750751) ABOVE THRESHOLD, see O2; heap 1441.3719037719038 bytes/n unattributed of 1441.3719037719038 total (threshold 360.3429759429759) ABOVE THRESHOLD, see O2; off-heap 2388.7073359073356 bytes/n unattributed of 2388.7073359073356 total (threshold 1636.6412404633204) ABOVE THRESHOLD, see O3; non-heap 0 bytes/n unattributed of -3.407387387387387 total (threshold 47.57440151351362); non-heap 0 bytes/n unattributed of -5.3760360360360355 total (threshold 10.023417827027002); non-heap 0 bytes/n unattributed of -6.815855855855856 total (threshold 13.117831091891793)

Coverage: jvm 17760 iterations, 0 of 17 entry points exercised; rust no iterations, 0 of 10 entry points exercised

## Open items

- **T5:os.rss**: column os.rss of T5 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.rss_anon**: column os.rss_anon of T5 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.vm_hwm**: column os.vm_hwm of T5 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:jvm.heap_used**: column jvm.heap_used of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:native.live_blocks**: column native.live_blocks of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:native.live_bytes**: column native.live_bytes of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:native.peak_bytes**: column native.peak_bytes of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:jvm.class:byte[].count**: column jvm.class:byte[].count of T5 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E5 # column jvm.class:byte[].count`
- **T5:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T5 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E5 # column jvm.class:java.lang.String.count`
- **T5:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T5 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E5 # column jvm.class:shop.codec.Entry.count`
- **T6:jvm.heap_used**: column jvm.heap_used of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:native.live_blocks**: column native.live_blocks of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:native.live_bytes**: column native.live_bytes of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:native.peak_bytes**: column native.peak_bytes of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:jvm.class:byte[].count**: column jvm.class:byte[].count of T6 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E7 # column jvm.class:byte[].count`
- **T6:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T6 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E7 # column jvm.class:java.lang.String.count`
- **T6:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T6 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E7 # column jvm.class:shop.codec.Entry.count`
- **T8:type:byte[]**: column type:byte[] of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E10 # column type:byte[]`
- **T8:type:byte[].count**: column type:byte[].count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E11 # column type:byte[].count`
- **T8:type:java.lang.String**: column type:java.lang.String of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E10 # column type:java.lang.String`
- **T8:type:java.lang.String.count**: column type:java.lang.String.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E11 # column type:java.lang.String.count`
- **T8:type:jdk.internal.vm.FillerElement[]**: column type:jdk.internal.vm.FillerElement[] of T8 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E10 # column type:jdk.internal.vm.FillerElement[]`
- **T8:type:java.lang.Object[]**: column type:java.lang.Object[] of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E10 # column type:java.lang.Object[]`
- **T8:type:jdk.jfr.internal.periodic.Batch**: column type:jdk.jfr.internal.periodic.Batch of T8 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E10 # column type:jdk.jfr.internal.periodic.Batch`
- **T8:type:jdk.jfr.internal.periodic.Batch.count**: column type:jdk.jfr.internal.periodic.Batch.count of T8 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E11 # column type:jdk.jfr.internal.periodic.Batch.count`
- **T8:type:java.lang.Long**: column type:java.lang.Long of T8 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E10 # column type:java.lang.Long`
- **T8:type:java.lang.Long.count**: column type:java.lang.Long.count of T8 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E11 # column type:java.lang.Long.count`
- **T10:jvm.class:byte[].count**: column jvm.class:byte[].count of T10 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E17 # column jvm.class:byte[].count`
- **T10:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T10 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E17 # column jvm.class:java.lang.String.count`
- **T10:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T10 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E17 # column jvm.class:shop.codec.Entry.count`
- **T11:jvm.class:byte[].count**: column jvm.class:byte[].count of T11 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E19 # column jvm.class:byte[].count`
- **T11:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T11 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E19 # column jvm.class:java.lang.String.count`
- **T11:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T11 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E19 # column jvm.class:shop.codec.Entry.count`

## Workers

- **jvm** (jvm): status done; 78 lines merged; slice done; torn tail: no
- **rust** (rust): status done; 12 lines merged; slice done; torn tail: no

## Findings

### jvm/F3: **L-COLLECT**

- State: confirmed
- Label: L-COLLECT (evidence: jvm/E8 growth-series, jvm/E15 retention-path)
- Growth: `probe:shop.codec.Audit.trail` 2.625 count/n (steady, gate pass, jvm/E12)
- Evidence classes: context, discriminator, growth, static
- Holder: static/global
- Owner: `shop.codec.Audit.trail`
- Region: heap
- Shape: steady
- Site: `kotlin/shop/Audit.kt:9`
- Writers: `kotlin/shop/Audit.kt:12`
- Annotation: heuristic name match over the project's sources
- Annotation: Audit.record appends every inline frame sample; no remover exists
- Verified: no
- Sibling hypotheses: jvm/F3.n1
- Supporting evidence: jvm/E4, jvm/E7, jvm/E12, jvm/E13
- Explanation (observed): Audit.trail (static holder in object Audit, Audit.kt:9) gets an Entry with a 1536-byte sample added by Audit.record at Audit.kt:12 for every inline frame; entries are never removed, so the list grows with each order.
- Explanation structure: adds: kotlin/shop/Audit.kt:12; removal: never; evidence: jvm/E15

### jvm/F19: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (native handle from Native.open for oversized frames never closed in OrderEncoder.encode bulk branch; native allocation leak, no alloc-stack evidence in this worker)
- Growth: `native.live_blocks@native/src/lib.rs:40` 0.125 count/n (steady, gate pass, jvm/E12)
- Evidence classes: allocation, context, discriminator, growth
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.open`
- Region: off-heap
- Shape: steady
- Site: `native/src/lib.rs:40`
- Writers: `kotlin/shop/Native.kt:9`, `kotlin/shop/Encoder.kt:11`
- Annotation: native allocation under shop.codec.Native.open (Java_shop_codec_Native_open); found by dla correlate
- Annotation: bulk-lane branch returns without Native.close(handle), so handle from Native.open is never released
- Verified: no
- Sibling hypotheses: jvm/F19.n1
- Supporting evidence: jvm/E10, jvm/E7, jvm/E12, jvm/E14
- Explanation (inferred): Native.open (native handle, called at Encoder.kt:11 and allocating at native/src/lib.rs:40) returns a handle per frame; the bulk-lane branch in OrderEncoder.encode returns early and a remover was not found: Native.close is never called for large wholesale frames.
- Explanation structure: adds: kotlin/shop/Encoder.kt:11; removal: not-found

### jvm/F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Site: `kotlin/shop/Audit.kt:9`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: duplicate of confirmed jvm/F3 (same site Audit.kt:9); no separate action
- Supporting evidence: jvm/E3

### jvm/F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.Entry.sample`
- Region: heap
- Site: `kotlin/shop/Audit.kt:3`
- Reason: not_exercised
- Remedy: Entry.sample is the payload of entries held by confirmed jvm/F3; fixing F3 releases it
- Supporting evidence: jvm/E4

### jvm/F4: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.OrderEncoder.Companion`
- Region: heap
- Site: `kotlin/shop/Encoder.kt:6`
- Reason: not_exercised
- Remedy: census lead only: static constant, companion, JNI binding declaration or fixed-size field; no growth column tied to it in harness H1
- Supporting evidence: jvm/E4

### jvm/F5: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.FramePacker.MAGIC`
- Region: heap
- Site: `kotlin/shop/FramePacker.kt:9`
- Reason: not_exercised
- Remedy: census lead only: static constant, companion, JNI binding declaration or fixed-size field; no growth column tied to it in harness H1
- Supporting evidence: jvm/E4

### jvm/F6: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.Native.INSTANCE`
- Region: heap
- Site: `kotlin/shop/Native.kt:5`
- Reason: not_exercised
- Remedy: census lead only: static constant, companion, JNI binding declaration or fixed-size field; no growth column tied to it in harness H1
- Supporting evidence: jvm/E4

### jvm/F7: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.<clinit>`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:6`
- Reason: not_exercised
- Remedy: census lead only: static constant, companion, JNI binding declaration or fixed-size field; no growth column tied to it in harness H1
- Supporting evidence: jvm/E4

### jvm/F8: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.open`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:9`
- Reason: not_exercised
- Remedy: census lead only: static constant, companion, JNI binding declaration or fixed-size field; no growth column tied to it in harness H1
- Supporting evidence: jvm/E4

### jvm/F9: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.checksum`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:10`
- Reason: not_exercised
- Remedy: census lead only: static constant, companion, JNI binding declaration or fixed-size field; no growth column tied to it in harness H1
- Supporting evidence: jvm/E4

### jvm/F10: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.size`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:11`
- Reason: not_exercised
- Remedy: census lead only: static constant, companion, JNI binding declaration or fixed-size field; no growth column tied to it in harness H1
- Supporting evidence: jvm/E4

### jvm/F11: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.lines`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:12`
- Reason: not_exercised
- Remedy: census lead only: static constant, companion, JNI binding declaration or fixed-size field; no growth column tied to it in harness H1
- Supporting evidence: jvm/E4

### jvm/F12: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.close`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:13`
- Reason: not_exercised
- Remedy: census lead only: static constant, companion, JNI binding declaration or fixed-size field; no growth column tied to it in harness H1
- Supporting evidence: jvm/E4

### jvm/F13: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.live`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:14`
- Reason: not_exercised
- Remedy: census lead only: static constant, companion, JNI binding declaration or fixed-size field; no growth column tied to it in harness H1
- Supporting evidence: jvm/E4

### jvm/F14: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.liveBytes`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:15`
- Reason: not_exercised
- Remedy: census lead only: static constant, companion, JNI binding declaration or fixed-size field; no growth column tied to it in harness H1
- Supporting evidence: jvm/E4

### jvm/F15: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.OrderGenerator.Companion`
- Region: heap
- Site: `kotlin/shop/Order.kt:12`
- Reason: not_exercised
- Remedy: census lead only: static constant, companion, JNI binding declaration or fixed-size field; no growth column tied to it in harness H1
- Supporting evidence: jvm/E4

### jvm/F16: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.OrderGenerator.skus`
- Region: heap
- Site: `kotlin/shop/Order.kt:14`
- Reason: not_exercised
- Remedy: census lead only: static constant, companion, JNI binding declaration or fixed-size field; no growth column tied to it in harness H1
- Supporting evidence: jvm/E4

### jvm/F17: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.Order.lines`
- Region: heap
- Site: `kotlin/shop/Order.kt:18`
- Reason: not_exercised
- Remedy: census lead only: static constant, companion, JNI binding declaration or fixed-size field; no growth column tied to it in harness H1
- Supporting evidence: jvm/E4

### jvm/F18: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.Stats.Companion`
- Region: heap
- Site: `kotlin/shop/Stats.kt:6`
- Reason: not_exercised
- Remedy: census lead only: static constant, companion, JNI binding declaration or fixed-size field; no growth column tied to it in harness H1
- Supporting evidence: jvm/E4

### rust/F1: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `native.src.frame.Frame.bytes`
- Region: heap
- Site: `native/src/frame.rs:11`
- Writers: `native/src/lib.rs:48`
- Annotation: Frame.bytes owned by FRAMES entries
- Reason: no_harness
- Remedy: Frame.bytes is owned by FRAMES entries; same lifetime as F2 (rust/F2). Confirm via JVM JNI harness plus dla correlate.
- Supporting evidence: rust/E3

### rust/F2: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: static/global
- Owner: `native.src.lib.FRAMES`
- Region: heap
- Site: `native/src/lib.rs:19`
- Writers: `native/src/lib.rs:48`
- Removers: `native/src/lib.rs:73`
- Annotation: FRAMES map: open inserts, close removes; Kotlin encode skips close on bulk path
- Reason: no_harness
- Remedy: Native.open inserts into FRAMES (native/src/lib.rs:48); Kotlin OrderEncoder.encode skips Native.close for frames over 16384 bytes. Rust crate is a cdylib with no bin or lib driver; confirm from the JVM worker's JNI harness then run dla correlate H1, tying native growth to this site.
- Supporting evidence: rust/E3

### jvm/F20: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Region: heap
- Shape: steady
- Reason: not_exercised
- Remedy: Entry count column is the same growth as confirmed jvm/F3 (Audit.trail holds the Entry objects); no separate holder to test
- Supporting evidence: jvm/E12

## Ruled out

- **jvm/F3.n1** (UNCLASSIFIED): discriminator jvm/E13
  - Ruled out by: plateau+purge (leak) jvm/E13
  - Hypothesis: N-WARM: audit trail fills to a cap
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: jvm/E13
  - Originally supported by: jvm/E12
- **jvm/F19.n1** (UNCLASSIFIED): discriminator jvm/E14
  - Ruled out by: plateau+purge (leak) jvm/E14
  - Hypothesis: N-ALLOC: native allocator retention, not live handles
  - Refuted if: live_blocks keeps growing after purge/extra cycles
  - Contradicting evidence: jvm/E14
  - Originally supported by: jvm/E12

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

- E4: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E4: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E4: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E4: limitation: 30192833 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E4: limitation: 30488173 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E4: limitation: 35762512 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E4: limitation: 22874470 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E4: limitation: 33655347 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E4: limitation: 24321188 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E4: limitation: 31232242 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E4: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E4: limitation: 31657097 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E4: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E4: limitation: gate thresholds are provisional (not calibrated)
- E5: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E5: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E5: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E5: limitation: 30192833 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E5: limitation: 30488173 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E5: limitation: 35762512 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E5: limitation: 22874470 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E5: limitation: 33655347 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E5: limitation: 24321188 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E5: limitation: 31232242 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E5: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E5: limitation: 31657097 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E5: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E5: limitation: gate thresholds are provisional (not calibrated)
- E6: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E6: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E6: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E6: limitation: 21123956 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: 21487180 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: 23549798 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: 36085299 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: 22286728 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: 32115969 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: 31712333 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E6: limitation: 30663954 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E6: limitation: gate thresholds are provisional (not calibrated)
- E7: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E7: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E7: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E7: limitation: 21123956 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: 21487180 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: 23549798 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: 36085299 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: 22286728 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: 32115969 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: 31712333 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E7: limitation: 30663954 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E7: limitation: gate thresholds are provisional (not calibrated)
- E8: limitation: series: per-point; x is the iteration count n of each point (2220, 8880, 13320, 19980); every point is a separate process and heap dump
- E8: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E8: limitation: holders absent from a dump's 200 largest have no figure at that point
- E8: limitation: gate thresholds are provisional (not calibrated)
- E9: limitation: series: per-point; x is the iteration count n of each point (2220, 8880, 13320, 19980); every point is a separate process and heap dump
- E9: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E9: limitation: holders absent from a dump's 200 largest have no figure at that point
- E9: limitation: gate thresholds are provisional (not calibrated)
- E10: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E10: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E11: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: partial coverage (2 of 3 examined)
- E12: limitation: jvm/F3: growth measured as a count; bytes not attributed
- E12: limitation: jvm/F19: growth measured as a count; bytes not attributed
- E12: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E12: limitation: threshold is provisional (not calibrated)
- E13: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E13: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- E13: limitation: 44055830 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: 32976350 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: 22809397 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: 52012378 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: 29459135 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: 40342024 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: 22148918 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: 37472775 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: 44179337 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- E13: limitation: gap.unexplained is negative at 2 point(s): the accounted figure exceeds os.rss (reserved but untouched memory)
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: partial coverage (2 of 3 examined)
- E14: limitation: jvm/F3: growth evidence is from S1, not S3; not subtracted
- E14: limitation: jvm/F19: growth evidence is from S1, not S3; not subtracted
- E14: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E14: limitation: threshold is provisional (not calibrated)
- E15: partial coverage (2 of 3 examined)
- E15: limitation: jvm/F3: growth evidence is from S1, not S5; not subtracted
- E15: limitation: jvm/F19: growth evidence is from S1, not S5; not subtracted
- E15: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E15: limitation: threshold is provisional (not calibrated)
- E16: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E16: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- E16: limitation: 44055830 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 32976350 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 22809397 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 52012378 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 29459135 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 40342024 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 22148918 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 37472775 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 44179337 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- E16: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E16: limitation: 30192833 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 30488173 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 35762512 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 22874470 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 33655347 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 24321188 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 31232242 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E16: limitation: 31657097 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E16: limitation: points 1 to 9 come from S1; the extra points from S2
- E16: limitation: 9 columns beyond the 64 the series allows were dropped
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- E17: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E17: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E17: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- E17: limitation: 44055830 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 32976350 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 22809397 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 52012378 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 29459135 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 40342024 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 22148918 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 37472775 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 44179337 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- E17: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E17: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E17: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E17: limitation: 30192833 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 30488173 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 35762512 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 22874470 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 33655347 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 24321188 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 31232242 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E17: limitation: 31657097 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E17: limitation: points 1 to 9 come from S1; the extra points from S2
- E17: limitation: 9 columns beyond the 64 the series allows were dropped
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- E18: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E18: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E18: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- E18: limitation: 44055830 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 32976350 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 22809397 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 52012378 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 29459135 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 40342024 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 22148918 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 37472775 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 44179337 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- E18: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E18: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E18: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E18: limitation: 21123956 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 21487180 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 23549798 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 36085299 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 22286728 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 32115969 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 31712333 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E18: limitation: 30663954 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E18: limitation: points 1 to 9 come from S1; the extra points from S4
- E18: limitation: 9 columns beyond the 64 the series allows were dropped
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E19: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E19: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- E19: limitation: 44055830 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 32976350 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 22809397 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 52012378 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 29459135 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 40342024 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 22148918 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 37472775 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 44179337 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- E19: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E19: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E19: limitation: 21123956 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 21487180 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 23549798 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 36085299 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 22286728 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 32115969 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 31712333 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E19: limitation: 30663954 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E19: limitation: points 1 to 9 come from S1; the extra points from S4
- E19: limitation: 9 columns beyond the 64 the series allows were dropped
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: partial coverage (3 of 3 examined)
- E20: limitation: jvm/F3: growth measured as a count; bytes not attributed
- E20: limitation: jvm/F19: growth measured as a count; bytes not attributed
- E20: limitation: threshold is provisional (not calibrated)
- E21: partial coverage (2 of 3 examined)
- E21: limitation: jvm/F3: growth evidence is from S1, not S3; not subtracted
- E21: limitation: jvm/F19: growth evidence is from S1, not S3; not subtracted
- E21: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E21: limitation: threshold is provisional (not calibrated)
- E22: partial coverage (2 of 3 examined)
- E22: limitation: jvm/F3: growth evidence is from S1, not S5; not subtracted
- E22: limitation: jvm/F19: growth evidence is from S1, not S5; not subtracted
- E22: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E22: limitation: threshold is provisional (not calibrated)
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
- jvm/E7: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E7: limitation: 44055830 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E7: limitation: 32976350 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E8: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E8: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E8: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E8: limitation: 44055830 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 32976350 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 22809397 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 52012378 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 29459135 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 40342024 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 22148918 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 37472775 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 44179337 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E8: limitation: gate thresholds are provisional (not calibrated)
- jvm/E9: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E9: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E9: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E9: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E9: limitation: 44055830 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 32976350 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 22809397 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 52012378 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 29459135 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 40342024 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 22148918 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 37472775 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 44179337 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E9: limitation: gate thresholds are provisional (not calibrated)
- jvm/E10: partial coverage (1 of 1 examined)
- jvm/E10: limitation: the stack is the project frame of the largest allocation stack at the site, not every frame
- jvm/E10: limitation: slopes are per unit of the series axis; the thresholds are provisional
- jvm/E11: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E11: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E11: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E11: limitation: 44055830 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 32976350 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 22809397 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 52012378 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 29459135 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 40342024 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 22148918 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 37472775 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 44179337 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E11: limitation: gate thresholds are provisional (not calibrated)
- jvm/E12: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E12: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E12: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E12: limitation: 44055830 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 32976350 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 22809397 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 52012378 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 29459135 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 40342024 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 22148918 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 37472775 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 44179337 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E12: limitation: gate thresholds are provisional (not calibrated)
- jvm/E13: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E13: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E13: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E13: limitation: 44055830 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 32976350 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 22809397 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 52012378 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 29459135 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 40342024 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 22148918 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 37472775 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 44179337 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E13: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E13: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E13: limitation: 30192833 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 30488173 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 35762512 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 22874470 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 33655347 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 24321188 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 31232242 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E13: limitation: 31657097 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E13: limitation: points 1 to 9 come from S1; the extra points from S2
- jvm/E13: limitation: 9 columns beyond the 64 the series allows were dropped
- jvm/E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E13: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E13: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E14: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E14: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E14: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E14: limitation: 44055830 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 32976350 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 22809397 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 52012378 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 29459135 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 40342024 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 22148918 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 37472775 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 44179337 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E14: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E14: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E14: limitation: 21123956 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 21487180 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 23549798 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 36085299 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 22286728 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 32115969 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 31712333 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E14: limitation: 30663954 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E14: limitation: points 1 to 9 come from S1; the extra points from S4
- jvm/E14: limitation: 9 columns beyond the 64 the series allows were dropped
- jvm/E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E14: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E14: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E15: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- jvm/E16: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- jvm/E17: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- jvm/E18: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- jvm/E19: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- jvm/E19: limitation: series: per-point; one process per point at n = 2220, 8880, 13320, 19980 (harness captures; checked: HPROF version, identifier size)
- jvm/E20: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- jvm/E21: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- jvm/E21: limitation: the files carry no time, so they are ordered as given
- jvm/E22: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- jvm/E22: limitation: the files carry no time, so they are ordered as given
- jvm/E22: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
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
| E4 | growth-series | test | dla 0.1.0 | artifact series/S2.json (f434867f42ca) | trend/T5.json |
| E5 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (f434867f42ca) | trend/T5.json |
| E6 | growth-series | test | dla 0.1.0 | artifact series/S4.json (a9c144430454) | trend/T6.json |
| E7 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (a9c144430454) | trend/T6.json |
| E8 | growth-series | dump | dla 0.1.0 | artifact series/S6.json (66e296076460) | trend/T7.json |
| E9 | unit-count-series | dump | dla 0.1.0 | artifact series/S6.json (66e296076460) | trend/T7.json |
| E10 | growth-series | dump | dla 0.1.0 | artifact series/S7.json (604b5df72786) | trend/T8.json |
| E11 | unit-count-series | dump | dla 0.1.0 | artifact series/S7.json (604b5df72786) | trend/T8.json |
| E12 | residual | test | dla 0.1.0 | artifact series/S1.json (93f4ddf299ae) | gap/residual-S1-E12.json |
| E13 | non-heap | test | dla 0.1.0 | artifact series/S8.json (9def3f58f414) | trend/T9.json |
| E14 | residual | test | dla 0.1.0 | artifact series/S3.json (c962bfbe1462) | gap/residual-S3-E14.json |
| E15 | residual | test | dla 0.1.0 | artifact series/S5.json (c19ab564689f) | gap/residual-S5-E15.json |
| E16 | growth-series | test | dla 0.1.0 | artifact series/S3.json (c962bfbe1462) | trend/T10.json |
| E17 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (c962bfbe1462) | trend/T10.json |
| E18 | growth-series | test | dla 0.1.0 | artifact series/S5.json (c19ab564689f) | trend/T11.json |
| E19 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (c19ab564689f) | trend/T11.json |
| E20 | residual | test | dla 0.1.0 | artifact series/S1.json (93f4ddf299ae) | gap/residual-S1-E20.json |
| E21 | residual | test | dla 0.1.0 | artifact series/S3.json (c962bfbe1462) | gap/residual-S3-E21.json |
| E22 | residual | test | dla 0.1.0 | artifact series/S5.json (c19ab564689f) | gap/residual-S5-E22.json |
| jvm/E1 | static-hit | static | spotbugs unknown | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-spotbugs.json |
| jvm/E2 | static-hit | static | detekt 2.0.0-alpha.6 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-detekt.json |
| jvm/E3 | static-hit | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-dla.json |
| jvm/E4 | census-holder | static | dla-census 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | census/jvm.json |
| jvm/E5 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | workload/jvm.json |
| jvm/E6 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | harness/H1/harness.json |
| jvm/E7 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (a8e940f79ec2) | harness/H1/run-S1.json |
| jvm/E8 | growth-series | test | dla 0.1.0 | artifact series/S1.json (93f4ddf299ae) | trend/T1.json |
| jvm/E9 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (93f4ddf299ae) | trend/T1.json |
| jvm/E10 | alloc-stack | test | dla-correlate 0.1.0 | artifact series/S1.json (93f4ddf299ae) | correlate/C1.json |
| jvm/E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (93f4ddf299ae) | trend/T2.json |
| jvm/E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (93f4ddf299ae) | trend/T2.json |
| jvm/E13 | discriminator | test | dla 0.1.0 | artifact series/S3.json (c962bfbe1462) | alloc/A1.json |
| jvm/E14 | discriminator | test | dla 0.1.0 | artifact series/S5.json (c19ab564689f) | alloc/A2.json |
| jvm/E15 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-172436-39b7/harness/H1/out/p1-native/capture/9/heap-1.hprof (13c658963373) | summaries/heap-1-13c65896.json |
| jvm/E16 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-172436-39b7/harness/H1/out/p4-native/capture/393/heap-1.hprof (d3fc181c6001) | summaries/heap-1-d3fc181c.json |
| jvm/E17 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-172436-39b7/harness/H1/out/p6-native/capture/620/heap-1.hprof (20665d5cfd5b) | summaries/heap-1-20665d5c.json |
| jvm/E18 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-172436-39b7/harness/H1/out/p9-native/capture/964/heap-1.hprof (46e566ab080b) | summaries/heap-1-46e566ab.json |
| jvm/E19 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S6.json (66e296076460) | series/S6.json |
| jvm/E20 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S6.json (66e296076460) | series/S6.json |
| jvm/E21 | histogram-diff | live | jcmd-class-histogram 0.1.0 | artifact histograms/H1.json (b0c4f8a1c91d) | histograms/H1.json |
| jvm/E22 | growth-series | live | jcmd-class-histogram 0.1.0 | artifact series/S7.json (604b5df72786) | series/S7.json |
| rust/E1 | static-hit | static | rapx 0.7.71 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/rust-rapx.json |
| rust/E2 | static-hit | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/rust-dla.json |
| rust/E3 | census-holder | static | dla-census 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | census/rust.json |
| rust/E4 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | workload/rust.json |
