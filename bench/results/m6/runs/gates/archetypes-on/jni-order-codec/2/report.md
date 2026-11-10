# DLA report

Run: `20261110-164514-32c4`

Status: INCOMPLETE (115 open)

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 23 enumerated, 23 dispositioned; confirmed 1, strong 1, refuted 5, suspected 16, unresolved 0

Residual: heap 1459.2658258258257 bytes/n unattributed of 1459.2658258258257 total (threshold 364.8164564564564) ABOVE THRESHOLD, see O1; heap 1442.091291291291 bytes/n unattributed of 1442.091291291291 total (threshold 360.5228228228228) ABOVE THRESHOLD, see O1; heap 1441.362962962963 bytes/n unattributed of 1441.362962962963 total (threshold 360.3407407407407) ABOVE THRESHOLD, see O1; heap 1441.8856856856855 bytes/n unattributed of 1441.8856856856855 total (threshold 360.4714214214214) ABOVE THRESHOLD, see O1; heap 1441.4391391391391 bytes/n unattributed of 1441.4391391391391 total (threshold 360.3597847847848) ABOVE THRESHOLD, see O1; heap 1443.2903153153152 bytes/n unattributed of 1443.2903153153152 total (threshold 360.8225788288288) ABOVE THRESHOLD, see O1; non-heap 22.68720720720721 bytes/n unattributed of 22.68720720720721 total (threshold 35.932499675676); non-heap 0 bytes/n unattributed of -1.7061261261261262 total (threshold 8.307849275675625); non-heap 0 bytes/n unattributed of -3.955675675675676 total (threshold 9.144997362162188); non-heap 0 bytes/n unattributed of -3.598698698698698 total (threshold 12.46704834834824); non-heap 0 bytes/n unattributed of -2.789109109109109 total (threshold 19.234798270270268); non-heap 0.8918918918918919 bytes/n unattributed of 0.8918918918918919 total (threshold 4.350295675675675)

Coverage: jvm 17760 iterations, 0 of 17 entry points exercised; rust no iterations, 0 of 10 entry points exercised

## Open items

- **O1**: open item O1: unexplained growth: 1459 bytes/n in heap (S1); add a hypothesis (dla ledger add --origin residual --evidence E14 ...) or record it unresolved. Fix: `dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"`
- **S10**: series S10 has no trend analysis. Fix: `dla trend S10`
- **S12**: series S12 has no trend analysis. Fix: `dla trend S12`
- **S14**: series S14 has no trend analysis. Fix: `dla trend S14`
- **S15**: series S15 has no trend analysis. Fix: `dla trend S15`
- **T2:os.vm_hwm**: column os.vm_hwm of T2 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T2:jvm.heap_committed**: column jvm.heap_committed of T2 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T2:jvm.heap_used**: column jvm.heap_used of T2 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T2:native.live_blocks**: column native.live_blocks of T2 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T2:native.live_bytes**: column native.live_bytes of T2 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T2:native.peak_bytes**: column native.peak_bytes of T2 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T2:jvm.heap_committed.pre_gc**: column jvm.heap_committed.pre_gc of T2 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T2:jvm.heap_used.pre_gc**: column jvm.heap_used.pre_gc of T2 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:os.rss**: column os.rss of T3 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:os.rss_anon**: column os.rss_anon of T3 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:os.vm_hwm**: column os.vm_hwm of T3 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:jvm.heap_used**: column jvm.heap_used of T3 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:native.live_blocks**: column native.live_blocks of T3 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:native.live_bytes**: column native.live_bytes of T3 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:native.peak_bytes**: column native.peak_bytes of T3 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:jvm.heap_committed.pre_gc**: column jvm.heap_committed.pre_gc of T3 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T3:jvm.class:byte[].count**: column jvm.class:byte[].count of T3 (steady) is not flat and is tied to no finding. Fix: `dla trend S3`
- **T3:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T3 (steady) is not flat and is tied to no finding. Fix: `dla trend S3`
- **T3:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T3 (steady) is not flat and is tied to no finding. Fix: `dla trend S3`
- **T4:os.rss**: column os.rss of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:os.rss_anon**: column os.rss_anon of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:jvm.heap_used**: column jvm.heap_used of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:native.live_blocks**: column native.live_blocks of T4 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:native.live_bytes**: column native.live_bytes of T4 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:native.peak_bytes**: column native.peak_bytes of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:jvm.class:byte[].count**: column jvm.class:byte[].count of T4 (steady) is not flat and is tied to no finding. Fix: `dla trend S5`
- **T4:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T4 (steady) is not flat and is tied to no finding. Fix: `dla trend S5`
- **T4:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T4 (steady) is not flat and is tied to no finding. Fix: `dla trend S5`
- **T5:os.rss**: column os.rss of T5 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.rss_anon**: column os.rss_anon of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:jvm.heap_committed**: column jvm.heap_committed of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:jvm.heap_used**: column jvm.heap_used of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:native.live_blocks**: column native.live_blocks of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:native.live_bytes**: column native.live_bytes of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:native.peak_bytes**: column native.peak_bytes of T5 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:jvm.heap_used.pre_gc**: column jvm.heap_used.pre_gc of T5 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S9 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:jvm.class:byte[].count**: column jvm.class:byte[].count of T5 (steady) is not flat and is tied to no finding. Fix: `dla trend S9`
- **T5:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T5 (steady) is not flat and is tied to no finding. Fix: `dla trend S9`
- **T5:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T5 (steady) is not flat and is tied to no finding. Fix: `dla trend S9`
- **T6:os.rss**: column os.rss of T6 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S11 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:os.rss_anon**: column os.rss_anon of T6 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S11 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:jvm.heap_committed**: column jvm.heap_committed of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S11 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:jvm.heap_used**: column jvm.heap_used of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S11 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:native.live_blocks**: column native.live_blocks of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S11 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:native.live_bytes**: column native.live_bytes of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S11 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:native.peak_bytes**: column native.peak_bytes of T6 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S11 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:jvm.class:byte[].count**: column jvm.class:byte[].count of T6 (steady) is not flat and is tied to no finding. Fix: `dla trend S11`
- **T6:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T6 (steady) is not flat and is tied to no finding. Fix: `dla trend S11`
- **T6:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T6 (steady) is not flat and is tied to no finding. Fix: `dla trend S11`
- **T7:os.rss**: column os.rss of T7 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S13 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:os.rss_anon**: column os.rss_anon of T7 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S13 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:os.vm_data**: column os.vm_data of T7 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S13 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:os.vm_hwm**: column os.vm_hwm of T7 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S13 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:jvm.heap_used**: column jvm.heap_used of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S13 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:native.live_blocks**: column native.live_blocks of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S13 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:native.live_bytes**: column native.live_bytes of T7 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S13 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:native.peak_bytes**: column native.peak_bytes of T7 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S13 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:jvm.class:byte[].count**: column jvm.class:byte[].count of T7 (steady) is not flat and is tied to no finding. Fix: `dla trend S13`
- **T7:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla trend S13`
- **T7:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla trend S13`
- **T8:os.rss**: column os.rss of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:os.rss_anon**: column os.rss_anon of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:os.vm_hwm**: column os.vm_hwm of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:jvm.heap_used**: column jvm.heap_used of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:native.live_blocks**: column native.live_blocks of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:native.live_bytes**: column native.live_bytes of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:native.peak_bytes**: column native.peak_bytes of T8 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T8:jvm.class:byte[].count**: column jvm.class:byte[].count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E5 # column jvm.class:byte[].count`
- **T8:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E5 # column jvm.class:java.lang.String.count`
- **T8:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E5 # column jvm.class:shop.codec.Entry.count`
- **T9:os.rss**: column os.rss of T9 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:os.rss_anon**: column os.rss_anon of T9 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:os.vm_data**: column os.vm_data of T9 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:os.vm_hwm**: column os.vm_hwm of T9 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:jvm.heap_committed**: column jvm.heap_committed of T9 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:jvm.heap_used**: column jvm.heap_used of T9 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:native.live_blocks**: column native.live_blocks of T9 (plateau) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:native.live_bytes**: column native.live_bytes of T9 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:native.peak_bytes**: column native.peak_bytes of T9 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:jvm.heap_committed.pre_gc**: column jvm.heap_committed.pre_gc of T9 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:jvm.class:byte[].count**: column jvm.class:byte[].count of T9 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E7 # column jvm.class:byte[].count`
- **T9:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T9 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E7 # column jvm.class:java.lang.String.count`
- **T9:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T9 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E7 # column jvm.class:shop.codec.Entry.count`
- **T11:type:byte[]**: column type:byte[] of T11 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E10 # column type:byte[]`
- **T11:type:byte[].count**: column type:byte[].count of T11 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E11 # column type:byte[].count`
- **T11:type:java.lang.String**: column type:java.lang.String of T11 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E10 # column type:java.lang.String`
- **T11:type:java.lang.String.count**: column type:java.lang.String.count of T11 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E11 # column type:java.lang.String.count`
- **T11:type:jdk.internal.vm.FillerElement[]**: column type:jdk.internal.vm.FillerElement[] of T11 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E10 # column type:jdk.internal.vm.FillerElement[]`
- **T11:type:jdk.internal.vm.FillerElement[].count**: column type:jdk.internal.vm.FillerElement[].count of T11 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E11 # column type:jdk.internal.vm.FillerElement[].count`
- **T11:type:java.lang.Object[]**: column type:java.lang.Object[] of T11 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E10 # column type:java.lang.Object[]`
- **T11:type:jdk.jfr.internal.periodic.Batch**: column type:jdk.jfr.internal.periodic.Batch of T11 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E10 # column type:jdk.jfr.internal.periodic.Batch`
- **T11:type:jdk.jfr.internal.periodic.Batch.count**: column type:jdk.jfr.internal.periodic.Batch.count of T11 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E11 # column type:jdk.jfr.internal.periodic.Batch.count`
- **T11:type:java.lang.Long**: column type:java.lang.Long of T11 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E10 # column type:java.lang.Long`
- **T11:type:java.lang.Long.count**: column type:java.lang.Long.count of T11 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E11 # column type:java.lang.Long.count`
- **T12:os.rss**: column os.rss of T12 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T12:os.rss_anon**: column os.rss_anon of T12 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T12:jvm.heap_used**: column jvm.heap_used of T12 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T12:native.live_blocks**: column native.live_blocks of T12 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T12:native.live_bytes**: column native.live_bytes of T12 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T12:native.peak_bytes**: column native.peak_bytes of T12 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T12:jvm.heap_used.pre_gc**: column jvm.heap_used.pre_gc of T12 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T12:jvm.class:byte[].count**: column jvm.class:byte[].count of T12 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E13 # column jvm.class:byte[].count`
- **T12:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T12 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E13 # column jvm.class:java.lang.String.count`
- **T12:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T12 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E13 # column jvm.class:shop.codec.Entry.count`
- **heap**: residual growth in heap (1459.266 bytes/n, threshold 364.816) has no resolved open item. Fix: `dla gap --residual then resolve the open item it raises`
- **heap**: residual growth in heap (1442.091 bytes/n, threshold 360.523) has no resolved open item. Fix: `dla gap --residual then resolve the open item it raises`
- **heap**: residual growth in heap (1441.363 bytes/n, threshold 360.341) has no resolved open item. Fix: `dla gap --residual then resolve the open item it raises`
- **heap**: residual growth in heap (1441.886 bytes/n, threshold 360.471) has no resolved open item. Fix: `dla gap --residual then resolve the open item it raises`
- **heap**: residual growth in heap (1441.439 bytes/n, threshold 360.360) has no resolved open item. Fix: `dla gap --residual then resolve the open item it raises`
- **heap**: residual growth in heap (1443.290 bytes/n, threshold 360.823) has no resolved open item. Fix: `dla gap --residual then resolve the open item it raises`

## Workers

- **jvm** (jvm): status done; 86 lines merged; slice done; torn tail: no
- **rust** (rust): status done; 12 lines merged; slice done; torn tail: no

## Findings

### jvm/F3: **L-COLLECT**

- State: strong
- Label: L-COLLECT (evidence: jvm/E8 growth-series, jvm/E15 retention-path)
- Growth: `probe:shop.codec.Audit.trail` 2.625 count/n (steady, gate pass, jvm/E12)
- Evidence classes: context, discriminator, growth, retention, static
- Strong: second class jvm/E15 retention-path that names the holder, independent of the growth record jvm/E12 (dla)
- Holder: static/global
- Owner: `shop.codec.Audit.trail`
- Region: heap
- Shape: steady
- Site: `kotlin/shop/Audit.kt:9`
- Writers: `kotlin/shop/Audit.kt:12`
- Annotation: heuristic name match over the project's sources
- Annotation: trail.add per inline frame; no remove/clear anywhere
- Verified: no
- Sibling hypotheses: jvm/F3.n1
- Supporting evidence: jvm/E4, jvm/E7, jvm/E12, jvm/E13, jvm/E15
- Explanation (observed): Audit.trail (static holder in Audit.kt:9) gets one Entry with a 1536-byte sample copy per inline frame from Audit.record at Audit.kt:12, called by OrderEncoder.encode; nothing removes or clears entries, so they are never released.
- Explanation structure: adds: kotlin/shop/Audit.kt:12; removal: never; evidence: jvm/E15

### jvm/F19: **L-HANDLE**

- State: confirmed
- Label: L-HANDLE (evidence: jvm/E27 growth-series, jvm/E26 alloc-stack)
- Growth: `native.live_blocks@native/src/lib.rs:40` 0.125 count/n (steady, gate pass, jvm/E12)
- Evidence classes: allocation, context, discriminator, growth
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.open`
- Region: off-heap
- Shape: steady
- Site: `native/src/lib.rs:40`
- Writers: `kotlin/shop/Native.kt:9`, `kotlin/shop/Encoder.kt:11`, `kotlin/shop/Encoder.kt:9`
- Removers: `kotlin/shop/Encoder.kt:20`, `kotlin/shop/Encoder.kt:18`
- Annotation: native allocation under shop.codec.Native.open (Java_shop_codec_Native_open); found by dla correlate
- Annotation: Native.open handle is not closed on the bulk-frame early return
- Verified: no
- Sibling hypotheses: jvm/F19.n1
- Supporting evidence: jvm/E10, jvm/E7, jvm/E12, jvm/E14
- Explanation (observed): Native.open at Encoder.kt:9 (native handle, lib.rs:40) allocates a native frame copy per order; OrderEncoder.encode calls Native.close at Encoder.kt:18 only when the frame is inline, and the bulk-frame early return at line 15 skips it, so those handles leak.
- Explanation structure: adds: kotlin/shop/Encoder.kt:9; removal: conditional at kotlin/shop/Encoder.kt:18; evidence: jvm/E27

### jvm/F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Site: `kotlin/shop/Audit.kt:9`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: same accumulation as confirmed jvm/F3 (Audit.trail of Entry); tracked there
- Supporting evidence: jvm/E3

### jvm/F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.Entry.sample`
- Region: heap
- Site: `kotlin/shop/Audit.kt:3`
- Reason: not_exercised
- Remedy: same accumulation as confirmed jvm/F3 (Audit.trail of Entry); tracked there
- Supporting evidence: jvm/E4

### jvm/F4: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.OrderEncoder.Companion`
- Region: heap
- Site: `kotlin/shop/Encoder.kt:6`
- Reason: not_exercised
- Remedy: constant/companion holder with no writer; no probe column grew; confirm only if a growth column ties to it
- Supporting evidence: jvm/E4

### jvm/F5: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.FramePacker.MAGIC`
- Region: heap
- Site: `kotlin/shop/FramePacker.kt:9`
- Reason: not_exercised
- Remedy: constant/companion holder with no writer; no probe column grew; confirm only if a growth column ties to it
- Supporting evidence: jvm/E4

### jvm/F6: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.Native.INSTANCE`
- Region: heap
- Site: `kotlin/shop/Native.kt:5`
- Reason: not_exercised
- Remedy: constant/companion holder with no writer; no probe column grew; confirm only if a growth column ties to it
- Supporting evidence: jvm/E4

### jvm/F7: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.<clinit>`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:6`
- Reason: not_exercised
- Remedy: JNI method declaration, not a holder; native growth is carried by jvm/F19 (native/src/lib.rs:40)
- Supporting evidence: jvm/E4

### jvm/F8: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.open`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:9`
- Reason: not_exercised
- Remedy: JNI method declaration, not a holder; native growth is carried by jvm/F19 (native/src/lib.rs:40)
- Supporting evidence: jvm/E4

### jvm/F9: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.checksum`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:10`
- Reason: not_exercised
- Remedy: JNI method declaration, not a holder; native growth is carried by jvm/F19 (native/src/lib.rs:40)
- Supporting evidence: jvm/E4

### jvm/F10: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.size`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:11`
- Reason: not_exercised
- Remedy: JNI method declaration, not a holder; native growth is carried by jvm/F19 (native/src/lib.rs:40)
- Supporting evidence: jvm/E4

### jvm/F11: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.lines`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:12`
- Reason: not_exercised
- Remedy: JNI method declaration, not a holder; native growth is carried by jvm/F19 (native/src/lib.rs:40)
- Supporting evidence: jvm/E4

### jvm/F12: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.close`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:13`
- Reason: not_exercised
- Remedy: JNI method declaration, not a holder; native growth is carried by jvm/F19 (native/src/lib.rs:40)
- Supporting evidence: jvm/E4

### jvm/F13: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.live`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:14`
- Reason: not_exercised
- Remedy: JNI method declaration, not a holder; native growth is carried by jvm/F19 (native/src/lib.rs:40)
- Supporting evidence: jvm/E4

### jvm/F14: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.liveBytes`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:15`
- Reason: not_exercised
- Remedy: JNI method declaration, not a holder; native growth is carried by jvm/F19 (native/src/lib.rs:40)
- Supporting evidence: jvm/E4

### jvm/F18: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: static/global
- Owner: `shop.codec.Stats.Companion`
- Region: heap
- Site: `kotlin/shop/Stats.kt:6`
- Reason: not_exercised
- Remedy: constant/companion holder with no writer; no probe column grew; confirm only if a growth column ties to it
- Supporting evidence: jvm/E4

### rust/F1: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `native.src.frame.Frame.bytes`
- Region: heap
- Site: `native/src/frame.rs:11`
- Annotation: Frame.bytes is owned by the FRAMES entry; retained only as long as the map entry (see F2).
- Reason: no_harness
- Remedy: Rust crate is a cdylib driven only through JNI: no bin/iteration workload and no lib target for a driver. Measure from the JVM worker with the shim and dla correlate H1; Encoder.kt bulk path never calls Native.close.
- Supporting evidence: rust/E3

### rust/F2: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: static/global
- Owner: `native.src.lib.FRAMES`
- Region: heap
- Site: `native/src/lib.rs:19`
- Writers: `native/src/lib.rs:45`
- Removers: `native/src/lib.rs:74`
- Annotation: FRAMES keeps one Frame per handle from Native.open until Native.close; the JVM Encoder.kt bulk path (size > 16384) returns without calling close (Encoder.kt:12-15), so those frames stay in FRAMES forever. Needs JVM-side harness + correlate to confirm.
- Reason: no_harness
- Remedy: Rust crate is a cdylib driven only through JNI: no bin/iteration workload and no lib target for a driver. Measure from the JVM worker with the shim and dla correlate H1; Encoder.kt bulk path never calls Native.close.
- Supporting evidence: rust/E3

## Ruled out

- **jvm/F3.n1** (UNCLASSIFIED): discriminator jvm/E13
  - Ruled out by: plateau+purge (leak) jvm/E13
  - Hypothesis: N-WARM: growth plateaus after warm-up
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: jvm/E13
  - Originally supported by: jvm/E12
- **jvm/F15** (UNCLASSIFIED): discriminator jvm/E24
  - Ruled out by: plateau+purge (non-leak) jvm/E24
  - Contradicting evidence: jvm/E24
  - Originally supported by: jvm/E4
- **jvm/F16** (UNCLASSIFIED): discriminator jvm/E23
  - Ruled out by: plateau+purge (non-leak) jvm/E23
  - Contradicting evidence: jvm/E23
  - Originally supported by: jvm/E4
- **jvm/F17** (UNCLASSIFIED): discriminator jvm/E25
  - Ruled out by: plateau+purge (non-leak) jvm/E25
  - Contradicting evidence: jvm/E25
  - Originally supported by: jvm/E4
- **jvm/F19.n1** (UNCLASSIFIED): discriminator jvm/E14
  - Ruled out by: plateau+purge (leak) jvm/E14
  - Hypothesis: N-WARM: growth plateaus after warm-up
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: jvm/E14
  - Originally supported by: jvm/E12

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

- E4: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E4: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E4: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E4: limitation: 21941505 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E4: limitation: 25406180 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E4: limitation: 23934570 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E4: limitation: 30094688 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E4: limitation: 30653167 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E4: limitation: 21157738 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E4: limitation: 28907713 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E4: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E4: limitation: 25130307 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E4: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E4: limitation: gate thresholds are provisional (not calibrated)
- E5: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E5: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E5: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E5: limitation: 21941505 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E5: limitation: 25406180 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E5: limitation: 23934570 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E5: limitation: 30094688 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E5: limitation: 30653167 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E5: limitation: 21157738 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E5: limitation: 28907713 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E5: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E5: limitation: 25130307 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E5: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E5: limitation: gate thresholds are provisional (not calibrated)
- E6: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E6: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E6: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E6: limitation: 29072110 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: 21075614 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: 30992471 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: 29405477 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: 28461152 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: 34355170 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: 28806788 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E6: limitation: 32085434 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E6: limitation: gate thresholds are provisional (not calibrated)
- E7: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E7: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E7: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E7: limitation: 29072110 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: 21075614 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: 30992471 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: 29405477 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: 28461152 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: 34355170 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: 28806788 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E7: limitation: 32085434 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E7: limitation: gate thresholds are provisional (not calibrated)
- E8: limitation: series: per-point; x is the iteration count n of each point (2220, 8880, 13320, 19980); every point is a separate process and heap dump
- E8: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E8: limitation: holders absent from a dump's 200 largest have no figure at that point
- E8: limitation: gate thresholds are provisional (not calibrated)
- E9: limitation: series: per-point; x is the iteration count n of each point (2220, 8880, 13320, 19980); every point is a separate process and heap dump
- E9: limitation: retained bytes are approximate (see the dump summaries); t_ms is not recorded
- E9: limitation: holders absent from a dump's 200 largest have no figure at that point
- E9: limitation: gate thresholds are provisional (not calibrated)
- E10: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E10: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E11: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E12: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E12: limitation: 31374632 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: 31882564 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: 29802845 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: 31009630 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: 30849045 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: 28542286 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: 30692170 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E12: limitation: 29757533 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: dropped 6 per-type columns (cap of 64, lowest growth first)
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E13: limitation: 31374632 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: 31882564 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: 29802845 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: 31009630 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: 30849045 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: 28542286 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: 30692170 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E13: limitation: 29757533 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: dropped 6 per-type columns (cap of 64, lowest growth first)
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: partial coverage (2 of 3 examined)
- E14: limitation: jvm/F3: growth measured as a count; bytes not attributed
- E14: limitation: jvm/F19: growth measured as a count; bytes not attributed
- E14: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E14: limitation: threshold is provisional (not calibrated)
- E15: partial coverage (2 of 3 examined)
- E15: limitation: jvm/F3: growth evidence is from S1, not S3; not subtracted
- E15: limitation: jvm/F19: growth evidence is from S1, not S3; not subtracted
- E15: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E15: limitation: threshold is provisional (not calibrated)
- E16: partial coverage (2 of 3 examined)
- E16: limitation: jvm/F3: growth evidence is from S1, not S5; not subtracted
- E16: limitation: jvm/F19: growth evidence is from S1, not S5; not subtracted
- E16: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E16: limitation: threshold is provisional (not calibrated)
- E17: partial coverage (2 of 3 examined)
- E17: limitation: jvm/F3: growth evidence is from S1, not S9; not subtracted
- E17: limitation: jvm/F19: growth evidence is from S1, not S9; not subtracted
- E17: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E17: limitation: threshold is provisional (not calibrated)
- E18: partial coverage (2 of 3 examined)
- E18: limitation: jvm/F3: growth evidence is from S1, not S11; not subtracted
- E18: limitation: jvm/F19: growth evidence is from S1, not S11; not subtracted
- E18: limitation: off-heap: no gap series for S11; run dla gap S11 first (node.external is used when present)
- E18: limitation: threshold is provisional (not calibrated)
- E19: partial coverage (2 of 3 examined)
- E19: limitation: jvm/F3: growth evidence is from S1, not S13; not subtracted
- E19: limitation: jvm/F19: growth evidence is from S1, not S13; not subtracted
- E19: limitation: off-heap: no gap series for S13; run dla gap S13 first (node.external is used when present)
- E19: limitation: threshold is provisional (not calibrated)
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
- jvm/E7: limitation: 43845090 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E7: limitation: 22310547 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E8: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E8: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E8: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E8: limitation: 43845090 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 22310547 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 31706220 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 46024992 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 21670916 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 49715763 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 30568617 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 20603120 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 43903528 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E8: limitation: gate thresholds are provisional (not calibrated)
- jvm/E9: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E9: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E9: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E9: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E9: limitation: 43845090 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 22310547 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 31706220 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 46024992 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 21670916 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 49715763 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 30568617 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 20603120 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 43903528 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E9: limitation: gate thresholds are provisional (not calibrated)
- jvm/E10: partial coverage (1 of 1 examined)
- jvm/E10: limitation: the stack is the project frame of the largest allocation stack at the site, not every frame
- jvm/E10: limitation: slopes are per unit of the series axis; the thresholds are provisional
- jvm/E11: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E11: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E11: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E11: limitation: 43845090 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 22310547 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 31706220 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 46024992 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 21670916 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 49715763 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 30568617 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 20603120 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 43903528 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E11: limitation: gate thresholds are provisional (not calibrated)
- jvm/E12: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E12: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E12: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E12: limitation: 43845090 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 22310547 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 31706220 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 46024992 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 21670916 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 49715763 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 30568617 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 20603120 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 43903528 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E12: limitation: gate thresholds are provisional (not calibrated)
- jvm/E13: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E13: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E13: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E13: limitation: 43845090 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 22310547 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 31706220 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 46024992 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 21670916 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 49715763 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 30568617 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 20603120 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 43903528 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E13: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E13: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E13: limitation: 21941505 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 25406180 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 23934570 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 30094688 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 30653167 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 21157738 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 28907713 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E13: limitation: 25130307 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E13: limitation: points 1 to 9 come from S1; the extra points from S2
- jvm/E13: limitation: 8 columns beyond the 64 the series allows were dropped
- jvm/E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E13: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E13: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E14: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E14: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E14: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E14: limitation: 43845090 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 22310547 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 31706220 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 46024992 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 21670916 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 49715763 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 30568617 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 20603120 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 43903528 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E14: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E14: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E14: limitation: 29072110 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 21075614 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 30992471 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 29405477 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 28461152 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 34355170 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 28806788 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E14: limitation: 32085434 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E14: limitation: points 1 to 9 come from S1; the extra points from S4
- jvm/E14: limitation: 9 columns beyond the 64 the series allows were dropped
- jvm/E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E14: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E14: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E14: limitation: gate and plateau thresholds are provisional (not calibrated)
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
- jvm/E23: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E23: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E23: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E23: limitation: 43845090 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: 22310547 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: 31706220 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: 46024992 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: 21670916 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: 49715763 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: 30568617 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: 20603120 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: 43903528 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E23: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E23: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E23: limitation: 31374632 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: 31882564 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: 29802845 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: 31009630 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: 30849045 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: 28542286 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: 30692170 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E23: limitation: 29757533 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E23: limitation: dropped 6 per-type columns (cap of 64, lowest growth first)
- jvm/E23: limitation: points 1 to 9 come from S1; the extra points from S8
- jvm/E23: limitation: 11 columns beyond the 64 the series allows were dropped
- jvm/E23: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E23: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E23: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E23: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E24: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E24: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E24: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E24: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E24: limitation: 43845090 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: 22310547 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: 31706220 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: 46024992 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: 21670916 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: 49715763 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: 30568617 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: 20603120 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: 43903528 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E24: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E24: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E24: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E24: limitation: 30502544 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: 30044397 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: 33099081 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: 33079137 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: 32242469 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: 29060654 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: 31875228 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E24: limitation: 33251117 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E24: limitation: dropped 13 per-type columns (cap of 64, lowest growth first)
- jvm/E24: limitation: points 1 to 9 come from S1; the extra points from S10
- jvm/E24: limitation: 4 columns beyond the 64 the series allows were dropped
- jvm/E24: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E24: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E24: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E24: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E25: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E25: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E25: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E25: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E25: limitation: 43845090 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: 22310547 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: 31706220 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: 46024992 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: 21670916 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: 49715763 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: 30568617 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: 20603120 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: 43903528 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E25: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E25: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E25: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E25: limitation: 21443363 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: 33579119 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: 30629050 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: 29182224 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: 31235785 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: 21160388 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: 30699497 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E25: limitation: 32754269 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E25: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E25: limitation: points 1 to 9 come from S1; the extra points from S12
- jvm/E25: limitation: 8 columns beyond the 64 the series allows were dropped
- jvm/E25: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E25: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E25: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E25: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E26: limitation: stacks are the shim's live allocations at exit, keyed by their first eight resolved frames; the shim's own hash changes between processes
- jvm/E26: limitation: each capture is the exit record of a separate process: the figures grow with the work done before exit
- jvm/E26: limitation: up to 49715763 live bytes sit in stacks with no frame in the project: they have no site and tie to nothing
- jvm/E27: limitation: stacks are keyed by their first eight resolved frames; each point is the exit record of a separate process
- jvm/E27: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- jvm/E28: limitation: OldObjectSample samples old objects that are still live; it is not a census, so counts and bytes are estimates and a count series over recordings is weak
- jvm/E28: limitation: holders need path-to-gc-roots=true and the recording started before the leak
- jvm/E28: limitation: the recordings are ordered as given
- jvm/E28: limitation: the profile is sampled: counts and bytes are estimates
- jvm/E29: limitation: OldObjectSample samples old objects that are still live; it is not a census, so counts and bytes are estimates and a count series over recordings is weak
- jvm/E29: limitation: holders need path-to-gc-roots=true and the recording started before the leak
- jvm/E29: limitation: the recordings are ordered as given
- jvm/E29: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
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
| E4 | growth-series | test | dla 0.1.0 | artifact series/S2.json (936164fadce7) | trend/T8.json |
| E5 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (936164fadce7) | trend/T8.json |
| E6 | growth-series | test | dla 0.1.0 | artifact series/S4.json (b7e409a7dca8) | trend/T9.json |
| E7 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (b7e409a7dca8) | trend/T9.json |
| E8 | growth-series | dump | dla 0.1.0 | artifact series/S6.json (21c7c7a510a4) | trend/T10.json |
| E9 | unit-count-series | dump | dla 0.1.0 | artifact series/S6.json (21c7c7a510a4) | trend/T10.json |
| E10 | growth-series | dump | dla 0.1.0 | artifact series/S7.json (8661e88779c8) | trend/T11.json |
| E11 | unit-count-series | dump | dla 0.1.0 | artifact series/S7.json (8661e88779c8) | trend/T11.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S8.json (2362ec5dc438) | trend/T12.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (2362ec5dc438) | trend/T12.json |
| E14 | residual | test | dla 0.1.0 | artifact series/S1.json (1607971da8bb) | gap/residual-S1-E14.json |
| E15 | residual | test | dla 0.1.0 | artifact series/S3.json (dff43eda510b) | gap/residual-S3-E15.json |
| E16 | residual | test | dla 0.1.0 | artifact series/S5.json (af0a00715263) | gap/residual-S5-E16.json |
| E17 | residual | test | dla 0.1.0 | artifact series/S9.json (d6de7945f383) | gap/residual-S9-E17.json |
| E18 | residual | test | dla 0.1.0 | artifact series/S11.json (4fa8eb3cb40b) | gap/residual-S11-E18.json |
| E19 | residual | test | dla 0.1.0 | artifact series/S13.json (bfe2664f3787) | gap/residual-S13-E19.json |
| jvm/E1 | static-hit | static | spotbugs unknown | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-spotbugs.json |
| jvm/E2 | static-hit | static | detekt 2.0.0-alpha.6 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-detekt.json |
| jvm/E3 | static-hit | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-dla.json |
| jvm/E4 | census-holder | static | dla-census 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | census/jvm.json |
| jvm/E5 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | workload/jvm.json |
| jvm/E6 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | harness/H1/harness.json |
| jvm/E7 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (6429c8e20ead) | harness/H1/run-S1.json |
| jvm/E8 | growth-series | test | dla 0.1.0 | artifact series/S1.json (1607971da8bb) | trend/T1.json |
| jvm/E9 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (1607971da8bb) | trend/T1.json |
| jvm/E10 | alloc-stack | test | dla-correlate 0.1.0 | artifact series/S1.json (1607971da8bb) | correlate/C1.json |
| jvm/E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (1607971da8bb) | trend/T2.json |
| jvm/E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (1607971da8bb) | trend/T2.json |
| jvm/E13 | discriminator | test | dla 0.1.0 | artifact series/S3.json (dff43eda510b) | alloc/A1.json |
| jvm/E14 | discriminator | test | dla 0.1.0 | artifact series/S5.json (af0a00715263) | alloc/A2.json |
| jvm/E15 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-164514-32c4/harness/H1/out/p1-native/capture/9/heap-1.hprof (ecea39f4dfe6) | summaries/heap-1-ecea39f4.json |
| jvm/E16 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-164514-32c4/harness/H1/out/p4-native/capture/392/heap-1.hprof (c4583228d68b) | summaries/heap-1-c4583228.json |
| jvm/E17 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-164514-32c4/harness/H1/out/p6-native/capture/621/heap-1.hprof (e3d7b24611cd) | summaries/heap-1-e3d7b246.json |
| jvm/E18 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-164514-32c4/harness/H1/out/p9-native/capture/960/heap-1.hprof (a0a25d2b024b) | summaries/heap-1-a0a25d2b.json |
| jvm/E19 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S6.json (21c7c7a510a4) | series/S6.json |
| jvm/E20 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S6.json (21c7c7a510a4) | series/S6.json |
| jvm/E21 | histogram-diff | live | jcmd-class-histogram 0.1.0 | artifact histograms/H1.json (b4ca7d9b45f7) | histograms/H1.json |
| jvm/E22 | growth-series | live | jcmd-class-histogram 0.1.0 | artifact series/S7.json (8661e88779c8) | series/S7.json |
| jvm/E23 | discriminator | test | dla 0.1.0 | artifact series/S9.json (d6de7945f383) | alloc/A3.json |
| jvm/E24 | discriminator | test | dla 0.1.0 | artifact series/S11.json (4fa8eb3cb40b) | alloc/A4.json |
| jvm/E25 | discriminator | test | dla 0.1.0 | artifact series/S13.json (bfe2664f3787) | alloc/A5.json |
| jvm/E26 | alloc-stack | live | shim-stack-groups 0.1.0 | artifact histograms/H2.json (3bf5fa95881e) | histograms/H2.json |
| jvm/E27 | growth-series | live | shim-stack-groups 0.1.0 | artifact series/S14.json (03301414031b) | series/S14.json |
| jvm/E28 | alloc-stack | live | jfr-old-object-sample 0.1.0 | artifact allocs/A1.json (4988e637ff41) | allocs/A1.json |
| jvm/E29 | growth-series | live | jfr-old-object-sample 0.1.0 | artifact series/S15.json (f5a21f54e8b9) | series/S15.json |
| rust/E1 | static-hit | static | rapx 0.7.71 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/rust-rapx.json |
| rust/E2 | static-hit | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/rust-dla.json |
| rust/E3 | census-holder | static | dla-census 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | census/rust.json |
| rust/E4 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | workload/rust.json |
