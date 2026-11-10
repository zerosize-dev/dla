# DLA report

Run: `20261110-175340-4263`

Status: INCOMPLETE (63 open)

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 8 enumerated, 8 dispositioned; confirmed 4, strong 0, refuted 4, suspected 0, unresolved 0

Residual: not measured

Coverage: rust 1760 iterations, 0 of 10 entry points exercised

## Open items

- **T1:os.rss**: column os.rss of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:os.rss_anon**: column os.rss_anon of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:os.vm_data**: column os.vm_data of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:os.vm_hwm**: column os.vm_hwm of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:native.live_blocks**: column native.live_blocks of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:native.live_bytes**: column native.live_bytes of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:native.peak_bytes**: column native.peak_bytes of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T2:os.rss**: column os.rss of T2 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T2:os.rss_anon**: column os.rss_anon of T2 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T2:os.vm_data**: column os.vm_data of T2 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T2:os.vm_hwm**: column os.vm_hwm of T2 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T2:native.live_blocks**: column native.live_blocks of T2 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T2:native.live_bytes**: column native.live_bytes of T2 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T2:native.peak_bytes**: column native.peak_bytes of T2 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:os.rss**: column os.rss of T3 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:os.rss_anon**: column os.rss_anon of T3 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:os.vm_data**: column os.vm_data of T3 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:os.vm_hwm**: column os.vm_hwm of T3 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:native.live_blocks**: column native.live_blocks of T3 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:native.live_bytes**: column native.live_bytes of T3 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:native.peak_bytes**: column native.peak_bytes of T3 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:os.rss**: column os.rss of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:os.rss_anon**: column os.rss_anon of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:os.vm_data**: column os.vm_data of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:os.vm_hwm**: column os.vm_hwm of T4 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:native.live_blocks**: column native.live_blocks of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:native.live_bytes**: column native.live_bytes of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:native.peak_bytes**: column native.peak_bytes of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.rss**: column os.rss of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.rss_anon**: column os.rss_anon of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.vm_data**: column os.vm_data of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.vm_hwm**: column os.vm_hwm of T5 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:native.live_blocks**: column native.live_blocks of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:native.live_bytes**: column native.live_bytes of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:native.peak_bytes**: column native.peak_bytes of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:os.rss**: column os.rss of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:os.rss_anon**: column os.rss_anon of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:os.vm_data**: column os.vm_data of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:os.vm_hwm**: column os.vm_hwm of T6 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:native.live_blocks**: column native.live_blocks of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:native.live_bytes**: column native.live_bytes of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:native.peak_bytes**: column native.peak_bytes of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:os.rss**: column os.rss of T7 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:os.rss_anon**: column os.rss_anon of T7 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:os.vm_data**: column os.vm_data of T7 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:os.vm_hwm**: column os.vm_hwm of T7 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:native.live_blocks**: column native.live_blocks of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:native.live_bytes**: column native.live_bytes of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:native.peak_bytes**: column native.peak_bytes of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:os.rss**: column os.rss of T8 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:os.rss_anon**: column os.rss_anon of T8 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:os.vm_data**: column os.vm_data of T8 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:os.vm_hwm**: column os.vm_hwm of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:native.live_blocks**: column native.live_blocks of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:native.live_bytes**: column native.live_bytes of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:native.peak_bytes**: column native.peak_bytes of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:os.rss**: column os.rss of T9 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:os.rss_anon**: column os.rss_anon of T9 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:os.vm_data**: column os.vm_data of T9 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:os.vm_hwm**: column os.vm_hwm of T9 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:native.live_blocks**: column native.live_blocks of T9 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:native.live_bytes**: column native.live_bytes of T9 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:native.peak_bytes**: column native.peak_bytes of T9 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`

## Findings

### F1: **L-CYCLE**

- State: confirmed
- Label: L-CYCLE (evidence: E10 unit-count-series, E15 discriminator)
- Growth: `native.live_blocks@src/section.rs:16` 40.54 count/n (steady, gate pass, E10)
- Evidence classes: context, growth, static
- Holder: cycle-only
- Owner: `src.section.Section.parent`
- Region: heap
- Shape: steady
- Site: `src/section.rs:10`
- Writers: `src/section.rs:27`
- Verified: no
- Sibling hypotheses: F1.n1
- Suggested archetype (unconfirmed): L-CYCLE
- Supporting evidence: E4, E8, E10
- Explanation (observed): Section.parent (src/section.rs:10) is a strong Rc back-link to the parent, set in Section::append (section.rs:27), while the parent's children Vec holds the child: a cycle never freed when the parse tree is dropped.
- Explanation structure: adds: src/section.rs:27; removal: never; evidence: E10

### F2: **L-CYCLE**

- State: confirmed
- Label: L-CYCLE (evidence: E10 unit-count-series, E16 discriminator)
- Growth: `native.live_blocks@src/section.rs:16` 40.54 count/n (steady, gate pass, E10)
- Evidence classes: context, growth, static
- Holder: cycle-only
- Owner: `src.section.Section.children`
- Region: heap
- Shape: steady
- Site: `src/section.rs:11`
- Writers: `src/section.rs:28`
- Verified: no
- Sibling hypotheses: F2.n1
- Suggested archetype (unconfirmed): L-CYCLE
- Supporting evidence: E4, E8, E10
- Explanation (observed): Section.children (src/section.rs:11) is appended to in Section::append (section.rs:28) and each child holds a strong parent Rc, so the cycle is never freed: removal never happens after the response is sent.
- Explanation structure: adds: src/section.rs:28; removal: never; evidence: E10

### F3: **L-CYCLE**

- State: confirmed
- Label: L-CYCLE (evidence: E10 unit-count-series, E11 discriminator)
- Growth: `native.live_blocks@src/section.rs:16` 40.54 count/n (steady, gate pass, E10)
- Evidence classes: context, growth, static
- Holder: cycle-only
- Owner: `src.section.Section.parent`
- Region: heap
- Shape: steady
- Site: `src/section.rs:10`
- Writers: `src/section.rs:27`
- Verified: no
- Sibling hypotheses: F3.n1
- Supporting evidence: E5, E8, E10
- Explanation (observed): Section.parent (src/section.rs:10) holds a strong Rc to the parent while the parent's children Vec (src/section.rs:11) holds Rc to the child; Section::append (section.rs:27) builds this cycle per heading, so refcounts never reach zero when parse's Parsed tree is dropped.
- Explanation structure: adds: src/section.rs:27; removal: never; evidence: E10

### F4: **L-CYCLE**

- State: confirmed
- Label: L-CYCLE (evidence: E10 unit-count-series, E14 discriminator)
- Growth: `native.live_blocks@src/section.rs:16` 40.54 count/n (steady, gate pass, E10)
- Evidence classes: context, growth, static
- Holder: cycle-only
- Owner: `src.section.Section.children`
- Region: heap
- Shape: steady
- Site: `src/section.rs:11`
- Writers: `src/section.rs:28`
- Annotation: heuristic name match over the project's sources
- Verified: no
- Sibling hypotheses: F4.n1
- Supporting evidence: E5, E8, E10
- Explanation (observed): Section.children (src/section.rs:11) is appended to in Section::append (section.rs:28) and each child holds a strong parent Rc, so the cycle is never freed: removal never happens after the response is sent.
- Explanation structure: adds: src/section.rs:28; removal: never; evidence: E10

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (leak) E15
  - Hypothesis: N-WARM: growth is warm-up/high-water that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E15
  - Originally supported by: E10
- **F2.n1** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (leak) E16
  - Hypothesis: N-WARM: growth is warm-up/high-water that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E16
  - Originally supported by: E10
- **F3.n1** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (leak) E11
  - Hypothesis: N-WARM: growth is warm-up/high-water that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E11
  - Originally supported by: E10
- **F4.n1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (leak) E14
  - Hypothesis: N-WARM: growth is warm-up/high-water that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E14
  - Originally supported by: E10

## Blind spots and unknowns

### Coverage gaps

- rust/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- rust/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)
- rust/rapx: blocked (cannot copy the project: Permission denied (os error 13))

### Runtime blind spots

- rust: the allocator shim does not record which entry points ran

### Entry points not exercisable

- rust: `src/parser.rs::ParseError.fmt`: no_harness

### Evidence limitations and unknowns

- E5: partial coverage
- E5: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E5: limitation: writers and removers are textual name matches
- E6: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E6: limitation: workloads were discovered by reading files; none was run
- E7: limitation: harness written; nothing was run (dla harness run measures it)
- E7: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E8: partial coverage (9 of 9 examined)
- E8: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: exercised entry points are not measured for rust: the allocator shim sees allocations, not calls
- E9: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: gate thresholds are provisional (not calibrated)
- E10: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: partial coverage (9 of 26 examined)
- E11: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: points 1 to 9 come from S1; the extra points from S2
- E11: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E11: limitation: gate and plateau thresholds are provisional (not calibrated)
- E12: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: partial coverage (9 of 32 examined)
- E14: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: points 1 to 9 come from S1; the extra points from S4
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: partial coverage (9 of 32 examined)
- E15: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: points 1 to 9 come from S1; the extra points from S6
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: partial coverage (9 of 32 examined)
- E16: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: points 1 to 9 come from S1; the extra points from S8
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: gate thresholds are provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | — |
| E2 | run-config | static | dla 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | probe/probe.json |
| E4 | static-hit | static | dla 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | sweep/rust-dla.json |
| E5 | census-holder | static | dla-census 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | census/rust.json |
| E6 | workload | static | dla 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | workload/workloads.json |
| E7 | workload | static | dla 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | harness/H1/harness.json |
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (a97bc07189a6) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (8a3433e415be) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (8a3433e415be) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (185fb773fd15) | alloc/A1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S2.json (e70aea5128a0) | trend/T3.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (e70aea5128a0) | trend/T3.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S5.json (c7aecd534c44) | alloc/A2.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S7.json (f5a25476d2e7) | alloc/A3.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S9.json (820987477cdb) | alloc/A4.json |
| E17 | growth-series | test | dla 0.1.0 | artifact series/S4.json (54dfd483d363) | trend/T7.json |
| E18 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (54dfd483d363) | trend/T7.json |
| E19 | growth-series | test | dla 0.1.0 | artifact series/S6.json (715517b83e39) | trend/T8.json |
| E20 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (715517b83e39) | trend/T8.json |
| E21 | growth-series | test | dla 0.1.0 | artifact series/S8.json (4b55ff39b54c) | trend/T9.json |
| E22 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (4b55ff39b54c) | trend/T9.json |
