# Node archetype rows

A row match opens a hypothesis and carries no verdict.

Read this only after `dla gate check --stage classify` passes. A row says where to look and what a
confirmed case looks like; the finding is confirmed by the ledger rules, then labelled with
`dla ledger label F --as <ID>`. A finding that fits no row stays UNCLASSIFIED. Runtime facts:
`references/node.md`; commands: `references/backends.md`.

| Row | Find candidates | Confirm | Fingerprint | Rule out |
|---|---|---|---|---|
| L-REGISTER | `dla sweep --lang node` rules `dla/node-listener-no-removal`, `eslint/rxjs-x/no-ignored-subscription`; `dla census --lang node` | `dla harness gen --lang node`, `dla harness run H1 --capture nonheap`, `dla trend S1` on `node.listeners` and `node.timers`; `dla summarize non-heap` on `resources.json` (a `Timeout@file:line` group ties to the finding at that line) | The registration count rises by a fixed amount per unit, grouped by one `file:line`. Node warns "possible EventEmitter memory leak" (`MaxListenersExceededWarning`) past 10 listeners per event, which is a lead and not a limit [N1] | `N-WARM`: the listener is removed or the timer cleared when the unit ends [N1], so the count is flat; `dla alloc-test H1 --finding F` (plateau test on the count column). `N-SCALE`: one per live connection; `dla trend S1 --against S2 --finding F` |
| L-CAPTURE | `dla census --lang node` (function-context variables); a function that creates a closure over a value it also replaces on a long-lived variable | `dla harness run H1 --capture heap`, then `dla summarize heap-graph` on 3 or more snapshots; the holder kind is read from the retention root; `dla ledger annotate F --holder closure/continuation --evidence <growth evidence>` when the finding names another holder | A closure keeps a reference to its lexical environment [M1], so each object keeps the one before it and the path from the root runs through a function-context variable (holder `closure/continuation`), not a module variable or a listener entry | `N-WARM`: the closure captures a copy of a number or string instead of the object, and the heap is flat; `dla alloc-test H1 --finding F` (plateau test on `node.heap_used`). `N-HEADROOM`: `dla alloc-test H1 --purge gc` |
| UNCLASSIFIED | Any growth with a holder and no row above | State the holder and the mechanism, then `dla ledger label F --as UNCLASSIFIED --note "<why nothing fits>"` | The holder is named by `dla census --lang node` or `dla summarize heap-graph`, and the mechanism is not one of the rows above | The same tests as every row: `N-WARM` and `N-HWM` by `dla alloc-test H1 --finding F`, `N-HEADROOM` by its gc purge |

## Sources

- [N1] https://nodejs.org/api/events.html
- [M1] https://developer.mozilla.org/en-US/docs/Web/JavaScript/Closures
