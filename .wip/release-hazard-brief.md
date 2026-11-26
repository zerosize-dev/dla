# Builder Q: a leak whose own drivers disagree is not confirmed until the difference is tested

Repository: /home/user/dla. Work in the working tree. Do NOT commit, push or switch branches.
Another builder (R) works in the same tree at the same time on `dla run abort`, the Stop hook
(`crates/dla-cli/src/gate_hooks.rs`), `crates/dla-cli/src/run/abort.rs` and the verifier after an
abort; R may also touch `crates/dla-core/src/gate.rs` and the report. Do not edit abort.rs or
gate_hooks.rs. In gate.rs and the report code, add your own functions and items and do not
reformat or move R's; re-read a file just before you write it. The tree also holds an uncommitted
one-line test fix in crates/dla-sample/tests/os.rs: leave it. Add one row per cause at the end of
docs/VERIFY.md (re-read it just before you write; R adds rows too). Do not touch harness/,
bench/fixtures/ or any fixture README or answer key. Do NOT edit SKILL.md.

## What a real run showed (do not name the project in the repository)

An unattended run on a JVM logging library confirmed 35 leaks. Ten of them are explained as "an
encoder dropped without `stop()` keeps its per-instance thread-local holder reachable from the
calling thread; grows only with unstopped units (H11), not with stopped ones (H10 all-stop)". The
model had written two drivers made for the same finding: H10 calls `stop()` on every unit and its
alloc-test favored non-leak; H11 skips `stop()` on half the units and its alloc-test favored leak.
H11's alloc-test was newer, and the ledger lets the newest alloc-test on a harness about the
finding decide, so `dla ledger confirm` passed. The model's own explanation (recorded with
`dla ledger explain F --removal conditional --remover <file:line>`, the `close()` that `stop()`
runs) says the holder is released on the path the API requires. A caller that skips a release
the API requires is misusing it; that is not a leak of the library. The library's documented leak
at that holder is a different one (it grows with `stop()` called, under a condition the drivers
did not create), and the run never measured with the release called under other conditions.

Two general holes:
1. Two drivers made for one finding disagreed (one non-leak, one leak), and the confirm went
   through on the newer one without anything naming or testing the difference between them.
2. There is no non-leak hypothesis for "grows only while the caller skips a release the API
   requires", so the with-release driver's flat result had no way to decide anything.

## The change

1. **A non-leak archetype for a skipped release.** Add `N-RELEASE` to data/archetypes.toml (read
   the N-* entries and how archetypes are loaded, listed and validated first; every place that
   lists the N-* ids, such as the SIBLING_REQUIRED refusal text in machine.rs, must include it).
   Summary: "Growth only while the caller skips a release the API requires (stop, close, dispose,
   unsubscribe, cancel); with the release called on every unit, the series is flat." Its sibling
   names the release: `dla ledger add --sibling-of F --origin hypothesis --hypothesis N-RELEASE
   --remover <file:line> --refute-if "..." --evidence <id>` (the remover line is the line that
   releases the holder; reuse the `--remover` notion of `dla ledger explain` if it fits).
   - It is decided only by an alloc-test on a harness made for the finding (the existing
     "made for" rule: Target/EntryPath, `not_made_for`) on which the remover line RAN. Find how
     line counts are read per runtime (crates/dla-workload/src/harness/lines.rs for Rust, Go,
     Node; the Python agent's `line_counts` in crates/dla-core/src/harness.rs; find whether the
     JVM records line or method coverage). Where the runtime gives a count, require count > 0 for
     the remover line. Where it cannot count the line, accept the harness with a recorded note
     "remover run not shown" that the report and `dla ledger show` print. Say in the report which
     runtimes check.
   - Confirm N-RELEASE: such an alloc-test favoring non-leak. Refute N-RELEASE: such an
     alloc-test favoring leak (the holder grows with the release called: a leak of the project).
     Any other evidence is refused with a fix naming that alloc-test.
   - Once N-RELEASE is confirmed, the root is refuted by that same alloc-test (it favors
     non-leak on a harness made for the finding, which is a discriminator) or closed
     `unresolved` with the remedy "grows only while callers skip <remover>"; check what the
     ledger allows and say which you chose. It must not stay confirmable as a leak.
2. **Drivers that disagree hold the confirm.** `dla ledger confirm F` (leak polarity) is refused
   with a new code `drivers-disagree` when, among the newest alloc-test per harness made for F's
   family, one favors leak and one favors non-leak, and no non-leak sibling of F's root added
   after the older of the two has been refuted by an alloc-test newer than both, on a harness made
   for the finding that is not the leak-favoring one. (A rerun of the flat harness with a longer
   `--extra` that now favors leak settles it; so does a third driver.) The refusal names both
   harnesses and their evidence ids and asks the one question: "what does <flat H> do that
   <leak H> does not?" Its fix offers, in this order: if the flat driver calls a release the
   other skips, `dla ledger add --sibling-of <root> --origin hypothesis --hypothesis N-RELEASE
   --remover <file:line> ...` and an alloc-test on a driver that calls it on every unit (for a
   thread-local or per-thread holder: also from the pooled or worker threads the project's
   callers use); otherwise the sibling that names the difference, refuted by a new alloc-test as
   above. Keep the text short; every command runnable as printed except `<file:line>` and the
   `--refute-if` text.
3. **An explanation that names a conditional remover asks for the release run.** When a leak
   finding's explanation says `removal: conditional` with a remover, and no N-RELEASE sibling of
   its root is refuted: (a) before confirm, `dla ledger confirm` is refused with `release-untested`
   and the fix of item 1; (b) after confirm (explanations usually come after), the final gate
   lists an item `release-untested` for it with the same fix, and if N-RELEASE is then confirmed
   the report and the `Findings:` line count the finding apart from leaks ("grows only while
   callers skip the release: N"), never as a confirmed leak. Find how `Verified: DISAGREES` and
   "non-leak explanations" are counted in the report and follow that pattern. The ledger has no
   confirmed -> refuted move; do not add one unless you find that the report cannot be honest
   without it (then say why).
4. **Bench policy.** If bench/runner/src/dynamic/policy.rs can meet `drivers-disagree` or
   `release-untested` on a fixture (it writes one driver per finding as a rule; check), make it
   follow the fix mechanically. Every leak fixture must still confirm its leak and every
   non-leak twin stay non-leak.
5. **Docs.** stages.md: one sentence each for 1, 2 and 3 where confirmation and non-leak
   siblings are explained. VERIFY rows.

## Tests

- dla-core unit or ledger tests: N-RELEASE is listed and validates; confirm of N-RELEASE with an
  alloc-test on a harness where the remover line ran (count > 0) and favored non-leak passes; on a
  harness where the line count is 0 it is refused; with an unrelated record it is refused.
- CLI (find the confirm tests: grown_chain.rs, ledger tests, alloc tests): the scene above, built
  from records (H10 non-leak older, H11 leak newer, both made for F): confirm F is refused
  `drivers-disagree` naming H10 and H11; after an N-RELEASE sibling confirmed by H10 (remover
  line ran), F cannot be confirmed and is refuted or closed as you chose. A variant where a new
  alloc-test on H10 (rerun) favors leak and refutes a named sibling: confirm passes.
- One harness only, or both favoring leak: confirm passes as before (no new refusal).
- `release-untested`: a finding explained `removal: conditional` before confirm is refused; after
  confirm the final gate lists the item; with N-RELEASE confirmed the report counts it apart.
- Mutation checks: drop the disagreement check (the scene test fails); let any record decide
  N-RELEASE (the count-0 test fails); drop the release-untested gate item (its test fails).

## Checks to run

`cargo test -p dla-core --lib`, `cargo test -p dla-core --tests`, `cargo test -p dla-cli --bin
dla`, the dla-cli test files you touch plus `grown_chain`, `grown_dump`, `gate_flow`,
`alloc_test`, `report` tests (whichever exist), `cargo test -p dla-bench --lib`, then `cargo
clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all --check`. Then the dynamic
acceptance on the whole set: `./target/debug/dla-bench dynamic --dla-bin ./target/debug/dla
--parallel 2 --json <scratch file>` from the repository root (expected: 63 pass and 1 expected
failure, `rust-tree-parent-links`). Every non-leak twin must stay non-leak, every leak fixture
must still confirm its leak.

## Rules

Shared `target/` only (R builds in it too; cargo waits on the lock, that is fine). Disk is tight:
before building check `df -h /`. Under 3 GB free, delete stale hash dirs older than 120 minutes
under the literal paths /home/user/dla/target/debug/build/dla-cli, .../dla-core, .../dla-workload
and .../dla-bench with `find <literal path> -mindepth 1 -maxdepth 1 -type d -mmin +120 -exec rm
-rf {} +`, and NEVER while a cargo test or a dla-bench run of yours or R's is running. No
`rm -rf "$VAR/..."`. Never kill processes by name. Never skip or weaken a test. No project names,
attribution lines or model names in the repository. Scratch:
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u150/.

## Report (final message)

Files changed; the rule as implemented for each item (the exact conditions, the codes, the route
texts, which runtimes check the remover line and the fallback, what happens to the root after
N-RELEASE is confirmed and why); tests with what each proves; mutation checks; every command with
pass/fail counts; dynamic results; existing tests whose expectations changed and why; anything you
are unsure of.
