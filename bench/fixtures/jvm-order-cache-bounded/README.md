# jvm-order-cache-bounded

Category `decoy-control` (partner `jvm-order-cache`). Cause `N-WARM`: the cache
fills to its capacity and stops.

## Mechanism

Same code as `jvm-order-cache`, except that `OrderCache` holds its orders in an
access-ordered `LinkedHashMap` that drops its eldest entry once it has more than
1000. Heap in use rises while the cache fills (about the first 1250 requests)
and is flat afterwards. A run that reports a confirmed growth here is a false
alarm. Both fixtures share the same `task.prompt`.

Non-leak site: `src/shop/OrderCache.java:10` (the map declaration).

## Minimal edit from the trap

`diff -ru` of the two `project/` directories (copied outside the repository so
the paths are neutral). Output is 24 lines, 10 of them added or removed lines
(`trap` is `jvm-order-cache`, `control` is this fixture):

```
diff -ru trap/src/shop/OrderCache.java control/src/shop/OrderCache.java
--- trap/src/shop/OrderCache.java
+++ control/src/shop/OrderCache.java
@@ -1,11 +1,18 @@
 package shop;
 
-import java.util.HashMap;
+import java.util.LinkedHashMap;
 import java.util.Map;
 
 /** Remembers confirmed orders so a client that repeats a request gets the same answer. */
 final class OrderCache {
-    private static final Map<String, Order> ORDERS_BY_ID = new HashMap<>();
+    private static final int CAPACITY = 1000;
+
+    private static final Map<String, Order> ORDERS_BY_ID = new LinkedHashMap<>(256, 0.75f, true) {
+        @Override
+        protected boolean removeEldestEntry(Map.Entry<String, Order> eldest) {
+            return size() > CAPACITY;
+        }
+    };
 
     private OrderCache() {}
 
```

## Measurements

```
cd project && ./build.sh run 8000
```

| requests | cached orders | heap in use after GC |
|---|---|---|
| 2000  | 1000 | 2021 KB |
| 8000  | 1000 | 2021 KB |
| 32000 | 1000 | 2021 KB |

The same three runs of the trap give 2444, 5424 and 17217 KB. Class histogram of
a 20 000 000-request run (`-Xmx3g`) sampled three times with
`jcmd <pid> GC.class_histogram`: `shop.Order` is 1000 instances every time
(48000 bytes), against 866649 to 2625000 in the trap after 3 000 000 requests.
Heap growth after the cache is full is zero; the slope of the second half of the
run is zero.
