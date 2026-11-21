# Pass 9 (after pass 8): what the grpc-go run showed (real run grpc-4)

Same rules as pass 8. Evidence: `<sp>/u36/grpc-4/` (run.json, transcript.jsonl, `dla/`:
report.md, ledger.jsonl, state.json, gate/final.json, series/, trend/, queue/, drivers of
H13 H15 H16 H56). The run directory itself is gone: work on copies of these files, and say
what could not be re-run.
`<sp>` = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad
Append to `<sp>/u49/report.md` under a "Pass 9" heading; short final message.

The run found and confirmed the documented leak (F204, `health.Server.updates`). Three things
around it are wrong.

## 9.1 The gate counts columns, the model reads the count

`gate/final.json`: `missing_total` 810 = trend_column 569, residual 137, finding 98,
trigger 3, open_item 2, queue 1. The 569 are whole-process columns (`go.heap_live`,
`os.rss`, `go.total`, ...) of about 150 trends; every one of them has the same fix, the two
open items O13 and its non-heap twin. The 137 residual items point at the same open items.
Of the 98 findings, 64 are queue candidates whose fix is the one batch command. The real
work was: the next queue batch, two open items, three triggers. The model read "810
missing items", tried to stop eleven times and gave up with 146 of 180 minutes left.

Wanted: `dla gate check` (every stage and `--final`), `dla run status` and the stop hook's
text report actions, not dependents:
- items that wait for the same thing are one item with a count and three example ids (all
  whole-process columns and residual rows that wait for one open item are that open item's
  line; all queue candidates are the queue's line; findings with the same fix command
  likewise);
- `missing_total` is the number of such actions; keep the full count in another field
  (`missing_dependents` or similar) and the full list in the stored gate file, so nothing
  is hidden;
- the list is ordered by what to do first (measurement queue, findings whose column grew,
  triggers, open items, the rest), each with one runnable command;
- `go.released` and any other column that never passes as growth (`not_a_growth_signal`)
  is not listed as "not flat".
Check every reader of `missing_total` and of the gate file (stop hook digest, report,
bundle, tests, bench): the digest the stop hook compares must still change when work is
done and not change when it is not.

## 9.2 A registry the program fills at start-up is reported as a confirmed leak

22 findings are `confirmed`; about 18 are registration tables (`balancer.m`,
`resolver.m`, `encoding.registeredCodecs`, `registeredCompressor`, `DefaultMetrics`,
`globalDialOptions`, `globalServerOptions`, `certprovider.m`, `leakcheck.goroutinesToIgnore`
...) that a driver grew by calling the registration function with a new name per unit. The
trigger line says "reproduced through the public API in H56 (varies: the balancer name
...)". The blind verifier reads each of them as N-INTENT and the report prints DISAGREES on
each, while the headline still says "confirmed 22".

Wanted:
a. The trigger of a holder whose only growth path in the entry-path harness is a direct
   call of its own writer (the entry given to `--entry` is the function that inserts, or
   the harness's unit calls that function itself: see what the harness manifest and the
   callers chain record) is not "reproduced through the public API". Give it its own
   reading ("grown only by calling the registration function itself; a leak only if
   run-time input reaches that call"), shown in the report's Trigger line and counted
   separately in the Findings line. Look at the census facts first (`init`, `startup`,
   `exported`, `param` on the writer sites; all in-project callers at start-up with
   constants): when they say the writers are start-up registrations, the driver to-do line
   for such a target should say that the holder must be reached through a run-time entry
   (a request, a message, a connection), not by calling the registration function with new
   names. Do not change the ledger states for this.
b. A finding the blind verifier reads as a non-leak label (N-...) while the finding carries
   a leak label is not counted under plain "confirmed" in the report's Findings line and
   in the final summary the skill prints: count it as "confirmed, disputed by the verifier
   (reads N-INTENT)" (your wording, plain), list it after the undisputed ones, and keep the
   DISAGREES detail. A label disagreement between two leak labels (UNCLASSIFIED vs
   L-COLLECT, as on F204) stays under confirmed. Check the report schema, the bundle and
   both graders' readers of these counts (`bench/runner`), and say what else reads them.

## 9.3 Small things seen

- `finding <id> is tied to growth that passed the gate (probe:.ServiceConfig.Methods of
  <id>) and was never tested`: the probe name has an empty package part
  (`probe:.ServiceConfig.Methods`). Find why and fix or report.
- The stop hook let the model try eleven times: the item count kept changing while
  background trends finished, so the "same digest three times" rule kept starting again.
  With 9.1 the digest is over actions; check that a run whose actions do not change is
  stamped after three attempts even when dependents change.

## Tests and checks

Each rule with a test seen failing once (unique marker, exact reverse). For 9.1 a test that
builds a run with one open item and many dependent columns and asserts one action. For 9.2
tests on a small ledger with a verifier record. Run `dla gate check --final` and `dla
report` on a copy of the evidence ledger if the commands work without the harness
directories (say what they print: the action list, the Findings line) or state why not.
fmt, clippy on what you touched, `cargo test -j 2 -p dla-core -p dla-report -p dla-cli`,
`dla schema --check`, `python3 bench/runner/test_grade.py`, the skill text test, docs and
VERIFY rows.
