# The order of candidates, second step: build brief (one builder)

Repo /home/user/dla, branch main. The working tree holds uncommitted work of other agents (the
measurement queue, the Go capture axis, three small fixes in progress, later a Go coverage
change). Do NOT commit, push, stash, reset or revert anything. Touch only what this job needs;
never undo a change you did not write; re-read a shared file right before each edit; keep the
tree compiling at all times (other agents build the same tree; when the build is broken by a
file that is not yours, wait a minute and retry, at most ten times, then report).
sp = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad
Your scratch dir: `<sp>/u33/` (create it; delete only it, by literal path; never
`rm -rf "$VAR/..."`).

## Why
`dla` ranks static candidates and makes the agent measure them in rank order, 8 per generated
workload (`dla ledger list`, `dla harness gen --lang <rt> --driver --next`; code:
`crates/dla-core/src/rank.rs` (tiers 1 to 6), `crates/dla-core/src/queue.rs`,
`crates/dla-cli/src/ledger_queue.rs`, `crates/dla-workload/src/harness/targets.rs`). A run has
time for two to four batches. With the tiers, the documented leak of grpc-go v1.84.0 is rank
159 of 298 and that of spring-data-commons 4.1.1 is rank 325 of 479: never measured.

A research pass measured which generic static signals fix this. Its script is the reference
for this job: `<sp>/u30/dlarank.py` (run with `python3 -I`; commands `signals`, `rank`, `why`,
`top`, `fixtures`, `summary`; signal extraction in `analyse_candidate`, features in `feat`,
weights in `BASE` and `SCHEMES`). Inputs it used: finished static stages of four real trees
under `<sp>/u29/{aiohttp,grpc,spring,axios}` (lists in `<sp>/u29/<name>-list.jsonl`) and of
all 64 bench fixtures under `<sp>/u30/fx/<fixture>/`. Its result (I reran it):

| order | aiohttp `web_app.py:82` | grpc `health/server.go:49` | spring `TypedPropertyPaths.java:53` | axios `http.js:908/925` | fixtures: true sites in top 1 / 3 / 8 of 38 |
|---|---|---|---|---|---|
| tiers (now) | 1 | 159 | 325 | 29 / 30 | 21 / 31 / 37 |
| script scheme B2 | 1 | 10 | 25 | 15 / 16 | 18 / 33 / 37 |
| script scheme with rule_dla=15, rule_strong=45, life_process=20 | 1 | 10 | 13 | 14 / 15 | 16 / 32 / 37 |

The script works on regexes over source lines and was off on about 1 site in 6 where it was
checked by hand. This job puts the signals into the census (syntax tree, exact where the tree
allows) and the score into the ranking.

## What to build
### 1. Census facts (crates/dla-census; Go, Java, Kotlin where the Java path covers it, Python, JavaScript/TypeScript; Rust and C keep what they have and leave the new fields empty = neutral)
Per holder (census entry):
- `container`: real container (map, set, list, slice, queue, deque, cache type, sync.Map,
  channel used as a queue), registry-like, or not a container (scalar, pointer or struct
  reassigned as a whole, logger, function value, TypeVar/ParamSpec/NewType, a pool type such as
  sync.Pool, a literal or immutable table that is only read). A non-container that is only
  reassigned is NOT listed as a holder with "writers" any more unless a DLA rule names it
  (chain-head growth does: keep those).
- `bound`: none, weak (weak keys or values: say which), bounded (LRU with a size, ring,
  bounded channel, `maxsize=` given), and `unbounded_cache` for a cache decorator without a
  size (`lru_cache(None)`, `functools.cache`).
- `lifetime`: process (static, package or module variable, class variable), object (field of
  an instance), unit (created per call: local, closure, timer or task or handle creation site).
- `private`: the holder cannot be named outside its file, class or package (Java `private`,
  Go unexported, Python leading underscore in a module, JS not exported). Writers and removers
  of a private holder are searched only where the language lets them be: a site in another
  file matched by NAME is dropped (today `Lazy.java:66` counts as a writer of
  `TypedPropertyPaths.resolved`).
Per writer and remover site (keep `writers`/`removers` as lists of `file:line` for old
readers and add a parallel structured list):
- `kind`: insert (keyed write, put, computeIfAbsent, setdefault, Store, add, append, push,
  offer, register), reassign (the variable itself is assigned), reset (assigned a fresh empty
  container, clear()), delete (remove, delete, pop, evict), and `depth`: 0 when the operation
  is on the holder itself, 1 or more when it is on an ELEMENT of it (`delete(s.updates[k], x)`
  and `s.updates[k][x] = v` are depth 1). Census bug to fix on the way: `x[k] = make(...)`
  and a constructor's `this.x = new T()` are counted as removers today; the first is an insert
  at depth 0, the second an initialisation.
- `function` (name of the enclosing function or method) and `init`: true when it is an
  initialiser (Go `init` or package-level `var =`, a constructor or `New*`/factory of the
  owner type, a Java static initialiser or constructor, Python module top level or `__init__`,
  JS module top level or constructor).
- `param`: the inserted key or value derives from a parameter of the enclosing function (the
  parameter itself, a field or method result of it, a local assigned from it: one hop).
  `const`: the key is a literal or a named constant.
- `exported`: the enclosing function is part of the public surface (exported or public name,
  not under `internal/`, not in a `_private` module, not a package-private class).
Store these on the census entry and on the ledger annotation the ranking reads
(`crates/dla-core/src/ledger.rs` `Annotation`; every new field optional with a default so
older ledgers and census files still load; regenerate schemas with `dla schema --out schemas`).
`dla ledger annotate` callers that add writers by hand keep working.

### 2. The score (crates/dla-core/src/rank.rs)
`score(finding, annotation) -> i32`, a pure function, replaces the order inside the eligible
set: descending score, ties by today's tier, then findings with two origins first, then ledger
order. Eligibility stays what it is (tiers 1 to 4; test, example, benchmark and documentation
code demoted; extend the demoted path names with testutils-style names: a path component
containing test, mock, fake, stub, e2e, fixture, interop, example, sample, demo or bench).
Weights (the script's `BASE` with three changes: `rule_dla` 15, `rule_strong` 45,
`life_process` 20; feature definitions are those of `feat` and `analyse_candidate`):
- origin: a DLA rule hit +15, and +45 more when the holder's lifetime is process or the rule
  is about a unit (timer, task, goroutine, thread, handle, listener registration, cache
  decorator, chain-head growth, a known-leak pattern); known-leak +60; two origins +2;
  tier 5 -20.
- holder: lifetime process +20, object +3; real container +20, registry-like +10,
  non-container -80; weak -6, bounded -14, unbounded cache decorator +10, literal or immutable
  table -25, key type bounded by the program (Class, enum, type descriptor) -10, pool type -15.
- writers: at least one credible insert at depth 0 or 1 +25; no credible writer -40;
  reassign-only -35; an insert in a non-initialiser function +20; every writer in an
  initialiser (or reachable only from one, where you can tell) -30; key or value from a
  parameter +15, constant keys only -10; a writer in a public function +6.
- removers: inserts at some depth with no delete or reset at that depth or above +25 (the
  grpc case: the outer map gets entries, only inner entries are deleted); no credible remover
  at all +8; a remover on the holder itself -12; removers that were only name matches count as
  none.
- registrations (listener, handler, callback, signal): receiver is a local of the function
  -20, receiver is a parameter or a field +8.
- owner: owner object created per call -10, owner created only at start-up +5 (only where
  the census can tell; unknown = 0).
Keep one table of weights in one place with a name per signal, so `dla ledger list` can print
for each row its `score` and `why` (the three largest contributions, by name), and the
report's note above the ranked static-only findings (`TIER_NOTE`) describes the order in two
sentences. The tier stays printed.

### 3. Holders of one owner are measured together (queue.rs, harness gen `--next`)
A driver that exercises a class reaches all of its state. `--next [N]` declares the next N
(default and maximum 8) pending candidates by rank and then also every other pending candidate
with the SAME owner type (same file and the same owner path up to the last segment: fields of
one struct or class, module-level containers of one module only when they share a writer
function), up to 16 targets in all; the added ones are marked `sibling_of` in the summary, the
manifest and the driver's target block. `--probe` keeps its limit of 8. The queue's `next`
list, `run status` and the queue item show the same batch. Example that must hold:
grpc `health.server.Server.statusMap` in a batch brings `health.server.Server.updates` with it.

### 4. Measure (report every number)
After the build, with a copy of the binary in `<sp>/u33/bin/` (`DLA_PLUGIN_ROOT=/home/user/dla`):
- Static stage on fresh copies of the four trees (`cp -r <sp>/u18/<name>/proj <sp>/u33/<name>`,
  remove `.dla`, then the commands of `<sp>/u29/static.sh` adapted to your binary and paths):
  rank of each documented site, the first 16 rows of `dla ledger list` with score and why, and
  the batch that `dla harness gen --driver --next` would declare first (use `dla ledger list`
  and the queue's `next`; generate a harness only on the copy). Targets: aiohttp 1; grpc
  within 16; spring within 16; axios 908 and 925 within 16. If one is outside, report why
  (which signal the census got differently from the script) instead of changing weights to
  fit: weights change only with a reason that holds for code in general, stated in the report.
- Two hold-outs that were NOT used to choose the weights (report whatever comes out, do not
  tune on them): tokio-util 0.7.1 (`<sp>/u18/tokio/proj`, Rust; the leak is in
  `src/sync/cancellation_token.rs`) and Jetty 12.0.31 (`<sp>/u18/jetty/12.0.31/*`, Java
  sources of three modules; the leak: `GzipHandler` creates a `GzipRequest` whose `Inflater`
  is not released when the response is not deflated, a native handle, not a container; find
  the site in the sources and report its rank before and after).
- The fixture guard on all 64 fixtures (`<sp>/u30/fx_static.sh` shows how the research ran the
  static stage per fixture; answer keys are next to each fixture: read only): every true leak
  site that is within rank 8 with the tiers stays within rank 8; report top 1 / 3 / 8 counts
  before and after and every site that moved by more than 2.
- `dla-bench static` in both modes (with and without `--archetypes off`): 64/64, and the set of
  candidates per fixture unchanged except where item 1 removes a non-container holder (list
  every removed candidate and say what it was; a removed candidate that an answer key names is
  a stop: report it, do not bend the rule or the key). Baselines `<sp>/u23/static-on.json`,
  `static-off.json` (the "off" baseline predates newly installed analyzers: compare with a
  fresh run of the binary at `<sp>/u28/Q1/bench/dla` if it differs for that reason).
- `dla-bench dynamic --parallel 2`, three fixtures per runtime you changed (python, go, jvm,
  node): outcomes equal to `<sp>/u23/dynamic.json`; a chance failure is traced, not skipped.

### 5. Tests you SEE fail without the change (revert the hunk, run, restore; say which)
Census unit tests per language for each fact (depth of an element write and delete, init
context, parameter-derived key, private holder with a same-named variable in another file,
non-container not listed, weak and bounded types, the `x[k] = make(...)` case); score unit
tests (each weight's sign, the three real shapes: a static keyed cache with parameter keys and
no remover outranks a rule hit on an instance field written only by `register*` methods; an
outer map with element-level deletes only outranks a map with a real remover; a package
variable assigned once in `init` ranks below both); a CLI test that `dla ledger list` prints
score and why in descending order and that `--next` adds the siblings of one struct.

## Rules
- The order is a pure function of the finding and its census data: no network, no model, no
  project names, no file names of the measured projects anywhere in code or tests (tests use
  small synthetic sources).
- Never edit anything under bench/fixtures/ or bench/fixtures-open/, nor EXPECTED_FAILURES in
  bench/runner/src/dynamic_cmd.rs. Never skip, ignore or weaken a test. An existing test that
  asserts the old order is changed only where the new order is the designed behaviour: list
  each one.
- Cargo with `-j 2`, the shared `target/` only; bench and real-tree runs use binary copies.
  Never kill processes by name. Disk: if `df --output=avail -B1M /` is below 2500, run
  `bash <sp>/prune-superseded.sh` and delete your own finished copies by literal path.
- Docs: `skills/detect-leak-anywhere/references/stages.md` (where the queue and the census are
  described: what the score is made of, `sibling_of`), `references/evidence-and-report.md` if
  it describes the ranking note, README.md if it states the tiers. SKILL.md only if a sentence
  there becomes false (it must stay within the limits of `cargo test -p dla-cli --test
  skill_text`). docs/VERIFY.md: one new row per item at the end of the `## M7` table
  (`| Item | Status | Evidence | Resolution |`, exactly 4 cells, escape pipes as `\|`, never
  nest backticks in one code span, "corrected" only when its tests ran, name every test; no
  model names, no line numbers, no scratch paths, no plan or package names, no names of the
  measured projects other than as "a Go RPC library", "a Java data-access library" and the
  like).
- Before reporting: `cargo test -j 2 --no-fail-fast -p dla-census -p dla-core -p dla-cli
  -p dla-workload -p dla-report`, `cargo clippy --workspace --all-targets -j 2 -- -D warnings`,
  the Windows-target clippy for dla-core and dla-report if it is set up
  (`cargo clippy -p dla-core -p dla-report --target x86_64-pc-windows-gnu -j 2 -- -D warnings`),
  `cargo fmt --all --check` (format only your own files), `target/debug/dla schema --check`,
  `python3 bench/runner/test_grade.py`, `cargo test -j 2 -p dla-cli --test skill_text`.
- No separate checker follows you: the orchestrator reruns your commands. Write the full report
  to `<sp>/u33/report.md` with the Bash tool (`cat > file <<'EOF'`), and return at most 60
  lines: the rank table (now, script, built) for the four trees and the two hold-outs, the
  fixture guard, removed candidates, files changed, tests (which you saw fail), every command
  with its result, what is unverified, and every place where you departed from this brief.
