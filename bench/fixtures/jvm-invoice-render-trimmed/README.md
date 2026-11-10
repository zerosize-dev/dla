# jvm-invoice-render-trimmed

Category `decoy-control` (control half of a pair, partner `jvm-invoice-render`).
Cause `N-HWM`: each worker thread's buffer is cut back after every invoice, so
it holds at most the segments of the largest single invoice.

## Mechanism

Same code as `jvm-invoice-render`, except that `InvoiceRenderer.render` calls
`RenderContext.truncate(mark)` after it has read its text, which removes the
segments the render appended. The `ThreadLocal<List<String>>` on the long-lived
pooled worker threads (a fixed pool of 4, alive at every checkpoint and at the dumps)
looks as suspicious as in the trap, and each list keeps its backing array
(a few dozen slots at most) for the life of the thread, which is a high-water
mark and not growth. A run that reports a confirmed growth here is a false alarm.

Non-leak site: `src/billing/RenderContext.java:13` (the append).

## Minimal edit from the trap

`diff -u` of the two projects, run from `bench/fixtures` (`trap` is
`jvm-invoice-render`):

```
--- RenderContext.java
@@ -22,4 +22,9 @@
         List<String> all = SEGMENTS.get();
         return String.join("\n", all.subList(mark, all.size()));
     }
+
+    static void truncate(int mark) {
+        List<String> all = SEGMENTS.get();
+        all.subList(mark, all.size()).clear();
+    }
 }

--- InvoiceRenderer.java
@@ -16,7 +16,9 @@
         RenderContext.append("Subtotal: " + money(invoice.subtotalCents()));
         RenderContext.append("Tax: " + money(tax));
         RenderContext.append("Total: " + money(invoice.subtotalCents() + tax));
-        return RenderContext.textFrom(mark);
+        String text = RenderContext.textFrom(mark);
+        RenderContext.truncate(mark);
+        return text;
     }
 
     private static String formatLine(InvoiceLine line) {
```

## Measurements

```
cd project && ./build.sh run 8000
```

| invoices | heap in use after GC |
|---|---|
| 600   | 1692 KB |
| 2000  | 1687 KB |
| 8000  | 1692 KB |
| 32000 | 1697 KB |

The trap gives 2435, 4111, 11615 and 40872 KB. The control is flat from the first
checkpoint; the slope of the second half is zero.
