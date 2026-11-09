# JVM (Java, Kotlin, Scala)

Worker name and runtime id: `jvm`. Read this after the router; facts come from `harness/jvm/README.md`
and `docs/VERIFY.md`.

## Harness
- Agent: `dla-agent.jar` is injected through `JAVA_TOOL_OPTIONS` (appended, never replaced) with
  `-XX:NativeMemoryTracking=summary`; the `Picked up JAVA_TOOL_OPTIONS` stderr line is expected.
  Tool JVMs (javac, maven, gradle, ant, sbt) are skipped, so builds may inherit the variable.
- Command mode (default): the workload's own `main` runs once per point; the agent writes one exit
  record after two `System.gc()` calls. Driver mode (`dla harness gen --lang jvm --driver
  [--package <pkg>]`): one process, a `checkpoint` per round, JFR enabled; fill each `DLA-HOOK`
  and derive ids from `key`. `--mode both` gives a fresh and a repeated-keys series.
- What the dump measures (after GC): `jvm.heap_used`, `jvm.heap_committed`, `jvm.nonheap_used`,
  `jvm.nmt_committed`, `jvm.threads`, `jvm.classes_loaded`, the `os.*` set, top-20 class counts,
  and one probe record per static reference field (`JvmClass`, `JvmStatic`): size, reachable
  objects (bounded walk, 2,000,000) and a shallow byte estimate assuming compressed oops.
- Purges: `trim-native-heap` (DiagnosticCommand) and `gc`.
- The harness never attaches: it reads NMT and the histogram in-process.

## Attach backends (`dla attach jcmd --pid <P> --op <op>`)
| Op | Tier | Notes |
|---|---|---|
| histogram, thread-dump, nmt | 2 | `Thread.dump_to_file -format=json` is verified on JDK 21 (19 and 20 untested); older JVMs fall back to `Thread.print -l` |
| jfr | 2 | recording holds JVM arguments and environment variables: may contain secrets |
| heap-dump | 4 | forces a full GC; size predicted from `GC.heap_info` used (within 0.4% in one test); the target must be able to write under the run directory |

Attach needs the same user and a writable `/tmp` (attach socket, `hsperfdata`). `dla sample --pid
<P> --counters jcmd` is tier 2 and sparse (each call takes a second or more).

## Artifacts and summarize ops
Read each capture with `dla summarize` (all adapters and rows: `references/backends.md`). In a repository
run, `dla harness run H1 --capture all` writes the files and prints the commands; on a process or
artifact run the files come from `dla attach jcmd` or the user.

| Capture | Command | What it proves |
|---|---|---|
| HPROF (`.hprof`, `.hprof.gz`) | `heap-graph <dump>...` | three or more dumps of one process: a retention path and a `holder:` column that ties by holder identity; `via_holders` name a static or field that keeps a set alive when the dominator is the super root or a thread (coroutines, shared sets) |
| HPROF or `GC.class_histogram` text | `type-histogram-diff` | growth by class. Ties by type only to the accumulation class recorded for a holder, the declared element type of a finding's field, or the field's own plain declared type when it is a project class (a type declared in the sources); an instance field whose type is its own class (a chain link such as `previous`) is not a holder of that type, the static root is; if several findings declare that type, it ties none. Generic types (`Object[]`, `String`, `java.util.*`, `kotlin.*` collections) never tie |
| JFR `.jfr` (tool `jfr`) or `jfr print` text | `alloc-stack-diff` | a sample of old objects with the root path: a `holder:static:<Class>.<field>` total ties by holder |
| `threads.json` | `non-heap` | thread count and groups; no creation site, so groups never tie |
| `classloader-stats.txt`, `nmt-diff.txt` | `non-heap` | loader, class and native category growth; never tie. One command per file kind |

- Retained sizes are approximate (instances sized as header plus fields, compressed references assumed).
- JFR is a sample, not a census. At the default step the old-object counts are small (about 20 samples)
  and the gate often fails; a tie needs a leak large enough for a strictly rising sample count (a larger
  `--step` helped). Samples with no static holder (thread or `root:<system>` roots) never tie.
  The recording needs `settings=profile` with `path-to-gc-roots=true` (the harness does this).
- Command-mode heap dumps are separate processes: `heap-graph` refuses them (`dumps-not-comparable`)
  and the harness prints no `heap-graph` command for them. A `holder:` column needs a driver
  (`dla harness gen --lang jvm --driver`).
- Kotlin: a coroutine's captured locals are fields `L$0`, `L$1` of the compiled continuation (class-file
  census with `DebugMetadata`). The ledger keeps one finding per suspension point, so two captures at
  one line become one finding.
- A thread dump from `Thread.dump_to_file` is verified on JDK 21 (19 and 20 untested; otherwise `Thread.print -l`, no JSON).

## Blind spots
- Off-heap and non-heap memory (direct buffers, mapped files, thread stacks, metaspace) is not in a
  heap dump.
- A heap dump is one stop-the-world moment: it shows what is reachable, not why it accumulated.
- Kotlin source is read by a line scanner; coroutine and continuation leaks need debug probes.
- A class-loader or redeploy leak needs a reload loop; one run does not show it.
- Native allocation stacks may lack Java frames (unverified).
