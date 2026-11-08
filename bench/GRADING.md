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

A run with several workers also holds `ledger.<worker>.jsonl` (the slices) and
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
  was added without; a value that is already set never changes).
- `finding_merged`: if the records' `canonical` and `duplicate` are strings
  that give two different keys, and a finding exists under both, the finding
  under the `duplicate` key is removed and the one under the `canonical` key
  keeps its own state and fields unchanged, so the claim is counted once.
  Otherwise the record has no effect. A later `finding_added` with the
  duplicate's key creates a finding again.
- Every other record type is ignored, in particular `verification`,
  `slice_started`, `slice_done` and `slice_merged`: they change no finding and
  no metric.

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
array, every row's `residual` and `total_slope` (a missing or non-numeric
value counts as 0; a row that is not an object is skipped), and `k` counts
those runs. A `coverage.json` that is not valid JSON is treated as absent.
`value` is null when `k` is 0 or `T` is 0.

## Usage

Per group: `runs` (count of graded runs), `incomplete` (run directories
skipped for lack of `run.json`), `errored` (runs whose `exit_code` is missing,
not an integer or not 0, or whose `timed_out` is true), `sources` (`{"ledger": n, "findings": n, "none": n}`),
for each of the four token fields `{"sum": n, "mean": round6(sum / runs)}`,
`wall_seconds` as `{"sum": round6, "mean": round6, "max": round6}` (every
`mean` and `max` is null when `runs` is 0) and
`cost_usd` as `{"sum": round6, "mean": round6}` over the runs with a numeric
`cost_usd` (`{"sum": 0.0, "mean": null}` when there are none).

## Per fixture

Per group and fixture: `category`, `runs`, `claim_runs` (runs with at least
one claim), `rule_out_runs` (runs with at least one rule-out), and `leaks`
mapping every expected leak `id` (confirmable or not) to
`{"found": n, "flagged": n, "grounded": n}` counted over runs (`grounded`:
found and the chosen finding's holder matches).

When at least one found leak's chosen finding has a worker, the fixture also
has `worker_provenance`: an object mapping the leak `id` to an object that maps
each worker name to the number of runs in which that leak was found with a
chosen finding of that worker. Leaks with no worker are left out, and the
member is absent when empty. It is informational and part of no metric.

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
      "fixtures": {"jvm-order-cache": {...}}
    }
  ]
}
```

Groups are sorted by `ablation`, then `archetypes`. All thirteen ratio metrics
are always present. Object key order does not matter. The file is UTF-8 JSON
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
