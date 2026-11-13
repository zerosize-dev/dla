# rust-ledger-sync

Category `multi-leak`, no partner. Two independent leaks in two modules, both with a
shipped Rust row:

- L1 `L-CYCLE` (about 67% of the growth), holder `cycle-only`, region `heap`, in `journal.rs`.
- L2 `L-OWNERSHIP` (about 33%), holder `native-handle/global-ref`, region `off-heap`, in `outbox.rs`.

## Mechanism

**L1, `src/journal.rs`.** The bank's feed is turned into a `Batch` (shared through `Rc`)
whose `Posting`s are kept in `Batch.postings` (`:8`). Each `Posting` names its batch for
reconciliation with `batch: Rc<Batch>` (`:15`, set at `:25`), so the batch holds its
postings and each posting holds the batch with strong counts. After `Sync::run_once`
returns, the local handles are dropped, but the counts never reach zero and the batch and
its four to seven postings stay allocated. The cycle is reachable only from itself
(`cycle-only`). The design differs from the `rust-tree-parent-links` fixture: the
objects are a flat batch and its members, not a tree, and the back edge is on a member
rather than on a node of a tree. Alternative sites: the push of the posting into the
batch (`:26`) and the two field declarations (`:8`, `:15`).

**L2, `src/outbox.rs`.** `Outbox::queue` renders each statement line and hands it to the writer as a
raw node: `Box::into_raw(Box::new(statement.open()))` (`:44`), kept in `pending` (`:34`, pushed at
`:45`). `flush` reads the nodes through `&*node`, writes the text and drains `pending`, which
drops the raw pointers; nothing ever turns a node back into a `Box` with `Box::from_raw`, so the
node and its text (about 160 bytes per batch) are never freed. `pending` is empty after every
flush (`waiting()` is 0 at the checkpoints), so the growth is not in the queue. The design
differs from `rust-meter-callbacks`: the raw pointer is a node of a write spool, not the user
pointer of a callback table.

The two leaks are independent: removing either one leaves the other's slope unchanged
(measured below). The ledger's `HashMap` holds a fixed pool of 2000 accounts and its size
is flat.

Expected: both leaks confirmed, L1 at `src/journal.rs:25` with holder `cycle-only`, L2 at
`src/outbox.rs:44` with holder `native-handle/global-ref`.

## Measurements

Toolchain: cargo 1.99.1. Checks and workload (`--iterations N` runs N waves of 100 batches):

```
cd project && cargo test --release     # 9 passed
cargo run --release -- --iterations 3200
# batches=320000 accounts=2000 net=26442993657 last_flush=0 bytes_written=34879730  (0.31 s)
```

External harness (kept outside the repository): a copy of `project/` with a counting
`#[global_allocator]` (live bytes, live blocks) and a read of `VmRSS`, printed at the end
of `main` (`ledger-sync --iterations N`). Two more copies isolate each leak: "no L2" frees the
node right after it is built (`Box::from_raw`) instead of queueing it, "no L1" gives each
posting a fresh empty batch in place of a reference to its own, which breaks the cycle.

| waves (batches) | live bytes | live blocks | RSS | live bytes, no L2 (= L1) | live bytes, no L1 (= L2) |
|---|---|---|---|---|---|
| 200 (20 000)   | 9 992 774   | 189 887   | 13 748 kB  | | |
| 800 (80 000)   | 39 775 566  | 760 304   | 48 460 kB  | 26 648 334  | 13 198 470 |
| 3200 (320 000) | 158 813 622 | 3 040 297 | 187 204 kB | 106 326 390 | 52 558 470 |

Slopes (3200 against 800 waves): total 496 live bytes and 9.5 blocks per batch; L1 about 332
bytes and 7.5 blocks per batch (67% of the growth); L2 about 164 bytes and 2 blocks per batch
(33%). Both are linear in the iteration count (4x the batches gives 4x the live bytes), and the
two variants add up to the unmodified program (106.3 MB plus 52.6 MB is 158.9 MB against
158.8 MB).
