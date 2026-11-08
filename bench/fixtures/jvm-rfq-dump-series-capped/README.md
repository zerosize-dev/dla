# jvm-rfq-dump-series-capped

Category `decoy-control`, entry path `artifact` (control half of a pair,
partner `jvm-rfq-dump-series`). Cause `N-WARM`. The fixture has no source:
`project/` holds three heap dumps of one process.

## Mechanism

Same process shape as the partner, with one change: requests go into an
access-ordered `LinkedHashMap` subclass (`quotes.RfqDesk.RECENT`) that evicts
its eldest entry once it holds 1500 entries. The first interval of the series
is the fill (clear growth); after that the holder is full and stays at the same
count and size.

Expected end: no confirmed leak. The growth is ruled out as a warm-up fill
(`N-WARM`) using the series: the holder grows between the first two dumps and
is flat between the last two. A run that confirms `quotes.RfqDesk.RECENT`
fails the fixture.

## Capture method

As in the partner: one JVM (`-Xmx64m -Xms64m -XX:+UseSerialGC`, JDK 21), dumps
by `jcmd <pid> GC.heap_dump` at 1000, 1500 and 3000 requests, compressed with
`gzip -9 -n`, neutral file names, order by header timestamp.

| file | requests | header timestamp (ms) | raw size | gzip size |
|---|---|---|---|---|
| `dumps/rfq-e5d2.hprof.gz` | 1000 | 1794150521688 | 4,791,082 | 1,269,291 |
| `dumps/rfq-3a86.hprof.gz` | 1500 | 1794150521809 | 5,131,309 | 1,314,174 |
| `dumps/rfq-90cb.hprof.gz` | 3000 | 1794150521925 | 5,925,801 | 1,387,499 |

Total added: 3,970,964 bytes.

## Measurements

Same parser as the partner:

| dump (requests) | `quotes.RfqDesk.RECENT` retained | dominated objects |
|---|---|---|
| 1000 | 510,061 B | 9,002 |
| 1500 | 762,061 B | 13,502 |
| 3000 | 764,281 B | 13,502 |

First interval: +252,000 B (+49.4%), +4,500 objects. Later interval: +2,220 B
(+0.29%), +0 objects. The holder is full at 1500 requests (13,502 dominated
objects) and does not change after that. The third dump is the largest file
although the holder did not grow; file size is not a signal.
