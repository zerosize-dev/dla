# DLA report

Run: `20261108-192926-aa2e`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 22 enumerated, 22 dispositioned; confirmed 2, strong 0, refuted 3, suspected 17, unresolved 0

Residual: heap 1450.1855855855856 bytes/n unattributed of 1450.1855855855856 total (threshold 362.5463963963964) ABOVE THRESHOLD, see jvm/O1; off-heap 0 bytes/n unattributed of -404.93247533247535 total (threshold 5252.435886949807); non-heap 3.4372372372372375 bytes/n unattributed of 3.4372372372372375 total (threshold 10.99890756756739)

Coverage: jvm 17760 iterations, 0 of 17 entry points exercised; rust no iterations, 0 of 10 entry points exercised

## Workers

- **jvm** (jvm): status done; 70 lines merged; slice done; torn tail: no
- **rust** (rust): status done; 12 lines merged; slice done; torn tail: no

## Findings

### jvm/F1: **Novel mechanism (UNCLASSIFIED)**

- State: confirmed
- Growth: `probe:shop.codec.Audit.trail` 2.625 count/n (steady, gate pass, jvm/E9)
- Evidence classes: context, growth, static
- Holder: static/global
- Region: heap
- Shape: steady
- Site: `kotlin/shop/Audit.kt:9`
- Annotation: Same site and mechanism as jvm/F3 (Audit.trail unbounded append); detector hit duplicate.
- Sibling hypotheses: jvm/F1.n1
- Suggested archetype (unconfirmed): L-COLLECT
- Supporting evidence: jvm/E3, jvm/E7, jvm/E9

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
- Annotation: Audit.trail only added to by record(); no remover anywhere
- Annotation: Static Audit.trail (ArrayList<Entry>) is appended per inline frame in OrderEncoder.encode (Audit.record copies 1536 bytes) and never trimmed; unbounded retention, no matching removal. Heap-side leak.
- Sibling hypotheses: jvm/F3.n1
- Supporting evidence: jvm/E4, jvm/E7, jvm/E9

### jvm/F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.Entry.sample`
- Region: heap
- Site: `kotlin/shop/Audit.kt:3`
- Reason: not_exercised
- Remedy: contents of Audit.trail (covered by confirmed jvm/F3); Entry.count column grew but is the same leak
- Supporting evidence: jvm/E4

### jvm/F4: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.OrderEncoder.Companion`
- Region: heap
- Site: `kotlin/shop/Encoder.kt:6`
- Reason: not_exercised
- Remedy: no own probe column grew; census holder is a constant, accessor or per-call object. Add a driver-mode probe to refute with a non-leak plateau
- Supporting evidence: jvm/E4

### jvm/F6: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.Native.INSTANCE`
- Region: heap
- Site: `kotlin/shop/Native.kt:5`
- Reason: not_exercised
- Remedy: no own probe column grew; census holder is a constant, accessor or per-call object. Add a driver-mode probe to refute with a non-leak plateau
- Supporting evidence: jvm/E4

### jvm/F7: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.<clinit>`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:6`
- Reason: not_exercised
- Remedy: no own probe column grew; census holder is a constant, accessor or per-call object. Add a driver-mode probe to refute with a non-leak plateau
- Supporting evidence: jvm/E4

### jvm/F8: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.open`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:9`
- Reason: not_exercised
- Remedy: harness has no native-handle column; add a Native.live() probe or driver-mode checkpoint, rerun trend; direct replay shows Native.live() == bulk frames (leak at Encoder.kt:16-19 missing Native.close)
- Supporting evidence: jvm/E4

### jvm/F9: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.checksum`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:10`
- Reason: not_exercised
- Remedy: no own probe column grew; census holder is a constant, accessor or per-call object. Add a driver-mode probe to refute with a non-leak plateau
- Supporting evidence: jvm/E4

### jvm/F10: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.size`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:11`
- Reason: not_exercised
- Remedy: no own probe column grew; census holder is a constant, accessor or per-call object. Add a driver-mode probe to refute with a non-leak plateau
- Supporting evidence: jvm/E4

### jvm/F11: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.lines`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:12`
- Reason: not_exercised
- Remedy: no own probe column grew; census holder is a constant, accessor or per-call object. Add a driver-mode probe to refute with a non-leak plateau
- Supporting evidence: jvm/E4

### jvm/F12: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.close`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:13`
- Writers: `kotlin/shop/Encoder.kt:13`
- Annotation: Bulk path in OrderEncoder.encode (Encoder.kt:16-19) returns without Native.close(handle); direct run showed nativeLive == bulk count (1000 of 8000 orders, nativeKB 6027 at i=2000 growing linearly). Holder is the Rust FRAMES map (native/src/lib.rs:20); rust worker owns that side. No DLA probe column for native live handles, so not harness-confirmed.
- Reason: not_exercised
- Remedy: harness has no native-handle column; add a Native.live() probe or driver-mode checkpoint, rerun trend; direct replay shows Native.live() == bulk frames (leak at Encoder.kt:16-19 missing Native.close)
- Supporting evidence: jvm/E4

### jvm/F13: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.live`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:14`
- Reason: not_exercised
- Remedy: no own probe column grew; census holder is a constant, accessor or per-call object. Add a driver-mode probe to refute with a non-leak plateau
- Supporting evidence: jvm/E4

### jvm/F14: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.liveBytes`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:15`
- Reason: not_exercised
- Remedy: no own probe column grew; census holder is a constant, accessor or per-call object. Add a driver-mode probe to refute with a non-leak plateau
- Supporting evidence: jvm/E4

### jvm/F15: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.OrderGenerator.Companion`
- Region: heap
- Site: `kotlin/shop/Order.kt:12`
- Reason: not_exercised
- Remedy: no own probe column grew; census holder is a constant, accessor or per-call object. Add a driver-mode probe to refute with a non-leak plateau
- Supporting evidence: jvm/E4

### jvm/F16: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.OrderGenerator.skus`
- Region: heap
- Site: `kotlin/shop/Order.kt:14`
- Reason: not_exercised
- Remedy: no own probe column grew; census holder is a constant, accessor or per-call object. Add a driver-mode probe to refute with a non-leak plateau
- Supporting evidence: jvm/E4

### jvm/F17: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.Order.lines`
- Region: heap
- Site: `kotlin/shop/Order.kt:18`
- Reason: not_exercised
- Remedy: no own probe column grew; census holder is a constant, accessor or per-call object. Add a driver-mode probe to refute with a non-leak plateau
- Supporting evidence: jvm/E4

### jvm/F18: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.Stats.Companion`
- Region: heap
- Site: `kotlin/shop/Stats.kt:6`
- Reason: not_exercised
- Remedy: no own probe column grew; census holder is a constant, accessor or per-call object. Add a driver-mode probe to refute with a non-leak plateau
- Supporting evidence: jvm/E4

### rust/F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `native.src.frame.Frame.bytes`
- Region: heap
- Site: `native/src/frame.rs:11`
- Writers: `native/src/lib.rs:44`
- Removers: `native/src/lib.rs:73`
- Annotation: Frame.bytes is owned by the FRAMES entry (see rust/F2) and is freed when the entry is removed. It has no independent lifetime: any retention comes from F2.
- Reason: not_exercised
- Remedy: Same JVM-driven replay as rust/F2; F1 is the payload of F2 entries and is not an independent holder.
- Supporting evidence: rust/E3

### rust/F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `native.src.lib.FRAMES`
- Region: heap
- Site: `native/src/lib.rs:19`
- Writers: `native/src/lib.rs:44`
- Removers: `native/src/lib.rs:73`
- Annotation: FRAMES is a process-lifetime static HashMap<i64,Frame>. open() inserts every frame (lib.rs:44); the only remover is close() (lib.rs:73), which is called by the JVM. kotlin/shop/Encoder.kt:11-14 returns early for frames over MAX_INLINE_BYTES (16384, more than 255 lines) without calling Native.close, so bulk frames stay in FRAMES forever. Missing pairing: close for every open on the bulk-lane path. Not exercised by DLA: the cdylib needs a JVM, and the rust harness has no lib or bin target to drive. Class of repair: release the handle on the bulk path too. The jvm worker owns the Kotlin side.
- Reason: not_exercised
- Remedy: Run a JVM-driven replay (./build.sh run N, with N large enough to produce frames over 16384 bytes) and trend Native.live()/liveBytes() (nativeLive column in Stats.checkpoint) with a non-leak sibling N-WARM; the rust harness cannot load a cdylib without a JVM. Expected: nativeLive grows with the count of bulk frames and drops to 0 only if close is called on the bulk path.
- Supporting evidence: rust/E3

## Ruled out

- **jvm/F1.n1** (UNCLASSIFIED): discriminator jvm/E16
  - Ruled out by: plateau+purge (leak) jvm/E16
  - Hypothesis: N-WARM: detector hit at Audit.kt:9 is a warm-up fill
  - Refuted if: growth continues over extra cycles
  - Contradicting evidence: jvm/E16
  - Originally supported by: jvm/E9
- **jvm/F3.n1** (UNCLASSIFIED): discriminator jvm/E10
  - Ruled out by: plateau+purge (leak) jvm/E10
  - Hypothesis: N-WARM: Audit.trail growth is a warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: jvm/E10
  - Originally supported by: jvm/E9
- **jvm/F5** (UNCLASSIFIED): discriminator jvm/E17
  - Ruled out by: plateau+purge (non-leak) jvm/E17
  - Contradicting evidence: jvm/E17
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

- E4: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E4: limitation: workloads were discovered by reading files; none was run
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
- jvm/E13: partial coverage (2 of 3 examined)
- jvm/E13: limitation: jvm/F3: growth measured as a count; bytes not attributed
- jvm/E13: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- jvm/E13: limitation: threshold is provisional (not calibrated)
- jvm/E14: partial coverage (1 of 28 examined)
- jvm/E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E14: limitation: gate thresholds are provisional (not calibrated)
- jvm/E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E15: limitation: gap.unexplained is negative at 7 point(s): the accounted figure exceeds os.rss (reserved but untouched memory)
- jvm/E15: limitation: gate thresholds are provisional (not calibrated)
- jvm/E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E16: limitation: points 1 to 9 come from S1; the extra points from S5
- jvm/E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E16: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E17: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E17: limitation: points 1 to 9 come from S1; the extra points from S7
- jvm/E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E17: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E17: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E18: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E18: limitation: points 1 to 9 come from S1; the extra points from S9
- jvm/E18: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E18: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E18: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E19: partial coverage (3 of 3 examined)
- jvm/E19: limitation: jvm/F1: growth measured as a count; bytes not attributed
- jvm/E19: limitation: jvm/F3: growth measured as a count; bytes not attributed
- jvm/E19: limitation: threshold is provisional (not calibrated)
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
| `intake.workload` | What can exercise the suspected path? | answered | E4 | — |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | — |
| E2 | run-config | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | probe/probe.json |
| E4 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | workload/workloads.json |
| jvm/E1 | static-hit | static | spotbugs unknown | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-spotbugs.json |
| jvm/E2 | static-hit | static | detekt 2.0.0-alpha.6 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-detekt.json |
| jvm/E3 | static-hit | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-dla.json |
| jvm/E4 | census-holder | static | dla-census 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | census/jvm.json |
| jvm/E5 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | workload/jvm.json |
| jvm/E6 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | harness/H1/harness.json |
| jvm/E7 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (99aab8e04767) | harness/H1/run-S1.json |
| jvm/E8 | growth-series | test | dla 0.1.0 | artifact series/S1.json (12f964b2b6b6) | trend/T1.json |
| jvm/E9 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (12f964b2b6b6) | trend/T1.json |
| jvm/E10 | discriminator | test | dla 0.1.0 | artifact series/S3.json (bd401349c333) | alloc/A1.json |
| jvm/E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (12f964b2b6b6) | trend/T3.json |
| jvm/E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (12f964b2b6b6) | trend/T3.json |
| jvm/E13 | residual | test | dla 0.1.0 | artifact series/S1.json (12f964b2b6b6) | gap/residual-S1-jvm_E13.json |
| jvm/E14 | growth-series | test | dla 0.1.0 | artifact series/S1.json (12f964b2b6b6) | trend/T4.json |
| jvm/E15 | non-heap | test | dla 0.1.0 | artifact series/S4.json (199f16b4b293) | trend/T5.json |
| jvm/E16 | discriminator | test | dla 0.1.0 | artifact series/S6.json (e3836fccc214) | alloc/A2.json |
| jvm/E17 | discriminator | test | dla 0.1.0 | artifact series/S8.json (53ab3c20e64b) | alloc/A3.json |
| jvm/E18 | discriminator | test | dla 0.1.0 | artifact series/S10.json (4f429369867f) | alloc/A4.json |
| jvm/E19 | residual | test | dla 0.1.0 | artifact series/S1.json (12f964b2b6b6) | gap/residual-S1-jvm_E19.json |
| rust/E1 | static-hit | static | rapx 0.7.71 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/rust-rapx.json |
| rust/E2 | static-hit | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/rust-dla.json |
| rust/E3 | census-holder | static | dla-census 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | census/rust.json |
| rust/E4 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | workload/rust.json |
