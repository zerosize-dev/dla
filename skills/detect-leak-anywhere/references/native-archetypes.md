# Native archetype rows

A row match opens a hypothesis and carries no verdict.

Read this only after `dla gate check --stage classify` passes. A row says where to look and what a
confirmed case looks like; the finding is confirmed by the ledger rules, then labelled with
`dla ledger label F --as <ID>`. A finding that fits no row stays UNCLASSIFIED. Runtime facts:
`references/native.md`; commands: `references/backends.md`.

| Row | Find candidates | Confirm | Fingerprint | Rule out |
|---|---|---|---|---|
| L-HANDLE | `dla sweep --lang native` rules `dla/native-fopen-no-fclose` and `dla/native-malloc-leak-path`; `dla census --lang native`; an early return after an allocation or an open | `dla harness gen --lang native`, `dla harness run H1`, `dla trend S1` on `native.live_bytes` and `os.fds`; `dla harness run H1 --tool valgrind` then `dla summarize alloc-stack-diff` on the `valgrind.xml` files | Definitely lost blocks whose allocation site (first project frame) is the same at every point, and a lost-block count that matches the count of units that took the early-return path. A definitely lost block has no holder to find, so the site is the attribution and the holder you record is the kind `native-handle/global-ref` (`dla ledger annotate F --holder native-handle/global-ref --evidence E..`); `ledger confirm` needs a holder and `ledger label` accepts only that kind for this row | `N-ALLOC`: `free` returns a block to the allocator, which may keep the pages, and `malloc_trim` "attempts to release free memory from the heap" [M2]; `dla alloc-test H1 --finding F --purge trim`. `N-WARM`: the resident set levels off; `dla alloc-test H1 --finding F` (plateau test) |
| UNCLASSIFIED | Any growth with a holder and no row above | State the holder and the mechanism, then `dla ledger label F --as UNCLASSIFIED --note "<why nothing fits>"` | The site is named by the allocation stack or `dla census --lang native`, and the mechanism is not the row above | The same tests as every row: `N-WARM` and `N-HWM` by `dla alloc-test H1 --finding F`, `N-ALLOC` by its `trim` purge |

## Sources

- [M2] https://man7.org/linux/man-pages/man3/malloc_trim.3.html
