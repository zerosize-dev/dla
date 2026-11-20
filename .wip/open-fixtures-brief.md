# The eight open fixtures: diagnosis (orchestrator brief)

Repo /home/user/dla, branch main at 4aae86f, clean tree. Do NOT commit or push. In this step
nobody edits repository files: you diagnose and propose.
sp = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad

## Background
DLA (the `dla` CLI in crates/, the skill in skills/detect-leak-anywhere, data in data/) finds
memory leaks in a project: a static stage (`dla probe`, `dla sweep`, `dla census`, DLA's own
rules) lists candidates, and a dynamic stage (`dla harness`, `dla trend`, `dla alloc-test`,
`dla summarize`, the ledger) confirms or refutes them with evidence. `dla-bench` checks DLA
mechanically against fixtures: small projects with a known leak (or none) and an answer key
`expected.toml`. `dla-bench static` requires that the census and DLA's own rules alone (no
external analyzer: the CI runner has none) flag every confirmable leak at one of its sites.
`dla-bench dynamic` drives the dynamic stage the way the skill's policy does
(bench/runner/src/dynamic_cmd.rs, bench/runner/src/dynamic/) and requires each confirmable leak
to be confirmed with the evidence its label needs, and nothing false to be claimed.

Eight fixtures are written and reviewed but do not pass; they are parked in
bench/fixtures-open/ (read its README.md first: it says for each one what fails, and how pairs
work). The goal of the whole job is to make DLA pass them by improving the product in a way that
is right for real projects, and then move them back. A fixture's answer key, README and project
are never changed to make it pass.

## Your fixture
You get one fixture. Work in your own scratch dir `<sp>/u13/<fixture>/` (create it; delete only
it, by literal path; never `rm -rf "$VAR/..."`).
1. Read the fixture's README.md, expected.toml and project code until you can say in two
   sentences what leaks, why, and what the fixed code would look like.
2. Make a scratch fixtures directory with a copy of the fixture (`<sp>/u13/<fixture>/fx/<fixture>`).
   If its expected.toml has a `pair` line, copy the partner from bench/fixtures/ too and add the
   line `pair = "<fixture>"` right after `category` in the copy of the partner's expected.toml
   (the validator refuses half a pair).
3. Run the acceptance on that directory with the built binaries (they exist; do not rebuild
   unless a command says the binary is missing; then `cargo build -j 2 -p dla-cli -p dla-bench`):
   `./target/debug/dla-bench --fixtures <dir> validate`,
   `./target/debug/dla-bench --fixtures <dir> static --dla-bin ./target/debug/dla --json <scratch>/static.json`,
   `./target/debug/dla-bench --fixtures <dir> dynamic --dla-bin ./target/debug/dla --parallel 1 --only <fixture> --keep-work --json <scratch>/dynamic.json --log-dir <scratch>/logs`.
   Check `dla-bench <cmd> --help` for the exact option names. Keep the logs.
4. Static, if it fails: list the candidates the census and DLA's rules produced (file:line,
   rule or resource kind), say why none is at a site of the leak, and name the general signal
   that should flag it: which construct, in which language, held by what, released by what. Read
   the code that would carry it (crates/dla-census, the rules under data/ and
   crates/dla-backends, crates/dla-core archetypes) and say where the change belongs.
5. Dynamic, if it fails: follow the per-step logs and find the first step where the chain to a
   confirmed leak breaks (workload discovery, harness generation or build, the series does not
   grow, no column passes the trend, the finding is not tied to the growth, alloc-test reads
   non-leak or inconclusive, summarize has no evidence for the label, the ledger refuses the
   confirmation or label and why). Read the code of that step. Say what a correct run needs
   there and which product change provides it. If the cause is in dla-bench's policy and not in
   the product, say so.
6. Judge the proposal: would the same change be right on a real project of that kind, or does
   it only fit this fixture (a rule keyed to a name in the fixture is not acceptable)? Which of
   the 60 fixtures in bench/fixtures could it change (false candidates on the non-leak and
   decoy fixtures of that language, a changed verdict)? What is the smallest test that would
   fail today?
7. If you think the answer key or the fixture itself is wrong (the leak is not real, cannot be
   observed by any tool, the site is wrong), say so with evidence. Do not change it.

## Rules
- No repository edits. Cargo with `-j 2` and the shared `target/` only. Never kill processes by
  name. Another agent works on the same 4 CPUs.
- Disk: if `df --output=avail -B1M /` is below 3000, run `bash <sp>/prune-superseded.sh`.
- Keep machine time to about 30 minutes; reading the code is the larger part of this job.
- Be exact: quote the log lines and the code (file:line) that carry each claim, and mark what
  you only suspect.

## What to return
`fixture`; `leak` (two sentences); `static` {passes, candidates seen, cause, proposal, where in
the code}; `dynamic` {passes, first broken step, cause with the log lines, proposal, where in
the code}; `generality` (why the change is right beyond this fixture, or why not);
`risk` (which fixtures or behaviours it could change); `tests` (the smallest failing tests);
`effort` (small: under a day of one engineer; medium; large) for static and dynamic each;
`key_doubts` (doubts about the answer key or fixture, or "none"); `unverified`.
