# Brief Q: a holder keyed by the object the caller passes needs a fresh object per unit (real run spring-5)

Rules as in your earlier briefs (Sonnet only; no commits; shared `target/`, `-j 2`; small exact
edits; literal paths in `rm`; never delete build directories under `target/` yourself, use
`<sp>/wip-notes/prune-superseded.sh` when disk is short (about 4 GB free now: delete your
copies and build outputs as you go); no process kills by name; unique-marker mutations
reversed by exact replacement; no test, fixture or bench policy weakened; never `dla consent`
on a real tree (unattended-runner environment on `dla run init`). CI is running on a frozen
snapshot of the tree; your edits stay uncommitted. Scratch `<sp>/u60/`, report
`<sp>/u60/report.md`, short final message.
`<sp>` = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad

## What happened (evidence `<sp>/u36/spring-5/`: run.json, transcript.jsonl, `dla/`; notes
`<sp>/wip-notes/real-run-plugin-spring-5.md`)

Real plugin run on spring-data-commons 4.1.1 (source `/root/dla-real/src/spring-data-commons`,
never work in the original: copy it). The documented leak is the static cache
`TypedPropertyPaths.resolved` (`src/main/java/org/springframework/data/core/TypedPropertyPaths.java:53`):
a map per class loader whose key is the lambda or property reference THE CALLER PASSES to
`TypedPropertyPaths.of(...)` (lines 103 to 115 and 142 to 152: `cache.computeIfAbsent(lambda, ...)`).
The run reached it: focus harness H13 read the probe constant at 190 "with a bounded set of
lambdas", and the model closed it unresolved with a wrong guess ("needs lambdas from many
class loaders"). The documented trigger is a loop of Kotlin property references, which are
a new object per evaluation. My note then said "needs a Kotlin caller". I now think that is
not the cause: the key is an object identity, and the unit passed the same few objects every
time. A Java method reference or a lambda that captures nothing is created once per call
site, so every unit used the same keys. A lambda that captures a local of the unit, or an
instance made with `new`, is a fresh object per evaluation. Not checked: whether the
library's reader (`SerializableLambdaReader`) accepts a capturing lambda.

## Wanted

1. Experiment first, on a copy (`<sp>/u60/spring`): init with the unattended-runner
   environment, probe, sweep, census (or reuse what you need of the evidence), a focus
   harness for the finding at `TypedPropertyPaths.java:53` only. Write the unit so that it
   passes a FRESH key object per unit, in Java (try in this order and say what each does: a
   serializable lambda that captures a local of the unit; a new instance of a small class
   implementing the functional interface; anything else the API accepts). Report whether
   the probe of `resolved` grows per unit and what `dla trend` prints. If no Java form can
   make a fresh key the library accepts, stop after this step and report exactly why (the
   exception, the code line): then the Kotlin unit is a separate decision.
2. If it grows: make the tool say it, in general terms, where the model writes the unit.
   - The census already records who feeds the keys of a holder (the `param` facts on writer
     sites, the callers chain). When the key of a ranked holder is a parameter of a public
     entry (the caller's object goes in as the key), the target block of the driver file
     (first pass and second look) says so in one line: the key is the object the caller
     passes, the holder grows only when each unit passes an object it has not seen, and, per
     runtime, what makes a fresh one. JVM: `new`, or a lambda that captures a local of the
     unit; a method reference or a lambda that captures nothing is one object per call site
     for the whole run. Node and Python: a function or object literal evaluated inside the
     unit is new each time; a module-level function is one object. Go: a new pointer or a
     value built from the key; a package-level func value is one. Rust: a new value built
     from the key (closures are rarely keys). Do not name any project in the text.
   - Where the census cannot tell that the key is a parameter, do not guess: say in the
     report how often that would apply on the spring evidence ledger.
   - If a second-look slot is the right place (a sixth slot "fresh key object or a reused
     one"), say why before adding one; the give-up limit and the refusal must still make
     sense with it. A line in the block may be enough.
3. `dla trend`'s reason for a `constant` probe of such a holder (key from a caller's
   parameter) adds one sentence: constant can mean the unit passed the same key objects; and
   the remedy the tool prints when such a finding is closed unresolved does not suggest
   class loaders or anything project-specific.

## Tests and checks

- Unit or template tests for the new line per runtime (present when the census fact is
  there, absent otherwise), each seen failing once.
- Real tree: on the spring copy, `dla harness gen --lang jvm --driver --probe <F>` for the
  finding shows the line; a unit written ONLY from what the block says grows the probe;
  `dla trend` reads the target `grew`; follow the printed commands as far as they carry
  (sibling, `ledger test`, `alloc-test`, refute, confirm) and stop at the first command
  that is refused or wrong. Say the numbers (probe size per point, slope).
- fmt; clippy on what you touched; `cargo test -j 2 --no-fail-fast -p dla-core
  -p dla-workload -p dla-cli`; `dla schema --check`; the skill text test and docs where the
  driver block is described; `docs/VERIFY.md` row.

Report: the experiment's result with the outputs, what changed, tests with what was seen
failing, what is unverified, anything wrong you saw outside this brief.
