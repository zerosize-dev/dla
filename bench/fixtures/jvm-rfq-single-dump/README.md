# jvm-rfq-single-dump

Category `impossible`, entry path `artifact`. `project/` holds one heap dump,
byte for byte the middle dump of `jvm-rfq-dump-series` (3000 requests;
`dumps/rfq-1c9e.hprof.gz`, 1,432,003 bytes gzip, 6,127,943 raw; git stores the
blob once). There is no `[[leak]]` and no `[[non_leak]]` in `expected.toml`:
the dump alone does not decide the question.

## Why nothing can be confirmed

The dump shows a large static holder, `quotes.RfqDesk.OPEN`, with 1,576,448
retained bytes and 27,002 dominated objects. A warm-up fill that will stop and
growth that will continue look identical in a single dump (the partner
`jvm-rfq-dump-series-capped` holds 800,272 bytes and 13,502 objects at the top
of its fill). Growth is a property of change between dumps of one process, and
there is only one point.

## Correct end

A suspected finding on the holder `quotes.RfqDesk.OPEN` (owner only, no site),
state `suspected_static_only` or `suspected`, with a remedy that asks for more
dumps of the same process taken at increasing load (the run needs at least
three). No confirmed claim, no claim carrying a source site, and no refutation
either: ruling the holder out from one dump is as unsupported as confirming it.

## Capture method

See `jvm-rfq-dump-series`: `jcmd <pid> GC.heap_dump` on a JVM
(`-Xmx64m -Xms64m -XX:+UseSerialGC`, JDK 21) after 3000 requests, `gzip -9 -n`.
