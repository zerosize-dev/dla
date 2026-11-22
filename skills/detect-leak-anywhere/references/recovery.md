# Recovery

What to do when something fails, and what to record so the run stays honest. Each failure leaves a
recorded gap or an open item; none is a reason to claim more than the evidence shows. A refusal
from `dla` prints the fixing command: run it first.

## Missing tool

A backend or tool the runtime could use is not on the machine (`missing` in coverage after
`dla probe`, `blocked` after `dla sweep`).
- Remedy: `dla install <backend>` once, after tier-1 consent, in the core before any dispatch.
- Record: if the install fails or is declined, leave the gap as coverage says; for an item that
  needed it, `dla ledger resolve <F> --as unresolved --reason tool_missing --remedy "install <tool>:
  <command>"`. Never run the tool by hand as a substitute and never install into project source.
- A `dla` command named in these files that the installed release lacks: same, reason
  `tool_missing`, remedy naming the command.

## Failed attach

`dla attach` or `dla sample --pid` fails (no such process, wrong uid, attach socket not writable,
ptrace refused, the tool exits non-zero), or the target exited.
- Remedy: read the printed effects and the error; check the pid with `dla run status` (a process
  DLA started is in its registry); retry once only if the cause is fixed. A JVM attach needs the
  same user and a writable `/tmp`; heaptrack `--pid` is refused while `kernel.yama.ptrace_scope`
  is above 0; memray needs to be importable by the target's own interpreter.
- Record: `dla ledger resolve <F|O> --as unresolved --reason env_unavailable --remedy "<what to
  change on the machine, then rerun dla attach ...>"`; reason `privilege_declined` when the tier
  was not consented. Then `dla run abort --reason "attach failed"` if the run is being abandoned:
  it undoes what was attached and never kills the target.

## Insufficient resources

`dla attach` refuses with `insufficient-resources` (exit 2): the expected dump times 1.2 exceeds free
disk, or the peak extra memory exceeds 0.8 of available memory (a Node heap snapshot costs about
12x the heap in use; a JVM heap dump is about the used heap).
- Remedy: do the cheaper operation (`jcmd histogram`, `nmt`, `thread-dump`, `node-inspector
  memory`), free disk space, or ask the user to run the capture where memory allows and give the
  files back (artifact path). Note a refused Node snapshot may leave the inspector open until the
  target exits.
- Record: the preflight is already in the evidence. Resolve the item `suspected_static_only` or
  `unresolved` with reason `env_unavailable` and a remedy naming the missing disk or memory in
  bytes.

## Truncated output

A summary reports `truncated` (object cap reached, `--max-objects`), a series drops columns beyond
64, or a command output is bounded and points to the full record.
- Remedy: read the full record named by the pointer; for a truncated heap summary rerun with a
  higher `--max-objects` only if memory allows.
- Record: treat the figures as lower bounds, keep the `limitations` text, and say in the report
  that retained sizes are approximate. Never confirm from a truncated summary alone when a smaller
  or cleaner capture is possible.

## Failed binary download

`dla` is not found, or the SessionStart download failed.
- Remedy: restart the session, run `command -v dla`, or use `${CLAUDE_PLUGIN_ROOT}/bin/dla`; the
  shim prints the path and the manual download command. Placing the binary by hand is described in
  the README.
- Record: with no binary no run can be recorded. Tell the user what is missing and the command to
  fix it; do not produce a diagnosis or report in prose.

## Worker killed or over budget

A worker subagent died, was interrupted or passed its deadline without `ledger done`.
- `dla ledger merge` copies its slice (a torn last line is dropped), marks the worker killed or
  over-budget, opens the item "worker <w> did not finish (slice ends at line N)" and the run is
  INCOMPLETE; the worker's open findings keep it so.
- Remedy: `dla run dispatch` refuses a worker name that is already registered, so a killed worker
  is not restarted in the same run. Resolve its open items yourself with reasons and remedies and
  tell the user a new run scoped to that runtime would cover it.
- Record: `dla ledger resolve <O> --as unresolved --reason not_exercised --remedy
  "start a new run for the <runtime> runtime"` for the merge item "worker <w> did not finish" (the
  reason `dla run status` and the gate print for it: it has no finding behind it and nothing
  measured it, so `inconclusive` is refused there; `env_unavailable` when the worker died of an
  environment failure); use `inconclusive` only for the worker's own findings or residual items that
  it measured. A second `dla ledger merge` is safe.

## Integrity mismatch (tamper)

`dla gate check`, `dla report` or `dla run status` names a protected file (`ledger.jsonl`, a worker
slice, `state.json`, `coverage.json`, `report.md`) whose length or hash differs from its seal.
- Remedy: find out what changed it (a restored backup, a merge conflict, a hand edit). If the change was
  not yours and the content is right, tell the user and run `dla run reseal --reason "<what changed and
  why>"` (core only). It appends an `integrity_resealed` record to the ledger, seals the files again,
  and the report lists the reseal. Do not reseal to make a gate pass.
- Record: the reseal is the record. Without it the run stays INCOMPLETE and the report names the file.
- A run created before schema 6 has no seals; it is reported as unsealed, which is not a mismatch.
  `report.md` edited after rendering is simply replaced by the next `dla report`, which says so.

## Stop hook, three stop attempts

The Stop hook blocks the end of the turn while `dla gate check --final` fails. It blocks the first two
stop attempts; at the third attempt with no change in the ledger it lets the stop through, stamps the run
INCOMPLETE with the reason "stop hook let the stop through after 3 stop attempts with no ledger change"
and the missing ids, and the report says so. While a `dla` command of the run (census, sweep, harness run,
trend, alloc-test, report, gap, intervene) is still running, a stop attempt is blocked with the command named
and is not counted (a command running longer than 30 minutes is taken for hung). A stamp does not close the
measurement queue: the time left decides, `dla run status` shows the queue and a `stop_stamp` field, and the next
command that works on the run (not status, gate check, report, bundle or ledger list) lifts the stamp before it
runs; the report keeps it in its `History:` line. An aborted run stays aborted.
- A `harness_not_run` item means a generated harness was never run, or never built: run it (`dla harness run H<n>`; for one that did not build, fix the build error first; a run refused for time is not a run), or record why it cannot run on `intake.workload` with `--runtime <rt>` (the runtime of the harness; it closes the queue of that runtime only), and ask the user first when only they can unblock it.
- A `coverage_contradiction` item means iterations ran and no entry point was exercised: fill the DLA-HOOK bodies of the driver harness so it calls one and run it again; when the harness that produced the zero is a command harness or a JUnit repeat (no hooks), make a driver (`dla harness gen --lang <rt> --driver`), fill its hooks and run the harness it prints; when the harness did not build, fix the build error first; or record on `intake.workload` why it cannot (with `--runtime <rt>`). A `budget_left_unrevisited` item means findings were closed while the budget could still measure them: run the command in its fix, or record the blocker on `intake.workload` with `--runtime <rt>`.
- A `queue` item leads the list while a runtime still has pending ranked candidates: run its batch command (`dla harness gen --lang <rt> --driver --next`, fill the hooks, `dla harness run H<n> --mode both`, `dla trend S<n>`); once the queue is empty or closed, `dla ledger close-rest`.
- Remedy: do not wait for it. Run `dla gate check --final`, fix the items it prints, or resolve each with
  `dla ledger resolve <F> --as unresolved --reason <r> --remedy "<text>"`, then `dla report`.
- Record: if the stamp was already set, the report stays INCOMPLETE; say plainly what was left open and why.

## A worker stalled

The SubagentStop hook blocks a worker's stop while its slice is not done and dispositioned. At the third
stop attempt with no change in the worker's own slice it lets the worker go and marks it stalled (writes by
other workers do not count as a change).
- Remedy: `dla ledger merge` copies the slice anyway. If the worker never ended its slice it opens the item
  "worker <w> did not finish" and the run is INCOMPLETE; if it ended the slice (`ledger done`) with findings
  or open items still undispositioned, it opens the item "worker <w> stalled: ..." instead. The worker's
  open findings block the report either way. Resolve the worker's open findings yourself: `dla ledger close-rest` for the candidates its queue has finished with (a pending one in an open queue is measured first, as for the core's own), else with reasons and remedies
  (`--as unresolved --reason not_exercised` when nothing measured it, the reason `dla run status` prints; `inconclusive` when
  the worker measured it; `env_unavailable` only when an environment failure stopped it), and tell the user a new run scoped to that
  runtime would cover it. A stalled worker is not restarted in the same run.
- Record: `dla ledger resolve <O> --as unresolved --reason ... --remedy "start a new run for the <runtime> runtime"`.

## Budget exhausted

A command refuses with `time-budget`. `dla harness run`, `dla alloc-test` and `dla intervene`
estimate their duration from the points already measured and refuse a plan that would end after
the worker's or the run's deadline; the refusal's fix names a smaller plan (`--iterations`,
`--step` or `--extra`). A point that outlasts the deadline is stopped there, and the result says
`partial run`: it is inconclusive and never refutes.
- Remedy: when the fix names a smaller command, run it. When the deadline has passed or the fix
  says even the smallest run does not fit, treat it as at the floor; more time is the other fix: `dla run extend <dur>`
  (core only; `dla run status` gives `budget.to_finish_minutes` and `budget.to_finish_findings`, your n). While more than the floor (the larger of 10 minutes and 20% of the budget) is left, never ask
  whether to continue and never close `time_budget`. At or under the floor, or when nothing smaller fits, the core of an
  interactive run asks the user once: "extend by <to_finish> minutes to measure the next <n>
  candidates?"; on yes extend and carry on; on no, or no answer, stop starting long commands and
  disposition everything still open. A worker, or a run whose `dla run init` reported a `consent`
  object (unattended), never extends: it closes `time_budget` with its remedy.
  A worker whose deadline passed cannot ask: the core waits on it and runs no commands. It closes what it cannot measure `time_budget` and says so in its hand-off; only the core extends, after the merge. An `auto` budget is sized
  at the first passing gate check from `sweep` on (in a dispatched run by each worker's first one, raising
  it only), so a run that stalls before the sweep still has the 60 minutes it started with.
- Record: `dla ledger resolve <F> --as unresolved --reason time_budget --remedy "<what to run next
  and how long it needs>"` for each unresolved item that the budget cut off (the ledger refuses `time_budget` while more than the larger of 10 minutes and 20% of the budget remains, unless a command of the core (of the worker, in a worker) was refused with nothing smaller that fits after the finding was added; a finding that was measured and left undecided is `inconclusive`, one nothing reached `not_exercised`) (a candidate with only static evidence may be
  `suspected_static_only` with the same reason), then `dla ledger done` in a worker, or
  `dla ledger merge`, `dla report` in the core. Report the run as INCOMPLETE with the list. The
  report's `Time:` line says how many findings were closed for lack of time and how many minutes
  measuring them needs; pass that to the user as the size of a new run (`--budget <m>m`).

## A candidate is pending (`target-pending`)

`dla ledger resolve F --reason not_exercised|no_harness|time_budget` is refused with `target-pending`
while F is ranked for measurement, no targeted workload has settled it, and the queue of its runtime is
open (budget over the floor, workload not blocked, candidates still pending). The message gives F's
rank, the minutes left and the fix.
- Remedy: run the batch command it prints (`dla harness gen --lang <rt> --driver --next`; a JVM project
  with JUnit 5 tests gets `dla harness gen --lang jvm --junit --next` first, the project's own tests in a
  loop, nothing to fill, then the driver for the candidates with no test), fill `setup` and
  `unit` of the driver so each unit reaches the listed targets with a new key, `dla harness run H<n>
  --mode both`, `dla trend S<n>`, read `targets`. Never answer the refusal with another reason, a blocker
  reason you do not have, or a loop of `ledger resolve` calls.
- A candidate settled by one flat run (`executed`) stays in the queue as a second look while budget is left and
  the first pass is over: run the batch command, write a different unit (see the block), a `VARIES.txt` line per
  target, run, trend. The unit hook is a skeleton of five slot variables taken from the key with the driver's helper: use each (or give one up in the form `slot <name>: not applicable: <reason>`, two at most); the run refuses an unused slot, naming it and what it means. Closing it is refused the same way until a different driver has looked.
- A target a harness missed (`constant`, `not_loaded`, `not_executed`, `unknown`) is refused too, until a
  focus harness (at most 3 targets) missed it as well with a different driver: the fix names the focus
  batch (`--next` when it is the best-ranked pending candidate). Call the writer function the block names
  with a new key; a target no driver can reach is given up with `--as unresolved --reason env_unavailable
  --remedy "<what it needs>"` (a reason that says what blocks it is accepted as before).
- When the queue is empty or closed, `dla ledger close-rest` closes the rest (`inconclusive` for a candidate a declared harness measured with verdict `constant`, `changed` or `executed`, else `not_exercised`, or `time_budget` at the floor). `dla ledger list` shows each
  candidate's queue status (`pending`, `settled`, `not-eligible`); never read `ledger.jsonl` by hand.
- When no driver can be built or run for the runtime (no toolchain, the build fails, a privilege was
  declined): `dla ledger resolve intake.workload --status unresolved --reason
  <env_unavailable|tool_missing|build_fails|privilege_declined> --remedy "<what blocks it>" --runtime <rt>`.
  The blocker names one runtime and closes the queue of that runtime only; the queues of the other
  runtimes stay open and `dla ledger close-rest` leaves their candidates pending. `--runtime` is required
  when more than one runtime has ranked candidates. It is refused (`workload-ran`) for a runtime whose
  harness already wrote a series: that workload ran. Use it only when it is true.
- Record: the batch you ran. A `constant`, `not_loaded` or `not_executed` target is run once more with a
  fixed unit; two driver runs with different driver files settle a candidate whatever the verdicts.

## Too many targets (`too-many-targets`)

`dla harness gen` was asked for more than 8 targets (`--probe` and `--next` together count; the siblings `--next` adds for the owners of its targets do not, up to 16 in all).
- Remedy: declare at most 8 per harness: `--next 5 --probe F12`, or fewer `--probe` flags; the rest go
  in the next harness (`dla harness gen --lang <rt> --driver --next`).

## A finding cannot get an entry-path harness (`entry-not-grown`)

`dla harness gen --driver --entry F` is refused with `entry-not-grown` when F is a non-leak explanation,
or has not grown in any harness: the trigger is asked for a holder that grows when a harness feeds it
keys.
- Remedy: measure it first with a hook driver (`dla harness gen --lang <rt> --driver --probe F`, fill
  the hooks, `dla harness run H<n> --mode both`, `dla trend S<n>`); when it has `grew`, `--entry F`.
  More than 8 findings are refused `too-many-targets`: give at most 8, the rest in the next harness.
  `--entry` needs `--driver` and excludes `--probe` and `--next` (`invalid-value`).

## An entry-path harness has no `VARIES.txt` line (`varies-missing`)

`dla harness run H<n>` refuses an entry-path harness whose `harness/H<n>/driver/VARIES.txt` is empty or
still the placeholder.
- Remedy: write one line in that file: what a caller varies per unit (for example: the request path, a
  different one per unit; unknown paths, which the project rejects). It is the trigger text of the report;
  if a unit creates handlers, functions or configuration per call, say so in it. Then `dla harness run
  H<n> --mode both`.

## A confirmed leak has no trigger (item of kind `trigger`)

`dla run status`, the final gate and the stop hook list `F8 grew when the harness fed it new keys;
whether a caller of the public API can do that is not shown` for a confirmed or strong leak-side finding
while the queue is on and more than the floor of the budget is left.
- Remedy: the fix's `dla harness gen --lang <rt> --driver --entry F8,...`, fill the hooks with public
  entry points only, write VARIES.txt, run, trend. After one harness without growth the item asks for a
  second, different one. Never close the finding, answer with another reason or call the leak disputed
  because the trigger is open; after two entry-path harnesses without growth the item is satisfied and
  the report says `Trigger: not found`.

## Tempting moves

Moves that feel like progress and are not. Where a gate also refuses the move, its refusal names the same step.

| The move | What to do instead |
|---|---|
| Many static findings, budget unspent: close them `suspected_static_only` and report. | Static-only is not done. Work the queue: `dla harness gen --lang <rt> --driver --next`, fill the hooks, `dla harness run H<n> --mode both`, `dla trend S<n>`; then `dla ledger close-rest` (stages.md, sections 4 and 9). |
| One generic workload ran, so close every candidate it did not move. | It reached only what its units call. Each candidate needs a unit that reaches its writers with a new key; `target-pending` refuses the close until a batch targeted it. |
| The first confirmed finding grew under a driver that feeds new keys, so call it a leak, or call it disputed because real use may not do that. | Growth under direct injection shows an unbounded holder. Run `dla harness gen --driver --entry F`: reproduced is a leak with its trigger; no trigger found after two different entry-path drivers is "an unbounded holder, no trigger found". |
| Ask the user "should I continue?" with time left. | Continue. Ask only at or under the floor, to extend (Time budget in SKILL.md), or for what only the user can supply (a service, a credential, data), once, naming the finding ids and the exact input; an unattended run records the blocker with a remedy instead. |
| A finding explains why memory is high, but nothing grows. | Record it as the `N-*` sibling, or an UNCLASSIFIED note; never confirm it as a leak. |
| No closing reason fits. | Take the nearest real one (what was not reached, or what blocked it) with a remedy; never `time_budget` while more than the floor is left. |
| Print the class path with Gradle, Maven or `java -cp` to run a JVM harness. | `dla harness run` resolves it; on a build failure follow references/jvm.md. |
