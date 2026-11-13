# rust-delayed-queue

Category `archetype-free`, no partner. Cause `UNCLASSIFIED`, holder `container-field`,
region `heap`.

## Mechanism

`Scheduler` keeps its tasks in a `BinaryHeap` ordered by due time (`src/scheduler.rs:41`, entries
pushed at `:53`).
`cancel` does not remove an entry: it sets a shared flag on it and lowers the `armed`
count (`src/scheduler.rs:58-64`), and `advance` drops a cancelled entry only when it
reaches the top of the heap, that is when its due time has passed (`:70-82`). `Orders::place`
arms a thirty-day payment-window task for every order (`src/orders.rs:28`) and the
payment cancels it within a second. The cancelled entry, with its boxed `Task` (three
strings) and its flag, stays in the heap for thirty days. The heap's size follows the
number of orders taken, not the number of timers armed (`timers_armed()` and the
number of orders waiting both stay in the dozens).

The entry is reachable (it sits in the heap), and the program does cancel its timers
correctly: `armed` is right. What stays is the entry, not a timer. No registration table,
thread, timer callback, capture or cycle is involved. The mechanism is lazy
deletion with a horizon far beyond the life of the process: at the measured 225 bytes per
order, the thirty-day horizon of a service taking 140 orders a second is about 360 million
entries (80 GB), so within any restart interval the growth does not level off. In the fixture's
workload the clock advances 7 ms per order, so the horizon is about 370 million orders;
no measured point comes near it. None of the shipped rows of `rust-archetypes.md` fits
(`L-STALE` has a JVM reference only), so the expected label is `UNCLASSIFIED`.

Expected: the leak confirmed at `src/scheduler.rs:53` (the push of the entry) or in
`cancel` (`:58-64`), holder `container-field`, labelled `UNCLASSIFIED`.

## Measurements

Toolchain: cargo 1.99.1. Checks and workload (`--iterations N` runs N waves of 200 orders):

```
cd project && cargo test --release     # 7 passed
cargo run --release -- --iterations 8000
# orders=1600000 paid=1599969 reminded=852807 cancelled=0 awaiting=31 timers=59  (1.7 s)
```

External harness (kept outside the repository): a copy of `project/` with a counting
`#[global_allocator]` (live bytes, live blocks) and a read of `VmRSS`, printed just before the
process exits (`order-timers --iterations N`; the program ends with `process::exit`, so the
scheduler is still alive when the numbers are read):

| waves (orders) | timers armed | live bytes | live blocks | RSS |
|---|---|---|---|---|
| 500 (100 000)    | 59 | 22 563 850  | 500 265   | 31 836 kB |
| 2000 (400 000)   | 61 | 90 198 772  | 2 000 270 | 120 892 kB |
| 8000 (1 600 000) | 59 | 360 729 634 | 8 000 262 | 477 168 kB |

Slope: about 225 live bytes and 5 blocks per order, linear in the iteration count (4x the
waves gives 4x the live bytes). `timers armed` and `awaiting` stay at about 60 and 30 at
every point, so the growth is not in the live timers or the open orders.
