# jvm-extension-reload

Category `archetype-leak` (trap half of a pair, partner `jvm-extension-reload-stopped`).
Cause `L-UNLOAD`, holder `registration-table`, region `heap` (classes and their statics
also fill metaspace).

## Mechanism

`ExtensionManager.reload` swaps in a freshly built extension: it makes a new
`URLClassLoader` over the extension directory, loads `ledger.LedgerExtension` through
it and calls `start()`. `start()` registers the extension's `Tally` listener on the
process-wide `EventBus` (`src/reports/EventBus.java:13`, the `LISTENERS` list on line 8).
`reload` drops its own references to the previous extension and loader but never calls
`stop()` on the previous instance, the only code that unsubscribes the listener (it is
called only by `shutdown`). The bus therefore keeps every old `Tally`, each `Tally`
keeps its class, the class keeps its defining class loader and the loader keeps the
extension's other class with its static 8 KB table. Per reload this retains
one loader, 2 loaded classes and about 13 KB of heap.

Background: a class can be unloaded only when its defining class loader can be
reclaimed (JLS 12.7,
https://docs.oracle.com/javase/specs/jls/se21/html/jls-12.html#jls-12.7); a live
reference from a long-lived object into any class of the loader prevents it.

Expected site `src/reports/EventBus.java:13`; alternatives: the list declaration at
line 8 and `ExtensionManager.reload` (`src/reports/ExtensionManager.java:26-31`).
Both fixtures of the pair share the same `task.prompt`.

## Measurements

OpenJDK 21. The extension is compiled separately into `out/ext`; the host loads it
from there.

```
cd project && ./build.sh run 2000      # builds host and extension, runs the test, 2000 reloads
java -cp out/host reports.Main out/ext 8000
```

The driver forces a GC and then prints listeners on the bus, loaded classes
(`ClassLoadingMXBean`) and heap in use:

| reloads | listeners | loaded classes | heap in use after GC |
|---|---|---|---|
| 500  | 501  | 1796  | 7786 KB |
| 1000 | 1001 | 2922  | 14585 KB |
| 1500 | 1501 | 3922  | 21157 KB |
| 2000 | 2001 | 4922  | 27708 KB |
| 8000 | 8001 | 16923 | 105651 KB |

About 13 KB of heap and 2 loaded classes per reload, linear in the iteration count;
2000 reloads take about 2 s and 20000 reloads about 17 s (the bus calls every
retained listener, so the run time grows faster than linearly).

## Minimal edit to the control

See `jvm-extension-reload-stopped/README.md`.
