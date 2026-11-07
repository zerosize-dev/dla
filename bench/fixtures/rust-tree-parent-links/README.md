# rust-tree-parent-links

Category `archetype-leak` (partner `rust-tree-parent-links-weak`). Cause `L-CYCLE`,
holder `cycle-only`, region `heap`.

## Mechanism

The outline service parses every submitted document into a tree of `Section`
values shared through `Rc`. `Section::append` links the two directions of the
tree with strong references: the parent keeps the child in `children`, and the
child keeps the parent in `parent`
(`*child.parent.borrow_mut() = Some(Rc::clone(parent))`, `src/section.rs:27`).
When `Service::handle` returns, the local `Rc` handles are dropped, but every
node is still referenced by its neighbour, so no reference count reaches zero and
the whole tree stays allocated. Rust has no tracing collector to find the cycle.
One tree per request is retained, so memory grows linearly with the number of
documents. The sections are only reachable from each other (`cycle-only`).

Expected: the leak confirmed at `src/section.rs:27`; the field declaration
(`:10`) and the `Rc::new` allocation (`:16-22`) are accepted as alternative sites.
Both fixtures in the pair share the same `task.prompt`.

## Measurements

Toolchain: cargo 1.97.0. Checks and workload (`cargo` run with a `CARGO_TARGET_DIR`
outside the repository):

```
cd project && cargo test --release     # 10 passed
cargo run --release -- --iterations 2000
# documents=2000 rejected=0 headings=78930 body_lines=78922 toc_bytes=4629988 deepest=4  (0.09 s)
cargo run --release -- --iterations 8000   # 0.31 s
```

External harness (kept outside the repository): a copy of `project/` with a
counting `#[global_allocator]` (live bytes and live blocks) and a read of `VmRSS`
from `/proc/self/status`, printed at checkpoints of one run:

| documents processed | live bytes | live blocks | RSS |
|---|---|---|---|
| 0     | 548        | 2       | 2.1 MB |
| 500   | 2 625 244  | 46 144  | 5.5 MB |
| 1000  | 5 256 980  | 92 296  | 8.8 MB |
| 1500  | 7 879 779  | 138 366 | 11.9 MB |
| 2000 (end of the 2000-document run) | 10 513 872 | 184 554 | 15.2 MB |
| 2000 (checkpoint of the 8000-document run) | 10 513 882 | 184 555 | 15.1 MB |
| 4000  | 21 029 963 | 369 188 | 27.9 MB |
| 6000  | 31 554 448 | 553 983 | 40.7 MB |
| 8000 (end of the 8000-document run) | 42 077 016 | 738 740 | 53.6 MB |

Slope: 5 260 live bytes and about 92 blocks per document (linear: 10.5 MB at 2000
documents, 42.1 MB at 8000, a factor of 4.0 for a factor of 4 in iterations);
RSS rises about 6.4 KB per document. Nothing is freed between requests.

## Minimal edit to the control

See `rust-tree-parent-links-weak/README.md`: the parent link becomes a `Weak`
reference (5 changed lines in `src/section.rs`); with it the same workload ends
at 548 bytes and 2 blocks live.
