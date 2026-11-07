# jvm-audit-chain

Category `archetype-free`. Cause `UNCLASSIFIED`, holder `static/global`, region
`heap`. The mechanism fits no taxonomy row: there is no collection, listener,
worker, thread-local or handle, and the retaining structure is a singly linked
chain of request objects.

## Mechanism

Every request gets a `RequestContext`. `RequestContext.open` passes the
previously created context (kept in the static field `latest`) to the
constructor, which stores it in the final field `previous`; the audit line then
reports the previous request's path and the gap since it arrived. The static
field always points at the newest context, each context points at the one
before it, so every context ever created (with its 384-byte body and strings)
stays reachable from the class. Handlers, ledger history (capped at 20 lines
per account) and the audit sink (counts only) are ordinary and flat.

Expected site: `src/gateway/RequestContext.java:22` (`this.previous = previous`);
alternatives: lines 5 (`latest` declaration), 13 (`previous` declaration),
27-28 (construction with `latest` and the assignment to it).

## Measurements

```
cd project && ./build.sh run 8000
```

| requests | heap in use after GC |
|---|---|
| 2000  | 2607 KB |
| 8000  | 5626 KB |
| 32000 | 17452 KB |

Inside the 32000-request run, 8000 requests = 6508 KB and 32000 requests =
17452 KB: about 0.46 KB per request, linear in the request count; no plateau.
The 8000-request run gives 2832 KB at 2000 and 5626 KB at 8000 (0.47 KB per
request).

Class histogram of `java -Xmx3g -cp out gateway.Main 6000000` sampled every
1.5 s with `jcmd <pid> GC.class_histogram` (live objects only):

| sample | `gateway.RequestContext` | `byte[]` |
|---|---|---|
| 1 | 427642  | 866199 |
| 2 | 877709  | 1766334 |
| 3 | 1352166 | 2715247 |

Instance counts rise linearly with the number of requests served.
