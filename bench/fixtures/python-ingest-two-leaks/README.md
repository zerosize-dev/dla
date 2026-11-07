# python-ingest-two-leaks

Category: `multi-leak`. No pair. Two independent retentions in different modules.

## Mechanism (for authors)

- **L1, `L-REGISTER`, holder `registration-table` (about 70%).** `Pipeline.process_batch`
  creates a `CheckpointWatcher` for every batch and subscribes it in
  `ingest/registry.py` (`_LISTENERS`, a module-level table). Checkpoint events only call
  the watchers; nothing ever unsubscribes them. Each watcher holds a label, the batch's
  sensor names and a per-sensor mark dictionary.
- **L2, `UNCLASSIFIED`, holder `static/global` (about 30%).** `ingest/parser.py` keeps the
  newest record in a module global `_newest` and links each new record to it through
  `record.previous` (used once, to compute a delta). The chain of `Record` objects
  therefore stays reachable from the module forever. No taxonomy row describes a
  linked chain hung off a module global, so the correct label is `UNCLASSIFIED`.

The two holders do not reference each other: watchers do not hold records, and
records do not hold watchers, so each can be sized separately.

## Measurements

Harness outside `project/`: runs `ingest.load.make_batches(n)` through a fresh
`Pipeline`, calls `gc.collect()`, then sizes the two holders with a deep
`sys.getsizeof` walk (shared objects counted once) and also diffs `tracemalloc`
snapshots by allocating file. `registry_kb` is `registry._LISTENERS`, `chain_kb` is the
chain starting at `parser._newest`.

```
python3 harness.py <project dir> 2000 8000
n=2000: listeners=2000 chain_length=4000  registry_kb=3901  chain_kb=1561 registry_share=0.714 chain_share=0.286
   tracemalloc kb by allocating file: parser.py 1569, pipeline.py 277, registry.py 15, watchers.py 3331
n=8000: listeners=8000 chain_length=16000 registry_kb=15728 chain_kb=6230 registry_share=0.716 chain_share=0.284
   tracemalloc kb by allocating file: parser.py 6252, pipeline.py 1118, registry.py 65, watchers.py 13449
slope: registry 2018 B/batch, chain 797 B/batch; shares of growth 0.717 / 0.283
```

Both halves grow linearly with batches: the listener count equals the batch count and
the chain length equals rows (2 per batch). Slopes: 2018 B/batch for the registry
(`L1`), 797 B/batch for the chain (`L2`); shares of growth 0.717 and 0.283 (expected.toml
uses 0.7 and 0.3). Instance sizes depend on the CPython version; the numbers above are
from CPython 3.13, where the split stays within a few points of 70/30.
In the `tracemalloc` view, `watchers.py`, `pipeline.py` (the watcher instance) and
`registry.py` allocations add up to the registry side, and `parser.py` is the chain.

Workload: `python3 -m ingest.load --iterations 8000` takes about 0.3 s.
Tests: `python3 -m unittest discover -s tests` from `project/` (8 tests, pass).
Size: 8 files, 244 lines.
