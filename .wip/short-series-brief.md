# Brief S: a measured leak that the tool kept silent about (real run axios-3)

Rules as before: Sonnet only; no commits; shared `target/`; literal paths in `rm`; no process
kills by name; mutations carry a unique marker and are reversed by exact replacement; do not
weaken, skip or delete a test, a fixture or a policy; no third-party text; nothing about the
plan or these notes in the repo. Other builders are editing the tree right now (census,
gate hooks, run state, `alloc_cmd.rs`, `alloc/pick.rs`, `summarize/tie.rs`, one block of
`trend_cmd.rs`): make small exact edits, never reformat or rewrite a whole file, rebuild
before blaming your own change for a compile error in a file you did not touch. Free disk is
about 3 GB: delete your build output and scratch copies (literal paths) as you go.
Scratch: `<sp>/u46/`. Report: `<sp>/u46/report.md` and a short final message.
`<sp>` = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad

## What happened (evidence: `<sp>/u36/axios-3/`, run dir `project/.dla/20261121-135442-fdbf`)

Real plugin run on axios 1.15.1 (documented leak: `socket.on('error', ...)` at
`lib/adapters/http.js:924`, one listener and its closures kept per request on a keep-alive
socket). The run ended COMPLETE after 1 h 33 min with "no leak confirmed".

The very first harness (H1, plain `axios.get` against a local server) reproduced the leak
and the series show it:

- `node.listeners@lib/adapters/http.js:924`: slope exactly 1.0 per iteration, every delta
  equal, noise 0 (S1, S20, S22, S23); also `:925`, `:971`, `:908`, `:903`, `:717` (7 per unit);
- `node.listeners`: 18 per iteration (15,896 to 63,402 in S1);
- `node.heap_used` (after GC): about 15 KB per iteration (25 MB to 64 MB in S1), in fresh and
  in repeated mode;
- the run's own detector fired: `MaxListenersExceededWarning: ... 11 error listeners added
  to [Socket]` (`detectors` of `harness/H1/run-S20.json`, `site: null`).

`dla trend` answered `"passed": []`, `"failed": 64` and one `next` line about missed targets.
Every one of these columns has `"gate": "fail", "gate_failed": ["n < 8"]` and nothing else
(`trend/T1.json`, `T41.json`). All 23 series of the run have 4 to 7 points after warm-up:

1. The first H1 run was cut by the time limit (300 s per process): 8 and 7 of 9 points.
   A unit was two HTTP requests; 440 units a point did not fit.
2. A later driver did not end at all: its HTTP servers kept the Node process alive until
   the kill, so no coverage data was written ("no coverage data" for every target).
3. After that the model ran everything with `--iterations 3..5` to save time.
4. The model reads `dla harness run ... | head -c 200`, so `limitations` and `detectors`
   were never on its screen; of `dla trend` it reads the first 1500 to 3500 bytes.

The finding at the documented site exists (F23, census, registration-table) but is tier 5,
score -51 (`w_none -40`, `tier5 -20`, `r_holder_level -12`, `recv_outer +8`): its annotation
has `writers: []` because no other statement "writes" `socket`, and `removers:
[lib/adapters/http.js:913]`. It was never in the queue and was closed as "tier 5 (not ranked
for measurement), not targeted". Its siblings on `req` (F21, F22, F24: score 47,
`recv_local -20`) were in the queue. The 30 above-threshold residual rows (heap 15 KB per
iteration unattributed) were closed in a loop as unresolved with one sentence; the open
item's text offers that ("... or record it unresolved").

Nothing here is specific to axios. Fix the tool, with tests on small synthetic projects.

## S1. A series too short for the gate is not a silent "nothing grew"

a. `dla trend` on a series with fewer points after warm-up than the gate's minimum: evaluate
   every other gate condition; the columns that fail only for the point count are listed in
   a new output field (`short`, at most 12, site/probe columns that have candidates first,
   then other counters, then heap bytes) and `next[0]` says so in one line: how many points
   the series has, how many the gate needs, the 3 strongest columns with slope per
   iteration and the candidate findings of their site, and the command that measures at
   full length (`dla harness run <H> --mode <mode>` without `--iterations`), then `dla trend`.
   The stored trend file keeps `gate: fail` for them (the gate itself is unchanged).
b. On such a series no declared target may be settled by "the code ran and no column
   passed": that verdict becomes `unknown` with the detail "series too short: N points, 8
   needed" (unknown does not settle a target; check `dla_core::queue` for how `unknown` is
   treated and keep that rule). A size probe that did not move at all stays `constant`.
c. `dla harness run`: when the series it wrote has too few points (a low `--iterations`, or a
   process cut by the time limit) the output says so first (see S4) and names the full-length
   command. Do not refuse low `--iterations`: tests and smoke checks use it.

## S2. The Node driver always delivers its points and always ends

a. The driver exits by itself when `main()` is done and the last checkpoint is flushed (open
   servers, keep-alive sockets and timers of the unit must not hold the process until the
   kill); coverage data (`NODE_V8_COVERAGE`) must still be written. Find the right exit
   point in `harness/node/dla-harness.cjs` / `dla-preload.cjs` (the last checkpoint may do
   asynchronous work: heap snapshot, flush).
b. The driver fits its points into the time limit: after the warm-up round it knows the
   time per unit; when the remaining rounds would not fit into the process's time limit
   (pass the limit to the driver in an environment variable; keep a margin), it lowers the
   step for the measured rounds (never below a small floor; say what the floor is and why)
   and records the step it used; the series x axis stays the cumulative unit count. The run
   output names the reduction. If even the floor does not fit, the run says that the unit
   is too slow for a series and what to do (a cheaper unit: one request, a local server).
c. The same cut-by-the-limit problem exists for the other driver templates (Python, Go,
   JVM, Rust). Do Node fully. For the others: say in the report which of them have the
   same loop and whether the same change fits; change them only if it is the same small
   change and you can test it; otherwise leave them and list them.

## S3. A runtime detector hit is evidence with a site

`MaxListenersExceededWarning` carries the emitter, the event name and the count
(`warning.emitter`, `.type`, `.count`). The preload counts registrations by site already.
When the warning fires, record which registration site(s) hold the listeners of that event
on that emitter (site and count), so the hit in `detectors` has a `site` (the dominant one)
and `sites`. Then:

- `dla harness run` names the hit first (S4) with the site, the finding(s) at that site if
  the ledger has one, and the rising `node.listeners@<site>` column;
- `dla trend` on that run's series puts it in `next` (after the short-series line, before
  the missed-targets line): the detector text, the site, the finding at the site, and what
  to do: if the finding is not a target of this harness, the command that makes it one
  (`dla harness gen --lang node --driver --probe <F>` reusing this unit), else the ledger
  step the trend already prints for a passed column.

Look at how detector hits are consumed today (`detectors` in the manifest and the run
file, anything in `summarize`/`correlate`/known-leaks) and connect to that, do not build a
second path. If a hit has no finding at its site, the printed command is the existing
`dla ledger add --origin ...` form that fits (check which origin the ledger accepts for a
runtime detector hit and that such a finding can then be tied by the site column).

## S4. The first bytes of `dla harness run` say what changes the next step

The model reads the first 100 to 300 bytes. The output object is printed with sorted keys
(`artifacts`, `candidates`, `command`, ...). Add one key that sorts first (`_attention`, an
array of at most 3 short lines, absent when there is nothing) carrying, in this order: a
detector hit (S3), a series that is too short or was cut (S1c, S2b), and "no coverage data
was written" with its cause when known. Keep each line under about 160 characters and
start it with the thing to do. Check the schema of the command output (`dla schema --check`)
and the docs that list the output fields.

## S5. A registration on an object handed in from outside is a candidate

In `dla-core/src/rank.rs`: a census registration-table finding (container `Registry`) whose
own site is the registration call is its own writer: it must not get `w_none`, and it must
not fall to tier 5 for having no other writers (`tier()` needs `writers > 0`; the site is
the insert). Use the facts the census already records (`receiver`, `remover_sites`, the
site's function); do not change the census (another builder is in it) unless a fact you
need is missing, and then say so instead of adding it. With the receiver `outer` (the
emitter is a parameter or comes from a callback: it can outlive the call) such a finding
should rank above the same registration on a receiver created in the function. Keep the
table of weights the single place for numbers; add the unit tests next to the existing
ones (F23's annotation above is the test case: it must become eligible and rank above
F21/F22/F24's shape). Run the ordering tests that pin the documented sites of aiohttp
(`dla ledger list` order on the aiohttp copy is in `<sp>/u36/aiohttp-3/` if you need the
real ledger) and say what moved.

Also the to-do line the driver file prints for such a target (see
`crates/dla-workload/src/harness/targets.rs`, the `LIFE` text and the per-target "to do"):
when the receiver is `outer`, say that the listener count grows only on an emitter that is
used again, so the units must share that object the way a long-lived caller does
(connection reuse, a pooled or shared object), and must not shorten its life (no per-unit
close of the shared object, no server-side timeouts that defeat reuse).

## S6. The residual open item names the next measurement

For a heap (or non-heap) residual above the threshold, the open item text and the `next` of
`dla gap --residual` name the capture that can attribute it, with the real command for that
harness and runtime (`dla harness run <H> --mode fresh --capture <kind>`; take the kind from
what the runtime's agent supports), before "record it unresolved". Text only: no new gate
rule.

## Tests and checks

- Unit/integration tests for each of S1 to S6, each seen failing once (unique marker,
  exact reverse), listed in the report with what was seen.
- A synthetic Node project (ES module is not needed) with a keep-alive style leak: a
  long-lived emitter, one listener added per call inside a callback, a remover that does
  not run. Drive it through `harness gen --driver --next`, a filled unit, `harness run`,
  `trend`: with a 4-point run the short line and `_attention` appear and nothing is
  settled; with the default run the site column passes and ties to the finding at that
  site; with a server left open in `setup` the process ends by itself and coverage exists;
  with a slow unit (sleep) the step is lowered and all points arrive.
- Real tree: `<sp>/u18/axios/proj-deps` (axios 1.15.1 with its three runtime deps; copy it
  to `<sp>/u46/axios`, never run in the original). Use the unattended-runner environment on
  `dla run init` (`DLA_CONSENT_TIERS=1-4 DLA_CONSENT_RAW_VALUES=1`); never `dla consent`.
  Run init, probe, sweep, census; `dla ledger list` must now show the finding at
  `lib/adapters/http.js:924` eligible and above its `req` siblings; generate the batch
  that holds it, fill the unit with plain sequential `axios.get` against a local server
  (the H1 unit of the run, `<sp>/u36/axios-3/project/.dla/.../harness/H1/driver/dla-harness.cjs`),
  run at full length, `dla trend`: report which column passes, which finding it ties to
  and how far the printed commands carry toward `ledger confirm` (follow them: sibling,
  `ledger test`, `alloc-test`, refute the sibling, confirm; stop and report at the first
  command that is refused or wrong; do not change the tie rule). I checked this much by
  hand in the finished run directory: the same H1 at full length took 92 s for 9 points and
  `dla trend` passed `node.listeners@lib/adapters/http.js:924` (slope 1.0, candidates F23,
  F22, F24) and printed `dla ledger add --sibling-of F23 ...`; it also passed `:717` (7 per
  unit), `:925` and `:971`. Also one 4-point run of the same harness: paste
  the first 300 bytes of the `harness run` output and `next[0]` of the trend.
- `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -j 2 -- -D warnings`;
  `cargo test -j 2 -p dla-core -p dla-workload -p dla-cli` (name any failure that is not
  yours and show it failing without your change or passing alone); `dla schema --check`;
  `python3 tests/test_grade.py` if you touched anything it reads; the skill text test if
  you touched docs. `dla-bench dynamic --only node-` (with `DLA_PLUGIN_ROOT=/home/user/dla`
  and a copy of your binary, not `target/` directly) must stay as it is now: paste the
  totals.
- Docs: only where a printed field or rule is described (`skills/.../references/*.md`,
  `harness/node/README.md`, `docs/VERIFY.md` rows for what you verified and what you did
  not).

## Report

What changed (files, rule in one sentence each), tests with what was seen failing, the
real-tree outputs, what is unverified or not done, anything you saw that is wrong and
outside this brief.
