# rust-slot-arena

Category `archetype-free`, no partner. Cause `UNCLASSIFIED`, holder `container-field`,
region `heap`.

## Mechanism

`Arena<T>` stores values in fixed pages of 256 `Slot`s addressed by a handle of
(index, generation). `Arena::remove` drops the value (the boxed `Hold` and its two
strings are freed) and bumps the slot's generation so old handles stop resolving
(`src/arena.rs:84-93`). The index is never put on the `free` list that `claim`
pops from (`src/arena.rs:41`), so a removed slot is never handed out again. Each
`insert` therefore takes a fresh slot, and `claim` adds a new page of slots
whenever the last one is full (`src/arena.rs:46`). The payload is freed, but the
slots that held it stay allocated and their number follows the total number of
holds ever placed. The slot is inline in the page (generation, a 32-byte label and
the box pointer, 48 bytes), so what stays is one slot per hold; the number of live
holds stays at 300 throughout.

No collection of live objects grows (the desk and the arena both report 300 live
holds), no registration table, thread, timer or capture is involved, and the memory
is not unreachable (the pages are reachable from `Arena.pages`), so none of the
shipped rows of `rust-archetypes.md` applies. The mechanism is an allocator-style
container that never reuses its own freed positions.

Expected: the leak confirmed at `src/arena.rs:46` (the page push) or in `remove`
(`:84-93`), holder `container-field`, labelled `UNCLASSIFIED`.

## Measurements

Toolchain: cargo 1.99.1. Checks and workload (`--iterations N` runs N waves of 500 booking requests):

```
cd project && cargo test --release     # 8 passed
cargo run --release -- --iterations 3200
# requests=1600000 placed=1504073 refused=95927 confirmed=501432 released=1503773 active=300  (0.35 s)
```

External harness (kept outside the repository): a copy of `project/` with a counting
`#[global_allocator]` (live bytes, live blocks) and a read of `VmRSS`, printed just before the
process exits (`booking-desk --iterations N`; the program ends with `process::exit`, so the
desk is still alive when the numbers are read):

| waves (requests) | placed holds | active holds | live bytes | live blocks | RSS |
|---|---|---|---|---|---|
| 200 (100 000)    | 94 011    | 300 | 4 570 018  | 1 275 | 6 632 kB |
| 800 (400 000)    | 376 117   | 300 | 18 135 983 | 2 377 | 19 884 kB |
| 3200 (1 600 000) | 1 504 073 | 300 | 72 375 175 | 6 783 | 72 904 kB |

Slope: about 48.1 live bytes per placed hold (4x the waves gives 4x the live bytes), linear in
the iteration count. Live blocks rise by one per 256 holds (one page each), and `active` is 300
at every point, so the growth is not in the live holds.
