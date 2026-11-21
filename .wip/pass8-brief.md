# Pass 8: a measured growth cannot be closed unmeasured (real run axios-4)

Rules as in the earlier passes: Sonnet only; no commits; shared `target/`, build and test with
`-j 2` (a real plugin run and another builder are on this machine; run no `dla-bench`);
literal paths in `rm`; no process kills by name; unique-marker mutations reversed by exact
replacement; no test, fixture or policy weakened; never `dla consent` (unattended-runner
environment on `dla run init`). Another builder is editing `harness/*`, the driver templates,
`dla-workload/src/harness/*` and `gap/residual.rs`: small exact edits, no reformatting.
Scratch `<sp>/u49/`, report `<sp>/u49/report.md`, short final message.
`<sp>` = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad

## What happened (evidence `<sp>/u36/axios-4/`: run.json, transcript.jsonl, project/.dla)

Real plugin run on axios 1.15.1 with the build of snapshot 0dbf4c9 (your passes 7 and 7b are
in it). 129 seconds, 29 turns, Status COMPLETE, "no leak confirmed", 178 of 180 budget
minutes unused, 26 of 39 ranked candidates never targeted.

The first harness (H1, full length, 9 points) measured the documented leak and the tool said
so. `dla harness run` began with `"_attention":["READ THE DETECTOR HIT: listeners pile up at
lib/adapters/http.js:924 (10 registered), finding F23; ..."]`; `dla trend S1` printed
`passed: node.listeners@lib/adapters/http.js:924, slope 1.0, candidates F23, F22, F24` (17
columns passed) and `next`:
1. `DETECTOR HIT (...): dla harness gen --lang node --driver --probe F23   # F23 holds it, H1 did not target it; copy the hooks of H1`
2. `H1 missed 12 target(s): ...`
3. `dla ledger add --sibling-of F23 --origin hypothesis --hypothesis N-WARM ... --evidence E10`
4. `dla ledger test F23 --evidence E8`

The model ignored all four, spent its turns on two flat probes of its declared targets
(F49, F50), and then closed the run:
- `dla ledger resolve intake.workload --status unresolved --reason tool_missing --remedy
  "... the ESM probe reader appears not to observe them ..."` was accepted, although
  `intake.workload` was answered at init and two harnesses had run (4 series). After it the
  queue was closed and `dla ledger close-rest` closed 38 findings (16 `suspected_static_only`,
  22 `unresolved`), F23 among them, leaving only F9 with `why: "its column grew: dla ledger
  test F9"`.
- `dla ledger test F9` (as printed) was refused: `harness-required`, it needs `--evidence`.
- `dla ledger resolve F9 --as unresolved --reason inconclusive --remedy "listener
  registration counts ... grew cumulatively (per request), removal exists on req close; need
  a live-listener probe ..."` was accepted. The residual open items went the same way.
- `dla gate check` passed, `dla report` said COMPLETE.

By hand, in the earlier run directory, the printed steps 3 and 4 followed by `dla alloc-test
H1 --finding F23`, refuting the sibling and `dla ledger confirm F23` end in `confirmed`
(alloc-test: favors leak, decided by the :924 column). The tool has everything; it let the
model walk away from it.

## What must hold afterwards (ledger and gate rules, not advice)

1. `intake.workload` (and any other intake item the queue or the gate reads) cannot be set
   to `unresolved` with a reason the run's own records contradict. With a harness of this
   run that ran and wrote a series, `--reason tool_missing` (and any reason that says no
   workload can be run) is refused: the refusal names the harness and series, and says what
   the model should do with a probe that reads flat (see the trend's target detail; a flat
   probe is a verdict about one target, not about the workload). Look at which reasons exist
   for intake items and which of them close the queue; refuse each one that the records
   contradict. Legitimate use (no toolchain, no harness could run) is unchanged.

2. A finding that a passed growth column of a full-length series points at is not closed
   unmeasured. "Points at" = the finding is the tie of a passed column, or the first
   candidate of a passed site or probe column (the same choice `dla trend` makes when it
   prints `dla ledger add --sibling-of <F>`), whether or not it was a declared target.
   For such a finding, until an alloc-test evidence bound to it exists:
   - `dla ledger close-rest` leaves it and prints the full commands in order (sibling when
     missing, `dla ledger test <F> --evidence <workload evidence of that harness>`,
     `dla alloc-test <H> --finding <F>`); never a command that is refused as printed
     (`dla ledger test F9` without `--evidence` was);
   - `dla ledger resolve <F> --as suspected_static_only|unresolved` is refused with the same
     commands as the fix, except `--reason time_budget` at the time floor (the existing
     floor rule) and an aborted run;
   - `dla gate check --final` lists it as missing (a new item kind or an existing one, your
     choice; say which) with the alloc-test command as the fix.
   After the alloc-test the existing rules decide (confirm, refute, or unresolved with the
   alloc-test evidence when it is inconclusive at twice the window).
   Whole-process columns tie to no finding and stay with the residual rows: unchanged.

3. A runtime detector hit that names a site (`detectors` of the run file with `site`; the
   trend's DETECTOR HIT line) is a lead the run must follow: while the finding at that site
   (or, with none, the hit itself) has no measurement verdict, the final gate lists it. When
   the column at the hit's site already passed in this series, the DETECTOR HIT line of
   `dla trend` prints the direct steps for that finding (sibling, `ledger test` with the
   evidence, `alloc-test`), not a new harness; the new-harness command stays for the case
   that the column is missing or short. (That line is built in `trend_cmd.rs` from the
   short-series builder's `detector_site`; change the text and the condition only.)

4. With 1 to 3 in place, check what this run would have met at the end: the queue stays
   open (26 candidates pending, 178 minutes left), `close-rest` leaves them, the final gate
   is not passed, the stop hook blocks. Say exactly which existing rule gives each of these
   and add a test for the chain on a small synthetic run (a harness that ran, a passed site
   column pointing at a finding that is not a target, pending queue candidates, budget left):
   resolve of intake.workload refused; close-rest leaves the finding with runnable commands;
   resolve of the finding refused; gate lists it; after sibling, test, alloc-test evidence
   (you may write the evidence records directly in the test if a real alloc-test is too slow)
   the finding can be confirmed and the gate item goes.

## Tests and checks

Each rule with a test seen failing once. Run the printed commands of rule 2 for real on a
copy of the evidence run directory (`<sp>/u36/axios-4/project/.dla` holds the ledger, series
and harness; the harness paths point at `/root/dla-real/axios-4/project`, which still
exists: copy that project directory to `<sp>/u49/axios-4` and work there, or say what could
not be re-run and why). Then: fmt; clippy on the crates you touched; `cargo test -j 2
-p dla-core -p dla-cli` (name failures that are not yours); `dla schema --check`; the skill
text test if you touch docs; docs (`skills/.../references/*.md` where the rules are
described, `docs/VERIFY.md` rows).

Report: what changed, tests with what was seen failing, the real-directory outputs, what is
unverified, anything wrong you saw outside this brief.
