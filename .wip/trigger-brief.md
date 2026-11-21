# The trigger of a confirmed leak: who feeds the keys? (brief for one builder)

Repo /home/user/dla, branch main (6a7ee29 or later). The working tree holds uncommitted work of
other agents; three of them edit it at the same time: the census and
`crates/dla-core/src/rank.rs`, `queue.rs` and `--next` in harness gen (candidate order);
`ledger_queue.rs`, `machine.rs`, the run record of a harness (close reasons, environment
reuse); the Go harness template and Go coverage. Do NOT commit, push, stash, reset or revert
anything. Touch only what this job needs, never undo a change you did not write, re-read a
shared file right before each edit, keep the tree compiling. When the build is broken by a file
that is not yours, wait a minute and retry (at most ten times), then report it.
sp = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad
Your scratch dir: `<sp>/u35/` (create it; delete only it, by literal path; never
`rm -rf "$VAR/..."`).

## The defect (a real run of the plugin on a Python HTTP library, notes in `<sp>/wip-notes/real-run-plugin-aiohttp-2.md`)
The queue made the agent write a driver for the first 8 ranked candidates. Its unit called an
`lru_cache(None)` function with a new argument per unit; the cache grew, the finding was
confirmed, labelled and explained. Then the blind verifier disagreed ("the harness varies the
key on every unit, which real use doesn't do"), and the agent's final answer was "both
confirmed findings are disputed and I don't think either is a leak that real use would
trigger". The first finding IS the documented leak of that version: a server built on the
library creates a new handler object for every request to a path with no route, so plain
requests grow the cache. Nothing in `dla` asks who feeds the keys: growth under direct
injection is confirmed as a leak, and whether a caller of the public API can produce it is left
to opinion. The second finding (a class-level cache keyed by a configuration string) grows
only when the driver invents new configuration: reporting it as a leak is wrong too.

## What to build
Two things are separated from now on: the holder GROWS when new keys reach it (what the queue's
drivers show), and a caller can MAKE new keys reach it (the trigger).

### 1. Entry-path harness
`dla harness gen --lang <rt> --driver --entry F[,F2,...]` (at most 8 findings; each must be a
leak-side finding that grew in some harness, else a refusal with the fix) writes a hook driver
like `--probe`, declares the findings as targets, and marks the harness `purpose: entry-path`
in its manifest (a new optional field; `dla schema --out schemas`). Its target block, above the
hooks, says for each finding:
- the writer functions (as today) and the CALLERS CHAIN of each: up to three hops of static
  callers (a new module in crates/dla-census: calls whose callee name matches the function,
  mapped to their enclosing function, by syntax tree, for Python, Go, Java and
  JavaScript/TypeScript; at most five callers per hop, public ones first and marked; say
  "matched by name" in the block; other runtimes print "callers: not computed");
- where the key comes from: which parameter of the writer function the key or stored value is
  built from, and the expression passed for it at each first-hop call site (one line each);
- the rules of an entry-path unit, as text of the block: call only the project's public entry
  points (the functions, endpoints, commands or messages a user of the project calls), with
  the objects under test built once in `setup`; per unit vary only data that a caller or a
  remote peer controls (ids, names, paths, methods, headers, bodies, file names, message
  contents), including inputs the project rejects (unknown paths, unsupported methods,
  malformed input: error paths often build objects per call); never create new functions,
  classes, handlers, loggers or configuration per unit unless a real program does so per
  request, and if you do, say so in the varies line.
The driver directory gets `VARIES.txt`; `dla harness run H` refuses an entry-path harness whose
`VARIES.txt` is empty or still the placeholder (`varies-missing`; fix: write one line: what a
caller varies per unit). The run record of the series stores that line.

### 2. The trigger of a finding (a pure function of the run directory, beside the queue code in dla-core)
- `reproduced`: the finding has verdict `grew` in the trend of an entry-path harness, OR it
  grew in a harness that runs the project's own workload (a fixed command of the project or a
  JUnit repeat: no hand-written hooks). The trigger text is the `VARIES.txt` line, or
  "the project's workload <id>".
- `not_found`: at least two entry-path harnesses with different driver digests targeted it,
  ran and were read with `dla trend`, and it grew in none.
- `unchecked`: anything else (it grew only in drivers that are not entry-path).
Shown: in `dla run status` (a `triggers` object for confirmed and strong findings), in
`dla ledger list` rows, in the verdict output of `dla trend` for an entry-path series
(`trigger: reproduced` and the next command), and in the report (item 4).

### 3. Enforcement
While the run was started with the queue on and more time than the floor is left: a leak-side
finding in state `confirmed` or `strong` whose trigger is `unchecked` gives one item (kind
`trigger`) in `dla run status`, in the final gate and in the stop hook, right after the queue
items: "F8 grew when the harness fed it new keys; whether a caller of the public API can do
that is not shown", fix: the `--entry` command with every unchecked finding of that runtime,
then fill, run, trend. After the first entry-path harness without growth the item's fix says
to write a second, different one (other entry points, other inputs, the rejected-input
paths); after two it is satisfied (`not_found`). At or under the floor the item is not raised
and the report says the trigger was not checked for lack of time. `dla ledger confirm` is NOT
blocked by this (confirmation says the holder grows). With `--queue off` nothing of this
applies (the benchmark's fixed dynamic policy must be unchanged: `dla-bench dynamic` outcomes
equal to before).

### 4. Report, verifier, skill
- Report: every confirmed or strong finding has a line `Trigger: reproduced through the public
  API in H5 (varies: <text>)`, or `Trigger: not found: it grows when the harness passes new
  keys to <writer function>; two workloads through the public API did not grow it (H5: <varies
  text>; H6: <varies text>)`, or `Trigger: not checked (<why>)`. The `Findings:` header line
  splits confirmed and strong into "reproduced through the public API N, no trigger found N,
  trigger not checked N", and the findings section lists reproduced ones first.
- Bundle and verifier (`dla bundle`, `agents/verifier.md`): the bundle of a finding with an
  entry-path run holds that harness's driver, its `VARIES.txt` and the trend verdict; the
  verifier's instructions say to judge the trigger from them (does the unit call public entry
  points only, is what it varies under a caller's control) and that growth under direct
  injection alone is not a reason to call a finding a leak or a non-leak.
- Skill (`skills/detect-leak-anywhere/SKILL.md` checklist and "Before you stop",
  `references/stages.md`, `recovery.md` for the new refusals, `evidence-and-report.md`,
  `agents/worker.md`): the entry-path step after a target grew and was confirmed; the final
  answer names leaks by trigger: a reproduced one is a leak with its trigger; one without a
  trigger is "an unbounded holder, no trigger found", never "a leak" and never "disputed" for
  that reason alone. SKILL.md must stay within the limits of
  `cargo test -p dla-cli --test skill_text`.

### 5. Tests you SEE fail without the change (revert the hunk, run, restore; say which)
Callers chain unit tests per language (three hops, name match, public first, cap); the key
provenance line; `--entry` refusals (a finding that never grew, more than 8); `varies-missing`;
the trigger function (reproduced by an entry-path harness, reproduced by a fixed-command
harness, not_found after two different entry-path drivers, unchecked after one, same driver
twice does not count); the gate, status and stop-hook item with its two fix texts and its
absence under the floor and with the queue off; the report lines and header counts; the bundle
contents. One end-to-end CLI test on a small Python project in the test's temp dir: a module
with an unbounded cache keyed by an object that an entry function creates per call for
rejected input, and a second cache keyed by a constructor argument that only set-up code
passes: direct driver grows both; the entry-path driver (the test fills the hooks) grows the
first only; triggers read `reproduced` and, after a second entry-path driver, `not_found`.

### 6. Real check
Python 3.13, a copy of the measured library: `cp -r <sp>/u18/aiohttp/proj <sp>/u35/aiohttp`,
remove its `.dla`, install its runtime dependencies into `<sp>/u35/deps` with
`pip install --target` (aiohappyeyeballs aiosignal attrs frozenlist "multidict<7" "yarl<2")
and export `PYTHONPATH` for every dla command; static stage (`dla run init`, `dla probe`,
`dla sweep --lang python`, `dla census --lang python`, the intake answers), first batch with
`dla harness gen --lang python --driver --next`, fill the hooks so the two top targets grow
(direct calls), confirm them the way `references/stages.md` says, then run YOUR entry-path
step on both by following only what the generated block and the tool's next commands say (do
not use what this brief says about the cause): report whether the block's callers chain and
key line led you to a public-API unit that grows the first holder, what `VARIES.txt` said,
and what the report prints for both. If the block did not lead there, say what was missing
from it: that is a finding about the block, fix the block, not the test.

## Rules
- Never edit anything under bench/fixtures/ or bench/fixtures-open/, nor EXPECTED_FAILURES in
  bench/runner/src/dynamic_cmd.rs. Never skip, ignore or weaken a test. No project names of
  measured libraries in code, tests, docs or VERIFY rows ("a Python HTTP library").
- Cargo with `-j 2`, the shared `target/` only; copies of the binaries for bench and real-tree
  runs (`DLA_PLUGIN_ROOT=/home/user/dla` when calling the copy directly). Never kill processes
  by name. Disk: if `df --output=avail -B1M /` is below 2500, run
  `bash <sp>/prune-superseded.sh` and delete your own finished copies by literal path.
- docs/VERIFY.md: one new row per item at the end of the `## M7` table
  (`| Item | Status | Evidence | Resolution |`, exactly 4 cells, escape pipes as `\|`, never
  nest backticks in one code span, "corrected" only when its tests ran, name every test; no
  model names, no line numbers, no scratch paths, no plan or package names).
- Before reporting: `cargo test -j 2 --no-fail-fast -p dla-census -p dla-core -p dla-cli
  -p dla-workload -p dla-report`, `cargo clippy --workspace --all-targets -j 2 -- -D warnings`,
  `cargo fmt --all --check` (format only your own files), `target/debug/dla schema --check`,
  `python3 bench/runner/test_grade.py`, `cargo test -j 2 -p dla-cli --test skill_text`,
  `dla-bench static` in both modes (64/64), `dla-bench dynamic --parallel 2 --only python-`
  and three fixtures of other runtimes (outcomes equal to `<sp>/u23/dynamic.json`; a chance
  failure is traced, not skipped).
- No separate checker follows you: the orchestrator reruns your commands. Write the full report
  to `<sp>/u35/report.md` with the Bash tool (`cat > file <<'EOF'`), and return at most 60
  lines: each item's state, the real check's outcome in full, files changed, tests (which you
  saw fail), every command with its result, what is unverified, where you departed from the
  brief.
