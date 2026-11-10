# Python archetype rows

A row match opens a hypothesis and carries no verdict.

Read this only after `dla gate check --stage classify` passes. A row says where to look and what a
confirmed case looks like; the finding is confirmed by the ledger rules, then labelled with
`dla ledger label F --as <ID>`. A finding that fits no row stays UNCLASSIFIED. Runtime facts:
`references/python.md`; commands: `references/backends.md`.

| Row | Find candidates | Confirm | Fingerprint | Rule out |
|---|---|---|---|---|
| L-COLLECT | `dla sweep --lang python` rules `ruff/B019`, `pylint/W1518`, `dla/python-cache-method`, `dla/python-unbounded-cache`, `dla/python-global-container-growth`; `dla census --lang python` | `dla harness gen --lang python`, `dla harness run H1 --capture alloc`, `dla trend S1` on `py.traced`, then `dla summarize alloc-stack-diff` on `py-*.alloc.json` (ties by the first project frame); the probe columns of module-level containers and `lru_cache` wrappers (`currsize`) | `lru_cache(maxsize=None)` "can grow without bound", and a cached method includes `self` in the cache and keeps references to its arguments [P1], so the cache holds every instance and key it has seen. Growth with fresh keys that does not level off | `N-WARM`: a cache with an integer `maxsize` evicts [P1], so `currsize` stops at the bound; `dla alloc-test H1 --finding F` (plateau test). `N-HEADROOM`: `py.traced.pre_gc` rises while `py.traced` is flat; `dla alloc-test H1 --purge gc` |
| UNCLASSIFIED | Any growth with a holder and no row above | State the holder and the mechanism, then `dla ledger label F --as UNCLASSIFIED --note "<why nothing fits>"` | The holder is named by `dla census --lang python` or the harness probes, and the mechanism is not the row above | The same tests as every row: `N-WARM` and `N-HWM` by `dla alloc-test H1 --finding F`, `N-ALLOC` by its `trim` purge |

## Sources

- [P1] https://docs.python.org/3/library/functools.html
