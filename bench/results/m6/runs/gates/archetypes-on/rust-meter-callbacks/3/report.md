# DLA report

Run: `20261110-174436-34a9`

Status: INCOMPLETE (71 open)

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 8 enumerated, 8 dispositioned; confirmed 3, strong 0, refuted 3, suspected 2, unresolved 0

Residual: not measured

Coverage: rust 1760 iterations, 0 of 14 entry points exercised

## Open items

- **T1:os.rss**: column os.rss of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:os.rss_anon**: column os.rss_anon of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:os.vm_data**: column os.vm_data of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:os.vm_hwm**: column os.vm_hwm of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:native.live_blocks**: column native.live_blocks of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:native.live_bytes**: column native.live_bytes of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:native.peak_bytes**: column native.peak_bytes of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.rss**: column os.rss of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.rss_anon**: column os.rss_anon of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.vm_data**: column os.vm_data of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.vm_hwm**: column os.vm_hwm of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:native.live_blocks**: column native.live_blocks of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:native.live_bytes**: column native.live_bytes of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:native.peak_bytes**: column native.peak_bytes of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:os.rss**: column os.rss of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:os.rss_anon**: column os.rss_anon of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:os.vm_data**: column os.vm_data of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:os.vm_hwm**: column os.vm_hwm of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:native.live_blocks**: column native.live_blocks of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:native.live_bytes**: column native.live_bytes of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:native.peak_bytes**: column native.peak_bytes of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:os.rss**: column os.rss of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:os.rss_anon**: column os.rss_anon of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:os.vm_data**: column os.vm_data of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:os.vm_hwm**: column os.vm_hwm of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:native.live_blocks**: column native.live_blocks of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:native.live_bytes**: column native.live_bytes of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:native.peak_bytes**: column native.peak_bytes of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T15:os.rss**: column os.rss of T15 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S11 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T15:os.rss_anon**: column os.rss_anon of T15 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S11 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T15:os.vm_data**: column os.vm_data of T15 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S11 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T15:os.vm_hwm**: column os.vm_hwm of T15 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S11 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T15:native.live_blocks**: column native.live_blocks of T15 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S11 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T15:native.live_bytes**: column native.live_bytes of T15 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S11 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T15:native.peak_bytes**: column native.peak_bytes of T15 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S11 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T17:stack:new**: column stack:new of T17 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E46 # column stack:new`
- **T17:stack:new.count**: column stack:new.count of T17 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E47 # column stack:new.count`
- **T17:stack:open#2**: column stack:open#2 of T17 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E46 # column stack:open#2`
- **T17:stack:open#2.count**: column stack:open#2.count of T17 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E47 # column stack:open#2.count`
- **T18:os.rss**: column os.rss of T18 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T18:os.rss_anon**: column os.rss_anon of T18 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T18:os.vm_data**: column os.vm_data of T18 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T18:os.vm_hwm**: column os.vm_hwm of T18 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T18:native.live_blocks**: column native.live_blocks of T18 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T18:native.live_bytes**: column native.live_bytes of T18 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T18:native.peak_bytes**: column native.peak_bytes of T18 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T19:os.rss**: column os.rss of T19 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T19:os.rss_anon**: column os.rss_anon of T19 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T19:os.vm_data**: column os.vm_data of T19 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T19:os.vm_hwm**: column os.vm_hwm of T19 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T19:native.live_blocks**: column native.live_blocks of T19 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T19:native.live_bytes**: column native.live_bytes of T19 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T19:native.peak_bytes**: column native.peak_bytes of T19 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T20:os.rss**: column os.rss of T20 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S10 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T20:os.rss_anon**: column os.rss_anon of T20 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S10 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T20:os.vm_data**: column os.vm_data of T20 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S10 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T20:os.vm_hwm**: column os.vm_hwm of T20 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S10 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T20:native.live_blocks**: column native.live_blocks of T20 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S10 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T20:native.live_bytes**: column native.live_bytes of T20 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S10 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T20:native.peak_bytes**: column native.peak_bytes of T20 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S10 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T21:os.rss**: column os.rss of T21 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S12 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T21:os.rss_anon**: column os.rss_anon of T21 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S12 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T21:os.vm_data**: column os.vm_data of T21 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S12 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T21:os.vm_hwm**: column os.vm_hwm of T21 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S12 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T21:native.live_blocks**: column native.live_blocks of T21 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S12 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T21:native.live_bytes**: column native.live_bytes of T21 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S12 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T21:native.peak_bytes**: column native.peak_bytes of T21 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S12 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T22:stack:open#2**: column stack:open#2 of T22 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E60 # column stack:open#2`
- **T22:stack:open#2.count**: column stack:open#2.count of T22 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E61 # column stack:open#2.count`
- **T23:stack:new**: column stack:new of T23 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E62 # column stack:new`
- **T23:stack:new.count**: column stack:new.count of T23 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E63 # column stack:new.count`

## Findings

### F2: **L-OWNERSHIP**

- State: confirmed
- Label: L-OWNERSHIP (evidence: E9 growth-series, E21 alloc-stack)
- Growth: `native.live_blocks@src/session.rs:19` 1 count/n (steady, gate pass, E10)
- Evidence classes: context, growth, static
- Holder: native-handle/global-ref
- Owner: `src.session.Session.open`
- Region: off-heap
- Shape: steady
- Site: `src/session.rs:19`
- Writers: `src/session.rs:19`
- Verified: no
- Sibling hypotheses: F2.n1
- Supporting evidence: E4, E8, E10
- Explanation (observed): Session::open boxes a Window via Box::into_raw at src/session.rs:19 and stores the raw pointer; Session::close only calls Registry::remove, discarding the returned user pointer, so the Box<Window> is never freed.
- Explanation structure: adds: src/session.rs:19; removal: never; evidence: E28

### F4: **L-OWNERSHIP**

- State: confirmed
- Label: L-OWNERSHIP (evidence: E9 growth-series, E21 alloc-stack); region does not match the archetype's
- Growth: `native.live_bytes@src/window.rs:22` 4096 bytes/n (steady, gate pass, E9)
- Evidence classes: context, growth
- Holder: native-handle/global-ref
- Owner: `Box<Window> leaked by Session::open`
- Region: heap
- Shape: steady
- Site: `src/window.rs:22`
- Writers: `src/session.rs:19`
- Annotation: Window sample buffer; freed only if the leaked Box<Window> of F2 were reclaimed
- Verified: no
- Sibling hypotheses: F4.n1
- Supporting evidence: E9, E8, E22
- Explanation (observed): The sample buffer at src/window.rs:22 is owned by the Box<Window> leaked by Session::open via Box::into_raw (src/session.rs:19), held only by a raw pointer; Session::close drops the registry entry but never frees it.
- Explanation structure: adds: src/session.rs:19; removal: never; evidence: E22

### F5: **L-OWNERSHIP**

- State: confirmed
- Label: L-OWNERSHIP (evidence: E9 growth-series, E21 alloc-stack); region does not match the archetype's
- Growth: `native.live_bytes@src/window.rs:9` 13 bytes/n (steady, gate pass, E9)
- Evidence classes: context, growth
- Holder: native-handle/global-ref
- Owner: `Box<Window> leaked by Session::open`
- Region: heap
- Shape: steady
- Site: `src/window.rs:9`
- Writers: `src/session.rs:19`
- Verified: no
- Sibling hypotheses: F5.n1
- Supporting evidence: E9, E8, E22
- Explanation (observed): The device name at src/window.rs:9 is owned by the Box<Window> leaked by Session::open via Box::into_raw (src/session.rs:19), held only by a raw pointer; Session::close drops the registry entry but never frees it.
- Explanation structure: adds: src/session.rs:19; removal: never; evidence: E22

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.registry.Registry.entries`
- Region: heap
- Site: `src/registry.rs:15`
- Writers: `src/registry.rs:22`
- Removers: `src/registry.rs:29`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: Registry.entries is emptied by Session::close (active=0 at exit); no growth at its site. Re-check with a probe on Registry::len if sessions can fail to close.
- Supporting evidence: E4

### F3: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Site: `src/session.rs:19`
- Suggested archetype (unconfirmed): L-OWNERSHIP
- Reason: not_exercised
- Remedy: Same site as confirmed F2 (src/session.rs:19); duplicate detector hit, covered by F2.
- Supporting evidence: E5

## Ruled out

- **F2.n1** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (leak) E11
  - Hypothesis: N-WARM
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E11
  - Originally supported by: E10
- **F4.n1** (UNCLASSIFIED): discriminator E30
  - Ruled out by: plateau+purge (leak) E30
  - Hypothesis: N-WARM
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E30
  - Originally supported by: E11
- **F5.n1** (UNCLASSIFIED): discriminator E31
  - Ruled out by: plateau+purge (leak) E31
  - Hypothesis: N-WARM
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E31
  - Originally supported by: E11

## Blind spots and unknowns

### Coverage gaps

- rust/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- rust/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)
- rust/rapx: blocked (cannot copy the project: Permission denied (os error 13))

### Runtime blind spots

- rust: the allocator shim does not record which entry points ran

### Entry points not exercisable

None recorded.

### Evidence limitations and unknowns

- E4: partial coverage
- E4: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E4: limitation: writers and removers are textual name matches
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
- E11: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: points 1 to 9 come from S1; the extra points from S2
- E11: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E11: limitation: gate and plateau thresholds are provisional (not calibrated)
- E12: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: points 1 to 9 come from S1; the extra points from S2
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: points 1 to 9 come from S1; the extra points from S2
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: points 1 to 9 come from S1; the extra points from S2
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: points 1 to 9 come from S1; the extra points from S2
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: partial coverage (0 of 9 examined)
- E20: limitation: valgrind slows the program down; points 1, 5, 9 of 9 ran under it and no series was written (dla harness run without --tool measures the series)
- E21: limitation: memcheck scans memory at exit: still-reachable blocks are live at exit, lost blocks have no pointer left; each file is one process
- E21: limitation: the files are ordered as given, and are runs of different lengths, not dumps of one process
- E22: limitation: memcheck scans memory at exit: still-reachable blocks are live at exit, lost blocks have no pointer left; each file is one process
- E22: limitation: the files are ordered as given, and are runs of different lengths, not dumps of one process
- E22: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E23: limitation: only the top loss record is followed, from one of its blocks; other records and blocks are not walked
- E23: limitation: a walk stops after 12 queries or at the first answer that names no root
- E24: limitation: only the top loss record is followed, from one of its blocks; other records and blocks are not walked
- E24: limitation: a walk stops after 12 queries or at the first answer that names no root
- E25: limitation: only the top loss record is followed, from one of its blocks; other records and blocks are not walked
- E25: limitation: a walk stops after 12 queries or at the first answer that names no root
- E26: limitation: each point is a separate process at exit, run with more work; the figure is the bytes of the top loss record whose chain ends in the key's data symbol
- E26: limitation: the loss record is not the symbol's whole retained set: other records may hang from the same symbol
- E26: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E27: limitation: memcheck scans memory at exit: still-reachable blocks are live at exit, lost blocks have no pointer left; each file is one process
- E27: limitation: the files are ordered as given, and are runs of different lengths, not dumps of one process
- E28: limitation: memcheck scans memory at exit: still-reachable blocks are live at exit, lost blocks have no pointer left; each file is one process
- E28: limitation: the files are ordered as given, and are runs of different lengths, not dumps of one process
- E28: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E29: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E29: limitation: points 1 to 9 come from S1; the extra points from S7
- E29: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E29: limitation: gate and plateau thresholds are provisional (not calibrated)
- E30: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E30: limitation: points 1 to 9 come from S1; the extra points from S9
- E30: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E30: limitation: gate and plateau thresholds are provisional (not calibrated)
- E31: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E31: limitation: points 1 to 9 come from S1; the extra points from S11
- E31: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E31: limitation: gate and plateau thresholds are provisional (not calibrated)
- E32: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E32: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E32: limitation: gate thresholds are provisional (not calibrated)
- E33: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E33: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E33: limitation: gate thresholds are provisional (not calibrated)
- E34: limitation: x is the point number in capture order; t_ms is the time since the first capture where the inputs record one
- E34: limitation: each point comes from a separate point of one process; a figure absent from a point has no value at that point
- E34: limitation: gate thresholds are provisional (not calibrated)
- E35: limitation: x is the point number in capture order; t_ms is the time since the first capture where the inputs record one
- E35: limitation: each point comes from a separate point of one process; a figure absent from a point has no value at that point
- E35: limitation: gate thresholds are provisional (not calibrated)
- E36: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E36: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E36: limitation: gate thresholds are provisional (not calibrated)
- E37: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E37: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E37: limitation: gate thresholds are provisional (not calibrated)
- E38: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E38: limitation: gate thresholds are provisional (not calibrated)
- E39: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E39: limitation: gate thresholds are provisional (not calibrated)
- E40: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E40: limitation: gate thresholds are provisional (not calibrated)
- E41: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E41: limitation: gate thresholds are provisional (not calibrated)
- E42: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E42: limitation: gate thresholds are provisional (not calibrated)
- E43: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E43: limitation: gate thresholds are provisional (not calibrated)
- E44: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E44: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E44: limitation: gate thresholds are provisional (not calibrated)
- E45: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E45: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E45: limitation: gate thresholds are provisional (not calibrated)
- E46: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E46: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E46: limitation: gate thresholds are provisional (not calibrated)
- E47: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E47: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E47: limitation: gate thresholds are provisional (not calibrated)
- E48: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E48: limitation: gate thresholds are provisional (not calibrated)
- E49: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E49: limitation: gate thresholds are provisional (not calibrated)
- E50: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E50: limitation: gate thresholds are provisional (not calibrated)
- E51: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E51: limitation: gate thresholds are provisional (not calibrated)
- E52: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E52: limitation: points 1 to 9 come from S1; the extra points from S9
- E52: limitation: gate thresholds are provisional (not calibrated)
- E53: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E53: limitation: points 1 to 9 come from S1; the extra points from S9
- E53: limitation: gate thresholds are provisional (not calibrated)
- E54: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E54: limitation: points 1 to 9 come from S1; the extra points from S11
- E54: limitation: gate thresholds are provisional (not calibrated)
- E55: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E55: limitation: points 1 to 9 come from S1; the extra points from S11
- E55: limitation: gate thresholds are provisional (not calibrated)
- E56: limitation: memcheck scans memory at exit: still-reachable blocks are live at exit, lost blocks have no pointer left; each file is one process
- E56: limitation: the files are ordered as given, and are runs of different lengths, not dumps of one process
- E57: limitation: memcheck scans memory at exit: still-reachable blocks are live at exit, lost blocks have no pointer left; each file is one process
- E57: limitation: the files are ordered as given, and are runs of different lengths, not dumps of one process
- E57: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E58: limitation: memcheck scans memory at exit: still-reachable blocks are live at exit, lost blocks have no pointer left; each file is one process
- E58: limitation: the files are ordered as given, and are runs of different lengths, not dumps of one process
- E59: limitation: memcheck scans memory at exit: still-reachable blocks are live at exit, lost blocks have no pointer left; each file is one process
- E59: limitation: the files are ordered as given, and are runs of different lengths, not dumps of one process
- E59: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E60: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E60: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E60: limitation: gate thresholds are provisional (not calibrated)
- E61: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E61: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E61: limitation: gate thresholds are provisional (not calibrated)
- E62: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E62: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E62: limitation: gate thresholds are provisional (not calibrated)
- E63: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E63: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E63: limitation: gate thresholds are provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo 8ba6ce044aa1c1ebf7e8fc28516f4590d46dd902 | — |
| E2 | run-config | static | dla 0.1.0 | repo 8ba6ce044aa1c1ebf7e8fc28516f4590d46dd902 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 8ba6ce044aa1c1ebf7e8fc28516f4590d46dd902 | probe/probe.json |
| E4 | census-holder | static | dla-census 0.1.0 | repo 8ba6ce044aa1c1ebf7e8fc28516f4590d46dd902 | census/rust.json |
| E5 | static-hit | static | dla 0.1.0 | repo 8ba6ce044aa1c1ebf7e8fc28516f4590d46dd902 | sweep/rust-dla.json |
| E6 | workload | static | dla 0.1.0 | repo 8ba6ce044aa1c1ebf7e8fc28516f4590d46dd902 | workload/workloads.json |
| E7 | workload | static | dla 0.1.0 | repo 8ba6ce044aa1c1ebf7e8fc28516f4590d46dd902 | harness/H1/harness.json |
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (81dc1b3240f6) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (f88d13d68d28) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (f88d13d68d28) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (c50131066b5b) | alloc/A1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S2.json (8e3fd3e1e614) | trend/T3.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (8e3fd3e1e614) | trend/T3.json |
| E14 | growth-series | test | dla 0.1.0 | artifact series/S3.json (c50131066b5b) | trend/T4.json |
| E15 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (c50131066b5b) | trend/T4.json |
| E16 | growth-series | test | dla 0.1.0 | artifact series/S2.json (8e3fd3e1e614) | trend/T5.json |
| E17 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (8e3fd3e1e614) | trend/T5.json |
| E18 | growth-series | test | dla 0.1.0 | artifact series/S3.json (c50131066b5b) | trend/T6.json |
| E19 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (c50131066b5b) | trend/T6.json |
| E20 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (7d8f8c1aa812) | harness/H1/run-valgrind.json |
| E21 | alloc-stack | live | valgrind-memcheck 0.1.0 | artifact allocs/A1.json (996586ec5f83) | allocs/A1.json |
| E22 | growth-series | live | valgrind-memcheck 0.1.0 | artifact series/S4.json (2855a51fdc76) | series/S4.json |
| E23 | retention-path | live | valgrind-vgdb 0.1.0 | artifact .dla/20261110-174436-34a9/harness/H1/out/p1-native/vgdb-1-leak.txt (0b3a2938b4f2) | summaries/vgdb-p1-native-232386f6.json |
| E24 | retention-path | live | valgrind-vgdb 0.1.0 | artifact .dla/20261110-174436-34a9/harness/H1/out/p5-native/vgdb-1-leak.txt (e813da2ca97c) | summaries/vgdb-p5-native-37d74fc0.json |
| E25 | retention-path | live | valgrind-vgdb 0.1.0 | artifact .dla/20261110-174436-34a9/harness/H1/out/p9-native/vgdb-1-leak.txt (26a52c5357b7) | summaries/vgdb-p9-native-4786bb07.json |
| E26 | growth-series | live | valgrind-vgdb 0.1.0 | artifact series/S5.json (186cf60ea758) | series/S5.json |
| E27 | alloc-stack | live | valgrind-memcheck 0.1.0 | artifact allocs/A2.json (5c210206944d) | allocs/A2.json |
| E28 | growth-series | live | valgrind-memcheck 0.1.0 | artifact series/S6.json (5a627c1fdf09) | series/S6.json |
| E29 | discriminator | test | dla 0.1.0 | artifact series/S8.json (6278571bb3c9) | alloc/A2.json |
| E30 | discriminator | test | dla 0.1.0 | artifact series/S10.json (6b0ac30aeead) | alloc/A3.json |
| E31 | discriminator | test | dla 0.1.0 | artifact series/S12.json (6530bd9d0cd4) | alloc/A4.json |
| E32 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (2855a51fdc76) | trend/T10.json |
| E33 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (2855a51fdc76) | trend/T10.json |
| E34 | growth-series | dump | dla 0.1.0 | artifact series/S5.json (186cf60ea758) | trend/T11.json |
| E35 | unit-count-series | dump | dla 0.1.0 | artifact series/S5.json (186cf60ea758) | trend/T11.json |
| E36 | growth-series | dump | dla 0.1.0 | artifact series/S6.json (5a627c1fdf09) | trend/T12.json |
| E37 | unit-count-series | dump | dla 0.1.0 | artifact series/S6.json (5a627c1fdf09) | trend/T12.json |
| E38 | growth-series | test | dla 0.1.0 | artifact series/S7.json (a677084289ee) | trend/T13.json |
| E39 | unit-count-series | test | dla 0.1.0 | artifact series/S7.json (a677084289ee) | trend/T13.json |
| E40 | growth-series | test | dla 0.1.0 | artifact series/S9.json (0f2b10fc4e23) | trend/T14.json |
| E41 | unit-count-series | test | dla 0.1.0 | artifact series/S9.json (0f2b10fc4e23) | trend/T14.json |
| E42 | growth-series | test | dla 0.1.0 | artifact series/S11.json (b05022c38b7f) | trend/T15.json |
| E43 | unit-count-series | test | dla 0.1.0 | artifact series/S11.json (b05022c38b7f) | trend/T15.json |
| E44 | growth-series | dump | dla 0.1.0 | artifact series/S4.json (2855a51fdc76) | trend/T16.json |
| E45 | unit-count-series | dump | dla 0.1.0 | artifact series/S4.json (2855a51fdc76) | trend/T16.json |
| E46 | growth-series | dump | dla 0.1.0 | artifact series/S6.json (5a627c1fdf09) | trend/T17.json |
| E47 | unit-count-series | dump | dla 0.1.0 | artifact series/S6.json (5a627c1fdf09) | trend/T17.json |
| E48 | growth-series | test | dla 0.1.0 | artifact series/S7.json (a677084289ee) | trend/T18.json |
| E49 | unit-count-series | test | dla 0.1.0 | artifact series/S7.json (a677084289ee) | trend/T18.json |
| E50 | growth-series | test | dla 0.1.0 | artifact series/S9.json (0f2b10fc4e23) | trend/T19.json |
| E51 | unit-count-series | test | dla 0.1.0 | artifact series/S9.json (0f2b10fc4e23) | trend/T19.json |
| E52 | growth-series | test | dla 0.1.0 | artifact series/S10.json (6b0ac30aeead) | trend/T20.json |
| E53 | unit-count-series | test | dla 0.1.0 | artifact series/S10.json (6b0ac30aeead) | trend/T20.json |
| E54 | growth-series | test | dla 0.1.0 | artifact series/S12.json (6530bd9d0cd4) | trend/T21.json |
| E55 | unit-count-series | test | dla 0.1.0 | artifact series/S12.json (6530bd9d0cd4) | trend/T21.json |
| E56 | alloc-stack | live | valgrind-memcheck 0.1.0 | artifact allocs/A3.json (e1df6f5952a7) | allocs/A3.json |
| E57 | growth-series | live | valgrind-memcheck 0.1.0 | artifact series/S13.json (8046b485bd57) | series/S13.json |
| E58 | alloc-stack | live | valgrind-memcheck 0.1.0 | artifact allocs/A4.json (918fa6f0cceb) | allocs/A4.json |
| E59 | growth-series | live | valgrind-memcheck 0.1.0 | artifact series/S14.json (91521bd7829c) | series/S14.json |
| E60 | growth-series | dump | dla 0.1.0 | artifact series/S13.json (8046b485bd57) | trend/T22.json |
| E61 | unit-count-series | dump | dla 0.1.0 | artifact series/S13.json (8046b485bd57) | trend/T22.json |
| E62 | growth-series | dump | dla 0.1.0 | artifact series/S14.json (91521bd7829c) | trend/T23.json |
| E63 | unit-count-series | dump | dla 0.1.0 | artifact series/S14.json (91521bd7829c) | trend/T23.json |
