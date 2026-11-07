# jvm-order-cache

Category `archetype-leak` (trap half of a pair, partner `jvm-order-cache-bounded`).
Cause `L-COLLECT`, holder `static/global`, region `heap`.

## Mechanism

`OrderCache` keeps every confirmed order in a static `HashMap` keyed by an ID
derived from the request (`ord-<customer>-<requestId>`). Entries are added by
`store` on each new request and nothing in the service removes them; the only
removal is `invalidateAll()`, which is called by tests alone. Roughly one request
in eight is a client retry that hits the cache, so the cache looks useful.
The key space grows with traffic, so the retained set grows with it.

Expected site: `src/shop/OrderCache.java:17` (the `put`), alternative: the field
declaration at line 8.

## Distinguishing edit (trap vs control)

The control swaps the field for an access-ordered `LinkedHashMap` that evicts
its eldest entry past 1000 entries (see the diff in the partner's README, 10
changed lines, 24 lines of `diff -ru` output).

## Measurements

Toolchain: OpenJDK 21.0.12. Build, test and run:

```
cd project && ./build.sh run 8000        # compiles, runs unit tests, replays 8000 requests
```

The driver forces a GC at each checkpoint and prints the heap in use. Output at
three iteration counts (checkpoint lines at 1/4 and 4/4 of the run):

| requests | cached orders | heap in use after GC |
|---|---|---|
| 2000  | 1750  | 2444 KB |
| 8000  | 7000  | 5424 KB |
| 32000 | 28000 | 17217 KB |

Within the 32000-request run the heap goes from 5404 KB (8000 requests) to
17217 KB (32000 requests): a slope of about 0.49 KB per request, 0.56 KB per
placed order, linear in the iteration count. Cached orders equal placed orders
(7/8 of the requests); there is no plateau.

Class histogram of a long run (`java -Xmx3g -cp out shop.Main 3000000`, then
`jcmd <pid> GC.class_histogram` at 2 s intervals; the command performs a full
GC first, so only live objects are counted):

| sample | `shop.Order` instances | `shop.Order$Line` instances |
|---|---|---|
| 1 | 866649  | 3218982 |
| 2 | 1902831 | 7067658 |
| 3 | 2625000 | 9750000 |

The final instance count equals the number of placed orders (2625000 of 3000000
requests), so every placed order is still reachable.
