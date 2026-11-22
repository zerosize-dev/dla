# JVM (Java, Kotlin, Scala)

Worker name and runtime id: `jvm`. Read this after the router; facts come from `harness/jvm/README.md`
and `docs/VERIFY.md`.

Archetype rows (what to look for, how to confirm, how to rule out): read `references/jvm-archetypes.md` only after `dla gate check --stage classify` passes.

## Harness
- Agent: `dla-agent.jar` is injected through `JAVA_TOOL_OPTIONS` (appended, never replaced) with
  `-XX:NativeMemoryTracking=summary`; the `Picked up JAVA_TOOL_OPTIONS` stderr line is expected.
  Tool JVMs (javac, maven, gradle, ant, sbt) are skipped, so builds may inherit the variable.
- Command mode (default): the workload's own `main` runs once per point; the agent writes one exit
  record after two `System.gc()` calls. Driver mode (`dla harness gen --lang jvm --driver
  [--package <pkg>]`): one process, a `checkpoint` per round, JFR enabled; fill each `DLA-HOOK`
  and derive ids from `key`. `--mode both` gives a fresh and a repeated-keys series.
  On a Gradle or Maven project the driver's class path is resolved by the build, as for `--junit`
  (the project's main classes of every language, resources and dependency jars; offline first, then
  online): `dla` resolves the class path and the main class, so never run `gradle`, `mvn` or
  `java -cp` to print or assemble one (the Gradle test task's own `jvmArgs` and `systemProperty` are
  not applied, see below). If the build fails, read the refusal and the build-log pointer it
  prints and run the command it names; when that does not fix it, record the item `unresolved` with reason
  `build_fails` (or `tool_missing`) with the last stderr lines in `--remedy`. A project built by `build.sh`,
  or whose build tool is not installed, keeps the workload's classes directory and the manifest says so.
- JUnit repeat (`dla harness gen --lang jvm --junit [--tests pkg.Class[#method],..]`): for a Gradle or
  Maven project with JUnit 5 tests, a driver with no hooks runs the selected tests again and again in
  one JVM (one unit = one run of the selection; defaults warm-up 1, rounds 8, step 2, timeout 1800 s),
  so a service whose tests take no iteration count is still measured. Run it with `dla harness run H<n>`;
  the first build resolves the test class path through Gradle or Maven (offline first, then online).
  JUnit 4 and TestNG tests are refused. A mocking library that instruments classes attaches itself to the
  running JVM unless it is loaded at start-up, which a hardened container or sandbox refuses (`Could not
  self-attach to current VM`, every unit fails): when the test class path holds `mockito-core` with a
  `Premain-Class` entry (version 5 and newer), else `byte-buddy-agent`, the build step adds it as a
  `-javaagent` of the loop's JVM next to DLA's own agent; the run output says so (`startup_agent`) and
  the manifest lists it in `tools` (`startup-agent:<jar>`). The test loop is the first pass of the queue
  on a JVM project with JUnit 5 tests: `dla harness gen --lang jvm --junit --next` takes the next
  ranked candidates that have a test class (by the test class's name holding the owner class's name,
  else by the test using it; at most 16 targets, 10 classes, 1000 tests and one module per harness, so
  a run is about two minutes; the 16 because each target keeps 3 probe columns inside the 64 of a
  series), prints the tests chosen and why (`tests_chosen`, `tests.json`), and declares them with their
  probes; nothing to fill. `dla census --lang jvm` writes the workload discovery result itself when
  the run has none, so the test loop is offered without a separate `dla workload discover`.
  Candidates with no test class wait for `--driver --next`. A class none of
  whose tests passes in the first run is dropped while another passed, and the loop lowers its step and
  drops the slowest classes when the runs would not fit the time limit (600 s for a batch); both are
  limitations of the series, `dla harness run` names them first (`TEST CLASS DROPPED`), and a flat
  reading of a target whose chosen classes were all dropped reads `not_executed`, a miss. A run in
  which some tests fail and others pass is a measurement of the ones that passed (the first failure
  is a limitation). If the batch cannot build or no test passes in any run, the output prints the
  driver command for the same targets and they are not counted as measured. Pick the test classes that exercise the
  target by name and by import (`--tests`), with `--probe F` for the finding. When the project's own tests start the application
  (`@SpringBootTest`), use `--junit` with such an integration test: it is one JVM and the Spring test
  context cache keeps the context across repetitions, so growth comes from the code under test;
  `@DirtiesContext` rebuilds the context every repetition. If the services the test needs (database,
  broker, cache) are not running, the first unit fails and the last stderr lines of the log
  (`DLA-JUNIT every unit failed: ...`) name the first failure and its root cause (a refused
  connection shows the exception, not always the host): look in the log for the address, then record
  `env_unavailable` with it, not `no_harness`. A run where the first unit passes and later units fail
  keeps its series (`failed_units=n` in the `DLA-JUNIT summary:` line): a test that asserts a fresh
  state (a count of 1) fails once state has built up, and an assertion on accumulated state is the
  leak, not a missing service; only a connection or startup error is `env_unavailable`. One test
  method that always fails (it needs a broker) fails every unit and there is no exclude option:
  list the passing methods with `--tests`. `--mode both` (or `repeated`) on a JUnit harness runs only
  its one mode (`fresh`) and records a limitation in `run-S<n>.json`, so it does not run the loop
  twice. If no test passed or failed, the run exits 3 and no series is written: read the last
  `DLA-JUNIT` lines. Tests aborted by an assumption (`assumeTrue(env var set)` for a service that
  is not running, also in `@BeforeAll`) show as `aborted=n`, `@Disabled` or conditionally disabled
  ones as `skipped=n`, and a class that fails before its tests (`@BeforeAll` throwing
  `ConnectException`, Testcontainers without Docker) is named by the cause line and `first_failure=`.
  Record `env_unavailable` for the service cases: the code under test never ran. A Kotlin
  backtick test is selected with its spaces: `--tests 'pkg.GetTest#publishes to the broker'`.
  The run measures `java` from `PATH`, not the Gradle `test` task's JVM: `jvmArgs`, `systemProperty`
  and `environment` of the task are not applied (set the profile or property the test needs before
  `dla harness run`). Select a `@ParameterizedTest` as `pkg.Class#method(java.lang.String,int)`.
- What the dump measures (after GC): `jvm.heap_used`, `jvm.heap_committed`, `jvm.nonheap_used`
  (the memory pools: Metaspace, class space, code cache; not the native categories),
  `jvm.nmt_committed`, `jvm.threads`, `jvm.classes_loaded` (loaded now), `jvm.classes_unloaded`
  (so far), `jvm.jit_seconds` (compile time so far, when the JVM reports it), the NMT categories
  `jvm.nmt.metaspace|class|code|compiler|symbol|arena_chunk|thread` (committed bytes; NMT is on in
  harness runs, and without it `jvm.metaspace_used` and `jvm.codecache_used` stand in), the
  `os.*` set, top-20 class counts,
  and one probe record per static reference field (`JvmClass`, `JvmStatic`): size, reachable
  objects (bounded walk, 2,000,000) and a shallow byte estimate assuming compressed oops. A
  `ClassLoader`-valued static is probed differently: a `.classes` count (the loaded classes it
  defined, through the agent's Instrumentation) plus the walk of its own fields (those of the
  project's subclass; the JDK's loader internals are not walked) into the reachable and bytes figures.
- A `jvm.class:<name>.count` column that ties to no finding (a generic type never ties) is accounted
  for by `dla gap <S> --residual`: rows under their threshold, or over it with a resolved open item.
- Entry points exercised: the agent lists the loaded classes (the project's first, never cut by
  the 4000-class cap) with the source file of each project class, and a discovered entry point
  counts when its file's class is loaded: by path (Java layout), by the class file's `SourceFile`
  (a Kotlin file holding a class of another name, or a package that does not mirror its
  directory) or, for a top-level function, when the file's facade class (`UtilKt`, or its
  `@file:JvmName`) is loaded. A driver and a JUnit repeat write the list in their first, second,
  fourth... and last checkpoint, so a repeat cut off by its timeout or the run deadline still
  reports the classes it loaded. A loaded class is not a called method: the count says which files
  ran, not which
  functions. `Coverage: jvm 750 iterations, 0 of 225 entry points exercised, 37 project classes
  loaded` means project code ran but no listed entry point sits in a loaded class: the harness
  drove other classes, so it measures those, not the entry points named (entry points in
  classes no harness reaches stay unexercised). That run is not "nothing
  ran"; say what the harness did drive.
- Non-heap growth (a rising `jvm.nonheap_used`, usually the non-heap residual item of
  `dla gap <S> --residual`) is decided, not guessed or closed for time. Route: `dla ledger add
  --origin residual --region non-heap --evidence E..`, an `N-WARM` sibling ("JIT profiling data and
  compiled code level off"), `dla ledger test F --evidence E..`, `dla alloc-test H1 --finding F`,
  then refute F and confirm the sibling with the commands it prints, and `dla ledger resolve O
  --by F.n1`; when the alloc-test favours leak instead (a real Metaspace or class leak), confirm F
  with the commands it prints and resolve the item `--by F`. The test judges `jvm.nonheap_used` and
  every `jvm.nmt.*` category that passed the gate: all must plateau over the extra window for
  `non-leak`; `jvm.classes_loaded`, `jvm.classes_unloaded` and `jvm.jit_seconds` are reported next
  to it and do not vote. Reading it: JIT warm-up is a plateau of non-heap and of Code and Compiler
  with a flat class count and
  compile time levelling off. Symbol growth, or Metaspace growth with rising unloads or a rising
  class count, is not JIT: capture `--capture nonheap` and read `nmt-diff.txt` for Symbol and
  `classloader-stats.txt` for Metaspace. A driver or JUnit run's own checkpoint work (class
  histogram, NMT call, probe walks) is part of the figure and decides small cases: an idle driver
  (no workload) rose 2.3 MB over 150 checkpoints, and after the agent's warm-up (`DLA_WARM_ROUNDS`)
  still drifts about 10 to 20 KB per checkpoint, so on a JVM with a non-heap of about 10 MB an idle
  workload reads `leak` on `jvm.nonheap_used` every time (7 of 7 runs at the default extra), and a
  longer `--extra` only adds drift. So a small JVM cannot decide: a leak reading of
  `jvm.nonheap_used` with a class count that is not rising (one that reads inconclusive with a net
  rise of more than 8 classes is rising), an extra rise under the drift line and no required column
  rising is reported as inconclusive (the result carries a `caution`, and its `next` prints no
  refute or confirm). The drift line is the larger of 1 MiB and 2x the measured drift (20 KB, the
  highest measured; so 1 MiB up to extra 25) of every extra checkpoint, so it grows with
  `--extra`; a rise over it, or a
  harness that does more work per checkpoint (a larger `--step` at `harness gen`, also with
  `--junit` for a JUnit repeat), decides. A command-mode harness (a fresh JVM per point) has no
  checkpoint drift and gets no such override. A required column rises by its kind (the `jvm.nmt.*`
  categories and thread counts are required when they passed the gate in the latest trend, and the
  thread counts also when the harness has no trend yet). `jvm.threads` rises when its plateau
  reads continues or its net rise over the extra window is more than 2 threads, whatever its
  plateau reads (threads added in bursts near the end of a window read plateau; the JVM starts and
  stops compiler threads by itself, and drivers that start no thread, in 59 runs many of them under
  load, rose 0 on `jvm.threads`; the slow leaks seen rose 6 and 10). `os.threads` rises only when
  its plateau reads continues: HotSpot's own GC worker threads moved it by 4 on a driver that
  starts no thread (the JVM told 32 processors), so a net rise of a few threads there is no
  growth. When `os.threads` rises alone, with `jvm.threads` and the NMT thread category not
  rising, the limitation names it and `next` gives the thread capture, but the result is
  inconclusive.
  A `jvm.nmt.*` byte category rises only when its plateau reads continues, as bytes jitter, and
  `jvm.nmt.thread` also rises with a thread count that itself rises (not with one that is not a
  required column or whose net rise is within the jitter). Every rising column is collected first,
  and the class count is not flat when its plateau reads continues or reads inconclusive with a net
  rise of more than 8 classes (the JVM loads a few classes by itself: idle drivers rose 0 to 2 when
  fast and 4 to 6 when the unit sleeps and the run is slow; a class leak that adds 8 or fewer over
  the extra window cannot be told from that by the count, so unless the count reads continues it
  reads inconclusive, never a leak, and when the count reads continues it is a rising column
  whatever the net rise). When a thread count or an NMT category rises, that is not the agent's
  drift and is never blamed on it: the limitation names every column that rises (the class count too
  when it is not flat), a rising `jvm.threads` keeps the leak verdict, a rising NMT category makes
  it inconclusive (unless the class count is not flat too: a class leak keeps its verdict, and the
  ledger commands stay beside the capture), and `next` points at the capture that decides it (`dla
  harness run H1 --capture nonheap`: `threads.json` for threads, `nmt-diff.txt` for the category),
  never at a longer `--extra`, and keeps the thread capture whenever a thread count rises; on a
  partial run, whose verdict is capped at inconclusive, `next` holds only that capture, with no
  refute, confirm or longer `--extra`. Only when no required column rises and the class count is not
  flat is the class count named alone (the leak
  verdict stands, except on a partial run, which is inconclusive). A result with no
  `jvm.classes_loaded` column is read with the class count unknown: if nothing rises it gets the
  agent-drift reading with a caution that the class count was not recorded or could not be judged.
  Do not call a small slope "the service's JIT" without the plateau test. When a required
  `jvm.nmt.*` category or thread count rises (for `jvm.threads` also a plateau with a net rise of
  more than 2 threads) while the pools plateau, the pools' plateau is held back as inconclusive
  and the limitation names the column. A sample (`dla sample --counters jcmd`) shows the shape live but
  cannot refute: `jvm.classes_loaded` there is the classes loaded now (the cumulative counter minus
  the unloads), as in the harness.
  The jcmd backend reads `GC.heap_info` of JDK 21 and of JDK 25: `jvm.heap_committed` is the heap's
  committed size in both (`total` up to JDK 21, `committed` on JDK 25), and where JDK 25 prints no
  Metaspace line `jvm.nonheap_used` comes from `VM.metaspace basic`.
- Purges: `trim-native-heap` (DiagnosticCommand) and `gc`.
- The harness never attaches: it reads NMT and the histogram in-process.

## Attach backends (`dla attach jcmd --pid <P> --op <op>`)
| Op | Tier | Notes |
|---|---|---|
| histogram, thread-dump, nmt | 2 | `Thread.dump_to_file -format=json` is verified on JDK 21 and JDK 25 (19, 20 and 22 to 24 untested); older JVMs fall back to `Thread.print -l` |
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
| HPROF (`.hprof`, `.hprof.gz`) | `heap-graph <dump>...` | three or more dumps of one process, or of one harness in command mode (per-point series): a retention path and a `holder:` column that ties by holder identity; `via_holders` name a static or field that keeps a set alive when the dominator is the super root or a thread (coroutines, shared sets) |
| HPROF or `GC.class_histogram` text | `type-histogram-diff` | growth by class. Ties by type only to the accumulation class recorded for a holder, the declared element type of a finding's field, or the field's own plain declared type when it is a project class (a type declared in the sources); an instance field whose type is its own class (a chain link such as `previous`) is not a holder of that type, the static root is; if several findings declare that type, it ties none. Generic types (`Object[]`, `String`, `java.util.*`, `kotlin.*` collections) never tie |
| JFR `.jfr` (tool `jfr`) or `jfr print` text | `alloc-stack-diff` | a sample of old objects with the root path: a `holder:static:<Class>.<field>` total ties by holder |
| `threads.json` | `non-heap` | thread count and groups; no creation site, so groups never tie |
| `classloader-stats.txt`, `nmt-diff.txt` | `non-heap` | loader, class and native category growth; only a loader's own group ties, by exact class to a finding whose holder is that loader class; the rest never tie. One command per file kind |

- Retained sizes are approximate (instances sized as header plus fields, compressed references assumed).
- JFR is a sample, not a census. At the default step the old-object counts are small (about 20 samples)
  and the gate often fails; a tie needs a leak large enough for a strictly rising sample count (a larger
  `--step` helped). Samples with no static holder (thread or `root:<system>` roots) never tie.
  The recording needs `settings=profile` with `path-to-gc-roots=true` (the harness does this).
- Command-mode heap dumps are separate processes. When 3 or more of them are captures of one harness
  (`harness/H<n>/captures.json`: kind heap, distinct x, one label), `heap-graph` forms a per-point series
  over n (`series_kind: per-point`): it skips the one-process checks, keeps the HPROF format checks, and
  uses the usual strict-growth gate and `dump-plateau` discriminator. The harness prints the command. Other
  dumps of different processes are refused (`dumps-not-comparable`). Driver mode
  (`dla harness gen --lang jvm --driver`) still gives one process. On copies of `jvm-audit-chain` the holder
  `gateway.RequestContext.latest` tied to the census finding by holder.
- JNI: when the project loads a native library through JNI, `dla harness gen --lang jvm` preloads the
  allocator shim and the series gains `native.live_bytes@<file:line>` columns. In this worker those
  columns tie to nothing (the native holder's finding lives in the rust or native worker, and ties never
  cross runtimes). After `dla harness run H1`, run `dla correlate H1` in this worker: when a native site
  under a JNI export grows at a rate that matches the managed footprint (its slope bounds the site's slope even if the footprint column misses the trend gate; the evidence records `managed.gated`), it adds a candidate here (origin
  `correlation`, site the native site, holder `native-handle/global-ref`, region off-heap, owner the
  managed method, writer its declaration) and the shim column ties to it by site. Then test, refute the
  sibling and confirm as usual. No match writes no candidate; a refusal names the missing boundary or
  series.
- Kotlin: a coroutine's captured locals are fields `L$0`, `L$1` of the compiled continuation (class-file
  census with `DebugMetadata`). The ledger keeps one finding per suspension point, so two captures at
  one line become one finding.
- A thread dump from `Thread.dump_to_file` is verified on JDK 21 and JDK 25 (19, 20 and 22 to 24 untested; otherwise `Thread.print -l`, no JSON).

## Static scope, CodeQL and Sonar
- Build logic is not part of the measured process: `*.gradle.kts`, `*.gradle` and everything under
  `buildSrc/` are never candidates or entry points (census, sweep and workload discovery skip them
  and detekt, Error Prone and SpotBugs exclude them); the Scope line counts them as `build logic not
  read`. A holder in `docker.gradle.kts` is not a finding. `*.main.kts` scripts are sources.
- CodeQL extracts Kotlin only from a traced build. With Kotlin in scope and tier 1, the sweep runs
  the project's Gradle (`classes`, offline, no daemon) or Maven (`compile`, offline) build in a copy
  under the run directory; without tier 1 or a build tool, a Java-only database is built and the
  limitation names the Kotlin files not read, and a Kotlin-only project's `jvm/codeql` row is
  `blocked` with that reason. When the traced build fails on a project that also has Java, the
  Java is still analysed without compiling and the limitation says that the Kotlin files were not
  extracted and why; when the log shows a cold dependency cache it adds to run the project's build
  online once (or a harness run, whose build step retries online) and sweep again. Not run against
  a real CodeQL bundle here.
- Sonar is server-only: the `jvm/sonar` row is `blocked` with a fixed reason that names the offline
  backends that ran, and has no install hint. It is not a gap to close.

## Blind spots
- Off-heap and non-heap memory (direct buffers, mapped files, thread stacks, metaspace) is not in a
  heap dump.
- A heap dump is one stop-the-world moment: it shows what is reachable, not why it accumulated.
- Kotlin source is read by a line scanner; coroutine and continuation leaks need debug probes.
- A class-loader or redeploy leak needs a reload loop; one run does not show it.
- Native allocation stacks may lack Java frames (unverified).
