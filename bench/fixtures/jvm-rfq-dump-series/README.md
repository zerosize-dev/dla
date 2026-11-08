# jvm-rfq-dump-series

Category `archetype-leak`, entry path `artifact` (trap half of a pair, partner
`jvm-rfq-dump-series-capped`). Cause `L-COLLECT`, holder `static/global`,
region `heap`. The fixture has no source: `project/` holds three heap dumps of
one process and nothing else.

## Mechanism

The process is a quote desk that registers every request-for-quote in a static
`HashMap` (`quotes.RfqDesk.OPEN`) and never takes one out. Each entry keeps a
256-byte terms array and a one-element list. The holder grows by a constant
amount per request, so its retained size and its object count rise at every
dump. The source of the generator is not part of the fixture; the owner is
named by the dump alone.

Expected finding: owner `quotes.RfqDesk.OPEN` (no `site`), holder
`static/global`, confirmed from the series. A confirmed claim that carries a
source site is an overclaim: the dumps' thread frames are context only.

## Capture method

One JVM (`-Xmx64m -Xms64m -XX:+UseSerialGC`, JDK 21) reads request counts from
its standard input and registers requests up to that count. After each count
`jcmd <pid> GC.heap_dump <file>` writes a dump of the same live process
(`GC.heap_dump` runs a full collection first, so only reachable objects are
written). Counts: 1500, 3000, 4500 requests. Dumps were compressed with
`gzip -9 -n`. File names are arbitrary hex tags that do not encode the order;
the order is the header timestamp.

| file | requests | header timestamp (ms) | raw size | gzip size |
|---|---|---|---|---|
| `dumps/rfq-7f3a.hprof.gz` | 1500 | 1794150521087 | 5,107,113 | 1,305,776 |
| `dumps/rfq-1c9e.hprof.gz` | 3000 | 1794150521213 | 6,127,943 | 1,432,003 |
| `dumps/rfq-b04d.hprof.gz` | 4500 | 1794150521354 | 7,173,366 | 1,558,491 |

Total added: 4,296,270 bytes (the middle dump is shared byte for byte with
`jvm-rfq-single-dump`, which adds no new blob).

## Measurements

Retained bytes and dominated object count of the static holder, as
`dla summarize heap-graph` reports them for the shipped dumps (retained bytes
are approximate; counts are exact):

| dump (requests) | `quotes.RfqDesk.OPEN` retained | dominated objects |
|---|---|---|
| 1500 | 788,256 B | 13,502 |
| 3000 | 1,576,448 B | 27,002 |
| 4500 | 2,372,832 B | 40,502 |

Both columns rise strictly across every interval. The increase of the second
interval (796,384 B, 13,500 objects) is 101% of the first (788,192 B, 13,500
objects), so the growth does not slow. The JVM baseline (about 4.5 MB of
classes and interned strings) is identical in every dump and is not retained by
this holder. File size is not a signal: it is dominated by the JVM
baseline data and compresses unevenly.

## Pair

The partner holds the same shape of data in an access-ordered map that evicts
past 1500 entries; its first interval looks like growth too, and the later one
is flat. Two dumps cannot tell the pair apart; three can.
