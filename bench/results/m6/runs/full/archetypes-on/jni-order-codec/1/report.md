# DLA report

Run: `20261110-195749-82a5`

Status: INCOMPLETE (90 open)

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 23 enumerated, 23 dispositioned; confirmed 1, strong 1, refuted 9, suspected 12, unresolved 0

Residual: heap 1451.4 bytes/n unattributed of 1451.4 total (threshold 362.85) ABOVE THRESHOLD, see O2; heap 1444.2678678678678 bytes/n unattributed of 1444.2678678678678 total (threshold 361.06696696696696) ABOVE THRESHOLD, see O2; heap 1444.4234234234234 bytes/n unattributed of 1444.4234234234234 total (threshold 361.10585585585585) ABOVE THRESHOLD, see O2; heap 1442.4494894894897 bytes/n unattributed of 1442.4494894894897 total (threshold 360.6123723723723) ABOVE THRESHOLD, see O2; heap 1444.067619047619 bytes/n unattributed of 1444.067619047619 total (threshold 361.0169047619047) ABOVE THRESHOLD, see O2; heap 1441.1166666666668 bytes/n unattributed of 1441.1166666666668 total (threshold 360.2791666666667) ABOVE THRESHOLD, see O2; heap 1441.549514899515 bytes/n unattributed of 1441.549514899515 total (threshold 360.3873787248788) ABOVE THRESHOLD, see O2; heap 1444.5788288288288 bytes/n unattributed of 1444.5788288288288 total (threshold 361.1447072072072) ABOVE THRESHOLD, see O2; heap 1442.2676876876876 bytes/n unattributed of 1442.2676876876876 total (threshold 360.5669219219219) ABOVE THRESHOLD, see O2; heap 1442.4736936936938 bytes/n unattributed of 1442.4736936936938 total (threshold 360.61842342342345) ABOVE THRESHOLD, see O2; heap 1451.351051051051 bytes/n unattributed of 1451.351051051051 total (threshold 362.83776276276274) ABOVE THRESHOLD, see O2; heap 1444.0563904563903 bytes/n unattributed of 1444.0563904563903 total (threshold 361.0140976140976) ABOVE THRESHOLD, see O2; heap 1440.7662462462465 bytes/n unattributed of 1440.7662462462465 total (threshold 360.19156156156157) ABOVE THRESHOLD, see O2; heap 1442.043243243243 bytes/n unattributed of 1442.043243243243 total (threshold 360.5108108108108) ABOVE THRESHOLD, see O2; heap 1446.0774774774775 bytes/n unattributed of 1446.0774774774775 total (threshold 361.5193693693694) ABOVE THRESHOLD, see O2; heap 1442.7078507078506 bytes/n unattributed of 1442.7078507078506 total (threshold 360.67696267696266) ABOVE THRESHOLD, see O2; heap 1443.96996996997 bytes/n unattributed of 1443.96996996997 total (threshold 360.9924924924925) ABOVE THRESHOLD, see O2; heap 1441.6411866411866 bytes/n unattributed of 1441.6411866411866 total (threshold 360.41029666029664) ABOVE THRESHOLD, see O2; heap 1443.558221958222 bytes/n unattributed of 1443.558221958222 total (threshold 360.8895554895555) ABOVE THRESHOLD, see O2; heap 1446.695195195195 bytes/n unattributed of 1446.695195195195 total (threshold 361.6737987987988) ABOVE THRESHOLD, see O2; heap 1442.638352638353 bytes/n unattributed of 1442.638352638353 total (threshold 360.65958815958817) ABOVE THRESHOLD, see O2; non-heap 6.048408408408408 bytes/n unattributed of 6.048408408408408 total (threshold 29.865479135135008); non-heap 0 bytes/n unattributed of -4.853745173745174 total (threshold 11.79953337451728); non-heap 0 bytes/n unattributed of -0.8338738738738739 total (threshold 9.55852255135125); non-heap 0 bytes/n unattributed of -10.06063063063063 total (threshold 16.71253694594589); non-heap 0 bytes/n unattributed of -6.945045045045045 total (threshold 11.805102324324324); non-heap 0.008408408408408397 bytes/n unattributed of 0.008408408408408397 total (threshold 24.0784162162163); non-heap 0 bytes/n unattributed of -3.4636236236236235 total (threshold 16.70272524204212); non-heap 0 bytes/n unattributed of -1.0725225225225226 total (threshold 11.378668783783782); non-heap 4.489317889317889 bytes/n unattributed of 4.489317889317889 total (threshold 8.05232525096506); non-heap 0 bytes/n unattributed of -2.373673673673674 total (threshold 11.342943699699712); non-heap 2.4108108108108106 bytes/n unattributed of 2.4108108108108106 total (threshold 11.384536216216215); non-heap 0 bytes/n unattributed of -1.232132132132132 total (threshold 8.234306972972972); non-heap 0 bytes/n unattributed of -0.3207207207207207 total (threshold 8.040385945945944); non-heap 0 bytes/n unattributed of -2.944975744975745 total (threshold 6.299991733887743); non-heap 0 bytes/n unattributed of -5.519219219219218 total (threshold 5.980297027027204); non-heap 0 bytes/n unattributed of -4.196996996996997 total (threshold 6.62726652252248); non-heap 0 bytes/n unattributed of -10.549463749463747 total (threshold 7.165763706563656); non-heap 0 bytes/n unattributed of -3.1396825396825396 total (threshold 8.7124597837838); non-heap 0 bytes/n unattributed of -1.6846846846846848 total (threshold 9.660675027027024); non-heap 11.63111111111111 bytes/n unattributed of 11.63111111111111 total (threshold 14.751469297297454); non-heap 0 bytes/n unattributed of -0.0707207207207207 total (threshold 7.346416567567568)

Coverage: jvm 17760 iterations, 0 of 17 entry points exercised; rust no iterations, 0 of 10 entry points exercised

## Open items

- **T3:jvm.class:byte[].count**: column jvm.class:byte[].count of T3 (steady) is not flat and is tied to no finding. Fix: `dla trend S3`
- **T3:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T3 (steady) is not flat and is tied to no finding. Fix: `dla trend S3`
- **T3:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T3 (steady) is not flat and is tied to no finding. Fix: `dla trend S3`
- **T4:jvm.class:byte[].count**: column jvm.class:byte[].count of T4 (steady) is not flat and is tied to no finding. Fix: `dla trend S5`
- **T4:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T4 (steady) is not flat and is tied to no finding. Fix: `dla trend S5`
- **T4:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T4 (steady) is not flat and is tied to no finding. Fix: `dla trend S5`
- **T5:jvm.class:byte[].count**: column jvm.class:byte[].count of T5 (steady) is not flat and is tied to no finding. Fix: `dla trend S15`
- **T5:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T5 (steady) is not flat and is tied to no finding. Fix: `dla trend S15`
- **T5:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T5 (steady) is not flat and is tied to no finding. Fix: `dla trend S15`
- **T6:jvm.class:byte[].count**: column jvm.class:byte[].count of T6 (steady) is not flat and is tied to no finding. Fix: `dla trend S17`
- **T6:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T6 (steady) is not flat and is tied to no finding. Fix: `dla trend S17`
- **T6:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T6 (steady) is not flat and is tied to no finding. Fix: `dla trend S17`
- **T7:jvm.class:byte[].count**: column jvm.class:byte[].count of T7 (steady) is not flat and is tied to no finding. Fix: `dla trend S19`
- **T7:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla trend S19`
- **T7:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T7 (steady) is not flat and is tied to no finding. Fix: `dla trend S19`
- **T8:jvm.class:byte[].count**: column jvm.class:byte[].count of T8 (steady) is not flat and is tied to no finding. Fix: `dla trend S21`
- **T8:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla trend S21`
- **T8:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T8 (steady) is not flat and is tied to no finding. Fix: `dla trend S21`
- **T9:jvm.class:byte[].count**: column jvm.class:byte[].count of T9 (steady) is not flat and is tied to no finding. Fix: `dla trend S23`
- **T9:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T9 (steady) is not flat and is tied to no finding. Fix: `dla trend S23`
- **T9:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T9 (steady) is not flat and is tied to no finding. Fix: `dla trend S23`
- **T10:jvm.class:byte[].count**: column jvm.class:byte[].count of T10 (steady) is not flat and is tied to no finding. Fix: `dla trend S25`
- **T10:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T10 (steady) is not flat and is tied to no finding. Fix: `dla trend S25`
- **T10:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T10 (steady) is not flat and is tied to no finding. Fix: `dla trend S25`
- **T11:jvm.class:byte[].count**: column jvm.class:byte[].count of T11 (steady) is not flat and is tied to no finding. Fix: `dla trend S27`
- **T11:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T11 (steady) is not flat and is tied to no finding. Fix: `dla trend S27`
- **T11:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T11 (steady) is not flat and is tied to no finding. Fix: `dla trend S27`
- **T12:jvm.class:byte[].count**: column jvm.class:byte[].count of T12 (steady) is not flat and is tied to no finding. Fix: `dla trend S29`
- **T12:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T12 (steady) is not flat and is tied to no finding. Fix: `dla trend S29`
- **T12:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T12 (steady) is not flat and is tied to no finding. Fix: `dla trend S29`
- **T13:jvm.class:byte[].count**: column jvm.class:byte[].count of T13 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E5 # column jvm.class:byte[].count`
- **T13:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T13 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E5 # column jvm.class:java.lang.String.count`
- **T13:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T13 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E5 # column jvm.class:shop.codec.Entry.count`
- **T14:jvm.class:byte[].count**: column jvm.class:byte[].count of T14 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E7 # column jvm.class:byte[].count`
- **T14:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T14 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E7 # column jvm.class:java.lang.String.count`
- **T14:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T14 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E7 # column jvm.class:shop.codec.Entry.count`
- **T16:type:byte[]**: column type:byte[] of T16 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E10 # column type:byte[]`
- **T16:type:byte[].count**: column type:byte[].count of T16 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E11 # column type:byte[].count`
- **T16:type:java.lang.String**: column type:java.lang.String of T16 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E10 # column type:java.lang.String`
- **T16:type:java.lang.String.count**: column type:java.lang.String.count of T16 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E11 # column type:java.lang.String.count`
- **T16:type:jdk.internal.vm.FillerElement[]**: column type:jdk.internal.vm.FillerElement[] of T16 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E10 # column type:jdk.internal.vm.FillerElement[]`
- **T17:stack:libjvm.so+0xd004e1#2**: column stack:libjvm.so+0xd004e1#2 of T17 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E12 # column stack:libjvm.so+0xd004e1#2`
- **T17:stack:libjvm.so+0xd004e1#2.count**: column stack:libjvm.so+0xd004e1#2.count of T17 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E13 # column stack:libjvm.so+0xd004e1#2.count`
- **T17:stack:{closure#0} (native/src/lib.rs:43)**: column stack:{closure#0} (native/src/lib.rs:43) of T17 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E12 # column stack:{closure#0} (native/src/lib.rs:43)`
- **T17:stack:libjvm.so+0xd004e1#6**: column stack:libjvm.so+0xd004e1#6 of T17 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E12 # column stack:libjvm.so+0xd004e1#6`
- **T17:stack:libjvm.so+0xd004e1#6.count**: column stack:libjvm.so+0xd004e1#6.count of T17 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E13 # column stack:libjvm.so+0xd004e1#6.count`
- **T18:stack:libjvm.so+0xd004e1#2**: column stack:libjvm.so+0xd004e1#2 of T18 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E14 # column stack:libjvm.so+0xd004e1#2`
- **T18:stack:libjvm.so+0xd004e1#2.count**: column stack:libjvm.so+0xd004e1#2.count of T18 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E15 # column stack:libjvm.so+0xd004e1#2.count`
- **T18:stack:{closure#0} (native/src/lib.rs:43)**: column stack:{closure#0} (native/src/lib.rs:43) of T18 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E14 # column stack:{closure#0} (native/src/lib.rs:43)`
- **T18:stack:libjvm.so+0xd004e1#6**: column stack:libjvm.so+0xd004e1#6 of T18 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E14 # column stack:libjvm.so+0xd004e1#6`
- **T18:stack:libjvm.so+0xd004e1#6.count**: column stack:libjvm.so+0xd004e1#6.count of T18 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E15 # column stack:libjvm.so+0xd004e1#6.count`
- **T22:metric:nmt.committed/GC**: column metric:nmt.committed/GC of T22 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E20 # column metric:nmt.committed/GC`
- **T22:metric:nmt.reserved/Code**: column metric:nmt.reserved/Code of T22 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E20 # column metric:nmt.reserved/Code`
- **T22:metric:nmt.committed.delta/Internal**: column metric:nmt.committed.delta/Internal of T22 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E20 # column metric:nmt.committed.delta/Internal`
- **T22:metric:nmt.reserved.delta/Internal**: column metric:nmt.reserved.delta/Internal of T22 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E20 # column metric:nmt.reserved.delta/Internal`
- **T22:metric:nmt.committed.delta/Code**: column metric:nmt.committed.delta/Code of T22 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E20 # column metric:nmt.committed.delta/Code`
- **T22:metric:nmt.committed.delta/Metaspace**: column metric:nmt.committed.delta/Metaspace of T22 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E20 # column metric:nmt.committed.delta/Metaspace`
- **T22:metric:nmt.committed/Thread**: column metric:nmt.committed/Thread of T22 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E20 # column metric:nmt.committed/Thread`
- **T22:metric:nmt.committed/Tracing**: column metric:nmt.committed/Tracing of T22 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E20 # column metric:nmt.committed/Tracing`
- **T22:metric:nmt.reserved/Tracing**: column metric:nmt.reserved/Tracing of T22 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E20 # column metric:nmt.reserved/Tracing`
- **T22:metric:nmt.committed.delta/Tracing**: column metric:nmt.committed.delta/Tracing of T22 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E20 # column metric:nmt.committed.delta/Tracing`
- **T22:metric:nmt.reserved.delta/Tracing**: column metric:nmt.reserved.delta/Tracing of T22 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E20 # column metric:nmt.reserved.delta/Tracing`
- **T22:metric:nmt.committed.delta/Native Memory Tracking**: column metric:nmt.committed.delta/Native Memory Tracking of T22 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E20 # column metric:nmt.committed.delta/Native Memory Tracking`
- **T22:metric:nmt.reserved.delta/Native Memory Tracking**: column metric:nmt.reserved.delta/Native Memory Tracking of T22 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E20 # column metric:nmt.reserved.delta/Native Memory Tracking`
- **T22:metric:nmt.committed/Native Memory Tracking**: column metric:nmt.committed/Native Memory Tracking of T22 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E20 # column metric:nmt.committed/Native Memory Tracking`
- **T22:metric:nmt.reserved/Native Memory Tracking**: column metric:nmt.reserved/Native Memory Tracking of T22 (stair) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E20 # column metric:nmt.reserved/Native Memory Tracking`
- **T23:jvm.class:byte[].count**: column jvm.class:byte[].count of T23 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E22 # column jvm.class:byte[].count`
- **T23:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T23 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E22 # column jvm.class:java.lang.String.count`
- **T23:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T23 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E22 # column jvm.class:shop.codec.Entry.count`
- **T24:jvm.class:byte[].count**: column jvm.class:byte[].count of T24 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E24 # column jvm.class:byte[].count`
- **T24:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T24 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E24 # column jvm.class:java.lang.String.count`
- **T24:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T24 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E24 # column jvm.class:shop.codec.Entry.count`
- **T25:jvm.class:byte[].count**: column jvm.class:byte[].count of T25 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E26 # column jvm.class:byte[].count`
- **T25:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T25 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E26 # column jvm.class:java.lang.String.count`
- **T25:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T25 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E26 # column jvm.class:shop.codec.Entry.count`
- **T26:jvm.class:byte[].count**: column jvm.class:byte[].count of T26 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E28 # column jvm.class:byte[].count`
- **T26:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T26 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E28 # column jvm.class:java.lang.String.count`
- **T26:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T26 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E28 # column jvm.class:shop.codec.Entry.count`
- **T27:jvm.class:byte[].count**: column jvm.class:byte[].count of T27 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E30 # column jvm.class:byte[].count`
- **T27:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T27 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E30 # column jvm.class:java.lang.String.count`
- **T27:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T27 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E30 # column jvm.class:shop.codec.Entry.count`
- **T28:jvm.class:byte[].count**: column jvm.class:byte[].count of T28 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E32 # column jvm.class:byte[].count`
- **T28:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T28 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E32 # column jvm.class:java.lang.String.count`
- **T28:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T28 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E32 # column jvm.class:shop.codec.Entry.count`
- **T29:jvm.class:byte[].count**: column jvm.class:byte[].count of T29 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E34 # column jvm.class:byte[].count`
- **T29:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T29 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E34 # column jvm.class:java.lang.String.count`
- **T29:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T29 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E34 # column jvm.class:shop.codec.Entry.count`
- **T30:jvm.class:byte[].count**: column jvm.class:byte[].count of T30 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E36 # column jvm.class:byte[].count`
- **T30:jvm.class:java.lang.String.count**: column jvm.class:java.lang.String.count of T30 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E36 # column jvm.class:java.lang.String.count`
- **T30:jvm.class:shop.codec.Entry.count**: column jvm.class:shop.codec.Entry.count of T30 (steady) is not flat and is tied to no finding. Fix: `dla ledger add --origin growth-series --evidence E36 # column jvm.class:shop.codec.Entry.count`

## Workers

- **jvm** (jvm): status done; 98 lines merged; slice done; torn tail: no
- **rust** (rust): status done; 11 lines merged; slice done; torn tail: no

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
- Writers: `kotlin/shop/Audit.kt:12`, `kotlin/shop/Encoder.kt:19`
- Annotation: heuristic name match over the project's sources
- Annotation: Audit.record adds one 1536-byte sample per inline frame; no remover
- Verified: agrees (verifier site `kotlin/shop/Audit.kt:12`, holder static/global, label L-COLLECT; bundle 9449eb761d7b)
- Sibling hypotheses: jvm/F3.n1
- Supporting evidence: jvm/E4, jvm/E7, jvm/E12, jvm/E13, jvm/E15, jvm/E20
- Explanation (inferred): Audit.trail (static ArrayList at kotlin/shop/Audit.kt:9) gains an Entry with a 1536-byte payload copy per inline frame at kotlin/shop/Audit.kt:12 (Audit.record); nothing removes entries, never, so the heap grows with orders.
- Explanation structure: adds: kotlin/shop/Audit.kt:12; removal: never

### jvm/F19: **L-HANDLE**

- State: confirmed
- Label: L-HANDLE (evidence: jvm/E8 growth-series, jvm/E25 alloc-stack)
- Growth: `native.live_blocks@native/src/lib.rs:40` 0.125 count/n (steady, gate pass, jvm/E12)
- Evidence classes: allocation, context, discriminator, growth
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.open`
- Region: off-heap
- Shape: steady
- Site: `native/src/lib.rs:40`
- Writers: `kotlin/shop/Native.kt:9`, `kotlin/shop/Encoder.kt:11`, `kotlin/shop/Encoder.kt:9`
- Annotation: native allocation under shop.codec.Native.open (Java_shop_codec_Native_open); found by dla correlate
- Annotation: Native.open allocates a native frame per encode; the bulk branch (size > 16384) returns without Native.close
- Annotation: Native.open at Encoder.kt:9; the bulk branch (line 14) returns without Native.close (line 18)
- Verified: DISAGREES (site: finding `native/src/lib.rs:40` vs verifier `kotlin/shop/Encoder.kt:13`; bundle f1fe411f8fbd); finding mechanism: Native.open (native/src/lib.rs:40) allocates a native frame per encode at kotlin/shop/Encoder.kt:9; the bulk branch for frames over 16384 bytes returns at line 14 and no remover found: Native.close is never reached, so the handle leaks.; verifier mechanism: OrderEncoder.encode opens a native frame handle (Rust HashMap of frames); the bulk branch (frames over 16384 bytes, every 8th wholesale order) returns early without Native.close, so one native frame per wholesale order stays live (native.live_blocks +0.125/unit, live_bytes growing)
- Sibling hypotheses: jvm/F19.n1
- Supporting evidence: jvm/E10, jvm/E7, jvm/E12, jvm/E14
- Explanation (inferred): Native.open (native/src/lib.rs:40) allocates a native frame per encode at kotlin/shop/Encoder.kt:9; the bulk branch for frames over 16384 bytes returns at line 14 and no remover found: Native.close is never reached, so the handle leaks.
- Explanation structure: adds: kotlin/shop/Encoder.kt:9; removal: not-found

### jvm/F1: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Site: `kotlin/shop/Audit.kt:9`
- Suggested archetype (unconfirmed): L-COLLECT
- Reason: not_exercised
- Remedy: Same site and holder as confirmed F3 (Audit.trail); detector duplicate, covered by F3
- Supporting evidence: jvm/E3

### jvm/F2: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: container-field
- Owner: `shop.codec.Entry.sample`
- Region: heap
- Site: `kotlin/shop/Audit.kt:3`
- Reason: not_exercised
- Remedy: Entry.sample is the payload retained through Audit.trail; one leak, confirmed as F3
- Supporting evidence: jvm/E4

### jvm/F7: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.<clinit>`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:6`
- Reason: not_exercised
- Remedy: JNI declaration holds no state itself; the native leak behind Native.open is confirmed as F19 (native/src/lib.rs:40); close() not called on bulk path in Encoder.kt
- Supporting evidence: jvm/E4

### jvm/F8: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.open`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:9`
- Reason: not_exercised
- Remedy: JNI declaration holds no state itself; the native leak behind Native.open is confirmed as F19 (native/src/lib.rs:40); close() not called on bulk path in Encoder.kt
- Supporting evidence: jvm/E4

### jvm/F9: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.checksum`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:10`
- Reason: not_exercised
- Remedy: JNI declaration holds no state itself; the native leak behind Native.open is confirmed as F19 (native/src/lib.rs:40); close() not called on bulk path in Encoder.kt
- Supporting evidence: jvm/E4

### jvm/F10: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.size`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:11`
- Reason: not_exercised
- Remedy: JNI declaration holds no state itself; the native leak behind Native.open is confirmed as F19 (native/src/lib.rs:40); close() not called on bulk path in Encoder.kt
- Supporting evidence: jvm/E4

### jvm/F11: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.lines`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:12`
- Reason: not_exercised
- Remedy: JNI declaration holds no state itself; the native leak behind Native.open is confirmed as F19 (native/src/lib.rs:40); close() not called on bulk path in Encoder.kt
- Supporting evidence: jvm/E4

### jvm/F12: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.close`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:13`
- Reason: not_exercised
- Remedy: JNI declaration holds no state itself; the native leak behind Native.open is confirmed as F19 (native/src/lib.rs:40); close() not called on bulk path in Encoder.kt
- Supporting evidence: jvm/E4

### jvm/F13: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.live`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:14`
- Reason: not_exercised
- Remedy: JNI declaration holds no state itself; the native leak behind Native.open is confirmed as F19 (native/src/lib.rs:40); close() not called on bulk path in Encoder.kt
- Supporting evidence: jvm/E4

### jvm/F14: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Evidence classes: static
- Holder: native-handle/global-ref
- Owner: `shop.codec.Native.liveBytes`
- Region: off-heap
- Site: `kotlin/shop/Native.kt:15`
- Reason: not_exercised
- Remedy: JNI declaration holds no state itself; the native leak behind Native.open is confirmed as F19 (native/src/lib.rs:40); close() not called on bulk path in Encoder.kt
- Supporting evidence: jvm/E4

### rust/F1: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: container-field
- Owner: `native.src.frame.Frame.bytes`
- Region: heap
- Site: `native/src/frame.rs:11`
- Reason: no_harness
- Remedy: Frame.bytes is owned by the FRAMES map entry (rust/F2); same growth, measure via the JVM harness shim and dla correlate H1
- Supporting evidence: rust/E3

### rust/F2: **UNCLASSIFIED**

- State: suspected_static_only(no_harness)
- Evidence classes: static
- Holder: static/global
- Owner: `native.src.lib.FRAMES`
- Region: heap
- Site: `native/src/lib.rs:19`
- Writers: `native/src/lib.rs:44`
- Removers: `native/src/lib.rs:70`
- Annotation: Native_open inserts into FRAMES; only Native_close removes; Kotlin encoder skips close for frames over 16384 bytes
- Reason: no_harness
- Remedy: cdylib with no lib/bin target for a Rust harness; measure from the JVM harness with the allocator shim and run dla correlate H1 to tie native/src/lib.rs:44 (FRAMES insert in Native_open) to the managed finding. Static reading: Kotlin OrderEncoder.encode returns without Native.close for frames over 16384 bytes, so FRAMES entries are never removed.
- Supporting evidence: rust/E3

## Ruled out

- **jvm/F3.n1** (UNCLASSIFIED): discriminator jvm/E13
  - Ruled out by: plateau+purge (leak) jvm/E13
  - Hypothesis: N-WARM: Audit.trail fills during warm-up then plateaus
  - Refuted if: trail size plateaus over extra cycles
  - Contradicting evidence: jvm/E13
  - Originally supported by: jvm/E9
- **jvm/F4** (UNCLASSIFIED): discriminator jvm/E36
  - Ruled out by: plateau+purge (non-leak) jvm/E36
  - Contradicting evidence: jvm/E36
  - Originally supported by: jvm/E4
- **jvm/F5** (UNCLASSIFIED): discriminator jvm/E37
  - Ruled out by: plateau+purge (non-leak) jvm/E37
  - Contradicting evidence: jvm/E37
  - Originally supported by: jvm/E4
- **jvm/F6** (UNCLASSIFIED): discriminator jvm/E40
  - Ruled out by: plateau+purge (non-leak) jvm/E40
  - Contradicting evidence: jvm/E40
  - Originally supported by: jvm/E4
- **jvm/F15** (UNCLASSIFIED): discriminator jvm/E38
  - Ruled out by: plateau+purge (non-leak) jvm/E38
  - Contradicting evidence: jvm/E38
  - Originally supported by: jvm/E4
- **jvm/F16** (UNCLASSIFIED): discriminator jvm/E35
  - Ruled out by: plateau+purge (non-leak) jvm/E35
  - Contradicting evidence: jvm/E35
  - Originally supported by: jvm/E4
- **jvm/F17** (UNCLASSIFIED): discriminator jvm/E41
  - Ruled out by: plateau+purge (non-leak) jvm/E41
  - Contradicting evidence: jvm/E41
  - Originally supported by: jvm/E4
- **jvm/F18** (UNCLASSIFIED): discriminator jvm/E39
  - Ruled out by: plateau+purge (non-leak) jvm/E39
  - Contradicting evidence: jvm/E39
  - Originally supported by: jvm/E4
- **jvm/F19.n1** (UNCLASSIFIED): discriminator jvm/E14
  - Ruled out by: plateau+purge (leak) jvm/E14
  - Hypothesis: N-ALLOC: native live bytes are allocator retention, plateau after purge
  - Refuted if: native.live_bytes plateaus after extra points and purge
  - Contradicting evidence: jvm/E14
  - Originally supported by: jvm/E9

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
- E4: limitation: 33232921 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E4: limitation: 21154046 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E4: limitation: 28724855 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E4: limitation: 26895661 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E4: limitation: 21705969 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E4: limitation: 21519839 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E4: limitation: 21039625 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E4: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E4: limitation: 31870625 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E4: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E4: limitation: gate thresholds are provisional (not calibrated)
- E5: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E5: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E5: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E5: limitation: 33232921 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E5: limitation: 21154046 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E5: limitation: 28724855 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E5: limitation: 26895661 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E5: limitation: 21705969 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E5: limitation: 21519839 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E5: limitation: 21039625 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E5: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E5: limitation: 31870625 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E5: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E5: limitation: gate thresholds are provisional (not calibrated)
- E6: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E6: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E6: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E6: limitation: 33379092 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: 21352133 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: 36044704 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: 31873022 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: 21566002 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: 33973490 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: 21625688 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E6: limitation: 21907103 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E6: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E6: limitation: gate thresholds are provisional (not calibrated)
- E7: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E7: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E7: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E7: limitation: 33379092 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: 21352133 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: 36044704 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: 31873022 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: 21566002 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: 33973490 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: 21625688 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E7: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E7: limitation: 21907103 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
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
- E12: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E12: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E13: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E14: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E14: limitation: gate thresholds are provisional (not calibrated)
- E15: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E15: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E15: limitation: gate thresholds are provisional (not calibrated)
- E16: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E16: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E16: limitation: gate thresholds are provisional (not calibrated)
- E17: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E17: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: x is the snapshot number in capture order; t_ms is the time since the first capture where the inputs record one
- E18: limitation: each point comes from a separate snapshot of one process; a figure absent from a snapshot has no value at that point
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: x is the snapshot number in capture order; t_ms is the time since the first capture where the inputs record one
- E19: limitation: each point comes from a separate snapshot of one process; a figure absent from a snapshot has no value at that point
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: x is the snapshot number in capture order; t_ms is the time since the first capture where the inputs record one
- E20: limitation: each point comes from a separate snapshot of one process; a figure absent from a snapshot has no value at that point
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E21: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E21: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E21: limitation: 32061162 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: 22499396 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: 20815895 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: 21480455 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: 33701231 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: 32293677 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: 21784017 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E21: limitation: 22003406 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E22: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E22: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E22: limitation: 32061162 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: 22499396 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: 20815895 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: 21480455 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: 33701231 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: 32293677 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: 21784017 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E22: limitation: 22003406 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E23: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E23: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E23: limitation: 34070154 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: 20967659 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: 30733097 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: 28934315 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: 23843319 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: 31885857 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: 32060946 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E23: limitation: 31809939 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E23: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E24: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E24: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E24: limitation: 34070154 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: 20967659 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: 30733097 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: 28934315 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: 23843319 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: 31885857 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: 32060946 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E24: limitation: 31809939 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E24: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E24: limitation: gate thresholds are provisional (not calibrated)
- E25: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E25: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E25: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E25: limitation: 19969257 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E25: limitation: 33962585 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E25: limitation: 24858616 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E25: limitation: 21217299 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E25: limitation: 32388697 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E25: limitation: 21039097 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E25: limitation: 23785941 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E25: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E25: limitation: 33752826 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E25: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E25: limitation: gate thresholds are provisional (not calibrated)
- E26: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E26: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E26: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E26: limitation: 19969257 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E26: limitation: 33962585 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E26: limitation: 24858616 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E26: limitation: 21217299 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E26: limitation: 32388697 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E26: limitation: 21039097 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E26: limitation: 23785941 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E26: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E26: limitation: 33752826 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E26: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E27: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E27: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E27: limitation: 22397798 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E27: limitation: 25224505 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E27: limitation: 30712098 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E27: limitation: 22987137 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E27: limitation: 31909497 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E27: limitation: 30432513 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E27: limitation: 25536588 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E27: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E27: limitation: 29232673 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E27: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E28: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E28: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E28: limitation: 22397798 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E28: limitation: 25224505 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E28: limitation: 30712098 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E28: limitation: 22987137 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E28: limitation: 31909497 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E28: limitation: 30432513 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E28: limitation: 25536588 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E28: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E28: limitation: 29232673 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E28: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E29: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E29: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E29: limitation: 31288143 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E29: limitation: 23245001 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E29: limitation: 31991280 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E29: limitation: 21060591 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E29: limitation: 22461799 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E29: limitation: 33032031 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E29: limitation: 25301579 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E29: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E29: limitation: 29582417 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E29: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E29: limitation: gate thresholds are provisional (not calibrated)
- E30: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E30: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E30: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E30: limitation: 31288143 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E30: limitation: 23245001 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E30: limitation: 31991280 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E30: limitation: 21060591 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E30: limitation: 22461799 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E30: limitation: 33032031 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E30: limitation: 25301579 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E30: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E30: limitation: 29582417 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E30: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E30: limitation: gate thresholds are provisional (not calibrated)
- E31: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E31: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E31: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E31: limitation: 31676701 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E31: limitation: 29043306 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E31: limitation: 34119851 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E31: limitation: 25954933 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E31: limitation: 31415518 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E31: limitation: 25532117 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E31: limitation: 32158088 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E31: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E31: limitation: 32081566 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E31: limitation: dropped 6 per-type columns (cap of 64, lowest growth first)
- E31: limitation: gate thresholds are provisional (not calibrated)
- E32: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E32: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E32: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E32: limitation: 31676701 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E32: limitation: 29043306 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E32: limitation: 34119851 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E32: limitation: 25954933 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E32: limitation: 31415518 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E32: limitation: 25532117 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E32: limitation: 32158088 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E32: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E32: limitation: 32081566 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E32: limitation: dropped 6 per-type columns (cap of 64, lowest growth first)
- E32: limitation: gate thresholds are provisional (not calibrated)
- E33: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E33: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E33: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E33: limitation: 22943193 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E33: limitation: 22017697 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E33: limitation: 23765967 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E33: limitation: 24673519 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E33: limitation: 20645201 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E33: limitation: 28657248 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E33: limitation: 25717674 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E33: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E33: limitation: 30782658 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E33: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E33: limitation: gate thresholds are provisional (not calibrated)
- E34: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E34: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E34: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E34: limitation: 22943193 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E34: limitation: 22017697 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E34: limitation: 23765967 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E34: limitation: 24673519 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E34: limitation: 20645201 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E34: limitation: 28657248 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E34: limitation: 25717674 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E34: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E34: limitation: 30782658 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E34: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E34: limitation: gate thresholds are provisional (not calibrated)
- E35: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E35: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E35: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E35: limitation: 22534617 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E35: limitation: 31314268 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E35: limitation: 21150290 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E35: limitation: 21879860 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E35: limitation: 20348617 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E35: limitation: 21688720 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E35: limitation: 20401941 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E35: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E35: limitation: 23337401 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E35: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E35: limitation: gate thresholds are provisional (not calibrated)
- E36: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- E36: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E36: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- E36: limitation: 22534617 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E36: limitation: 31314268 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E36: limitation: 21150290 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E36: limitation: 21879860 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E36: limitation: 20348617 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E36: limitation: 21688720 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E36: limitation: 20401941 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E36: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- E36: limitation: 23337401 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E36: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- E36: limitation: gate thresholds are provisional (not calibrated)
- E37: partial coverage (2 of 3 examined)
- E37: limitation: jvm/F3: growth measured as a count; bytes not attributed
- E37: limitation: jvm/F19: growth measured as a count; bytes not attributed
- E37: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E37: limitation: threshold is provisional (not calibrated)
- E38: partial coverage (2 of 3 examined)
- E38: limitation: jvm/F3: growth evidence is from S1, not S2; not subtracted
- E38: limitation: jvm/F19: growth evidence is from S1, not S2; not subtracted
- E38: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E38: limitation: threshold is provisional (not calibrated)
- E39: partial coverage (2 of 3 examined)
- E39: limitation: jvm/F3: growth evidence is from S1, not S3; not subtracted
- E39: limitation: jvm/F19: growth evidence is from S1, not S3; not subtracted
- E39: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E39: limitation: threshold is provisional (not calibrated)
- E40: partial coverage (2 of 3 examined)
- E40: limitation: jvm/F3: growth evidence is from S1, not S4; not subtracted
- E40: limitation: jvm/F19: growth evidence is from S1, not S4; not subtracted
- E40: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E40: limitation: threshold is provisional (not calibrated)
- E41: partial coverage (2 of 3 examined)
- E41: limitation: jvm/F3: growth evidence is from S1, not S5; not subtracted
- E41: limitation: jvm/F19: growth evidence is from S1, not S5; not subtracted
- E41: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E41: limitation: threshold is provisional (not calibrated)
- E42: partial coverage (2 of 3 examined)
- E42: limitation: jvm/F3: growth evidence is from S1, not S14; not subtracted
- E42: limitation: jvm/F19: growth evidence is from S1, not S14; not subtracted
- E42: limitation: off-heap: no gap series for S14; run dla gap S14 first (node.external is used when present)
- E42: limitation: threshold is provisional (not calibrated)
- E43: partial coverage (2 of 3 examined)
- E43: limitation: jvm/F3: growth evidence is from S1, not S15; not subtracted
- E43: limitation: jvm/F19: growth evidence is from S1, not S15; not subtracted
- E43: limitation: off-heap: no gap series for S15; run dla gap S15 first (node.external is used when present)
- E43: limitation: threshold is provisional (not calibrated)
- E44: partial coverage (2 of 3 examined)
- E44: limitation: jvm/F3: growth evidence is from S1, not S16; not subtracted
- E44: limitation: jvm/F19: growth evidence is from S1, not S16; not subtracted
- E44: limitation: off-heap: no gap series for S16; run dla gap S16 first (node.external is used when present)
- E44: limitation: threshold is provisional (not calibrated)
- E45: partial coverage (2 of 3 examined)
- E45: limitation: jvm/F3: growth evidence is from S1, not S17; not subtracted
- E45: limitation: jvm/F19: growth evidence is from S1, not S17; not subtracted
- E45: limitation: off-heap: no gap series for S17; run dla gap S17 first (node.external is used when present)
- E45: limitation: threshold is provisional (not calibrated)
- E46: partial coverage (2 of 3 examined)
- E46: limitation: jvm/F3: growth evidence is from S1, not S18; not subtracted
- E46: limitation: jvm/F19: growth evidence is from S1, not S18; not subtracted
- E46: limitation: off-heap: no gap series for S18; run dla gap S18 first (node.external is used when present)
- E46: limitation: threshold is provisional (not calibrated)
- E47: partial coverage (2 of 3 examined)
- E47: limitation: jvm/F3: growth evidence is from S1, not S19; not subtracted
- E47: limitation: jvm/F19: growth evidence is from S1, not S19; not subtracted
- E47: limitation: off-heap: no gap series for S19; run dla gap S19 first (node.external is used when present)
- E47: limitation: threshold is provisional (not calibrated)
- E48: partial coverage (2 of 3 examined)
- E48: limitation: jvm/F3: growth evidence is from S1, not S20; not subtracted
- E48: limitation: jvm/F19: growth evidence is from S1, not S20; not subtracted
- E48: limitation: off-heap: no gap series for S20; run dla gap S20 first (node.external is used when present)
- E48: limitation: threshold is provisional (not calibrated)
- E49: partial coverage (2 of 3 examined)
- E49: limitation: jvm/F3: growth evidence is from S1, not S21; not subtracted
- E49: limitation: jvm/F19: growth evidence is from S1, not S21; not subtracted
- E49: limitation: off-heap: no gap series for S21; run dla gap S21 first (node.external is used when present)
- E49: limitation: threshold is provisional (not calibrated)
- E50: partial coverage (2 of 3 examined)
- E50: limitation: jvm/F3: growth evidence is from S1, not S22; not subtracted
- E50: limitation: jvm/F19: growth evidence is from S1, not S22; not subtracted
- E50: limitation: off-heap: no gap series for S22; run dla gap S22 first (node.external is used when present)
- E50: limitation: threshold is provisional (not calibrated)
- E51: partial coverage (2 of 3 examined)
- E51: limitation: jvm/F3: growth evidence is from S1, not S23; not subtracted
- E51: limitation: jvm/F19: growth evidence is from S1, not S23; not subtracted
- E51: limitation: off-heap: no gap series for S23; run dla gap S23 first (node.external is used when present)
- E51: limitation: threshold is provisional (not calibrated)
- E52: partial coverage (2 of 3 examined)
- E52: limitation: jvm/F3: growth evidence is from S1, not S24; not subtracted
- E52: limitation: jvm/F19: growth evidence is from S1, not S24; not subtracted
- E52: limitation: off-heap: no gap series for S24; run dla gap S24 first (node.external is used when present)
- E52: limitation: threshold is provisional (not calibrated)
- E53: partial coverage (2 of 3 examined)
- E53: limitation: jvm/F3: growth evidence is from S1, not S25; not subtracted
- E53: limitation: jvm/F19: growth evidence is from S1, not S25; not subtracted
- E53: limitation: off-heap: no gap series for S25; run dla gap S25 first (node.external is used when present)
- E53: limitation: threshold is provisional (not calibrated)
- E54: partial coverage (2 of 3 examined)
- E54: limitation: jvm/F3: growth evidence is from S1, not S26; not subtracted
- E54: limitation: jvm/F19: growth evidence is from S1, not S26; not subtracted
- E54: limitation: off-heap: no gap series for S26; run dla gap S26 first (node.external is used when present)
- E54: limitation: threshold is provisional (not calibrated)
- E55: partial coverage (2 of 3 examined)
- E55: limitation: jvm/F3: growth evidence is from S1, not S27; not subtracted
- E55: limitation: jvm/F19: growth evidence is from S1, not S27; not subtracted
- E55: limitation: off-heap: no gap series for S27; run dla gap S27 first (node.external is used when present)
- E55: limitation: threshold is provisional (not calibrated)
- E56: partial coverage (2 of 3 examined)
- E56: limitation: jvm/F3: growth evidence is from S1, not S28; not subtracted
- E56: limitation: jvm/F19: growth evidence is from S1, not S28; not subtracted
- E56: limitation: off-heap: no gap series for S28; run dla gap S28 first (node.external is used when present)
- E56: limitation: threshold is provisional (not calibrated)
- E57: partial coverage (2 of 3 examined)
- E57: limitation: jvm/F3: growth evidence is from S1, not S29; not subtracted
- E57: limitation: jvm/F19: growth evidence is from S1, not S29; not subtracted
- E57: limitation: off-heap: no gap series for S29; run dla gap S29 first (node.external is used when present)
- E57: limitation: threshold is provisional (not calibrated)
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
- jvm/E7: limitation: 41313513 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E7: limitation: 25651041 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E7: limitation: 20941983 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E8: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E8: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E8: limitation: 41313513 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 25651041 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 20941983 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E8: limitation: 40655998 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 30830364 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 50173288 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 31535194 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 30956320 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: 38821955 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E8: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E8: limitation: gate thresholds are provisional (not calibrated)
- jvm/E9: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E9: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E9: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E9: limitation: 41313513 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 25651041 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 20941983 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E9: limitation: 40655998 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 30830364 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 50173288 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 31535194 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 30956320 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: 38821955 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E9: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E9: limitation: gate thresholds are provisional (not calibrated)
- jvm/E10: partial coverage (1 of 1 examined)
- jvm/E10: limitation: the stack is the project frame of the largest allocation stack at the site, not every frame
- jvm/E10: limitation: slopes are per unit of the series axis; the thresholds are provisional
- jvm/E11: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E11: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E11: limitation: 41313513 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 25651041 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 20941983 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E11: limitation: 40655998 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 30830364 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 50173288 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 31535194 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 30956320 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: 38821955 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E11: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E11: limitation: gate thresholds are provisional (not calibrated)
- jvm/E12: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E12: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E12: limitation: 41313513 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 25651041 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 20941983 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E12: limitation: 40655998 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 30830364 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 50173288 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 31535194 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 30956320 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: 38821955 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E12: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E12: limitation: gate thresholds are provisional (not calibrated)
- jvm/E13: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E13: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E13: limitation: 41313513 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 25651041 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 20941983 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E13: limitation: 40655998 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 30830364 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 50173288 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 31535194 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 30956320 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 38821955 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E13: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E13: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E13: limitation: 33232921 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 21154046 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 28724855 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 26895661 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 21705969 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 21519839 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: 21039625 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E13: limitation: 31870625 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E13: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E13: limitation: points 1 to 9 come from S1; the extra points from S2
- jvm/E13: limitation: 9 columns beyond the 64 the series allows were dropped
- jvm/E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E13: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E13: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E14: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E14: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E14: limitation: 41313513 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 25651041 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 20941983 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E14: limitation: 40655998 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 30830364 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 50173288 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 31535194 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 30956320 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 38821955 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E14: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E14: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E14: limitation: 33379092 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 21352133 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 36044704 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 31873022 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 21566002 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 33973490 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: 21625688 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E14: limitation: 21907103 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E14: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E14: limitation: points 1 to 9 come from S1; the extra points from S4
- jvm/E14: limitation: 8 columns beyond the 64 the series allows were dropped
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
- jvm/E23: limitation: stacks are the shim's live allocations at exit, keyed by their first eight resolved frames; the shim's own hash changes between processes
- jvm/E23: limitation: each capture is the exit record of a separate process: the figures grow with the work done before exit
- jvm/E23: limitation: up to 50173288 live bytes sit in stacks with no frame in the project: they have no site and tie to nothing
- jvm/E24: limitation: stacks are keyed by their first eight resolved frames; each point is the exit record of a separate process
- jvm/E24: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- jvm/E25: limitation: stacks are the shim's live allocations at exit, keyed by their first eight resolved frames; the shim's own hash changes between processes
- jvm/E25: limitation: each capture is the exit record of a separate process: the figures grow with the work done before exit
- jvm/E25: limitation: up to 50173288 live bytes sit in stacks with no frame in the project: they have no site and tie to nothing
- jvm/E26: limitation: stacks are keyed by their first eight resolved frames; each point is the exit record of a separate process
- jvm/E26: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- jvm/E27: limitation: OldObjectSample samples old objects that are still live; it is not a census, so counts and bytes are estimates and a count series over recordings is weak
- jvm/E27: limitation: holders need path-to-gc-roots=true and the recording started before the leak
- jvm/E27: limitation: the recordings are ordered as given
- jvm/E27: limitation: the profile is sampled: counts and bytes are estimates
- jvm/E28: limitation: OldObjectSample samples old objects that are still live; it is not a census, so counts and bytes are estimates and a count series over recordings is weak
- jvm/E28: limitation: holders need path-to-gc-roots=true and the recording started before the leak
- jvm/E28: limitation: the recordings are ordered as given
- jvm/E28: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- jvm/E29: limitation: the JDK 21 thread dump has no creation site and no state: groups never tie to a finding by site
- jvm/E29: limitation: thread names are shown only as a plain normalized pattern; any other name is a shape
- jvm/E30: limitation: the JDK 21 thread dump has no creation site and no state: groups never tie to a finding by site
- jvm/E30: limitation: thread names are shown only as a plain normalized pattern; any other name is a shape
- jvm/E30: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- jvm/E31: limitation: loaders and classes are whole-runtime counts: they never tie to a finding
- jvm/E32: limitation: loaders and classes are whole-runtime counts: they never tie to a finding
- jvm/E32: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- jvm/E33: limitation: native memory categories are whole-runtime figures: they never tie to a finding
- jvm/E33: limitation: values are in KB as NMT prints them (committed and reserved)
- jvm/E34: limitation: native memory categories are whole-runtime figures: they never tie to a finding
- jvm/E34: limitation: values are in KB as NMT prints them (committed and reserved)
- jvm/E34: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- jvm/E35: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E35: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E35: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E35: limitation: 41313513 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E35: limitation: 25651041 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E35: limitation: 20941983 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E35: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E35: limitation: 40655998 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E35: limitation: 30830364 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E35: limitation: 50173288 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E35: limitation: 31535194 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E35: limitation: 30956320 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E35: limitation: 38821955 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E35: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E35: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E35: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E35: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E35: limitation: 32061162 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E35: limitation: 22499396 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E35: limitation: 20815895 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E35: limitation: 21480455 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E35: limitation: 33701231 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E35: limitation: 32293677 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E35: limitation: 21784017 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E35: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E35: limitation: 22003406 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E35: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E35: limitation: points 1 to 9 come from S1; the extra points from S14
- jvm/E35: limitation: 2 columns beyond the 64 the series allows were dropped
- jvm/E35: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E35: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E35: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E35: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E36: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E36: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E36: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E36: limitation: 41313513 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E36: limitation: 25651041 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E36: limitation: 20941983 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E36: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E36: limitation: 40655998 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E36: limitation: 30830364 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E36: limitation: 50173288 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E36: limitation: 31535194 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E36: limitation: 30956320 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E36: limitation: 38821955 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E36: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E36: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E36: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E36: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E36: limitation: 34070154 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E36: limitation: 20967659 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E36: limitation: 30733097 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E36: limitation: 28934315 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E36: limitation: 23843319 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E36: limitation: 31885857 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E36: limitation: 32060946 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E36: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E36: limitation: 31809939 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E36: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E36: limitation: points 1 to 9 come from S1; the extra points from S16
- jvm/E36: limitation: 8 columns beyond the 64 the series allows were dropped
- jvm/E36: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E36: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E36: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E36: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E37: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E37: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E37: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E37: limitation: 41313513 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E37: limitation: 25651041 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E37: limitation: 20941983 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E37: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E37: limitation: 40655998 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E37: limitation: 30830364 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E37: limitation: 50173288 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E37: limitation: 31535194 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E37: limitation: 30956320 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E37: limitation: 38821955 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E37: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E37: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E37: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E37: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E37: limitation: 19969257 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E37: limitation: 33962585 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E37: limitation: 24858616 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E37: limitation: 21217299 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E37: limitation: 32388697 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E37: limitation: 21039097 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E37: limitation: 23785941 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E37: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E37: limitation: 33752826 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E37: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E37: limitation: points 1 to 9 come from S1; the extra points from S18
- jvm/E37: limitation: 9 columns beyond the 64 the series allows were dropped
- jvm/E37: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E37: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E37: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E37: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E38: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E38: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E38: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E38: limitation: 41313513 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E38: limitation: 25651041 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E38: limitation: 20941983 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E38: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E38: limitation: 40655998 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E38: limitation: 30830364 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E38: limitation: 50173288 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E38: limitation: 31535194 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E38: limitation: 30956320 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E38: limitation: 38821955 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E38: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E38: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E38: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E38: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E38: limitation: 22397798 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E38: limitation: 25224505 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E38: limitation: 30712098 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E38: limitation: 22987137 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E38: limitation: 31909497 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E38: limitation: 30432513 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E38: limitation: 25536588 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E38: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E38: limitation: 29232673 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E38: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E38: limitation: points 1 to 9 come from S1; the extra points from S20
- jvm/E38: limitation: 8 columns beyond the 64 the series allows were dropped
- jvm/E38: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E38: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E38: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E38: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E39: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E39: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E39: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E39: limitation: 41313513 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E39: limitation: 25651041 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E39: limitation: 20941983 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E39: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E39: limitation: 40655998 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E39: limitation: 30830364 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E39: limitation: 50173288 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E39: limitation: 31535194 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E39: limitation: 30956320 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E39: limitation: 38821955 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E39: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E39: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E39: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E39: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E39: limitation: 31288143 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E39: limitation: 23245001 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E39: limitation: 31991280 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E39: limitation: 21060591 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E39: limitation: 22461799 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E39: limitation: 33032031 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E39: limitation: 25301579 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E39: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E39: limitation: 29582417 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E39: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E39: limitation: points 1 to 9 come from S1; the extra points from S22
- jvm/E39: limitation: 8 columns beyond the 64 the series allows were dropped
- jvm/E39: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E39: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E39: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E39: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E40: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E40: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E40: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E40: limitation: 41313513 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E40: limitation: 25651041 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E40: limitation: 20941983 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E40: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E40: limitation: 40655998 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E40: limitation: 30830364 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E40: limitation: 50173288 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E40: limitation: 31535194 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E40: limitation: 30956320 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E40: limitation: 38821955 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E40: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E40: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E40: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E40: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E40: limitation: 31676701 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E40: limitation: 29043306 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E40: limitation: 34119851 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E40: limitation: 25954933 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E40: limitation: 31415518 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E40: limitation: 25532117 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E40: limitation: 32158088 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E40: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E40: limitation: 32081566 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E40: limitation: dropped 6 per-type columns (cap of 64, lowest growth first)
- jvm/E40: limitation: points 1 to 9 come from S1; the extra points from S24
- jvm/E40: limitation: 11 columns beyond the 64 the series allows were dropped
- jvm/E40: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E40: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E40: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E40: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E41: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E41: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E41: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E41: limitation: 41313513 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E41: limitation: 25651041 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E41: limitation: 20941983 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E41: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E41: limitation: 40655998 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E41: limitation: 30830364 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E41: limitation: 50173288 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E41: limitation: 31535194 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E41: limitation: 30956320 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E41: limitation: 38821955 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E41: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E41: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E41: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E41: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E41: limitation: 22943193 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E41: limitation: 22017697 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E41: limitation: 23765967 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E41: limitation: 24673519 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E41: limitation: 20645201 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E41: limitation: 28657248 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E41: limitation: 25717674 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E41: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E41: limitation: 30782658 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E41: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E41: limitation: points 1 to 9 come from S1; the extra points from S26
- jvm/E41: limitation: 8 columns beyond the 64 the series allows were dropped
- jvm/E41: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E41: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E41: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E41: limitation: gate and plateau thresholds are provisional (not calibrated)
- jvm/E42: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (61430816 B live at the last point)
- jvm/E42: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E42: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E42: limitation: 41313513 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E42: limitation: 25651041 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E42: limitation: 20941983 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E42: limitation: allocator shim: more than 4096 distinct allocation stacks; the rest is counted in shim.overflow_*
- jvm/E42: limitation: 40655998 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E42: limitation: 30830364 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E42: limitation: 50173288 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E42: limitation: 31535194 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E42: limitation: 30956320 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E42: limitation: 38821955 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E42: limitation: dropped 15 per-type columns (cap of 64, lowest growth first)
- jvm/E42: limitation: native.live_bytes@native/src/lib.rs:40 is Java_shop_codec_Native_open, the native method shop.codec.Native.open declared at kotlin/shop/Native.kt:9 (116025056 B live at the last point)
- jvm/E42: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- jvm/E42: limitation: allocator shim: raw_sites lists the largest 1024 stacks only; smaller live stacks are omitted
- jvm/E42: limitation: 22534617 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E42: limitation: 31314268 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E42: limitation: 21150290 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E42: limitation: 21879860 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E42: limitation: 20348617 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E42: limitation: 21688720 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E42: limitation: 20401941 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E42: limitation: allocator shim: purge kind is not supported by the allocator shim (supported: trim, jemalloc)
- jvm/E42: limitation: 23337401 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- jvm/E42: limitation: dropped 8 per-type columns (cap of 64, lowest growth first)
- jvm/E42: limitation: points 1 to 9 come from S1; the extra points from S28
- jvm/E42: limitation: 9 columns beyond the 64 the series allows were dropped
- jvm/E42: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- jvm/E42: limitation: note: JNI library codec is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines
- jvm/E42: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- jvm/E42: limitation: gate and plateau thresholds are provisional (not calibrated)
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
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | — |
| E2 | run-config | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | probe/probe.json |
| E4 | growth-series | test | dla 0.1.0 | artifact series/S2.json (ab71d6ea11db) | trend/T13.json |
| E5 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (ab71d6ea11db) | trend/T13.json |
| E6 | growth-series | test | dla 0.1.0 | artifact series/S4.json (e5215277d33b) | trend/T14.json |
| E7 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (e5215277d33b) | trend/T14.json |
| E8 | growth-series | dump | dla 0.1.0 | artifact series/S6.json (3667c1dbf33c) | trend/T15.json |
| E9 | unit-count-series | dump | dla 0.1.0 | artifact series/S6.json (3667c1dbf33c) | trend/T15.json |
| E10 | growth-series | dump | dla 0.1.0 | artifact series/S7.json (c97ef2c7a9d7) | trend/T16.json |
| E11 | unit-count-series | dump | dla 0.1.0 | artifact series/S7.json (c97ef2c7a9d7) | trend/T16.json |
| E12 | growth-series | dump | dla 0.1.0 | artifact series/S8.json (93281b87fdf6) | trend/T17.json |
| E13 | unit-count-series | dump | dla 0.1.0 | artifact series/S8.json (93281b87fdf6) | trend/T17.json |
| E14 | growth-series | dump | dla 0.1.0 | artifact series/S9.json (890a9b634f07) | trend/T18.json |
| E15 | unit-count-series | dump | dla 0.1.0 | artifact series/S9.json (890a9b634f07) | trend/T18.json |
| E16 | growth-series | dump | dla 0.1.0 | artifact series/S10.json (8d9f0e234c61) | trend/T19.json |
| E17 | unit-count-series | dump | dla 0.1.0 | artifact series/S10.json (8d9f0e234c61) | trend/T19.json |
| E18 | unit-count-series | dump | dla 0.1.0 | artifact series/S11.json (084db6c22e4e) | trend/T20.json |
| E19 | unit-count-series | dump | dla 0.1.0 | artifact series/S12.json (c4edb0cbf011) | trend/T21.json |
| E20 | growth-series | dump | dla 0.1.0 | artifact series/S13.json (6ff96812309e) | trend/T22.json |
| E21 | growth-series | test | dla 0.1.0 | artifact series/S14.json (3a25a0582de1) | trend/T23.json |
| E22 | unit-count-series | test | dla 0.1.0 | artifact series/S14.json (3a25a0582de1) | trend/T23.json |
| E23 | growth-series | test | dla 0.1.0 | artifact series/S16.json (e80f60f8efa2) | trend/T24.json |
| E24 | unit-count-series | test | dla 0.1.0 | artifact series/S16.json (e80f60f8efa2) | trend/T24.json |
| E25 | growth-series | test | dla 0.1.0 | artifact series/S18.json (ffd9226388e9) | trend/T25.json |
| E26 | unit-count-series | test | dla 0.1.0 | artifact series/S18.json (ffd9226388e9) | trend/T25.json |
| E27 | growth-series | test | dla 0.1.0 | artifact series/S20.json (741cb77ad699) | trend/T26.json |
| E28 | unit-count-series | test | dla 0.1.0 | artifact series/S20.json (741cb77ad699) | trend/T26.json |
| E29 | growth-series | test | dla 0.1.0 | artifact series/S22.json (1a7a774b562b) | trend/T27.json |
| E30 | unit-count-series | test | dla 0.1.0 | artifact series/S22.json (1a7a774b562b) | trend/T27.json |
| E31 | growth-series | test | dla 0.1.0 | artifact series/S24.json (2ad5d14caaed) | trend/T28.json |
| E32 | unit-count-series | test | dla 0.1.0 | artifact series/S24.json (2ad5d14caaed) | trend/T28.json |
| E33 | growth-series | test | dla 0.1.0 | artifact series/S26.json (e4c43f97828b) | trend/T29.json |
| E34 | unit-count-series | test | dla 0.1.0 | artifact series/S26.json (e4c43f97828b) | trend/T29.json |
| E35 | growth-series | test | dla 0.1.0 | artifact series/S28.json (33c722c2f169) | trend/T30.json |
| E36 | unit-count-series | test | dla 0.1.0 | artifact series/S28.json (33c722c2f169) | trend/T30.json |
| E37 | residual | test | dla 0.1.0 | artifact series/S1.json (a6ac2145f8cb) | gap/residual-S1-E37.json |
| E38 | residual | test | dla 0.1.0 | artifact series/S2.json (ab71d6ea11db) | gap/residual-S2-E38.json |
| E39 | residual | test | dla 0.1.0 | artifact series/S3.json (ec6c39acbda2) | gap/residual-S3-E39.json |
| E40 | residual | test | dla 0.1.0 | artifact series/S4.json (e5215277d33b) | gap/residual-S4-E40.json |
| E41 | residual | test | dla 0.1.0 | artifact series/S5.json (7f682e702a23) | gap/residual-S5-E41.json |
| E42 | residual | test | dla 0.1.0 | artifact series/S14.json (3a25a0582de1) | gap/residual-S14-E42.json |
| E43 | residual | test | dla 0.1.0 | artifact series/S15.json (e61022881100) | gap/residual-S15-E43.json |
| E44 | residual | test | dla 0.1.0 | artifact series/S16.json (e80f60f8efa2) | gap/residual-S16-E44.json |
| E45 | residual | test | dla 0.1.0 | artifact series/S17.json (d97fa3f022ac) | gap/residual-S17-E45.json |
| E46 | residual | test | dla 0.1.0 | artifact series/S18.json (ffd9226388e9) | gap/residual-S18-E46.json |
| E47 | residual | test | dla 0.1.0 | artifact series/S19.json (7728e91dea2e) | gap/residual-S19-E47.json |
| E48 | residual | test | dla 0.1.0 | artifact series/S20.json (741cb77ad699) | gap/residual-S20-E48.json |
| E49 | residual | test | dla 0.1.0 | artifact series/S21.json (8e8944ba1d55) | gap/residual-S21-E49.json |
| E50 | residual | test | dla 0.1.0 | artifact series/S22.json (1a7a774b562b) | gap/residual-S22-E50.json |
| E51 | residual | test | dla 0.1.0 | artifact series/S23.json (e120fca5241e) | gap/residual-S23-E51.json |
| E52 | residual | test | dla 0.1.0 | artifact series/S24.json (2ad5d14caaed) | gap/residual-S24-E52.json |
| E53 | residual | test | dla 0.1.0 | artifact series/S25.json (3a25398ed35d) | gap/residual-S25-E53.json |
| E54 | residual | test | dla 0.1.0 | artifact series/S26.json (e4c43f97828b) | gap/residual-S26-E54.json |
| E55 | residual | test | dla 0.1.0 | artifact series/S27.json (fce748400eb5) | gap/residual-S27-E55.json |
| E56 | residual | test | dla 0.1.0 | artifact series/S28.json (33c722c2f169) | gap/residual-S28-E56.json |
| E57 | residual | test | dla 0.1.0 | artifact series/S29.json (1a7029686571) | gap/residual-S29-E57.json |
| jvm/E1 | static-hit | static | spotbugs unknown | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-spotbugs.json |
| jvm/E2 | static-hit | static | detekt 2.0.0-alpha.6 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-detekt.json |
| jvm/E3 | static-hit | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/jvm-dla.json |
| jvm/E4 | census-holder | static | dla-census 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | census/jvm.json |
| jvm/E5 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | workload/jvm.json |
| jvm/E6 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | harness/H1/harness.json |
| jvm/E7 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (fba440a30aa9) | harness/H1/run-S1.json |
| jvm/E8 | growth-series | test | dla 0.1.0 | artifact series/S1.json (a6ac2145f8cb) | trend/T1.json |
| jvm/E9 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (a6ac2145f8cb) | trend/T1.json |
| jvm/E10 | alloc-stack | test | dla-correlate 0.1.0 | artifact series/S1.json (a6ac2145f8cb) | correlate/C1.json |
| jvm/E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (a6ac2145f8cb) | trend/T2.json |
| jvm/E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (a6ac2145f8cb) | trend/T2.json |
| jvm/E13 | discriminator | test | dla 0.1.0 | artifact series/S3.json (ec6c39acbda2) | alloc/A1.json |
| jvm/E14 | discriminator | test | dla 0.1.0 | artifact series/S5.json (7f682e702a23) | alloc/A2.json |
| jvm/E15 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-195749-82a5/harness/H1/out/p1-native/capture/9/heap-1.hprof (f0031dbd8b37) | summaries/heap-1-f0031dbd.json |
| jvm/E16 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-195749-82a5/harness/H1/out/p4-native/capture/393/heap-1.hprof (eeb6e13b324a) | summaries/heap-1-eeb6e13b.json |
| jvm/E17 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-195749-82a5/harness/H1/out/p6-native/capture/621/heap-1.hprof (177f9f64fc6e) | summaries/heap-1-177f9f64.json |
| jvm/E18 | retention-path | dump | dla-hprof 0.1.0 | artifact .dla/20261110-195749-82a5/harness/H1/out/p9-native/capture/961/heap-1.hprof (5208b5a4d31d) | summaries/heap-1-5208b5a4.json |
| jvm/E19 | growth-series | dump | dla-hprof 0.1.0 | artifact series/S6.json (3667c1dbf33c) | series/S6.json |
| jvm/E20 | discriminator | dump | dla-hprof 0.1.0 | artifact series/S6.json (3667c1dbf33c) | series/S6.json |
| jvm/E21 | histogram-diff | live | jcmd-class-histogram 0.1.0 | artifact histograms/H1.json (f684ac13c5e4) | histograms/H1.json |
| jvm/E22 | growth-series | live | jcmd-class-histogram 0.1.0 | artifact series/S7.json (c97ef2c7a9d7) | series/S7.json |
| jvm/E23 | histogram-diff | live | shim-stack-groups 0.1.0 | artifact histograms/H2.json (4233a64d2c96) | histograms/H2.json |
| jvm/E24 | growth-series | live | shim-stack-groups 0.1.0 | artifact series/S8.json (93281b87fdf6) | series/S8.json |
| jvm/E25 | alloc-stack | live | shim-stack-groups 0.1.0 | artifact histograms/H3.json (efa60582bdbf) | histograms/H3.json |
| jvm/E26 | growth-series | live | shim-stack-groups 0.1.0 | artifact series/S9.json (890a9b634f07) | series/S9.json |
| jvm/E27 | alloc-stack | live | jfr-old-object-sample 0.1.0 | artifact allocs/A1.json (154ad4c09f0e) | allocs/A1.json |
| jvm/E28 | growth-series | live | jfr-old-object-sample 0.1.0 | artifact series/S10.json (8d9f0e234c61) | series/S10.json |
| jvm/E29 | non-heap | live | jvm-thread-dump 0.1.0 | artifact nonheap/N1.json (f5f3afe6ec9d) | nonheap/N1.json |
| jvm/E30 | growth-series | live | jvm-thread-dump 0.1.0 | artifact series/S11.json (084db6c22e4e) | series/S11.json |
| jvm/E31 | non-heap | live | jvm-classloader-stats 0.1.0 | artifact nonheap/N2.json (8b89e4e382d0) | nonheap/N2.json |
| jvm/E32 | growth-series | live | jvm-classloader-stats 0.1.0 | artifact series/S12.json (c4edb0cbf011) | series/S12.json |
| jvm/E33 | non-heap | live | jvm-nmt 0.1.0 | artifact nonheap/N3.json (b65510e035ec) | nonheap/N3.json |
| jvm/E34 | growth-series | live | jvm-nmt 0.1.0 | artifact series/S13.json (6ff96812309e) | series/S13.json |
| jvm/E35 | discriminator | test | dla 0.1.0 | artifact series/S15.json (e61022881100) | alloc/A3.json |
| jvm/E36 | discriminator | test | dla 0.1.0 | artifact series/S17.json (d97fa3f022ac) | alloc/A4.json |
| jvm/E37 | discriminator | test | dla 0.1.0 | artifact series/S19.json (7728e91dea2e) | alloc/A5.json |
| jvm/E38 | discriminator | test | dla 0.1.0 | artifact series/S21.json (8e8944ba1d55) | alloc/A6.json |
| jvm/E39 | discriminator | test | dla 0.1.0 | artifact series/S23.json (e120fca5241e) | alloc/A7.json |
| jvm/E40 | discriminator | test | dla 0.1.0 | artifact series/S25.json (3a25398ed35d) | alloc/A8.json |
| jvm/E41 | discriminator | test | dla 0.1.0 | artifact series/S27.json (fce748400eb5) | alloc/A9.json |
| jvm/E42 | discriminator | test | dla 0.1.0 | artifact series/S29.json (1a7029686571) | alloc/A10.json |
| rust/E1 | static-hit | static | rapx 0.7.71 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/rust-rapx.json |
| rust/E2 | static-hit | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | sweep/rust-dla.json |
| rust/E3 | census-holder | static | dla-census 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | census/rust.json |
| rust/E4 | workload | static | dla 0.1.0 | repo 1b6fac58763e9538eaf357191899d22764c3d75c | workload/rust.json |
