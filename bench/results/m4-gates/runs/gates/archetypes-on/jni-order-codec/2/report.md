# DLA report

Run: `20261108-195205-33ca`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 23 enumerated, 23 dispositioned; confirmed 3, strong 0, refuted 8, suspected 12, unresolved 0

Residual: not measured

Coverage: jvm 17760 iterations, 0 of 17 entry points exercised; rust no iterations, 0 of 10 entry points exercised

## Workers

- **jvm** (jvm): status done; 73 lines merged; slice done; torn tail: no
- **rust** (rust): status done; 12 lines merged; slice done; torn tail: no

## Findings

### jvm/F1: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:shop.codec.Audit.trail` 2.625 count/n (steady, gate pass, jvm/E11)
- Evidence classes: context, growth, static
- Holder: static/global
- Region: heap
- Shape: steady
- Site: `kotlin/shop/Audit.kt:9`
- Annotation: Same holder as jvm/F3: Audit.trail unbounded ArrayList (Audit.kt:9), detector hit confirmed by the same growth.
- Sibling hypotheses: jvm/F1.n1
- Suggested archetype (unconfirmed): L-COLLECT
- Supporting evidence: jvm/E3, jvm/E9, jvm/E11

### jvm/F3: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:shop.codec.Audit.trail` 2.625 count/n (steady, gate pass, jvm/E11)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `shop.codec.Audit.trail`
- Region: heap
- Shape: steady
- Site: `kotlin/shop/Audit.kt:9`
- Writers: `kotlin/shop/Audit.kt:12`
- Annotation: heuristic name match over the project's sources
- Annotation: Audit.trail (static ArrayList, Audit.kt:9) is appended per inline-lane encode (Audit.record, called from Encoder.kt:20) with a 1536-byte copy and no remover; grows unbounded with orders processed. Missing pairing: bounded retention/eviction after reconciliation. Class label not assigned (unlabelled).
- Sibling hypotheses: jvm/F3.n1
- Supporting evidence: jvm/E4, jvm/E9, jvm/E11

### jvm/F8: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: not tied to this finding (passing columns of jvm/E13: `os.rss`, `os.rss_anon`, `os.vm_hwm`)
- Evidence classes: context, growth, static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.open`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:9`
- Writers: `kotlin/shop/Encoder.kt:11`
- Removers: `kotlin/shop/Encoder.kt:19`
- Annotation: Native.open (Encoder.kt:11) allocates a native frame handle; Native.close (Encoder.kt:19) is reached only on the inline lane. The bulk lane (size > 16384, wholesale orders, every 8th: 24592-byte frames) returns at Encoder.kt:14-16 without closing, so each bulk encode leaks one native handle (harness log: nativeLive == bulk count, nativeKB ~12/order-avg growth; trim-native-heap purge retained). Missing pairing: close on every exit path of encode (try/finally). Holder: native handle table in libcodec keyed by a Long held by no JVM object. Unlabelled.
- Sibling hypotheses: jvm/F8.n1
- Supporting evidence: jvm/E4, jvm/E9, jvm/E13

### jvm/F2: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.Entry.sample`
- Region: heap
- Site: `kotlin/shop/Audit.kt:3`
- Reason: no_harness
- Remedy: Entry.sample bytes are the payload of jvm/F3 (Audit.trail); growth is covered by the confirmed F3, no separate probe for the field
- Supporting evidence: jvm/E4

### jvm/F7: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.<clinit>`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:6`
- Reason: no_harness
- Remedy: JNI binding declaration only (census native-method); the leak is the unpaired open/close on the bulk path, recorded in jvm/F8; no separate growth column
- Supporting evidence: jvm/E4

### jvm/F9: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.checksum`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:10`
- Reason: no_harness
- Remedy: JNI binding declaration only (census native-method); the leak is the unpaired open/close on the bulk path, recorded in jvm/F8; no separate growth column
- Supporting evidence: jvm/E4

### jvm/F10: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.size`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:11`
- Reason: no_harness
- Remedy: JNI binding declaration only (census native-method); the leak is the unpaired open/close on the bulk path, recorded in jvm/F8; no separate growth column
- Supporting evidence: jvm/E4

### jvm/F11: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.lines`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:12`
- Reason: no_harness
- Remedy: JNI binding declaration only (census native-method); the leak is the unpaired open/close on the bulk path, recorded in jvm/F8; no separate growth column
- Supporting evidence: jvm/E4

### jvm/F12: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.close`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:13`
- Reason: no_harness
- Remedy: JNI binding declaration only (census native-method); the leak is the unpaired open/close on the bulk path, recorded in jvm/F8; no separate growth column
- Supporting evidence: jvm/E4

### jvm/F13: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.live`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:14`
- Reason: no_harness
- Remedy: JNI binding declaration only (census native-method); the leak is the unpaired open/close on the bulk path, recorded in jvm/F8; no separate growth column
- Supporting evidence: jvm/E4

### jvm/F14: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.liveBytes`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:15`
- Reason: no_harness
- Remedy: JNI binding declaration only (census native-method); the leak is the unpaired open/close on the bulk path, recorded in jvm/F8; no separate growth column
- Supporting evidence: jvm/E4

### jvm/F16: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.OrderGenerator.skus`
- Region: heap
- Site: `kotlin/shop/Order.kt:14`
- Reason: no_harness
- Remedy: no probe column was derived for this holder (fixed-size 512-entry array / per-order list, no cross-order writer); add a driver-mode probe to confirm it does not grow. The whole-process rss verdict is explained by jvm/F3 and jvm/F8
- Supporting evidence: jvm/E4

### jvm/F17: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.Order.lines`
- Region: heap
- Site: `kotlin/shop/Order.kt:18`
- Reason: no_harness
- Remedy: no probe column was derived for this holder (fixed-size 512-entry array / per-order list, no cross-order writer); add a driver-mode probe to confirm it does not grow. The whole-process rss verdict is explained by jvm/F3 and jvm/F8
- Supporting evidence: jvm/E4

### rust/F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `native.src.frame.Frame.bytes`
- Region: heap
- Site: `native/src/frame.rs:11`
- Annotation: Frame.bytes is the per-entry payload owned by FRAMES (see rust/F2); it is retained only because its map entry is never removed on the bulk path. Not an independent holder.
- Reason: not_exercised
- Remedy: Same as rust/F2: payload retained via FRAMES entries; exercise through the JVM replay driver.
- Supporting evidence: rust/E3

### rust/F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `native.src.lib.FRAMES`
- Region: heap
- Site: `native/src/lib.rs:19`
- Writers: `native/src/lib.rs:42`
- Removers: `native/src/lib.rs:73`
- Annotation: FRAMES (static HashMap<i64,Frame>) is filled by Native.open and emptied only by Native.close. The Rust side removes correctly; the caller kotlin/shop/Encoder.kt:14-18 returns early for frames over MAX_INLINE_BYTES (16384) without calling Native.close, so those handles (and their Vec<u8>) stay in FRAMES forever. Missing pairing: close on the bulk path. Not exercised by DLA in the rust worker (cdylib, no Rust driver; JVM driver Main.kt belongs to the jvm worker).
- Reason: not_exercised
- Remedy: Run the Kotlin replay (kotlin/shop/Main.kt, build.sh run) as a loop and check Native.live()/liveBytes() growth for orders over 16384 bytes; confirm via jvm worker harness. Repair class: pair every Native.open with Native.close on the bulk path.
- Supporting evidence: rust/E3

## Ruled out

- **jvm/F1.n1** (UNCLASSIFIED): discriminator jvm/E22
  - Ruled out by: plateau+purge (leak) jvm/E22
  - Hypothesis: N-WARM: Audit.trail growth is a warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: jvm/E22
  - Originally supported by: jvm/E11
- **jvm/F3.n1** (UNCLASSIFIED): discriminator jvm/E12
  - Ruled out by: plateau+purge (leak) jvm/E12
  - Hypothesis: N-WARM: Audit.trail growth is a warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles of alloc-test
  - Contradicting evidence: jvm/E12
  - Originally supported by: jvm/E11
- **jvm/F4** (UNCLASSIFIED): discriminator jvm/E16
  - Ruled out by: plateau+purge (non-leak) jvm/E16
  - Contradicting evidence: jvm/E16
  - Originally supported by: jvm/E4
- **jvm/F5** (UNCLASSIFIED): discriminator jvm/E15
  - Ruled out by: plateau+purge (non-leak) jvm/E15
  - Contradicting evidence: jvm/E15
  - Originally supported by: jvm/E4
- **jvm/F6** (UNCLASSIFIED): discriminator jvm/E17
  - Ruled out by: plateau+purge (non-leak) jvm/E17
  - Contradicting evidence: jvm/E17
  - Originally supported by: jvm/E4
- **jvm/F8.n1** (UNCLASSIFIED): discriminator jvm/E14
  - Ruled out by: plateau+purge (leak) jvm/E14
  - Hypothesis: N-ALLOC: RSS growth is allocator retention of freed native frames, not live handles
  - Refuted if: alloc-test with trim-native-heap shows retained growth / Native.live grows with bulk frames
  - Contradicting evidence: jvm/E14
  - Originally supported by: jvm/E13
- **jvm/F15** (UNCLASSIFIED): discriminator jvm/E18
  - Ruled out by: plateau+purge (non-leak) jvm/E18
  - Contradicting evidence: jvm/E18
  - Originally supported by: jvm/E4
- **jvm/F18** (UNCLASSIFIED): discriminator jvm/E21
  - Ruled out by: plateau+purge (non-leak) jvm/E21
  - Contradicting evidence: jvm/E21
  - Originally supported by: jvm/E4

## Blind spots and unknowns

### Coverage gaps

- jvm/codeql: missing (codeql is not on PATH or in the tools directory). Install hint: dla install codeql
- jvm/errorprone: missing (Error Prone jars not found in the tools directory (or DLA_ERRORPRONE_HOME)). Install hint: dla install errorprone
- jvm/sonar: missing (sonar-scanner is not installed and Sonar rules need a SonarQube or SonarCloud server). Install hint: install sonar-scanner, run a SonarQube or SonarCloud server and set SONAR_HOST_URL and SONAR_TOKEN (dla install sonar prints the steps)

### Runtime blind spots

None recorded.

### Entry points not exercisable

- jvm: `kotlin/shop/Audit.kt::Audit.record`: no_harness
- jvm: `kotlin/shop/Audit.kt::Audit.size`: no_harness
- jvm: `kotlin/shop/Encoder.kt::OrderEncoder.encode`: no_harness
- jvm: `kotlin/shop/FramePacker.kt::FramePacker.frameSize`: no_harness
- jvm: `kotlin/shop/FramePacker.kt::FramePacker.pack`: no_harness
- jvm: `kotlin/shop/Native.kt::Native.checksum`: no_harness
- jvm: `kotlin/shop/Native.kt::Native.close`: no_harness
- jvm: `kotlin/shop/Native.kt::Native.lines`: no_harness
- jvm: `kotlin/shop/Native.kt::Native.live`: no_harness
- jvm: `kotlin/shop/Native.kt::Native.liveBytes`: no_harness
- jvm: `kotlin/shop/Native.kt::Native.open`: no_harness
- jvm: `kotlin/shop/Native.kt::Native.size`: no_harness
- jvm: `kotlin/shop/Order.kt::Order.totalCents`: no_harness
- jvm: `kotlin/shop/Order.kt::OrderGenerator.next`: no_harness
- jvm: `kotlin/shop/Stats.kt::Stats.checkpoint`: no_harness
- jvm: `kotlin/shop/Stats.kt::Stats.rssKb`: no_harness
- jvm: `kotlin/shop/Stats.kt::Stats.summary`: no_harness
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
- jvm/E6: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E7: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- jvm/E7: limitation: workloads were discovered by reading files; none was run
- jvm/E8: limitation: harness written; nothing was run (dla harness run measures it)
- jvm/E8: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E8: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E9: partial coverage (9 of 9 examined)
- jvm/E9: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E10: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E10: limitation: gate thresholds are provisional (not calibrated)
- jvm/E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E11: limitation: gate thresholds are provisional (not calibrated)
- jvm/E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E12: limitation: points 1 to 9 come from S1; the extra points from S2
- jvm/E12: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E12: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E12: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E13: partial coverage (6 of 28 examined)
- jvm/E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E13: limitation: gate thresholds are provisional (not calibrated)
- jvm/E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E14: limitation: points 1 to 9 come from S1; the extra points from S4
- jvm/E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E14: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E15: limitation: points 1 to 9 come from S1; the extra points from S6
- jvm/E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E15: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E16: limitation: points 1 to 9 come from S1; the extra points from S8
- jvm/E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E16: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E17: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E17: limitation: points 1 to 9 come from S1; the extra points from S10
- jvm/E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E17: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E17: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E18: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E18: limitation: points 1 to 9 come from S1; the extra points from S12
- jvm/E18: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E18: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E18: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E19: limitation: points 1 to 9 come from S1; the extra points from S14
- jvm/E19: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E19: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E19: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E20: limitation: points 1 to 9 come from S1; the extra points from S16
- jvm/E20: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E20: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E20: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E21: limitation: points 1 to 9 come from S1; the extra points from S18
- jvm/E21: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E21: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E21: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E22: limitation: points 1 to 9 come from S1; the extra points from S20
- jvm/E22: limitation: scratch copy: 23 path(s) not copied (not a regular file)
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
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E3 | — |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E3 | — |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E3 | — |
| `intake.build` | Does the project build locally? | answered | E3 | — |
| `intake.workload` | What can exercise the suspected path? | answered | E3 | — |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E3 | — |

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
| jvm/E7 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | workload/jvm.json |
| jvm/E8 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | harness/H2/harness.json |
| jvm/E9 | workload | test | dla 0.1.0 | artifact harness/H2/harness.json (ecafdc9ef7d7) | harness/H2/run-S1.json |
| jvm/E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (83a3518325ef) | trend/T1.json |
| jvm/E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (83a3518325ef) | trend/T1.json |
| jvm/E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (4594cee709fa) | alloc/A1.json |
| jvm/E13 | growth-series | test | dla 0.1.0 | artifact series/S1.json (83a3518325ef) | trend/T3.json |
| jvm/E14 | discriminator | test | dla 0.1.0 | artifact series/S5.json (1d2f653249e4) | alloc/A2.json |
| jvm/E15 | discriminator | test | dla 0.1.0 | artifact series/S7.json (483540652ff7) | alloc/A3.json |
| jvm/E16 | discriminator | test | dla 0.1.0 | artifact series/S9.json (cb82b1b05f50) | alloc/A4.json |
| jvm/E17 | discriminator | test | dla 0.1.0 | artifact series/S11.json (5bb0b65765ad) | alloc/A5.json |
| jvm/E18 | discriminator | test | dla 0.1.0 | artifact series/S13.json (c41841bdc0b6) | alloc/A6.json |
| jvm/E19 | discriminator | test | dla 0.1.0 | artifact series/S15.json (31d2ee7d6061) | alloc/A7.json |
| jvm/E20 | discriminator | test | dla 0.1.0 | artifact series/S17.json (b6e183071e16) | alloc/A8.json |
| jvm/E21 | discriminator | test | dla 0.1.0 | artifact series/S19.json (3d7680181ffb) | alloc/A9.json |
| jvm/E22 | discriminator | test | dla 0.1.0 | artifact series/S21.json (74a188d1d306) | alloc/A10.json |
| rust/E1 | static-hit | static | rapx 0.7.71 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/rust-rapx.json |
| rust/E2 | static-hit | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/rust-dla.json |
| rust/E3 | census-holder | static | dla-census 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | census/rust.json |
| rust/E4 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | workload/rust.json |
