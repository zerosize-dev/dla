# The open fixtures, first fix pass (orchestrator brief)

Repo /home/user/dla, branch main at 4aae86f. Do NOT commit or push.
sp = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad

Eight fixtures are parked in bench/fixtures-open/ because DLA does not pass them (read its
README.md). Each was diagnosed and the diagnosis verified independently:
`<sp>/u13/results/<fixture>.json` holds both (`diagnosis` and `verdict`; where they disagree the
`verdict` is the later and better-checked one: read its `corrections`, `better_proposal`,
`recommended_change` and `fixtures_at_risk` in full). The long form and the logs of each
diagnosis are under `<sp>/u13/<fixture>/`, the verifier's files under `<sp>/u13v/<fixture>/`.
This pass makes the product changes that need no decision of the owner. The goal is a product
that is right for real projects; a fixture passing is the check, not the aim.

## Rules (all packages)
- Never edit anything under bench/fixtures/ or bench/fixtures-open/ (no answer key, README or
  project file), and do not move fixtures: a later step does that. To run a parked fixture, copy
  it to a scratch fixtures directory (with its partner from bench/fixtures/ and the partner's
  `pair = "<fixture>"` line added in the copy, when its expected.toml has a `pair` line) and pass
  `--fixtures <dir>` to dla-bench. To run with changed files under data/, they are read from
  `--repo-root` (default `.`), so the working tree's data is used as it is.
- Do not edit EXPECTED_FAILURES in bench/runner/src/dynamic_cmd.rs, and no bench policy change
  (bench/runner/src/dynamic/**) unless your package says so. Never make a rule, table row or
  code path depend on a name that only a fixture has.
- Never skip, ignore or weaken a test. Every changed behaviour gets a test you SEE fail without
  the change (revert the hunk, run, restore); say which you saw fail and which you judged by
  reading.
- Two packages edit the tree at once. Each package lists its files; a file of another package is
  not yours. Shared files (docs/VERIFY.md, crates/dla-core/src/series.rs, docs and references
  under skills/): re-read right before each edit, change only your own lines, never revert what
  you did not write. Keep the tree compiling at all times (finish one file before the next;
  `cargo check -j 2 --workspace --all-targets` after each step). Run `rustfmt` on your own files
  only, never `cargo fmt --all`.
- Cargo with `-j 2`, the shared `target/` only (never a second target dir, never a scratch copy
  of the workspace that builds a binary named `dla`). For bench runs copy target/debug/dla and
  target/debug/dla-bench into your scratch dir after building and use the copies. Never kill
  processes by name. Scratch dir `<sp>/u14/<package>/` (delete only it, by literal path; never
  `rm -rf "$VAR/..."`). If a test binary vanishes or is busy, the other package rebuilt target/:
  rerun it.
- Disk: if `df --output=avail -B1M /` is below 3000, run `bash <sp>/prune-superseded.sh`; if still
  below 1500, wait up to 30 minutes, then stop and report.
- docs/VERIFY.md: one new row per changed behaviour in the `## M7` table (`| Item | Status |
  Evidence | Resolution |`, exactly 4 cells, escape pipes as `\|`, never nest backticks in one
  code span). "corrected" only when its tests ran; name every test; say how each was checked in
  words that match what you did. No internal plan or package names (F-a, pass, round, refix,
  "the checker"), no model names, no attribution text, no line numbers, no scratch paths.
- The skill's SKILL.md stays within the limit that `cargo test -p dla-cli --test skill_text`
  checks. Reference files under skills/detect-leak-anywhere/references/ get the new behaviour in
  the section where a reader looks for it. Never copy text from third-party sources.
- Before reporting: `cargo test -j 2 --no-fail-fast` for every crate you touched (all test
  binaries), `cargo clippy --workspace --all-targets -j 2 -- -D warnings`, clippy for
  `--target x86_64-pc-windows-gnu -p dla-core -p dla-probe -p dla-report` if you touched one of
  them, `cargo fmt --all --check` (a failure only in another package's file is not yours: name
  it), `target/debug/dla schema --check` (regenerate with `target/debug/dla schema --out schemas`
  after rebuilding if a serialized type or its doc changed), `python3 bench/runner/test_grade.py`,
  `cargo test -j 2 -p dla-cli --test skill_text`, then the bench runs your package names
  (`dla-bench ... static` in both modes, with and without `--archetypes off`, and
  `dla-bench ... dynamic --parallel 1 --only <fixture>`; compare fixtures of bench/fixtures with
  `<sp>/u11acc/dynamic.json`, the last full run: a worse outcome is a problem).
- Write your final report to `<sp>/u14/<package>/report-<step>.md` as well as returning it:
  each item's state (done / not needed / not done, with evidence), files changed, tests and
  which you saw fail, checks run with results, bench runs with results, what is unverified.

## F-a: a registration leak can be labelled on its allocation stack (go-ticker-legacy)
Files: data/archetypes.toml, crates/dla-cli/tests/ledger_m6.rs, crates/dla-core/tests/archetypes_data.rs,
skills/detect-leak-anywhere/references/go-archetypes.md, bench/README.md, your VERIFY row.
1. In data/archetypes.toml give L-REGISTER the further alternative `["growth-series",
   "alloc-stack"]`, last in its `requires` list (L-HANDLE and L-COLLECT accept the same pair).
2. Tests as the verdict's recommended change lists them (a Go finding labelled L-REGISTER on
   growth-series + alloc-stack with the evidence named explicitly; growth alone still refused;
   the shipped row ends with the new alternative). The existing test that appends a synthetic
   L-REGISTER row hid the gap: keep it, and say in its comment what it does and does not show.
3. go-archetypes.md: say that a timer or ticker alone before Go 1.23 is the known-leaks row's
   case (L-REGISTER, registration-table), beside the L-WORKER fingerprint.
4. bench/README.md: where it describes this fixture's label refusal, say what holds now (label
   accepted on growth-series + alloc-stack; with archetypes off the leak is still unconfirmed,
   because the holder comes from the known-leaks row).
Bench: go-ticker-legacy with its partner (both pass); the six go-* fixtures of bench/fixtures and
the fixtures that label L-REGISTER (node-socket-heartbeat, node-socket-heartbeat-cleared,
node-gateway-merge, python-ingest-two-leaks).
Not in this package: the known-leaks row counting only when the probe lists it true.

## F-b: Python census and agent (python-frozen-gc; static part of python-zstd-options-3144)
Files: crates/dla-census/src/lang/python.rs (and usage.rs if removers are recorded),
harness/python/sitecustomize.py, harness/python/README.md, crates/dla-core/src/series.rs
(only your `column_meta` line and its test), tests in crates/dla-workload/tests and
crates/dla-cli/tests, skills/detect-leak-anywhere/references/python.md, your VERIFY rows.
1. Census: a `gc.freeze()` call inside a function is listed (kind `gc-freeze`, holder
   cycle-only, region heap); a module-level call is not, `gc.collect()` and `gc.set_threshold()`
   are not (python-graph-churn calls `gc.set_threshold` and must gain no candidate).
2. Agent: wrap `gc.freeze` and `gc.unfreeze` without ever raising into the target; attribute the
   rise of `gc.get_freeze_count()` to the calling site; write the group `py.gc_frozen` with a
   share per site that sums to the current freeze count (not a cumulative sum: frozen objects
   freed by reference counting leave the count); nothing when freeze was never called; a
   limitation line when the freeze count is above zero (py.gc_objects and the type counts
   exclude the permanent generation).
3. Catalog: `py.gc_frozen` is a count, counter, heap column (without the entry the bound
   alloc-test applies the byte minimum to an object count and reads plateau).
4. Census vocabulary: native codec contexts are listed as kind `handle`, holder
   native-handle/global-ref, off-heap, like the existing `mmap.mmap` and `sqlite3.connect` rows:
   constructors ending in `compressobj`, `decompressobj`, `BZ2Compressor`, `BZ2Decompressor`,
   `LZMACompressor`, `LZMADecompressor`, `ZstdCompressor`, `ZstdDecompressor`, `ZstdDict`
   (anchored; `zlib.crc32`, `zstd.compress`, `zstd.decompress` are not listed).
5. Tests as the two verdicts list them; docs: the signature of the frozen-garbage leak and its
   fix in python.md, the agent's new group in harness/python/README.md.
Bench: python-frozen-gc passes static (both modes) and dynamic; python-zstd-options-3144 with
its partner passes static (both modes; its dynamic needs a decision that is not in this pass:
report what its dynamic run reads now); all eight python-* fixtures of bench/fixtures keep their
outcome (python-graph-churn first).

## F-c: Node probes and registration holders (node-signal-compose-24)
Files: harness/node/dla-preload.js, harness/node/README.md, crates/dla-parsers/src/node/holders.rs,
crates/dla-cli/src/summarize/node_heapsnapshot.rs, tests in crates/dla-workload/tests,
crates/dla-cli/tests and crates/dla-parsers, the Node sections of the skill references, your
VERIFY rows.
1. A probe whose root cannot be measured says so and writes no column: a function, a leaf or
   opaque value, or an object of a class (prototype neither Object.prototype nor null; not an
   Array, Map or Set) with no own keys, which is what a class with only `#private` fields looks
   like. Today such a probe reads a constant 1 and a true finding is refuted on it.
2. A registration holder for FinalizationRegistry cells: cells with a live weak `target` grouped
   by target; the owner named by a bounded reverse walk from the target (rules in the verdict);
   exactly one owner or nothing project-owned; cells without a target (cleanup pending) are not
   counted. The summarize adapter gets the view `registry:finalization:<owner>`, prefers it over
   a `static:` holder of the same owner, and keeps its filter of `node:` internals.
3. When the only strictly growing holders are inside `node:` modules, heap-graph says that in
   its reasons, not "take another snapshot".
4. Tests as the verdict lists them, docs as it lists them.
Bench: node-signal-compose-24 with its partner: 24 passes static and dynamic, 22 claims nothing
(say what its F4 ends as; do not touch its README); all eleven node-* fixtures of bench/fixtures
keep their outcome (the non-leak and decoy ones first).

## F-d: Go static rules (static part of go-feed-merge, go-header-index, go-trace-context)
Files: crates/dla-backends/src/dla_rules/go.rs, chain.rs, data/detectors/go.toml,
crates/dla-core/src/static_hit.rs, crates/dla-backends/src/sweep.rs (and what reads a hit into a
candidate), crates/dla-census/src/lang/go.rs, crates/dla-census/src/usage.rs,
skills/detect-leak-anywhere/references/go.md and go-archetypes.md (your sections), your VERIFY rows.
1. One new rule, `go-slice-view-retained` (archetype "", prior about 0.3; the toml row): a slice
   parameter stored un-copied into accumulating receiver or package state (`x.f[k] = p`,
   `x.f = append(x.f, p)`, an insert call on `x.f` whose argument is `p` or a composite literal
   holding `p`), reported at the store line, only when a call site in the project passes a view
   of a larger buffer (a sub-slice expression or the result of bytes.Cut, bytes.SplitN,
   bytes.Fields and the like, also through a struct field assigned from one); silent for
   `bytes.Clone`, `slices.Clone`, `append([]byte(nil), p...)`, `string(p)`, `copy` into a fresh
   slice, a `string` parameter, and a pool `Put` fed only by buffers from `Get` or `make`
   (go-buffer-pool's pool must gain no candidate). The message names the parameter, the
   container, the feeding call and the view's origin.
2. A static hit can carry holder, owner and region (optional; none for external backends), and
   the candidate takes them. The new rule sets container-field (or static/global) and heap.
3. Go joins the existing chain-head-growth rule (`<runtime>-chain-head-growth` in chain.rs, used
   for JVM, Node and Python): a field of a long-lived struct reassigned from a call that takes
   the field's own value (`p.ctx = start(p.ctx, ...)`), reported at the assignment; quiet for a
   local, for `p.ctx = context.Background()`, `p.n = max(p.n, x)`, `p.items = append(p.items,
   x)` and a string field. The census lists such a field as container-field with the write as
   its writer.
4. Tests as the three verdicts list them.
Bench: `dla-bench static` in both modes on scratch copies of go-feed-merge, go-header-index and
go-trace-context with no external Go analyzer on PATH flags every leak; the candidates of the
six go-* fixtures of bench/fixtures are unchanged (list any difference and justify it); their
dynamic outcome is unchanged. Report what the dynamic run of the three parked fixtures reads
now; it is not expected to pass yet.

## F-e: Go command mode measures while the workload runs
Background: in command mode the Go agent wraps the project's `main` and writes its exit record
after `main` returned. State owned by a local of the workload is dead by then, so the series is
flat and the heap profile empty although the heap grew by megabytes while the workload ran
(go-header-index, go-trace-context, the second leak of go-feed-merge: figures in their verdicts).
A real command-mode run of a Go program has the same blind spot.
This package starts with a design step that another agent reviews; the fixer implements the
reviewed design (`<sp>/u14/F-e/design.md`, then `design-review.md`). Constraints for the design:
- The existing columns keep their meaning and values unless a measurement on every go-* fixture
  of bench/fixtures shows a change is harmless; the new readings are additional columns (named so
  that no existing rule mistakes them: not `.pre_gc`), typed in the catalog.
- A reading taken while the workload runs, after a collection, on a schedule that does not miss
  short points and whose value is not biased upward by the number of samples (a maximum over
  noisy samples is); with alloc capture, a heap profile taken in the run and listed in the exit
  record's captures; a limitation line when no in-run reading was possible.
- The cost of the collections is measured (run time with and without, on the largest Go fixture)
  and bounded. The verdicts of go-buffer-pool, go-gc-tuned-batch (it is about collector tuning:
  its headroom evidence must not change), go-ticker-modern, go-subscription-workers and
  go-subscription-workers-closed stay as they are.
- The trend, the choice of the primary heap column and the unbound alloc-test treat the new
  columns correctly (they have no purge reading).
- Driver mode is unchanged.
Files (after the design): harness/go/*.tmpl and README.md, crates/dla-core/src/harness.rs and
series.rs (your lines), crates/dla-workload/src/harness/dumps.rs and build.rs,
crates/dla-cli/src/alloc/pick.rs, crates/dla-cli/src/summarize/go_pprof_heap.rs (header and what
reads the in-run profile), tests, skills/detect-leak-anywhere/references/go.md (your section),
your VERIFY rows.
Bench: all six go-* fixtures of bench/fixtures keep their outcome; on scratch copies of
go-header-index, go-trace-context and go-feed-merge the series grows and the in-run profile
shows the allocating stacks. With the static rules of F-d in the tree, go-header-index is
expected to pass static and dynamic; go-trace-context and go-feed-merge are expected to stay
unconfirmed (they need a decision on the tie rule that is not in this pass): report exactly
where each stops.
