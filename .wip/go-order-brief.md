# Brief G2: Go candidates in the order that finds leaks, and drivers for packages of nested modules

Repository: /home/user/dla (Rust workspace, branch `main`). The working tree holds uncommitted
changes of earlier builders. Leave them as they are; where you must edit one of those files, add
your own lines only. You are the only builder in the tree.
Scratch directory: /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u124
(create it; write nothing else outside the repository).

The machine has 4 cores. Use `cargo ... -j 3`, one cargo command at a time. Never touch
/root/dla-real.

## What happened (a real unattended run on a Go library the tool had never seen: one
## repository, 32 Go modules)

1. The model gave up after 7 minutes of a 3 hour budget and said why: most packages are in
   nested modules (`sdk/go.mod`, `exporters/.../go.mod`), and the generated driver could not
   be built for them; it wrote `GOWORK` files by hand in a temporary directory; coverage
   collection failed in a cross-module package; and its first "leaks" were test helpers of
   the project (`.../tracetest/recorder.go`).
   Reproduced now: `dla harness gen --lang go --driver --package sdk/trace/tracetest` answers
   `"sdk/trace/tracetest" holds no Go package to drive`, although the directory has Go files
   (`crates/dla-workload/src/harness/gen.rs` near line 1575: the package directory is joined
   to the working directory of the default workload, which lies in another module, and the
   copy of the project is cut to that module).
2. The documented leak is a slice field of an unexported type, appended in fourteen exported
   methods and reset in one method (`m.instruments = nil` when a delegate is set). After the
   census fix it is listed, at rank 118 of 149: its score is 61 (`w_grow` +25, `container`
   +20, `w_runtime` +20 and small ones). Fifteen registration rows with 0 writers have 89.
   Twelve of the first forty ranks are test-support packages (`tracetest`, `otlptracetest`,
   `internaltest`, `otest`): `is_non_product_site` (`crates/dla-core/src/rank.rs`) splits path
   components into words and `tracetest` is one word.
   The ranked list of that project is in
   /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u119/otel-go-list.jsonl
   (one JSON object per line: id, rank, tier, score, why, site, holder, writers, removers).
   A copy of the project is at
   /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u112/otel-go
   (copy it to your scratch directory and work on the copy; environment
   `DLA_CONSENT_TIERS=1-4 DLA_CONSENT_RAW_VALUES=1 DLA_PLUGIN_ROOT=/home/user/dla`; `dla run
   init`, `dla probe`, `dla census --lang go`, `dla ledger list --all`).

## The change

### 1. The score (`crates/dla-core/src/rank.rs`, every runtime unless said otherwise)

Read `contributions()` first. Three additions, each a named weight with its own unit tests:

- `w_many`: a holder that is added to from many places is more likely to be fed at run time
  than one with a single writer. Count the distinct functions among the adding writer sites
  that are neither initialisers nor start-up sites: 3 or 4 functions give +8, 5 or more +16.
- `r_reset_only`: when every removing site is a whole-container reset (`SiteKind::Reset`, not
  a removal of one entry) and at least one adding site runs at run time, nothing bounds the
  holder between two resets: +15. Today such a holder loses `growth_unmatched` and `r_none`
  exactly as if entries were removed one by one.
- Go only, `go_support_dir`: a directory component of the site whose name ends in `test` and
  is longer than `test` (`tracetest`, `httptest`, `internaltest`, `otest`) is, by the
  language's convention, a package of test helpers: -25. A penalty, not a demotion: a project
  whose own product lives in such a directory keeps its order, since every candidate there
  gets the same penalty. Do not touch `is_non_product_site` or the tiers.

Also check in the Go census (`crates/dla-census/src/lang/go.rs`) how `exported` is set for a
writer site: an exported method of an unexported type is reachable through the interface the
type implements, so it counts as exported (`w_exported`). If it does not today, change that
and say so.

Before and after, on the copy of the real project: the rank, score and `why` of the slice at
`internal/global/meter.go:95`, of the first forty ranks (how many are in directories ending
in `test`), and the members of the first batch (`dla run status`, `queue.go.next`). The aim:
the slice is in the first batch of its package and the helper packages are below the product
packages. Do not tune weights to this one project beyond the three rules above; if the aim is
not met with them, report the numbers and stop.

### 2. A driver for a package of a nested module (`crates/dla-workload/src/harness/gen.rs`,
### `scratch.rs`, `commands.rs`, `run.rs`: find the Go paths)

- The module of a Go driver is the module of its package: the nearest `go.mod` at or above
  the package directory inside the project. `dla harness gen --lang go --driver` with
  `--package <dir>`, `--next`, `--probe <F>` or `--entry <F>` builds and runs there, whatever
  module the default workload is in. The copy of the project must hold that module and what
  it needs from the repository.
- When the project has more than one `go.mod`, the work copy gets a `go.work` at its root
  that lists every module directory of the copy (unless the project has its own `go.work`),
  so that the modules of the repository are used from the copy, not fetched as released
  versions; the build and run commands of the harness use it. Check what happens when two
  modules name different Go versions and say so.
- Coverage (`-coverprofile`, `-coverpkg`) covers the package's own module at least; say what
  it covers when the target's writers are in another module of the repository.
- A package directory whose only non-test Go files were left out as generated: the refusal
  says that (not "holds no Go package"), and `--next` goes on to the next package of the
  queue and says in `_attention` which package it skipped and why.

On the copy of the real project: `dla harness gen --lang go --driver --package
sdk/trace/tracetest` (and one more package of another nested module of your choice) builds;
with hooks that do nothing but compile (`setup` returns, `unit` returns) `dla harness run <H>
--mode fresh` writes a series and `dla trend` reads coverage for the targets. Report the
commands and the first lines of each output. Delete the run directory of your copy afterwards.

### 3. Texts

`skills/detect-leak-anywhere/references/go.md` and `harness/go/README.md`: the changes in a few
sentences, pinned in `crates/dla-cli/tests/skill_text.rs`. The text that explains the ranking
(`TIER_NOTE` in rank.rs and where the references describe the score: find it) names the three
new weights. `docs/VERIFY.md`: rows at the end, state `corrected`, "a Go library the tool had
not been run on". Do NOT touch `skills/detect-leak-anywhere/SKILL.md`.

## Tests (each seen failing under a mutation you apply and revert)

- rank.rs unit tests for the three weights: present when the rule holds, absent when it does
  not (one writer; a per-entry remover beside a reset; a directory named `test` or `contest`
  with a non-Go file; a Go file in `latest/`: say what you decided for English words that end
  in `test` and why).
- `crates/dla-cli/tests/rank_score.rs` and the tests that pin the order of documented leaks
  (find them: they came with the change "order candidates so documented leaks land in the
  first batches"): all pass; list any whose expected order changes and why.
- A CLI test with real Go builds, guarded like the existing Go driver tests: a repository
  with two modules (`go.mod` at the root and `svc/go.mod` that requires the root module with
  a `replace`), a holder in `svc/store`: `dla harness gen --lang go --driver --package
  svc/store` builds, the run measures the holder growing, coverage reads the target
  `executed`. The same repository without the `replace` line builds through the `go.work`.
- Existing tests that change: say which and why. Do not weaken a test. Nothing under
  `bench/fixtures` is edited. Run `dla-bench static` (all fixtures, no model) and `dla-bench
  dynamic --only <name>` for three Go fixtures (two leaks, one non-leak) and report the
  results; if one changes, stop and report it.

## Rules

- No `git commit`, `git stash`, `git checkout` (in any form), `git reset`, `git restore` or
  branch change. Leave the tree dirty.
- Shared `target/` only (`cargo` from /home/user/dla). No `rm -rf` on variable paths. Do not
  kill processes by name. Do not run `dla consent`. Do not start any model, agent or verifier.
  No package installs.
- Run each cargo command on its own, in the foreground, piped through `tail` or `grep`. If an
  action is denied, stop that action and report it; do not ask anyone to do it for you and do
  not try another way to the same end.
- Keep to this brief. If you find another defect, report it; do not fix it.
- Plain technical English, full sentences, no em dashes. Never name a real third-party project,
  library or its APIs in the repository (the names above are for you; in tests and texts use
  neutral names). No model names, no attribution lines.
- Mutations by a small script that asserts the old text occurs exactly once. Before you finish,
  `grep -rIl "MUT_" crates harness skills docs bench agents hooks data` must print nothing.
- If the schema of a record changes, update `schemas/` and run `./target/debug/dla schema
  --check`.

## Checks before you report

`cargo fmt --all --check`; `cargo clippy -j 3 --workspace --all-targets -- -D warnings`;
`cargo build -j 3 -p dla-cli -p dla-bench` and `./target/debug/dla schema --check`;
`cargo test -j 3 -p dla-core --lib`, `cargo test -j 3 -p dla-census`,
`cargo test -j 3 -p dla-workload --lib`, `cargo test -j 3 -p dla-cli --bin dla`, the Go driver
test files of dla-cli (list them), `--test queue --test gate_flow --test skill_text
--test rank_score`, and the tests you added.

## Report (final message; read by the orchestrator)

Two sentences of result first. Then per part: files changed, what works, one real output
(first lines); the before and after numbers of the real project; each test with its mutation;
the fixture results; existing tests changed and why; anything in this brief that was wrong;
what you did not verify. Counts only.
