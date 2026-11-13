# native-tombstone-table

Category `archetype-free`, cause `UNCLASSIFIED` (no pair). Runtime native: C11,
`make`, system compiler.

## Mechanism (never shown to the model)

`src/lease_table.c` is an open-addressing hash table (linear probing, 32-byte
slots) that holds the broker's active leases. A removal marks the slot
`SLOT_DELETED`; the slot is not reused, and only a rebuild clears it. The load
limit counts deleted slots together with live ones (line 102). When the limit
is crossed, `lease_table_put` always grows the table by a quarter and rebuilds
it (line 103), whatever the number of live entries. Rebuilding drops the
deleted slots, but the new, larger array can then absorb that many more
removals before the limit is crossed again, so the array grows with the number
of leases granted, not with the number held.

The workload (`src/broker.c`) holds a window of 4096 leases: every tick grants
one lease and, once the window is full, returns a random one. The number of
live entries is flat at 4096; the capacity grows by about 0.32 slots per tick (10 bytes of
array); the peak resident size counts the old array too while a rebuild copies
it, which gives about 18 bytes per tick.

Expected diagnosis: the growth is the slot array (`malloc` in `alloc_slots`,
line 24, called from the rebuild at line 103). The live count is flat and the
capacity climbs, so the memory is in deleted slots and in the table's growth
policy. It is not a table of entries that are never removed, and no block is
unreferenced at exit.

Why `UNCLASSIFIED`: no shipped archetype row for native describes a container
whose capacity grows while its contents stay flat. `L-HANDLE` (a release
obligation not met) does not apply: every block is freed or reachable, and
valgrind reports nothing lost. The holder is recorded as `native-handle/global-ref`
because that is what the census and the allocation-stack evidence give for a
`malloc`ed array; the array is reached through `lease_table.slots` of the
broker's table (a `container-field` in the source).

Differs from `native-parcel-router`'s second leak (an interning pool): there
the number of entries grows with the traffic; here the entry count is flat and
only the capacity grows.

## Commands and measurements

Build and test:

```
$ cd project && make && make test
all tests passed          (7 tests)
```

Workload, peak resident size (`VmHWM` sampled from `/proc/<pid>/status` by a
wrapper kept outside the repository). `active` is the number of live leases;
the table capacity is read by a small program outside the repository that links the project sources:

| command | active | table capacity | peak RSS |
|---|---|---|---|
| `./leasebroker --iterations 50000` | 4096 | 29090 | 3132 kB |
| `./leasebroker --iterations 100000` | 4096 | 45452 | 4056 kB |
| `./leasebroker --iterations 200000` | 4096 | 88772 | 6416 kB |
| `./leasebroker --iterations 400000` | 4096 | 138706 | 9204 kB |
| `./leasebroker --iterations 800000` | 4096 | 270908 | 16732 kB |
| `./leasebroker --iterations 1600000` | 4096 | 529116 | 31248 kB |
| `./leasebroker --iterations 3200000` | 4096 | 1033428 | 59604 kB |

Growth is linear in the tick count:
800000 to 1600000 adds 14.5 MB (18.1 bytes per tick), 1600000 to 3200000 adds
28.4 MB (17.8 bytes per tick). The 3200000-tick run takes 0.46 s.

Control run (same source with the growth step rebuilt at the same capacity when
the live entries fill less than a quarter of the table, measured in a scratch
copy): peak RSS 2608 kB at 100000 and 400000 ticks, 2612 kB at 1600000, 2992 kB
at 3200000. Flat, so the growth comes from the growth policy and not from the
workload.

Valgrind (`valgrind --leak-check=full ./leasebroker --iterations 200000`):

```
total heap usage: 23 allocs, 21 frees, 14,110,120 bytes allocated
definitely lost: 0 bytes in 0 blocks
still reachable: 2,873,480 bytes in 2 blocks
```

At 800000 ticks the reachable block of the slot array is 8,669,056 bytes (the
other is the 32 KiB list of active ids). Nothing is lost; the memory is held and
still referenced.

Heap growth evidence from the dynamic acceptance (heaptrack, three runs):
`lease_table_put::resize::alloc_slots` at `src/lease_table.c:24` grows by
3.7 MB between runs, one stack, 7 blocks.

## Cleanup

`make clean` removes `leasebroker`, `test_lease` and the object files; none are
tracked.
