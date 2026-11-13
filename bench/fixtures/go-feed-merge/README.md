# go-feed-merge

Category `multi-leak`. Two independent leaks in different packages; no partner.

| Leak | Package | Cause | Holder | Region | Share |
|---|---|---|---|---|---|
| L1 | `internal/fetcher` | `L-WORKER` | `blocked-stack` | `non-heap` | 0.6 |
| L2 | `internal/indexer` | `UNCLASSIFIED` | `container-field` | `heap` | 0.4 |

## Mechanism

**L1, a goroutine per feed read.** `Merger.Round` opens a `fetcher.Session` on each of
three upstream feeds (`merge.go:37`). `newSession` starts a goroutine for the session
(`session.go:30`); it waits in a `select` on its read and `done` channels
(`session.go:36-46`) and returns only when `done` is closed, which only
`Session.Close` does. `Round` reads the document and drops the session without calling
`Close`; the fetcher keeps no table of sessions, so each goroutine is referenced only
by its own blocked stack. Per session one goroutine (4 KB of stack), its 1 KB scratch
buffer and its channels stay alive: three goroutines per round. `Close` exists and the
unit tests use it, but no production caller does.

**L2, headlines that pin their documents.** `Parse` returns each item's headline as a
sub-slice of the 6 KB feed document (`item.go:36`), and `Round` hands the three newest
headlines of the round to `Indexer.Remember` (`merge.go:61`), which stores the slice in
a ring of 120000 headlines (`indexer.go:24`). The ring is capped, so its entry count is
bounded, but a retained 30-byte headline keeps the whole document it was cut from: about
two documents (12 KB) pinned per round. The ring reaches its cap after 40000 rounds, far
beyond any replay of this workload. The mechanism fits no taxonomy row: the container is
capped and the growth comes from what its elements point into. The repair is to copy the
headline (`bytes.Clone`) before storing it.

The two leaks touch no common code: the fetcher knows nothing of the indexer, and the
merger only calls both.

## Measurements

Toolchain: go1.24.7 linux/amd64, standard library only. Checks and workload:

```
cd project && go vet ./... && go test ./... && go run ./cmd/feedmerged -n 4000
# rounds=4000 items=1575132 unique=1575132 checksum=097ad9fb   (1.7 s wall)
```

External harness (kept outside the repository): copies `project/`, adds a `cmd/measure`
main that runs `workload.NewMerger()` rounds and, at the listed counts, runs
`runtime.GC()` and prints `runtime.NumGoroutine()` and `runtime.ReadMemStats`
(`HeapAlloc`, `StackInuse`). The same harness ran two variants to split the growth: one
with `sess.Close()` after each read (L2 alone) and one with `bytes.Clone` on the stored
headline (L1 alone).

| rounds | goroutines | heap alloc after GC | stack in use | max RSS |
|---|---|---|---|---|
| 1    | 4     | 2.9 MB   | 352 KB  | 3.0 MB |
| 2000 | 6001  | 40.1 MB  | 23.9 MB | 98 MB  |
| 4000 | 12001 | 76.9 MB  | 47.3 MB | 194 MB |
| 6000 | 18001 | 113.7 MB | 70.8 MB | 292 MB |
| 8000 | 24001 | 150.6 MB | 94.2 MB | 395 MB |

Between 2000 and 8000 rounds (heap plus stack): 30.9 KB per round in total, constant at
every checkpoint. Variant with the fetcher closing its sessions: 1 goroutine at every
count and 12.7 KB of heap per round (L2, 41%). Variant with `bytes.Clone`: 3.0 goroutines
and 18.2 KB per round (heap 6.2 KB plus stack 12.0 KB; L1, 59%). The shares in
`expected.toml` are 0.6 and 0.4. Growth is linear in the number of rounds for both; there
is no plateau within the replay range.
