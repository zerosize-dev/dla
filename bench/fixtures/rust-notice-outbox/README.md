# rust-notice-outbox

Category `archetype-free` (trap half of a pair, partner `rust-notice-outbox-linked`).
Cause `UNCLASSIFIED`, holder `container-field` (a field of a value kept in a `thread_local!`), region
`heap`.

## Mechanism

Four long-lived worker threads send alerts (`Notice`, 48 bytes) through a per-thread outbox
(`thread_local!`, `src/outbox.rs:17`). The outbox offers each notice to a hand-off of fixed
size, the `Link` (512 slots, `src/link.rs`), and keeps the notice in its own `Vec` when the
link has no room (`src/outbox.rs:41-44`: the vector grows by 4096 notices at a time, then the
notice is pushed on the same line, `:44`). The link is emptied by `Link::collect`, which moves nothing until the uplink
is attached (`Link::attach`, `src/link.rs:32`), and the workload never attaches it. After the
first 512 notices the link is full for good, so every later notice of every worker goes to
its thread's vector, and nothing ever takes it out: the outbox only gives notices back to a
link that has room. One vector per worker thread, one block each, grows by a chunk of 4096
notices (196 KB) at a time, linear in the number of waves.

The state is reachable only through a thread-local of a thread that lives as long as the
process, so it is invisible to the project's own counters (`waiting()` stays at 512 and
`delivered` at 0) and to a reachability walk from statics. No registration table, cycle,
capture or detached thread per unit is involved, and none of the shipped Rust rows fits (the
thread-local row needs a retention path, which a Rust process does not have), so the expected
label is `UNCLASSIFIED`.

Why it needs the allocation capture: the program ends with `std::process::exit(0)` while the
workers are parked in `recv`, so `exit()` runs the thread-local destructors of the main thread
only and the workers' outboxes are still live when the allocator shim writes its exit record. A
driver that runs the same code on the thread that ends the process would lose it at exit; the
drivers `dla harness gen` writes hand the shim a snapshot after their last unit for that case.

Expected: the leak confirmed at `src/outbox.rs:44` (the push and the growth of the vector) or
its declaration (`:12`, `:17`), holder `container-field`, labelled `UNCLASSIFIED`.

## Measurements

Toolchain: cargo with the offline std toolchain, no dependencies. Checks and workload
(`--iterations N` runs N waves of 240 notices):

```
cd project && cargo test --release     # 7 passed
cargo run --release -- --iterations 2000
# waves=2000 raised=480000 delivered=0 waiting=512
```

The same binary started with the allocator shim preloaded (`LD_PRELOAD`, record at exit):

| waves | live bytes | live blocks | peak bytes | RSS |
|---|---|---|---|---|
| 500  | 6 326 956  | 57 | 6 330 948  | 9 352 kB |
| 2000 | 25 201 324 | 57 | 25 205 316 | 26 196 kB |
| 8000 | 100 698 796 | 57 | 100 702 788 | 93 724 kB |

Slope: about 52 live bytes per notice (a vector of 48-byte notices that grows in chunks of
4096), linear in the iteration count, with a constant 57 blocks: the growth is four buffers that
are resized, not new blocks. The stack of the growing blocks ends at `src/outbox.rs:44`.
