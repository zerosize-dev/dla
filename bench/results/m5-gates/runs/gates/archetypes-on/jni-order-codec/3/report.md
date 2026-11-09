# DLA report

Run: `20261109-203606-0633`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 23 enumerated, 23 dispositioned; confirmed 2, strong 0, refuted 2, suspected 19, unresolved 0

Residual: not measured

Coverage: jvm 17760 iterations, 0 of 17 entry points exercised; rust no iterations, 0 of 10 entry points exercised

## Workers

- **jvm** (jvm): status done; 71 lines merged; slice done; torn tail: no
- **rust** (rust): status done; 11 lines merged; slice done; torn tail: no

## Findings

### jvm/F3: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `holder:static:shop.codec.Audit.trail` 8501451 bytes/n (steady, gate pass, jvm/E15)
- Evidence classes: context, discriminator, growth, static
- Holder: static/global
- Owner: `shop.codec.Audit.trail`
- Region: heap
- Shape: steady
- Site: `kotlin/shop/Audit.kt:9`
- Writers: `kotlin/shop/Audit.kt:12`, `kotlin/shop/Encoder.kt:19`
- Annotation: heuristic name match over the project's sources
- Annotation: Audit.trail (ArrayList in object Audit) gets a 1536-byte frame copy per inline order via OrderEncoder.encode; no remover or cap, so heap grows linearly. Missing pairing: a bound or drain of the audit trail. UNCLASSIFIED (no taxonomy label available).
- Sibling hypotheses: jvm/F3.n1
- Supporting evidence: jvm/E4, jvm/E7, jvm/E15, jvm/E18, jvm/E19, jvm/E16

### jvm/F19: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `native.live_bytes@native/src/lib.rs:40` 3074 bytes/n (steady, gate pass, jvm/E21)
- Evidence classes: allocation, context, discriminator, growth
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.open`
- Region: off-heap
- Shape: steady
- Site: `native/src/lib.rs:40`
- Writers: `kotlin/shop/Native.kt:9`
- Annotation: native allocation under shop.codec.Native.open (Java_shop_codec_Native_open); found by dla correlate
- Sibling hypotheses: jvm/F19.n1
- Supporting evidence: jvm/E10, jvm/E7, jvm/E21, jvm/E22, jvm/E20

### jvm/F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Site: `kotlin/shop/Audit.kt:9`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: No growth column tied to this holder (constant or bounded in the harness); not refuted by plateau test. Add a driver probe for it and rerun if suspected.
- Supporting evidence: jvm/E3

### jvm/F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.Entry.sample`
- Region: heap
- Site: `kotlin/shop/Audit.kt:3`
- Reason: not_exercised
- Remedy: No growth column tied to this holder (constant or bounded in the harness); not refuted by plateau test. Add a driver probe for it and rerun if suspected.
- Supporting evidence: jvm/E4

### jvm/F4: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.OrderEncoder.Companion`
- Region: heap
- Site: `kotlin/shop/Encoder.kt:6`
- Reason: not_exercised
- Remedy: No growth column tied to this holder (constant or bounded in the harness); not refuted by plateau test. Add a driver probe for it and rerun if suspected.
- Supporting evidence: jvm/E4

### jvm/F5: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.FramePacker.MAGIC`
- Region: heap
- Site: `kotlin/shop/FramePacker.kt:9`
- Reason: not_exercised
- Remedy: No growth column tied to this holder; not refuted by plateau test. Add a driver probe for it and rerun if suspected.
- Supporting evidence: jvm/E4

### jvm/F6: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.Native.INSTANCE`
- Region: heap
- Site: `kotlin/shop/Native.kt:5`
- Reason: not_exercised
- Remedy: No growth column tied to this holder; not refuted by plateau test. Add a driver probe for it and rerun if suspected.
- Supporting evidence: jvm/E4

### jvm/F7: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.<clinit>`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:6`
- Reason: not_exercised
- Remedy: No growth column tied to this holder; not refuted by plateau test. Add a driver probe for it and rerun if suspected.
- Supporting evidence: jvm/E4

### jvm/F8: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.open`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:9`
- Reason: not_exercised
- Remedy: No growth column tied to this holder; not refuted by plateau test. Add a driver probe for it and rerun if suspected.
- Supporting evidence: jvm/E4

### jvm/F9: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.checksum`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:10`
- Reason: not_exercised
- Remedy: No growth column tied to this holder; not refuted by plateau test. Add a driver probe for it and rerun if suspected.
- Supporting evidence: jvm/E4

### jvm/F10: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.size`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:11`
- Reason: not_exercised
- Remedy: No growth column tied to this holder; not refuted by plateau test. Add a driver probe for it and rerun if suspected.
- Supporting evidence: jvm/E4

### jvm/F11: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.lines`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:12`
- Reason: not_exercised
- Remedy: No growth column tied to this holder; not refuted by plateau test. Add a driver probe for it and rerun if suspected.
- Supporting evidence: jvm/E4

### jvm/F12: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.close`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:13`
- Reason: not_exercised
- Remedy: No growth column tied to this holder; not refuted by plateau test. Add a driver probe for it and rerun if suspected.
- Supporting evidence: jvm/E4

### jvm/F13: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.live`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:14`
- Reason: not_exercised
- Remedy: No growth column tied to this holder; not refuted by plateau test. Add a driver probe for it and rerun if suspected.
- Supporting evidence: jvm/E4

### jvm/F14: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.liveBytes`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:15`
- Reason: not_exercised
- Remedy: No growth column tied to this holder; not refuted by plateau test. Add a driver probe for it and rerun if suspected.
- Supporting evidence: jvm/E4

### jvm/F15: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.OrderGenerator.Companion`
- Region: heap
- Site: `kotlin/shop/Order.kt:12`
- Reason: not_exercised
- Remedy: No growth column tied to this holder; not refuted by plateau test. Add a driver probe for it and rerun if suspected.
- Supporting evidence: jvm/E4

### jvm/F16: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.OrderGenerator.skus`
- Region: heap
- Site: `kotlin/shop/Order.kt:14`
- Reason: not_exercised
- Remedy: No growth column tied to this holder; not refuted by plateau test. Add a driver probe for it and rerun if suspected.
- Supporting evidence: jvm/E4

### jvm/F17: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.Order.lines`
- Region: heap
- Site: `kotlin/shop/Order.kt:18`
- Reason: not_exercised
- Remedy: No growth column tied to this holder; not refuted by plateau test. Add a driver probe for it and rerun if suspected.
- Supporting evidence: jvm/E4

### jvm/F18: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.Stats.Companion`
- Region: heap
- Site: `kotlin/shop/Stats.kt:6`
- Reason: not_exercised
- Remedy: No growth column tied to this holder; not refuted by plateau test. Add a driver probe for it and rerun if suspected.
- Supporting evidence: jvm/E4

### rust/F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `native.src.frame.Frame.bytes`
- Region: heap
- Site: `native/src/frame.rs:11`
- Annotation: Frame.bytes is the payload owned by a FRAMES entry; lifetime is that of rust/F2, not an independent holder.
- Reason: not_exercised
- Remedy: cdylib has no driver/main; run the JVM harness with the allocator shim and dla correlate H1 in the JVM worker, then tie native site native/src/lib.rs to the managed finding; fix class: close the handle on the bulk path in Encoder.encode
- Supporting evidence: rust/E2

### rust/F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `native.src.lib.FRAMES`
- Region: heap
- Site: `native/src/lib.rs:19`
- Writers: `native/src/lib.rs:44`
- Removers: `native/src/lib.rs:73`
- Annotation: Static FRAMES map: Native_open inserts a Frame per call, only Native_close removes. Caller kotlin/shop/Encoder.kt:12-15 returns on the bulk path (size > 16384) without Native.close, so those handles stay in FRAMES for process lifetime. Cause location is the Rust-side holder; measure from the JVM worker via dla correlate. Non-leak sibling N-WARM would be refuted by plateau of native.live_blocks.
- Reason: not_exercised
- Remedy: cdylib has no driver/main; run the JVM harness with the allocator shim and dla correlate H1 in the JVM worker, then tie native site native/src/lib.rs to the managed finding; fix class: close the handle on the bulk path in Encoder.encode
- Supporting evidence: rust/E2

## Ruled out

- **jvm/F3.n1** (UNCLASSIFIED): discriminator jvm/E19
  - Ruled out by: plateau+purge (leak) jvm/E19
  - Hypothesis: N-INTENT: audit trail is intended bounded retention (nightly reconciliation)
  - Refuted if: trail grows without bound in proportion to inline frames, no cap or drain
  - Contradicting evidence: jvm/E19
  - Originally supported by: jvm/E15
- **jvm/F19.n1** (UNCLASSIFIED): discriminator jvm/E20
  - Ruled out by: plateau+purge (leak) jvm/E20
  - Hypothesis: N-ALLOC: native growth is allocator retention that plateaus after trim
  - Refuted if: native.live_bytes keeps growing after trim in alloc-test
  - Contradicting evidence: jvm/E20
  - Originally supported by: jvm/E10

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
- jvm/E7: limitation: 39878291 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E7: limitation: 30367347 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E7: limitation: 33029588 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E8: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E8: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E8: limitation: 39878291 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 30367347 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 33029588 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E8: limitation: 40123231 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 30532620 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 41056763 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 34149178 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 31803121 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 40096947 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: dropped 13 per-type columns (cap of 64, lowest growth first)
- jvm/E8: limitation: gate thresholds are provisional (not calibrated)
- jvm/E9: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E9: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E9: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E9: limitation: 39878291 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 30367347 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 33029588 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E9: limitation: 40123231 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 30532620 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 41056763 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 34149178 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 31803121 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 40096947 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: dropped 13 per-type columns (cap of 64, lowest growth first)
- jvm/E9: limitation: gate thresholds are provisional (not calibrated)
- jvm/E10: partial coverage (1 of 1 examined)
- jvm/E10: limitation: the stack is the project frame of the largest allocation stack at the site, not every frame
- jvm/E10: limitation: slopes are per unit of the series axis; the thresholds are provisional
- jvm/E11: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- jvm/E12: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- jvm/E13: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- jvm/E14: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- jvm/E15: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- jvm/E15: limitation: series: per-point; one process per point at n = 2220, 8880, 13320, 19980 (harness captures; checked: HPROF version, identifier size)
- jvm/E16: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- jvm/E17: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- jvm/E17: limitation: the files carry no time, so they are ordered as given
- jvm/E18: limitation: a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them
- jvm/E18: limitation: the files carry no time, so they are ordered as given
- jvm/E18: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- jvm/E19: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E19: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E19: limitation: 39878291 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 30367347 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 33029588 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E19: limitation: 40123231 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 30532620 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 41056763 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 34149178 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 31803121 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 40096947 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: dropped 13 per-type columns (cap of 64, lowest growth first)
- jvm/E19: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E19: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E19: limitation: 22008124 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 29164420 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 31702468 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 34071950 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 34075440 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 30267724 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 31415146 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E19: limitation: 32672872 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E19: limitation: points 1 to 9 come from S1; the extra points from S4
- jvm/E19: limitation: 8 columns beyond the 64 the series allows were dropped
- jvm/E19: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E19: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E19: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E19: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E20: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E20: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E20: limitation: 39878291 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 30367347 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 33029588 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E20: limitation: 40123231 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 30532620 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 41056763 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 34149178 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 31803121 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 40096947 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: dropped 13 per-type columns (cap of 64, lowest growth first)
- jvm/E20: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E20: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E20: limitation: 23463645 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 19642482 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 35609549 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 20756931 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 34153809 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 32141264 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 30649711 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E20: limitation: 29737737 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E20: limitation: points 1 to 9 come from S1; the extra points from S6
- jvm/E20: limitation: 9 columns beyond the 64 the series allows were dropped
- jvm/E20: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E20: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E20: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E20: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E21: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E21: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E21: limitation: 39878291 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 30367347 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 33029588 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E21: limitation: 40123231 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 30532620 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 41056763 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 34149178 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 31803121 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 40096947 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: dropped 13 per-type columns (cap of 64, lowest growth first)
- jvm/E21: limitation: gate thresholds are provisional (not calibrated)
- jvm/E22: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E22: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E22: limitation: 39878291 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 30367347 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 33029588 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E22: limitation: 40123231 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 30532620 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 41056763 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 34149178 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 31803121 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 40096947 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: dropped 13 per-type columns (cap of 64, lowest growth first)
- jvm/E22: limitation: gate thresholds are provisional (not calibrated)
- rust/E2: partial coverage
- rust/E2: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- rust/E2: limitation: writers and removers are textual name matches
- rust/E3: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- rust/E3: limitation: workloads were discovered by reading files; none was run

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
| jvm/E7 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (5c486a891598) | harness/H1/run-S1.json |
| jvm/E8 | growth-series | test | dla 0.1.0 | artifact series/S1.json (478c5518504f) | trend/T1.json |
| jvm/E9 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (478c5518504f) | trend/T1.json |
| jvm/E10 | alloc-stack | test | dla-correlate 0.1.0 | artifact series/S1.json (478c5518504f) | correlate/C1.json |
| jvm/E11 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261109-203606-0633/harness/H1/out/p1-native/capture/9/heap-1.hprof (a3768289d899) | summaries/heap-1-a3768289.json |
| jvm/E12 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261109-203606-0633/harness/H1/out/p4-native/capture/388/heap-1.hprof (61edf9ff6018) | summaries/heap-1-61edf9ff.json |
| jvm/E13 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261109-203606-0633/harness/H1/out/p6-native/capture/615/heap-1.hprof (be8cef1044e9) | summaries/heap-1-be8cef10.json |
| jvm/E14 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261109-203606-0633/harness/H1/out/p9-native/capture/954/heap-1.hprof (0de7e4b9e3bd) | summaries/heap-1-0de7e4b9.json |
| jvm/E15 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S2.json (84d5de31b9d0) | series/S2.json |
| jvm/E16 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S2.json (84d5de31b9d0) | series/S2.json |
| jvm/E17 | histogram-diff | live | jcmd-class-histogram 0.1.0 | artifact histograms/H1.json (c68fcd61cf9f) | histograms/H1.json |
| jvm/E18 | growth-series | live | jcmd-class-histogram 0.1.0 | artifact series/S3.json (4d784a760266) | series/S3.json |
| jvm/E19 | discriminator | test | dla 0.1.0 | artifact series/S5.json (68345824905b) | alloc/A1.json |
| jvm/E20 | discriminator | test | dla 0.1.0 | artifact series/S7.json (e37e3bf59f4b) | alloc/A2.json |
| jvm/E21 | growth-series | test | dla 0.1.0 | artifact series/S1.json (478c5518504f) | trend/T4.json |
| jvm/E22 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (478c5518504f) | trend/T4.json |
| rust/E1 | static-hit | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/rust-dla.json |
| rust/E2 | census-holder | static | dla-census 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | census/rust.json |
| rust/E3 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | workload/rust.json |
