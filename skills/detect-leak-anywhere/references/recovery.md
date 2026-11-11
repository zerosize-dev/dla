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
- Record: `dla ledger resolve <O> --as unresolved --reason time_budget|env_unavailable --remedy
  "start a new run for the <runtime> runtime"`. A second `dla ledger merge` is safe.

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
and the missing ids, and the report says so.
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
  open findings block the report either way. Resolve the worker's open findings yourself with reasons and remedies
  (`--as unresolved --reason time_budget|env_unavailable`), and tell the user a new run scoped to that
  runtime would cover it. A stalled worker is not restarted in the same run.
- Record: `dla ledger resolve <O> --as unresolved --reason ... --remedy "start a new run for the <runtime> runtime"`.

## Budget exhausted

A command refuses with `time-budget`. `dla harness run`, `dla alloc-test` and `dla intervene`
estimate their duration from the points already measured and refuse a plan that would end after
the worker's or the run's deadline; the refusal's fix names a smaller plan (`--iterations`,
`--step` or `--extra`). A point that outlasts the deadline is stopped there, and the result says
`partial run`: it is inconclusive and never refutes.
- Remedy: when the fix names a smaller command, run it. When the deadline has passed or the fix
  says even the smallest run does not fit, stop starting long commands and disposition everything
  still open.
- Record: `dla ledger resolve <F> --as unresolved --reason time_budget --remedy "<what to run next
  and how long it needs>"` for each unresolved item (a candidate with only static evidence may be
  `suspected_static_only` with the same reason), then `dla ledger done` in a worker, or
  `dla ledger merge`, `dla report` in the core. Report the run as INCOMPLETE with the list.
