# jvm-invoice-render

Category `archetype-leak` (trap half of a pair, partner `jvm-invoice-render-trimmed`).
Cause `L-TLS`, holder `thread-local`, region `heap`.

## Mechanism

`InvoiceRenderer` builds each invoice as text segments appended to a per-thread
buffer, `RenderContext.SEGMENTS`, a `ThreadLocal<List<String>>`
(`src/billing/RenderContext.java:8`; the append is line 13). A render takes a mark
(the list length when it starts) and reads back only the segments after the mark,
so the output of one invoice never contains another. Nothing ever shortens the
list again. The request loop (`Main`, the process's main thread, which stays blocked in
`System.exit` while the JVM shuts down) calls the service for every invoice, so the
thread's list keeps every segment of every invoice it has rendered (about 10
strings, 1.2 KB, per invoice). The values are reachable through
`Thread.threadLocals` of the long-lived thread. Pooled worker threads would behave
the same; the fixture renders on the calling thread because a heap-graph walk
attributes the contents of a pooled thread's map to no holder (see Dynamic result).

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
| 600   | 2406 KB |
| 2000  | 4096 KB |
| 8000  | 11405 KB |
| 32000 | 43073 KB |

Slope about 1.3 KB per invoice, linear in the iteration count (2000 to 32000:
38977 KB over 30000 invoices); no plateau. The 32000-invoice run takes a few
seconds.

## Dynamic result

`dla-bench dynamic --only jvm-invoice-render` (pinned binaries): the decoy passes;
the trap is not confirmed. The harness runs and the heap-graph summary does find the
leak: it adds a finding with holder `thread-local`, owner `thread:main`, and a
discriminator that favors leak (retained bytes of `thread:main` 1389240, 5571024,
8375008, 12556112 over 4 dumps, gate pass). The in-process probes cannot see it: the
census finds `RenderContext.SEGMENTS` and its probe measures the `ThreadLocal`
object, which is flat, so the policy refutes that finding (`favors non-leak`). The
finding that the heap-graph step adds is left `unresolved` by the policy, which adds
the refuted sibling and the test only for findings that a passing growth column ties
before the summarize step.

## Minimal edit to the control

See `jvm-invoice-render-trimmed/README.md`.
