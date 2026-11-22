# Brief W1: a workload blocker closes the queue of one runtime only

Repository: /home/user/dla (Rust workspace: crates dla-core, dla-workload, dla-cli, dla-report and
others; plugin files under agents/, skills/, harness/, data/). You are a builder. The working tree
is clean and equals `main`; work in it.

Scratchpad (sp): /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad
Your scratch directory: <sp>/u75 (create it). You cannot write a report file with the Write tool:
give the report as your final message (you may also write it with a shell redirect to
<sp>/u75/report.md).

## Hard rules

- No `git commit`, `git stash`, `git checkout`, `git reset`, `git restore` in /home/user/dla.
- Never kill a process by name. No `rm -rf "$VAR/..."`: only literal paths, only under <sp>/u75.
- Use the shared `target/` directory only (`cargo ... -j 2`). Disk is tight (about 5 GB free):
  run `bash <sp>/wip-notes/prune-superseded.sh` before a whole-suite run.
- Nothing in the repository may name a model, an AI assistant, or any third-party skill collection.
  No attribution lines. Plain technical English, short sentences. Do not name the project of the
  evidence run in the repository (say "a Python library with C extension modules").
- A marker mutation is an exact text replacement that carries a unique marker such as
  `/*MUT_B1*/`; apply it with a script that asserts the old text occurs once (see
  <sp>/u66/mut.py for the pattern), see the test fail, reverse it with the exact reverse
  replacement. At the end `grep -rIl "MUT_" crates harness skills docs bench agents hooks data`
  must print nothing.
- Do not run `dla consent`, and do not start any model, verifier or agent. A scratch run is
  initialised with `DLA_CONSENT_TIERS=1-4 DLA_CONSENT_RAW_VALUES=1` in the environment of
  `dla run init`.
- If something you are told here is wrong, say so in the report; do not widen the task.
- If a command of yours is denied by a permission check, do not work around it and do not ask the
  orchestrator to run it for you: report it and go on with the rest.

## What happened (the reason for this brief)

A real run of the tool on a Python library with C extension modules ended after 4 minutes with a
COMPLETE report and 329 of 346 ranked candidates untested. The run had two runtime queues: python
(186 candidates; two Python harnesses had run and written series S1..S4) and native (160
candidates in the C sources). The agent generated a native harness (a C loop driver), saw that the
C sources are extension modules that only Python can drive, and recorded what the tool's own text
offers:

    dla ledger resolve intake.workload --status unresolved --reason build_fails \
        --remedy "native candidates in src/*.c need a CPython-embedding C driver ..."

After that `dla run status` showed BOTH queues `closed: "blocked"`, and `dla ledger close-rest`
closed 567 findings (563 as suspected_static_only / not_exercised). The text the tool prints says
"(that closes the queue of the runtime)"; the code closes every queue.

Evidence: <sp>/u36/pillow-1 (transcript.jsonl; dla/ is the run directory: ledger.jsonl seq 1137 is
the blocker). A read-only survey of the code paths, with file:line for every reader of the blocker,
every message that offers it and every test that pins it: <sp>/u74/0.md. Read it first; check its
line numbers against the tree before you rely on them.

The cause in the code, as the survey found it:
- `gate::workload_unresolved(view, blocker_only)` has no runtime; `queue::queues` computes
  `blocked` once (queue.rs, near `let blocked = gate::workload_unresolved(view, true)`) and applies
  `Closed::Blocked` to every runtime queue. The same flag turns off the second look and the test
  loop.
- The refusal `workload-ran` (`gate::workload_ran`, `refuse_contradicted_workload_blocker` in
  dla-cli ledger_cmd.rs) fires only when EVERY runtime with ranked candidates has a harness series.
  Here python had series and native had none, so the blocker was accepted.

## The change

A workload blocker names one runtime and acts on that runtime only.

1. `dla ledger resolve intake.workload --status unresolved --reason <blocker reason> --remedy <text>
   --runtime <rt>`: a new optional flag, valid only for the checklist item `intake.workload` with
   status unresolved (refuse it elsewhere with a plain error). The runtime is stored in the
   checklist answer record (an optional field, absent when not given; follow how earlier optional
   fields were added to ledger records, keep old ledgers readable, and run
   `./target/debug/dla schema --check`; regenerate the schema files the way the repository does it
   if the check asks for it).
   - `<rt>` must be a runtime of this run (one that has ranked candidates, or one the probe
     detected). An unknown name is refused, and the refusal lists the valid names.
   - With a blocker reason (`env_unavailable`, `tool_missing`, `build_fails`,
     `privilege_declined`): when more than one runtime has ranked candidates, `--runtime` is
     required; the refusal says so and lists the runtimes with their pending counts. When exactly
     one runtime has ranked candidates, the flag may be left out and that runtime is recorded.
     When no runtime has ranked candidates the answer is stored without a runtime, as today.
   - The other reasons the item accepts today (for example `not_exercised`, `no_harness`,
     `time_budget`) keep their present behaviour; `--runtime` is allowed with them and then limits
     the answer to that runtime wherever the answer is read per runtime (see 3).
2. The refusal `workload-ran` is per runtime: a blocker for runtime R is refused when a harness of
   R wrote a series in this run. The message names the runtime, the harness and the series (keep
   the code `workload-ran` and the present wording, add the runtime). A blocker for a runtime with
   no series is accepted whatever the other runtimes have.
3. Every reader of the blocker asks about one runtime. An answer with a runtime applies to that
   runtime only; an answer without one (old ledgers, or the no-candidates case) applies to every
   runtime, as today. Readers (from the survey, check each): the queue (`Closed::Blocked`, the
   second look, the test-loop plan), `trigger::unchecked_because` and `trigger::items` (a finding of
   an open runtime is not excused by another runtime's blocker; the text becomes
   "intake.workload records a blocker for <rt>" when the answer names one), the gate rules
   `harness_not_generated`, `budget_left_unrevisited_items`, `coverage_contradiction_items`,
   `workload_blocker` with its users (`open_harness_step`, `build_step_harness`), `blocker_recorded`,
   and the report line "Blocked: intake.workload records ..." (name the runtime when there is one).
   A finding's runtime is what the gate already computes for it (`gate::finding_runtime_with` or
   the queue's `Ranked.runtime`).
4. `dla ledger close-rest` then needs no rule of its own: with one runtime blocked and one open it
   closes the blocked runtime's candidates and leaves the open runtime's candidates pending, and
   says so. Check that its output and `next` name the open queue's command.
5. Every text that offers the blocker carries the runtime: the queue steps text ("If no driver can
   be built or run for <rt>, record why: dla ledger resolve intake.workload --status unresolved
   --reason <...> --remedy "<text>" --runtime <rt> (that closes the queue of <rt> only; the queues
   of other runtimes stay open)"), and the other texts the survey lists (gate fixes for
   harness-not-run, no-harness, failed build, coverage contradiction, budget-left; the focus-batch
   refusal; the JUnit-repeat failure advice in dla-workload). Where a text has no runtime in scope,
   get one (the harness's runtime, the finding's runtime) rather than leaving the flag out.
6. Texts for the agent: find every place under skills/ and agents/ that describes the workload
   blocker (grep `intake.workload`, `build_fails`, `env_unavailable`) and make it agree: the
   blocker names a runtime, it closes that runtime's queue only, and it is refused for a runtime
   whose harness already produced a series. Keep `cargo test -p dla-cli --test skill_text` green
   (update its pins where the text changes).
7. `docs/VERIFY.md`: one row in the style of the neighbouring rows (state `corrected`): a blocker
   recorded for one runtime closed every runtime's queue; what is verified and what is not.

## Tests

- New binary tests (crates/dla-cli/tests; the survey names the existing ones to extend:
  queue.rs, grown_chain.rs, gate_harness.rs, keep_going.rs) on a project with two runtimes that
  both have ranked candidates (python and native are the case of the evidence run):
  1. a blocker without `--runtime` is refused and the refusal lists both runtimes;
  2. with a python harness that wrote a series: a python blocker is refused and names the harness
     and the series; a native blocker is accepted; `dla run status` then shows the native queue
     `closed: "blocked"` and the python queue open with its pending count unchanged;
  3. `dla ledger close-rest` after that closes the native candidates and leaves the python ones;
     its output names how many were left and the python queue's command;
  4. the trigger line of a confirmed or strong python finding is not "not checked (intake.workload
     records a blocker ...)" because of the native blocker (unit test in dla-core next to the
     existing trigger tests is enough);
  5. an old-style answer without a runtime, written straight into the ledger of a two-runtime run
     (or produced through the one-runtime path), still closes every queue;
  6. `--runtime` on another checklist item, on a finding, or with an unknown name is refused.
- Update the existing tests that record the blocker (the survey lists them) so they pass with the
  new rule; in one-runtime projects the flag stays optional, so most need no change. Do not weaken
  any assertion: where a test pinned the old all-runtimes behaviour, change it to the new rule and
  say so in the report.
- Each new rule seen failing with a marker mutation, then reversed (list rule, mutation, failing
  test in the report).

## Checks you run

- `cargo fmt --all`, `cargo clippy -j 2 --workspace --all-targets -- -D warnings`,
  `./target/debug/dla schema --check`.
- The whole suite once at the end (prune first):
  `cargo test -j 2 --no-fail-fast --workspace > <sp>/u75/test-all.log 2>&1`; count the
  `^test result:` lines and the failures from the log (about 17 minutes).

## Report

Lead with the result. What changed (files), the tests and the mutation each was seen failing
with, the whole-suite counts, the texts you changed (old and new, one line each), and what is not
verified. Plain sentences.
