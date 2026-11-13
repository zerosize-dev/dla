# native-parcel-router

Category `multi-leak`, no pair. Runtime native: C11, `make`, system compiler.
Two leaks in two modules.

## Mechanism (never shown to the model)

`router_handle` (`src/router.c`) takes each parcel record from the feed in
three steps: it interns a trace label for the parcel (`src/intern.c`), opens a
spool file to stage the manifest (`src/spool.c`), then checks the destination
hub and the manifest checksum.

**L1, `L-HANDLE`, about 60% of the growth.** `spool_open` allocates a context
(line 31), a 4 KiB staging buffer (line 34) and opens the file (line 39). The
unknown-hub path and the success path call `spool_close`; the checksum-mismatch
path (`router.c:45`) returns without closing the spool, so the descriptor, the
context and the staging buffer stay behind. The feed damages every 48th manifest
(`--damage-every 48`, the default), so about 2% of the parcels take that path
and each strands about 4.2 KB and one descriptor. Holder
`native-handle/global-ref`, region `off-heap`.

**L2, `UNCLASSIFIED`, about 40% of the growth.** `label_intern` builds a label
from the destination hub, the lane and the parcel sequence number
(`hub-AMS3/lane-02/parcel-00000017`) and keeps it in a pool of 4096 hash
buckets with chained entries (`src/intern.c`, line 52). Every parcel has its own
sequence number, so no label is ever found again and the pool takes one entry
per parcel (49 bytes per entry, 64 with the allocator's header) for the life of
the process. No row of the native archetype reference describes a lookup pool
that only ever adds; the cause is `UNCLASSIFIED`. Holder
`native-handle/global-ref`, region `off-heap`, which is what the census and the
allocation-stack evidence give for a `malloc`ed entry; the entries hang off the
router's `labels` field.

The two shapes differ from `native-tombstone-table`: here the number of entries
in the pool grows by one for every parcel and each entry is a block of its own;
there the live entry count is flat and only the capacity of one array grows.

## Commands and measurements

Build and test:

```
$ cd project && make && make test
all tests passed          (6 tests)
```

Workload, peak resident size (`VmHWM` sampled from `/proc/<pid>/status` by a
wrapper kept outside the repository). The default is 20000 parcels; the damage
rate gives 1 spool left open per 48 parcels, so a run of N parcels holds N/48
descriptors (833 at 40000, under the usual limit of 1024; with a lower limit
`open` fails and the router reports the spool error, which stops the L1 growth):

| command | routed | rejected | peak RSS |
|---|---|---|---|
| `./parcelroute --iterations 2500` | 2423 | 77 | 2116 kB |
| `./parcelroute --iterations 5000` | 4846 | 154 | 2476 kB |
| `./parcelroute --iterations 10000` | 9691 | 309 | 3192 kB |
| `./parcelroute --iterations 20000` | 19382 | 618 | 4684 kB |
| `./parcelroute --iterations 40000` | 38763 | 1237 | 7572 kB |
| `./parcelroute --iterations 80000` (`ulimit -n 4096`) | 77527 | 2473 | 13436 kB |
| `./parcelroute --iterations 160000` (`ulimit -n 4096`) | 155052 | 4948 | 25112 kB |

Growth is linear in the parcel count: 10000 to 20000 adds 1492 kB (149 bytes per
parcel), 20000 to 40000 adds 2888 kB (144 bytes), 80000 to 160000 adds 11676 kB
(146 bytes). The 160000-parcel run takes 0.7 s.

Each leak on its own, measured in scratch copies:

- Spool closed on the checksum path (only L2 left): 3012 kB at 20000, 4252 kB at
  40000, 11760 kB at 160000, about 62 bytes per parcel.
- No damaged manifests (`--damage-every 0`, only L2 left): 4256 kB at 40000, the
  same slope.
- Both leaks give 146 bytes per parcel, so L1 is about 84 bytes (57%) and L2
  about 62 bytes (43%) of the resident growth.

Valgrind (`valgrind --leak-check=full --track-fds=yes ./parcelroute --iterations N`):

| N | definitely + indirectly lost (L1) | still reachable (L2, without the 32 KiB bucket array) | open spool descriptors |
|---|---|---|---|
| 5000 | 424,360 bytes in 103 spools | 245,000 bytes in 5000 labels | 103 |
| 20000 | 1,697,440 bytes in 412 spools | 980,000 bytes in 20000 labels | 412 |
| 40000 | 3,399,000 bytes in 825 spools | 1,960,000 bytes in 40000 labels | 825 |

L1 is 63% and L2 37% of the heap bytes held at exit; the shares in
`expected.toml` (0.6 and 0.4) are rounded between the resident-memory split and
this one.

## Cleanup

`make clean` removes `parcelroute`, `test_router` and the object files; none are
tracked. The spool path defaults to `/dev/null`, so no file is created.
