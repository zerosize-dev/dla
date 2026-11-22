# Real plugin run spring-7 (spring-data-commons 4.1.1), 2026-11-22 06:47-07:41 UTC

Build: seventh frozen snapshot 55f31eb (test-loop-first queue with review fixes). Model
claude-sonnet-5-5 through /root/dla-real/run.py (sandbox, warm Maven home). Stopped by me after
54 minutes (exit 143) once the question was answered; the model was hand-writing drivers for the
remaining candidates. Evidence: <sp>/u36/spring-7 (run.json, transcript.jsonl, dla/ = run directory
without harness scratch copies and workload/).

## What happened

- init, intake, sweep, census (census wrote the workload discovery result itself). The model ran
  `dla harness gen --lang jvm --driver --next` once out of habit (H1, a hook driver for candidates
  without a test, never run), read the hint in its output, then said "388 candidates queued ...
  starting with the JUnit test-loop batches".
- Batch 1 (H2) by hand: `gen --junit --next`, `run`, `trend`; F436 (`ReturnedType.cache`) grew and
  was taken through sibling, test, alloc-test, refute, confirm.
- Batches 2-24: the model wrote a shell loop (`gen --junit --next`, `run`, `trend` per batch) and
  ran it in the background; all 24 batches ran. H3/S3: F144 and F160 grew.
- F160 = `TypedPropertyPaths.resolved` (TypedPropertyPaths.java:53), the documented leak:
  13.75 entries per run of the tests, confirmed by the printed route, labelled L-COLLECT, explained.
- The model also wrote an entry-path driver (H32) for F436 and F160: both read `constant` (Java
  method references at fixed call sites are one object each).
- Verifier (blind, bundle only): disagreed on both, `N-INTENT`. For F160: "deliberate metadata
  cache ... soft references ... no entry-path directory, judged on holder and mechanism alone".
  The bundle of F160 held two evidence files and nothing about the tests that fed it.
- Report: F160 is the first finding, state confirmed, "Verified: DISAGREES"; header "leak
  hypotheses: confirmed 0, confirmed but disputed by the verifier (reads N-INTENT) 2". The model's
  summary: "an unbounded holder, no trigger found, not a leak".
- Queue at the stop: 354 of 388 ranked candidates targeted in 34 harnesses (grew 4, executed 211,
  not reached 15, missed once 116); 110 pending for hand-written drivers; 128 of 180 minutes left.

## Result

The documented leak is now found, measured and confirmed by a real run without any hand-written
unit. It is not yet reported as a leak: the verifier reads it as an intended cache.

## Causes to fix (brief U, wip-notes/verifier-testloop-brief.md)

1. The verifier is not given the tests that fed the growth (F160: TypedPropertyPathUnitTests and
   KTypedPropertyPathUnitTests) and has no rule for a cache whose key is an identity-compared
   object made per call (it never hits) or for what soft references do and do not bound.
2. The trigger line names the workload of the batch's first class (QuerydslBindingsUnitTests), not
   the classes chosen for the finding.
3. A JVM candidate held in an instance field has a `jvm-class` probe that reads static fields
   only; with the class loaded and no static field it gets no column and `dla trend` says
   `not_loaded` ("the workload never loaded that code"). Wrong for F8 (S1) and F60 (S52, S53). The
   queue then asks for focus drivers that can never read anything else.

## Other observations (not changed)

- The model generated a hook driver before reading the status (one wasted harness).
- F436 (`ReturnedType.cache`): the tests build a new factory per test; `N-INTENT` is a fair
  reading there.
- 17 of 18 runs of the tests in batch 1 had a failing test (a Mockito spy on a mock that survives
  the first run); the series was kept (the new rule) and the first failure is a limitation.
- The residual of the JUnit series (about 0.9 MB per iteration, Mockito and test churn) produced
  many gap rows and open items the model closed as `inconclusive` one by one.
