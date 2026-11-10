# JVM archetype rows

A row match opens a hypothesis and carries no verdict.

Read this only after `dla gate check --stage classify` passes. A row says where to look and what a
confirmed case looks like; the finding is confirmed by the ledger rules, then labelled with
`dla ledger label F --as <ID>`. A finding that fits no row stays UNCLASSIFIED. Runtime facts:
`references/jvm.md`; commands: `references/backends.md`.

| Row | Find candidates | Confirm | Fingerprint | Rule out |
|---|---|---|---|---|
| L-COLLECT | `dla census --lang jvm` (static or field `Map`, `List`, `Set`); `dla sweep --lang jvm` rules `dla/jvm-unbounded-collection` and `dla/jvm-micrometer-request-tag` | `dla harness gen --lang jvm --driver`, `dla harness run H1 --mode both --capture heap`, `dla trend S1`, then `dla summarize heap-graph` on the dumps; `dla ledger confirm F --evidence ...` | The `holder:` column ties to a `static/global` or `container-field` holder, and the probe column of that field rises with the units. Growth with fresh keys that does not level off, and none with repeated keys, points at keys from request data | `N-WARM`: `dla alloc-test H1 --finding F` (plateau test) when the collection stops growing, for example an access-ordered `LinkedHashMap` that evicts through `removeEldestEntry` [J3]; `dla trend S1 --against S2 --finding F` (growth in neither key mode) |
| L-TLS | `dla census --lang jvm` (`ThreadLocal` fields); `dla sweep --lang jvm` rules `errorprone/ThreadLocalUsage`, `spotbugs/SIC_THREADLOCAL_DEADLY_EMBRACE`, `sonar/S5164` (Sonar rows are unverified in `docs/VERIFY.md`) | Driver harness with `--capture heap`; `dla summarize heap-graph` on the dumps, whose retention path starts at a thread and not at a static field; `jvm.threads` stays flat while `jvm.heap_used` rises | A thread holds its copy of a thread-local "as long as the thread is alive and the `ThreadLocal` instance is accessible" [J1], so the growth sits in the values of a fixed set of long-lived pooled threads; the census names the holder kind `thread-local` | `N-HWM`: the value is cut back after each unit (`remove()` or truncation [J1]) and holds only the largest single unit; `dla alloc-test H1 --finding F` (plateau test, supports `N-HWM`) |
| L-UNLOAD | A workload that reloads or redeploys a module (`dla workload discover`); `dla census --lang jvm` for static registries the module writes to | `dla harness run H1 --capture nonheap`, `dla summarize non-heap` on `classloader-stats.txt`; `jvm.classes_loaded` rises with the reload count; `dla summarize heap-graph` for the path from the long-lived holder | A class can be unloaded if and only if its defining class loader can be reclaimed, and a loader that can be reclaimed has no live reference to it or to any class it defined, from instances or from code [J2]. So a registry, static or thread-local that still references the loader, one of its classes or an instance of one of its classes stops the unload. Loaded classes and loaders rise by a fixed amount per reload | `N-WARM`: loaded classes rise for the first reloads and then stay flat once old loaders are collected; `dla alloc-test H1 --finding F --column jvm.classes_loaded` (plateau test) |
| UNCLASSIFIED | Any growth with a holder and no row above | State the holder and the mechanism, then `dla ledger label F --as UNCLASSIFIED --note "<why nothing fits>"` | The holder is named by `dla census --lang jvm` or `dla summarize heap-graph`, and the mechanism is not one of the rows above | The same tests as every row: `N-WARM` and `N-HWM` by `dla alloc-test H1 --finding F`, `N-HEADROOM` by its gc purge |

## Sources

- [J1] https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/lang/ThreadLocal.html
- [J2] https://docs.oracle.com/javase/specs/jls/se21/html/jls-12.html (section 12.7)
- [J3] https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/util/LinkedHashMap.html
