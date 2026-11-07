# rust-batch-buffers

Category `non-leak`, cause `N-HWM` (high-water mark), no partner.

## Mechanism

`Rollup` keeps one `Vec<Sample>` and one `HashMap<String, SeriesStats>` for the
whole life of the process and empties both after every batch with `clear()`
(`src/rollup.rs:57` and `:58`). `clear()` drops the elements but keeps the
allocation, so each container holds the capacity of the largest batch it has ever
seen. Most batches carry 300 to 6000 samples; batch 11 is a 60 000-sample catch-up
burst. Both containers reach their final size at batch 11 (the vector doubles to
65 536 slots, the map to 65 536 buckets) and never grow again, whatever the number
of batches. An append that is never trimmed looks like growth in a static reading;
the measurements show a single step and a plateau, and zero entries between
batches.

Expected: no confirmed leak; both retained buffers ruled out with the plateau as
the discriminator.

## Measurements

Toolchain: cargo 1.97.0. Checks and workload:

```
cd project && cargo test --release     # 7 passed
cargo run --release -- --iterations 2000
# batches=2000 samples=6356304 series=6268437 rejected=0 largest_batch=60000 hottest_mean=99.99  (3.4 s)
cargo run --release -- --iterations 8000   # 13.3 s
```

External harness (kept outside the repository): a copy of `project/` with a
counting `#[global_allocator]` and a read of `VmRSS` from `/proc/self/status`,
printed before the listed batch (that is, after the previous flush). One run of
8000 batches (the 2000-batch run prints identical values at its checkpoints):

| batch | live bytes | live blocks | RSS |
|---|---|---|---|
| 0 (start)  | 548       | 2 | 2.1 MB |
| 1    | 795 200   | 5 | 3.3 MB |
| 5    | 795 200   | 5 | 3.3 MB |
| 11   | 795 200   | 5 | 3.3 MB |
| 12   | 6 357 568 | 5 | 11.9 MB |
| 20   | 6 357 568 | 5 | 11.9 MB |
| 100  | 6 357 568 | 5 | 11.9 MB |
| 1000 | 6 357 568 | 5 | 11.9 MB |
| 2000 | 6 357 568 | 5 | 11.9 MB |
| 4000 | 6 357 568 | 5 | 11.9 MB |
| 8000 (end) | 6 357 556 | 4 | 11.9 MB |

The batch-11 step is the catch-up burst (+5.5 MB live). From batch 12 to batch 8000
live bytes and RSS do not change at all (slope zero over 7988 batches); the 2000-batch
run ends at the same 6.36 MB / 11.9 MB as the 8000-batch run, so quadrupling the
iterations adds nothing.
