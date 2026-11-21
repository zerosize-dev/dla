# The measurement queue (orchestrator brief, two packages: Q1 code, Q2 text and bench)

Repo /home/user/dla, branch main. The working tree holds uncommitted work of an earlier pass
and another fixer is editing `crates/dla-cli/src/summarize/go_pprof_heap.rs`, its tests, maybe
the capture-axis handling in `crates/dla-stats` or the trend code, and the Go reference files.
Do NOT commit, push, stash, reset or revert anything. Never undo a hunk you did not write;
re-read a shared file right before each edit and change only your own lines.
sp = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad
Code map with file:line references for everything below: `<sp>/wip-notes/queue-survey.md`
(read it first; it was made on this tree).

## Why
A real run of the plugin on a Python library with a documented leak (aiohttp 3.10.10; 187
static candidates, the leak among them as F8, `aiohttp/web_app.py:82`, a rule hit joined with
its census entry) ended after 5 minutes without a finding. The agent wrote ONE generic
workload, saw no growth tied to a candidate, and closed all 187 candidates with
`dla ledger resolve F<i> --as unresolved --reason not_exercised --remedy "<text>"` in a shell
loop while 2 h 55 min of its 3 h budget were left. It did what the tool said: `dla run status`
and `dla gate check --final` print exactly that command as the fix of every open candidate,
`revisit_total` was 0, the final gate then passed and the report said COMPLETE. Every existing
"measure before you close" rule asks only whether SOME harness of the runtime ran. Twice the
agents also tried `dla ledger list` (it does not exist) and then read `ledger.jsonl` by hand.

What a run that finds the leak does (seen in a hand-driven run): pick the most promising
candidates, write a workload whose unit reaches THEIR code with a new key per unit, probe
their holders, run, read which grew, repeat with the next ones while time is left.

## The design (decided; do not redesign, report doubts instead)

### 1. Ranking (one pure function, shared by the queue and the report)
A candidate is a leak-side finding in state `candidate` that is not a sibling (`N-*`) and not
a duplicate. Tier of a candidate (lower is better), from the finding and its annotation only:
- 1: origin or `also_origins` holds `known-leak:*` or `detector:dla/*` (DLA's own rules);
- 2: census (origin or `also_origins`) with writers > 0, removers == 0 and holder in
  {registration-table, container-field, static/global};
- 3: any other `detector:*` origin;
- 4: census with writers > 0 (removers present, or another holder kind);
- 5: everything else.
A candidate whose site path is test, example, benchmark or documentation code (a path
component `test`, `tests`, `testing`, `testdata`, `__tests__`, `spec`, `example`, `examples`,
`benchmark`, `benchmarks`, `bench`, `docs`, `doc`, or a file name `*_test.go`, `test_*.py`,
`*_test.py`, `*.test.js|ts`, `*.spec.js|ts`, `*Test.java`, `*Tests.java`, `*Test.kt`,
`conftest.py`) is tier 6 whatever else holds. Tiers 1 to 4 are ELIGIBLE for the queue; 5 and
6 never are. Order: tier, then findings that carry more than one origin (`also_origins`
non-empty) first, then ledger order. `dla-report`'s `static_tier` (crates/dla-report/src/lib.rs)
becomes this function (move it to dla-core, keep the report's rendering; the report's TIER
note text is updated to the five tiers above plus the demotion).

### 2. Targets of a harness
- `dla harness gen ... --probe F<n>` keeps its meaning and now also DECLARES F as a target of
  the harness. A finding that has no probe target of its own is no longer refused or dropped
  for that reason: it is recorded as a target without a probe (the manifest gap note stays).
  Duplicates still resolve to the canonical finding; a finding of another runtime is refused.
- New `--next [N]` (default and maximum 8): declares the next N PENDING queue entries (see 4)
  of the harness's runtime as targets, in rank order, exactly as if each were given with
  `--probe`. With no pending entry it says so and declares none.
- At most 8 declared targets per harness (`--probe` and `--next` together): more is refused
  with code `too-many-targets` naming the limit.
- The manifest (`HarnessManifest`, dla-core harness.rs) records `targets: Vec<FindingId>`
  (serde default; schema regenerated). The probe column of a declared target is never dropped
  by the 64-column cap of a series (it ranks above automatic probes).
- `dla harness gen` prints for each declared target (in `summary.targets`, and as a comment
  block at the top of every generated DRIVER file, all runtimes' templates, above the first
  DLA-HOOK): finding id, site, holder kind, owner, the rule's message when a detector raised
  it (from the static-hit evidence), up to 3 writers and up to 3 removers as `file:line` with
  the name of the enclosing function where it can be resolved, what
  its probe reads or "no probe: tied by allocation stacks or type counts", and one line of
  instruction by holder kind:
  - a cached function (census kind cache-decorator): "call <owner> with arguments that differ
    for every unit (derive them from key)";
  - static/global, container-field: "make every unit reach a writer so that it adds an entry
    whose key derives from the unit's key";
  - registration-table: "register through the public API once per unit, and release only what
    a normal caller of that API releases";
  - other kinds: "make every unit run the code at <site>".
  plus one general line: "use the project's public API the way a caller would; in fresh mode a
  unit must never reuse an earlier unit's key". `summary.next` of a driver harness with targets
  reads "fill the hooks so that each unit reaches the targets listed in the driver, then dla
  harness run <H> --mode both".

### 3. Verdict per target after a run
`dla trend S<n>` computes, for every declared target of the series' harness and for every
candidate an automatic probe of that harness stands for, one verdict and stores the list in the
trend report (`targets: [{finding, verdict, column, detail}]`, serde default, schema
regenerated). Verdicts, first that applies:
- `grew`: a column tied to the finding (`finding` or `candidates` of the column) passed the gate;
- `changed`: a probe column of the finding exists, did not pass, and its values over the
  points are not all equal;
- `constant`: a probe column of the finding exists and all its values are equal;
- `not_loaded`: the harness has a probe for the finding and the series has no column for it;
- `executed` / `not_executed`: no probe; the run's raw coverage shows that the function
  enclosing the finding's site or one of its writer sites ran, or that none did (Python 3.12+
  from the dump's coverage items, which carry `file:line` of each function; Go from the cover
  profile; Node from the V8 coverage files; JVM: the class of the site or of a writer is among
  the loaded classes). Find the enclosing function the way `trend/candidates.rs` already
  matches "the same enclosing function";
- `unknown`: nothing can tell (Rust, native, Python before 3.12, no coverage data).
`detail` is one short sentence a reader can act on ("probe size stayed 1 over 9 points: every
unit used the same key or never reached a writer", "function _build_middlewares at
aiohttp/web_app.py:82 was not executed"). `dla trend` prints the verdicts of the declared
targets in its JSON (`targets`, at most 8) and counts by verdict for the rest. Verdicts of a
series are re-read from the stored trend reports; nothing recomputes them later.

### 4. Pending and settled
For an eligible candidate F, with the run's stored trend reports:
SETTLED when one of:
- some verdict of F is `grew` or `changed`;
- F was a declared target of a DRIVER-mode harness run (a series exists) with verdict
  `executed` or `unknown`;
- F was a declared target of TWO driver-mode harness runs whose driver files differ (digest of
  the driver directory content at run time; store it in the run record if it is not there),
  whatever the verdicts;
otherwise PENDING. Verdicts under a fixed command (command mode, JUnit mode) settle only by
`grew` or `changed`.
The queue of a runtime is its PENDING eligible candidates in rank order. The queue is "open"
when all of these hold: the run's queue setting is on (5), the budget is over the floor
(`gate::budget_over_floor`, the worker's deadline in worker mode), the runtime's workload is not
recorded as blocked (`workload_unresolved`), and the queue is not empty.

### 5. Enforcement and guidance
- `dla run init --queue on|off` (default on), recorded in the run config evidence and the run
  state. Off restores today's behaviour everywhere below, and the report says so in its header
  ("Queue: off: candidates the workloads did not reach were closed without a targeted
  workload"). It exists for fixed test policies that cannot write a workload; the skill text
  never mentions it.
- `dla ledger resolve F --reason not_exercised|no_harness|time_budget` on a PENDING candidate
  while its runtime's queue is open is refused with the new code `target-pending`: the message
  gives F's rank, the minutes left, and the fix, which is the next batch command for that
  runtime: `dla harness gen --lang <rt> --driver --next` (a JVM project with JUnit tests and no
  driver support for its build keeps its existing generation command with `--next`), "fill the
  hooks so each unit reaches the listed targets with a new key, dla harness run <H> --mode both,
  dla trend S<n>", and the way out when no driver can be built or run for that runtime:
  "record why: dla ledger resolve intake.workload --status unresolved --reason
  <env_unavailable|tool_missing|build_fails|privilege_declined> --remedy <text>" (that closes
  the queue of the runtime, as it lifts the harness refusals today). Blocker reasons
  (env_unavailable, tool_missing, build_fails, privilege_declined) on a finding stay accepted,
  and so does everything about `inconclusive`, `refute`, `test`, `confirm`.
- `status()` / `gate::missing` (final): while a runtime's queue is open, ONE item of the new
  kind `queue` per runtime leads the list: "N candidate(s) of <rt> are ranked for measurement
  and not targeted yet; M of T budget minutes remain" with that batch command as the fix and the
  ids of the next batch. The per-finding item of a PENDING candidate carries the same batch
  command as its fix (never the resolve command). The per-finding item of every other open
  candidate carries `dla ledger close-rest` as its fix. `next_command` of `dla run status` is
  the batch command while a queue is open. The stop hook's block reason leads with the queue
  item. `dla run status` gets a `queue` object per runtime: eligible, settled, pending, the next
  batch (ids), the command, and `open` with the reason when it is not (off, floor reached,
  workload blocked, empty).
- New `dla ledger list [--state <s>] [--limit N] [--all]`: one line of JSON per finding in rank
  order (id, state, tier, rank among eligible, origin(s), site, holder, owner, writers and
  removers counts, queue status: pending, settled with the verdicts by harness, not-eligible),
  default limit 40, with totals. Read-only.
- New `dla ledger close-rest [--remedy <text>]`: dispositions every open candidate that is
  not PENDING-in-an-open-queue, through the same transition rules as `ledger resolve`:
  `suspected_static_only` when the finding has static evidence, else `unresolved`; reason
  `not_exercised`, or `time_budget` when the queue closed because the floor was reached and the
  ledger accepts it; the remedy is the caller's text or, by default, one built from the
  finding's history ("targeted in H2 and H3, its holder did not change: write a unit that adds
  a new key to <owner>"; "ranked 143 (tier 4), not targeted: dla harness gen --driver --probe
  F143"). It prints counts by disposition, the ids it left open and why (with the batch
  command), and writes one evidence record of kind run-config that lists what it closed.
  Under-test findings are not touched.
- Report: a header line after `Coverage` ("Queue: 24 of 121 ranked candidates targeted in 3
  harnesses: grew 1, changed 3, executed 14, not reached 6; 97 not targeted"), and each
  static-only or unresolved finding section or table row says how it was targeted ("Targeted:
  H2 constant, H3 not executed" or "Not targeted (rank 143)").
- Worker mode: a worker's queue is its runtime's; the same refusal, status object and
  `close-rest` work with `--worker`; `dla ledger merge` and the core's final gate treat a
  worker's still-pending candidates like the core's own.

### 6. What must not change
- With `--queue off` every existing behaviour and message is as before.
- `dla-bench dynamic` passes `--queue off` at `dla run init` (package Q2) and its 64 fixtures
  keep their results. The ledger's state machine, the confirm and refute rules, the time-budget
  floor, the graders.

## Rules for both packages
- Never edit anything under bench/fixtures/ or bench/fixtures-open/, nor EXPECTED_FAILURES in
  bench/runner/src/dynamic_cmd.rs. Never skip, ignore or weaken a test. Every changed behaviour
  gets a test you SEE fail without the change (revert the hunk, run, restore); say which you
  saw fail and which you judged by reading.
- Cargo with `-j 2`, the shared `target/` only (builds of other agents hold its lock: wait).
  For bench runs copy `target/debug/dla` and `target/debug/dla-bench` into your scratch dir and
  use the copies (`DLA_PLUGIN_ROOT=/home/user/dla` when you call the copy of dla directly).
  Never kill processes by name. Scratch dir `<sp>/u28/<package>/` (delete only it, by literal
  path; never `rm -rf "$VAR/..."`).
- Disk: if `df --output=avail -B1M /` is below 3000, run `bash <sp>/prune-superseded.sh`.
- docs/VERIFY.md: one new row per changed behaviour at the end of the `## M7` table
  (`| Item | Status | Evidence | Resolution |`, exactly 4 cells, escape pipes as `\|`, never nest
  backticks in one code span; "corrected" only when its tests ran; name every test; no plan or
  package names, no model names, no line numbers, no scratch paths).
- Before reporting: `cargo test -j 2 --no-fail-fast` for every crate you touched,
  `cargo clippy --workspace --all-targets -j 2 -- -D warnings`, the Windows-target clippy for
  dla-core and dla-report if you touched them (`cargo clippy -j 2 --target
  x86_64-pc-windows-msvc -p dla-core -p dla-report -- -D warnings`), `cargo fmt --all --check`
  (format only your own files), `target/debug/dla schema --check` after regenerating with
  `dla schema --out schemas` when a serialized type changed, `python3
  bench/runner/test_grade.py`, `cargo test -j 2 -p dla-cli --test skill_text`.
- No separate checker follows you: the orchestrator reruns your commands and then runs the
  real plugin on real projects. Write the full report to `<sp>/u28/<package>/report.md`; return
  at most 60 lines: each item's state, files changed, tests (which you saw fail), every command
  with its result, what is unverified, and every doubt about the design.

## Package Q1: the code
Everything in sections 1 to 5 above in crates/dla-core, dla-cli, dla-workload (generation and
the driver templates under harness/), dla-report and schemas, with its tests:
- unit tests of the tier function (each tier, the demotion, the order), of pending/settled
  (each clause, a fixed-command verdict that does not settle, two driver runs with the same
  driver that do not settle), of each verdict;
- CLI tests on a small Python project built in the test (a module with an `lru_cache(None)`
  function keyed by its argument, a class-level dict cache with a writer and no remover, a
  dict with a writer and a remover, and a test file with a candidate): after `run init`,
  `sweep`, `census`, a driver harness whose unit touches none of them and its trend:
  `ledger resolve F --as unresolved --reason not_exercised` is refused `target-pending` and
  names the batch command; `run status` and `gate check --final` lead with the `queue` item;
  `harness gen --driver --next` declares the targets, prints them and writes the comment block;
  a unit that calls the cached function with a new key per unit gives `grew` for it and
  `constant` for the dict it does not touch; a second driver run with a changed driver settles
  the constant one; `ledger close-rest` then closes what is left with the right dispositions
  and the final gate passes; the same project with `--queue off` behaves as before; with the
  budget under the floor (`--budget 90s` and a wait, or the test hook the existing budget tests
  use) the resolve is accepted; a blocker reason is accepted; `ledger list` prints rank order;
- the existing tests keep passing unchanged unless one asserts a text this design changes
  (then update the assertion and say so).
Then, with copies of the built binaries, run `dla-bench static` in both modes (64 fixtures
pass; candidates unchanged against `<sp>/u23/static-on.json` and `static-off.json`). The
dynamic bench will fail on `target-pending` until Q2 has wired `--queue off`: run
`dla-bench dynamic --parallel 2 --only python-` only after the file `<sp>/u28/Q2/bench-wired`
exists (wait for it at the end of your work, up to 40 minutes, then run it; all python
fixtures must keep the outcome of `<sp>/u23/dynamic.json`).

## Package Q2: the skill text, the bench wiring and the documents
Works from this brief's command names; the binary that implements them is being built by Q1.
1. bench/runner: `dla-bench dynamic` (and the worker and artifact paths under
   bench/runner/src/dynamic/) passes `--queue off` to `dla run init`; its doc comments say why
   (a fixed policy cannot write a workload). `dla-bench run` (the model benchmark) does not
   pass it. Add or adjust the runner's unit tests. When this compiles and its tests pass, create
   the empty file `<sp>/u28/Q2/bench-wired` (Q1 waits for it).
2. The skill (skills/detect-leak-anywhere/SKILL.md, references/stages.md, recovery.md,
   evidence-and-report.md, agents/worker.md): the flow becomes
   - stage 3 ends with `dla ledger list` to see the ranked candidates;
   - stage 4: the first driver harness is already a batch: `dla harness gen --lang <rt>
     --driver --next`; read the target block in the driver; write `setup` and `unit` so every
     unit reaches each target with a new key, using the public API as a caller would; run in
     both modes; `dla trend` and read `targets`: `grew` goes to `dla ledger test`; `constant`,
     `not_loaded` and `not_executed` mean the unit did not reach it or reused a key: fix the
     unit and run once more; then the next batch;
   - the loop continues while `dla run status` shows an open `queue`; `dla ledger close-rest`
     closes what is left when the queue is empty or closed; a `target-pending` refusal is fixed
     by the batch command it prints, never by another reason;
   - the passage of stage 7 that tells the agent to close every candidate the harness did not
     exercise, the "Before you stop" bullets and stage 9 (revisit) are rewritten to this loop;
     recovery.md gets `target-pending` and `too-many-targets`; evidence-and-report.md the
     report's Queue line and the "Targeted:" lines; the worker agent text the same loop.
   Keep SKILL.md within the limit `cargo test -j 2 -p dla-cli --test skill_text` checks (that
   test also checks other properties of the text: read it first). Write for an agent that
   follows instructions literally: imperative, concrete commands, no rationale paragraphs.
3. README.md and docs where the flow or the commands are listed (`dla ledger` subcommands,
   `dla run init` options, `dla harness gen` options): add the new ones.
4. docs/VERIFY.md rows for the skill and bench changes (Q1 writes the rows of the code).
5. After Q1 reports that the tree builds (the file `<sp>/u28/Q1/built` exists; wait for it up
   to 3 hours, checking every few minutes while you do the text work), run with copies of the
   binaries: `cargo test -j 2 -p dla-cli --test skill_text`, `cargo test -j 2 -p dla-bench`
   (the runner's crate; find its name in bench/runner/Cargo.toml), `dla-bench dynamic
   --parallel 2 --only go-ticker` and `--only jvm-order-cache` and `--only node-price-board`
   (the outcomes of `<sp>/u23/dynamic.json`), and check by hand on a scratch copy of
   bench/fixtures/python-report-memo/project that the commands the skill text names exist with
   the options it names (`dla ledger list --help`, `dla ledger close-rest --help`,
   `dla harness gen --help` shows `--next`, `dla run init --help` shows `--queue`).
Q1 creates `<sp>/u28/Q1/built` as soon as `cargo build -j 2 -p dla-cli -p dla-bench` succeeds
with the new commands present, and again leaves it in place after later changes compile.
