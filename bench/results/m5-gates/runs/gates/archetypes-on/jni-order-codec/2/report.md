# DLA report

Run: `20261109-201142-d807`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 23 enumerated, 23 dispositioned; confirmed 2, strong 0, refuted 2, suspected 18, unresolved 1

Residual: not measured

Coverage: jvm 17760 iterations, 0 of 17 entry points exercised; rust no iterations, 0 of 10 entry points exercised

## Workers

- **jvm** (jvm): status done; 73 lines merged; slice done; torn tail: no
- **rust** (rust): status done; 12 lines merged; slice done; torn tail: no

## Findings

### jvm/F3: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:shop.codec.Audit.trail.bytes` 1437 bytes/n (steady, gate pass, jvm/E8)
- Evidence classes: context, discriminator, growth, static
- Holder: static/global
- Owner: `shop.codec.Audit.trail`
- Region: heap
- Shape: steady
- Site: `kotlin/shop/Audit.kt:9`
- Writers: `kotlin/shop/Audit.kt:12`, `kotlin/shop/Encoder.kt:17`
- Annotation: heuristic name match over the project's sources
- Annotation: Audit.trail (object Audit) is appended per inline frame in OrderEncoder.encode and never removed; 1536-byte sample per entry
- Sibling hypotheses: jvm/F3.n1
- Supporting evidence: jvm/E4, jvm/E7, jvm/E8, jvm/E9, jvm/E11

### jvm/F19: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `native.live_bytes@native/src/lib.rs:40` 3074 bytes/n (steady, gate pass, jvm/E8)
- Evidence classes: allocation, context, discriminator, growth
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.open`
- Region: off-heap
- Shape: steady
- Site: `native/src/lib.rs:40`
- Writers: `kotlin/shop/Native.kt:9`, `kotlin/shop/Encoder.kt:9`
- Annotation: native allocation under shop.codec.Native.open (Java_shop_codec_Native_open); found by dla correlate
- Annotation: Native.open handle in OrderEncoder.encode is not closed on the bulk path (size > MAX_INLINE_BYTES returns before Native.close); missing pairing is close on every exit
- Sibling hypotheses: jvm/F19.n1
- Supporting evidence: jvm/E10, jvm/E7, jvm/E8, jvm/E12

### jvm/F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.Entry.sample`
- Region: heap
- Site: `kotlin/shop/Audit.kt:3`
- Writers: `kotlin/shop/Audit.kt:12`
- Annotation: Audit.trail only added to (Encoder.kt:17), never removed; 1536-byte sample per inline frame
- Reason: not_exercised
- Remedy: Entry.sample is the payload of confirmed jvm/F3 (Audit.trail); no separate holder, retained via trail
- Supporting evidence: jvm/E4

### jvm/F4: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.OrderEncoder.Companion`
- Region: heap
- Site: `kotlin/shop/Encoder.kt:6`
- Reason: not_exercised
- Remedy: no probe column grew for this holder in H1 (companion/constant/field with no unbounded writer); add a driver probe to refute with alloc-test
- Supporting evidence: jvm/E4

### jvm/F5: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.FramePacker.MAGIC`
- Region: heap
- Site: `kotlin/shop/FramePacker.kt:9`
- Reason: not_exercised
- Remedy: no probe column grew for this holder in H1 (companion/constant/field with no unbounded writer); add a driver probe to refute with alloc-test
- Supporting evidence: jvm/E4

### jvm/F6: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.Native.INSTANCE`
- Region: heap
- Site: `kotlin/shop/Native.kt:5`
- Reason: not_exercised
- Remedy: no probe column grew for this holder in H1 (companion/constant/field with no unbounded writer); add a driver probe to refute with alloc-test
- Supporting evidence: jvm/E4

### jvm/F7: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.<clinit>`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:6`
- Reason: not_exercised
- Remedy: JNI declaration; native leak attributed to confirmed jvm/F19 (Native.open handle not closed on bulk path)
- Supporting evidence: jvm/E4

### jvm/F8: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.open`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:9`
- Reason: not_exercised
- Remedy: JNI declaration; native leak attributed to confirmed jvm/F19 (Native.open handle not closed on bulk path)
- Supporting evidence: jvm/E4

### jvm/F9: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.checksum`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:10`
- Reason: not_exercised
- Remedy: JNI declaration; native leak attributed to confirmed jvm/F19 (Native.open handle not closed on bulk path)
- Supporting evidence: jvm/E4

### jvm/F10: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.size`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:11`
- Reason: not_exercised
- Remedy: JNI declaration; native leak attributed to confirmed jvm/F19 (Native.open handle not closed on bulk path)
- Supporting evidence: jvm/E4

### jvm/F11: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.lines`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:12`
- Reason: not_exercised
- Remedy: JNI declaration; native leak attributed to confirmed jvm/F19 (Native.open handle not closed on bulk path)
- Supporting evidence: jvm/E4

### jvm/F12: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.close`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:13`
- Reason: not_exercised
- Remedy: JNI declaration; native leak attributed to confirmed jvm/F19 (Native.open handle not closed on bulk path)
- Supporting evidence: jvm/E4

### jvm/F13: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.live`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:14`
- Reason: not_exercised
- Remedy: JNI declaration; native leak attributed to confirmed jvm/F19 (Native.open handle not closed on bulk path)
- Supporting evidence: jvm/E4

### jvm/F14: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.liveBytes`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:15`
- Reason: not_exercised
- Remedy: JNI declaration; native leak attributed to confirmed jvm/F19 (Native.open handle not closed on bulk path)
- Supporting evidence: jvm/E4

### jvm/F15: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.OrderGenerator.Companion`
- Region: heap
- Site: `kotlin/shop/Order.kt:12`
- Reason: not_exercised
- Remedy: no probe column grew for this holder in H1 (companion/constant/field with no unbounded writer); add a driver probe to refute with alloc-test
- Supporting evidence: jvm/E4

### jvm/F16: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.OrderGenerator.skus`
- Region: heap
- Site: `kotlin/shop/Order.kt:14`
- Reason: not_exercised
- Remedy: no probe column grew for this holder in H1 (companion/constant/field with no unbounded writer); add a driver probe to refute with alloc-test
- Supporting evidence: jvm/E4

### jvm/F17: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.Order.lines`
- Region: heap
- Site: `kotlin/shop/Order.kt:18`
- Reason: not_exercised
- Remedy: no probe column grew for this holder in H1 (companion/constant/field with no unbounded writer); add a driver probe to refute with alloc-test
- Supporting evidence: jvm/E4

### jvm/F18: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.Stats.Companion`
- Region: heap
- Site: `kotlin/shop/Stats.kt:6`
- Reason: not_exercised
- Remedy: no probe column grew for this holder in H1 (companion/constant/field with no unbounded writer); add a driver probe to refute with alloc-test
- Supporting evidence: jvm/E4

### rust/F1: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `native.src.frame.Frame.bytes`
- Region: heap
- Site: `native/src/frame.rs:11`
- Annotation: Frame.bytes owned by FRAMES entries; lifetime follows F2
- Reason: no_harness
- Remedy: Frame.bytes lives only inside FRAMES entries (F2); same JNI measurement via the JVM worker
- Supporting evidence: rust/E3

### rust/F2: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: static/global
- Owner: `native.src.lib.FRAMES`
- Region: heap
- Site: `native/src/lib.rs:19`
- Writers: `native/src/lib.rs:43`
- Removers: `native/src/lib.rs:69`
- Annotation: FRAMES map; insert in open, remove only in close; Kotlin Encoder skips close for frames >16384 bytes (bulk lane)
- Reason: no_harness
- Remedy: cdylib has no bin or lib target for the Rust harness; measure from the JVM worker with the shim and run dla correlate H1. Cause location: native/src/lib.rs:43 inserts into FRAMES, only Native.close (lib.rs:69) removes; kotlin/shop/Encoder.kt returns early without Native.close for frames over 16384 bytes
- Supporting evidence: rust/E3

### jvm/F1: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Site: `kotlin/shop/Audit.kt:9`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: same site Audit.kt:9 as confirmed jvm/F3; detector duplicate, merge dedupes by site/holder
- Supporting evidence: jvm/E3

## Ruled out

- **jvm/F3.n1** (UNCLASSIFIED): discriminator jvm/E11
  - Ruled out by: plateau+purge (leak) jvm/E11
  - Hypothesis: N-WARM: Audit.trail fills to a bounded warm-up size then plateaus
  - Refuted if: growth continues over the extra alloc-test cycles (dump-plateau favors leak)
  - Contradicting evidence: jvm/E11
  - Originally supported by: jvm/E8
- **jvm/F19.n1** (UNCLASSIFIED): discriminator jvm/E12
  - Ruled out by: plateau+purge (leak) jvm/E12
  - Hypothesis: N-ALLOC: native live bytes are allocator retention that plateaus after purge
  - Refuted if: native.live_bytes at lib.rs:40 keeps growing across extra cycles
  - Contradicting evidence: jvm/E12
  - Originally supported by: jvm/E8

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
- jvm/E7: limitation: 44282127 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E7: limitation: 33401305 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E8: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E8: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E8: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E8: limitation: 44282127 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 33401305 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 30447128 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 39155941 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 21651799 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 40535964 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 30240237 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 20065118 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 38998800 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E8: limitation: gate thresholds are provisional (not calibrated)
- jvm/E9: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E9: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E9: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E9: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E9: limitation: 44282127 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 33401305 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 30447128 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 39155941 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 21651799 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 40535964 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 30240237 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 20065118 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 38998800 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E9: limitation: gate thresholds are provisional (not calibrated)
- jvm/E10: partial coverage (1 of 1 examined)
- jvm/E10: limitation: the stack is the project frame of the largest allocation stack at the site, not every frame
- jvm/E10: limitation: slopes are per unit of the series axis; the thresholds are provisional
- jvm/E11: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E11: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E11: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E11: limitation: 44282127 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 33401305 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 30447128 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 39155941 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 21651799 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 40535964 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 30240237 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 20065118 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 38998800 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E11: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E11: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E11: limitation: 32404627 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 21185179 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 33027515 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 37539336 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 20431586 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 29154391 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 31470470 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E11: limitation: 21586312 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E11: limitation: points 1 to 9 come from S1; the extra points from S2
- jvm/E11: limitation: 7 columns beyond the 64 the series allows were dropped
- jvm/E11: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E11: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E11: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E11: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E12: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E12: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E12: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E12: limitation: 44282127 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 33401305 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 30447128 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 39155941 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 21651799 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 40535964 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 30240237 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 20065118 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 38998800 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E12: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E12: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E12: limitation: 22008099 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 23328441 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 21151993 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 34494211 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 21285765 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 21726823 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 24864837 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E12: limitation: 36458465 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E12: limitation: points 1 to 9 come from S1; the extra points from S4
- jvm/E12: limitation: 8 columns beyond the 64 the series allows were dropped
- jvm/E12: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E12: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E12: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E12: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E13: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E13: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E13: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E13: limitation: 44282127 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 33401305 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 30447128 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 39155941 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 21651799 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 40535964 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 30240237 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 20065118 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 38998800 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E13: limitation: gate thresholds are provisional (not calibrated)
- jvm/E14: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E14: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E14: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E14: limitation: 44282127 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 33401305 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 30447128 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 39155941 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 21651799 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 40535964 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 30240237 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 20065118 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 38998800 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E14: limitation: gate thresholds are provisional (not calibrated)
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
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E3 | — |
| `intake.build` | Does the project build locally? | answered | E3 | — |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | — |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | — |
| E2 | run-config | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | probe/probe.json |
| jvm/E1 | static-hit | static | spotbugs unknown | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-spotbugs.json |
| jvm/E2 | static-hit | static | detekt 2.0.0-alpha.6 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-detekt.json |
| jvm/E3 | static-hit | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-dla.json |
| jvm/E4 | census-holder | static | dla-census 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | census/jvm.json |
| jvm/E5 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | workload/jvm.json |
| jvm/E6 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | harness/H1/harness.json |
| jvm/E7 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (af580051ce4f) | harness/H1/run-S1.json |
| jvm/E8 | growth-series | test | dla 0.1.0 | artifact series/S1.json (05712829fd33) | trend/T1.json |
| jvm/E9 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (05712829fd33) | trend/T1.json |
| jvm/E10 | alloc-stack | test | dla-correlate 0.1.0 | artifact series/S1.json (05712829fd33) | correlate/C1.json |
| jvm/E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (9b206f2bf382) | alloc/A1.json |
| jvm/E12 | discriminator | test | dla 0.1.0 | artifact series/S5.json (c0ba81b949b4) | alloc/A2.json |
| jvm/E13 | growth-series | test | dla 0.1.0 | artifact series/S1.json (05712829fd33) | trend/T4.json |
| jvm/E14 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (05712829fd33) | trend/T4.json |
| jvm/E15 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261109-201142-d807/harness/H1/out/p1-native/capture/9/heap-1.hprof (5763c8035ddc) | summaries/heap-1-5763c803.json |
| jvm/E16 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261109-201142-d807/harness/H1/out/p4-native/capture/388/heap-1.hprof (149de4bd1d54) | summaries/heap-1-149de4bd.json |
| jvm/E17 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261109-201142-d807/harness/H1/out/p6-native/capture/618/heap-1.hprof (a54cd4b345f5) | summaries/heap-1-a54cd4b3.json |
| jvm/E18 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261109-201142-d807/harness/H1/out/p9-native/capture/959/heap-1.hprof (96f915a85ee1) | summaries/heap-1-96f915a8.json |
| jvm/E19 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S6.json (25f12de0db4d) | series/S6.json |
| jvm/E20 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S6.json (25f12de0db4d) | series/S6.json |
| jvm/E21 | histogram-diff | live | jcmd-class-histogram 0.1.0 | artifact histograms/H1.json (644564e19add) | histograms/H1.json |
| jvm/E22 | growth-series | live | jcmd-class-histogram 0.1.0 | artifact series/S7.json (5f6bf3990e57) | series/S7.json |
| rust/E1 | static-hit | static | rapx 0.7.71 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/rust-rapx.json |
| rust/E2 | static-hit | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/rust-dla.json |
| rust/E3 | census-holder | static | dla-census 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | census/rust.json |
| rust/E4 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | workload/rust.json |
