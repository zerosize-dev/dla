# python-frozen-gc

Category: `archetype-free`. Cause `UNCLASSIFIED`, holder `cycle-only`, region `heap`. No pair.

## Mechanism (for authors)

Pricing an itinerary builds a `Journey` whose `Leg`s point back at it (`leg.journey`),
at their neighbours (`previous`, `following`) and are listed in `journey.legs`: a few
dozen objects in reference cycles that only the cycle collector can free. That is
ordinary. `BatchWorker._after_batch` then calls `gc.freeze()` after every batch, so that
the fare table's pages stay shared with the parent process. `gc.freeze()` moves every
object the collector tracks into the permanent generation, including the cyclic garbage
of the batch that the collector has not reached yet (up to one young-generation interval
of allocations at the end of each batch). Objects in the permanent generation are
ignored by every later collection, so that garbage is never freed. Nothing in the
program refers to it: the retained objects are unreachable, and only a reference cycle
keeps each one alive.

This is the close look-alike of a reference-cycle leak. It is `archetype-free` because the
cycles themselves are ordinary and would be collected: the cause is a call that hides
them from the collector, and the shipped Python row (`L-COLLECT`) needs a growing
container that is not here. A run that labels it `L-CYCLE` is forcing a fit: there is no
`L-CYCLE` row for Python. The expected label is `UNCLASSIFIED`.

Minimal edit that removes the growth: delete the `gc.freeze()` call (or call
`gc.unfreeze()` and `gc.collect()` after the batch).

## Measurements

Harness outside `project/` (drives `journeys.load.itineraries` through `BatchWorker`
in batches of 25, `gc.collect()` before each reading, `tracemalloc` started after the
imports, one fresh process per count; `frozen` is `gc.get_freeze_count()`):

```
python3 -B harness.py <project dir> 2000
n=2000 i=1000 traced_kb=866  frozen=18212
n=2000 i=1500 traced_kb=1295 frozen=21410
n=2000 i=2000 traced_kb=1724 frozen=24612
n=2000 after gc.unfreeze() and gc.collect(): traced_kb=1 frozen=0
python3 -B harness.py <project dir> 8000
n=8000 i=4000 traced_kb=3471 frozen=37637
n=8000 i=6000 traced_kb=5212 frozen=50627
n=8000 i=8000 traced_kb=6965 frozen=63666
n=8000 after gc.unfreeze() and gc.collect(): traced_kb=1 frozen=0
```

Growth is linear in itineraries: 0.87 KB per itinerary (1724 KB at 2k, 6965 KB at 8k,
slope 0.87 KB/itinerary), and the frozen count grows by about 6.5 objects per itinerary
on top of the fare table. After `gc.unfreeze()` and a collection the heap is back at
1 KB, so the objects are garbage, not live data. `ru_maxrss` was 21 MB after 2k and
34 MB after 8k. The same numbers hold on 3.12.13 (1708 and 6902 KB) and 3.14.4
(1724 and 6965 KB); the table above is from 3.13.18.

Workload: `python3 -m journeys.load --iterations 8000` takes about 0.2 s.
Tests: `python3 -m unittest discover -s tests` from `project/` (8 tests, pass).
Size: 7 files, 218 lines.
