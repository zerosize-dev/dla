Real output of `runtime/pprof` from go1.27.0, written by a small program that replays the
`go-subscription-workers` bench fixture (10 requests per round, three rounds, `runtime.GC()`
before each write, `runtime.MemProfileRate = 64`) and starts three goroutines per round on a
channel held by a global:

- `heap-N.pb.gz`: `pprof.Lookup("heap").WriteTo(f, 0)`;
- `goroutine-N.pb.gz`: `pprof.Lookup("goroutine").WriteTo(f, 0)`;
- `goroutine-N.txt`: the same profile at `debug=2`;
- `goroutineleak-3.txt`: `pprof.Lookup("goroutineleak").WriteTo(f, 2)` after round 3.

The program was built inside `.../harness/H1/work/`, as the harness does, so the file paths
carry that marker. `main.AKIAIOSFODNN7EXAMPLE1` is a function name made to look like a
key, for the redaction tests.
