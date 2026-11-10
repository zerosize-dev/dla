# DLA report

Run: `20261110-185929-13a3`

Status: INCOMPLETE (96 open)

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 11 enumerated, 11 dispositioned; confirmed 2, strong 0, refuted 2, suspected 3, unresolved 4

Residual: heap 2681.6564393939393 bytes/n unattributed of 2681.6564393939393 total (threshold 670.4141098484848) ABOVE THRESHOLD, see O1; heap 2680.01375 bytes/n unattributed of 2680.01375 total (threshold 670.0034375) ABOVE THRESHOLD, see O1; heap 2679.198088023088 bytes/n unattributed of 2679.198088023088 total (threshold 669.799522005772) ABOVE THRESHOLD, see O1; heap 2679.864090909091 bytes/n unattributed of 2679.864090909091 total (threshold 669.9660227272727) ABOVE THRESHOLD, see O1; heap 2679.2368506493503 bytes/n unattributed of 2679.2368506493503 total (threshold 669.8092126623376) ABOVE THRESHOLD, see O1; heap 2679.1433662927984 bytes/n unattributed of 2679.1433662927984 total (threshold 669.7858415731996) ABOVE THRESHOLD, see O1

Coverage: python 3520 iterations, 8 of 11 entry points exercised

## Open items

- **O1**: open item O1: unexplained growth: 2682 bytes/n in heap (S1); add a hypothesis (dla ledger add --origin residual --evidence E30 ...) or record it unresolved. Fix: `dla ledger resolve O1 --as unresolved --reason time_budget --remedy "<text>"`
- **T1:os.rss**: column os.rss of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:os.rss_anon**: column os.rss_anon of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:os.vm_data**: column os.vm_data of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:os.vm_hwm**: column os.vm_hwm of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:py.gc_objects**: column py.gc_objects of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:py.traced**: column py.traced of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:py.traced_peak**: column py.traced_peak of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:py.gc_collections**: column py.gc_collections of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:py.gc_objects.pre_gc**: column py.gc_objects.pre_gc of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T1:py.traced.pre_gc**: column py.traced.pre_gc of T1 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S1 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:os.rss**: column os.rss of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:os.rss_anon**: column os.rss_anon of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:os.vm_data**: column os.vm_data of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:os.vm_hwm**: column os.vm_hwm of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:py.gc_objects**: column py.gc_objects of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:py.traced**: column py.traced of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:py.traced_peak**: column py.traced_peak of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:py.gc_collections**: column py.gc_collections of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:py.gc_objects.pre_gc**: column py.gc_objects.pre_gc of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:py.traced.pre_gc**: column py.traced.pre_gc of T4 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S2 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T4:py.type:ingest.models.Record.count**: column py.type:ingest.models.Record.count of T4 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E16 # column py.type:ingest.models.Record.count`
- **T4:py.type:ingest.watchers.CheckpointWatcher.count**: column py.type:ingest.watchers.CheckpointWatcher.count of T4 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E16 # column py.type:ingest.watchers.CheckpointWatcher.count`
- **T5:os.rss**: column os.rss of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.rss_anon**: column os.rss_anon of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.vm_data**: column os.vm_data of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:os.vm_hwm**: column os.vm_hwm of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:py.gc_objects**: column py.gc_objects of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:py.traced**: column py.traced of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:py.traced_peak**: column py.traced_peak of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:py.gc_collections**: column py.gc_collections of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:py.gc_objects.pre_gc**: column py.gc_objects.pre_gc of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:py.traced.pre_gc**: column py.traced.pre_gc of T5 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S3 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T5:py.type:ingest.models.Record.count**: column py.type:ingest.models.Record.count of T5 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E18 # column py.type:ingest.models.Record.count`
- **T5:py.type:ingest.watchers.CheckpointWatcher.count**: column py.type:ingest.watchers.CheckpointWatcher.count of T5 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E18 # column py.type:ingest.watchers.CheckpointWatcher.count`
- **T6:os.rss**: column os.rss of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:os.rss_anon**: column os.rss_anon of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:os.vm_data**: column os.vm_data of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:os.vm_hwm**: column os.vm_hwm of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:py.gc_objects**: column py.gc_objects of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:py.traced**: column py.traced of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:py.traced_peak**: column py.traced_peak of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:py.gc_collections**: column py.gc_collections of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:py.gc_objects.pre_gc**: column py.gc_objects.pre_gc of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:py.traced.pre_gc**: column py.traced.pre_gc of T6 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S4 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T6:py.type:ingest.models.Record.count**: column py.type:ingest.models.Record.count of T6 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E20 # column py.type:ingest.models.Record.count`
- **T6:py.type:ingest.watchers.CheckpointWatcher.count**: column py.type:ingest.watchers.CheckpointWatcher.count of T6 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E20 # column py.type:ingest.watchers.CheckpointWatcher.count`
- **T7:os.rss**: column os.rss of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:os.rss_anon**: column os.rss_anon of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:os.rss_file**: column os.rss_file of T7 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:os.vm_data**: column os.vm_data of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:os.vm_hwm**: column os.vm_hwm of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:py.gc_objects**: column py.gc_objects of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:py.traced**: column py.traced of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:py.traced_peak**: column py.traced_peak of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:py.gc_collections**: column py.gc_collections of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:py.gc_objects.pre_gc**: column py.gc_objects.pre_gc of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:py.traced.pre_gc**: column py.traced.pre_gc of T7 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S5 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T7:py.type:ingest.models.Record.count**: column py.type:ingest.models.Record.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E22 # column py.type:ingest.models.Record.count`
- **T7:py.type:ingest.watchers.CheckpointWatcher.count**: column py.type:ingest.watchers.CheckpointWatcher.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E22 # column py.type:ingest.watchers.CheckpointWatcher.count`
- **T8:type:ingest.models.Record**: column type:ingest.models.Record of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E25 # column type:ingest.models.Record`
- **T8:type:ingest.models.Record.count**: column type:ingest.models.Record.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E26 # column type:ingest.models.Record.count`
- **T8:type:ingest.watchers.CheckpointWatcher**: column type:ingest.watchers.CheckpointWatcher of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E25 # column type:ingest.watchers.CheckpointWatcher`
- **T8:type:ingest.watchers.CheckpointWatcher.count**: column type:ingest.watchers.CheckpointWatcher.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E26 # column type:ingest.watchers.CheckpointWatcher.count`
- **T8:type:list**: column type:list of T8 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E25 # column type:list`
- **T9:os.rss**: column os.rss of T9 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:os.rss_anon**: column os.rss_anon of T9 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:os.rss_file**: column os.rss_file of T9 (stair) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:os.vm_data**: column os.vm_data of T9 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:os.vm_hwm**: column os.vm_hwm of T9 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:py.gc_objects**: column py.gc_objects of T9 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:py.traced**: column py.traced of T9 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:py.traced_peak**: column py.traced_peak of T9 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:py.gc_collections**: column py.gc_collections of T9 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:py.gc_objects.pre_gc**: column py.gc_objects.pre_gc of T9 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:py.traced.pre_gc**: column py.traced.pre_gc of T9 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S8 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T9:py.type:ingest.models.Record.count**: column py.type:ingest.models.Record.count of T9 (steady) is not flat and is tied to no finding. Fix: `dla trend S8`
- **T9:py.type:ingest.watchers.CheckpointWatcher.count**: column py.type:ingest.watchers.CheckpointWatcher.count of T9 (steady) is not flat and is tied to no finding. Fix: `dla trend S8`
- **T10:os.rss**: column os.rss of T10 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T10:os.rss_anon**: column os.rss_anon of T10 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T10:os.vm_data**: column os.vm_data of T10 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T10:os.vm_hwm**: column os.vm_hwm of T10 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T10:py.gc_objects**: column py.gc_objects of T10 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T10:py.traced**: column py.traced of T10 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T10:py.traced_peak**: column py.traced_peak of T10 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T10:py.gc_collections**: column py.gc_collections of T10 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T10:py.gc_objects.pre_gc**: column py.gc_objects.pre_gc of T10 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T10:py.traced.pre_gc**: column py.traced.pre_gc of T10 (steady) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item. Fix: `dla gap S7 --residual (rows under the threshold, or over it with a resolved open item, account for the column)`
- **T10:py.type:ingest.models.Record.count**: column py.type:ingest.models.Record.count of T10 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E29 # column py.type:ingest.models.Record.count`
- **T10:py.type:ingest.watchers.CheckpointWatcher.count**: column py.type:ingest.watchers.CheckpointWatcher.count of T10 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E29 # column py.type:ingest.watchers.CheckpointWatcher.count`
- **heap**: residual growth in heap (2681.656 bytes/n, threshold 670.414) has no resolved open item. Fix: `dla gap --residual then resolve the open item it raises`
- **heap**: residual growth in heap (2680.014 bytes/n, threshold 670.003) has no resolved open item. Fix: `dla gap --residual then resolve the open item it raises`
- **heap**: residual growth in heap (2679.198 bytes/n, threshold 669.800) has no resolved open item. Fix: `dla gap --residual then resolve the open item it raises`
- **heap**: residual growth in heap (2679.864 bytes/n, threshold 669.966) has no resolved open item. Fix: `dla gap --residual then resolve the open item it raises`
- **heap**: residual growth in heap (2679.237 bytes/n, threshold 669.809) has no resolved open item. Fix: `dla gap --residual then resolve the open item it raises`
- **heap**: residual growth in heap (2679.143 bytes/n, threshold 669.786) has no resolved open item. Fix: `dla gap --residual then resolve the open item it raises`

## Findings

### F3: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Module global ingest.parser._newest holds the latest Record; each Record.previous links to the prior one, so the whole history of parsed records stays reachable (unbounded linked list).)
- Growth: `probe:ingest.parser._newest` 15.97 count/n (steady, gate pass, E12)
- Evidence classes: context, growth, static
- Holder: static/global
- Owner: `ingest.parser._newest`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:3`
- Writers: `ingest/parser.py:29`
- Annotation: heuristic name match over the project's sources
- Annotation: module global _newest chains every Record via Record.previous; never cut
- Verified: no
- Sibling hypotheses: F3.n1
- Supporting evidence: E7, E10, E12
- Explanation (inferred): ingest/parser.py:3 _newest is a module-level static holder; parse_line sets record.previous to the prior record at parser.py:29, so every Record chains to all earlier ones and is never released.
- Explanation structure: adds: ingest/parser.py:29; removal: never

### F6: **UNCLASSIFIED**

- State: confirmed
- Label: UNCLASSIFIED (Module dict ingest.registry._LISTENERS gets a new CheckpointWatcher per batch (pipeline.py:28) and nothing unsubscribes; each watcher holds a sensors tuple and a marks dict.)
- Growth: `probe:ingest.registry._LISTENERS` 7.974 count/n (steady, gate pass, E12)
- Evidence classes: context, growth, static
- Holder: registration-table
- Owner: `ingest.registry._LISTENERS`
- Region: heap
- Shape: steady
- Site: `ingest/registry.py:3`
- Writers: `ingest/registry.py:7`, `ingest/pipeline.py:28`, `ingest/registry.py:6`
- Removers: `ingest/registry.py:24`
- Annotation: heuristic name match over the project's sources
- Annotation: a CheckpointWatcher is subscribed per batch; no unsubscribe exists (reset only for tests)
- Verified: no
- Sibling hypotheses: F6.n1
- Supporting evidence: E7, E10, E12
- Explanation (inferred): ingest/registry.py:3 _LISTENERS is a registration-table holder; process_batch at pipeline.py:28 subscribes a new CheckpointWatcher every batch; the only remover, registry.py:24 reset(), is never reached in production (tests only), so watchers accumulate.
- Explanation structure: adds: ingest/pipeline.py:28; removal: unreachable at ingest/registry.py:24

### F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Site: `ingest/parser.py:29`
- Reason: not_exercised
- Remedy: Same site and mechanism as confirmed F3 (parser._newest chain).
- Supporting evidence: E6

### F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Site: `ingest/registry.py:3`
- Reason: not_exercised
- Remedy: Same site and mechanism as confirmed F6 (registry._LISTENERS); no separate test needed.
- Supporting evidence: E6

### F5: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: registration-table
- Owner: `ingest.pipeline.Pipeline.process_batch`
- Region: heap
- Site: `ingest/pipeline.py:28`
- Reason: not_exercised
- Remedy: Registration call site of confirmed F6; not a separate holder.
- Supporting evidence: E7

### F4: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: context, static
- Holder: container-field
- Owner: `ingest.pipeline.Pipeline.per_sensor`
- Region: heap
- Site: `ingest/pipeline.py:15`
- Writers: `ingest/pipeline.py:25`
- Annotation: heuristic name match over the project's sources
- Sibling hypotheses: F4.n1
- Reason: not_exercised
- Remedy: per_sensor has at most 64 keys (SENSORS); no probe column of its own, whole-process growth belongs to F3/F6. Add a driver probe on Pipeline.per_sensor to refute.
- Supporting evidence: E7, E10

### F4.n1: Hypothesis N-HWM: per_sensor is bounded by the 64 sensor ids (UNCLASSIFIED)

- State: unresolved(not_exercised)
- Evidence classes: static
- Sibling hypothesis of: F4
- Hypothesis: N-HWM: per_sensor is bounded by the 64 sensor ids
- Refuted if: per_sensor entries keep growing over extra cycles
- Reason: not_exercised
- Remedy: Needs a per_sensor-specific probe.
- Supporting evidence: E7

### F7: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: static/global
- Owner: `ingest.models.Record.previous`
- Region: heap
- Shape: steady
- Site: `ingest/parser.py:29`
- Reason: not_exercised
- Remedy: Same leak as confirmed F3 (Record chain); type-count column is a symptom, F3 carries the evidence.
- Supporting evidence: E12

### F8: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Evidence classes: growth
- Holder: registration-table
- Owner: `ingest.watchers.CheckpointWatcher`
- Region: heap
- Shape: steady
- Site: `ingest/pipeline.py:28`
- Reason: not_exercised
- Remedy: Same leak as confirmed F6 (CheckpointWatcher in _LISTENERS); type-count column is a symptom.
- Supporting evidence: E12

## Ruled out

- **F3.n1** (UNCLASSIFIED): discriminator E13
  - Ruled out by: plateau+purge (leak) E13
  - Hypothesis: N-WARM: warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E13
  - Originally supported by: E12
- **F6.n1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (leak) E14
  - Hypothesis: N-WARM: warm-up fill that plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E14
  - Originally supported by: E12

## Blind spots and unknowns

### Coverage gaps

- python/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- python/codeql: missing (codeql is not on PATH or in the tools directory). Install hint: dla install codeql
- python/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)
- python/memray: missing (memray is not installed in the target's python interpreter, so --capture alloc has the tracemalloc report only). Install hint: pip install memray in the interpreter the workload runs under, then dla harness run <H> --capture alloc

### Runtime blind spots

None recorded.

### Entry points not exercisable

None recorded.

### Evidence limitations and unknowns

- E7: partial coverage
- E7: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E7: limitation: writers and removers are textual name matches
- E8: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E8: limitation: workloads were discovered by reading files; none was run
- E9: limitation: harness written; nothing was run (dla harness run measures it)
- E9: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E10: partial coverage (9 of 9 examined)
- E10: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E10: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E10: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E10: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E10: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E11: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E11: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E11: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E11: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E11: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E12: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E12: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E12: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E12: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E13: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E13: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E13: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E13: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E13: limitation: points 1 to 9 come from S1; the extra points from S2
- E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- E14: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E14: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E14: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E14: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E14: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E14: limitation: points 1 to 9 come from S1; the extra points from S4
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E17: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E17: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E17: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E17: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E17: limitation: points 1 to 9 come from S1; the extra points from S2
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E18: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E18: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E18: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E18: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E18: limitation: points 1 to 9 come from S1; the extra points from S2
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E21: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E21: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E21: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E21: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E21: limitation: points 1 to 9 come from S1; the extra points from S4
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E22: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E22: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E22: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E22: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E22: limitation: points 1 to 9 come from S1; the extra points from S4
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: partial coverage
- E23: limitation: the agent lists the 300 types with the most instances, so a type outside the list at a capture has no figure there
- E23: limitation: bytes are shallow sys.getsizeof sizes: what an object points to is counted under its own type
- E23: limitation: objects are those the garbage collector tracks, counted after a full collection; the files are ordered as given
- E23: limitation: each histogram lists only the largest types; a type outside the list has no figure
- E24: limitation: the agent lists the 300 types with the most instances, so a type outside the list at a capture has no figure there
- E24: limitation: bytes are shallow sys.getsizeof sizes: what an object points to is counted under its own type
- E24: limitation: objects are those the garbage collector tracks, counted after a full collection; the files are ordered as given
- E24: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E25: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E25: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E26: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: limitation: p1-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E27: limitation: p4-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E27: limitation: p6-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E27: limitation: p9-native: alloc capture failed: memray is not installed in this interpreter (/usr/local/bin/python3); pip install memray there to add its live-at-exit report
- E27: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E27: limitation: points 1 to 9 come from S1; the extra points from S7
- E27: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E27: limitation: gate and plateau thresholds are provisional (not calibrated)
- E28: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E29: limitation: gate thresholds are provisional (not calibrated)
- E30: partial coverage (1 of 3 examined)
- E30: limitation: F3: growth measured as a count; bytes not attributed
- E30: limitation: F6: growth measured as a count; bytes not attributed
- E30: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E30: limitation: threshold is provisional (not calibrated)
- E31: partial coverage (1 of 3 examined)
- E31: limitation: F3: growth evidence is from S1, not S2; not subtracted
- E31: limitation: F6: growth evidence is from S1, not S2; not subtracted
- E31: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E31: limitation: threshold is provisional (not calibrated)
- E32: partial coverage (1 of 3 examined)
- E32: limitation: F3: growth evidence is from S1, not S3; not subtracted
- E32: limitation: F6: growth evidence is from S1, not S3; not subtracted
- E32: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E32: limitation: threshold is provisional (not calibrated)
- E33: partial coverage (1 of 3 examined)
- E33: limitation: F3: growth evidence is from S1, not S4; not subtracted
- E33: limitation: F6: growth evidence is from S1, not S4; not subtracted
- E33: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E33: limitation: threshold is provisional (not calibrated)
- E34: partial coverage (1 of 3 examined)
- E34: limitation: F3: growth evidence is from S1, not S5; not subtracted
- E34: limitation: F6: growth evidence is from S1, not S5; not subtracted
- E34: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E34: limitation: threshold is provisional (not calibrated)
- E35: partial coverage (1 of 3 examined)
- E35: limitation: F3: growth evidence is from S1, not S8; not subtracted
- E35: limitation: F6: growth evidence is from S1, not S8; not subtracted
- E35: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E35: limitation: threshold is provisional (not calibrated)

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
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | — |
| E2 | run-config | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | probe/probe.json |
| E4 | static-hit | static | ruff 0.16.9 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | sweep/python-ruff.json |
| E5 | static-hit | static | pylint 4.1.3 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | sweep/python-pylint.json |
| E6 | static-hit | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | sweep/python-dla.json |
| E7 | census-holder | static | dla-census 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | census/python.json |
| E8 | workload | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | workload/workloads.json |
| E9 | workload | static | dla 0.1.0 | repo cc284ad9719515b28a7ad90ae6308c336cacaac9 | harness/H1/harness.json |
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (a436fe44f821) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (61eda8b90212) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (61eda8b90212) | trend/T1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S3.json (f2d9502cc9fc) | alloc/A1.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S5.json (83dce4615d65) | alloc/A2.json |
| E15 | growth-series | test | dla 0.1.0 | artifact series/S2.json (d987fa95999e) | trend/T4.json |
| E16 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (d987fa95999e) | trend/T4.json |
| E17 | growth-series | test | dla 0.1.0 | artifact series/S3.json (f2d9502cc9fc) | trend/T5.json |
| E18 | unit-count-series | test | dla 0.1.0 | artifact series/S3.json (f2d9502cc9fc) | trend/T5.json |
| E19 | growth-series | test | dla 0.1.0 | artifact series/S4.json (b0230794837e) | trend/T6.json |
| E20 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (b0230794837e) | trend/T6.json |
| E21 | growth-series | test | dla 0.1.0 | artifact series/S5.json (83dce4615d65) | trend/T7.json |
| E22 | unit-count-series | test | dla 0.1.0 | artifact series/S5.json (83dce4615d65) | trend/T7.json |
| E23 | histogram-diff | live | py-types 0.1.0 | artifact histograms/H1.json (f0300f812a84) | histograms/H1.json |
| E24 | growth-series | live | py-types 0.1.0 | artifact series/S6.json (fcd175394635) | series/S6.json |
| E25 | growth-series | dump | dla 0.1.0 | artifact series/S6.json (fcd175394635) | trend/T8.json |
| E26 | unit-count-series | dump | dla 0.1.0 | artifact series/S6.json (fcd175394635) | trend/T8.json |
| E27 | discriminator | test | dla 0.1.0 | artifact series/S8.json (c2baa710f231) | alloc/A3.json |
| E28 | growth-series | test | dla 0.1.0 | artifact series/S7.json (9d3e66757baf) | trend/T10.json |
| E29 | unit-count-series | test | dla 0.1.0 | artifact series/S7.json (9d3e66757baf) | trend/T10.json |
| E30 | residual | test | dla 0.1.0 | artifact series/S1.json (61eda8b90212) | gap/residual-S1-E30.json |
| E31 | residual | test | dla 0.1.0 | artifact series/S2.json (d987fa95999e) | gap/residual-S2-E31.json |
| E32 | residual | test | dla 0.1.0 | artifact series/S3.json (f2d9502cc9fc) | gap/residual-S3-E32.json |
| E33 | residual | test | dla 0.1.0 | artifact series/S4.json (b0230794837e) | gap/residual-S4-E33.json |
| E34 | residual | test | dla 0.1.0 | artifact series/S5.json (83dce4615d65) | gap/residual-S5-E34.json |
| E35 | residual | test | dla 0.1.0 | artifact series/S8.json (c2baa710f231) | gap/residual-S8-E35.json |
