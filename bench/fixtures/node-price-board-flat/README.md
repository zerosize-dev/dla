# node-price-board-flat

Category `decoy-control` (control half of a pair, partner `node-price-board`).
Cause `N-WARM`: the closure captures a number, so no snapshot reaches an older one
and heap stays at its starting level.

## Mechanism

Same code as `node-price-board`, except that the closure `changeNote` captures
`previousCount`, a number copied from the previous snapshot, and not the snapshot
itself. The shared context of `publish` then holds only the symbol, the ticks of
this update, the number and the snapshot, and the chain back to the first update is
gone. The two closures sharing a scope, the `latest` field and the
previous-snapshot logic look as suspicious as in the trap.

Non-leak site: `src/board.js:15`.

## Minimal edit from the trap

`diff -u` of `src/board.js`, run from `bench/fixtures`:

```
@@ -12,9 +12,9 @@
   // board's current snapshot. `options.log` receives a note about the change when
   // `options.verbose` is set.
   publish(symbol, ticks, options = {}) {
-    const previous = this.latest;
+    const previousCount = this.latest ? this.latest.count : null;
     const changeNote = () =>
-      previous ? `${symbol}: ${previous.count} -> ${ticks.length} ticks` : `${symbol}: first snapshot`;
+      previousCount !== null ? `${symbol}: ${previousCount} -> ${ticks.length} ticks` : `${symbol}: first snapshot`;
 
     const snapshot = {
       id: ++this.sequence,
```

## Measurements

Same harness as the partner (`node --expose-gc measure.js project N`):

| updates | heap after GC (KB) |
|---|---|
| 500   | 41 |
| 2000  | 41 |
| 8000  | 54 |
| 32000 | 57 |

The trap ends at 315797 KB for 32000 updates. The slope of the second half is zero.
Unit tests: `node --test` (5 pass).
