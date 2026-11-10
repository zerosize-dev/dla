# DLA report

Run: `20261110-192004-7ac4`

Status: INCOMPLETE (48 open)

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 23 enumerated, 23 dispositioned; confirmed 1, strong 1, refuted 2, suspected 19, unresolved 0

Residual: heap 1452.6507507507508 bytes/n unattributed of 1452.6507507507508 total (threshold 363.1626876876877) ABOVE THRESHOLD, see O1; non-heap 0.4276276276276276 bytes/n unattributed of 0.4276276276276276 total (threshold 47.37756108108117)

Coverage: jvm 17760 iterations, 0 of 17 entry points exercised; rust no iterations, 0 of 10 entry points exercised

## Open items

- **T4:os.rss**: column os.rss of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:os.rss_anon**: column os.rss_anon of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:jvm.heap_used**: column jvm.heap_used of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:native.live_blocks**: column native.live_blocks of T4 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:native.live_bytes**: column native.live_bytes of T4 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:native.peak_bytes**: column native.peak_bytes of T4 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:jvm.heap_used.pre_gc**: column jvm.heap_used.pre_gc of T4 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:jvm.class:byte[].count**: column jvm.class:byte[].count of T4 (steady) is not flat and is tied to no finding. Fix: `dla trend S7`
- **T4:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T4 (steady) is not flat and is tied to no finding. Fix: `dla trend S7`
- **T4:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T4 (steady) is not flat and is tied to no finding. Fix: `dla trend S7`
- **T8:jvm.class:byte[].count**: column jvm.class:byte[].count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E11 # column jvm.class:byte[].count`
- **T8:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E11 # column jvm.class:java.lang.String.count`
- **T8:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E11 # column jvm.class:shop.codec.Entry.count`
- **T10:type:byte[]**: column type:byte[] of T10 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E14 # column type:byte[]`
- **T10:type:byte[].count**: column type:byte[].count of T10 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E15 # column type:byte[].count`
- **T10:type:java.lang.String**: column type:java.lang.String of T10 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E14 # column type:java.lang.String`
- **T10:type:java.lang.String.count**: column type:java.lang.String.count of T10 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E15 # column type:java.lang.String.count`
- **T10:type:jdk.internal.vm.FillerElement[]**: column type:jdk.internal.vm.FillerElement[] of T10 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E14 # column type:jdk.internal.vm.FillerElement[]`
- **T11:os.rss**: column os.rss of T11 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T11:os.rss_anon**: column os.rss_anon of T11 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T11:os.vm_hwm**: column os.vm_hwm of T11 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T11:jvm.heap_committed**: column jvm.heap_committed of T11 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T11:jvm.heap_used**: column jvm.heap_used of T11 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T11:native.live_blocks**: column native.live_blocks of T11 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T11:native.live_bytes**: column native.live_bytes of T11 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T11:native.peak_bytes**: column native.peak_bytes of T11 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T11:jvm.class:byte[].count**: column jvm.class:byte[].count of T11 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E17 # column jvm.class:byte[].count`
- **T11:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T11 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E17 # column jvm.class:java.lang.String.count`
- **T11:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T11 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E17 # column jvm.class:shop.codec.Entry.count`
- **T12:jvm.heap_used**: column jvm.heap_used of T12 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T12:native.live_blocks**: column native.live_blocks of T12 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T12:native.live_bytes**: column native.live_bytes of T12 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T12:native.peak_bytes**: column native.peak_bytes of T12 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T12:jvm.heap_committed.pre_gc**: column jvm.heap_committed.pre_gc of T12 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T12:jvm.class:byte[].count**: column jvm.class:byte[].count of T12 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E19 # column jvm.class:byte[].count`
- **T12:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T12 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E19 # column jvm.class:java.lang.String.count`
- **T12:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T12 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E19 # column jvm.class:shop.codec.Entry.count`
- **T13:os.rss**: column os.rss of T13 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T13:os.rss_anon**: column os.rss_anon of T13 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T13:os.rss_file**: column os.rss_file of T13 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T13:os.vm_data**: column os.vm_data of T13 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T13:os.vm_hwm**: column os.vm_hwm of T13 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T13:jvm.heap_committed**: column jvm.heap_committed of T13 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T13:jvm.heap_used**: column jvm.heap_used of T13 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T13:jvm.heap_used.pre_gc**: column jvm.heap_used.pre_gc of T13 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T13:jvm.class:byte[].count**: column jvm.class:byte[].count of T13 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E21 # column jvm.class:byte[].count`
- **T13:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T13 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E21 # column jvm.class:java.lang.String.count`
- **T13:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T13 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E21 # column jvm.class:shop.codec.Entry.count`

## Workers

- **jvm** (jvm): status done; 78 lines merged; slice done; torn tail: no
- **rust** (rust): status done; 12 lines merged; slice done; torn tail: no

## Findings

### jvm/F3: **UNCLASSIFIED**

- State: strong
- Label: UNCLASSIFIED (Static Audit.trail ArrayList in shop.codec.Audit accumulates a 1536-byte Entry sample per inline order via OrderEncoder.encode; no removal, unbounded managed heap growth)
- Growth: `probe:shop.codec.Audit.trail` 2.625 count/n (steady, gate pass, jvm/E12)
- Evidence classes: context, discriminator, growth, retention, static
- Strong: second class jvm/E16 retention-path that names the holder, independent of the growth record jvm/E12 (dla)
- Holder: static/global
- Owner: `shop.codec.Audit.trail`
- Region: heap
- Shape: steady
- Site: `kotlin/shop/Audit.kt:9`
- Writers: `kotlin/shop/Audit.kt:12`, `kotlin/shop/Encoder.kt:16`, `kotlin/shop/Encoder.kt:17`
- Annotation: heuristic name match over the project's sources
- Annotation: OrderEncoder.encode appends a 1536-byte sample per inline order to Audit.trail; nothing ever removes entries
- Verified: no
- Sibling hypotheses: jvm/F3.n1
- Supporting evidence: jvm/E4, jvm/E7, jvm/E12, jvm/E17, jvm/E21, jvm/E16
- Explanation (inferred): Static holder Audit.trail (kotlin/shop/Audit.kt:9) is an ArrayList that Audit.record adds to at Audit.kt:12 for every inline order; entries are never removed, so the heap grows.
- Explanation structure: adds: kotlin/shop/Audit.kt:12; removal: never

### jvm/F19: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Native handle from Native.open (lib.rs:40) is never closed on the bulk path in OrderEncoder.encode (size > 16384 returns before Native.close); native memory leaks per wholesale order)
- Growth: `native.live_blocks@native/src/lib.rs:40` 0.125 count/n (steady, gate pass, jvm/E12)
- Evidence classes: allocation, context, discriminator, growth
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.open`
- Region: off-heap
- Shape: steady
- Site: `native/src/lib.rs:40`
- Writers: `kotlin/shop/Native.kt:9`, `kotlin/shop/Encoder.kt:9`
- Removers: `kotlin/shop/Encoder.kt:18`
- Annotation: native allocation under shop.codec.Native.open (Java_shop_codec_Native_open); found by dla correlate
- Annotation: Native.open handle is closed only on the inline path; the bulk path (size > 16384) returns at Encoder.kt:15 without Native.close
- Verified: no
- Sibling hypotheses: jvm/F19.n1
- Supporting evidence: jvm/E10, jvm/E7, jvm/E12, jvm/E22
- Explanation (inferred): Native.open at Encoder.kt:9 allocates a native-handle (lib.rs:40) that is closed at Encoder.kt:18 only when the frame is inline; the bulk path returns at line 15 without close, so native memory leaks.
- Explanation structure: adds: kotlin/shop/Encoder.kt:9; removal: conditional at kotlin/shop/Encoder.kt:18

### jvm/F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Site: `kotlin/shop/Audit.kt:9`
- Reason: not_exercised
- Remedy: Same retention as confirmed jvm/F3 (Audit.trail / Entry.sample); fixing F3 covers it, no separate growth column tied
- Supporting evidence: jvm/E3

### jvm/F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.Entry.sample`
- Region: heap
- Site: `kotlin/shop/Audit.kt:3`
- Reason: not_exercised
- Remedy: Same retention as confirmed jvm/F3 (Audit.trail / Entry.sample); fixing F3 covers it, no separate growth column tied
- Supporting evidence: jvm/E4

### jvm/F4: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.OrderEncoder.Companion`
- Region: heap
- Site: `kotlin/shop/Encoder.kt:6`
- Reason: not_exercised
- Remedy: Constant, singleton or fixed-size holder with no growth in S1 and no adding site; confirm with a longer run only if its size is expected to change
- Supporting evidence: jvm/E4

### jvm/F5: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.FramePacker.MAGIC`
- Region: heap
- Site: `kotlin/shop/FramePacker.kt:9`
- Reason: not_exercised
- Remedy: Constant, singleton or fixed-size holder with no growth in S1 and no adding site; confirm with a longer run only if its size is expected to change
- Supporting evidence: jvm/E4

### jvm/F6: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.Native.INSTANCE`
- Region: heap
- Site: `kotlin/shop/Native.kt:5`
- Reason: not_exercised
- Remedy: Constant, singleton or fixed-size holder with no growth in S1 and no adding site; confirm with a longer run only if its size is expected to change
- Supporting evidence: jvm/E4

### jvm/F7: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.<clinit>`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:6`
- Reason: not_exercised
- Remedy: JNI method declaration, not a holder; native growth is attributed to jvm/F19 (Native.open handle left open on bulk path)
- Supporting evidence: jvm/E4

### jvm/F8: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.open`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:9`
- Reason: not_exercised
- Remedy: JNI method declaration, not a holder; native growth is attributed to jvm/F19 (Native.open handle left open on bulk path)
- Supporting evidence: jvm/E4

### jvm/F9: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.checksum`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:10`
- Reason: not_exercised
- Remedy: JNI method declaration, not a holder; native growth is attributed to jvm/F19 (Native.open handle left open on bulk path)
- Supporting evidence: jvm/E4

### jvm/F10: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.size`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:11`
- Reason: not_exercised
- Remedy: JNI method declaration, not a holder; native growth is attributed to jvm/F19 (Native.open handle left open on bulk path)
- Supporting evidence: jvm/E4

### jvm/F11: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.lines`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:12`
- Reason: not_exercised
- Remedy: JNI method declaration, not a holder; native growth is attributed to jvm/F19 (Native.open handle left open on bulk path)
- Supporting evidence: jvm/E4

### jvm/F12: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.close`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:13`
- Reason: not_exercised
- Remedy: JNI method declaration, not a holder; native growth is attributed to jvm/F19 (Native.open handle left open on bulk path)
- Supporting evidence: jvm/E4

### jvm/F13: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.live`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:14`
- Reason: not_exercised
- Remedy: JNI method declaration, not a holder; native growth is attributed to jvm/F19 (Native.open handle left open on bulk path)
- Supporting evidence: jvm/E4

### jvm/F14: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.liveBytes`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:15`
- Reason: not_exercised
- Remedy: JNI method declaration, not a holder; native growth is attributed to jvm/F19 (Native.open handle left open on bulk path)
- Supporting evidence: jvm/E4

### jvm/F15: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.OrderGenerator.Companion`
- Region: heap
- Site: `kotlin/shop/Order.kt:12`
- Reason: not_exercised
- Remedy: Constant, singleton or fixed-size holder with no growth in S1 and no adding site; confirm with a longer run only if its size is expected to change
- Supporting evidence: jvm/E4

### jvm/F16: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.OrderGenerator.skus`
- Region: heap
- Site: `kotlin/shop/Order.kt:14`
- Reason: not_exercised
- Remedy: Constant, singleton or fixed-size holder with no growth in S1 and no adding site; confirm with a longer run only if its size is expected to change
- Supporting evidence: jvm/E4

### jvm/F17: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.Order.lines`
- Region: heap
- Site: `kotlin/shop/Order.kt:18`
- Reason: not_exercised
- Remedy: Constant, singleton or fixed-size holder with no growth in S1 and no adding site; confirm with a longer run only if its size is expected to change
- Supporting evidence: jvm/E4

### jvm/F18: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.Stats.Companion`
- Region: heap
- Site: `kotlin/shop/Stats.kt:6`
- Reason: not_exercised
- Remedy: Constant, singleton or fixed-size holder with no growth in S1 and no adding site; confirm with a longer run only if its size is expected to change
- Supporting evidence: jvm/E4

### rust/F1: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `native.src.frame.Frame.bytes`
- Region: heap
- Site: `native/src/frame.rs:11`
- Writers: `native/src/lib.rs:47`
- Removers: `native/src/lib.rs:76`
- Annotation: Insert in open, removal only via close called from JVM
- Reason: no_harness
- Remedy: cdylib has no bin or lib target for dla harness; measure via the JVM worker harness with the shim and run dla correlate H1 to tie native site native/src/lib.rs:47 (Native.open insert into FRAMES) to the managed finding. Static reading: Rust close() at lib.rs:76 removes correctly; handles leak when the Kotlin caller (Encoder.kt bulk path, size>16384) never calls Native.close
- Supporting evidence: rust/E3

### rust/F2: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: static/global
- Owner: `native.src.lib.FRAMES`
- Region: heap
- Site: `native/src/lib.rs:19`
- Writers: `native/src/lib.rs:47`
- Annotation: Frame.bytes owned by Frame inside FRAMES
- Reason: no_harness
- Remedy: Frame.bytes is owned by the entries of FRAMES (F1); same JVM-side harness and correlate H1 cover it; not an independent holder
- Supporting evidence: rust/E3

## Ruled out

- **jvm/F3.n1** (UNCLASSIFIED): discriminator jvm/E21
  - Ruled out by: plateau+purge (leak) jvm/E21
  - Hypothesis: N-WARM: Audit.trail growth is a warm-up fill that plateaus
  - Refuted if: growth continues over the extra alloc-test cycles
  - Contradicting evidence: jvm/E21
  - Originally supported by: jvm/E12
- **jvm/F19.n1** (UNCLASSIFIED): discriminator jvm/E22
  - Ruled out by: plateau+purge (leak) jvm/E22
  - Hypothesis: N-HWM: native live bytes are an allocator high-water mark that plateaus
  - Refuted if: live blocks keep rising over extra cycles after purge
  - Contradicting evidence: jvm/E22
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
- E8: limitation: 31810680 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: 22647071 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: 22658760 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: 20600909 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: 32957816 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: 31812496 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: 22038962 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E8: limitation: 22298398 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- E8: limitation: gate thresholds are provisional (not calibrated)
- E9: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E9: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E9: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E9: limitation: 31810680 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: 22647071 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: 22658760 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: 20600909 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: 32957816 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: 31812496 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: 22038962 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E9: limitation: 22298398 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- E9: limitation: gate thresholds are provisional (not calibrated)
- E10: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- E10: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E10: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E10: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- E10: limitation: 43238927 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: 29380255 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: 24395491 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: 44207972 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: 20914932 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: 44734036 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: 30847854 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: 32962025 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: 40644816 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E11: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E11: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- E11: limitation: 43238927 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: 29380255 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: 24395491 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: 44207972 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: 20914932 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: 44734036 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: 30847854 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: 32962025 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: 40644816 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: series: per-point; x is the iteration count n of each point (2220, 8880, 13320, 19980); every point is a separate process and heap dump
- E12: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E12: limitation: holders absent from a dump's 200 largest have no figure at that point
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: series: per-point; x is the iteration count n of each point (2220, 8880, 13320, 19980); every point is a separate process and heap dump
- E13: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E13: limitation: holders absent from a dump's 200 largest have no figure at that point
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E14: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E15: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E16: limitation: 31810680 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 22647071 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 22658760 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 20600909 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 32957816 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 31812496 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 22038962 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E16: limitation: 22298398 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E17: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E17: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E17: limitation: 31810680 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 22647071 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 22658760 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 20600909 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 32957816 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 31812496 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 22038962 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E17: limitation: 22298398 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- E18: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E18: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E18: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- E18: limitation: 43238927 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 29380255 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 24395491 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 44207972 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 20914932 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 44734036 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 30847854 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 32962025 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 40644816 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- E18: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E18: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E18: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E18: limitation: 31810680 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 22647071 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 22658760 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 20600909 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 32957816 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 31812496 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 22038962 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E18: limitation: 22298398 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- E18: limitation: points 1 to 9 come from S1; the extra points from S4
- E18: limitation: 2 columns beyond the 64 the series allows were dropped
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E19: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E19: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- E19: limitation: 43238927 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 29380255 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 24395491 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 44207972 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 20914932 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 44734036 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 30847854 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 32962025 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 40644816 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- E19: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E19: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E19: limitation: 31810680 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 22647071 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 22658760 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 20600909 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 32957816 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 31812496 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 22038962 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E19: limitation: 22298398 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- E19: limitation: points 1 to 9 come from S1; the extra points from S4
- E19: limitation: 2 columns beyond the 64 the series allows were dropped
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E20: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E20: limitation: 22094598 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: 30765871 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: 21550418 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: 28374104 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: 30381584 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: 30943481 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: 26968080 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E20: limitation: 22176206 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E21: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E21: limitation: 22094598 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: 30765871 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: 21550418 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: 28374104 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: 30381584 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: 30943481 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: 26968080 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E21: limitation: 22176206 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: partial coverage (2 of 3 examined)
- E22: limitation: jvm/F3: growth measured as a count; bytes not attributed
- E22: limitation: jvm/F19: growth measured as a count; bytes not attributed
- E22: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
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
- jvm/E7: limitation: 43238927 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E7: limitation: 29380255 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E8: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E8: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E8: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E8: limitation: 43238927 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 29380255 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 24395491 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 44207972 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 20914932 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 44734036 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 30847854 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 32962025 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 40644816 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E8: limitation: gate thresholds are provisional (not calibrated)
- jvm/E9: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E9: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E9: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E9: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E9: limitation: 43238927 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 29380255 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 24395491 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 44207972 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 20914932 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 44734036 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 30847854 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 32962025 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 40644816 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E9: limitation: gate thresholds are provisional (not calibrated)
- jvm/E10: partial coverage (1 of 1 examined)
- jvm/E10: limitation: the stack is the project frame of the largest allocation stack at the site, not every frame
- jvm/E10: limitation: slopes are per unit of the series axis; the thresholds are provisional
- jvm/E11: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E11: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E11: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E11: limitation: 43238927 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 29380255 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 24395491 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 44207972 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 20914932 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 44734036 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 30847854 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 32962025 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 40644816 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E11: limitation: gate thresholds are provisional (not calibrated)
- jvm/E12: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E12: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E12: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E12: limitation: 43238927 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 29380255 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 24395491 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 44207972 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 20914932 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 44734036 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 30847854 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 32962025 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 40644816 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E12: limitation: gate thresholds are provisional (not calibrated)
- jvm/E13: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- jvm/E14: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- jvm/E15: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- jvm/E16: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- jvm/E17: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- jvm/E17: limitation: series: per-point; one process per point at n = 2220, 8880, 13320, 19980 (harness captures; checked: HPROF version, identifier size)
- jvm/E18: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- jvm/E19: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- jvm/E19: limitation: the files carry no time, so they are ordered as given
- jvm/E20: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- jvm/E20: limitation: the files carry no time, so they are ordered as given
- jvm/E20: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- jvm/E21: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E21: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E21: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E21: limitation: 43238927 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 29380255 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 24395491 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 44207972 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 20914932 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 44734036 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 30847854 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 32962025 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 40644816 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E21: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E21: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E21: limitation: 31810680 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 22647071 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 22658760 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 20600909 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 32957816 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 31812496 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 22038962 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E21: limitation: 22298398 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E21: limitation: points 1 to 9 come from S1; the extra points from S4
- jvm/E21: limitation: 2 columns beyond the 64 the series allows were dropped
- jvm/E21: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E21: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E21: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E21: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E22: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E22: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E22: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E22: limitation: 43238927 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 29380255 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 24395491 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 44207972 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 20914932 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 44734036 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 30847854 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 32962025 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 40644816 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E22: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E22: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E22: limitation: 22094598 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 30765871 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 21550418 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 28374104 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 30381584 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 30943481 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 26968080 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E22: limitation: 22176206 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E22: limitation: points 1 to 9 come from S1; the extra points from S6
- jvm/E22: limitation: 9 columns beyond the 64 the series allows were dropped
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
| E4 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (08189a22949c) | trend/T5.json |
| E5 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (08189a22949c) | trend/T5.json |
| E6 | growth-series | dump | dla 0.1.0 | artifact series/S3.json (d1a65cef1431) | trend/T6.json |
| E7 | unit-count-series | dump | dla 0.1.0 | artifact series/S3.json (d1a65cef1431) | trend/T6.json |
| E8 | growth-series | test | dla 0.1.0 | artifact series/S4.json (c9594b0f1099) | trend/T7.json |
| E9 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (c9594b0f1099) | trend/T7.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (8787c906f172) | trend/T8.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (8787c906f172) | trend/T8.json |
| E12 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (08189a22949c) | trend/T9.json |
| E13 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (08189a22949c) | trend/T9.json |
| E14 | growth-series | dump | dla 0.1.0 | artifact series/S3.json (d1a65cef1431) | trend/T10.json |
| E15 | unit-count-series | dump | dla 0.1.0 | artifact series/S3.json (d1a65cef1431) | trend/T10.json |
| E16 | growth-series | test | dla 0.1.0 | artifact series/S4.json (c9594b0f1099) | trend/T11.json |
| E17 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (c9594b0f1099) | trend/T11.json |
| E18 | growth-series | test | dla 0.1.0 | artifact series/S5.json (80677f35563b) | trend/T12.json |
| E19 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (80677f35563b) | trend/T12.json |
| E20 | growth-series | test | dla 0.1.0 | artifact series/S6.json (c4cf285cdaf7) | trend/T13.json |
| E21 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (c4cf285cdaf7) | trend/T13.json |
| E22 | residual | test | dla 0.1.0 | artifact series/S1.json (8787c906f172) | gap/residual-S1-E22.json |
| jvm/E1 | static-hit | static | spotbugs unknown | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-spotbugs.json |
| jvm/E2 | static-hit | static | detekt 2.0.0-alpha.6 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-detekt.json |
| jvm/E3 | static-hit | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-dla.json |
| jvm/E4 | census-holder | static | dla-census 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | census/jvm.json |
| jvm/E5 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | workload/jvm.json |
| jvm/E6 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | harness/H1/harness.json |
| jvm/E7 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (ef701bdb6b92) | harness/H1/run-S1.json |
| jvm/E8 | growth-series | test | dla 0.1.0 | artifact series/S1.json (8787c906f172) | trend/T1.json |
| jvm/E9 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (8787c906f172) | trend/T1.json |
| jvm/E10 | alloc-stack | test | dla-correlate 0.1.0 | artifact series/S1.json (8787c906f172) | correlate/C1.json |
| jvm/E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (8787c906f172) | trend/T2.json |
| jvm/E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (8787c906f172) | trend/T2.json |
| jvm/E13 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-192004-7ac4/harness/H1/out/p1-native/capture/9/heap-1.hprof (7727da729c91) | summaries/heap-1-7727da72.json |
| jvm/E14 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-192004-7ac4/harness/H1/out/p4-native/capture/391/heap-1.hprof (877e8a20f05b) | summaries/heap-1-877e8a20.json |
| jvm/E15 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-192004-7ac4/harness/H1/out/p6-native/capture/621/heap-1.hprof (f7a0f1edbb7c) | summaries/heap-1-f7a0f1ed.json |
| jvm/E16 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-192004-7ac4/harness/H1/out/p9-native/capture/959/heap-1.hprof (891c1fc90357) | summaries/heap-1-891c1fc9.json |
| jvm/E17 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S2.json (08189a22949c) | series/S2.json |
| jvm/E18 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S2.json (08189a22949c) | series/S2.json |
| jvm/E19 | histogram-diff | live | jcmd-class-histogram 0.1.0 | artifact histograms/H1.json (a77fbe2e3f66) | histograms/H1.json |
| jvm/E20 | growth-series | live | jcmd-class-histogram 0.1.0 | artifact series/S3.json (d1a65cef1431) | series/S3.json |
| jvm/E21 | discriminator | test | dla 0.1.0 | artifact series/S5.json (80677f35563b) | alloc/A1.json |
| jvm/E22 | discriminator | test | dla 0.1.0 | artifact series/S7.json (9d2a1067796d) | alloc/A2.json |
| rust/E1 | static-hit | static | rapx 0.7.71 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/rust-rapx.json |
| rust/E2 | static-hit | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/rust-dla.json |
| rust/E3 | census-holder | static | dla-census 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | census/rust.json |
| rust/E4 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | workload/rust.json |
