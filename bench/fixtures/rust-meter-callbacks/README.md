# rust-meter-callbacks

Category `archetype-leak` (trap half of a pair, partner `rust-meter-callbacks-reclaimed`).
Cause `L-OWNERSHIP`, holder `native-handle/global-ref`, region `off-heap`.

## Mechanism

`Session::open` boxes a `Window` (a 512-slot sample buffer, about 4 KB) and hands
the raw pointer to the driver callback table as the callback's opaque user pointer
(`Box::into_raw(...)`, `src/session.rs:19`). `Registry::remove` drops the table
entry and returns the pointer. `Session::close` ignores the returned pointer
(`src/session.rs:33`), and nothing else ever converts it back with `Box::from_raw`,
so neither side owns the window any more: the registry forgot it and the session
only keeps a raw pointer, which has no destructor. One window (3 allocations,
about 4.1 KB) is stranded per session, so memory grows linearly with the number
of sessions served. The window is not reachable from any root once the entry is
removed; it is invisible to a reachability walk and visible only as allocated
bytes that are never returned.

Documentation for the mechanism: `Box::into_raw` consumes the box and "the caller
is responsible for the memory previously managed by the `Box`"; the easiest way to
release it is to convert the pointer back with `Box::from_raw`
(https://doc.rust-lang.org/std/boxed/struct.Box.html#method.into_raw).

Expected site `src/session.rs:19` (the `Box::into_raw`); alternative `src/session.rs:33`
(the ignored return of `remove`). Both fixtures in the pair share the same
`task.prompt`.

## Measurements

Toolchain: cargo with the offline std toolchain, no dependencies. Checks and workload:

```
cd project && cargo test --release              # 5 passed
cargo run --release -- --iterations 2000
# sessions=2000 readings=80000 active=0
```

A copy of `project/` outside the repository with a counting `#[global_allocator]`
(live bytes, live blocks) and a read of `VmRSS` at the end of each run
(`meter-gateway --iterations N`):

| sessions | active entries | live bytes | live blocks | RSS |
|---|---|---|---|---|
| 2000   | 0 | 8 332 747   | 6007   | 10 320 kB |
| 8000   | 0 | 33 328 747  | 24007  | 34 980 kB |
| 32000  | 0 | 133 312 748 | 96007  | 133 552 kB |
| 128000 | 0 | 533 248 748 | 384007 | 528 104 kB |

Slope: 4166 live bytes and 3 blocks per session, linear in the iteration count
(factor 4 in sessions gives factor 4 in live bytes). The registry is empty after
every session (`active=0`), so the growth is not in the table.
The 128000-session run takes about 0.2 s.
