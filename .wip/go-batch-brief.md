# Go measurement batches: one package per batch, size probes for struct fields

You are a builder on the `dla` tool (repo /home/user/dla, Rust workspace). Read first:
`/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/wip-notes/real-run-plugin-grpc-2.md`
(what a real run on grpc-go showed), then the code named below. `<sp>` below is
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad. Your scratch
directory is `<sp>/u38/` (exists).

## Working rules (binding)
- No git commit, push, stash, reset, checkout of files. Never restore a file from a copy made
  earlier: another builder edits the same tree (it works in `crates/dla-core/src/trigger.rs`,
  `crates/dla-cli/src/ledger_queue.rs`, `crates/dla-cli/src/ledger_cmd.rs` and the report
  crate; do not touch those). Edit with exact replacements; re-read a file right before editing.
- cargo always with `-j 2`; only the shared `target/`. The machine has 4 CPUs and other work
  runs on it: a timing test that fails is rerun alone before you call it a failure.
- Never kill processes by name (no pkill/killall). No `rm -rf` with a variable in the path:
  literal paths only.
- Every behaviour change gets a test that you have SEEN fail without the change (revert the
  hunk by exact replacement, run, put it back by the reverse replacement, run again).
- Nothing in the repository names a real open-source project, a model, a plan or an agent.
  Tests use small synthetic Go projects in temp directories.
- docs/VERIFY.md: one row per behaviour change at the end of the `## M7` table, 4 cells, tests
  named, like the rows already there.
- Write your report with the Bash tool: `cat > <sp>/u38/report.md <<'EOF' ... EOF`. Your final
  message is a short summary plus that path.

## Job 1: a Go `--next` batch is one package
Today `dla harness gen --lang go --driver --next` takes the next 8 pending candidates by rank
(plus same-owner siblings up to 16) from any package and generates the driver in the package
of the discovered workload. A Go test file can read unexported names of its own package only,
so most targets cannot be reached or probed.
Wanted:
- The batch of a Go queue is: the top-ranked pending candidate, plus the other pending
  candidates whose site is in the same package directory, in rank order, up to 8; then
  same-owner siblings as now (up to 16 in all; they are in the same package by construction).
- The driver is generated in that package directory as an in-package test (what
  `--package <dir>` does today), and `harness.json` runs `go test ... ./<that dir>`.
- `--next --package <dir>` takes the pending candidates of that package (refusal with the fix
  when none is pending there). `--next` without `--package` picks the package as above.
- The queue item of `dla run status` and the `--next` output name the package of the batch.
  The target block at the top of the driver says which package the driver is in and that
  unexported names of it are reachable.
- A package directory that cannot hold the driver (no buildable non-test Go file, or
  `go list` fails there): say so in the output notes and take the next package by rank; the
  skipped candidates stay pending and are not counted as targeted.
- Other runtimes: unchanged. (If you see the same problem for Java package-private state, do
  not fix it; describe it in your report.)
Code: `crates/dla-core/src/queue.rs` (`with_siblings`, `RuntimeQueue::batch_for`, `declarable`),
`crates/dla-workload/src/harness/gen.rs` (`go_package_dir_for`, the driver block),
`crates/dla-workload/src/harness/targets.rs`, `crates/dla-cli/src/harness_cmd.rs`,
`crates/dla-core/src/view.rs` (queue item text).

## Job 2: size probes for struct fields in Go driver mode
Today a Go probe exists for package variables only (`GoVar`); for a field holder the output
says "is not a package variable, so it is recorded as a target without a probe"
(`crates/dla-workload/src/harness/probes.rs`). Most real holders are fields.
Wanted:
- A target whose holder is a field of a struct type (census owner `<pkg path>.<file>.<Type>.<field>`,
  kinds container-field and registration-table) gets a probe entry in `probes.json` (a new
  kind next to GoVar; keep the schema additive) with an id, the type name and the field name.
- The driver engine gets `dlaTrack(v any)`: the model calls it in `setup` (or in `unit` for
  objects created later) with each object it builds. At every checkpoint the engine reads, for
  each field probe, the tracked objects whose (pointer-to-)struct type has that name and sums
  the size of the named field by reflection: `Len()` for map, slice, array, chan and string;
  for a pointer or interface it follows one level; a `sync.Map` or a type with no length is
  reported as size unknown with a note. Reflection can read `Len()` of an unexported field;
  do not use unsafe. Tracked objects are held by the engine (say in the comment that tracking
  keeps the object alive, so only long-lived objects are tracked).
- Nothing tracked for a probe's type by the first measured checkpoint: the probe record carries
  an error text "no tracked object of type T: call dlaTrack(<your T>) in setup", and the
  target's verdict in `dla trend` says so (find the existing verdict for a probe that could
  not be read, `not_loaded` or its Go equivalent, in `crates/dla-cli/src/trend/verdicts.rs`;
  do not let such a target read `executed` and settle without a size).
- When tracked, the probe column feeds the growth gate exactly like a package-variable probe:
  the target reads `grew` or `constant`.
- The same probes in an entry-path harness (`--entry`) of Go.
- The hook comment in the driver lists, per field probe, the one line to write
  (`dlaTrack(<the T your setup built>)`), and the package-variable probes keep their
  `dlaProbes` hook as now.
Template: `harness/go/zz_dla_driver_test.go.tmpl` (the engine block is shared with
`zz_dla_harness_test.go.tmpl`: a test asserts the two engine blocks are identical; keep it so).

## Job 3: a rising size probe is not a plateau
Real case: unit adds one small entry to a `map[string]int32` field per key. `dla alloc-test`
read `plateau` on go.heap_live (a Go map grows in steps) and the finding was refuted.
First write the test (a synthetic Go project, driver mode, field probe tracked, small
entries): `dla trend` must read the target `grew`; then `dla alloc-test H --finding F`:
read `crates/dla-cli/src/alloc_test*` (or wherever `plateau+purge` lives) and find which
columns it judges. Wanted rule: when the finding has a size probe column in the series of the
test (main and extra rounds) and that column keeps rising through the extra rounds (use the
existing growth gate, not a new threshold), the result favors `leak` for that finding
whatever the whole-process byte columns read, and the output says which column decided. When
the probe column is flat through the extra rounds it favors `non-leak` as today. Check what
the Python and JVM paths do with their probe columns and keep one rule for all runtimes; if
they already do this and only Go lacked the probe, say so and change nothing there.

## Job 4: check on a real tree (tool check, not a model check)
Copy `<sp>/u18/grpc/proj` to `<sp>/u38/grpc` (cp -r; it is a plain source tree). In the copy,
with your built binary and `DLA_PLUGIN_ROOT=/home/user/dla`: the static stage as in
`<sp>/u29/static.sh` (`run init`, `probe`, `sweep --lang go`, `census --lang go`,
`workload discover`, `ledger list`). Start the run the way the unattended benchmark runner
does: environment `DLA_CONSENT_TIERS=1-4 DLA_CONSENT_RAW_VALUES=1` on `dla run init` (see
`/root/dla-real/run.py`). Never run `dla consent` yourself, and if a command is refused by
the permission system do not look for another way: report it.
Then: `harness gen --lang go --driver --next` and show the batches by package (generate, look,
and for the check use `--next --package health` to get the batch of the `health` package
directly). Fill that driver: setup builds `NewServer()` and tracks it; unit sets a serving
status for a new service name and opens and cancels one `Watch` for a new service name (a
small fake stream type in the hook body is fine). `harness run --mode both`, `trend`: both
fields of the server must read `grew` from their size probes in the fresh series;
`alloc-test` for both must not favor non-leak. Record the exact outputs in the report.
(You know the cause here; that is fine, this checks the tool.)
Delete `<sp>/u38/grpc/.dla/*/harness/*/work` copies you no longer need (literal paths) and
keep the disk above 2 GB free (`df -h /`); Go build caches grow fast: `go clean -cache` only
if you set GOCACHE to a directory under `<sp>/u38/` for your runs (do that).

## Acceptance (say exactly what you could not run)
- `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -j 2 -- -D warnings`.
- `cargo test -j 2 --no-fail-fast -p dla-core -p dla-workload -p dla-cli -p dla-census -p dla-report`.
- `target/debug/dla schema --check` (regenerate with `dla schema --out schemas` if you changed
  a schema); `python3 bench/runner/test_grade.py`; `cargo test -j 2 -p dla-cli --test skill_text`.
- `dla-bench dynamic --only go-` with your binary (copy `target/debug/dla` and
  `target/debug/dla-bench` to `<sp>/u38/bin/` first; run from /home/user/dla with
  `DLA_PLUGIN_ROOT=/home/user/dla`): all Go fixtures pass.
- Docs: `skills/detect-leak-anywhere/references/go.md` (batches by package, dlaTrack, what the
  verdicts mean), `references/stages.md` only where the batch command is described,
  `harness/go/README.md`. SKILL.md has word limits checked by the skill_text test: do not
  grow it.

## Report
Per job: what changed (files, functions), tests and that you saw each fail, the real-tree
outputs of job 4, anything you decided not to do and why, anything unverified.
