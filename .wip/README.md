# wip/progress

Snapshots of unfinished work, pushed so that nothing is lost when the build container is
recycled. This branch is never merged as is: finished work goes to `main` without `.wip/`.

- `RESUME.md`: where the work stands and what comes next.
- `bench-todo.md`: the planned benchmark work.
- `bench-baseline.md`: per-fixture outcomes of the last full `dla-bench dynamic` run before
  this round (compare new runs against it; a tracked fixture that got worse is a regression).
- `prune-superseded.sh`: frees disk in `target/debug/deps` (old test executables and old
  `libdla_*` library copies) while builds run.
