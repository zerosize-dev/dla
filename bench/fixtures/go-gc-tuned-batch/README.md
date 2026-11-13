# go-gc-tuned-batch

Category `non-leak`, cause `N-HEADROOM`, no partner.

## Mechanism

Every batch is decoded into a fresh slice of about 60000 events (`event.go:45`), applied
to a ledger of 3000 accounts with 365 daily buckets each (about 9 MB, fixed at start-up),
and dropped. A batch allocates about 20 MB of short-lived data. The launcher
`run.sh` sets `GOGC=400` (`run.sh:5`), so the Go collector waits until the heap is five
times the live data before it runs: heap in use sawtooths between about 30 and 80 MB and
the process's resident set sits at about 103 MB, eleven times the live heap. Nothing
accumulates: the account table has a fixed key set, the status list keeps the last 32 reports
(`report.go`, re-sliced on every `Add`), and the per-batch slices are garbage. A reader who
sees a resident set far above the data, and a list that is appended to, will suspect a
leak; a forced collection shows 9.3 MB at every checkpoint.

Expected: no confirmed leak; the growth ruled out with the series after a forced collection
as the discriminator. `N1` is the collector setting in `run.sh`, `N2` the per-batch slice.

## Measurements

Toolchain: go1.24.7 linux/amd64, standard library only. Checks and workload:

```
cd project && go vet ./... && go test ./... && GOGC=400 go run ./cmd/tallyd -n 200
# batches=200 events=11908775 rejected=91225 checksum=085c309a   (7.5 s wall; ./run.sh does the same)
```

External harness (kept outside the repository): copies `project/`, adds a `cmd/measure`
main that runs `workload.NewPipeline()` batches and, at the listed counts, prints
`HeapInuse` as it stands, then runs `runtime.GC()` and prints `HeapAlloc`, `NumGC` and the
peak `HeapInuse` and `VmHWM` so far. Run with `GOGC=400` (the launcher's value):

| batch | heap in use before the forced GC | heap alloc after GC | collections | peak heap in use | max RSS |
|---|---|---|---|---|---|
| 1   | 32.6 MB | 9.11 MB | 2   | 32.6 MB | 34.7 MB  |
| 50  | 49.1 MB | 9.12 MB | 26  | 74.1 MB | 97.6 MB  |
| 100 | 50.2 MB | 9.12 MB | 51  | 77.4 MB | 100.6 MB |
| 200 | 55.5 MB | 9.12 MB | 102 | 77.4 MB | 103.3 MB |
| 300 | 72.7 MB | 9.12 MB | 149 | 78.9 MB | 105.1 MB |
| 400 | 52.9 MB | 9.12 MB | 196 | 78.9 MB | 105.1 MB |

After the first ten to fifty batches the series is flat: the heap after a collection moves
by 2 KB between batch 50 and batch 400, the peak heap
in use stops rising at batch 300, and max RSS of `tallyd -n N` is 92 MB at 20 batches,
102 MB at 50 and 103 MB at 200. With `GOGC=100` (the default) the same replay has a
heap after GC of 9.12 MB, a peak heap in use of 27 MB and a max RSS of 40 MB at 200 batches,
so the setting accounts for the gap between 105 MB and 40 MB.
