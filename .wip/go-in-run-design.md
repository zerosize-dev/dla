# F-e design: Go command mode measures while the workload runs

Status: design only. Nothing in the repository was edited. Every figure below was measured on
this machine (4 cores, go1.24.7 linux/amd64, a second package building in the same tree, so wall
times carry noise: the load average was 2 to 10) in scratch copies of go-header-index,
go-trace-context, go-buffer-pool and go-gc-tuned-batch under
`/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u14/F-e/` (`w/<project>/` are the
run directories, `res/` the saved dumps, `sampler.go.txt`, `patch.py`, `patch2.py` the prototype, `an2.py`,
`gates.py`, `mkseries.py` the analysis). The prototype is a sampler appended to the GENERATED
`zz_dla_harness_test.go` in the run directory (`dla harness gen`, `dla harness run`, edit the copy,
`dla harness run` again: `harness run` keeps an edited wrapper).

## 1. Decision

Shape: the first of the three proposals, with a different sampler and a different statistic.

* New columns, additional to every existing one: `go.heap_live.in_run` and `go.heap_objects.in_run`.
  Existing columns keep their meaning and values (the exit record is still written after `main`
  returned, after the two forced collections).
* A sampler goroutine in the command-mode template (never in driver mode) forces one `runtime.GC()` per
  sample on a geometric schedule, reads `runtime/metrics`, and stops before the exit path starts.
* The value written for a point is the FLOOR of the after-collection readings taken in the last third
  of the run (the lowest `go.heap_live` among the samples that ended at or after 2/3 of the run time; the
  last sample when the window holds none). Not the maximum, not the last sample.
* With alloc capture, the heap profile of the last valid sample replaces the exit-time heap profile
  (one `go-<n>.heap.pb.gz` per point, listed in the exit record's captures, role says "in run").
* Where no reading was possible the record says so in `limitations` and the series has no in-run column.
* `go.heap_live.in_run` is the primary heap column of `dla alloc-test --finding` only when the latest
  trend passed it through the gate; every other column choice stays as it is.

## 2. What the current exit reading shows (the defect, measured)

Exit `go.heap_live` (KB) over the nine points, and the in-run floor (MB) of the final sampler:

| project | iterations | exit `go.heap_live` | in-run floor |
|---|---|---|---|
| go-header-index (leak) | 1330 ... 11970 | 170 178 179 179 181 184 182 182 183 KB | 18.1 34.0 54.5 68.0 79.8 96.6 122.3 121.9 155.3 MB |
| go-trace-context (leak) | 22220 ... 199980 | 164 159 164 165 166 165 165 166 167 KB | 1.9 6.8 11.6 14.8 24.5 24.5 42.5 23.5 58.6 MB |
| go-gc-tuned-batch (no leak) | 44 ... 396 | 175 ... 189 KB | 11.3 10.9 12.5 12.9 15.6 10.7 11.1 11.0 10.2 MB |
| go-buffer-pool (no leak) | 440 ... 3960 | 175 ... 198 KB | 1.5 1.9 1.8 2.3 2.3 2.5 1.7 2.2 2.2 MB |

(Runs fin-r1 of each project; the leaks' state is owned by locals of `workload.Run`, dead when `main`
returns.) The truth for the leaks: header-index 15.7 KB per request, trace-context 0.32 KB per lookup.

## 3. The three proposals against measurement

Measured with the same prototype (all offline statistics are computed from the SAME sample logs, so
they differ only by the statistic; "def" = ratio 1.1, budget 15 %; "dense" = ratio 1.03, budget 50 %;
"final" = ratio 1.15, budget 25 %).

### 3.1 Proposal 2 (a collection per 100 ms tick, `.run` columns): misses short points

The existing ticker runs every `DLA_TICK_MS` = 100 ms (`run.rs`). Ticks that fall in each point of the
baseline run (go-trace-context, run lengths 33 46 90 98 124 158 193 176 244 ms): 0 0 0 0 1 1 1 1 2.
Four of nine points have no tick at all, so the column would have four holes and the other points read
a state 18 to 48 % of the run old. go-header-index has 2 to 19 ticks per point (fine), go-gc-tuned-batch
21 to 179. A tick also keeps running after `main` returned unless the exit path stops it first (the
u13 verdict saw the ticks at 44, 48 and 51 ms overwrite the live profile with a 31 KB one). The
geometric schedule (first sample at 2 ms, ratio 1.15, time budget) gives go-trace-context 2 to 5
samples in the same points and the last sample is 35 % of the run old on average (section 5); that is
the best a pointer-heavy heap allows (a collection of 60 MB of small objects costs 43 ms in a 250 ms run).

### 3.2 Proposal 3 (peak of the samples replaces `go.heap_live`, `.at_exit` twins): wrong statistic, wrong place

* The maximum of noisy after-collection readings grows with the number of samples. go-gc-tuned-batch
  (flat, 9.3 MB retained, 20 MB of batch in flight at times), the dense run subsampled to a
  schedule of ratio r, mean over the nine points:

  | ratio r | samples per point | max | last | floor (last third) |
  |---|---|---|---|---|
  | none (dense, 128) | 128 | 21.89 +- 1.05 MB | 12.48 +- 2.04 | 10.05 +- 0.30 |
  | 1.1 | 47 | 20.29 | 13.38 | 10.35 |
  | 1.25 | 25 | 19.83 | 12.29 | 11.42 |
  | 1.5 | 16 | 18.83 | 12.79 | 12.79 |
  | 2.0 | 10 | 19.44 | 12.88 | 12.88 |
  | 3.0 | 7 | 18.30 | 15.60 | 15.60 |

  The max rises from 18.3 to 21.9 MB (+20 %, roughly monotone; one dense run subsampled, so the steps carry its noise) over a 19-fold range of sample counts, and a run of n
  batches takes proportionally more samples than a run of n/2: a built-in upward drift in n on a flat
  workload. The floor falls with the count (15.6 to 10.05, toward the real retained 9.3 MB) and so is
  biased DOWN, which can only hide a leak slightly, never invent one; last and median are unbiased but
  noisy (sd 2.0 to 3.7 MB).
* It replaces `go.heap_live` and `go.heap_objects`: the exit figures, 0.18 MB for every Go fixture, would be
  replaced by an in-flight peak (20 MB on gc-tuned-batch). That changes the meaning of the columns of
  all six go-* fixtures of bench/fixtures and of the gc twin: `go.heap_objects.pre_gc` vs `go.heap_objects`
  is gc-tuned-batch's headroom evidence (section 6). Emulated (peak replacing the base columns) in 6.3.
* Replacing also loses the post-return reading, which is the right one for state held in package variables.

### 3.3 Proposal 1 (the last sample as extra columns): right shape, noisy statistic

Extra columns are right. The last sample is unbiased but carries the whole transient: on
go-buffer-pool one run's last values were 2.6 2.0 2.2 2.1 2.2 2.3 2.5 2.5 5.4 MB and the trend gate PASSED
the column (a false positive), and so did one final run of go-gc-tuned-batch (last sample 9.9 to 19.3 MB
over the runs, sd 2.0 to 3.2 MB); 2 false passes in 13 runs of the two non-leaks. The floor of the last
third is flat on both (gc-tuned 9.7 to 10.7 MB in the first runs, 9.9 to 15.6 MB in the final ones;
buffer-pool 1.5 to 2.4 MB) and the gate fails on 13 of 13 of those runs (gc-tuned: 3 def, 1 dense, 3
final; buffer-pool: 3 def, 1 dense, 2 final), while the leaks keep passing it (below).

Gate results of `dla trend` run on a copy of the baseline series with the column added (`mkseries.py`,
`gates.py`):

The "4" runs are def x3 + dense; "final" is the final schedule (3 runs of header-index, 5 of
trace-context, 3 of gc-tuned, 2 of buffer-pool). A pass on a leak is wanted, a pass on a non-leak is a false positive.

| statistic | header-index (leak) | trace-context (leak) | gc-tuned (no leak) | buffer-pool (no leak) |
|---|---|---|---|---|
| last sample | pass 4/4 | pass 3/4 | pass 0/4 | pass 1/4 (false) |
| max | = last (a leak is monotone) | = last | pass 0/4 | pass 0/4 |
| floor, last half (fl50) | pass 3/4 | pass 3/4 | pass 0/4 | pass 0/4 |
| floor, last third (fl67) | pass 4/4 | pass 2/4 | pass 0/4 | pass 0/4 |
| last sample, final | pass 3/3 | pass 5/5 | pass 1/3 (false) | pass 0/2 |
| floor, last third, final | pass 3/3 | pass 5/5 | pass 0/3 (slopes -3256 +4864 +2431 B/batch) | pass 0/2 |

## 4. The design

### 4.1 Sampling schedule (command-mode template only)

* Start: `dlaStartSampler()` right after `dlaStartTicker()` in `TestDLAHarness`; times are
  milliseconds since `dlaStart` (package initialisation, the same clock as the records' `t_ms`).
* First sample at 2 ms. After a sample that ended at `t` with `spent` ms spent inside samples in total
  (collection, metrics, profile), the next one starts at `max(1.15 * t, spent / 0.25)`. So the
  schedule is geometric (a short point still gets samples at 2, 2.3+, ... ms) and the samples never
  take more than a quarter of the elapsed time at the moment a sample starts (a sample in progress can
  overshoot, section 5).
* A sample: `runtime.GC()` once (it returns when the cycle is complete), `dlaReadMetrics`, and when alloc
  capture is on, `pprof.Lookup("heap").WriteTo(&buf, 0)` into memory. The sample is committed only if
  the exit path has not started when it ends; otherwise it is dropped (the race that gave a 0.2 MB
  reading in the first prototype: a sample that ends after `main` returned reads a dead heap).
* The exit path (the deferred function) first sets the finished flag under the sampler's mutex,
  then stops the goroutine and waits for it, then builds the reading, then writes the record.
* Environment: `DLA_INRUN=off` switches the sampler off (a limitation line says so); there is no
  other knob in the product (the ratio, the budget and the first sample are constants of the
  template, named, with the measurements of section 5 in the comment).

### 4.2 The reading

At the exit path, with `end` = elapsed ms: the window is the samples with end time `>= 2/3 * end`;
the reading is the sample with the lowest `go.heap_live` in the window (ties: the later one), the last sample when
the window is empty; `go.heap_objects.in_run` comes from the same sample. The report of which sample:
`reading_ms`.

Why the floor and the last third: section 3 (unbiased by sample count in the upward direction, flat on
non-leaks, monotone on leaks, a leak reads 65 to 87 % of its true slope: header-index floor slope
11.8 to 12.3 KB per request against 15.7, trace-context 207 to 279 B against 320). The magnitude is a
lower bound of the growth, which the evidence line says.

### 4.3 Record fields (exit record only; tick records and driver records are unchanged)

```
"values_in_run": {"go.heap_live": <bytes>, "go.heap_objects": <bytes>}      // absent when no reading
"in_run": {"samples": N, "window_samples": W, "reading_ms": t, "end_ms": T, "spent_ms": S,
           "statistic": "floor-last-third"}                                   // present when any sample ran
```

dla-core `HarnessDump` gets `values_in_run: Option<BTreeMap<String, Option<f64>>>` (like `values_pre_gc`)
and `in_run: Option<DumpInRun>` with `DumpInRun { samples: Option<u32>, window_samples: Option<u32>,
reading_ms: Option<f64>, end_ms: Option<f64>, spent_ms: Option<f64>, statistic: Option<String> }`.
`harness-dump.schema.json` is regenerated (`dla schema --out schemas`).

### 4.4 Series columns and catalog types

`build.rs` next to the `values_pre_gc` loop: for every `(name, v)` of `values_in_run` (skip `os.`
names like the pre_gc loop), `t.set(&format!("{name}.in_run"), i, v, src)`.

| column | unit | kind | region | scope | baseline | after_gc | source |
|---|---|---|---|---|---|---|---|
| `go.heap_live.in_run` | bytes | gauge | heap | process | false | true | go-harness |
| `go.heap_objects.in_run` | bytes | gauge | heap | process | false | true | go-harness |

`column_meta` (series.rs): `name.strip_suffix(".in_run")` guarded by `base.starts_with("go.")`, returns
`column_meta(base)` with `baseline: false` (and `after_gc` kept true), placed beside the `.pre_gc` branch
and documented in the doc comment above `column_meta`. The region matters beyond display:
`gate::column_rows` takes the residual rows of the column's own region from `column_meta`, so a
Process-typed column would be judged against every residual row.

Not `.pre_gc` (the gc test pairs `<name>` with `<name>.pre_gc`: `alloc_cmd.rs` looks up
`format!("{name}.pre_gc")` and hands both to `judge::gc`; `gate::is_whole_process` strips only `.pre_gc`), not `.run`
(verdict 2's name; it also reads like a probe name).

### 4.5 Capture

With alloc capture on, `dlaCapture`'s alloc branch writes the in-run profile bytes (the last valid
sample's profile, in memory until the exit path) as `go-<n>.heap.pb.gz` INSTEAD of the exit-time heap
profile, through the existing `dlaWriteProfile`-style temp file and rename, and lists it in the record's
captures with role `heap profile (in-use bytes by stack, taken in the run at <ms> ms, after GC)`.
`capture.rs gather` takes the role from the agent's note (`c.role`), `classify` keeps matching
`go-<n>.heap.pb.gz`: no Rust change. One heap profile per point is essential: with both the in-run and the
exit profile listed, `dla summarize alloc-stack-diff` would read 8 files for 4 points and the exit
profiles (about 30 KB of runtime stacks) break strict growth. Goroutine captures are unchanged (they stay
at exit: blocked goroutines are roots). When alloc capture was asked and no sample produced a profile,
the exit profile is written as today and the limitation line says so.

The prototype kept a ring of the last four profiles and wrote the one of the floor sample when it was
still in the ring, else the newest; the design keeps only the newest (one buffer, the largest state, the
same stacks grow), so the figures below are of a slightly different pick, measured on the profile of one of
the last samples either way.

Measured: the in-run profile is 8 to 13 KB (header-index) and 3 to 8 KB (trace-context); building it
costs 1.2 ms on average and 7.6 ms at most per sample on a 190 MB heap, 1.9 ms (max 9.1 ms) on
trace-context, against 13.9 and 16.6 ms for the collection. `dla summarize alloc-stack-diff` on the
four in-run profiles of go-header-index: gate pass, passed stacks `(*Index).Note` (two) and `ReadFrame`
(+119 to 122 MB at frame.go:23) and `bytes.growSlice`, tied to the census finding F2 (index.go:10, writer
index.go:36). The same on go-trace-context: gate pass, four `trace.Start` stacks (trace.go:33, 38, 43,
42, 41) growing 19 + 8 + 6 + 2 + 1 MB, tied to nothing yet (the census finding that matches is
`Span.Attrs` through writer trace.go:38 per verdict 2, not L1 at poller.go:32: the tie-rule decision
that is not in this pass).

### 4.6 Limitation lines (the record's `limitations`, merged into the series limitations by `build.rs`)

* No sample ended before the exit path: `go command mode: no in-run reading (the workload ended before
  the first sample finished, 2 ms; or the sampler was off); the in-run columns have no value at this point`.
* The reading is old: when `end - reading_ms > end / 2`: `go command mode: the in-run reading is from <t> ms
  of a <T> ms run (<N> samples; the time budget of the sampler allows few collections of a heap this
  size); it understates the state at the end`.
* A metric is unavailable (Go before 1.21 has no `/gc/heap/live:bytes`): the existing note
  `runtime metric unavailable: ...` plus `no in-run reading`.
* Alloc capture with no in-run profile: `go command mode: no heap profile was taken in the run; the exit
  profile is written (state owned by locals of the workload is not in it)`.
* Always, when a reading exists: `go.*.in_run: the lowest after-collection reading of the last third of
  the run (floor), a lower bound of the live heap while the workload ran; the base go.* columns are read
  after main returned`.
* The driver-mode and manifest `gaps` line about probes is unchanged.

## 5. Measured behaviour of the final schedule (ratio 1.15, budget 25 %)

Run-time cost. Sum over the nine points of the exit record's `t_ms`, without and with the sampler
(runs interleaved off, on, off, on; loads 2.5 to 10):

| project | without | with | change |
|---|---|---|---|
| go-header-index (190 MB, byte buffers) | 8129 7568 7372 ms | 7852 7576 8117 ms | +2.1 % |
| go-trace-context (60 MB, small objects) | 1092 1084 1099 1002 ms | 1290 1236 1256 1256 1183 ms | +16.4 % (worst; 4 points under 100 ms) |
| go-buffer-pool | 37285 34537 ms | 37285 35734 ms | +1.7 % |
| go-gc-tuned-batch (largest: 9 points, 90 to 100 s) CPU user+sys of the whole `dla harness run`, 3 pairs | 111.8 115.3 114.9 s | 113.8 120.3 116.2 s | +2.4 % (wall 104.5 98.2 98.1 s without against 101.9 101.0 106.1 s with: inside the noise) |

Time inside the sampler (sum of collection + profile over the run time): header-index 10 %, trace-context
26 to 34 %, buffer-pool 2 %, gc-tuned 3 to 5 %; worst single point 48 % (trace-context, a point of 33 ms).
The 25 % budget is checked before a sample starts, so a short point overshoots by one sample. The cost
is bounded by construction (25 % of elapsed + one sample) and measured at +16 % in the worst case, +2 to
+3 % on the large fixtures. With alloc capture the profile adds 1.2 to 1.9 ms per sample: run time
header-index 9042 8684 ms off, 9162 8949 ms on (+2.2 %); trace-context 1346 1246 1229 ms off, 1478 1436
1449 ms on (+14 %).

Scatter between repeated runs (final schedule): header-index floor slope 12240 12301 11767 B per request
(cv 2.4 %; the last-sample slopes 14636 15179 14345); trace-context 261 258 279 207 267 B per lookup
(cv 10 %, 65 to 87 % of 320; the last sample is 35 % of the run old on average, 3.3 samples per
point); gc-tuned mean floor 11.8 11.4 12.0 MB per run (flat); buffer-pool 2.06 2.02 MB.

Schedule alternatives tried on go-trace-context (5 runs each, last-sample slope per run, B per lookup
against 320; mean age of the last sample as a share of the run; mean run-time sum):

| budget, allowance | slopes | age | samples/point | run time |
|---|---|---|---|---|
| ratio 1.15, 10 % | 64 252 232 262 221 | 0.41 | 2.2 | 1284 ms |
| 20 % | 277 181 240 153 152 | 0.40 | 3.0 | 1344 |
| 20 % + 25 ms allowance | 212 141 137 258 157 | 0.31 | 5.8 | 1502 |
| 35 % | 260 197 255 270 262 | 0.28 | 4.1 | 1377 |
| off | | | | 1186 |

A fixed allowance buys no accuracy; the 25 % budget sits between 20 % and 35 %.

Growth with the iteration count. `go.heap_live.in_run` rises with n on both leaks (tables of section 2)
and is flat on both non-leaks; the trend gate passes the floor column on 8 of 8 final runs of the two leaks and on
0 of 5 final runs of the two non-leaks (0 of 13 non-leak runs over every schedule tried).

## 6. Collector figures and the non-leak verdicts

### 6.1 go-gc-tuned-batch base columns, sampler on against off (means over the nine points, MB)

| run | pre-GC heap_objects | pre-GC go.total | pre-GC os.rss | post go.total | post go.released | exit go.heap_live |
|---|---|---|---|---|---|---|
| off-1 | 23.7 +- 2.4 | 47.6 | 38.6 | 47.6 | 12.0 | 0.2 |
| off-2 | 22.4 +- 3.2 | 48.1 | 38.5 | 48.1 | 13.4 | 0.2 |
| def-1 (ratio 1.1, 15 %) | 20.4 +- 3.5 | 48.9 | 38.8 | 48.9 | 16.1 | 0.2 |
| def-2 | 22.4 +- 2.6 | 48.0 | 38.5 | 48.0 | 12.6 | 0.2 |
| def-3 | 23.2 +- 3.1 | 48.5 | 39.8 | 48.5 | 13.3 | 0.2 |
| dense | 22.1 +- 2.9 | 49.3 | 38.8 | 49.3 | 14.6 | 0.2 |

The pre-collection heap objects (the sawtooth of GOGC headroom, 20 to 24 MB against 0.2 MB after the
collection) stay inside the spread between the two runs without the sampler; a forced collection resets
the pacing, so the lowest mean (20.4) is the sampler's and stays 1 sd under the others. The unbound
alloc-test comparison (the collector evidence read by `judge::gc`) is in 6.3.

### 6.2 Verdicts of the non-leak fixtures with the in-run floor as the heap column (stress emulation)

The wrapper writes the in-run floor under the base names (`go.heap_live`, `go.heap_objects`), so the
real alloc-test code paths judge it (no purge entry, no gc twin: the same position the new column has).
This is a stricter test than the design needs, because the design only prefers the in-run column when its
own trend passed the gate.

| fixture | run | unbound alloc-test |
|---|---|---|
| go-buffer-pool | 1 | non-leak: go.heap_live plateau (purge not-run), os.rss plateau (released), os.vm_hwm plateau |
| go-buffer-pool | 2 | non-leak: go.heap_live plateau, os.rss plateau, os.vm_hwm growth (reported, no vote) |
| go-gc-tuned-batch | 1 | non-leak: go.heap_live plateau, os.rss plateau, os.vm_hwm plateau; the trend passed no column |
| go-gc-tuned-batch | 2 | non-leak: go.heap_live plateau, os.rss no-growth, os.vm_hwm growth (reported) |

And for the leak (go-header-index), same emulation, final schedule:
trend passes `go.heap_live` (steady, 11535 B per request); unbound alloc-test is inconclusive (leak votes
from `go.heap_live` growth-continues, `go.total` retained, `go.heap_objects.pre_gc`; non-leak votes from
`os.rss` and `os.rss_anon` purge released, which is the exit-time reading and the same inconclusive as
today); `dla alloc-test H1 --finding F2` (census finding at index.go:10, region heap): with the in-run
column as primary **favors leak** (go.heap_live growth-continues, purge not-run, `next` names the
sibling and the confirm), against **favors non-leak** (plateau, N-WARM, N-HWM, "dla ledger refute F2")
on the unchanged exit column, which is the failure the u13 verdict measured.

### 6.3 Collector evidence of gc-tuned-batch, alloc-test with the sampler off, on, and shape 3 (peak replacing)

One run each of `dla alloc-test H1 --column go.heap_objects` (the column that has the `.pre_gc` twin, so
`judge::gc` runs) and of the unbound alloc-test, on go-gc-tuned-batch, after a harness run and a trend with:
A the sampler off (`DLA_S_OFF=1`), B the sampler on (final schedule, nothing replaced: what the design does
to the base columns), C the peak of the in-run samples replacing `go.heap_live` and `go.heap_objects`
(proposal 3, emulated with `DLA_S_OVERRIDE=1 DLA_S_STAT=max`).

| run | trend passed | unbound alloc-test | `--column go.heap_objects`: gc test | plateau | supports |
|---|---|---|---|---|---|
| A off | os.vm_hwm | non-leak, N-WARM N-HWM | verdict none (pre_rise 35.9 KB, post_rise -25.65 MB) | plateau | N-WARM N-HWM |
| B on | nothing | non-leak, N-WARM N-HWM | verdict none (pre_rise 3.55 MB, post_rise -25.64 MB) | plateau | N-WARM N-HWM |
| C peak replaces | nothing | non-leak, N-WARM N-HWM | verdict none (pre_rise 5.31 MB, post_rise -5.30 MB) | plateau | N-WARM N-HWM |

Verdicts and supports are identical in the three. B leaves `post_rise` untouched and moves `pre_rise` by
3.5 MB (one run, the spread between two runs of A was 1.3 MB in the pre-collection mean and the figure is
noise-sized against 20 MB of sawtooth); C changes the post column's own meaning (post_rise -25.65 MB becomes
-5.30 MB, because the exit-time column now carries the in-flight peak). With a single repetition of each
this shows no change of verdict, not that the evidence cannot move; the 3 CPU-timing pairs, the 6 base-column
runs of 6.1 and the 2 stress runs of 6.2 are the other repetitions.

## 7. What the trend, the primary column and the alloc-test do with the new columns

* Trend (`dla trend`): nothing to change. The column is a byte gauge of region heap; `gate::is_whole_process`
  (`go.` prefix) keeps it from being tied to a finding by the trend, like `go.heap_live`. A passing
  `go.heap_live.in_run` is covered in the final gate by the residual row of its region (`column_rows` uses
  `column_meta`); see the residual note in section 9.
* Purge: `purge.before/after` carry no in-run entry, so `judge::purge` reads `not-run` for the column
  (shown above: "purge not-run"); no change to the purge figures list (`dlaPurgeFigures`).
* Gc test: `alloc_cmd.rs` looks up `<name>.pre_gc`; `go.heap_live.in_run.pre_gc` does not exist, `gc` is
  `None`. The in-run columns have no twin by design.
* Primary heap column (`pick.rs region_primary`): gets the latest trend as a parameter. For region heap, if
  the series has `go.heap_live.in_run` AND that column has `GateVerdict::Pass` in the trend, it is the
  primary (basis string "the primary column of the region of F, the in-run reading"); else the list as
  today (`jvm.heap_used`, `go.heap_live`, ...). Reason: on a series whose in-run column is flat the
  exit column decides exactly as it does today (the verdicts of the non-leaks do not move unless the
  in-run column falsely passes the gate, 0 of 13 measured); on a series where it grows, the exit column
  is flat for a reason that has nothing to do with the finding. `tied()` is untouched (a column tied by
  finding, site, owner or trend candidate still wins over the region primary).
* Unbound alloc-test (`pick()` third branch): `.in_run` columns that passed the gate join the voting list
  like any passed column (sorted by the existing key; they have no site and no candidate, so they sort
  with the other whole-process columns), plateau-only (no purge, no gc). The "nothing passed" fallback
  `region_primary(Heap)` keeps `go.heap_live` (the in-run column cannot have passed there). The
  `os.rss` purge reading is still taken at exit, so an unbound test of a leak held by locals stays
  inconclusive when `os.rss` reads released while `.in_run` reads leak; this is the same
  inconclusive as today and the bound test (policy step 5) decides. NOT changed on purpose.
* Alloc-test `--column go.heap_live.in_run` works (the column is in the series).

## 8. Files and functions to change

| file | change |
|---|---|
| `harness/go/zz_dla_harness_test.go.tmpl` | engine block: `var dlaInRunHook func(notes *[]string) (map[string]float64, map[string]any)` and `var dlaInRunProfile func() []byte` (nil unless command mode sets them, like `dlaProbeHook`); `dlaBuild` (non-tick): call the hook, write `values_in_run` and `in_run`, merge its notes; `dlaCapture` alloc branch: use the in-run bytes when present (new helper `dlaWriteBytes(dir, file, kind, role, b)` beside `dlaWriteProfile`), else today's path plus the limitation line. Command-mode part, outside the engine block: `dlaStartSampler()`, the constants (first 2 ms, ratio 1.15, budget 0.25, window 2/3), `dlaInRunReading()`, `TestDLAHarness` starts it after `dlaStartTicker()` and stops it first in the deferred function. Header comment: the new records, `DLA_INRUN`. |
| `harness/go/zz_dla_driver_test.go.tmpl` | the engine block copy only (hooks stay nil, driver output byte-identical); `diff` of the engine blocks must stay empty. |
| `harness/go/README.md` | "What the dump records": `values_in_run`, `in_run`; capture table: the heap profile of a command-mode point is the in-run profile; the schedule, the statistic, the budget and the limitation lines. |
| `crates/dla-core/src/harness.rs` | `HarnessDump.values_in_run`, `HarnessDump.in_run`, `DumpInRun`; the roundtrip test beside line 774. |
| `crates/dla-core/src/series.rs` | `column_meta`: the `.in_run` branch; doc comment; unit test beside `column_meta("py.traced.pre_gc")`. |
| `crates/dla-workload/src/harness/dumps.rs` | `parse_record`: `values_in_run` in the `numeric_only` key list and in the drop-one-by-one list; tests. |
| `crates/dla-workload/src/harness/build.rs` | the `.in_run` loop beside the `values_pre_gc` loop (line 240); test beside `values_groups_probes_and_pre_gc_become_catalog_columns`. |
| `crates/dla-cli/src/alloc/pick.rs` | `region_primary(series, region, trend)`; `pick()` passes `trend` in both calls; update the three existing `region_primary` tests; new tests. |
| `crates/dla-cli/src/summarize/go_pprof_heap.rs` | header doc ("written right after `runtime.GC()`; in command mode the harness writes the profile in the run"), `FIX` text, one more line in `limitations()` ("a command-mode profile is taken while the workload ran, at the last sample before its end: state freed before that is absent"). |
| `schemas/harness-dump.schema.json` | regenerate with `target/debug/dla schema --out schemas`. |
| `skills/detect-leak-anywhere/references/go.md` | a section (command mode, in-run columns): what they are, when the in-run column decides, the limitation lines, the lower bound. |
| `docs/VERIFY.md` | one row per behaviour in `## M7`: in-run column, in-run heap profile replaces the exit profile, primary column preference, limitation lines, base columns and gc-tuned collector evidence unchanged. |

Not changed (and why): `crates/dla-workload/src/harness/capture.rs` (role comes from the agent's note),
`run.rs` (`DLA_TICK_MS` stays 100), bench policy, `dla-core/src/gate.rs`.

## 9. Tests (what fails today)

"Fails today" is by reading the code (nothing was run against a changed tree; the design step edits no file).

1. `series.rs`, `column_meta("go.heap_live.in_run")` is bytes, gauge, heap, after_gc true, baseline false;
   `py.foo.in_run` is not special; `.pre_gc` unchanged. Fails today: the name falls to the default arm
   (region process, after_gc false).
2. `harness.rs` roundtrip with `values_in_run` and `in_run`; `dumps.rs` `parse_record` keeps numeric entries
   of `values_in_run` and drops a malformed one without losing `values`. Does not compile today
   (field absent).
3. `build.rs`: points with `values_in_run` give `go.heap_live.in_run` and `go.heap_objects.in_run` with the
   catalog meta; a point without one has `None`; an `os.` key in `values_in_run` gives no column. Fails
   today: no column.
4. `pick.rs`: with a series holding both columns, the bound pick for a heap finding chooses
   `go.heap_live.in_run` when the trend passed it and `go.heap_live` when the trend failed it or none exists;
   existing NonHeap/Heap tests keep their results with the new parameter. Fails today: always
   `go.heap_live`.
5. `alloc_cmd.rs` (beside the test at the Go flat-run shape): `test_column` of a rising `.in_run` column
   without a purge entry favors leak with `purge` `not-run` and `gc` `None`; a flat one favors non-leak.
   Passes today for any column name (guards the "no purge reading" contract); judged by reading.
6. `crates/dla-workload/tests/harness_run.rs` (skips without `go`, like `go_goroutines_are_counted_by_creation_site`):
   an inline Go project whose `main` fills a local slice of 8 KB buffers `n` times and returns. With the
   agent template: exit `go.heap_live` is below 1 MB at every point; `go.heap_live.in_run` is above
   0.5 x n x 8 KB at the last point and rises over the points; the `limitations` of the series hold the
   floor line. Fails today: no column.
7. same file, `--capture alloc` (RunOptions capture): one heap profile per point, its role contains "in the run",
   and `dla_parsers::go::heap` decodes it with a stack through the allocating function in the project.
   Fails today: the exit profile has no such stack.
8. same file: `n` so small that no sample ends: no `values_in_run`, the limitation line is present, the
   series has no in-run column, the base columns are intact. Fails today: the line is absent.
9. same file, driver mode (an existing Go driver test): the dump has no `values_in_run` and no `in_run`
   (guards "driver mode unchanged"); a test that the two templates' engine blocks are identical
   (`sed -n '/DLA-ENGINE-BEGIN/,/DLA-ENGINE-END/p'`, in `templates.rs` tests). Passes today for the diff;
   it guards the edit.
10. `gate.rs`: `column_rows` for `go.heap_live.in_run` returns only the heap-region residual rows and
    `is_whole_process("go.heap_live.in_run")` is true (a coverage manifest with heap and non-heap rows).
    Fails today: all rows of the series come back (region process).
11. `go_pprof_heap.rs`: `limitations()` holds the in-run sentence (adapter output test).
12. Bench: `dla-bench dynamic --parallel 1 --only` each of the six go-* fixtures keeps its outcome (compare
    with `u11acc/dynamic.json`); scratch copies of go-header-index, go-trace-context, go-feed-merge: the
    series grows (`go.heap_live.in_run` passes the trend), the in-run profile shows the allocating stacks.

## 10. Where each open fixture stops with this change (measured with the prototype; F-d is not in this design)

* go-header-index: in-run column passes the trend (3 of 3); bound alloc-test of a heap finding reads leak
  (6.2); the in-run profile ties to the census finding F2 through its writer index.go:36 (gate pass).
  With the F-d rule's finding at index.go:36 (holder container-field, region heap) the remaining steps are
  the ones the policy runs today.
* go-trace-context: the column passes (5 of 5), the profile shows four `trace.Start` stacks growing; the
  tie to L1 at poller.go:32 is the tie-rule question (the stack's first project frame is `trace.Start`, the
  finding's site is its caller): it stops there.
* go-feed-merge: not run here (no scratch copy built); the verdict's numbers (last-tick live 39.4 to 161.9
  MB against 12.7 to 50.7 MB at exit) say the in-run column would show L2; it ties through the same
  rule as trace-context.

## 11. Risks and what was not measured

* Statistic. The floor is a lower bound: 65 to 87 % of the true slope here. A leak whose growth is
  smaller than the floor's noise (0.3 MB on gc-tuned) per run of the series is not visible; the exit
  columns and the profile remain as they are today.
* A workload that is never idle at the sampler's instants and frees nothing between batches reads its
  working set as the floor (like a leak of constant size): flat in n, so the trend does not pass it.
* Short pointer-heavy runs (go-trace-context: points of 33 to 250 ms, collection of the whole heap 15
  % of the run): 2 to 5 samples, the last 35 % of the run old; 5 of 5 runs of the final schedule pass the
  gate but the 20 % schedule failed one run in three, so the verdict on such a project can flip
  between runs. This is the limit of "after a collection", not of the schedule.
* Cost. +16 % on a 1-second Go run, +2 to +3 % on 8 to 100 s runs, +2 to +14 % with alloc capture
  (profile). Heaps of gigabytes were not measured: a collection of 1 GB of small objects takes
  seconds; the budget keeps it at 25 % of elapsed, the staleness grows, a limitation line says so.
* gc-tuned-batch: the pre-collection heap objects mean fell from 22.4 to 23.7 MB (two runs without the
  sampler) to 20.4 to 23.2 MB (four runs with it), inside the spread. The three alloc-test
  comparisons of 6.3 are the evidence; 3 repetitions of the sampler on run were made for the cost, 1 for the
  alloc-test.
* Not measured: Windows and macOS (the sampler uses only `runtime`, `runtime/metrics` and `pprof`, but the
  timings were not taken there); Go toolchains other than go1.24.7 (the metric `/gc/heap/live:bytes`
  exists from Go 1.21); `GOGC=off` and `GOMEMLIMIT` workloads; `GOMAXPROCS=1`; a workload that calls
  `runtime.GC()` itself in a loop; heaps over 200 MB (largest measured 190 MB); workloads that exit by
  `os.Exit(n)` with `n != 0` (no exit record at all, ticks only: they have no in-run reading; an
  extension would copy the best-so-far sample into the tick records; not designed here);
  go-feed-merge (not run); the whole dynamic bench with the change (it needs the change).
* `dla gap --residual` (`gap/residual.rs HEAP` list) is not changed: it still reads `go.heap_live` and so
  does not credit in-run growth to findings; the final gate covers the in-run column through the heap
  residual row (under threshold). A follow-up could prefer `.in_run` there; doing it changes the final
  gate of every leaking Go fixture and needs its own bench run. Not in this design.
* The unbound alloc-test of a leak held by locals stays inconclusive while `os.rss` reads purge released at
  exit. Deliberate (7); an option is to let a passing in-run column outvote the exit-time purge
  readings, which was not tried.
* Sample goroutine and the workload's own `runtime.GC`/finalizers: a sample is one more forced
  collection; programs that count GCs (go-gc-tuned-batch's collector figures) see more collections
  (in the 6.1 runs the pre-collection heap objects stayed within the run-to-run spread).
