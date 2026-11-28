# rust-notice-outbox-linked

Category `decoy-control` (partner `rust-notice-outbox`). Cause `N-WARM`.

## Minimal edit

One line in `src/main.rs`: the uplink is attached before the first wave (`link.attach()`,
line 34). Everything else is identical, so is the task prompt.

## Mechanism

The same per-thread outbox and the same fixed-size link as the trap (`src/outbox.rs`,
`src/link.rs`). With the uplink attached, `Link::collect` empties the queue after every wave
(240 notices against 512 slots), so the link always has room, a worker's outbox never keeps a
notice, and its vector stays empty at 0 capacity. Memory is the thread stacks and the queue,
the same at every iteration count. The vector, its chunked growth and the thread-locals are
still in the code: a reader who only sees the container sees what the trap has.

Expected: no confirmed leak; the growth ruled out with its discriminator (live bytes flat
from the first wave).

## Measurements

```
cd project && cargo test --release     # 7 passed
cargo run --release -- --iterations 2000
# waves=2000 raised=480000 delivered=480000 waiting=0
```

With the allocator shim preloaded (record at exit):

| waves | live bytes | live blocks | peak bytes | RSS |
|---|---|---|---|---|
| 500  | 35 500 | 53 | 39 492 | 3 228 kB |
| 2000 | 35 500 | 53 | 39 492 | 3 196 kB |
| 8000 | 35 500 | 53 | 39 492 | 3 212 kB |

Flat: the slope of the second half is zero at every point.
