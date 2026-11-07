# go-subscription-workers-closed

Category `decoy-control` (control half of a pair, partner `go-subscription-workers`).
The growth is ruled out; expected non-leak cause `N-WARM` (only the runtime's
start-up rise is visible).

## Mechanism

Identical to the partner except that `Handler.Serve` does `defer sub.Close()`
right after opening the subscription. `Close` closes `done`, the worker's `select`
returns, and `Close` waits for the worker to exit, so no worker outlives its
request. The `go` statement in `subscription.go:33` and the `select` that waits on
`done` are the same code as in the trap and look just as suspicious.

## Distinguishing edit (trap vs control)

One added line; 11 lines of `diff -ru` output, run from `bench/fixtures`
(`diff -ru go-subscription-workers/project go-subscription-workers-closed/project`):

```
--- go-subscription-workers/project/internal/notify/handler.go
+++ go-subscription-workers-closed/project/internal/notify/handler.go
@@ -19,6 +19,7 @@
 	if err != nil {
 		return Response{}, err
 	}
+	defer sub.Close()
 	items := append([]Item(nil), req.Items...)
 	sort.SliceStable(items, func(i, j int) bool { return items[i].Priority > items[j].Priority })
 
```

(The real output also has a `diff -ru ...` header line and file timestamps on the
two `---`/`+++` lines, 11 lines in all.)

## Measurements

Toolchain: go1.24.7. Same external harness as the partner (GC, then
`runtime.NumGoroutine()` and `ReadMemStats`), same request stream.

| requests | goroutines | heap alloc after GC | stack in use | max RSS (`notifyd -n N`) |
|---|---|---|---|---|
| 0     | 1 | 84 KB  | 320 KB | |
| 2000  | 1 | 88 KB  | 352 KB | 13.1 MB |
| 8000  | 1 | 98 KB  | 416 KB | 13.1 MB |
| 32000 | 1 | 106 KB | 416 KB | 13.1 MB |

Goroutine count is 1 at every checkpoint of every run and max RSS is identical at
2000, 8000 and 32000 requests; the heap creeps by about 0.001 KB per request, which
is runtime bookkeeping, not growth proportional to the work. Contrast with the
trap: 32001 goroutines and 170 MB of live heap at 32000 requests.
Workload time at 32000 requests: 0.3 s.
