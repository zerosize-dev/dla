# jvm-extension-reload-stopped

Category `decoy-control` (control half of a pair, partner `jvm-extension-reload`).
Cause `N-WARM`: old extension instances are stopped on reload, their class loaders
become unreachable and are collected with their classes.

## Mechanism

Same code as `jvm-extension-reload`, except that `ExtensionManager.reload` stops the
running instance (which unsubscribes its listener from the `EventBus`) before it drops
it. After a reload the bus holds one listener. A new class loader per reload, a
static registry on the bus and a listener that comes from the plugin all look as
suspicious as in the trap. Loaded classes rise during the first reloads until the
first collections and then stay flat while old loaders are unloaded.

Non-leak site: `src/reports/EventBus.java:13` (the registration).

## Minimal edit from the trap

`diff -u` of `src/reports/ExtensionManager.java`, run from `bench/fixtures`:

```
@@ -25,6 +25,9 @@
 
     /** Picks up a rebuilt extension without restarting the process. */
     public void reload() throws Exception {
+        if (current != null) {
+            current.stop();
+        }
         loader = null;
         current = null;
         load();
```

## Measurements

```
cd project && ./build.sh run 2000
java -cp out/host reports.Main out/ext 8000
```

| reloads | listeners | loaded classes | heap in use after GC |
|---|---|---|---|
| 500  | 1 | 796 | 1266 KB |
| 1000 | 1 | 920 | 1508 KB |
| 1500 | 1 | 922 | 1530 KB |
| 2000 | 1 | 922 | 1517 KB |

The 8000-reload run ends at 927 loaded classes and 1647 KB. The trap ends the same
run at 16923 classes and 105651 KB. Slope of the second half: zero.
