# jvm-invoice-render

Category `archetype-leak` (trap half of a pair, partner `jvm-invoice-render-trimmed`).
Cause `L-TLS`, holder `thread-local`, region `heap`.

## Mechanism

`InvoiceRenderer` builds each invoice as text segments appended to a per-thread
buffer, `RenderContext.SEGMENTS`, a `ThreadLocal<List<String>>`
(`src/billing/RenderContext.java:8`; the append is line 13). A render takes a mark
(the list length when it starts) and reads back only the segments after the mark,
so the output of one invoice never contains another. Nothing ever shortens the
list again. `InvoiceService` renders on a fixed pool of 4 worker threads that
lives as long as the service (daemon threads, never shut down; they are alive at
every checkpoint and when the exit-time dump is written). Each worker's list keeps
every segment of every invoice that worker has rendered (about 10 strings, 1.2 KB,
per invoice). The values are reachable through `Thread.threadLocals` of the
long-lived pooled threads.

Background: "Each thread holds an implicit reference to its copy of a thread-local
variable as long as the thread is alive and the `ThreadLocal` instance is
accessible" (ThreadLocal javadoc,
https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/lang/ThreadLocal.html);
Tomcat's `clearReferencesThreadLocals` and `renewThreadsWhenStoppingContext`
settings exist because pooled threads keep such values
(https://tomcat.apache.org/tomcat-10.1-doc/config/context.html).

Expected site `src/billing/RenderContext.java:13`; alternative: the declaration
at line 8. Both fixtures of the pair share the same `task.prompt`.

## Measurements

OpenJDK 21. Build, test and run:

```
cd project && ./build.sh run 8000      # compiles, runs unit tests, renders 8000 invoices
```

The driver forces a GC at each checkpoint and prints the heap in use (last line
of each run, `java -cp out billing.Main N`):

| invoices | heap in use after GC |
|---|---|
| 600   | 2435 KB |
| 2000  | 4111 KB |
| 8000  | 11615 KB |
| 32000 | 40872 KB |

Slope about 1.2 KB per invoice, linear in the iteration count (2000 to 32000:
36761 KB over 30000 invoices); no plateau. The 32000-invoice run takes a few
seconds.

## Dynamic result

`dla-bench dynamic --only jvm-invoice-render` (pinned binaries): the trap's leak
is confirmed and the decoy is not.

- The in-process probe of the static field `RenderContext.SEGMENTS` measures the
  `ThreadLocal` through its values: the sum over the 4 live worker threads of the
  list each holds in `threadLocals` (about 12.5 segments, 1.2 KB, per invoice). That
  column passes the trend gate, the alloc-test favors leak, the N-WARM sibling is
  refuted and the finding is confirmed.
- `dla summarize heap-graph` over the driver's 4 heap dumps reports the holder
  `thread-local:billing.RenderContext.SEGMENTS` (holder kind `thread-local`, owner the
  static field that holds the `ThreadLocal`), retained bytes 1393848, 5561736,
  8321800 and 12542416, gate pass. It ties to the census finding by holder identity
  and the confirmation cites it, so the leak is dump backed. The pooled threads are
  also reached through their executor, so no thread dominates its own values; the
  reader finds them through each thread's `threadLocals` map.
- The decoy (`jvm-invoice-render-trimmed`): the probe stays flat, the heap-graph gate
  fails and nothing is confirmed.

## Minimal edit to the control

See `jvm-invoice-render-trimmed/README.md`.
