# Real plugin run spring-8 (spring-data-commons 4.1.1), 2026-11-22 09:55-10:55 UTC

Build: eighth frozen snapshot b3b5ff4 (measured test-class attribution, verifier inputs, JVM
verdicts). Model claude-sonnet-5-5 through /root/dla-real/run.py (sandbox, warm Maven home). The run
ended by itself: exit 0, 3606 s, 107 turns, 116 tool calls. Evidence: <sp>/u36/spring-8 (run.json,
transcript.jsonl, dla/ = run directory without harness scratch copies and workload/).

## Result

Report status COMPLETE, 59 of 180 budget minutes used.
"leak hypotheses: confirmed 1, confirmed but disputed by the verifier (reads N-INTENT) 1".

- The confirmed one is the documented leak, first in the report: F282 `TypedPropertyPaths.resolved`
  (TypedPropertyPaths.java:53), L-COLLECT, 13.67 entries per iteration, "Trigger: reproduced through
  the public API in H3 (varies: the project's own tests KPropertyReferenceUnitTests (14.6 entries per
  run))". The verifier agreed on holder, mechanism and label.
- The disputed one is `ReturnedType.cache` (F555): fed by ReturnedTypeUnitTests, which builds a new
  factory per test; the verifier reads it as intended. A fair reading.
- Queue: 364 of 367 ranked candidates targeted in 32 harnesses (grew 2, changed 1, executed 301,
  missed once 58).

## Remaining defects seen

1. "Verified: DISAGREES (site ...)" on the confirmed leak: the finding's site is the declaration
   (line 53), its recorded writers are lines 111 and 149, the verifier named the adding call at line
   116 (5 lines from a writer, tolerance is 3). Same function. The rule should accept a site in the
   same function as the finding's site or one of its writers.
2. The model closed most of the 58 missed candidates with `--reason env_unavailable` and says itself
   "that reason is a stretch for some of them". The tool accepts a blocker reason per candidate
   without any evidence of a blocker.
3. Whole-process heap and non-heap growth of the JUnit loop is above threshold in 86 of 92 series
   and is closed as inconclusive in bulk (O3, O4): the residual of a test loop (mocking library,
   test churn) is noise the report carries as 92 rows.
4. F214 (`resolvableTypeCache`, a 128-entry LRU): the alloc-test favoured a leak; the model left it
   unresolved from reading the source. The plateau window was too short for a cache that fills
   to 128.
