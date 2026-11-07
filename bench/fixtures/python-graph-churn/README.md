# python-graph-churn

Category: `non-leak`. Look-alike cause: `N-HEADROOM`. No pair.

## Mechanism (for authors)

Every conversion parses an outline into a `Tree` whose `Node`s point at their parent
and at the tree, and the tree points at its nodes: each request creates a few hundred
objects in reference cycles that reference counting alone cannot free. That is
ordinary for the cycle collector. `OutlineService.__init__` calls
`tuning.install()`, which raises the young-generation threshold from 700 to 40,000, so
cyclic garbage accumulates for roughly 330 requests (about 5 MB) before a collection
runs, and then all of it is reclaimed. A memory graph therefore shows a sawtooth with a
rising envelope over the first collections and a flat envelope afterwards, which looks
like a leak on a coarse dashboard. Nothing is retained: after a forced collection the
heap is back at its baseline.

## Measurements

Harness outside `project/` (drives `outlines.load.requests`, samples
`tracemalloc` current bytes and the number of `Node` objects found with
`gc.get_objects()` every n/16 requests, with no manual collection in the loop):

```
python3 harness.py <project dir> 2000 8000
n=2000 i=125  traced_kb=2385 node_objects=7556
n=2000 i=250  traced_kb=4807 node_objects=15272
n=2000 i=375  traced_kb=1073 node_objects=3393
n=2000 i=500  traced_kb=3393 node_objects=10774
n=2000 i=625  traced_kb=5809 node_objects=18472
n=2000 i=750  traced_kb=1988 node_objects=6309
... (16 samples)
n=2000 automatic gen0 collections=6 min/max traced_kb=279/5809  mean first half=3055 second half=2939
n=2000 end: after gc.collect(): 3 KB, 0 Node objects
n=8000 i=500  traced_kb=3392 node_objects=10774
n=8000 i=1000 traced_kb=616  node_objects=1941
n=8000 i=4500 traced_kb=5919 node_objects=18827
n=8000 i=5500 traced_kb=451  node_objects=1405
... (16 samples)
n=8000 automatic gen0 collections=23 min/max traced_kb=451/5919  mean first half=2998 second half=3089
n=8000 end: after gc.collect(): 2 KB, 0 Node objects
```

The series rises and falls; the mean of the second half equals the mean of the first
half (2998 vs 3089 KB at 8k, 3055 vs 2939 KB at 2k) and the peak does not grow with
iterations (5.8 MB at 2k, 5.9 MB at 8k). A forced `gc.collect()` returns the heap to
about 3 KB and zero `Node` objects at both iteration counts. With the threshold at the
interpreter default (700) the same workload stays between 98 and 854 KB. Process
`ru_maxrss` after both runs: 35 MB.

Workload: `python3 -m outlines.load --iterations 8000` takes about 2.5 s.
Tests: `python3 -m unittest discover -s tests` from `project/` (8 tests, pass).
Size: 8 files, 254 lines.
