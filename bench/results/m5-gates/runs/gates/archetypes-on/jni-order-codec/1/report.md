# DLA report

Run: `20261109-194249-683f`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 23 enumerated, 23 dispositioned; confirmed 2, strong 0, refuted 2, suspected 19, unresolved 0

Residual: not measured

Coverage: jvm 17760 iterations, 0 of 17 entry points exercised; rust no iterations, 0 of 10 entry points exercised

## Workers

- **jvm** (jvm): status done; 68 lines merged; slice done; torn tail: no
- **rust** (rust): status done; 13 lines merged; slice done; torn tail: no

## Findings

### jvm/F3: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:shop.codec.Audit.trail` 2.625 count/n (steady, gate pass, jvm/E9)
- Evidence classes: context, discriminator, growth, static
- Holder: static/global
- Owner: `shop.codec.Audit.trail`
- Region: heap
- Shape: steady
- Site: `kotlin/shop/Audit.kt:9`
- Writers: `kotlin/shop/Audit.kt:12`
- Annotation: heuristic name match over the project's sources
- Annotation: Audit.trail (object, process lifetime) gets a 1536-byte sample per inline frame via OrderEncoder.encode; nothing ever removes entries. Missing pairing: bounded or flushed trail.
- Sibling hypotheses: jvm/F3.n1
- Supporting evidence: jvm/E4, jvm/E7, jvm/E9, jvm/E11

### jvm/F19: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `native.live_blocks@native/src/lib.rs:40` 0.125 count/n (steady, gate pass, jvm/E9)
- Evidence classes: allocation, context, discriminator, growth
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.open`
- Region: off-heap
- Shape: steady
- Site: `native/src/lib.rs:40`
- Writers: `kotlin/shop/Native.kt:9`
- Annotation: native allocation under shop.codec.Native.open (Java_shop_codec_Native_open); found by dla correlate
- Annotation: Native.open handle in OrderEncoder.encode is never Native.close()d on the bulk path (size > MAX_INLINE_BYTES returns early); native frame stays live. Missing pairing: close in finally.
- Sibling hypotheses: jvm/F19.n1
- Supporting evidence: jvm/E10, jvm/E7, jvm/E9, jvm/E12

### jvm/F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Site: `kotlin/shop/Audit.kt:9`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: duplicate detector hit of confirmed jvm/F3 (same site Audit.kt:9); no separate action
- Supporting evidence: jvm/E3

### jvm/F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.Entry.sample`
- Region: heap
- Site: `kotlin/shop/Audit.kt:3`
- Reason: not_exercised
- Remedy: census lead only: no passing growth column tied to this holder (Entry.sample is retained via confirmed F3; Native method/companion/constant entries are not holders). Re-test with a driver probe if suspected.
- Supporting evidence: jvm/E4

### jvm/F4: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.OrderEncoder.Companion`
- Region: heap
- Site: `kotlin/shop/Encoder.kt:6`
- Reason: not_exercised
- Remedy: census lead only: no passing growth column tied to this holder (Entry.sample is retained via confirmed F3; Native method/companion/constant entries are not holders). Re-test with a driver probe if suspected.
- Supporting evidence: jvm/E4

### jvm/F5: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.FramePacker.MAGIC`
- Region: heap
- Site: `kotlin/shop/FramePacker.kt:9`
- Reason: not_exercised
- Remedy: census lead only: no passing growth column tied to this holder (Entry.sample is retained via confirmed F3; Native method/companion/constant entries are not holders). Re-test with a driver probe if suspected.
- Supporting evidence: jvm/E4

### jvm/F6: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.Native.INSTANCE`
- Region: heap
- Site: `kotlin/shop/Native.kt:5`
- Reason: not_exercised
- Remedy: census lead only: no passing growth column tied to this holder (Entry.sample is retained via confirmed F3; Native method/companion/constant entries are not holders). Re-test with a driver probe if suspected.
- Supporting evidence: jvm/E4

### jvm/F7: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.<clinit>`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:6`
- Reason: not_exercised
- Remedy: census lead only: no passing growth column tied to this holder (Entry.sample is retained via confirmed F3; Native method/companion/constant entries are not holders). Re-test with a driver probe if suspected.
- Supporting evidence: jvm/E4

### jvm/F8: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.open`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:9`
- Reason: not_exercised
- Remedy: census lead only: no passing growth column tied to this holder (Entry.sample is retained via confirmed F3; Native method/companion/constant entries are not holders). Re-test with a driver probe if suspected.
- Supporting evidence: jvm/E4

### jvm/F9: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.checksum`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:10`
- Reason: not_exercised
- Remedy: census lead only: no passing growth column tied to this holder (Entry.sample is retained via confirmed F3; Native method/companion/constant entries are not holders). Re-test with a driver probe if suspected.
- Supporting evidence: jvm/E4

### jvm/F10: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.size`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:11`
- Reason: not_exercised
- Remedy: census lead only: no passing growth column tied to this holder (Entry.sample is retained via confirmed F3; Native method/companion/constant entries are not holders). Re-test with a driver probe if suspected.
- Supporting evidence: jvm/E4

### jvm/F11: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.lines`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:12`
- Reason: not_exercised
- Remedy: census lead only: no passing growth column tied to this holder (Entry.sample is retained via confirmed F3; Native method/companion/constant entries are not holders). Re-test with a driver probe if suspected.
- Supporting evidence: jvm/E4

### jvm/F12: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.close`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:13`
- Reason: not_exercised
- Remedy: census lead only: no passing growth column tied to this holder (Entry.sample is retained via confirmed F3; Native method/companion/constant entries are not holders). Re-test with a driver probe if suspected.
- Supporting evidence: jvm/E4

### jvm/F13: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.live`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:14`
- Reason: not_exercised
- Remedy: census lead only: no passing growth column tied to this holder (Entry.sample is retained via confirmed F3; Native method/companion/constant entries are not holders). Re-test with a driver probe if suspected.
- Supporting evidence: jvm/E4

### jvm/F14: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.liveBytes`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:15`
- Reason: not_exercised
- Remedy: census lead only: no passing growth column tied to this holder (Entry.sample is retained via confirmed F3; Native method/companion/constant entries are not holders). Re-test with a driver probe if suspected.
- Supporting evidence: jvm/E4

### jvm/F15: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.OrderGenerator.Companion`
- Region: heap
- Site: `kotlin/shop/Order.kt:12`
- Reason: not_exercised
- Remedy: census lead only: no passing growth column tied to this holder (Entry.sample is retained via confirmed F3; Native method/companion/constant entries are not holders). Re-test with a driver probe if suspected.
- Supporting evidence: jvm/E4

### jvm/F16: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.OrderGenerator.skus`
- Region: heap
- Site: `kotlin/shop/Order.kt:14`
- Reason: not_exercised
- Remedy: census lead only: no passing growth column tied to this holder (Entry.sample is retained via confirmed F3; Native method/companion/constant entries are not holders). Re-test with a driver probe if suspected.
- Supporting evidence: jvm/E4

### jvm/F17: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.Order.lines`
- Region: heap
- Site: `kotlin/shop/Order.kt:18`
- Reason: not_exercised
- Remedy: census lead only: no passing growth column tied to this holder (Entry.sample is retained via confirmed F3; Native method/companion/constant entries are not holders). Re-test with a driver probe if suspected.
- Supporting evidence: jvm/E4

### jvm/F18: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.Stats.Companion`
- Region: heap
- Site: `kotlin/shop/Stats.kt:6`
- Reason: not_exercised
- Remedy: census lead only: no passing growth column tied to this holder (Entry.sample is retained via confirmed F3; Native method/companion/constant entries are not holders). Re-test with a driver probe if suspected.
- Supporting evidence: jvm/E4

### rust/F1: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `native.src.frame.Frame.bytes`
- Region: heap
- Site: `native/src/frame.rs:11`
- Annotation: x
- Annotation: Correction to earlier placeholder note: Frame.bytes is the payload owned by each FRAMES entry; it is retained only as long as the FRAMES entry exists. Not an independent holder.
- Reason: no_harness
- Remedy: Payload of FRAMES entries (rust/F2); same JVM-side harness and correlate apply
- Supporting evidence: rust/E3

### rust/F2: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: static/global
- Owner: `native.src.lib.FRAMES`
- Region: heap
- Site: `native/src/lib.rs:19`
- Writers: `native/src/lib.rs:44`
- Removers: `native/src/lib.rs:73`
- Annotation: Static FRAMES map keyed by handle: Native_open inserts, Native_close removes. Rust side is balanced; entries stay only if the caller never calls close. kotlin/shop/Encoder.kt encode() returns early without Native.close(handle) when size > MAX_INLINE_BYTES (bulk lane), so those frames stay in FRAMES forever. Missing pairing is the close for every open on the bulk path. Cause location is the JVM caller; measure via JVM worker harness and dla correlate.
- Reason: no_harness
- Remedy: Crate is a cdylib with no main or lib target, so no Rust harness exists; measure from the JVM worker with the shim and run dla correlate H1 to tie native.live_bytes growth to the bulk-frame path in Encoder.kt
- Supporting evidence: rust/E3

## Ruled out

- **jvm/F3.n1** (UNCLASSIFIED): discriminator jvm/E11
  - Ruled out by: plateau+purge (leak) jvm/E11
  - Hypothesis: N-WARM: Audit.trail fills then plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: jvm/E11
  - Originally supported by: jvm/E9
- **jvm/F19.n1** (UNCLASSIFIED): discriminator jvm/E12
  - Ruled out by: plateau+purge (leak) jvm/E12
  - Hypothesis: N-HWM: native live bytes plateau as a pool high-water mark
  - Refuted if: native.live_bytes keeps growing over extra cycles
  - Contradicting evidence: jvm/E12
  - Originally supported by: jvm/E9

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
- jvm/E7: limitation: 41803200 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E7: limitation: 24080971 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E7: limitation: 31590420 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E8: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E8: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E8: limitation: 41803200 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 24080971 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 31590420 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E8: limitation: 42059044 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 30396684 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 44296984 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 30635529 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 33065660 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 44105264 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: dropped 13 per-type columns (cap of 64, lowest growth first)
- jvm/E8: limitation: gate thresholds are provisional (not calibrated)
- jvm/E9: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E9: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E9: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E9: limitation: 41803200 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 24080971 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 31590420 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E9: limitation: 42059044 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 30396684 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 44296984 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 30635529 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 33065660 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 44105264 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: dropped 13 per-type columns (cap of 64, lowest growth first)
- jvm/E9: limitation: gate thresholds are provisional (not calibrated)
- jvm/E10: partial coverage (1 of 1 examined)
- jvm/E10: limitation: the stack is the project frame of the largest allocation stack at the site, not every frame
- jvm/E10: limitation: slopes are per unit of the series axis; the thresholds are provisional
- jvm/E11: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E11: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E11: limitation: 41803200 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 24080971 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 31590420 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E11: limitation: 42059044 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 30396684 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 44296984 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 30635529 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 33065660 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 44105264 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: dropped 13 per-type columns (cap of 64, lowest growth first)
- jvm/E11: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E11: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E11: limitation: 31452875 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 32507197 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 32801039 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 21302104 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 28723469 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 31825901 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 21611328 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E11: limitation: 29901000 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E11: limitation: points 1 to 9 come from S1; the extra points from S2
- jvm/E11: limitation: 8 columns beyond the 64 the series allows were dropped
- jvm/E11: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E11: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E11: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E11: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E12: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E12: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E12: limitation: 41803200 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 24080971 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 31590420 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E12: limitation: 42059044 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 30396684 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 44296984 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 30635529 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 33065660 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 44105264 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: dropped 13 per-type columns (cap of 64, lowest growth first)
- jvm/E12: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E12: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E12: limitation: 33319626 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 31147000 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 32422087 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 32066774 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 29768864 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 28137391 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 20186758 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E12: limitation: 31190855 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E12: limitation: points 1 to 9 come from S1; the extra points from S4
- jvm/E12: limitation: 7 columns beyond the 64 the series allows were dropped
- jvm/E12: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E12: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E12: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E12: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E13: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- jvm/E14: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- jvm/E15: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- jvm/E16: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- jvm/E17: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- jvm/E17: limitation: series: per-point; one process per point at n = 2220, 8880, 13320, 19980 (harness captures; checked: HPROF version, identifier size)
- jvm/E18: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
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
| jvm/E1 | static-hit | static | spotbugs unknown | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-spotbugs.json |
| jvm/E2 | static-hit | static | detekt 2.0.0-alpha.6 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-detekt.json |
| jvm/E3 | static-hit | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-dla.json |
| jvm/E4 | census-holder | static | dla-census 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | census/jvm.json |
| jvm/E5 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | workload/jvm.json |
| jvm/E6 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | harness/H1/harness.json |
| jvm/E7 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (3d37e3999c7e) | harness/H1/run-S1.json |
| jvm/E8 | growth-series | test | dla 0.1.0 | artifact series/S1.json (ae7854effa2c) | trend/T1.json |
| jvm/E9 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (ae7854effa2c) | trend/T1.json |
| jvm/E10 | alloc-stack | test | dla-correlate 0.1.0 | artifact series/S1.json (ae7854effa2c) | correlate/C1.json |
| jvm/E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (8729b1ffc0f4) | alloc/A1.json |
| jvm/E12 | discriminator | test | dla 0.1.0 | artifact series/S5.json (1e30c5e0b2a1) | alloc/A2.json |
| jvm/E13 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261109-194249-683f/harness/H1/out/p1-native/capture/9/heap-1.hprof (8b3c4d2eee2b) | summaries/heap-1-8b3c4d2e.json |
| jvm/E14 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261109-194249-683f/harness/H1/out/p4-native/capture/390/heap-1.hprof (a9bbf5f0e7fc) | summaries/heap-1-a9bbf5f0.json |
| jvm/E15 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261109-194249-683f/harness/H1/out/p6-native/capture/617/heap-1.hprof (80ea87ffbb81) | summaries/heap-1-80ea87ff.json |
| jvm/E16 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261109-194249-683f/harness/H1/out/p9-native/capture/954/heap-1.hprof (3ea6ca9bde37) | summaries/heap-1-3ea6ca9b.json |
| jvm/E17 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S6.json (1e1274aebe53) | series/S6.json |
| jvm/E18 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S6.json (1e1274aebe53) | series/S6.json |
| rust/E1 | static-hit | static | rapx 0.7.71 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/rust-rapx.json |
| rust/E2 | static-hit | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/rust-dla.json |
| rust/E3 | census-holder | static | dla-census 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | census/rust.json |
| rust/E4 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | workload/rust.json |
