# Stages

The detail behind the nine stages in `SKILL.md`. Read the stage you are on. After each stage run
`dla run status` and work through its `blocking` list before moving on. In a worker, every command
is `dla --run <id> --worker <name> ...` (see `agents/worker.md`).

## Entry paths

### Repository
Stages 1 to 9. `dla run init --entry repository` with optional `--scope module|service
--scope-path <dir> --exclude <path>` (repeatable). Scope limits the census, the workload scan, the
sweep and the harness copy; skipped files are counted per runtime and the report prints a Scope
line. Ask for the scope once (`intake.scope`); the default is the whole repository.

### Local process
`dla run init --entry process`. The consent question must include tier 2 (and 3 or 4 when you
intend to use those operations). Stages 1, 2, then:

- `dla sample --pid <P> [--duration 60s] [--counters auto|none|<list>] [--tree]`: tier 1 for a
  process DLA started, tier 2 for any other. Sampled runtime heap columns are not read after a
  collection: a lead, not proof.
- `dla attach <backend> --pid <P> --op <op> [--duration 30s]`: preflight first (target, expected
  signal, stopping condition, effects, cleanup, free disk, available memory), then the capture.
  Tier per operation:

  | Tier | Operations |
  |---|---|
  | 2 | `jcmd` histogram, thread-dump, nmt, jfr; `node-inspector` memory |
  | 3 | `memray` record, `heaptrack` record, `goref` heap-profile (unverified here) |
  | 4 | `jcmd` heap-dump, `node-inspector` heap-snapshot (memory contents; may hold secrets) |

  A process DLA started is tier 1. Read the preflight before relying on the result. If it refuses
  with `insufficient-resources` (dump size or peak extra memory does not fit), do not retry the
  same operation: see `references/recovery.md`.
- Captured artifacts land under `attach/P<n>/`. Read each with the `dla summarize` operation for
  its format (stage 6, `references/backends.md`); a format no adapter reads is refused with the
  supported list.
- With the source tree available (ask once), continue as the repository path with the sample as
  live evidence. Without source no finding can carry a `file:line`, so a growing process ends with
  growth evidence and an `unresolved` item whose remedy is the source or a dump series run through
  the artifact path.
- Interrupted or finished early: `dla run abort --reason "<why>"` stops processes DLA started and
  undoes attaches (it never kills the target).

### Artifact
`dla run init --entry artifact --artifact <path>...` (paths relative to the project root, one flag
per file or directory). No source, so no site is ever added or annotated on an artifact run.

1. Intake (target = the dump files, symptom, consent: summarizing reads files and needs no tier;
   the dumps may hold secrets, keep them in place).
2. `dla summarize heap-graph <dump1> <dump2> <dump3> ...` once, with ALL dumps of the one process:
   the series is built from the files named in one call, ordered by header timestamp. It writes a
   retention-path evidence per dump, a dump series, a growth evidence and a `dump-plateau`
   discriminator per holder of interest. Add `--finding F` to bind the plateau test to an existing
   finding.
3. At least three dumps are needed. With three or more, a holder whose retained size and count
   grow strictly across every interval (the last interval at least half the first) passes the gate
   and is added as a candidate with an owner (`class.field`) and no site.
4. For that candidate `F`: add an `N-WARM` (or `N-HWM`) sibling whose hypothesis starts with the
   `N-*` id and states what would refute it (`dla ledger add --sibling-of F --origin hypothesis
   --hypothesis "N-WARM: ..." --refute-if "..." --evidence <growth>`); refute it with the
   dump-plateau discriminator that favors leak (`dla ledger refute F.n1 --evidence <E>`);
   `dla ledger test F --evidence <growth>`; `dla ledger confirm F --evidence <growth>`. The holder
   is required; the owner replaces the site.
5. One or two dumps: the gate fails with "dumps < 3" and says how many more are needed. Resolve the
   candidate `suspected_static_only` with reason `not_exercised` and the remedy "provide N more
   heap dumps of the same process taken after more traffic" (a warm-up fill and a leak look alike
   over one interval).
6. Dumps from different processes, different HPROF versions or id sizes are refused; two runs of
   the same program are not told apart, so ask the user which process produced them.
7. Stages 8 and 9. The report must make no source-mapping claim.

### Symptom only
Only when the request names no target and the working directory holds no source code (a request
that describes a service while you sit in its repository is the repository path). Ask once for
the target. No answer or an unattended run: `intake.target` unresolved as in
`SKILL.md`, `dla report`, stop. With an answer, re-route to the path it picks (the run is already
`--entry symptom`; start a new run with the right `--entry` if the target is a process or dumps).

## 1. Intake checklist

Ask the user only for what the repository cannot tell you. Create the run (flags per path above,
plus `--budget ${user_config.time_budget} --session-id ${CLAUDE_SESSION_ID}`). Ask the consent
question once, as a single question covering tiers 1 to 4 up to the privilege ceiling. If `dla run
init` reports a `consent` object, an unattended runner already answered: ask nothing, cite its
evidence for `intake.consent`. Otherwise `dla consent --tiers <list> --ceiling
${user_config.privilege_ceiling}`, adding `--raw-values` only if the user explicitly agrees to see
raw values. A second consent is refused; never ask twice.

## 2. Probe

`dla probe` records runtimes, versions, tools, privileges, test runners and library flags. A
backend the runtimes could use but the machine lacks is recorded in coverage as `missing`, with an
install hint. After tier-1 consent run `dla probe --build` (`--build-timeout <secs>`, default 300);
a failed build is a coverage gap (`build_fails`), not a reason to stop. For each missing backend,
`dla install <backend>` once, in the core and before `dla run dispatch` (installs write to a shared
directory without a lock). A failed install stays a recorded gap; never retry in a loop and never
install into project source.

## 3. Static sweep and holder census

For every runtime probe found (`jvm`, `go`, `rust`, `node`, `python`, `native`), in this order:
`dla sweep --lang <l>` runs the available static backends and adds one candidate per rule hit
(origin `detector:<backend>/<rule>`); an absent or failing backend shows as `missing` or `blocked`,
never hand-run it. `dla census --lang <l>` adds one candidate per process-lifetime holder (origin
`census`). `dla workload discover` lists tests, entry points and dev servers a loop could run.
Census writers and removers are name-matched leads: for each holder open the source, find the code
that adds to it and the code that removes from it, and record `dla ledger annotate F<n> --writer
file:line --remover file:line --note "<text>"` (both flags repeat). Annotating changes no state. A
holder with writers and no remover is the lead for stage 4.

## 4. Classify growth

Measure first and explain afterwards. These commands build and run project code in a scratch copy
under the run directory and need tier 1.

- `dla harness gen --lang <l> [--workload <id>]` copies the project into
  `.dla/<run>/harness/H1/work/`, finds the iteration-count argument, derives probes from census
  candidates and writes the manifest. It runs nothing. On `iteration-arg-unknown` pass `--iter-arg
  "--iterations {n}"`. For a library with no main, or to compare fresh with repeated keys, add
  `--driver [--package <dir>]`: it prints the file to edit; replace the body of each `DLA-HOOK`
  and derive ids from `key` so a repeated key reproduces an earlier request. Go probes of package
  variables work only in driver mode.
- `dla harness run H1` builds once and measures every point (`--mode both` in driver mode). Give a
  harness run or alloc-test Bash call a timeout of 600000 ms.
- Raw captures for stage 6 come from flags of the same command; each is a separate, costlier run, so
  add one only for a harness that grew (a passing column), not for every harness:
  - JVM: `--capture heap,histogram,jfr,nonheap` or `--capture all` (`--dumps <N>` heap dumps,
    default 4). Command-mode heap dumps are separate processes, so `heap-graph` cannot compare them;
    a driver (`--driver`) takes all dumps in one process.
  - Rust and native, program started directly: `--tool valgrind` (memcheck XML plus the vgdb chain
    files) or `--tool heaptrack` (`heaptrack_print` report and a first-to-last diff) instead of a
    measured series. A wrapper (make, sh) gets no vgdb chain. A missing tool refuses with
    `tool-missing` and leaves a `missing` coverage row.
  - The output lists the files under `captures` and prints the `dla summarize` commands to run
    next (the full list is in `harness/H<n>/captures.json` when the output was cut).
- `dla trend S1` gives the shape and slope gate per column; read the `passed` columns and their
  `candidates`. A passed column with no candidate is growth nothing explains yet.
  `dla trend S1 --against S2 --finding F3` compares fresh with repeated keys.
- `dla sample --pid <P>` samples a running process (see Local process). `dla gap S1` splits
  process memory into accounted and unexplained; `dla gap S1 --residual` totals growth, subtracts
  what confirmed findings explain and opens an open item for the rest.
- Column names, the gate and the residual rule: `references/evidence-and-report.md`.

## 5. Confirm

For each passed column tied to a candidate `F3`:

- Sibling first: `dla ledger add --sibling-of F3 --origin hypothesis --hypothesis N-WARM --refute-if
  "<result that would refute it>" --evidence E7`. The hypothesis starts with one of `N-ALLOC`,
  `N-HEADROOM`, `N-WARM`, `N-HWM`, `N-SCALE`, `N-INTENT`, `N-OBSERVER`.
- `dla ledger test F3 --evidence E9`, citing the harness `workload` evidence.
- `dla alloc-test H1 --finding F3` re-runs the harness with extra points and a purge and records a
  discriminator whose `favors` is `leak`, `non-leak` or `inconclusive`.
- Refute the loser: `dla ledger refute F3.n1 --evidence E13` when it favors leak,
  `dla ledger refute F3 --evidence E13` when it favors non-leak.
- `dla ledger annotate F3 --holder <holder>` when the finding has no holder (`--owner`, `--region`,
  `--site` fill other gaps), then `dla ledger confirm F3 --evidence E11`, citing the passing
  `dla trend` evidence. A refusal names what is missing.
- The passing column must be tied to `F3`: its probe (`probe:<owner>`), a group column at its site,
  or a column that lists it under `candidates`. A refusal `growth-not-tied` means the growth you cite
  belongs to another holder or to the whole process (`os.rss`, heap totals). Then add a probe for
  `F3`'s holder and rerun the harness, or resolve `F3` `suspected_static_only` with a remedy; never
  confirm one holder from another holder's growth.

## 6. Attribute and explain

Turn the captures into bounded evidence with `dla summarize <op> <files...>`. Run the commands
`dla harness run` printed, one per op and file format (the op x runtime x backend table, tools and
coverage rows are in `references/backends.md`; `dla summarize --list` shows what is installed).

| Op | Use it for | Gives |
|---|---|---|
| `heap-graph` | HPROF dumps of one process (3 or more), or the native vgdb chain | retention path per dump, a `holder:` growth column, `via_holders` |
| `type-histogram-diff` | class histograms, shim stack groups | growth by type or by allocation stack |
| `alloc-stack-diff` | JFR old objects, valgrind, heaptrack, dhat, LeakSanitizer | growth by allocation stack, site is the first project frame |
| `non-heap` | thread dumps, class loaders, NMT, descriptors | counts by group; never ties to a finding |

Reading the result:

- `gate` pass or fail says whether a column grows strictly enough. `passed` lists the columns that
  did, each with a `tie` (rule `holder`, `site` or `type`, and the key). `needed` says what is
  missing; `candidates` lists every finding tied by exact identity (holder, site or recorded writer) at one place; `ambiguous` lists only type matches that hit several findings (they tie none).
- Growth confirms only the holder it measures. A tie exists only when the column measures the same
  holder as the finding, carries the finding's site (or a writer the census or you recorded), or the
  growing type is the finding's recorded accumulation class, its declared element type, or its own plain declared type when that is a project class (not a generic, JDK or Kotlin type; an instance field of its own class type is a chain link, not a holder). `--finding F`
  narrows to such columns; it never forces a tie. Whole-process and total columns (`os.*`,
  `jvm.*`, thread totals) never tie.
- An empty, flat or failed-gate result never refutes a leak: the capture may have missed it (a
  sample, a short run, a retention the tool cannot see). Only `dla alloc-test` or the
  `dump-plateau` discriminator can favor non-leak.
- A missing tool is a coverage row (auto) or a `tool-missing` refusal (explicit `--backend`): record
  the candidate `unresolved` with reason `tool_missing` and the install remedy; do not hand-run the tool.
- Data values (thread names, labels, paths outside the project) are shaped as `<len:pattern:hash>`
  unless the run has the raw-values consent. Read `raw` (path and sha256) only when the bounded
  result is not enough.

Cite it. The `growth-series` evidence id of a tied, passing column goes into `dla ledger confirm F3
--evidence E11,E14` together with the passing `dla trend` evidence; the retention-path or
allocation-stack evidence ids of the same call are the attribution. If the ledger answers
`growth-not-tied`, the cited column does not measure F3's holder: pick another capture or resolve
`suspected_static_only`, never loosen the tie.

Name the missing pairing: what should have released, removed or stopped, not a weak-reference
workaround. Record holder and mechanism in your own words with `dla ledger annotate F3 --note
"<text>"`. The label, `strong` and explain commands arrive with the taxonomy; when `dla ledger
--help` lists them use them, otherwise leave the finding unlabelled (UNCLASSIFIED) and say so.
Verification of each confirmed finding is done by the core: `dla bundle F3`, one `dla:verifier`
subagent given only the bundle directory, then `dla ledger verify` (see `SKILL.md`). A disagreement
opens an open item to resolve.

## 7. Rule out non-leaks

Test the sibling explanations: allocator retention (`N-ALLOC`, `dla alloc-test` trims or purges and
looks for a plateau), deferred collection (`N-HEADROOM`), warm-up caches (`N-WARM`), pool
high-water marks (`N-HWM`), workload scaling (`N-SCALE`), intended retention (`N-INTENT`), the
instrument itself (`N-OBSERVER`). Growth a discriminator shows to be non-leak ends refuted:
`dla alloc-test H1 --finding F3`, then `dla ledger refute F3 --evidence E12`. A candidate the
harness exercised whose column did not grow gets the same test and is refuted when the plateau
verdict favors non-leak. When no column tied to any candidate grew, one unbound `dla alloc-test
H1` judges the whole workload; if it favors non-leak, refute each exercised candidate with that
evidence. A candidate the harness could not exercise is neither confirmed nor refuted:
`dla ledger resolve F6 --as suspected_static_only --reason not_exercised --remedy "<text>"
--evidence E4` (reasons `not_exercised`, `no_harness`, `build_fails`), or `--as unresolved` when
no static evidence exists. A failed build or a run without usable points (`harness-failed`)
confirms nothing.

## 8. Final gate

`dla run status`: work through every item in `blocking`. What cannot be done is recorded:
`dla ledger resolve F3 --as unresolved --reason time_budget --remedy "<what to do next>"`. Reasons:
`not_exercised`, `no_harness`, `env_unavailable`, `build_fails`, `privilege_declined`,
`tool_missing`, `time_budget`. With workers: `dla ledger merge` first, then this stage.

## 9. Report and revisit

`dla report` writes `report.md` from the ledger and prints it; show it exactly as printed. While
budget remains, return to findings that are `unresolved` or `suspected_static_only`:
`dla run status` lists up to 10 under `revisit` (`revisit_total` gives the count), each with `id`,
`state`, `reason` and `remedy`. To stop early, `dla run abort --reason "<why>"` closes the run.
