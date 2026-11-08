# DLA report

Run: `20261108-202005-bcf9`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 21 enumerated, 21 dispositioned; confirmed 1, strong 0, refuted 6, suspected 4, unresolved 10

Residual: heap 1447.2411411411413 bytes/n unattributed of 1447.2411411411413 total (threshold 361.8102852852853) ABOVE THRESHOLD, see jvm/O1; off-heap 0 bytes/n unattributed of -1880.765336765337 total (threshold 3628.139124633204); non-heap 7.058258258258258 bytes/n unattributed of 7.058258258258258 total (threshold 7.771724324324236)

Coverage: jvm 17760 iterations, 0 of 17 entry points exercised; rust no iterations, 0 of 10 entry points exercised

## Workers

- **jvm** (jvm): status done; 70 lines merged; slice done; torn tail: no
- **rust** (rust): status done; 13 lines merged; slice done; torn tail: no

## Findings

### jvm/F3: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:shop.codec.Audit.trail` 2.625 count/n (steady, gate pass, jvm/E9)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `shop.codec.Audit.trail`
- Region: heap
- Shape: steady
- Site: `kotlin/shop/Audit.kt:9`
- Writers: `kotlin/shop/Audit.kt:12`
- Annotation: heuristic name match over the project's sources
- Annotation: Audit.trail is an append-only process-lifetime list; Encoder.kt:68 adds one 1536-byte sample per inline order, no remover
- Annotation: Audit.trail (static ArrayList) retains a 1536-byte copy per inline order (Audit.kt:12, called from Encoder.kt:68); nothing ever removes entries (missing eviction/bound for the reconciliation job). Heap probe grows linearly (~1.4 KB/order); no taxonomy label available, left UNCLASSIFIED.
- Sibling hypotheses: jvm/F3.n1
- Supporting evidence: jvm/E4, jvm/E7, jvm/E9

### jvm/F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Site: `kotlin/shop/Audit.kt:9`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: same site and holder as confirmed jvm/F3 (Audit.kt:9); detector duplicate, merge dedupes
- Supporting evidence: jvm/E3

### jvm/F8: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.open`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:9`
- Writers: `kotlin/shop/Encoder.kt:60`
- Annotation: Native.open handle is closed only on inline path (Encoder.kt:69); bulk path returns at Encoder.kt:66 without Native.close
- Annotation: Observed in a project replay (./build.sh run 8000): nativeLive rises 1 -> 251 -> 501 -> 751 -> 1000, nativeKB 24 -> 18035 at i=6000 (~24 KB per bulk frame, one per 8 orders); matches Encoder.kt bulk path returning without Native.close. Not a DLA harness measurement, so left suspected_static_only.
- Reason: not_exercised
- Remedy: Encoder.kt:63-67 returns on the bulk path (size>16384, every 8th order, 384 lines) without Native.close; harness has no column for Native.live()/liveBytes (printed by Stats.checkpoint only). Rust worker should confirm via native live-handle count; or add a probe of Native.live per point
- Supporting evidence: jvm/E4

### rust/F1: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `native.src.frame.Frame.bytes`
- Region: heap
- Site: `native/src/frame.rs:11`
- Writers: `native/src/lib.rs:43`
- Removers: `native/src/lib.rs:68`
- Annotation: x
- Annotation: Frame.bytes is owned by the FRAMES map entry (see rust/F2); retained only because the entry is never removed on the Kotlin bulk path (Encoder.kt:14-15). Same root cause as rust/F2, not an independent holder.
- Reason: no_harness
- Remedy: Exercise via the JVM replay (./build.sh run N, watch Native.live()/nativeLive grow with bulk frames) in the jvm worker's harness, or add a Rust bin/driver calling open without close; fix class: close the handle on the bulk path (try/finally) in Encoder.kt
- Supporting evidence: rust/E3

### rust/F2: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: static/global
- Owner: `native.src.lib.FRAMES`
- Region: heap
- Site: `native/src/lib.rs:19`
- Writers: `native/src/lib.rs:43`
- Removers: `native/src/lib.rs:68`
- Annotation: FRAMES (static Mutex<HashMap<i64,Frame>>) gets an entry per Native.open and drops it only on Native.close. Rust side is balanced; the missing close is the caller: kotlin/shop/Encoder.kt:14-15 returns on the size>16384 'bulk' path without Native.close(handle), so each bulk frame stays in FRAMES with its Vec<u8> (frame.rs:11). Static reading only; no rust harness (cdylib, no bin/iteration arg). Non-leak sibling N-WARM/N-INTENT would be refuted if native live count returns to 0 after bulk frames.
- Reason: no_harness
- Remedy: Exercise via the JVM replay (./build.sh run N, watch Native.live()/nativeLive grow with bulk frames) in the jvm worker's harness, or add a Rust bin/driver calling open without close; fix class: close the handle on the bulk path (try/finally) in Encoder.kt
- Supporting evidence: rust/E3

### jvm/F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.Entry.sample`
- Region: heap
- Site: `kotlin/shop/Audit.kt:3`
- Reason: not_exercised
- Remedy: Entry.sample is the payload retained through confirmed jvm/F3 (Audit.trail); no own probe column; fix lives at the trail bound
- Supporting evidence: jvm/E4

### jvm/F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.<clinit>`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:6`
- Reason: not_exercised
- Remedy: JNI binding declaration, not an independent holder; native handle leak is tracked under jvm/F8 and the rust worker
- Supporting evidence: jvm/E4

### jvm/F9: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.checksum`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:10`
- Reason: not_exercised
- Remedy: JNI binding declaration, not an independent holder; native handle leak is tracked under jvm/F8 and the rust worker
- Supporting evidence: jvm/E4

### jvm/F10: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.size`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:11`
- Reason: not_exercised
- Remedy: JNI binding declaration, not an independent holder; native handle leak is tracked under jvm/F8 and the rust worker
- Supporting evidence: jvm/E4

### jvm/F11: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.lines`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:12`
- Reason: not_exercised
- Remedy: JNI binding declaration, not an independent holder; native handle leak is tracked under jvm/F8 and the rust worker
- Supporting evidence: jvm/E4

### jvm/F12: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.close`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:13`
- Reason: not_exercised
- Remedy: JNI binding declaration, not an independent holder; native handle leak is tracked under jvm/F8 and the rust worker
- Supporting evidence: jvm/E4

### jvm/F13: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.live`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:14`
- Reason: not_exercised
- Remedy: JNI binding declaration, not an independent holder; native handle leak is tracked under jvm/F8 and the rust worker
- Supporting evidence: jvm/E4

### jvm/F14: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.liveBytes`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:15`
- Reason: not_exercised
- Remedy: JNI binding declaration, not an independent holder; native handle leak is tracked under jvm/F8 and the rust worker
- Supporting evidence: jvm/E4

### jvm/F16: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.OrderGenerator.skus`
- Region: heap
- Site: `kotlin/shop/Order.kt:14`
- Reason: not_exercised
- Remedy: no probe column for this instance field (alloc-test basis was another holder); inspect with a heap dump series if heap growth beyond jvm/F3 remains
- Supporting evidence: jvm/E4

### jvm/F17: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.Order.lines`
- Region: heap
- Site: `kotlin/shop/Order.kt:18`
- Reason: not_exercised
- Remedy: no probe column for this instance field (alloc-test basis was another holder); inspect with a heap dump series if heap growth beyond jvm/F3 remains
- Supporting evidence: jvm/E4

## Ruled out

- **jvm/F3.n1** (UNCLASSIFIED): discriminator jvm/E10
  - Ruled out by: plateau+purge (leak) jvm/E10
  - Hypothesis: N-WARM: Audit.trail is a warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles (dump-plateau/alloc-test favors leak)
  - Contradicting evidence: jvm/E10
  - Originally supported by: jvm/E9
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
- jvm/E7: partial coverage (9 of 9 examined)
- jvm/E7: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E8: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E8: limitation: gate thresholds are provisional (not calibrated)
- jvm/E9: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E9: limitation: gate thresholds are provisional (not calibrated)
- jvm/E10: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E10: limitation: points 1 to 9 come from S1; the extra points from S2
- jvm/E10: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E10: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E10: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E11: limitation: gate thresholds are provisional (not calibrated)
- jvm/E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E12: limitation: gate thresholds are provisional (not calibrated)
- jvm/E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E13: limitation: gap.unexplained is negative at 7 point(s): the accounted figure exceeds os.rss (reserved but untouched memory)
- jvm/E13: limitation: gate thresholds are provisional (not calibrated)
- jvm/E14: partial coverage (3 of 3 examined)
- jvm/E14: limitation: jvm/F3: growth measured as a count; bytes not attributed
- jvm/E14: limitation: threshold is provisional (not calibrated)
- jvm/E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E15: limitation: points 1 to 9 come from S1; the extra points from S5
- jvm/E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E15: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E16: limitation: points 1 to 9 come from S1; the extra points from S7
- jvm/E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E16: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E17: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E17: limitation: points 1 to 9 come from S1; the extra points from S9
- jvm/E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E17: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E17: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E18: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E18: limitation: points 1 to 9 come from S1; the extra points from S11
- jvm/E18: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E18: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E18: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E19: limitation: points 1 to 9 come from S1; the extra points from S13
- jvm/E19: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E19: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E19: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E20: limitation: points 1 to 9 come from S1; the extra points from S15
- jvm/E20: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E20: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E20: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E21: limitation: points 1 to 9 come from S1; the extra points from S17
- jvm/E21: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E21: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E21: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E22: limitation: points 1 to 9 come from S1; the extra points from S19
- jvm/E22: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E22: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E22: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E23: partial coverage (3 of 3 examined)
- jvm/E23: limitation: jvm/F3: growth measured as a count; bytes not attributed
- jvm/E23: limitation: threshold is provisional (not calibrated)
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
| jvm/E7 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (c83afcc10216) | harness/H1/run-S1.json |
| jvm/E8 | growth-series | test | dla 0.1.0 | artifact series/S1.json (b80eaafdfeca) | trend/T1.json |
| jvm/E9 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (b80eaafdfeca) | trend/T1.json |
| jvm/E10 | discriminator | test | dla 0.1.0 | artifact series/S3.json (447789eb72bb) | alloc/A1.json |
| jvm/E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (b80eaafdfeca) | trend/T3.json |
| jvm/E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (b80eaafdfeca) | trend/T3.json |
| jvm/E13 | non-heap | test | dla 0.1.0 | artifact series/S4.json (7f08f4f5b6d5) | trend/T4.json |
| jvm/E14 | residual | test | dla 0.1.0 | artifact series/S1.json (b80eaafdfeca) | gap/residual-S1-jvm_E14.json |
| jvm/E15 | discriminator | test | dla 0.1.0 | artifact series/S6.json (4636d534ae3e) | alloc/A2.json |
| jvm/E16 | discriminator | test | dla 0.1.0 | artifact series/S8.json (7afff25af0b8) | alloc/A3.json |
| jvm/E17 | discriminator | test | dla 0.1.0 | artifact series/S10.json (0193912ace55) | alloc/A4.json |
| jvm/E18 | discriminator | test | dla 0.1.0 | artifact series/S12.json (9c538050314d) | alloc/A5.json |
| jvm/E19 | discriminator | test | dla 0.1.0 | artifact series/S14.json (c70f3d16dc03) | alloc/A6.json |
| jvm/E20 | discriminator | test | dla 0.1.0 | artifact series/S16.json (2fb33c094057) | alloc/A7.json |
| jvm/E21 | discriminator | test | dla 0.1.0 | artifact series/S18.json (6fd8197f3376) | alloc/A8.json |
| jvm/E22 | discriminator | test | dla 0.1.0 | artifact series/S20.json (71f51c6cf2e4) | alloc/A9.json |
| jvm/E23 | residual | test | dla 0.1.0 | artifact series/S1.json (b80eaafdfeca) | gap/residual-S1-jvm_E23.json |
| rust/E1 | static-hit | static | rapx 0.7.71 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/rust-rapx.json |
| rust/E2 | static-hit | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/rust-dla.json |
| rust/E3 | census-holder | static | dla-census 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | census/rust.json |
| rust/E4 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | workload/rust.json |
