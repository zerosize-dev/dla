# Builder M: heap dumps of a run stay within a size limit, and a capture checks free disk first

Repository: /home/user/dla. Work in the working tree. Do NOT commit, push or switch branches.
The tree has uncommitted, checked work of another builder that is in CI now. Do not revert or
reformat files you did not write. Add one row at the end of docs/VERIFY.md (re-read it first).
Do not touch bench/fixtures/ or any fixture README or answer key.

## What a real run showed (do not name the project in the repository)

An unattended run on a Node library, 3-hour budget. Each harness it made was run once with
`dla harness run H<n> --mode fresh --capture alloc,heap --dumps 9`, and `dla summarize heap-graph`
read the nine V8 snapshots. Each snapshot was about 110 MB, so each harness kept about 1 GB under
`harness/H<n>/out/<label>/capture/<pid>/`. Five harnesses in 12 minutes used 4 GB, and free disk
fell to 1.1 GB. A 3-hour run makes 20 to 40 harnesses, so it would have needed tens of GB. A full
disk breaks every later write of the run (ledger, series, captures) and the model reads the errors
as findings about the project. JVM heap dumps (HPROF) are the same size class.

What the tool does today (read these first):
- `crates/dla-workload/src/harness/run.rs` `clear_stale_capture_files`: at the start of a capturing
  run of a harness, the capture directory of that harness's point is removed. So within one
  harness only the newest set is kept. Across harnesses nothing is bounded.
- `crates/dla-workload/src/harness/capture.rs`: `gather`, `prune` (other pids of the same point).
- `crates/dla-core/src/harness.rs` `CaptureFile` (kind, path, label, seq, x, role, bytes, ms,
  captured_at), written to `harness/<H>/captures.json`.
- `crates/dla-core/src/grown.rs` `made_from_capture_of`: a summary record's raw file lists the
  sha256 of each input it read (`captures` array). The harness is "about" a finding through a
  capture when a summary that names the finding read a file of that harness's captures.json. It
  hashes the capture files on disk on every call (a 1 GB set is read again each time).
- `crates/dla-cli/src/attach_cmd.rs`: `free_disk`, and the `DLA_ATTACH_FREE_DISK` override that
  tests use.

## The change

1. **Digests are stored.** `CaptureFile` gets `sha256: Option<String>` (serde default, skipped
   when None), filled in by `gather` for every file it lists. `made_from_capture_of` uses the stored
   digest when present and hashes the file only when it is absent (an older captures.json). This
   also stops a large capture set from being read on every call.
2. **A size limit for the large captures of a run.** After a capturing `dla harness run` writes
   its captures.json, the tool adds up the bytes of the heap dumps and histograms (the kinds
   `CaptureKind::Heap` and any other kind whose files are routinely over a few MB, decide from the
   enum and say which) listed in the captures.json of every harness of the run directory (the
   worker's own slice when a worker runs it; read how worker run dirs are laid out). When the total
   is over the limit, it deletes whole capture sets of other harnesses, oldest `captured_at` first,
   until the total is under it. A capture set is the files of one harness and one label. Two rules
   are never broken:
   - The set the current run just wrote is never deleted.
   - A set that no summary has read (no summary raw file lists one of its digests) is never
     deleted. The capture-first rule makes the run summarize every set, so an unread set is work
     the model still has to do.
   The limit is `DLA_CAPTURE_KEEP_MB` (default 2048). The deleted files stay listed in captures.json
   with `pruned: true` (new field, serde default false), with their digest, so the capture link of
   `made_from_capture_of` and the provenance of every summary still hold. The run's JSON output
   lists what was deleted (`pruned_captures`: harness, label, files, bytes) and says it in one
   `_attention` line ("deleted the heap dumps of H2 p1-fresh (980 MB), read by E41, to stay under
   DLA_CAPTURE_KEEP_MB=2048").
3. **Free disk before a capture.** Before a capturing run starts any process, estimate what it
   will write: the number of heap dumps it takes (read how `--dumps` and the default are turned into
   the number of dumps per point and per key mode) times the size of the largest heap dump of this
   harness's previous captures, else of any harness of the run, else no estimate. Delete what rule
   2 may delete first. Then refuse when free disk (the run directory's volume) minus the estimate is
   under `DLA_DISK_FLOOR_MB` (default 1024). Use a refusal code that exists if one fits, else add
   `disk-low` with a schema entry where codes are listed. The message gives free disk, the estimate
   and the sets that could not be deleted because nothing read them. The fix names the summarize
   command of the oldest unread set (from that harness's last run output, or built from its
   captures.json), and `--dumps <n>` with the largest n that fits (never under 3; if 3 does not fit,
   say to free disk). Tests set free disk with an override env (follow `DLA_ATTACH_FREE_DISK`; name
   it `DLA_FREE_DISK_BYTES` and use it in both places if that is cleaner, without breaking attach).
4. **A summarize of a deleted file.** `dla summarize heap-graph` (and any summarizer that takes
   capture paths) given a path that captures.json marks `pruned` refuses with the fix
   `dla harness run <H> --capture heap` (its own options from the manifest where the tool already
   prints them) instead of a plain file-not-found.
5. **Docs.** One sentence in skills/detect-leak-anywhere/references/stages.md where `--capture` and
   `--dumps` are explained (find it): the run keeps its heap dumps under DLA_CAPTURE_KEEP_MB by
   deleting the oldest sets a summary already read, and refuses a capture that would leave less than
   DLA_DISK_FLOOR_MB free. Do NOT edit SKILL.md. VERIFY row.

## Tests

dla-core / dla-workload unit tests for the choice of sets to delete (pure function over a list of
sets with bytes, captured_at, read or not, current or not): oldest first, only read sets, never the
current one, stops as soon as the total fits, deletes nothing when under the limit.

CLI tests (crates/dla-cli/tests; find the capture tests, for example harness_capture_node.rs, and
how they fake or run captures; prefer fake captures.json files and fake dump files of chosen sizes,
written under a real run directory, over real Node runs where the rule does not need a real run):
1. Three harnesses with read 900 MB-style sets (use small files and a small DLA_CAPTURE_KEEP_MB),
   then a fourth capture: the oldest sets go until the total fits, captures.json marks them pruned
   with their digests, `pruned_captures` and `_attention` say it.
2. An unread set is never deleted, even when that leaves the total over the limit.
3. Free disk under the floor with nothing deletable: the capture is refused with the summarize
   command of the unread set and a smaller `--dumps`, and no process was started.
4. A summarize of a pruned file is refused with the harness run fix.
5. `made_from_capture_of` still links a summary to a harness whose set was pruned (the stored
   digest), and does not read the files when the digest is stored.
6. One real Node capture run (an existing fixture used by harness_capture_node tests) still writes
   digests that match the files.

Mutation checks: delete unread sets too (test 2 fails); skip the free-disk check (test 3 fails);
hash from disk instead of the stored digest after pruning (test 5 fails).

## Checks to run

`cargo test -p dla-core --lib`, `cargo test -p dla-workload`, the dla-cli test files you touch and
`cargo test -p dla-cli --test harness_capture_node` (and the JVM capture tests if they exist and the
tools are installed), then `cargo clippy --workspace --all-targets -- -D warnings` and
`cargo fmt --all --check`. Then the dynamic acceptance on the fixtures that take heap captures (find
them: Node and JVM fixtures whose policy uses `--capture heap`; at least three), with
`./target/debug/dla-bench dynamic --dla-bin ./target/debug/dla --parallel 2 --only <substring>
--json <scratch file>` once per substring, from the repository root. Report pass/fail per fixture.
A failing fixture is diagnosed, never changed.

## Rules

Shared `target/` only. Disk is tight here: before building, check `df -h /`. If it shows less than
3 GB free, delete stale hash dirs older than 120 minutes under the literal paths
/home/user/dla/target/debug/build/dla-cli, /home/user/dla/target/debug/build/dla-core,
/home/user/dla/target/debug/build/dla-workload and /home/user/dla/target/debug/build/dla-bench with
`find <literal path> -mindepth 1 -maxdepth 1 -type d -mmin +120 -exec rm -rf {} +`. No
`rm -rf "$VAR/..."`. Never kill processes by name. Never skip or weaken a test. No project names,
attribution lines or model names in the repository. Scratch files go under
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u143/.

## Report (final message)

Files changed; the rule as implemented (which kinds count, the limit, the order, the refusal text
and fix); the tests with what each proves; the mutation checks; every command run with pass/fail
counts; dynamic fixtures with results; existing tests whose expectations changed and why; anything
you are unsure of (in particular how workers and their run directories are counted).
