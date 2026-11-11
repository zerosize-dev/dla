# Grading

Two graders implement this document independently: `dla-bench grade` (Rust)
and `bench/runner/grade.py` (Python, standard library only). On the same
results directory they must write identical `metrics.json` files, compared as
parsed JSON. Any difference is a bug in a grader or an ambiguity here; fix
the ambiguity here first.

## Inputs

```
<results>/runs/<ablation>/archetypes-<on|off>/<fixture>/<n>/
  run.json        always present (written by the runner)
  ledger.jsonl    present when the run used dla
  findings.json   present when the model wrote one
  coverage.json   present when dla wrote one
```

A finished run that used `dla` also holds `report.md` and `coverage.json`
(the M7 metrics read both; see "M7 metrics"). A run with several workers also holds `ledger.<worker>.jsonl` (the slices) and
`bundle/<id>/index.json`. The graders read neither: the merged `ledger.jsonl`
is the only ledger input, and a run that was killed before the merge is graded
on whatever `ledger.jsonl` holds.

`<ablation>` is `no-skill`, `prose`, `gates` or `full`; `<n>` is the run index
starting at 1. Fixtures are read from `bench/fixtures/<fixture>/expected.toml`
(override with `--fixtures <dir>`). A run directory whose fixture has no
`expected.toml` is an error.

Directory rules: a missing `runs/` directory is an error. Under `runs/`, a
first-level directory that is not one of the four ablation names is ignored,
a second-level directory not named exactly `archetypes-on` or
`archetypes-off` is ignored, and so is a run directory whose name is not a
positive decimal integer without leading zeros. A run directory without
`run.json` is an unfinished run: it is skipped and counted in
`usage.incomplete`. A `run.json` that exists but is not a JSON object is an
error. A group exists when it has at least one run directory, finished or
not; a group whose runs are all unfinished is written with `runs` 0. Only
fixtures with at least one finished run need an `expected.toml` and appear
under `fixtures`.

Encoding: every input file is UTF-8. A `run.json` or `expected.toml` that is
not valid UTF-8 is an error; a `findings.json` or `coverage.json` that is not
valid UTF-8 is treated as absent. `ledger.jsonl` is split into lines on the
byte `\n` and each line is decoded on its own; a line that is not valid UTF-8
is skipped like any other line that is not a JSON object. "Trimming
whitespace" means removing leading and trailing ASCII space, tab, carriage
return and line feed only.

`run.json` fields used here: `exit_code` (integer), `timed_out` (bool),
`wall_seconds` (number), `cost_usd` (number or null) and `usage` with
`input_tokens`, `output_tokens`, `cache_read_input_tokens`,
`cache_creation_input_tokens` (integers). A missing or non-integer token
field counts as 0, a missing or non-numeric `wall_seconds` counts as 0, and
a non-numeric `cost_usd` is treated as absent.

Fixture fields: a leak without `confirmable` is confirmable (default `true`).
The expected `site` path and `alt_sites` paths are used exactly as written.
The expected `site` is parsed with the same expression as a finding's `site`
(below), so a trailing `:column` is ignored; a line number, in a `site` or an
`alt_sites` entry, that does not fit in 64 bits gives no location.
An `alt_sites` entry that is not `path:line` or `path:start-end` is ignored.
A leak may have no `site`; it then has an `owner` (a `package.Class.field`
string). `entry` is `repository` (default), `artifact` or `symptom`; only
`artifact` matters here (see `source_overclaim_rate`). `runtime`, `runtimes` and
a leak's own `runtime` are not used for grading.

Every float sum in this document is taken in ascending order of fixture id,
then run index.

## Normalised findings

Every run is reduced to a list of findings with these fields:
`state` (one of `confirmed`, `strong`, `refuted`, `suspected`, `unresolved`,
`open`), `site` (string or null), `owner` (string or null), `holder` (string
or null), `label` (string or null), `worker` (string or null), `hypothesis`
(bool) and `discriminator` (bool).

**Source selection.** If `ledger.jsonl` exists and contains at least one
`finding_added` record, the source is `ledger`. Otherwise, if `findings.json`
exists and is a JSON object whose `findings` member is an array, the source
is `findings`. Otherwise the source is `none` and the list is empty.

**From `ledger.jsonl`.** Read the file line by line. Skip a line that is not
a JSON object or has no string `type`. Apply records in file order.

A finding id may carry a worker prefix (`jvm/F3`), and a line may carry a
`worker` member. The **key** of a finding id `i` on a line is `i` when `i`
contains `/` or the line has no `worker` member that is a non-empty string;
otherwise it is the line's `worker`, then `/`, then `i`. Every id below (the
finding's own `id`, a record's `finding`, `canonical` and `duplicate`) is
turned into a key this way, using the line it appears on, and findings are
looked up and replaced by key. The finding's **worker** is the text before the
first `/` of its key, or null when the key has no `/` or that text is empty.

- `finding_added`: create a finding from the record's `finding` object with
  state `open`, `site`, `owner`, `holder` and `label` taken from its `site`,
  `owner`, `holder` and `archetype` members (null when absent or not a
  string), and
  `hypothesis` true when its `parent` member is a non-empty string. Key it by
  its `id`. A repeated `id` removes the earlier finding and adds the new one at
  the position of the later line. A record whose `finding` is not an object or
  whose `id` is not a string creates no finding but still counts as a
  `finding_added` record for source selection.
- `transition`: if a finding with the record's `finding` id exists, set its
  state from the record's `to`: `confirmed` → `confirmed`, `strong` →
  `strong`, `refuted` → `refuted`, `suspected_static_only` → `suspected`,
  `unresolved` → `unresolved`, anything else → `open`.
- `finding_annotated`: if a finding with the record's `finding` id exists,
  set its `holder` from the record's `holder` when the finding's `holder` is
  null and the record's is a string, and likewise `site` and `owner` (this is
  how `dla ledger annotate --holder/--site/--owner` fills a value the finding
  was added without; a value that is already set never changes), except that a
  record with a `replaced_holder` member (a ledger schema 5 holder correction,
  backed by heap-graph growth evidence) sets the `holder` from the record's
  `holder` when that is a string, whatever the finding held.
- `finding_labeled`: if a finding with the record's `finding` id exists and the
  record's `label` is a string, set the finding's `label` to it (this is how
  `dla ledger label` assigns an archetype, schema 6; the later of two records
  for one finding wins; a `label` that is not a string changes nothing).
- `finding_merged`: if the records' `canonical` and `duplicate` are strings
  that give two different keys, and a finding exists under both, the finding
  under the `duplicate` key is removed and the one under the `canonical` key
  keeps its own state and fields unchanged, so the claim is counted once.
  Otherwise the record has no effect. A later `finding_added` with the
  duplicate's key creates a finding again.
- `verification`: changes no finding. It is read for the verification metrics
  (see "Verification"). It counts when `finding` is a string, `agrees_site` and
  `agrees_holder` are booleans, and `agrees_label` is a boolean, null or absent;
  any other `verification` record (the form without the agreement flags) counts
  for nothing. The key of its finding is built like the keys above. The later of
  two records for one finding wins. A record whose finding key matches no finding
  at the end of the replay (it never existed, or `finding_merged` removed it)
  counts for nothing.
- Every other record type is ignored, in particular `slice_started`,
  `slice_done` and `slice_merged`: they change no finding and no metric.

For this source, `discriminator` is true exactly when the final state is
`refuted` (the ledger refuses `refuted` without a discriminator). Findings
keep the order of their `finding_added` lines.

**From `findings.json`.** Each array element that is a JSON object becomes a
finding in array order; other elements are skipped. `verdict`
`leak-confirmed` → `confirmed`, `leak-suspected` → `suspected`, `not-a-leak`
→ `refuted`, anything else → `open`. `site` and `holder` are taken when they
are strings. `label` is `cause` when it is a string matching
`^(L-[A-Z]+|N-[A-Z]+|UNCLASSIFIED)$`, else null. `hypothesis` is false.
`discriminator` is true when the verdict is `not-a-leak` and `discriminator`
is a string that is non-empty after trimming whitespace. `owner` is taken when
it is a string; `worker` is null.

A finding is a **claim** when it is not a hypothesis and its state is
`confirmed` or `strong`. It is a **rule-out** when its state is `refuted` and
`discriminator` is true (hypotheses included).

## Matching a finding to an expected leak

A leak with a `site` is matched by site, as follows. A leak without a `site`
is matched by owner (below); an `owner` on a leak that has a `site` is not
used for matching.

Parse the finding's `site` with `^(.*?):([0-9]+)(?::[0-9]+)?$`, matched
against the whole string (a trailing newline does not match), into a path and
a line; a site that does not match has no location. Normalise the path by
replacing every `\` with `/` and removing every leading `./`. It matches the
leak when the path equals the leak's path or ends with `/` followed by the
leak's path, and the line lies inside the leak's `lines` (default
`[line, line]`). It also matches when the same path rule and line test hold
for any entry of the leak's `alt_sites`; an entry is `path:line` (range
`[line, line]`) or `path:start-end` (range `[start, end]`).

**Owner matching.** A leak without `site` matches a finding whose `owner` is
not null and, after trimming whitespace from both, equals the leak's `owner`
or ends with `.` followed by the leak's `owner`. The comparison is exact and
case-sensitive; a finding with no `owner` never matches. A leak with neither
`site` nor `owner` matches nothing. The finding's `site` plays no part in an
owner match.

A leak is **found** in a run when some non-hypothesis finding in state
`confirmed` or `strong` matches it, and **flagged** when some non-hypothesis
finding in state `confirmed`, `strong`, `suspected`, `unresolved` or `open`
matches it. The **chosen finding** of a found leak is the first matching
non-hypothesis finding in state `strong`, or else the first matching
non-hypothesis finding in state `confirmed`.

## Metrics

Metrics are computed per **group**: one ablation with one archetypes setting.
Every ratio metric is accumulated as an integer numerator and denominator.
The *leak set* of a run is every expected leak with `confirmable = true` of a
fixture whose category is `archetype-leak`, `archetype-free` or `multi-leak`.

| Metric | Denominator | Numerator |
|---|---|---|
| `leak_recall` | leaks in the leak set | found |
| `archetype_free_recall` | leaks in the leak set with `cause = "UNCLASSIFIED"` | found |
| `flagged_recall` | leaks in the leak set | flagged |
| `second_leak_recall` | for `multi-leak` runs with at least one leak of the leak set found: (leaks in the leak set) − 1 | (found in the leak set) − 1 |
| `grounded_attribution` | found leaks in the leak set | chosen finding's `holder` equals the leak's `holder` |
| `label_accuracy` | found leaks with `cause` other than `UNCLASSIFIED` whose chosen finding has a non-null label | label equals `cause` |
| `forced_fit_rate` | found leaks with `cause = "UNCLASSIFIED"` whose chosen finding has a non-null label | label is not `UNCLASSIFIED` |
| `overclaim_rate` | runs of `impossible` fixtures | runs with at least one claim |
| `false_confirmation_rate` | runs of `decoy-control` and `non-leak` fixtures | runs with at least one claim |
| `non_leak_precision` | runs of `non-leak` fixtures | runs with no claim and at least one rule-out |
| `trap_miss_rate` | runs of `archetype-leak` fixtures that have a `pair` | runs where some leak with `confirmable = true` is not found |
| `trap_false_alarm_rate` | runs of `decoy-control` fixtures | runs with at least one claim |
| `source_overclaim_rate` | runs of fixtures with `entry = "artifact"` | runs with at least one claim whose `site` is not null and not empty after trimming |
| `undiscriminated_refutations` | refuted findings (hypotheses included) | refuted findings without a discriminator |
| `uncited_findings` | findings that are not hypotheses, in runs graded from the ledger | those with no evidence ID (see "M7 metrics") |
| `uncited_checklist_answers` | checklist items with an answer other than `unresolved`, in runs graded from the ledger | those whose latest answer cites no evidence ID |
| `dropped_rate` | runs graded from the ledger whose `report.md` says `Status: COMPLETE` | those that still hold an undispositioned finding or an unresolved residual |

Each ratio metric is written as:

```json
{"num": 7, "den": 12, "value": 0.583333,
 "per_run": [{"run": 1, "num": 2, "den": 4, "value": 0.5}, ...],
 "mean": 0.583333, "stdev": 0.058926}
```

- `value` is `round6(num / den)`, or null when `den` is 0.
- `per_run` has one entry for every run index from 1 to the largest index in
  the group, in ascending order, computed over that run index of every
  fixture.
- `mean` and `stdev` use the unrounded per-run ratios `num / den` of the
  entries whose `den` is not 0, in ascending run order: `mean = round6(s / k)`
  where `s` is their sum and `k` their count; `stdev = round6(sqrt(q / k))`
  where `q` is the sum of the squared differences from the unrounded `s / k`.
  Both are null when `k` is 0.
- `round6(x)` rounds to six decimal places, half to even, on the exact binary
  value: Python `float(f"{x:.6f}")`, Rust `format!("{x:.6}").parse::<f64>()`.

`unexplained_growth_share` is written as
`{"value": round6(R / T) or null, "runs": k}`, where `R` and `T` sum, over the
runs whose `coverage.json` is a JSON object with a non-empty `residual`
array, every row's `residual` and `max(total_slope, 0)` (a missing or
non-numeric value counts as 0; a row that is not an object is skipped), and
`k` counts those runs. A `coverage.json` that is not valid JSON is treated as
absent. `value` is null when `k` is 0 or `T` is 0.

A region whose total slope is negative (the series shrinks there) has no growth
to explain, so its total counts as 0. Before M7 the negative total lowered `T`
and the share passed 1. The M6 acceptance run of `dla-bench dynamic` showed
`residual_share` values of 1.0009 to 1.3317 (the driver's own share of one
fixture, not this grader metric; both had the defect and both are fixed). The
share is now at most 1.

## Usage

Per group: `runs` (count of graded runs), `incomplete` (run directories
skipped for lack of `run.json`), `errored` (runs whose `exit_code` is missing,
not an integer or not 0, or whose `timed_out` is true), `sources` (`{"ledger": n, "findings": n, "none": n}`),
for each of the four token fields `{"sum": n, "mean": round6(sum / runs)}`,
`wall_seconds` as `{"sum": round6, "mean": round6, "max": round6}` (every
`mean` and `max` is null when `runs` is 0) and
`cost_usd` as `{"sum": round6, "mean": round6}` over the runs with a numeric
`cost_usd` (`{"sum": 0.0, "mean": null}` when there are none).

## Verification

The independent verifier (`agents/verifier.md`) reads a finding's evidence
bundle without the finding's site or label and `dla ledger verify` records what
it concluded. These counts measure how often it was asked and how often it
agreed. They are not part of the seventeen ratio metrics and are informational.

Per run whose findings come from the ledger, a **claim** is a finding that
passes `is_claim` (confirmed or strong, not a hypothesis). A claim is
**verified** when it has a counting `verification` record. Over all runs of a
group:

- `claims`: the number of claims; `verified`: the number of verified claims.
- `verified_share`: `{num: verified, den: claims, value}`.
- `site_agreement`: verified claims whose record has `agrees_site` true, over
  verified claims.
- `holder_agreement`: the same for `agrees_holder`.
- `label_agreement`: verified claims whose record has `agrees_label` true, over
  verified claims whose `agrees_label` is a boolean (a record with null or no
  `agrees_label` is left out of both numerator and denominator).

Each is `{"num": n, "den": n, "value": round6(num / den)}` with `value` null
when `den` is 0. The group has a `verification` member only when at least one
run of the group has a counting `verification` record of a finding that exists
at the end of the replay (verifications of refuted findings and hypotheses
count for that rule); otherwise the member is absent. Runs whose findings come
from `findings.json` or from nothing contribute zero everywhere.

## Per fixture

Per group and fixture: `category`, `runs`, `claim_runs` (runs with at least
one claim), `rule_out_runs` (runs with at least one rule-out), and `leaks`
mapping every expected leak `id` (confirmable or not) to
`{"found": n, "flagged": n, "grounded": n}` counted over runs (`grounded`:
found and the chosen finding's holder matches).

Every fixture also has `tokens_total` (the sum over its runs of the four token
fields, `input + output + cache_read + cache_creation`) and `agreement` (see
"Agreement and close calls").

When at least one found leak's chosen finding has a worker, the fixture also
has `worker_provenance`: an object mapping the leak `id` to an object that maps
each worker name to the number of runs in which that leak was found with a
chosen finding of that worker. Leaks with no worker are left out, and the
member is absent when empty. It is informational and part of no metric.

## M7 metrics

Four ratio metrics, per-fixture agreement and per-row decoy results were added
in M7. Both graders compute them; a run whose findings do not come from the
ledger has `(0, 0)` for the three that read the ledger.

**What is read.** Besides `ledger.jsonl`, a run directory holds `report.md`
(the rendered report) and `coverage.json`, collected by the runner. Both are
read as UTF-8; a file that is not valid UTF-8 is treated as absent.

**`undiscriminated_refutations`.** Over every finding in state `refuted`,
hypotheses included: those whose `discriminator` is false. From the ledger a
refuted finding always has one (the ledger refuses `refuted` without a
discriminator), so the metric is non-zero only for `findings.json` runs whose
`not-a-leak` verdict has a blank `discriminator`.

**`uncited_findings`.** Over the findings that are not hypotheses, in runs
graded from the ledger: those that cite no evidence ID. A finding cites the
non-empty strings in the `supports` and `contradicts` arrays of its
`finding_added` record and in the `evidence` array of each `transition` to it.
A repeated `id` starts again with its new record. A `finding_merged` record
adds the evidence of the duplicate to the canonical finding (when both exist
and differ) and removes the duplicate, which is then not counted. Keys are the
worker-qualified keys of "Normalised findings".

**`uncited_checklist_answers`.** Over the checklist items that have an answer, in
runs graded from the ledger. A `checklist_answer` record has `item`, `status`
and `evidence`; the later record of an item replaces the earlier one. Items
whose latest answer has `status` `unresolved` are left out of both counts: an
unresolved answer cites a reason and a remedy and has no evidence by design. The
rest count as uncited when their `evidence` has no non-empty string.

**`dropped_rate`.** Over the runs graded from the ledger whose `report.md` is
COMPLETE, that is, whose first line starting with `Status:` (after trimming
space, tab and carriage return) is exactly `Status: COMPLETE`. A run counts as
dropped when, after the replay,

- some finding is still in state `open` (a candidate or under test; hypotheses
  included, merged duplicates not), or
- `coverage.json` has a `residual` row (an object with numeric `residual` and
  `threshold`) with `residual > threshold` and no resolved open item. The row
  names its item in `open_item`; the item is resolved when the ledger has an
  `open_item` record and an `open_item_resolved` record for that key. When the
  row's ID has no `/` (a worker raised it as `O1`; the merge renamed it
  `jvm/O1`) a resolved item with a `/`, the same number and the text
  ` in <region> (` also covers it. This is the reading of `gate check --final`.

A COMPLETE report cannot be written while a finding or an item is open, so a
non-zero count means the report and the ledger disagree.

## Agreement and close calls

Each fixture block has `agreement`:

```json
{"runs": 3, "unanimous": false,
 "outcomes": [{"run": 1, "outcome": "found:L1,L2"}, {"run": 2, "outcome": "found:L1"}, ...]}
```

The **outcome** of a run is `found:<ids>` for a fixture of category
`archetype-leak`, `archetype-free` or `multi-leak`: the IDs of the confirmable
leaks that are found, in fixture order and comma separated, or `found:none`. For
every other category it is `claim` or `no-claim` (some claim, or none).
`unanimous` is true when all outcomes are equal (a fixture with one run is
unanimous).

A **close call** is a fixture of one arm (an ablation with one archetypes
setting) that is not unanimous. It gets two more runs, 4 and 5.
`dla-bench close-calls` lists them and prints the command that adds the runs.
The extra runs count in every pooled count and in the decoy results; the means
of the gates use runs 1 to 3 only (see "Release gates"), because runs 4 and 5
cover only the close-call fixtures.

## Decoys per archetype row

A group has `decoys` (absent when empty): one entry per archetype row (a `cause`
other than `UNCLASSIFIED` of a confirmable leak) of an `archetype-leak` fixture
that has a `pair`:

```json
"L-COLLECT": {"pairs": ["jvm-order-cache/jvm-order-cache-bounded"],
              "leak": {"found_runs": 3, "runs": 3},
              "decoy": {"claim_runs": 0, "runs": 3}, "pass": true}
```

`leak.runs` counts the graded runs of the leak fixtures of the row and
`leak.found_runs` those in which every confirmable leak is found; `decoy.runs`
counts the runs of the decoy fixtures named by `pair` and `decoy.claim_runs`
those with at least one claim. `pass` is true when both sides have runs, the
leak was found in every run and no run of the decoy claimed. A row with several
pairs adds them up. A decoy without runs in the group leaves the row failing.

## Release gates

`dla-bench gates <metrics.json>` evaluates the ten gates of the plan and exits 1
when one fails (2 on an unreadable input). The arms:

- Gates 1 and 3 to 8 judge the shipped arm, `--ablation` (default `full`) with
  archetypes on.
- Gate 2 compares `--recall-ablation` (default `gates`) with archetypes on and
  off; `gates` is the only ablation that has an archetypes-off arm.
- Gate 9 reports every arm and judges the shipped arm.
- Gate 10 judges every arm that is in both the metrics and the previous
  release.

Means and standard deviations are taken over the **base
runs**, per-run entries 1 to 3 that have a denominator, with the formulas of "Metrics".
Counts (zero-claim gates, decoys) use every run, close-call runs included.

| # | Gate | Passes when |
|---|---|---|
| 1 | No overclaiming | `overclaim_rate` has 0 claims over at least one impossible-fixture run |
| 2 | Archetypes never cost recall | in `--recall-ablation`, the mean `leak_recall` and the mean `archetype_free_recall` with archetypes on are not below those with archetypes off; each has 3 base runs with data in both arms |
| 3 | Second leaks are found | the mean `second_leak_recall` is at or above its baseline |
| 4 | No forced fits | the mean `forced_fit_rate` is at or below its baseline |
| 5 | Decoys hold | every `decoys` row passes; every `L-*` table of `--rows` (default `data/archetypes.toml`, skipped when absent) has a row; at least one row exists. Metrics graded before M7 have no `decoys` block: the gate says so and asks for a new grading |
| 6 | Non-leaks are ruled out with evidence | the mean `non_leak_precision` is at or above its baseline, and `undiscriminated_refutations` is 0 |
| 7 | Every claim is cited | `uncited_findings` is 0 over at least one finding, and `uncited_checklist_answers` is 0 |
| 8 | Nothing is dropped | `dropped_rate` is 0 over at least one COMPLETE report |
| 9 | Context cost | the shipped arm's tokens per run on the smoke subset are at most 110% of the previous release's; the other arms are printed, not judged; `--skill` (default `skills/detect-leak-anywhere/SKILL.md`) has fewer than 200 lines. A smoke subset with runs but no token counts (metrics graded before `tokens_total`) fails with "the metrics do not have it" |
| 10 | Regression | in every arm present in both files, no compared metric is worse than the previous release's by more than the tolerance below; no arm in common fails |

A baseline that is missing fails gates 3, 4 and 6 (except in the run that writes
it, see below); a metric without data fails its gate. Gates 9 (the token part) and 10 pass with a note when the baseline file
has no previous release: v1 has none, and its baseline is its own measurement.

**Baselines set in M7.** Second-leak recall, forced-fit rate and non-leak
precision baselines are the base-run means of the M7 baseline run. They never
get looser: `--write-baseline` keeps the stricter of the stored and the new
value (higher for the two recalls and the precision, lower for the forced-fit
rate). A stored value is replaced only by a stricter one.

**Context cost.** Tokens per run are `input + cache_creation + cache_read +
output` from the `usage` block of `run.json`, as the sum `tokens_total` of the
fixture blocks. The smoke subset is `native-vendor-codec`, `python-report-memo`
and `python-report-memo-bounded` (`--smoke`). Tokens per run of an arm are the
summed `tokens_total` of the smoke fixtures that have runs, over their runs.

**Regression.** A metric is worse than the previous release when its base-run
mean is worse by more than `max(2 x the larger of the two standard deviations,
1 / d)`. `d` is the denominator of one run in the previous release (its mean
per-run denominator). Higher is better for the recalls, grounded
attribution, label accuracy and non-leak precision; lower is better for every
other metric. A metric without data in either release is skipped.

**Baseline file.** A JSON file, committed:

```json
{"schema_version": 1,
 "baselines": {"second_leak_recall": 0.8, "forced_fit_rate": 0.1, "non_leak_precision": 1.0},
 "release": {"label": "1.0.0-rc.1", "ablation": "full",
             "arms": {"full/on": {"leak_recall": {"mean": 1.0, "stdev": 0.0, "den_run": 28.0}}},
             "tokens_per_run": {"full/on": 1234567.0}}}
```

`--baseline FILE` judges against it. `--write-baseline FILE` first judges the
metrics against the values FILE already holds (and `--baseline G` when given:
the stricter of the two per value, the previous release from `G`, else from
FILE), then writes the baselines (tightened as above) and replaces the
`release` block with these metrics, unless `--baselines-only` is given;
`--label` names the block. When any gate fails nothing is written: the command
exits non-zero and says which gates failed, so a second identical run cannot
pass against what a failing run wrote. The exception is `--baselines-only`, the
M7 baseline run: it writes only the three baselines (tightened as above) even
when a gate fails, and keeps the previous-release block of FILE itself, never
the one of `--baseline`. The release block holds the statistics of every arm
and the smoke tokens of every arm; it is not written from metrics without token
counts. A baseline value FILE does not hold yet is set by this run: its gate
passes with a note ("this run sets it"). The first baseline run therefore
passes gates 3, 4 and 6; every later run is judged against what the first one
stored.

## Commands

- `dla-bench close-calls <results|metrics.json> [--arm A[/on|off]]... [--json]`:
  the close calls of each arm and the `dla-bench run ... --fixtures <ids> --runs 5
  --out <results>` command that adds runs 4 and 5. `dla-bench run` skips run
  directories that already hold a `run.json`, so only runs 4 and 5 start. The
  command repeats `--model` from the `run.json` files of the earlier runs (the
  runner does not check it; when the runs used several models the command has
  no `--model` and the output says so) and `--fixtures-dir` when `--fixtures`
  is not the default.
- `dla-bench gates <metrics.json> [--baseline F] [--write-baseline F] ...`: above.
- `dla-bench calib-extract <log-dir> --out FILE [--fixtures DIR] [--meta K=V]...`:
  reads the `--log-dir` tree of `dla-bench dynamic` and writes one JSON file for
  the calibration of the gate thresholds. Per fixture it holds `expected` (category,
  pair, leaks, non-leaks), `trends` (series, thresholds, the harness's iterations,
  warm-up and step), `columns` (one row per trend column: `n`, `p_increasing`,
  `slope`, `sen_lo`, `sen_hi`, `half_first`, `half_second`, `half_ratio`, `noise`,
  `rise`, `rise_over_noise`, `shape`, `gate`, `gate_failed`, `candidates`, `tied`
  and `expected_gate`), `alloc` (one entry per alloc-test with the plateau and
  purge numbers of each column) and `residual` (one row per residual evidence row:
  `total`, `attributed`, `residual`, `threshold`, `share`, `over`, `open_item`, and
  the `column`, `span` and `noise` recovered from the series and trend of that
  series, null when the series has no primary column for the region).
  `tied` compares exact finding IDs, worker prefix included (`jvm/F1` is not
  `go/F1`), for the IDs in `candidates` and for the column's own `finding`.
  For an off-heap residual row the `column`, `span` and `noise` come from the
  `gap.unexplained` column of the newest gap series derived from the row's
  series (the series `dla gap --residual` fitted), with the noise computed as
  `dla gap --residual` does (1.4826 x the median absolute deviation around the
  Theil-Sen line); without such a series, from `node.external` of the source series.
  `expected_gate` is `"fail"` for every column of a `decoy-control`, `non-leak` or
  `impossible` fixture, `"pass"` for a column tied to a finding at the site of an
  expected leak on a fixture with a leak set, and null otherwise.
  `dla-bench dynamic` takes `--iterations`, `--step` and `--warmup` and passes
  them to `dla harness gen`, so a grid of settings can be run and extracted.

## `metrics.json`

```json
{
  "schema_version": 1,
  "groups": [
    {
      "ablation": "no-skill",
      "archetypes": "on",
      "metrics": {"leak_recall": {...}, "...": {...}},
      "unexplained_growth_share": {"value": null, "runs": 0},
      "usage": {...},
      "verification": {...},
      "decoys": {"L-COLLECT": {...}},
      "fixtures": {"jvm-order-cache": {...}}
    }
  ]
}
```

Groups are sorted by `ablation`, then `archetypes`. All seventeen ratio metrics
are always present; `verification` only when the group has a counting
verification record, and `decoys` only when a pair was graded. Object key order does not matter. The file is UTF-8 JSON
with a trailing newline.

## `findings.json` (runs without dla)

The runner asks runs that have no `dla` ledger (`no-skill`, `prose`) to write
this file in the project root:

```json
{
  "findings": [
    {
      "site": "src/orders/cache.py:42",
      "holder": "container-field",
      "region": "heap",
      "verdict": "leak-confirmed",
      "mechanism": "what keeps the memory alive and why it is never released",
      "cause": "optional label",
      "evidence": "what was run or measured",
      "discriminator": "for not-a-leak: the observation that rules the leak out"
    }
  ]
}
```

`verdict` is `leak-confirmed` (shown by running or measuring), `leak-suspected`
(seen in the code but not demonstrated) or `not-a-leak` (growth that was
investigated and ruled out). Holder and region use the vocabularies in
`FIXTURES.md`.
