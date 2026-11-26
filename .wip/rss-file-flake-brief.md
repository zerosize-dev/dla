# Builder P: file-backed resident memory does not read as a leak on a flat run

Repository: /home/user/dla. Work in the working tree. Do NOT commit, push or switch branches.
Another builder (O) works in the same tree at the same time on dump leads (grown.rs, gate.rs,
summarizers, ledger, bench policy): do not edit those files. Your area is
`crates/dla-cli/src/alloc/` (judge.rs, pick.rs), `crates/dla-cli/src/alloc_cmd.rs` and
`crates/dla-cli/tests/alloc_test.rs`. If you need to change anything else, say so in the report and
keep it minimal. Add one row at the end of docs/VERIFY.md (re-read it just before you write; O adds
rows too). Do not touch harness/, bench/fixtures/ or any fixture README or answer key.

## The problem

`crates/dla-cli/tests/alloc_test.rs`
`a_flat_file_backed_column_that_passed_the_gate_reads_non_leak_on_a_flat_run` failed on the Ubuntu
CI runner four times in two days, on different trees. It passes locally almost always. The fixture
is a bounded (flat) Python program. The test edits the first trend so that only `os.rss_file`
passes the gate, runs an unbound `dla alloc-test H1`, and asserts the result does not favor leak.
The CI failure output (one of the four, verbatim JSON from the test's panic):

{"columns":[{"favors":"leak","name":"os.rss_file","plateau":"growth-continues","purge":"retained"},
{"favors":"leak","name":"os.rss","plateau":"growth-continues","purge":"partial"},
{"favors":"non-leak","name":"os.vm_hwm","plateau":"plateau","purge":"partial"}],
"decided_by":"os.rss_file","favors":"leak", ...}

So on the runner, during the extra cycles of a flat Python run, file-backed resident memory
(`os.rss_file`: pages of mapped files, such as shared libraries and the interpreter's lazily
touched modules) kept rising, and with it `os.rss`. The alloc-test called a bounded program a
leak. That is a false positive any user can get, not just a test problem: file-backed resident
pages are the page cache of mapped files, not memory the program allocated and holds.

Read first: judge.rs (`plateau`, the purge rules, `combine`), pick.rs (`purge_does_not_apply` and
which columns vote; an earlier builder made the purge "not apply" to os.rss_file and others), and
how `os.rss`, `os.rss_anon` and `os.rss_file` are sampled (crates/dla-sample or the harness
samplers: find them).

## The change

1. File-backed resident memory never decides `leak` by itself. A leak is memory the program
   allocated and keeps, which is anonymous memory. A growing `os.rss_file` is pages of files
   touched for the first time, or kept warm. It can make a verdict inconclusive, or agree with a
   leak another column shows, but it is never the deciding column of a leak. Decide this from the
   code, and write the rule in the doc comment.
2. `os.rss` growth that the file-backed part explains is not a leak either. When the run samples
   both `os.rss_anon` and `os.rss_file`, and the anonymous part did not grow over the extra window
   (plateau, or a rise within its noise) while the file-backed part did, `os.rss` cannot favor
   leak. Its growth is the file-backed growth. When `os.rss_anon` is not sampled (find on which
   platforms it is), say what the rule falls back to and why.
3. Keep every existing verdict that does not hinge on file-backed growth: a real leak fixture whose
   anonymous memory grows must still read leak. Read the existing tests of these columns first
   (`alloc_cmd` unit tests, judge tests, alloc_test.rs) and never weaken one.
4. The test: make it deterministic where it can be, by also feeding the judge a recorded series
   in a unit test (anon flat, file-backed rising, os.rss rising by the file-backed amount: not
   leak). Keep the CLI test. Its assertion (`favors != leak`) must now hold whatever the runner's
   file pages do, because the rule no longer lets those columns decide leak.

## Tests

- judge/alloc_cmd unit tests: the CI case above, built as series (anon flat, file rising, rss
  rising): favors is not leak, decided by neither file column. Anon rising with file flat: leak, as
  before. Both rising: leak, decided by anon or rss.
- Mutation checks: let os.rss_file decide leak again (the unit test of the CI case fails); drop the
  "rss explained by file-backed" rule (a unit test fails).

## Checks to run

`cargo test -p dla-cli --bin dla`, `cargo test -p dla-cli --test alloc_test` five times in a row
(report each), the other alloc tests you find, `cargo clippy --workspace --all-targets -- -D
warnings`, `cargo fmt --all --check`. Then the dynamic acceptance on the fixtures whose verdicts
depend on os.* columns: at least `--only python-`, `--only jvm-pool`, `--only native-` and
`--only rust-`, with `./target/debug/dla-bench dynamic --dla-bin ./target/debug/dla --parallel 2
--only <substring> --json <scratch file>`. Every leak fixture must still confirm its leak and every
non-leak twin must stay non-leak.

## Rules

Shared `target/` only (O builds in it too; cargo waits on the lock, that is fine). Disk is tight:
under 3 GB free, delete stale hash dirs older than 120 minutes under the literal paths
/home/user/dla/target/debug/build/dla-cli, .../dla-core, .../dla-workload and .../dla-bench with
`find <literal path> -mindepth 1 -maxdepth 1 -type d -mmin +120 -exec rm -rf {} +`. No
`rm -rf "$VAR/..."`. Never kill processes by name. Never skip or weaken a test. No project names,
attribution lines or model names in the repository. Scratch:
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u148/.

## Report (final message)

Files changed; the rules as implemented (which columns, the conditions, the fallback without
os.rss_anon); tests with what each proves; mutation checks; every command with pass/fail counts
(the five alloc_test runs each); dynamic fixtures with results; existing tests whose expectations
changed and why; anything you are unsure of.
