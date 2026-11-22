READ-ONLY survey of /home/user/dla. No cargo run, no edits. All paths are under crates/. Line numbers are from the working tree as it is now.

KEY FINDING, checked against the Pillow run: the tree already has a refusal that names a series, and it did not fire for Pillow because it is all-runtime. `dla ledger resolve intake.workload --status unresolved --reason <blocker>` is refused with code `workload-ran` (core/src/error.rs:133-135, ledger_cmd.rs:1141-1171). It is refused only when EVERY runtime that has ranked candidates has a harness series (gate.rs:1770-1793, `workload_ran`). For Pillow, python had series and native had none, so `wanted.iter().all(..)` was false, the function returned None and the blocker was accepted. Once accepted, the blocker closes ALL queues, because `Closed::Blocked` is not per runtime (queue.rs:934, 1085).

1. WHERE intake.workload=unresolved with a blocker reason is read

Reason set: `is_blocker_reason` (core/src/gate.rs:1751-1763) is true for EnvUnavailable, ToolMissing, BuildFails, PrivilegeDeclined. NoHarness, NotExercised, TimeBudget and Inconclusive are not blockers.

Core predicate, `workload_unresolved(view, blocker_only)` (gate.rs:2153-2169): true if ANY ledger ChecklistAnswer for `intake.workload` has status Unresolved, a matching reason, and a non-empty remedy. It has no runtime parameter and ignores which runtime the remedy text names. With blocker_only=false it accepts any reason except time_budget while time is left.

Callers of `workload_unresolved(.., true)`:
- queue.rs:934, in `queues()`: `let blocked = ...` is computed once for all runtimes. It then does three things.
  - queue.rs:962 turns off the second-look pass.
  - queue.rs:1004 turns off the JVM test-loop plan.
  - queue.rs:1085-1086 `else if blocked { Some(Closed::Blocked) }`. This is checked after Off and Floor/Over, before Empty. Pending candidates stay listed but `q.open()` (queue.rs:795) is false. `Closed::text` = "intake.workload records a blocker" (queue.rs:596). This is the `closed: "blocked"` in `run status`.
- trigger.rs:598-600, `unchecked_because`: returns "intake.workload records a blocker". The report text is built at trigger.rs:764, `format!("not checked ({why})")`, which gives "Trigger: not checked (intake.workload records a blocker)". `trigger::items` (trigger.rs:622-624) returns no blocking items when `unchecked_because` is Some, so the final gate stops asking for entry-path runs.
- gate.rs:2382, `harness_not_generated`: returns None, so the "no harness generated" rule is lifted for every runtime.
- gate.rs:2581, `budget_left_unrevisited_items`: returns early, so the "budget left, findings closed unmeasured" gate item is cleared for all runtimes.

Callers of `workload_unresolved(.., false)`: gate.rs:2538, `coverage_contradiction_items`. Any non-time_budget unresolved answer clears that item for all runtimes.

Other readers of the blocker:
- `workload_blocker(view)` (gate.rs:2407-2428) returns the latest blocker as "reason: remedy".
  - gate.rs:2466-2467, `open_harness_step`, returns `HarnessStep::Blocked`.
  - dla-report/src/lib.rs:411-414 prints "Blocked: intake.workload records {why}." in the Coverage warning.
  - gate.rs:2265, `build_step_harness`, returns None.
  - Neither takes a runtime. Both apply to whichever runtime is asked about.
- `blocker_recorded(view, h)` (gate.rs:1826-1839) is true if a blocker was recorded after harness h was generated. Used at gate.rs:1906 (`unrun_harness`, so the unrun-harness rule is lifted) and gate.rs:1993. It is not runtime-scoped.
- cli/src/run.rs:715: `next_command` treats revisit items closed with a blocker reason as not a next step. It reads finding reasons, not intake.workload.

`dla ledger close-rest` (cli/src/ledger_queue.rs:296-):
- It builds `queues = queue::queues(..)` (:303) and leaves a candidate only if `q.open() && q.pending.contains(id)` (:350-361).
- With the blocker recorded, all queues are non-open, so every candidate falls through to resolve (:402-424). The reason is `NotExercised` (:389, or Inconclusive if measured, TimeBudget if Closed::Floor).
- `resolve_finding` then calls the target-pending refusal, which returns Ok when there is no open queue (ledger_cmd.rs:~752 `let Some(q) = q else { return Ok(()) }`; the pending refusal is at :779-807).
- Exceptions that still stay open: a column that grew (:335-345 and :362-371) and the grown gate in ledger_cmd.rs:1125-1135.
- This is how 567 findings were closed.

Gates, summarized: with the blocker recorded, the queue item is gone, `trigger` items are gone, `harness-not-generated`, `budget_left_unrevisited` and `coverage_contradiction` are cleared. `harness-not-run` (ledger_cmd.rs:880-915) and `blocker_recorded` also lift. The final gate passes through these.

2. DOES THE BLOCKER KNOW A RUNTIME?

No. Not found in any of these places:
- `ChecklistAnswer` fields (ledger_cmd.rs:1088-1095): item, status, evidence, note, reason, remedy. There is no runtime field.
- `ResolveArgs` (ledger_cmd.rs:236-262) flags: target (positional), `--as` (unresolved|suspected_static_only), `--reason`, `--remedy`, `--by`, `--status`, `--evidence` (comma list), `--note`, plus a hidden `--archetype` in the struct above it (:215-218). There is no `--runtime`.
- `resolve_checklist` rejects `--as` and `--by` (ledger_cmd.rs:1074-1077).
- Convention in the remedy text: not found. The remedy is only checked for non-empty (ledger.rs ~1465 and gate.rs:2166). `refuse_contradicted_workload_blocker` ignores the remedy.
- `checklist.rs:46` defines "intake.workload" as "What can exercise the suspected path?", with no runtime.

3. MESSAGES THAT TELL THE AGENT TO RECORD THE BLOCKER (text and condition)

- queue.rs:1151, `steps(q)`. Printed whenever an open queue gives guidance (refusal fix, `run status`, gate queue item). Text: "If no driver can be built or run for {runtime}, record why: dla ledger resolve intake.workload --status unresolved --reason <env_unavailable|tool_missing|build_fails|privilege_declined> --remedy \"<text>\" (that closes the queue of the runtime)." This is the "closes the queue" string; the text says "of the runtime", the code closes all queues. It does not mention that other runtimes stay open or that a series exists.
- ledger_cmd.rs:779-781 (focus-batch refusal, TARGET_PENDING, `q.missed` case): fix names the reasons "env_unavailable, tool_missing, build_fails, privilege_declined" per finding, not intake.workload.
- ledger_cmd.rs:807-812 (generic TARGET_PENDING): expected = "a workload whose units reach {id} (...), or the blocker of the {runtime} workload recorded"; the fix is `queue::steps` (the text above).
- ledger_cmd.rs:902-903 (HARNESS_NOT_RUN): "{harness} run, or its blocker recorded on intake.workload", fix from `u.fix`.
- gate.rs:1729 (harness-not-run fix): "If {h} cannot run, record why: dla ledger resolve intake.workload ... --remedy \"<what blocks {h}>\"..."; printed when a harness was generated and not run.
- gate.rs:2143 (`NoHarness` fix): "...or record it once: dla ledger resolve intake.workload --status unresolved --reason <...> --remedy \"<what blocks the harness>\"..."; printed when no harness exists and entry points exist.
- gate.rs:2321: failed-build text, "...or record the blocker with dla ledger resolve intake.workload --status unresolved --reason build_fails --remedy \"<what breaks the build>\"".
- gate.rs:2553-2559 (coverage_contradiction): `--reason <env_unavailable|tool_missing|build_fails|privilege_declined>` while `open_queue(view, rt)` is Some, otherwise `not_exercised`.
- gate.rs:2633: budget_left_unrevisited fix, "If nothing can be measured, record why: dla ledger resolve intake.workload ...".
- dla-workload/src/harness/run.rs:264: JUnit-repeat harness failure advice, `--reason env_unavailable --remedy "<the service {id} needs>"`.
- ledger.rs:1444 and 1459: REASON_REQUIRED and INVALID_VALUE refusals that list the reasons for a checklist item (no_harness, not_exercised etc.).
- bench/runner/src/dynamic_cmd.rs:20-23, 1120, 1215, 1744-1751: benchmark logic, noted for completeness only.
Not found: any text that says a blocker closes only one runtime, or that offers `--runtime`. The agent's skills/agents text under /home/user/dla/{skills,agents,harness,data} was not searched in this pass (skill_text.rs:418 pins "`build_fails`" in a skill, so there is text there to check).

4. WHAT THE TOOL KNOWS ABOUT HARNESSES PER RUNTIME

- `RunView::with_disk_facts` (core/src/view.rs:119-143) loads each series file and fills `series_harness: series -> harness` and `harness_series` (set of harnesses with a series), from `SeriesFile.source.harness`. This is the "harness ran and produced a series" fact.
- `HarnessManifest::load(run_dir, h)` gives `m.runtime`. `gate::harness_ids(run_dir)` (gate.rs:1796-1807) lists H1..Hn.
- `queue::Index::of(view)` (queue.rs:289-366): `runtime: harness -> runtime` for every non-clone harness, and `run: set of harnesses with a series`; `harness_ran(h)` (:~372). Combining the two gives "runtime R has a harness with a series". No helper does this per runtime yet; `queues()` computes `ix` (queue.rs:920) before `blocked`, so it is in scope.
- `gate::workload_ran(view)` (gate.rs:1770-1793) already builds `ran: runtime -> (harness, series)` from `series_harness` + manifests, but returns Some only if all wanted runtimes are in it.
- `gate::runtime_has_harness(run_dir, runtime)` (gate.rs:2349-2354): harness with manifest of that runtime, run or not.
- `gate::harness_ran(view, run_dir, h)` (gate.rs:1851-1860): attempt that ran, a run-*.json, or a series.
- `queue::ranked(view)` (queue.rs:162) gives `Ranked.runtime` per candidate; `queues()` groups by runtime (queue.rs:923-931).
- Caveat: `Index::of` skips clone harnesses (CLONE_MARKER, queue.rs:301-306), `workload_ran` does not.
- Which string names the native runtime vs python: `Ranked.runtime` / `m.runtime.as_str()`; the blocker's remedy text carries no runtime key, so a runtime argument is needed.

5. TESTS THAT PIN THE CURRENT BEHAVIOUR

Blocker closes a queue:
- cli/tests/queue.rs:272 `a_workload_answer_that_is_not_a_blocker_leaves_the_queue_open` (asserts `st["queue"]["python"]["closed"] == "blocked"` at :304, then close succeeds).
- cli/tests/queue.rs:212 `a_candidate_cannot_be_closed_unmeasured_and_the_gate_and_status_lead_with_the_queue` (asserts the fix contains "dla ledger resolve intake.workload" at :238; blocker reason on a finding accepted at :244).
- cli/tests/queue.rs:311 `with_the_queue_off_everything_is_as_before` (build_fails close at :341).
- cli/tests/grown_chain.rs:193 `a_workload_blocker_is_refused_while_a_harness_ran_for_every_runtime_with_candidates`: code `workload-ran`, message "harness H1 ran and wrote series S1", queue python stays open (:213-220). Also grown_chain.rs:223 `a_target_read_unknown_does_not_make_the_workload_unrunnable` and :627 `a_workload_blocker_is_accepted_when_no_harness_ran`; the comment at queue.rs:951 mentions workload-ran.
- No test found for one blocked runtime plus one open runtime.

close-rest after a blocker:
- cli/tests/queue.rs:763 `close_rest_says_inconclusive_for_a_measured_candidate_and_not_exercised_for_the_rest` (records build_fails on intake.workload first, "The queue holds nothing back", :769-776).
- cli/tests/queue.rs:467 `verdicts_settle_candidates_and_close_rest_closes_the_rest`, :976 `a_stamped_run_with_time_left_keeps_its_queue_and_close_rest_lifts_the_stamp`, :823 `at_the_budget_floor_...` (floor path, not blocker).

Gate and report texts:
- cli/tests/gate_harness.rs:250 helper `answer_workload_blocker`, :319 `the_reasons_that_name_a_blocker_are_never_refused`, :416 `an_attempt_that_ran_or_failed_the_workload_clears_the_rule_and_a_failed_build_keeps_it`, :474 `a_blocker_on_intake_workload_after_generation_clears_the_rule`; :302 asserts a fix contains "intake.workload".
- cli/tests/keep_going.rs:398 `a_blocker_reason_is_never_refused` (:1317 pins "--reason <env_unavailable|tool_missing|build_fails|privilege_declined>"), :412 `a_blocker_on_the_workload_item_clears_the_rule`, :889 `a_recorded_workload_blocker_replaces_the_step_in_the_warning` (pins "Blocked: intake.workload records env_unavailable: the service needs a database."), :515 and :624 pin fix text "intake.workload --status unresolved --reason not_exercised --remedy" and "--status unresolved".
- core/tests/trigger_items.rs:247 `a_workload_blocker_asks_nothing`; core/tests/inconclusive.rs:16 `a_checklist_answer_refuses_inconclusive`, :52 `the_gate_decides_every_reason_explicitly`.
- cli/tests/skill_text.rs:418 pins skill text with `build_fails`.
- Also set up the blocker: gate_flow.rs:86, gate_hooks.rs:130, budget_auto.rs:94.
- I did not find a test that pins the string "Trigger: not checked (intake.workload records a blocker)"; trigger.rs:599 is the only source. A grep of tests for it was not done beyond the greps above (listed "not checked" hits were all in src).

6. MINIMAL CHANGE PROPOSAL (no code)

Design: the blocker names a runtime; blocked is per runtime. Add an optional `runtime` to the checklist answer and to `dla ledger resolve`.

Touch points:
a) Data and CLI. `ChecklistAnswer` (core, the record type used at ledger_cmd.rs:1088) gets `runtime: Option<String>`, serialized only when present (ledger schema version bump if the ledger is versioned, as for v4-v6). `ResolveArgs` gets `--runtime <python|native|jvm|node|go|rust...>` (ledger_cmd.rs:236-262). `resolve_checklist` rejects `--runtime` for non-workload items and for findings, and requires it with a blocker reason on intake.workload (otherwise a refusal "name the runtime the blocker is about: --runtime <rt>", listing the candidate runtimes). A blocker without `--runtime` should stay accepted only if exactly one runtime has ranked candidates (decide: required vs implied; old ledgers have none, so treat an answer without runtime as "all runtimes" for reading only, keeping old runs working).
b) Reading. Replace `workload_unresolved(view, true)` with `workload_blocked_for(view, runtime: Option<&str>)` (gate.rs:2153) that matches answers whose runtime is None (legacy) or equals the runtime. Apply at the callers:
- queue.rs:934, compute `blocked` inside the `by_rt` closure per runtime, so queue.rs:962, 1004 and 1085 use the runtime of that queue.
- gate.rs:2382 already has `runtime`, pass it.
- trigger.rs:598 `unchecked_because(view)` has no runtime; give it the finding's runtime or compute per finding (and make `trigger::items` at trigger.rs:622 per runtime).
- gate.rs:2581 `budget_left_unrevisited_items` needs per-finding runtime (it already groups `by_runtime`, gate.rs:2584-2586).
- `workload_blocker` (gate.rs:2407) and its users `open_harness_step` (gate.rs:2466, has `runtime`), `build_step_harness` (gate.rs:2265, has `runtime`), `blocker_recorded` (gate.rs:1826).
- The report text at dla-report/src/lib.rs:413 and trigger.rs:599 should say which runtime: "intake.workload records a blocker for {runtime}".
- gate.rs:2538 (`workload_unresolved(.., false)`) per runtime `rt`, which is already a loop variable there.
c) Refusal. Change `workload_ran` (gate.rs:1770) from the all-runtimes rule to the named runtime: `workload_ran(view, runtime) -> Option<(harness, series)>` returns the first series of a harness whose manifest runtime == runtime, from `view.series_harness` + `HarnessManifest::load`, the code it already has at gate.rs:1780-1786. Keep `refuse_contradicted_workload_blocker` (ledger_cmd.rs:1141) and pass the runtime from the new flag. Message already names harness and series ("harness H1 ran and wrote series S1"); keep the wording, add the runtime: "intake.workload cannot be recorded as {reason} for {runtime}: harness {H} ran and wrote series {S}". Code `workload-ran` stays. For the Pillow case the python blocker would be refused (series exists); a native blocker would be accepted.
d) close-rest follows automatically. `close_rest` (ledger_queue.rs:296) uses `q.open()` per queue (:350), so once `Closed::Blocked` is per runtime, a candidate of the open runtime stays "pending in the {rt} queue" and only the blocked runtime's candidates are closed (:361-371 stays as is).
e) Texts. queue.rs:1151 `steps(q)`: add `--runtime {q.runtime}` to the command and change the parenthetical to "(that closes the queue of {runtime} only; other runtimes stay open)". Add the same `--runtime` to gate.rs:1729, :2143, :2321, :2553-2559, :2633 (those have a runtime in scope or can get one) and run.rs:264 of dla-workload. Also check skills/ and agents/ text and skill_text.rs:418.

Tests that change:
- cli/tests/queue.rs:272 (`a_workload_answer_that_is_not_a_blocker_leaves_the_queue_open`): add `--runtime python` to the resolve call at :294-300.
- cli/tests/queue.rs:763 (close_rest after blocker), :212 (fix text), :311.
- cli/tests/grown_chain.rs:193, :223, :627: add `--runtime`; add the case of a blocker for a different runtime than the one with a series (accepted).
- gate_harness.rs:250/319/416/474, keep_going.rs:117/398/412/889/1317, gate_flow.rs:86, gate_hooks.rs:130, budget_auto.rs:94, queue.rs:769, grown_chain.rs:204 and 238 (all calls to resolve intake.workload with a blocker reason, plus tests pinning the old fix strings).
- core/tests/trigger_items.rs:247 and core/tests/inconclusive.rs:263-285 (ChecklistAnswer literals need the new field).
- dla-report tests for the new "Blocked:" text, and the test that renders "Trigger: not checked".
New tests: (1) one blocked runtime plus one open runtime: `run status` shows `queue.native.closed = "blocked"` and `queue.python.open = true`; (2) `close-rest` closes the native candidates and leaves python ones with "pending in the python queue"; (3) a python blocker with a python series is refused naming H and S, a native blocker with no native harness is accepted; (4) a legacy answer without runtime still closes all queues; (5) `--runtime` on a finding or on another checklist item is refused.