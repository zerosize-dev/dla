# The axis of Go heap-profile captures (brief for one fixer)

Repo /home/user/dla, branch main. The working tree holds uncommitted work of another pass
(49 files; `git status`). Do NOT commit, push, stash, reset or revert anything. Touch only the
files this job needs; never undo a change you did not write.
sp = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad
Your scratch dir: `<sp>/u26/` (create it; delete only it, by literal path; never
`rm -rf "$VAR/..."`).

## The defect
`dla-bench dynamic` on the fixture `go-ticker-legacy` fails by chance (seen once in a run under
load; logs `<sp>/u24/go-all.logs/go-ticker-legacy/`, a passing run for comparison
`<sp>/u23/dynamic.logs/go-ticker-legacy/`; the bench's decision trace is at the end of
`<sp>/u24/go-all.out`). The chain: `dla harness run H1 --capture alloc,nonheap` writes one heap
profile per captured point (4 of the 9 points), `dla summarize alloc-stack-diff <4 files>`
builds a series with one point per capture and applies the growth gate per stack column. In the
failing run the leak's stack did not pass:

    stack ...reserve.awaitConfirm   3.28 MB, 16.97 MB, 29.36 MB, 35.88 MB
    halves first 13.70 MB, second 6.52 MB, ratio 0.476  -> "decaying half-split" (threshold 0.5)

so the growing stacks were tied to no finding and nothing was confirmed. In the passing run the
same column read 3.79, 16.73, 27.70, 35.75 MB (ratio 0.62). The leak is exactly linear in the
work done (points of 440 x 1, 4, 6, 9 units), so the ratio should be near 1.0: the reading is
biased, and 0.5 is hit by chance.

Cause (read `harness/go/zz_dla_harness_test.go.tmpl`, `dlaStartSampler`, `dlaCapture`, and
`crates/dla-cli/src/summarize/go_pprof_heap.rs`): in command mode the heap profile of a point
is the one of the LAST IN-RUN SAMPLE (the exit-time profile misses state owned by locals of the
workload). Samples follow a geometric schedule (each 15 % later than the last ended, at most a
quarter of the elapsed time inside samples), so the last sample lands at a different share of
each run: somewhere between about 65 % and 100 % of the work. The capture series then uses
"x is the capture number in capture order" as its axis, as if every profile were taken at the
end of its point. This is on `main` since the commit that added the in-run profile; the same
bias applies to every command-mode Go fixture (`go-header-index` among them).

## The fix to build
Give each capture of a Go heap profile an axis value that says how much work the process had
done WHEN THE PROFILE WAS TAKEN, taken from the profile itself: a Go heap profile carries the
cumulative `alloc_space` (and `alloc_objects`) of every stack next to `inuse_space`, both as of
the same collection, so the sum of `alloc_space` over all samples is the number of bytes the
process had allocated at that moment. A loop workload allocates the same per unit of work, so
this sum is proportional to the work done at the sample, in command mode, in driver mode and
for profiles of one long-running process taken by `dla attach` alike.

1. In `dla summarize alloc-stack-diff` for the Go heap backend: when every capture has a
   positive total `alloc_space` and the totals strictly increase in capture order, the series'
   x axis is that total (bytes allocated when the profile was taken); otherwise the axis stays
   the capture number, as today, and a limitation line says why. The limitation text that
   describes the axis says which one was used. Find where the axis and the series' `axis`
   field are set and what else reads x of a capture series (trend, the half-split, Sen slope,
   `t_ms`, the report, `dla trend S<n>` on the same series later): every consumer must agree.
   Check `crates/dla-stats` for how the halves and slopes use x (they must use the x values,
   not the index). Units and wording of slopes printed for such a series must stay true
   ("per byte allocated" is not "per iteration").
2. Decide by measurement, not by argument. Before changing code, collect capture sets: run
   `<sp>/u26/bin/dla-bench dynamic --dla-bin <sp>/u26/bin/dla --parallel 1 --only
   go-ticker-legacy --keep-work --log-dir <dir> --json <file>` (copy `target/debug/dla` and
   `target/debug/dla-bench` to `<sp>/u26/bin/` first; build them with `cargo build -j 2 -p
   dla-cli -p dla-bench` if the tree changed) at least 6 times, and `go-header-index` 4 times;
   keep the 4 profile files of each run (the kept work directory is printed or is under
   `/tmp/dla-bench-dynamic-*`; copy the `capture` directories to your scratch dir and delete
   the kept work directories afterwards, they are large). For each set compute, for the top
   growing stack, the half-split ratio with the capture-number axis and with the
   alloc_space-total axis (a small script over `go tool pprof -raw` or `-sample_index` output,
   or the summarize code itself in a test). Report both ratios per set. Expected: near 1.0 with
   the new axis on go-ticker-legacy. If the new axis is not clearly better (ratios not within
   0.8 to 1.25 on the linear leak), stop and report what you measured instead of shipping it.
   IMPORTANT: before any `dla-bench dynamic` run, wait until the file
   `<sp>/u24/rounds.out` contains the line `ROUNDS-DONE` (another job measures the same
   fixtures until then; check with grep, sleep in steps of 60 s at most 60 times).
3. Tests you SEE fail without the change (revert the hunk, run, restore): a unit or CLI test
   that feeds summarize four heap profiles of a linear leak taken at 67 %, 87 %, 100 % and 82 %
   of runs of 1, 4, 6 and 9 units (build the pprof protobufs in the test the way the existing
   tests of `crates/dla-cli/tests/summarize_go.rs` do, with inuse and alloc figures) and
   asserts the leak's stack passes the gate; a test of the fallback (a capture without alloc
   totals, or totals that do not increase: capture-number axis and the limitation line); a
   test that a flat stack still fails and a plateau (a real one in x) still reads plateau.
4. Other Go fixtures must keep their outcome: after the change run
   `dla-bench dynamic --parallel 2 --only go-` three times (all pass each time; the baseline is
   `<sp>/u23/dynamic.json`), then `go-ticker-legacy` and `go-header-index` five more times
   each. Report every run. A chance failure is not "flaky, moving on": trace it and report it.
5. Docs: `skills/detect-leak-anywhere/references/go.md` and `references/backends.md` where the
   capture axis or the in-run profile is described; `harness/go/README.md` if it states the
   axis. docs/VERIFY.md: one new row at the end of the `## M7` table
   (`| Item | Status | Evidence | Resolution |`, exactly 4 cells, escape pipes as `\|`, never
   nest backticks in one code span, "corrected" only when its tests ran, name every test; no
   model names, no line numbers, no scratch paths, no plan or package names).

## Rules
- Never edit anything under bench/fixtures/ or bench/fixtures-open/, nor EXPECTED_FAILURES in
  bench/runner/src/dynamic_cmd.rs. No change to thresholds of the gate (0.5 stays): fix the
  measurement, not the bar. Never skip, ignore or weaken a test.
- Cargo with `-j 2`, the shared `target/` only. Use copies of the binaries for bench runs
  (`DLA_PLUGIN_ROOT=/home/user/dla` when you call the copy of dla directly). Never kill
  processes by name.
- Disk: if `df --output=avail -B1M /` is below 3000, run `bash <sp>/prune-superseded.sh` and
  delete kept bench work directories of your own finished runs by literal path.
- Before reporting: `cargo test -j 2 --no-fail-fast -p dla-cli -p dla-stats` (and every other
  crate you touched), `cargo clippy --workspace --all-targets -j 2 -- -D warnings`,
  `cargo fmt --all --check` (format only your own files), `target/debug/dla schema --check`
  (regenerate with `dla schema --out schemas` only if a serialized type changed),
  `cargo test -j 2 -p dla-cli --test skill_text`.
- No separate checker follows you: the orchestrator reruns your commands. Write the full report
  to `<sp>/u26/report.md` (the measurements as a table) and return at most 50 lines: what you
  measured before and after, files changed, tests (which you saw fail), every command with its
  result, what is unverified.
