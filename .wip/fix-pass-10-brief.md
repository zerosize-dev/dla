# Fix pass U10: the native summary reads the wrong process's record (orchestrator brief)

Repo /home/user/dla, branch main, HEAD 9781542. Do NOT commit or push.
sp = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad

ANOTHER PASS IS EDITING THE SAME TREE NOW (its brief: <sp>/u9-brief.md). Its files are not yours:
crates/dla-cli/src/alloc/**, crates/dla-cli/src/alloc_cmd.rs, crates/dla-cli/src/summarize/jvm_nonheap.rs,
crates/dla-cli/tests/alloc_test*.rs, attach.rs, harness_capture.rs, crates/dla-parsers/**,
crates/dla-sample/**, crates/dla-workload/tests/**, harness/jvm/**, harness/python/**, and its rows
in docs/VERIFY.md. Never review, change, format or revert them, and do not report on them. Your
files: crates/dla-cli/src/summarize/shim_groups.rs, crates/dla-cli/tests/summarize_native.rs (your
own tests only), crates/dla-workload/src/harness/capture.rs, dumps.rs and run.rs where the decision
needs it, bench/runner/src/dynamic_cmd.rs (the failure text only), one new row in docs/VERIFY.md,
and a doc sentence if one states the old rule.

## What fails
The CI job "dynamic acceptance on the fixtures" failed by chance on `jni-order-codec` (twice in
about six runs, never in a local run): `leak L1 (L-HANDLE) label refused label-evidence-missing`,
because both summaries of the allocator shim failed their gate: "nothing grows strictly across all
17 captures; stack:Java_shop_codec_Native_open (native/src/lib.rs:40): holder absent from dump 1".

## Cause (measured on files a local run left behind)
The fixture's command is `sh ./build.sh run {n}`. With the shim preloaded it starts two JVMs that
each write a shim exit record with stacks: the project's test program (`shop.codec.CodecTestKt`)
and then the program (`shop.codec.MainKt {n}`). The harness splits those records into
`harness/H1/out/p<k>-native/shim-dump.jsonl` (`shim_dump_commands` in
crates/dla-workload/src/harness/capture.rs keeps every native exit record with stacks). The reader
(`shim_run` in crates/dla-cli/src/summarize/shim_groups.rs, used by `type-histogram-diff via
shim-stack-groups` and `alloc-stack-diff via shim-alloc-stacks`) then keeps, per file, "the one
with most live bytes". Totals of `raw_sites[*].live_bytes` in
/tmp/dla-bench-dynamic-2919/jni-order-codec/.dla/20261113-001707-b611/harness/H1/out/:
- p1-native: test JVM (pid 10920) 41858379, program (pid 10872, argv ends `shop.codec.MainKt 2220`) 47769802
- p2-native: test JVM 18908890, program 48416586
The test JVM's total moves by tens of MB from run to run (JIT arenas alive at exit); at the first
point the program leads by about 6 MB, which is its leak at that point. When the test JVM's record
is the larger one, capture 1 is the wrong process. It opened and closed its handles, so the leak's
stack has no live block there: "holder absent from dump 1". The harness's own series is right,
because it chooses by identity (`select_measured` in crates/dla-workload/src/harness/dumps.rs: not
a tool process, argv carries the iteration count; run.rs merges the shim record of that process
with `merge_shim`).

## Decision
The record of a capture is chosen by identity, never by size while identity is known.
1. Where the harness writes `shim-dump.jsonl`, it knows the measured process of the point: write
   that process's shim record only (the same process `merge_shim` used for the series). If the
   measured process has no shim record with stacks, write none for that point and say so in the
   run's limitations; do not fall back to another process's record.
2. The reader must also be right for files it did not get from step 1 (a native or Rust harness
   whose `dump.jsonl` is the shim's own and holds a build tool or a child process, older run
   directories, files given by hand): among the records with stacks drop tool processes, prefer
   the process whose argv carries the record's iteration count (the rule and helpers of
   `select_measured`; reuse them, do not copy a second version that can drift), and only when that
   still leaves several processes use the size, and then add a limitation line that names the
   choice (how many processes, which was kept). Check what `x` and the argv hold in command mode
   and in driver mode before relying on them, on real dump files.
3. Look for the same "largest wins" choice among several processes in the other readers of
   harness dump files (crates/dla-cli/src/summarize/*, crates/dla-workload/src/harness/capture.rs,
   for example the Node note near capture.rs:605) and report each: same defect (fix it the same
   way), or fine and why.
4. `dla-bench dynamic`: when a fixture fails because a label or confirmation was refused for
   missing evidence, the failure text must let a reader of the CI log see why without the
   per-fixture files (they cannot be downloaded): for each summary whose gate failed, its reasons
   (already printed), its limitation lines, and for the holder the gate names the per-capture
   values (bytes and count per capture, `-` where absent) together with the pid and the last argv
   words of the record each capture was read from, where the summary's output has them (add them
   to the summary's JSON output if that is the clean way; `dla schema --check` must still pass).
   Keep it to a few lines per failed fixture. Do not touch EXPECTED_FAILURES or what is graded.

## Rules
- Do not loosen the gate in crates/dla-cli/src/summarize/rule.rs. Do not edit bench/fixtures/**.
- Keep the tree compiling at all times: the other pass and a CI snapshot can build it at any
  moment. Finish one file's edit before you start the next, and run
  `cargo check -j 2 -p dla-cli -p dla-workload -p dla-bench` after each step.
- Every changed behaviour gets a test you SEE fail without the change (revert the hunk, run,
  restore): at least (a) a file with two processes' records where the other process holds more
  live bytes: the summary reads the measured one, for both operations; (b) the harness split
  keeps the measured process's record only; (c) the fallback's limitation line. Also replay the
  real case: copy the 17 `shim-dump.jsonl` files of the run directory above into your scratch
  dir, raise the test JVM's byte figures in p1 above the program's, and show the old binary fails
  with "holder absent from dump 1" and the new one passes (say exactly how you ran it; if the old
  binary is not at hand, build nothing extra for it and judge that part by reading).
- Disk: if `df --output=avail -B1M /` is below 3000, run `bash <sp>/prune-superseded.sh`; if still
  below 1500, wait up to 30 minutes, then stop and report. Cargo with `-j 2`, the shared `target/`
  only (never a second target dir, never a scratch copy of the workspace that builds a binary
  named `dla`: it would replace the shared one). Never kill processes by name. Scratch dir
  `<sp>/u10/` (delete only it, by literal path; never `rm -rf "$VAR/..."`). Do not delete the
  /tmp/dla-bench-dynamic-* directories of other runs.
- If a test binary vanishes or is busy ("No such file", "Text file busy"), the other pass rebuilt
  target/: rerun it. Run `rustfmt` on your own files only, not `cargo fmt --all`.
- docs/VERIFY.md: one new row in the `## M7` table (`| Item | Status | Evidence | Resolution |`,
  exactly 4 cells, escape pipes as `\|`, never nest backticks in one code span). Re-read the file
  right before the edit and add only your row. "corrected" only when its tests ran; name every
  test; say how each was checked in words that match what you did. No internal plan or package
  names (U9, U10, N1, pass, round), no model names, no attribution text, no line numbers.
- Before reporting: `cargo test -j 2 --no-fail-fast -p dla-cli --test summarize_native --test
  summarize_robust`, the unit tests of the modules you changed (`cargo test -j 2 -p dla-cli --lib
  summarize::shim_groups`, `cargo test -j 2 -p dla-workload --lib harness::`), the dla-bench tests
  if you changed it, `cargo clippy --workspace --all-targets -j 2 -- -D warnings` (a warning in a
  file of the other pass is not yours: name it and go on), `cargo fmt --all --check` (same),
  `target/debug/dla schema --check` (regenerate with `target/debug/dla schema --out schemas` if a
  serialized type changed), `python3 bench/runner/test_grade.py`, then
  `cargo build -j 2 -p dla-cli -p dla-bench` and `./target/debug/dla-bench dynamic --dla-bin
  ./target/debug/dla --parallel 1 --only <f>` for jni-order-codec, native-vendor-codec,
  rust-private-registry and node-native-addon (compare with <sp>/u8acc/dynamic.json; a worse
  outcome is a problem unless its cause is in a file of the other pass: then say which).
- Report: the cause as you verified it, each decision's state (done / not needed, with evidence),
  files changed, tests and which you saw fail, checks run, anything not done and why.
