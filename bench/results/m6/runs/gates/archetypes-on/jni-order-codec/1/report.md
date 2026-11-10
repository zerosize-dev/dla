# DLA report

Run: `20261110-160751-7b73`

Status: INCOMPLETE (231 open)

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 23 enumerated, 23 dispositioned; confirmed 1, strong 1, refuted 9, suspected 12, unresolved 0

Residual: heap 1451.0784684684686 bytes/n unattributed of 1451.0784684684686 total (threshold 362.7696171171171) ABOVE THRESHOLD, see O1; non-heap 10.443603603603604 bytes/n unattributed of 10.443603603603604 total (threshold 41.32813329729724)

Coverage: jvm 17760 iterations, 0 of 17 entry points exercised; rust no iterations, 0 of 10 entry points exercised

## Open items

- **T3:os.rss**: column os.rss of T3 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:os.rss_anon**: column os.rss_anon of T3 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:jvm.heap_used**: column jvm.heap_used of T3 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:native.live_blocks**: column native.live_blocks of T3 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:native.live_bytes**: column native.live_bytes of T3 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:native.peak_bytes**: column native.peak_bytes of T3 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:jvm.class:byte[].count**: column jvm.class:byte[].count of T3 (steady) is not flat and is tied to no finding. Fix: `dla trend S4`
- **T3:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T3 (steady) is not flat and is tied to no finding. Fix: `dla trend S4`
- **T3:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T3 (steady) is not flat and is tied to no finding. Fix: `dla trend S4`
- **T4:os.rss**: column os.rss of T4 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:os.rss_anon**: column os.rss_anon of T4 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:jvm.heap_used**: column jvm.heap_used of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:native.live_blocks**: column native.live_blocks of T4 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:native.live_bytes**: column native.live_bytes of T4 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:native.peak_bytes**: column native.peak_bytes of T4 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:jvm.heap_committed.pre_gc**: column jvm.heap_committed.pre_gc of T4 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S6 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:jvm.class:byte[].count**: column jvm.class:byte[].count of T4 (steady) is not flat and is tied to no finding. Fix: `dla trend S6`
- **T4:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T4 (steady) is not flat and is tied to no finding. Fix: `dla trend S6`
- **T4:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T4 (steady) is not flat and is tied to no finding. Fix: `dla trend S6`
- **T5:os.rss**: column os.rss of T5 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.rss_anon**: column os.rss_anon of T5 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.vm_hwm**: column os.vm_hwm of T5 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:jvm.heap_committed**: column jvm.heap_committed of T5 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:jvm.heap_used**: column jvm.heap_used of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:jvm.nmt_committed**: column jvm.nmt_committed of T5 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:native.live_blocks**: column native.live_blocks of T5 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:native.live_bytes**: column native.live_bytes of T5 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:native.peak_bytes**: column native.peak_bytes of T5 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:jvm.heap_used.pre_gc**: column jvm.heap_used.pre_gc of T5 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:jvm.class:byte[].count**: column jvm.class:byte[].count of T5 (steady) is not flat and is tied to no finding. Fix: `dla trend S8`
- **T5:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T5 (steady) is not flat and is tied to no finding. Fix: `dla trend S8`
- **T5:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T5 (steady) is not flat and is tied to no finding. Fix: `dla trend S8`
- **T6:os.rss**: column os.rss of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S10 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:os.rss_anon**: column os.rss_anon of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S10 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:os.vm_hwm**: column os.vm_hwm of T6 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S10 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:jvm.heap_committed**: column jvm.heap_committed of T6 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S10 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:jvm.heap_used**: column jvm.heap_used of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S10 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:jvm.nmt_committed**: column jvm.nmt_committed of T6 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S10 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:native.live_blocks**: column native.live_blocks of T6 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S10 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:native.live_bytes**: column native.live_bytes of T6 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S10 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:native.peak_bytes**: column native.peak_bytes of T6 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S10 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:jvm.heap_committed.pre_gc**: column jvm.heap_committed.pre_gc of T6 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S10 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:jvm.heap_used.pre_gc**: column jvm.heap_used.pre_gc of T6 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S10 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:jvm.class:byte[].count**: column jvm.class:byte[].count of T6 (steady) is not flat and is tied to no finding. Fix: `dla trend S10`
- **T6:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T6 (steady) is not flat and is tied to no finding. Fix: `dla trend S10`
- **T6:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T6 (steady) is not flat and is tied to no finding. Fix: `dla trend S10`
- **T7:os.rss**: column os.rss of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S12 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:os.rss_anon**: column os.rss_anon of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S12 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:os.vm_hwm**: column os.vm_hwm of T7 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S12 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:jvm.heap_used**: column jvm.heap_used of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S12 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:native.live_blocks**: column native.live_blocks of T7 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S12 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:native.live_bytes**: column native.live_bytes of T7 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S12 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:native.peak_bytes**: column native.peak_bytes of T7 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S12 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:jvm.heap_committed.pre_gc**: column jvm.heap_committed.pre_gc of T7 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S12 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:jvm.class:byte[].count**: column jvm.class:byte[].count of T7 (steady) is not flat and is tied to no finding. Fix: `dla trend S12`
- **T7:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla trend S12`
- **T7:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla trend S12`
- **T8:os.rss**: column os.rss of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S14 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:os.rss_anon**: column os.rss_anon of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S14 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:os.vm_data**: column os.vm_data of T8 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S14 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:os.vm_hwm**: column os.vm_hwm of T8 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S14 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:jvm.heap_committed**: column jvm.heap_committed of T8 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S14 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:jvm.heap_used**: column jvm.heap_used of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S14 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:jvm.nmt_committed**: column jvm.nmt_committed of T8 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S14 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:native.live_blocks**: column native.live_blocks of T8 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S14 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:native.live_bytes**: column native.live_bytes of T8 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S14 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:native.peak_bytes**: column native.peak_bytes of T8 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S14 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:jvm.heap_committed.pre_gc**: column jvm.heap_committed.pre_gc of T8 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S14 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:jvm.heap_used.pre_gc**: column jvm.heap_used.pre_gc of T8 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S14 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:jvm.class:byte[].count**: column jvm.class:byte[].count of T8 (steady) is not flat and is tied to no finding. Fix: `dla trend S14`
- **T8:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla trend S14`
- **T8:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla trend S14`
- **T9:os.rss**: column os.rss of T9 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S16 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:os.rss_anon**: column os.rss_anon of T9 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S16 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:os.vm_hwm**: column os.vm_hwm of T9 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S16 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:jvm.heap_used**: column jvm.heap_used of T9 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S16 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:native.live_blocks**: column native.live_blocks of T9 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S16 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:native.live_bytes**: column native.live_bytes of T9 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S16 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:native.peak_bytes**: column native.peak_bytes of T9 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S16 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:jvm.heap_used.pre_gc**: column jvm.heap_used.pre_gc of T9 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S16 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:jvm.class:byte[].count**: column jvm.class:byte[].count of T9 (steady) is not flat and is tied to no finding. Fix: `dla trend S16`
- **T9:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T9 (steady) is not flat and is tied to no finding. Fix: `dla trend S16`
- **T9:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T9 (steady) is not flat and is tied to no finding. Fix: `dla trend S16`
- **T10:os.rss**: column os.rss of T10 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S18 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T10:os.rss_anon**: column os.rss_anon of T10 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S18 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T10:os.vm_hwm**: column os.vm_hwm of T10 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S18 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T10:jvm.heap_used**: column jvm.heap_used of T10 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S18 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T10:jvm.nmt_committed**: column jvm.nmt_committed of T10 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S18 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T10:native.live_blocks**: column native.live_blocks of T10 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S18 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T10:native.live_bytes**: column native.live_bytes of T10 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S18 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T10:native.peak_bytes**: column native.peak_bytes of T10 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S18 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T10:jvm.heap_committed.pre_gc**: column jvm.heap_committed.pre_gc of T10 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S18 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T10:jvm.class:byte[].count**: column jvm.class:byte[].count of T10 (steady) is not flat and is tied to no finding. Fix: `dla trend S18`
- **T10:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T10 (steady) is not flat and is tied to no finding. Fix: `dla trend S18`
- **T10:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T10 (steady) is not flat and is tied to no finding. Fix: `dla trend S18`
- **T11:os.rss**: column os.rss of T11 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S20 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T11:os.rss_anon**: column os.rss_anon of T11 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S20 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T11:os.vm_hwm**: column os.vm_hwm of T11 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S20 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T11:jvm.heap_committed**: column jvm.heap_committed of T11 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S20 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T11:jvm.heap_used**: column jvm.heap_used of T11 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S20 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T11:native.live_blocks**: column native.live_blocks of T11 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S20 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T11:native.live_bytes**: column native.live_bytes of T11 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S20 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T11:native.peak_bytes**: column native.peak_bytes of T11 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S20 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T11:jvm.class:byte[].count**: column jvm.class:byte[].count of T11 (steady) is not flat and is tied to no finding. Fix: `dla trend S20`
- **T11:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T11 (steady) is not flat and is tied to no finding. Fix: `dla trend S20`
- **T11:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T11 (steady) is not flat and is tied to no finding. Fix: `dla trend S20`
- **T12:os.rss**: column os.rss of T12 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S22 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T12:os.rss_anon**: column os.rss_anon of T12 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S22 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T12:os.vm_hwm**: column os.vm_hwm of T12 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S22 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T12:jvm.heap_committed**: column jvm.heap_committed of T12 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S22 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T12:jvm.heap_used**: column jvm.heap_used of T12 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S22 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T12:jvm.nmt_committed**: column jvm.nmt_committed of T12 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S22 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T12:native.live_blocks**: column native.live_blocks of T12 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S22 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T12:native.live_bytes**: column native.live_bytes of T12 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S22 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T12:native.peak_bytes**: column native.peak_bytes of T12 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S22 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T12:jvm.heap_committed.pre_gc**: column jvm.heap_committed.pre_gc of T12 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S22 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T12:jvm.class:byte[].count**: column jvm.class:byte[].count of T12 (steady) is not flat and is tied to no finding. Fix: `dla trend S22`
- **T12:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T12 (steady) is not flat and is tied to no finding. Fix: `dla trend S22`
- **T12:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T12 (steady) is not flat and is tied to no finding. Fix: `dla trend S22`
- **T13:os.rss**: column os.rss of T13 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S23 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T13:os.rss_anon**: column os.rss_anon of T13 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S23 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T13:os.vm_hwm**: column os.vm_hwm of T13 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S23 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T13:native.live_blocks**: column native.live_blocks of T13 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S23 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T13:native.live_bytes**: column native.live_bytes of T13 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S23 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T13:native.peak_bytes**: column native.peak_bytes of T13 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S23 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T13:jvm.heap_committed.pre_gc**: column jvm.heap_committed.pre_gc of T13 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S23 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T13:jvm.heap_used.pre_gc**: column jvm.heap_used.pre_gc of T13 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S23 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T15:jvm.heap_committed**: column jvm.heap_committed of T15 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T15:jvm.heap_used**: column jvm.heap_used of T15 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T15:jvm.nonheap_used**: column jvm.nonheap_used of T15 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T15:native.live_blocks**: column native.live_blocks of T15 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T15:jvm.heap_committed.pre_gc**: column jvm.heap_committed.pre_gc of T15 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T15:jvm.class:byte[].count**: column jvm.class:byte[].count of T15 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E7 # column jvm.class:byte[].count`
- **T15:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T15 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E7 # column jvm.class:java.lang.String.count`
- **T15:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T15 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E7 # column jvm.class:shop.codec.Entry.count`
- **T16:os.rss**: column os.rss of T16 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T16:os.rss_anon**: column os.rss_anon of T16 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T16:os.vm_hwm**: column os.vm_hwm of T16 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T16:jvm.heap_used**: column jvm.heap_used of T16 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T16:native.live_blocks**: column native.live_blocks of T16 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T16:native.live_bytes**: column native.live_bytes of T16 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T16:native.peak_bytes**: column native.peak_bytes of T16 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T16:jvm.class:byte[].count**: column jvm.class:byte[].count of T16 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E9 # column jvm.class:byte[].count`
- **T16:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T16 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E9 # column jvm.class:java.lang.String.count`
- **T16:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T16 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E9 # column jvm.class:shop.codec.Entry.count`
- **T17:os.rss**: column os.rss of T17 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T17:os.rss_anon**: column os.rss_anon of T17 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T17:os.vm_hwm**: column os.vm_hwm of T17 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T17:jvm.heap_used**: column jvm.heap_used of T17 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T17:native.live_blocks**: column native.live_blocks of T17 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T17:native.live_bytes**: column native.live_bytes of T17 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T17:native.peak_bytes**: column native.peak_bytes of T17 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T17:jvm.class:byte[].count**: column jvm.class:byte[].count of T17 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E11 # column jvm.class:byte[].count`
- **T17:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T17 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E11 # column jvm.class:java.lang.String.count`
- **T17:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T17 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E11 # column jvm.class:shop.codec.Entry.count`
- **T18:os.rss**: column os.rss of T18 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T18:os.rss_anon**: column os.rss_anon of T18 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T18:os.vm_hwm**: column os.vm_hwm of T18 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T18:jvm.heap_used**: column jvm.heap_used of T18 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T18:native.live_blocks**: column native.live_blocks of T18 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T18:native.live_bytes**: column native.live_bytes of T18 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T18:native.peak_bytes**: column native.peak_bytes of T18 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T18:jvm.class:byte[].count**: column jvm.class:byte[].count of T18 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E13 # column jvm.class:byte[].count`
- **T18:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T18 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E13 # column jvm.class:java.lang.String.count`
- **T18:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T18 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E13 # column jvm.class:shop.codec.Entry.count`
- **T19:os.rss**: column os.rss of T19 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S11 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T19:os.rss_anon**: column os.rss_anon of T19 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S11 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T19:os.vm_hwm**: column os.vm_hwm of T19 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S11 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T19:jvm.heap_committed**: column jvm.heap_committed of T19 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S11 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T19:jvm.heap_used**: column jvm.heap_used of T19 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S11 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T19:native.live_blocks**: column native.live_blocks of T19 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S11 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T19:native.live_bytes**: column native.live_bytes of T19 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S11 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T19:native.peak_bytes**: column native.peak_bytes of T19 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S11 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T19:jvm.class:byte[].count**: column jvm.class:byte[].count of T19 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E15 # column jvm.class:byte[].count`
- **T19:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T19 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E15 # column jvm.class:java.lang.String.count`
- **T19:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T19 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E15 # column jvm.class:shop.codec.Entry.count`
- **T20:os.rss**: column os.rss of T20 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S13 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T20:os.rss_anon**: column os.rss_anon of T20 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S13 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T20:os.vm_data**: column os.vm_data of T20 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S13 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T20:os.vm_hwm**: column os.vm_hwm of T20 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S13 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T20:jvm.heap_committed**: column jvm.heap_committed of T20 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S13 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T20:jvm.heap_used**: column jvm.heap_used of T20 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S13 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T20:jvm.nmt_committed**: column jvm.nmt_committed of T20 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S13 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T20:native.live_blocks**: column native.live_blocks of T20 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S13 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T20:native.live_bytes**: column native.live_bytes of T20 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S13 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T20:native.peak_bytes**: column native.peak_bytes of T20 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S13 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T20:jvm.heap_committed.pre_gc**: column jvm.heap_committed.pre_gc of T20 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S13 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T20:jvm.class:byte[].count**: column jvm.class:byte[].count of T20 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E17 # column jvm.class:byte[].count`
- **T20:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T20 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E17 # column jvm.class:java.lang.String.count`
- **T20:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T20 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E17 # column jvm.class:shop.codec.Entry.count`
- **T21:os.rss**: column os.rss of T21 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S15 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T21:os.rss_anon**: column os.rss_anon of T21 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S15 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T21:os.vm_hwm**: column os.vm_hwm of T21 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S15 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T21:jvm.heap_used**: column jvm.heap_used of T21 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S15 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T21:native.live_blocks**: column native.live_blocks of T21 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S15 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T21:native.live_bytes**: column native.live_bytes of T21 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S15 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T21:native.peak_bytes**: column native.peak_bytes of T21 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S15 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T21:jvm.heap_used.pre_gc**: column jvm.heap_used.pre_gc of T21 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S15 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T21:jvm.class:byte[].count**: column jvm.class:byte[].count of T21 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E19 # column jvm.class:byte[].count`
- **T21:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T21 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E19 # column jvm.class:java.lang.String.count`
- **T21:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T21 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E19 # column jvm.class:shop.codec.Entry.count`
- **T22:os.rss**: column os.rss of T22 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S17 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T22:os.rss_anon**: column os.rss_anon of T22 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S17 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T22:os.vm_hwm**: column os.vm_hwm of T22 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S17 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T22:jvm.heap_used**: column jvm.heap_used of T22 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S17 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T22:native.live_blocks**: column native.live_blocks of T22 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S17 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T22:native.live_bytes**: column native.live_bytes of T22 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S17 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T22:native.peak_bytes**: column native.peak_bytes of T22 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S17 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T22:jvm.class:byte[].count**: column jvm.class:byte[].count of T22 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E21 # column jvm.class:byte[].count`
- **T22:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T22 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E21 # column jvm.class:java.lang.String.count`
- **T22:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T22 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E21 # column jvm.class:shop.codec.Entry.count`
- **T23:os.rss_anon**: column os.rss_anon of T23 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S19 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T23:os.rss_file**: column os.rss_file of T23 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S19 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T23:os.vm_hwm**: column os.vm_hwm of T23 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S19 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T23:jvm.heap_used**: column jvm.heap_used of T23 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S19 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T23:native.live_blocks**: column native.live_blocks of T23 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S19 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T23:native.live_bytes**: column native.live_bytes of T23 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S19 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T23:native.peak_bytes**: column native.peak_bytes of T23 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S19 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T23:jvm.class:byte[].count**: column jvm.class:byte[].count of T23 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E23 # column jvm.class:byte[].count`
- **T23:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T23 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E23 # column jvm.class:java.lang.String.count`
- **T23:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T23 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E23 # column jvm.class:shop.codec.Entry.count`
- **T24:os.rss**: column os.rss of T24 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S21 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T24:os.rss_anon**: column os.rss_anon of T24 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S21 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T24:os.vm_hwm**: column os.vm_hwm of T24 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S21 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T24:jvm.heap_used**: column jvm.heap_used of T24 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S21 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T24:native.live_blocks**: column native.live_blocks of T24 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S21 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T24:native.live_bytes**: column native.live_bytes of T24 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S21 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T24:native.peak_bytes**: column native.peak_bytes of T24 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S21 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T24:jvm.class:byte[].count**: column jvm.class:byte[].count of T24 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E25 # column jvm.class:byte[].count`
- **T24:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T24 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E25 # column jvm.class:java.lang.String.count`
- **T24:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T24 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E25 # column jvm.class:shop.codec.Entry.count`

## Workers

- **jvm** (jvm): status done; 84 lines merged; slice done; torn tail: no
- **rust** (rust): status done; 11 lines merged; slice done; torn tail: no

## Findings

### jvm/F3: **L-COLLECT**

- State: strong
- Label: L-COLLECT (evidence: jvm/E17 growth-series, jvm/E13 retention-path)
- Growth: `holder:static:shop.codec.Audit.trail` 8501451 bytes/n (steady, gate pass, jvm/E17)
- Evidence classes: context, growth, intervention, static
- Strong: second class jvm/E29 intervention that removed the growth, independent of the growth record jvm/E17 (dla-hprof)
- Holder: static/global
- Owner: `shop.codec.Audit.trail`
- Region: heap
- Shape: steady
- Site: `kotlin/shop/Audit.kt:9`
- Writers: `kotlin/shop/Audit.kt:12`
- Annotation: heuristic name match over the project's sources
- Annotation: trail only ever added to; no remover
- Verified: no
- Sibling hypotheses: jvm/F3.n1
- Supporting evidence: jvm/E4, jvm/E7, jvm/E17, jvm/E12, jvm/E29
- Explanation (inferred): Audit.trail (static holder in object Audit, Audit.kt:9) gets an Entry with a 1536-byte sample added by Audit.record at Audit.kt:12 for every inline frame; it is never cleared and nothing removes entries, so the list grows with each order.
- Explanation structure: adds: kotlin/shop/Audit.kt:12; removal: never

### jvm/F19: **UNCLASSIFIED**

- State: confirmed
- Growth: `native.live_blocks@native/src/lib.rs:40` 0.125 count/n (steady, gate pass, jvm/E12)
- Evidence classes: allocation, context, discriminator, growth
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.open`
- Region: off-heap
- Shape: steady
- Site: `native/src/lib.rs:40`
- Writers: `kotlin/shop/Native.kt:9`, `kotlin/shop/Encoder.kt:12`
- Removers: `kotlin/shop/Encoder.kt:21`
- Annotation: native allocation under shop.codec.Native.open (Java_shop_codec_Native_open); found by dla correlate
- Annotation: bulk branch returns without Native.close(handle)
- Annotation: close is only called on the inline path; bulk path returns early
- Verified: no
- Sibling hypotheses: jvm/F19.n1
- Supporting evidence: jvm/E10, jvm/E7, jvm/E12, jvm/E20
- Explanation (inferred): Native.open allocates a native frame handle (native/src/lib.rs:40, native-handle holder); Native.close at Encoder.kt:21 runs only when the frame is within 16384 bytes, and the bulk branch returns early, so those handles are never released.
- Explanation structure: adds: native/src/lib.rs:40; removal: conditional at kotlin/shop/Encoder.kt:21

### jvm/F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Site: `kotlin/shop/Audit.kt:9`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: detector hit duplicates F3 (same site Audit.kt:9), confirmed there
- Supporting evidence: jvm/E3

### jvm/F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.Entry.sample`
- Region: heap
- Site: `kotlin/shop/Audit.kt:3`
- Reason: not_exercised
- Remedy: Entry.sample is the element retained by Audit.trail (F3, confirmed); no separate holder
- Supporting evidence: jvm/E4

### jvm/F7: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.<clinit>`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:6`
- Reason: not_exercised
- Remedy: JNI declaration only; the native handle growth is confirmed as F19 (Encoder bulk path never closes handle)
- Supporting evidence: jvm/E4

### jvm/F8: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.open`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:9`
- Reason: not_exercised
- Remedy: JNI declaration only; the native handle growth is confirmed as F19 (Encoder bulk path never closes handle)
- Supporting evidence: jvm/E4

### jvm/F9: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.checksum`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:10`
- Reason: not_exercised
- Remedy: JNI declaration only; the native handle growth is confirmed as F19 (Encoder bulk path never closes handle)
- Supporting evidence: jvm/E4

### jvm/F10: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.size`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:11`
- Reason: not_exercised
- Remedy: JNI declaration only; the native handle growth is confirmed as F19 (Encoder bulk path never closes handle)
- Supporting evidence: jvm/E4

### jvm/F11: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.lines`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:12`
- Reason: not_exercised
- Remedy: JNI declaration only; the native handle growth is confirmed as F19 (Encoder bulk path never closes handle)
- Supporting evidence: jvm/E4

### jvm/F12: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.close`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:13`
- Reason: not_exercised
- Remedy: JNI declaration only; the native handle growth is confirmed as F19 (Encoder bulk path never closes handle)
- Supporting evidence: jvm/E4

### jvm/F13: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.live`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:14`
- Reason: not_exercised
- Remedy: JNI declaration only; the native handle growth is confirmed as F19 (Encoder bulk path never closes handle)
- Supporting evidence: jvm/E4

### jvm/F14: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.liveBytes`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:15`
- Reason: not_exercised
- Remedy: JNI declaration only; the native handle growth is confirmed as F19 (Encoder bulk path never closes handle)
- Supporting evidence: jvm/E4

### rust/F1: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `native.src.frame.Frame.bytes`
- Region: heap
- Site: `native/src/frame.rs:11`
- Writers: `native/src/lib.rs:46`
- Removers: `native/src/lib.rs:79`
- Annotation: Frame.bytes is owned by the FRAMES entry; freed only when close removes the entry
- Reason: no_harness
- Remedy: Rust cdylib has no standalone entry (JNIEnv needed); measure from the JVM worker with the shim and run dla correlate H1 to tie native site native/src/lib.rs:46 to the managed finding for the Encoder.kt bulk path (no close for frames >16384 bytes)
- Supporting evidence: rust/E2

### rust/F2: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: static/global
- Owner: `native.src.lib.FRAMES`
- Region: heap
- Site: `native/src/lib.rs:19`
- Writers: `native/src/lib.rs:46`
- Removers: `native/src/lib.rs:79`
- Annotation: FRAMES map: open inserts, close removes; static review shows Encoder.kt returns early for frames >16384 bytes without Native.close, so bulk handles stay in FRAMES (cause in JVM caller; Rust side is the holder)
- Reason: no_harness
- Remedy: Rust cdylib has no standalone entry (JNIEnv needed); measure from the JVM worker with the shim and run dla correlate H1 to tie native site native/src/lib.rs:46 to the managed finding for the Encoder.kt bulk path (no close for frames >16384 bytes)
- Supporting evidence: rust/E2

## Ruled out

- **jvm/F3.n1** (UNCLASSIFIED): discriminator jvm/E19
  - Ruled out by: plateau+purge (leak) jvm/E19
  - Hypothesis: N-WARM: Audit.trail growth is a warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: jvm/E19
  - Originally supported by: jvm/E12
- **jvm/F4** (UNCLASSIFIED): discriminator jvm/E23
  - Ruled out by: plateau+purge (non-leak) jvm/E23
  - Contradicting evidence: jvm/E23
  - Originally supported by: jvm/E4
- **jvm/F5** (UNCLASSIFIED): discriminator jvm/E21
  - Ruled out by: plateau+purge (non-leak) jvm/E21
  - Contradicting evidence: jvm/E21
  - Originally supported by: jvm/E4
- **jvm/F6** (UNCLASSIFIED): discriminator jvm/E24
  - Ruled out by: plateau+purge (non-leak) jvm/E24
  - Contradicting evidence: jvm/E24
  - Originally supported by: jvm/E4
- **jvm/F15** (UNCLASSIFIED): discriminator jvm/E25
  - Ruled out by: plateau+purge (non-leak) jvm/E25
  - Contradicting evidence: jvm/E25
  - Originally supported by: jvm/E4
- **jvm/F16** (UNCLASSIFIED): discriminator jvm/E27
  - Ruled out by: plateau+purge (non-leak) jvm/E27
  - Contradicting evidence: jvm/E27
  - Originally supported by: jvm/E4
- **jvm/F17** (UNCLASSIFIED): discriminator jvm/E28
  - Ruled out by: plateau+purge (non-leak) jvm/E28
  - Contradicting evidence: jvm/E28
  - Originally supported by: jvm/E4
- **jvm/F18** (UNCLASSIFIED): discriminator jvm/E26
  - Ruled out by: plateau+purge (non-leak) jvm/E26
  - Contradicting evidence: jvm/E26
  - Originally supported by: jvm/E4
- **jvm/F19.n1** (UNCLASSIFIED): discriminator jvm/E20
  - Ruled out by: plateau+purge (leak) jvm/E20
  - Hypothesis: N-HWM: native live bytes are a pool high-water mark that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: jvm/E20
  - Originally supported by: jvm/E12

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

- E4: limitation: series: per-point; x is the iteration count n of each point (2220, 8880, 13320, 19980); every point is a separate process and heap dump
- E4: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E4: limitation: holders absent from a dump's 200 largest have no figure at that point
- E4: limitation: gate thresholds are provisional (not calibrated)
- E5: limitation: series: per-point; x is the iteration count n of each point (2220, 8880, 13320, 19980); every point is a separate process and heap dump
- E5: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E5: limitation: holders absent from a dump's 200 largest have no figure at that point
- E5: limitation: gate thresholds are provisional (not calibrated)
- E6: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E6: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E6: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E6: limitation: 21567672 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: 20253190 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: 29889806 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: 29072665 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: 32254955 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: 21577891 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: 32533949 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E6: limitation: 21080386 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E6: limitation: gate thresholds are provisional (not calibrated)
- E7: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E7: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E7: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E7: limitation: 21567672 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: 20253190 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: 29889806 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: 29072665 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: 32254955 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: 21577891 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: 32533949 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E7: limitation: 21080386 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E7: limitation: gate thresholds are provisional (not calibrated)
- E8: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E8: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E8: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E8: limitation: 32371207 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: 30859183 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: 22570586 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: 21604238 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: 35638838 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: 26207464 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: 34133456 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E8: limitation: 21066961 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E8: limitation: gate thresholds are provisional (not calibrated)
- E9: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E9: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E9: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E9: limitation: 32371207 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: 30859183 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: 22570586 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: 21604238 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: 35638838 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: 26207464 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: 34133456 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E9: limitation: 21066961 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E9: limitation: gate thresholds are provisional (not calibrated)
- E10: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E10: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E10: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E10: limitation: 33753176 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: 22401027 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: 31047192 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: 21685058 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: 21849703 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: 34139010 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: 30695563 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E10: limitation: 30848776 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E11: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E11: limitation: 33753176 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: 22401027 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: 31047192 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: 21685058 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: 21849703 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: 34139010 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: 30695563 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E11: limitation: 30848776 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E12: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E12: limitation: 31814778 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: 35695608 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: 21305002 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: 20689490 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: 31274555 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: 32323118 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: 31521086 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E12: limitation: 26976106 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E13: limitation: 31814778 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: 35695608 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: 21305002 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: 20689490 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: 31274555 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: 32323118 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: 31521086 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E13: limitation: 26976106 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E14: limitation: 33519586 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: 31796451 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: 21748689 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: 20089103 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: 29124298 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: 30485453 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: 22020112 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E14: limitation: 31315591 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E15: limitation: 33519586 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: 31796451 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: 21748689 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: 20089103 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: 29124298 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: 30485453 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: 22020112 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E15: limitation: 31315591 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E16: limitation: 22848954 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 31655852 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 21254544 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 23361449 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 30044503 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 30326628 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: 36102663 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E16: limitation: 32354513 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E17: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E17: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E17: limitation: 22848954 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 31655852 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 21254544 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 23361449 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 30044503 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 30326628 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: 36102663 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E17: limitation: 32354513 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E18: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E18: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E18: limitation: 30944055 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 24233539 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 30358665 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 21170216 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 29712508 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 32182210 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: 33975254 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E18: limitation: 33928156 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E19: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E19: limitation: 30944055 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 24233539 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 30358665 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 21170216 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 29712508 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 32182210 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: 33975254 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E19: limitation: 33928156 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E20: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E20: limitation: 34295303 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: 20753345 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: 34275977 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: 33444579 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: 20480690 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: 23412617 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: 29849369 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E20: limitation: 21547208 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E21: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E21: limitation: 34295303 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: 20753345 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: 34275977 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: 33444579 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: 20480690 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: 23412617 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: 29849369 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E21: limitation: 21547208 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E22: limitation: 29130186 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: 23682162 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: 31708106 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: 30229389 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: 20857488 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: 21156641 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: 22863177 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E22: limitation: 37554366 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E23: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E23: limitation: 29130186 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: 23682162 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: 31708106 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: 30229389 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: 20857488 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: 21156641 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: 22863177 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E23: limitation: 37554366 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E24: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E24: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E24: limitation: 30554893 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: 20991735 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: 19914759 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: 33087527 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: 33701073 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: 21165778 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: 30173507 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E24: limitation: 30032383 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E25: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E25: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E25: limitation: 30554893 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E25: limitation: 20991735 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E25: limitation: 19914759 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E25: limitation: 33087527 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E25: limitation: 33701073 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E25: limitation: 21165778 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E25: limitation: 30173507 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E25: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E25: limitation: 30032383 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E25: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: partial coverage (2 of 3 examined)
- E26: limitation: jvm/F3: growth measured as a count; bytes not attributed
- E26: limitation: jvm/F19: growth measured as a count; bytes not attributed
- E26: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E26: limitation: threshold is provisional (not calibrated)
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
- jvm/E7: limitation: 47165612 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E7: limitation: 21751582 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E8: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E8: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E8: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E8: limitation: 47165612 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 21751582 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 33148084 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 45501391 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 24778564 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 39716848 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 31820397 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 22557142 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 39486001 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E8: limitation: gate thresholds are provisional (not calibrated)
- jvm/E9: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E9: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E9: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E9: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E9: limitation: 47165612 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 21751582 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 33148084 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 45501391 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 24778564 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 39716848 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 31820397 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 22557142 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 39486001 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E9: limitation: gate thresholds are provisional (not calibrated)
- jvm/E10: partial coverage (1 of 1 examined)
- jvm/E10: limitation: the stack is the project frame of the largest allocation stack at the site, not every frame
- jvm/E10: limitation: slopes are per unit of the series axis; the thresholds are provisional
- jvm/E11: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E11: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E11: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E11: limitation: 47165612 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 21751582 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 33148084 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 45501391 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 24778564 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 39716848 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 31820397 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 22557142 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 39486001 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E11: limitation: gate thresholds are provisional (not calibrated)
- jvm/E12: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E12: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E12: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E12: limitation: 47165612 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 21751582 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 33148084 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 45501391 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 24778564 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 39716848 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 31820397 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 22557142 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 39486001 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E12: limitation: gate thresholds are provisional (not calibrated)
- jvm/E13: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- jvm/E14: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- jvm/E15: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- jvm/E16: limitation: byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)
- jvm/E17: limitation: dump-series rule: at least 3 dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- jvm/E17: limitation: series: per-point; one process per point at n = 2220, 8880, 13320, 19980 (harness captures; checked: HPROF version, identifier size)
- jvm/E18: limitation: dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)
- jvm/E19: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E19: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E19: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E19: limitation: 47165612 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 21751582 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 33148084 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 45501391 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 24778564 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 39716848 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 31820397 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 22557142 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 39486001 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E19: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E19: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E19: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E19: limitation: 21567672 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 20253190 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 29889806 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 29072665 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 32254955 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 21577891 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: 32533949 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E19: limitation: 21080386 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E19: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E19: limitation: points 1 to 9 come from S1; the extra points from S3
- jvm/E19: limitation: 9 columns beyond the 64 the series allows were dropped
- jvm/E19: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E19: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E19: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E19: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E20: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E20: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E20: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E20: limitation: 47165612 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 21751582 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 33148084 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 45501391 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 24778564 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 39716848 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 31820397 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 22557142 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 39486001 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E20: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E20: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E20: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E20: limitation: 32371207 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 30859183 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 22570586 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 21604238 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 35638838 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 26207464 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: 34133456 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E20: limitation: 21066961 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E20: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E20: limitation: points 1 to 9 come from S1; the extra points from S5
- jvm/E20: limitation: 9 columns beyond the 64 the series allows were dropped
- jvm/E20: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E20: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E20: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E20: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E21: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E21: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E21: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E21: limitation: 47165612 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 21751582 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 33148084 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 45501391 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 24778564 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 39716848 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 31820397 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 22557142 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 39486001 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E21: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E21: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E21: limitation: 33753176 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 22401027 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 31047192 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 21685058 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 21849703 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 34139010 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: 30695563 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E21: limitation: 30848776 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E21: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E21: limitation: points 1 to 9 come from S1; the extra points from S7
- jvm/E21: limitation: 2 columns beyond the 64 the series allows were dropped
- jvm/E21: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E21: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E21: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E21: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E22: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E22: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E22: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E22: limitation: 47165612 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 21751582 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 33148084 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 45501391 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 24778564 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 39716848 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 31820397 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 22557142 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 39486001 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E22: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E22: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E22: limitation: 31814778 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 35695608 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 21305002 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 20689490 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 31274555 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 32323118 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: 31521086 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E22: limitation: 26976106 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E22: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E22: limitation: points 1 to 9 come from S1; the extra points from S9
- jvm/E22: limitation: 8 columns beyond the 64 the series allows were dropped
- jvm/E22: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E22: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E22: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E22: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E23: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E23: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E23: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E23: limitation: 47165612 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: 21751582 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: 33148084 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: 45501391 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: 24778564 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: 39716848 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: 31820397 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: 22557142 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: 39486001 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E23: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E23: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E23: limitation: 33519586 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: 31796451 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: 21748689 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: 20089103 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: 29124298 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: 30485453 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: 22020112 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E23: limitation: 31315591 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E23: limitation: points 1 to 9 come from S1; the extra points from S11
- jvm/E23: limitation: 8 columns beyond the 64 the series allows were dropped
- jvm/E23: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E23: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E23: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E23: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E24: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E24: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E24: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E24: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E24: limitation: 47165612 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: 21751582 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: 33148084 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: 45501391 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: 24778564 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: 39716848 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: 31820397 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: 22557142 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: 39486001 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E24: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E24: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E24: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E24: limitation: 22848954 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: 31655852 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: 21254544 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: 23361449 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: 30044503 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: 30326628 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: 36102663 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E24: limitation: 32354513 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E24: limitation: points 1 to 9 come from S1; the extra points from S13
- jvm/E24: limitation: 8 columns beyond the 64 the series allows were dropped
- jvm/E24: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E24: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E24: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E24: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E25: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E25: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E25: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E25: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E25: limitation: 47165612 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: 21751582 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: 33148084 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: 45501391 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: 24778564 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: 39716848 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: 31820397 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: 22557142 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: 39486001 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E25: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E25: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E25: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E25: limitation: 30944055 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: 24233539 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: 30358665 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: 21170216 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: 29712508 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: 32182210 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: 33975254 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E25: limitation: 33928156 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E25: limitation: points 1 to 9 come from S1; the extra points from S15
- jvm/E25: limitation: 9 columns beyond the 64 the series allows were dropped
- jvm/E25: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E25: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E25: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E25: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E26: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E26: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E26: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E26: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E26: limitation: 47165612 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E26: limitation: 21751582 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E26: limitation: 33148084 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E26: limitation: 45501391 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E26: limitation: 24778564 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E26: limitation: 39716848 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E26: limitation: 31820397 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E26: limitation: 22557142 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E26: limitation: 39486001 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E26: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E26: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E26: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E26: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E26: limitation: 34295303 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E26: limitation: 20753345 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E26: limitation: 34275977 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E26: limitation: 33444579 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E26: limitation: 20480690 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E26: limitation: 23412617 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E26: limitation: 29849369 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E26: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E26: limitation: 21547208 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E26: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E26: limitation: points 1 to 9 come from S1; the extra points from S17
- jvm/E26: limitation: 8 columns beyond the 64 the series allows were dropped
- jvm/E26: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E26: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E26: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E26: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E27: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E27: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E27: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E27: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E27: limitation: 47165612 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E27: limitation: 21751582 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E27: limitation: 33148084 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E27: limitation: 45501391 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E27: limitation: 24778564 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E27: limitation: 39716848 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E27: limitation: 31820397 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E27: limitation: 22557142 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E27: limitation: 39486001 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E27: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E27: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E27: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E27: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E27: limitation: 29130186 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E27: limitation: 23682162 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E27: limitation: 31708106 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E27: limitation: 30229389 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E27: limitation: 20857488 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E27: limitation: 21156641 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E27: limitation: 22863177 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E27: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E27: limitation: 37554366 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E27: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E27: limitation: points 1 to 9 come from S1; the extra points from S19
- jvm/E27: limitation: 9 columns beyond the 64 the series allows were dropped
- jvm/E27: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E27: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E27: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E27: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E28: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E28: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E28: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E28: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E28: limitation: 47165612 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E28: limitation: 21751582 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E28: limitation: 33148084 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E28: limitation: 45501391 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E28: limitation: 24778564 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E28: limitation: 39716848 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E28: limitation: 31820397 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E28: limitation: 22557142 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E28: limitation: 39486001 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E28: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E28: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E28: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E28: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E28: limitation: 30554893 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E28: limitation: 20991735 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E28: limitation: 19914759 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E28: limitation: 33087527 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E28: limitation: 33701073 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E28: limitation: 21165778 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E28: limitation: 30173507 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E28: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E28: limitation: 30032383 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E28: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E28: limitation: points 1 to 9 come from S1; the extra points from S21
- jvm/E28: limitation: 8 columns beyond the 64 the series allows were dropped
- jvm/E28: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E28: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E28: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E28: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E29: limitation: tested in a scratch copy; the project was not changed
- jvm/E29: limitation: gate thresholds are provisional (not calibrated); the 0.2 and 0.5 slope fractions are placeholders
- jvm/E29: limitation: exercised is null: the harness records entry points, not lines, so whether the edited line ran is not checked
- jvm/E29: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E29: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
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
| E4 | growth-series | dump | dla 0.1.0 | artifact series/S2.json (e41a11e843f0) | trend/T14.json |
| E5 | unit-count-series | dump | dla 0.1.0 | artifact series/S2.json (e41a11e843f0) | trend/T14.json |
| E6 | growth-series | test | dla 0.1.0 | artifact series/S3.json (a1c677bf9b17) | trend/T15.json |
| E7 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (a1c677bf9b17) | trend/T15.json |
| E8 | growth-series | test | dla 0.1.0 | artifact series/S5.json (517332765849) | trend/T16.json |
| E9 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (517332765849) | trend/T16.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S7.json (b864d10ec5f0) | trend/T17.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S7.json (b864d10ec5f0) | trend/T17.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S9.json (559db574b475) | trend/T18.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S9.json (559db574b475) | trend/T18.json |
| E14 | growth-series | test | dla 0.1.0 | artifact series/S11.json (51be36cbeb1c) | trend/T19.json |
| E15 | unit-count-series | test | dla 0.1.0 | artifact series/S11.json (51be36cbeb1c) | trend/T19.json |
| E16 | growth-series | test | dla 0.1.0 | artifact series/S13.json (9a771243de43) | trend/T20.json |
| E17 | unit-count-series | test | dla 0.1.0 | artifact series/S13.json (9a771243de43) | trend/T20.json |
| E18 | growth-series | test | dla 0.1.0 | artifact series/S15.json (d7c8d0de5bd3) | trend/T21.json |
| E19 | unit-count-series | test | dla 0.1.0 | artifact series/S15.json (d7c8d0de5bd3) | trend/T21.json |
| E20 | growth-series | test | dla 0.1.0 | artifact series/S17.json (160dc0d3d868) | trend/T22.json |
| E21 | unit-count-series | test | dla 0.1.0 | artifact series/S17.json (160dc0d3d868) | trend/T22.json |
| E22 | growth-series | test | dla 0.1.0 | artifact series/S19.json (3454dd918691) | trend/T23.json |
| E23 | unit-count-series | test | dla 0.1.0 | artifact series/S19.json (3454dd918691) | trend/T23.json |
| E24 | growth-series | test | dla 0.1.0 | artifact series/S21.json (6f71379a0178) | trend/T24.json |
| E25 | unit-count-series | test | dla 0.1.0 | artifact series/S21.json (6f71379a0178) | trend/T24.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S1.json (e4ec29bf1933) | gap/residual-S1-E26.json |
| jvm/E1 | static-hit | static | spotbugs unknown | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-spotbugs.json |
| jvm/E2 | static-hit | static | detekt 2.0.0-alpha.6 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-detekt.json |
| jvm/E3 | static-hit | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-dla.json |
| jvm/E4 | census-holder | static | dla-census 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | census/jvm.json |
| jvm/E5 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | workload/jvm.json |
| jvm/E6 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | harness/H1/harness.json |
| jvm/E7 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (fe15bb0d21b4) | harness/H1/run-S1.json |
| jvm/E8 | growth-series | test | dla 0.1.0 | artifact series/S1.json (e4ec29bf1933) | trend/T1.json |
| jvm/E9 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (e4ec29bf1933) | trend/T1.json |
| jvm/E10 | alloc-stack | test | dla-correlate 0.1.0 | artifact series/S1.json (e4ec29bf1933) | correlate/C1.json |
| jvm/E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (e4ec29bf1933) | trend/T2.json |
| jvm/E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (e4ec29bf1933) | trend/T2.json |
| jvm/E13 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-160751-7b73/harness/H1/out/p1-native/capture/9/heap-1.hprof (a9be90338318) | summaries/heap-1-a9be9033.json |
| jvm/E14 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-160751-7b73/harness/H1/out/p4-native/capture/392/heap-1.hprof (edbc564f4b48) | summaries/heap-1-edbc564f.json |
| jvm/E15 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-160751-7b73/harness/H1/out/p6-native/capture/624/heap-1.hprof (cad5b287df92) | summaries/heap-1-cad5b287.json |
| jvm/E16 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-160751-7b73/harness/H1/out/p9-native/capture/964/heap-1.hprof (1fd0562ef7fe) | summaries/heap-1-1fd0562e.json |
| jvm/E17 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S2.json (e41a11e843f0) | series/S2.json |
| jvm/E18 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S2.json (e41a11e843f0) | series/S2.json |
| jvm/E19 | discriminator | test | dla 0.1.0 | artifact series/S4.json (bbc1759d8105) | alloc/A1.json |
| jvm/E20 | discriminator | test | dla 0.1.0 | artifact series/S6.json (d06445aeb4a6) | alloc/A2.json |
| jvm/E21 | discriminator | test | dla 0.1.0 | artifact series/S8.json (60b0deada220) | alloc/A3.json |
| jvm/E22 | discriminator | test | dla 0.1.0 | artifact series/S10.json (2bac354a54f9) | alloc/A4.json |
| jvm/E23 | discriminator | test | dla 0.1.0 | artifact series/S12.json (2cb212b541d0) | alloc/A5.json |
| jvm/E24 | discriminator | test | dla 0.1.0 | artifact series/S14.json (869558358283) | alloc/A6.json |
| jvm/E25 | discriminator | test | dla 0.1.0 | artifact series/S16.json (312f7cf310f2) | alloc/A7.json |
| jvm/E26 | discriminator | test | dla 0.1.0 | artifact series/S18.json (2ec43b0f73cd) | alloc/A8.json |
| jvm/E27 | discriminator | test | dla 0.1.0 | artifact series/S20.json (968659e8f599) | alloc/A9.json |
| jvm/E28 | discriminator | test | dla 0.1.0 | artifact series/S22.json (1103f7b0759b) | alloc/A10.json |
| jvm/E29 | intervention | test | dla 0.1.0 | artifact series/S23.json (7ffc604c046d) | intervene/I1 |
| rust/E1 | static-hit | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/rust-dla.json |
| rust/E2 | census-holder | static | dla-census 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | census/rust.json |
| rust/E3 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | workload/rust.json |
