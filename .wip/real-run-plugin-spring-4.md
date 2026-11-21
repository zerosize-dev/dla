# Real plugin run spring-4 (2026-11-21 10:18 to 10:57 UTC), spring-data-commons 4.1.1, combined build

39 minutes of a 180-minute budget, 130 turns, report INCOMPLETE (477 open), no leak confirmed.
Kept: scratch u36/spring-4/ (report.md, ledger.jsonl, transcript.jsonl, H5 driver, and the
whole project copy with its run directory: u36/spring-4/project/.dla/, 150 MB, for timing).

What happened
- 602 findings, 389 ranked. The documented holder (static map `TypedPropertyPaths.resolved`,
  TypedPropertyPaths.java:53) is F135; it was a declared target of H4 and H5 (batches 4 and 5,
  9 targets each). Its block said: "writers: TypedPropertyPaths.java:111 (TypedPropertyPaths.of),
  :149 (TypedPropertyPaths.of) ... probe reads: static field ... to do: make every unit reach a
  writer".
- The model's unit in H4 and in H5 called code for three of the nine targets ("a fresh hidden
  class per unit for F288 and F57", then F57 and F125) and never called
  `TypedPropertyPaths.of`. `dla trend` read F135 `not_loaded` in both. Having been declared in
  two different drivers it was settled by the two-drivers rule; the model moved on.
- Same pattern as the Go run: a batch names 9 to 16 targets, the model writes code for 2 or 3,
  the others are settled after their second declaration without ever being attempted.
- "Queue: 36 of 389 ranked candidates targeted in 4 harnesses: grew 1, executed 3, not reached
  32; 353 not targeted".
- `dla trend` on this ledger takes about a minute per series on a quiet machine (62 s for S8:
  38 s of CPU; every trend report carries verdicts for about 320 findings, 300 of them
  `not_loaded`), more than two minutes under load: the model's command ran into the 120 s
  foreground limit again and again and it spent many turns waiting for background trends.
- The model stopped when a batch needed "a real MappingContext or Spring container" ("I can't
  clear them without building real fixtures"); the stop hook let the unchanged ledger through.

Causes to fix
1. A target that a big batch declared and whose code did not run must get a real attempt: a
   small harness of its own (at most three targets) that names the writer functions to call.
   Only that, or an executed verdict, or an explicit waiver with a reason, settles it. Being
   named in two big batches is not an attempt.
2. `dla trend` speed on a ledger of several hundred findings (and the other commands of the
   loop: run status, ledger list, harness gen --next, gate check).
