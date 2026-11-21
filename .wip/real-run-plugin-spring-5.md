# Real plugin run spring-5 (spring-data-commons 4.1.1, third frozen build, snapshot 89bc8a8), 2026-11-21

Started 21:10 UTC, 4501 s wall, 252 turns, cost 5.85, Status INCOMPLETE (1 open: the queue),
30 harness runs, 113 series, 1 capture, budget 3 h, 1 h 14 min used. 712 candidates, 367
ranked, about 60 targeted; 23 stop attempts. Evidence: u36/spring-5 (run dir deleted).

- No leak confirmed (1 confirmed but disputed by the verifier, N-INTENT).
- The documented holder was reached and measured: F282 TypedPropertyPaths.resolved
  (TypedPropertyPaths.java:53): H10 not loaded, focus harness H13 constant: "probe stayed flat
  at 190 with a bounded set of lambdas". Closed unresolved(inconclusive) with the remedy
  "needs lambdas from many class loaders" (a wrong guess).
- Why flat: the documented trigger is a loop of Kotlin property references (a new reference
  object per evaluation). The driver is Java; Java method references at one call site are one
  cached object, so the key never changes. The project's own Kotlin tests could do it, but the
  JUnit loop is blocked here: Mockito cannot self-attach to the JVM in this sandbox.
- Big-project pattern again: the model gives up on the queue long before the budget
  ("each batch needs a hand-written driver").

Reading: this leak needs a caller in another language than the driver's. Not a defect of the
measurement; a limit of the Java driver (no Kotlin unit) plus the sandbox (JUnit loop).
