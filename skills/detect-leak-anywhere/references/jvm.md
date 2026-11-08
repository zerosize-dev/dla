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
| histogram, thread-dump, nmt | 2 | `Thread.dump_to_file -format=json` needs JDK 21; older JVMs fall back to `Thread.print -l` |
| jfr | 2 | recording holds JVM arguments and environment variables: may contain secrets |
| heap-dump | 4 | forces a full GC; size predicted from `GC.heap_info` used (within 0.4% in one test); the target must be able to write under the run directory |

Attach needs the same user and a writable `/tmp` (attach socket, `hsperfdata`). `dla sample --pid
<P> --counters jcmd` is tier 2 and sparse (each call takes a second or more).

## Artifacts
- HPROF (`.hprof`, `.hprof.gz`): `dla summarize heap-graph <dump>...`. Three or more dumps of one
  process give a dump series and the artifact path of `references/stages.md`. Retained sizes are
  approximate (instances sized as header plus fields, compressed references assumed).
- JFR, class histograms and thread dumps are not summarized yet: other formats are M5. The files
  stay under `attach/P<n>/`.

## Blind spots
- Off-heap and non-heap memory (direct buffers, mapped files, thread stacks, metaspace) is not in a
  heap dump.
- A heap dump is one stop-the-world moment: it shows what is reachable, not why it accumulated.
- Kotlin source is read by a line scanner; coroutine and continuation leaks need debug probes.
- A class-loader or redeploy leak needs a reload loop; one run does not show it.
- Native allocation stacks may lack Java frames (unverified).
