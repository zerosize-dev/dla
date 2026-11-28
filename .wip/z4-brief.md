# Round Z4: the verifier sees what ran in the measured program

Repository /home/user/dla on `main` with uncommitted round Z changes (Z1 to Z3, see `git diff`; keep
them). Same rules as /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/wip-notes/z-brief.md
(build env `CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_PROFILE_TEST_DEBUG=line-tables-only CARGO_INCREMENTAL=0`;
the disk has about 1.3 GB free: run only the test targets you touch; no commit, push, stash, branch switch,
`cargo clean`; no Claude or model names; never name a third-party project or its APIs in the repository;
every new test fails under a mutation, marker count 0 after restore; plain short English).

## Why (measured)

On a real Rust run's bundle the verifier was rerun 6 times. With the round Z rules it said `N-SCALE`,
`N-SCALE`, `L-COLLECT`; before them `N-HWM`, `N-HWM`, `N-INTENT`. Every non-leak reading rested on one
false premise: "the collector thread drains the ring every interval". In the measured program no consumer
was ever started. The coverage of both drivers behind the growth shows it: the producer functions ran
(`commit_collect`, `force_send_command`, `send_command`), the consumer function (the one that drains the
ring) never ran. The bundle gives the verifier neither the drivers nor what ran, and the verifier must not
read the run directory outside the bundle, so it cannot check whether a drain it assumes runs.

## Change

1. `dla bundle` (`crates/dla-cli/src/bundle_cmd.rs`): for the harnesses of `harness_list` (the harnesses the
   evidence comes from; at most 4, those whose series or captures are cited by the bundle's evidence first,
   then the others in id order; say in `index.json` how many were left out), write under `harnesses/<H>/`:
   - `driver/<rel>`: the driver files with the generated target block removed (`without_target_block`) and
     scrubbed like every other text (`bundle_bytes(.., needles, false)`), files over `MAX_DRIVER_BYTES`
     skipped (named in `omitted`). Reuse the entry-path code; do not duplicate it (factor a helper).
   - `ran.txt`: the project functions that ran in the measured program of that harness: the union of the
     `functions` of the harness's coverage records (the `executed*.json` files `dla trend` reads for its
     series: find the reader and reuse it), without the driver's own generated file(s), one line per function
     `<file>:<line> <name>`, sorted and deduplicated, scrubbed like other text, capped (a const, e.g. 3000
     lines; when cut, a last line that says how many were left out). Its first line says what it is, e.g.
     `# project functions that ran in the measured program of H5 (coverage of 3520 units; a function not
     listed did not run there)` with the units when the record has them (`fresh_units`), else without.
     JVM records list loaded classes, not functions: write them as `ran-classes.txt` with the same header
     shape ("classes loaded"). A runtime or run with no coverage record (native, Python before 3.12, a
     failed coverage run): no file, and the harness entry says `"ran": "not recorded: <why>"`.
   - The harness entries of `index.json` gain `driver` (the files), `ran` (the file, or the not-recorded
     text) and `functions_ran` (the count).
   The finding id, site and writers must stay scrubbed as now (the verifier stays blind to the claim).
2. `agents/verifier.md`: in step 1, read `harnesses/<H>/` too. Add to the rules in step 2 (keep the round Z
   rules; one short rule): a reading that needs code to run (a consumer, a drain, a flush, a release, a
   periodic task) must find that function in the `ran.txt` of a harness behind the growth. A function not
   listed there did not run in the measured program, so what it would remove stayed. Read the driver to see
   what the units do and what they never call. When no `ran.txt` exists, say the reading is not checked.
3. The skill text that describes the bundle (grep `skills/` and `agents/` for `entry-path/` or `index.json`):
   add one sentence on `harnesses/<H>/` where the bundle is described.
4. `docs/VERIFY.md`: one Z4 row after the Z3 row, 5 cells like the Z rows (claim | `corrected` | what changed
   | the check | limits). Use the measured facts above in general words (no project or API names).

## Tests

dla-cli integration test(s) near the existing bundle tests (grep `excerpt.txt`, `entry-path/` in
`crates/dla-cli/tests/`): a run with a driver harness behind the evidence and a coverage record: the bundle
has `harnesses/<H>/driver/...` without the target block (the finding id, the site and the target block
text are absent), `ran.txt` with the project functions and without the driver's own functions, the header
line, `index.json` `ran`/`functions_ran`; the cap with its count line; a harness with no coverage record
says `not recorded`; at most 4 harnesses with the left-out count. Mutations for each. Run fmt, clippy
(-D warnings), `cargo test -p dla-cli --bin dla`, the bundle integration targets you touched,
`cargo test -p dla-cli --test skill_text` if a skill text changed, and `schema --check` if a schema changed.

Final message: files changed, what each change does, tests and mutations, command results, anything not done.
