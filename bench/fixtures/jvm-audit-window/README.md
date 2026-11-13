# jvm-audit-window

Category `non-leak`. Cause `N-INTENT`: growth that is retention on purpose,
with a time bound; the retained set reaches a size set by the traffic rate and
the window and stays there.

## Mechanism

`AuditTail` keeps the administrative actions of the last three hours for the
operator console (its Javadoc says so) in an `ArrayDeque`. Each `append` drops
entries older than three hours measured on the newest entry's timestamp
(`AuditTail.java:17-22`). The driver runs on a simulated clock at about one
action per 1.15 s, so the tail settles near 9400 entries (about 270 B each with
the payload) after about 9400 actions. Until then the heap rises with the
count of entries; after that it is flat. Each entry is dropped three hours after
it arrived, which is the intended behaviour. The discriminator is the window,
not a cap on count: with faster traffic the plateau is higher.

Non-leak site: `src/audit/AuditTail.java:19` (the append).

## Measurements

Toolchain: OpenJDK 21.0.12.

```
cd project && ./build.sh run 30000       # compiles, runs the unit tests, records 30000 actions
```

Single run of 30000 actions (checkpoints at 7500, 15000, 22500, 30000): entries
7500, 9396, 9396, 9397; heap in use 3846, 4432, 4431, 4430 KB. Separate runs,
last checkpoint:

| actions | simulated traffic | entries shown | heap in use after GC |
|---|---|---|---|
| 7500   | 2 h  | 7500 | 3831 KB |
| 30000  | 9 h  | 9397 | 4428 KB |
| 120000 | 38 h | 9396 | 4430 KB |

The heap rises by about 2.3 MB over the first 9400 actions, then the slope is
zero (30000 and 120000 actions differ by 2 KB). Unit tests: 4 in
`test/audit/AuditServiceTest.java`, all pass.
