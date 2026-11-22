# Real plugin run spring-6 (spring-data-commons 4.1.1, sixth frozen build = snapshot 4a72a50), 2026-11-22

Started 02:53 UTC, 933 s wall, 108 turns, cost 1.67, Status INCOMPLETE (1 open: the jvm queue),
10 harness runs, 28 series, 0 captures, budget 3 h, 15 min used. 604 findings, 388 ranked, 35
targeted in 11 harnesses, 353 not targeted. Evidence: u36/spring-6 (run dir deleted).

- No documented leak confirmed. 1 confirmed but disputed by the verifier (F551
  `ParameterTypes.cache`, N-INTENT).
- The documented holder F160 `TypedPropertyPaths.resolved` was reached again: H9 not loaded,
  focus harness H11 constant. The driver file of H11 carried the new `key:` line (three
  times, once per target). The model wrote
  `TypedPropertyPaths.of((PropertyReference<Class<?>, String>) Class::getName)`: one method
  reference, one key for the whole run. Its remedy: "F160 constant (weak/soft cache keyed by
  call site)". It understood the cause and did not act on the line: no capturing lambda, no
  factory form. A printed hint did not change what the model wrote (as in tokio-3).
- The model tried to stop from the first batch on ("I can't clear the rest by hand", "each
  batch of 2 to 3 candidates needs a hand-filled driver"), 11 stop attempts, and was let go
  with 164 of 180 budget minutes left. It never tried the JUnit repeat (`--junit`); in
  spring-5 it did and every unit failed: Mockito "Could not self-attach to current VM using
  external process" in this sandbox.

Reading:
1. The key line is true and did nothing here. In Java this library accepts only the factory
   form; the natural trigger is a Kotlin caller (a SAM conversion of a property reference
   makes a new object per evaluation). The project's own Kotlin tests do exactly that.
2. On a JVM project with a test suite the cheap broad workload is the project's own tests
   in a loop. Here that route is dead for an environment reason (the mocking library
   attaches to the running JVM through an external process), so every candidate costs a
   hand-written unit and the model gives up. A mocking library can be loaded as a start-up
   agent instead; newer JDKs and hardened containers refuse the attach anyway.
3. The queue asks for 388 hand-written measurements. No model run has gone beyond about 60.
