# DLA report

Run: `20261110-185937-99d7`

Status: INCOMPLETE (65 open)

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 9 enumerated, 9 dispositioned; confirmed 4, strong 0, refuted 4, suspected 0, unresolved 1

Residual: heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O2; heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O2; heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O2; heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O2; heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O2; heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O2; heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O2; heap 4165 bytes/n unattributed of 4165 total (threshold 1041.25) ABOVE THRESHOLD, see O2

Coverage: rust 3520 iterations, 0 of 14 entry points exercised

## Open items

- **O2**: open item O2: unexplained growth: 4165 bytes/n in heap (S1); add a hypothesis (dla ledger add --origin residual --evidence E30 ...) or record it unresolved. Fix: `dla ledger resolve O2 --as unresolved --reason time_budget --remedy "<text>"`
- **T3:os.rss**: column os.rss of T3 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:os.rss_anon**: column os.rss_anon of T3 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:os.vm_data**: column os.vm_data of T3 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:os.vm_hwm**: column os.vm_hwm of T3 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:native.live_blocks**: column native.live_blocks of T3 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:native.live_bytes**: column native.live_bytes of T3 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:native.peak_bytes**: column native.peak_bytes of T3 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:os.rss**: column os.rss of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:os.rss_anon**: column os.rss_anon of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:os.vm_data**: column os.vm_data of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:os.vm_hwm**: column os.vm_hwm of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:native.live_blocks**: column native.live_blocks of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:native.live_bytes**: column native.live_bytes of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:native.peak_bytes**: column native.peak_bytes of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.rss**: column os.rss of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.rss_anon**: column os.rss_anon of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.vm_data**: column os.vm_data of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.vm_hwm**: column os.vm_hwm of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:native.live_blocks**: column native.live_blocks of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:native.live_bytes**: column native.live_bytes of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:native.peak_bytes**: column native.peak_bytes of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:os.rss**: column os.rss of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:os.rss_anon**: column os.rss_anon of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:os.vm_data**: column os.vm_data of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:os.vm_hwm**: column os.vm_hwm of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:native.live_blocks**: column native.live_blocks of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:native.live_bytes**: column native.live_bytes of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:native.peak_bytes**: column native.peak_bytes of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:os.rss**: column os.rss of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:os.rss_anon**: column os.rss_anon of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:os.vm_data**: column os.vm_data of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:os.vm_hwm**: column os.vm_hwm of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:native.live_blocks**: column native.live_blocks of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:native.live_bytes**: column native.live_bytes of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:native.peak_bytes**: column native.peak_bytes of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:os.rss**: column os.rss of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:os.rss_anon**: column os.rss_anon of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:os.vm_data**: column os.vm_data of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:os.vm_hwm**: column os.vm_hwm of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:native.live_blocks**: column native.live_blocks of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:native.live_bytes**: column native.live_bytes of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:native.peak_bytes**: column native.peak_bytes of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:os.rss**: column os.rss of T9 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:os.rss_anon**: column os.rss_anon of T9 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:os.vm_data**: column os.vm_data of T9 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:os.vm_hwm**: column os.vm_hwm of T9 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:native.live_blocks**: column native.live_blocks of T9 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:native.live_bytes**: column native.live_bytes of T9 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:native.peak_bytes**: column native.peak_bytes of T9 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T10:os.rss**: column os.rss of T10 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T10:os.rss_anon**: column os.rss_anon of T10 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T10:os.vm_data**: column os.vm_data of T10 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T10:os.vm_hwm**: column os.vm_hwm of T10 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T10:native.live_blocks**: column native.live_blocks of T10 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T10:native.live_bytes**: column native.live_bytes of T10 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T10:native.peak_bytes**: column native.peak_bytes of T10 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **heap**: residual growth in heap (4165.000 bytes/n, threshold 1041.250) has no resolved open item. Fix: `dla gap --residual then resolve the open item it raises`
- **heap**: residual growth in heap (4165.000 bytes/n, threshold 1041.250) has no resolved open item. Fix: `dla gap --residual then resolve the open item it raises`
- **heap**: residual growth in heap (4165.000 bytes/n, threshold 1041.250) has no resolved open item. Fix: `dla gap --residual then resolve the open item it raises`
- **heap**: residual growth in heap (4165.000 bytes/n, threshold 1041.250) has no resolved open item. Fix: `dla gap --residual then resolve the open item it raises`
- **heap**: residual growth in heap (4165.000 bytes/n, threshold 1041.250) has no resolved open item. Fix: `dla gap --residual then resolve the open item it raises`
- **heap**: residual growth in heap (4165.000 bytes/n, threshold 1041.250) has no resolved open item. Fix: `dla gap --residual then resolve the open item it raises`
- **heap**: residual growth in heap (4165.000 bytes/n, threshold 1041.250) has no resolved open item. Fix: `dla gap --residual then resolve the open item it raises`
- **heap**: residual growth in heap (4165.000 bytes/n, threshold 1041.250) has no resolved open item. Fix: `dla gap --residual then resolve the open item it raises`

## Findings

### F1: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Session::open leaks a Box<Window> via Box::into_raw (src/session.rs:19); the raw pointer is stored in the Registry entry and in Session. Session::close calls registry.remove(id), which returns the user pointer, but the result is discarded and never passed to Box::from_raw, so each session's Window (512-sample Vec plus device String) is never freed.)
- Growth: `native.live_blocks@src/session.rs:19` 1 count/n (steady, gate pass, E10)
- Evidence classes: context, growth, static
- Holder: native-handle/global-ref
- Region: heap
- Shape: steady
- Site: `src/session.rs:19`
- Writers: `src/session.rs:19`
- Verified: no
- Sibling hypotheses: F1.n1
- Supporting evidence: E4, E8, E10
- Explanation (observed): Holder native-handle/global-ref (Session::open): src/session.rs:19 leaks Box<Window> via Box::into_raw; Session::close drops the pointer returned by registry.remove and never calls Box::from_raw, so removal never happens.
- Explanation structure: adds: src/session.rs:19; removal: never; evidence: E10

### F3: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Same leak as F1: census view of the Box::into_raw at src/session.rs:19, never reclaimed in Session::close.)
- Growth: `native.live_blocks@src/session.rs:19` 1 count/n (steady, gate pass, E10)
- Evidence classes: context, growth, static
- Holder: native-handle/global-ref
- Owner: `src.session.Session.open`
- Region: off-heap
- Shape: steady
- Site: `src/session.rs:19`
- Writers: `src/session.rs:19`
- Verified: no
- Sibling hypotheses: F3.n1
- Supporting evidence: E5, E8, E10
- Explanation (observed): Holder native-handle/global-ref (Session::open): src/session.rs:19 Box::into_raw hands the Window to the registry; Session::close discards the pointer from remove and never calls Box::from_raw, so it is never freed.
- Explanation structure: adds: src/session.rs:19; removal: never; evidence: E10

### F4: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Consequence of F1: the 512-capacity samples Vec (src/window.rs:22) of each leaked Window stays allocated because the Box<Window> is never freed.)
- Growth: `native.live_blocks@src/window.rs:22` 1 count/n (steady, gate pass, E10)
- Evidence classes: context, growth
- Holder: native-handle/global-ref
- Owner: `src.session.Session.open`
- Region: heap
- Shape: steady
- Site: `src/window.rs:22`
- Writers: `src/window.rs:22`
- Verified: no
- Sibling hypotheses: F4.n1
- Supporting evidence: E10, E8
- Explanation (observed): Holder native-handle/global-ref (Session::open): the samples Vec allocated at src/window.rs:22 lives inside a Box<Window> leaked by Box::into_raw; Session::close never calls Box::from_raw, so it is never freed.
- Explanation structure: adds: src/window.rs:22; removal: never; evidence: E10

### F5: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Consequence of F1: the device name String clone (src/window.rs:9) of each leaked Window stays allocated because the Box<Window> is never freed.)
- Growth: `native.live_blocks@src/window.rs:9` 1 count/n (steady, gate pass, E10)
- Evidence classes: context, growth
- Holder: native-handle/global-ref
- Owner: `src.session.Session.open`
- Region: heap
- Shape: steady
- Site: `src/window.rs:9`
- Writers: `src/window.rs:9`
- Verified: no
- Sibling hypotheses: F5.n1
- Supporting evidence: E10, E8
- Explanation (observed): Holder native-handle/global-ref (Session::open): the device String cloned at src/window.rs:9 lives inside a Box<Window> leaked by Box::into_raw; Session::close never calls Box::from_raw, so it is never freed.
- Explanation structure: adds: src/window.rs:9; removal: never; evidence: E10

### F2: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `src.registry.Registry.entries`
- Region: heap
- Site: `src/registry.rs:15`
- Writers: `src/registry.rs:22`
- Removers: `src/registry.rs:29`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: No growth column ties to Registry.entries; the workload prints active=0 after all sessions, so entries are removed. Not confirmed as a leak.
- Supporting evidence: E5

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (leak) E11
  - Hypothesis: N-WARM
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E11
  - Originally supported by: E10
- **F3.n1** (UNCLASSIFIED): discriminator E19
  - Ruled out by: plateau+purge (leak) E19
  - Hypothesis: N-WARM
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E19
  - Originally supported by: E10
- **F4.n1** (UNCLASSIFIED): discriminator E20
  - Ruled out by: plateau+purge (leak) E20
  - Hypothesis: N-WARM
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E20
  - Originally supported by: E10
- **F5.n1** (UNCLASSIFIED): discriminator E21
  - Ruled out by: plateau+purge (leak) E21
  - Hypothesis: N-WARM
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E21
  - Originally supported by: E10

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

- E5: partial coverage
- E5: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E5: limitation: writers and removers are textual name matches
- E6: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E6: limitation: workloads were discovered by reading files; none was run
- E7: limitation: harness written; nothing was run (dla harness run measures it)
- E7: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E8: partial coverage (12 of 12 examined)
- E8: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: exercised entry points are not measured for rust: the allocator shim sees allocations, not calls
- E9: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: gate thresholds are provisional (not calibrated)
- E10: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E11: limitation: gate and plateau thresholds are provisional (not calibrated)
- E12: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E17: limitation: gate and plateau thresholds are provisional (not calibrated)
- E18: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E18: limitation: gate and plateau thresholds are provisional (not calibrated)
- E19: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E19: limitation: gate and plateau thresholds are provisional (not calibrated)
- E20: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E20: limitation: gate and plateau thresholds are provisional (not calibrated)
- E21: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E21: limitation: gate and plateau thresholds are provisional (not calibrated)
- E22: partial coverage (1 of 3 examined)
- E22: limitation: F1: growth measured as a count; bytes not attributed
- E22: limitation: F3: growth measured as a count; bytes not attributed
- E22: limitation: F4: growth measured as a count; bytes not attributed
- E22: limitation: F5: growth measured as a count; bytes not attributed
- E22: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E22: limitation: threshold is provisional (not calibrated)
- E23: partial coverage (1 of 3 examined)
- E23: limitation: F1: growth evidence is from S1, not S2; not subtracted
- E23: limitation: F3: growth evidence is from S1, not S2; not subtracted
- E23: limitation: F4: growth evidence is from S1, not S2; not subtracted
- E23: limitation: F5: growth evidence is from S1, not S2; not subtracted
- E23: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E23: limitation: threshold is provisional (not calibrated)
- E24: partial coverage (1 of 3 examined)
- E24: limitation: F1: growth evidence is from S1, not S3; not subtracted
- E24: limitation: F3: growth evidence is from S1, not S3; not subtracted
- E24: limitation: F4: growth evidence is from S1, not S3; not subtracted
- E24: limitation: F5: growth evidence is from S1, not S3; not subtracted
- E24: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E24: limitation: threshold is provisional (not calibrated)
- E25: partial coverage (1 of 3 examined)
- E25: limitation: F1: growth evidence is from S1, not S4; not subtracted
- E25: limitation: F3: growth evidence is from S1, not S4; not subtracted
- E25: limitation: F4: growth evidence is from S1, not S4; not subtracted
- E25: limitation: F5: growth evidence is from S1, not S4; not subtracted
- E25: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E25: limitation: threshold is provisional (not calibrated)
- E26: partial coverage (1 of 3 examined)
- E26: limitation: F1: growth evidence is from S1, not S5; not subtracted
- E26: limitation: F3: growth evidence is from S1, not S5; not subtracted
- E26: limitation: F4: growth evidence is from S1, not S5; not subtracted
- E26: limitation: F5: growth evidence is from S1, not S5; not subtracted
- E26: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E26: limitation: threshold is provisional (not calibrated)
- E27: partial coverage (1 of 3 examined)
- E27: limitation: F1: growth evidence is from S1, not S6; not subtracted
- E27: limitation: F3: growth evidence is from S1, not S6; not subtracted
- E27: limitation: F4: growth evidence is from S1, not S6; not subtracted
- E27: limitation: F5: growth evidence is from S1, not S6; not subtracted
- E27: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E27: limitation: threshold is provisional (not calibrated)
- E28: partial coverage (1 of 3 examined)
- E28: limitation: F1: growth evidence is from S1, not S7; not subtracted
- E28: limitation: F3: growth evidence is from S1, not S7; not subtracted
- E28: limitation: F4: growth evidence is from S1, not S7; not subtracted
- E28: limitation: F5: growth evidence is from S1, not S7; not subtracted
- E28: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E28: limitation: threshold is provisional (not calibrated)
- E29: partial coverage (1 of 3 examined)
- E29: limitation: F1: growth evidence is from S1, not S8; not subtracted
- E29: limitation: F3: growth evidence is from S1, not S8; not subtracted
- E29: limitation: F4: growth evidence is from S1, not S8; not subtracted
- E29: limitation: F5: growth evidence is from S1, not S8; not subtracted
- E29: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E29: limitation: threshold is provisional (not calibrated)
- E30: partial coverage (1 of 3 examined)
- E30: limitation: F1: growth measured as a count; bytes not attributed
- E30: limitation: F3: growth measured as a count; bytes not attributed
- E30: limitation: F4: growth measured as a count; bytes not attributed
- E30: limitation: F5: growth measured as a count; bytes not attributed
- E30: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E30: limitation: threshold is provisional (not calibrated)
- E31: partial coverage (1 of 3 examined)
- E31: limitation: F1: growth evidence is from S1, not S2; not subtracted
- E31: limitation: F3: growth evidence is from S1, not S2; not subtracted
- E31: limitation: F4: growth evidence is from S1, not S2; not subtracted
- E31: limitation: F5: growth evidence is from S1, not S2; not subtracted
- E31: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E31: limitation: threshold is provisional (not calibrated)
- E32: partial coverage (1 of 3 examined)
- E32: limitation: F1: growth evidence is from S1, not S3; not subtracted
- E32: limitation: F3: growth evidence is from S1, not S3; not subtracted
- E32: limitation: F4: growth evidence is from S1, not S3; not subtracted
- E32: limitation: F5: growth evidence is from S1, not S3; not subtracted
- E32: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E32: limitation: threshold is provisional (not calibrated)
- E33: partial coverage (1 of 3 examined)
- E33: limitation: F1: growth evidence is from S1, not S4; not subtracted
- E33: limitation: F3: growth evidence is from S1, not S4; not subtracted
- E33: limitation: F4: growth evidence is from S1, not S4; not subtracted
- E33: limitation: F5: growth evidence is from S1, not S4; not subtracted
- E33: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E33: limitation: threshold is provisional (not calibrated)
- E34: partial coverage (1 of 3 examined)
- E34: limitation: F1: growth evidence is from S1, not S5; not subtracted
- E34: limitation: F3: growth evidence is from S1, not S5; not subtracted
- E34: limitation: F4: growth evidence is from S1, not S5; not subtracted
- E34: limitation: F5: growth evidence is from S1, not S5; not subtracted
- E34: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E34: limitation: threshold is provisional (not calibrated)
- E35: partial coverage (1 of 3 examined)
- E35: limitation: F1: growth evidence is from S1, not S6; not subtracted
- E35: limitation: F3: growth evidence is from S1, not S6; not subtracted
- E35: limitation: F4: growth evidence is from S1, not S6; not subtracted
- E35: limitation: F5: growth evidence is from S1, not S6; not subtracted
- E35: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E35: limitation: threshold is provisional (not calibrated)
- E36: partial coverage (1 of 3 examined)
- E36: limitation: F1: growth evidence is from S1, not S7; not subtracted
- E36: limitation: F3: growth evidence is from S1, not S7; not subtracted
- E36: limitation: F4: growth evidence is from S1, not S7; not subtracted
- E36: limitation: F5: growth evidence is from S1, not S7; not subtracted
- E36: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E36: limitation: threshold is provisional (not calibrated)
- E37: partial coverage (1 of 3 examined)
- E37: limitation: F1: growth evidence is from S1, not S8; not subtracted
- E37: limitation: F3: growth evidence is from S1, not S8; not subtracted
- E37: limitation: F4: growth evidence is from S1, not S8; not subtracted
- E37: limitation: F5: growth evidence is from S1, not S8; not subtracted
- E37: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E37: limitation: threshold is provisional (not calibrated)

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
| E4 | static-hit | static | dla 0.1.0 | repo 8ba6ce044aa1c1ebf7e8fc28516f4590d46dd902 | sweep/rust-dla.json |
| E5 | census-holder | static | dla-census 0.1.0 | repo 8ba6ce044aa1c1ebf7e8fc28516f4590d46dd902 | census/rust.json |
| E6 | workload | static | dla 0.1.0 | repo 8ba6ce044aa1c1ebf7e8fc28516f4590d46dd902 | workload/workloads.json |
| E7 | workload | static | dla 0.1.0 | repo 8ba6ce044aa1c1ebf7e8fc28516f4590d46dd902 | harness/H1/harness.json |
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (e44bf49a82b7) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (7cef6e50979a) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (7cef6e50979a) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S2.json (9caae0494d62) | alloc/A1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S1.json (7cef6e50979a) | trend/T3.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (7cef6e50979a) | trend/T3.json |
| E14 | growth-series | test | dla 0.1.0 | artifact series/S2.json (9caae0494d62) | trend/T4.json |
| E15 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (9caae0494d62) | trend/T4.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S3.json (2dac48f65722) | alloc/A2.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S4.json (d46b9c00dfa3) | alloc/A3.json |
| E18 | discriminator | test | dla 0.1.0 | artifact series/S5.json (0c191e7bf9c5) | alloc/A4.json |
| E19 | discriminator | test | dla 0.1.0 | artifact series/S6.json (fae3c0649e11) | alloc/A5.json |
| E20 | discriminator | test | dla 0.1.0 | artifact series/S7.json (f81a8bc40a5d) | alloc/A6.json |
| E21 | discriminator | test | dla 0.1.0 | artifact series/S8.json (b5f01c0508ac) | alloc/A7.json |
| E22 | residual | test | dla 0.1.0 | artifact series/S1.json (7cef6e50979a) | gap/residual-S1-E22.json |
| E23 | residual | test | dla 0.1.0 | artifact series/S2.json (9caae0494d62) | gap/residual-S2-E23.json |
| E24 | residual | test | dla 0.1.0 | artifact series/S3.json (2dac48f65722) | gap/residual-S3-E24.json |
| E25 | residual | test | dla 0.1.0 | artifact series/S4.json (d46b9c00dfa3) | gap/residual-S4-E25.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S5.json (0c191e7bf9c5) | gap/residual-S5-E26.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S6.json (fae3c0649e11) | gap/residual-S6-E27.json |
| E28 | residual | test | dla 0.1.0 | artifact series/S7.json (f81a8bc40a5d) | gap/residual-S7-E28.json |
| E29 | residual | test | dla 0.1.0 | artifact series/S8.json (b5f01c0508ac) | gap/residual-S8-E29.json |
| E30 | residual | test | dla 0.1.0 | artifact series/S1.json (7cef6e50979a) | gap/residual-S1-E30.json |
| E31 | residual | test | dla 0.1.0 | artifact series/S2.json (9caae0494d62) | gap/residual-S2-E31.json |
| E32 | residual | test | dla 0.1.0 | artifact series/S3.json (2dac48f65722) | gap/residual-S3-E32.json |
| E33 | residual | test | dla 0.1.0 | artifact series/S4.json (d46b9c00dfa3) | gap/residual-S4-E33.json |
| E34 | residual | test | dla 0.1.0 | artifact series/S5.json (0c191e7bf9c5) | gap/residual-S5-E34.json |
| E35 | residual | test | dla 0.1.0 | artifact series/S6.json (fae3c0649e11) | gap/residual-S6-E35.json |
| E36 | residual | test | dla 0.1.0 | artifact series/S7.json (f81a8bc40a5d) | gap/residual-S7-E36.json |
| E37 | residual | test | dla 0.1.0 | artifact series/S8.json (b5f01c0508ac) | gap/residual-S8-E37.json |
