# Rust archetype rows

A row match opens a hypothesis and carries no verdict.

Read this only after `dla gate check --stage classify` passes. A row says where to look and what a
confirmed case looks like; the finding is confirmed by the ledger rules, then labelled with
`dla ledger label F --as <ID>`. A finding that fits no row stays UNCLASSIFIED. Runtime facts:
`references/rust.md` and `references/native.md`; commands: `references/backends.md`.

| Row | Find candidates | Confirm | Fingerprint | Rule out |
|---|---|---|---|---|
| L-CYCLE | `dla sweep --lang rust` rule `dla/rust-rc-strong-cycle`; `dla census --lang rust` (`Rc` or `Arc` fields that point both ways) | `dla harness gen --lang rust --driver`, `dla harness run H1`, `dla trend S1` on `native.live_blocks` and `native.live_bytes`; `dla harness run H1 --tool valgrind` then `dla summarize alloc-stack-diff` on the `valgrind.xml` files | A cycle of strong counts never reaches 0 and its values are never dropped [R1]. A block that no pointer reaches is "definitely lost" to memcheck, and a block that only lost blocks point at is "indirectly lost" [V1], so a cycle that no live variable points into shows no holder in the dump, the site comes from the allocation stack, and the holder you record is the kind `cycle-only` (`dla ledger annotate F --holder cycle-only --evidence E..`; `ledger label` accepts only that kind for this row); live blocks rise by a fixed amount per unit | `N-ALLOC`: a `Weak` back edge breaks the cycle [R1] and the resident set settles; `dla alloc-test H1 --finding F --purge trim` (purge and plateau tests). `N-WARM`: `dla alloc-test H1 --finding F` (plateau test) |
| L-OWNERSHIP | `dla sweep --lang rust` rules `dla/rust-into-raw-unpaired`, `dla/rust-box-leak`, `dla/rust-mem-forget`, `rapx/leak` | Driver harness `native.live_bytes` and `native.live_blocks@file:line` columns; `dla harness run H1 --tool valgrind` then `dla summarize alloc-stack-diff`; `dla intervene H1 --finding F --patch fix.diff` re-runs the harness with the release added | After `Box::into_raw` "the caller is responsible for the memory previously managed by the `Box`" [R2], and a raw pointer has no destructor. The registry entry is gone and no pointer reaches the block, which memcheck reports as "definitely lost" [V1], so memory grows with the units while the table stays empty. Record the holder kind `native-handle/global-ref` (`dla ledger annotate F --holder native-handle/global-ref --evidence E..`); `ledger label` accepts only that kind for this row | `N-ALLOC`: `dla alloc-test H1 --finding F --purge trim` (a drop of at least half the growth favors non-leak). `N-WARM`: the pointer is turned back with `Box::from_raw` [R2] and live bytes are flat; `dla alloc-test H1 --finding F` (plateau test) |
| UNCLASSIFIED | Any growth with a holder and no row above | State the holder and the mechanism, then `dla ledger label F --as UNCLASSIFIED --note "<why nothing fits>"` | The site is named by the allocation stack or `dla census --lang rust`, and the mechanism is not one of the rows above | The same tests as every row: `N-WARM` and `N-HWM` by `dla alloc-test H1 --finding F`, `N-ALLOC` by its `trim` purge |

## Sources

- [R1] https://doc.rust-lang.org/book/ch15-06-reference-cycles.html
- [R2] https://doc.rust-lang.org/std/boxed/struct.Box.html (`Box::into_raw`)
- [V1] https://valgrind.org/docs/manual/mc-manual.html (section 4.2.10, Memory leak detection)
