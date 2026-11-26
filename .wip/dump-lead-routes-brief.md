# Builder O: dump leads the model can decide without confirming a false leak

Repository: /home/user/dla. Work in the working tree. Do NOT commit, push or switch branches.
Another builder (P) works in the same tree at the same time on `crates/dla-cli/src/alloc/` and
`crates/dla-cli/tests/alloc_test.rs`: do not edit those. Do not revert or reformat files you did
not write. Add one row per cause at the end of docs/VERIFY.md (re-read it first; P adds rows too,
so re-read just before you write). Do not touch harness/, bench/fixtures/ or any fixture README or
answer key.

## What a real run showed (do not name the project in the repository)

An unattended run on a JVM library ran `dla run abort` after 76 of 180 minutes, with this reason:
"remaining open items are heap-dump holders owned by the measurement driver, the JFR/DLA agent or
the JUnit loop (...) that the ledger lets me only confirm as leaks; confirming them would be false,
so the run is ended with them undecided". The findings were added from heap-dump growth (origin
`growth-series`, added through the gate's residual-capture item `dla ledger add --origin
growth-series --evidence E<n>`). Their owners:
- static fields of the model's own driver class (`<project package>.DlaLoop.TCP`, `.MASK`,
  `.PARSER`, `.MDC`, ...: the driver keeps its encoders in statics on purpose);
- the DLA agent (`dla.harness.DlaDump.ROOT_CLASS`);
- the JDK flight recorder the capture starts (`jdk.jfr.internal.StringPool.cache`);
- JDK caches (`java.net.InetAddress.cache`, `.expirySet`) and `thread:main` thread-locals.
Growth over dumps makes a dump lead (`crates/dla-core/src/grown.rs`, `dump_grown`, from an earlier
round). The only route a dump lead offers is the confirm route (`confirm_commands`). The lead is
lifted only by a newer dump discriminator that does not favor leak, an alloc-test that favors
non-leak, or a leak confirmed on dumps of the same capture. Nothing the model could do decided
these holders, so it aborted, and its 35 confirmations went without the verifier.

A second case, from CI: a non-leak JVM fixture (`jvm-invoice-render-trimmed`, a bounded holder) had
its heap-graph gate pass by chance while the holder filled. The dump lead's route ended at
`dla ledger annotate F9 --site <file:line>` (a placeholder), the bench policy stopped there, and the
final gate failed with F9 under test. Locally the same tree passed (the gate failed by chance).

## The change

1. **The measurement's own holders are never leads.** A holder whose owner belongs to the
   measurement and not the project is left out of findings, dump leads and residual-capture
   items. The owner belongs to the measurement when it is:
   - a class or module of the harness's own driver sources (read where the driver files of a
     harness live, `harness/<H>/driver/` or what the manifest records, and how class names are
     derived from them per runtime);
   - the DLA agent (the `dla.` namespace of the JVM agent, the agent files of the Node and Python
     harnesses: find them under harness/);
   - the profiler the capture starts (JFR: `jdk.jfr.`).
   JDK, runtime and third-party classes stay leads. The project can fill a JDK cache, and a
   `thread:<name>` holder can hold the project's own ThreadLocal values. Do NOT exclude those.
   Where the summarizers print their holder table and the `needed`/residual commands, list the
   measurement's own holders on one line ("the measurement's own holders, left out: ..."). Find
   every place that turns a dump holder column into a finding or an item (`summarize heap-graph`
   for JVM, Node and Python, `gate.rs` residual capture, `measured_holders.rs`, `dump_grown`), and
   use one shared predicate in dla-core.
2. **A dump lead can be decided by a confirmed non-leak explanation.** Confirming a non-leak
   sibling of the root (an `N-*` hypothesis such as N-OBSERVER or N-INTENT) lifts the dump lead,
   like the other lifts in `dump_grown` (newest record decides, as there). First read what
   `dla ledger confirm` requires of a non-leak sibling (crates/dla-core/src/machine.rs and the
   ledger's refusals). Require evidence that shows the holder and what keeps it: a retention-path
   or heap-graph record of that holder (authority `dump`). Make the confirm of such a sibling
   refuse other evidence, with a fix naming that record. The route a dump lead prints (in the
   refusal, the gate item and `dla run status`) then offers both ways: the confirm route as today,
   or "if the growth is not a leak of the project: dla ledger add --sibling-of <F> --origin
   hypothesis --hypothesis N-OBSERVER|N-INTENT --refute-if \"...\" --evidence <dump record>; dla
   ledger confirm <F>.n<k> --evidence <the retention-path record of the holder>". Keep the texts
   short and every command runnable as printed, except for the choice between N-OBSERVER and
   N-INTENT.
   Check: can the root be refuted or closed after its sibling is confirmed, as for other confirmed
   non-leak siblings? Say what happens.
3. **No placeholder blocks a dump lead.** When the dump route of a lead needs an annotation it
   cannot fill (`--site <file:line>`, `--holder <holder>`), it first offers `dla alloc-test <H>
   --finding <F>` on the harness the dumps came from (an alloc-test favoring non-leak already lifts
   a dump lead). In the bench policy (bench/runner/src/dynamic/policy.rs, `follow_growth_route`),
   when the route reaches a command with a placeholder, the policy runs that alloc-test instead and
   follows its result. That is the mechanical stand-in for a model that would annotate or measure.
4. **Docs.** stages.md: one sentence each for 1 and 2 where dump leads are explained. Do NOT edit
   SKILL.md. VERIFY rows.

## Tests

- dla-core: the predicate (driver class of a harness, agent namespace, JFR internals: measurement;
  a JDK cache, `thread:main`, a project class: not).
- CLI (find the dump-lead tests: crates/dla-cli/tests/grown_dump.rs): a dump series growing a
  holder of the driver's class and one of the agent makes no lead and no residual item, and the
  summary names them on the left-out line; a project holder in the same dumps still makes a lead.
- CLI: a dump lead on a JDK cache holder: add an N-OBSERVER sibling, confirm it with the
  retention-path record, and the lead is gone. Confirming it with an unrelated record is refused.
  A newer growth record brings the lead back (newest decides).
- CLI: a dump lead with no site: the route's first command is the alloc-test. An alloc-test favoring
  non-leak on that harness lifts it.
- Bench: a policy unit test, or the fixture `jvm-invoice-render-trimmed`, if you can make its
  heap-graph gate pass deterministically. If you cannot, say so and test the policy with a faked
  route.
- Mutation checks: the predicate always false (the driver-holder test fails); the sibling lift off
  (the N-OBSERVER test fails); the placeholder alternative off (the no-site test fails).

## Checks to run

`cargo test -p dla-core --lib`, `cargo test -p dla-cli --bin dla`, the dla-cli test files you touch
plus `grown_dump`, `grown_chain`, `gate_flow`, `residual_capture`, `summarize_jvm`,
`summarize_node`, `summarize_python` (whichever exist), `cargo test -p dla-bench --lib`, then
`cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all --check`. Then the
dynamic acceptance on the JVM and Node fixtures (`--only jvm-`, `--only node-`) with
`./target/debug/dla-bench dynamic --dla-bin ./target/debug/dla --parallel 2 --only <substring>
--json <scratch file>`. Every non-leak twin must stay non-leak, and every leak fixture must still
confirm its leak.

## Rules

Shared `target/` only (P builds in it too; cargo waits on the lock, that is fine). Disk is tight:
before building check `df -h /`. Under 3 GB free, delete stale hash dirs older than 120 minutes
under the literal paths /home/user/dla/target/debug/build/dla-cli, .../dla-core, .../dla-workload
and .../dla-bench with `find <literal path> -mindepth 1 -maxdepth 1 -type d -mmin +120 -exec rm
-rf {} +`. No `rm -rf "$VAR/..."`. Never kill processes by name. Never skip or weaken a test. No
project names, attribution lines or model names in the repository. Scratch:
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u147/.

## Report (final message)

Files changed; the rule as implemented for each item (the predicate's exact namespaces and how
the driver's classes are found per runtime, what the ledger requires to confirm the sibling, the
route texts); tests with what each proves; mutation checks; every command with pass/fail counts;
dynamic fixtures with results; existing tests whose expectations changed and why; anything you are
unsure of.
