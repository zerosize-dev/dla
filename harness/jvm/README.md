# JVM agent and driver

| File | Role |
|---|---|
| `DlaAgent.java` | `premain` agent (package `dla.harness`): registers the exit dump and optional ticks |
| `DlaDump.java` | dump, probe and walk code plus a tiny JSON reader/writer (package `dla.harness`); shared by the agent and the driver |
| `DlaCapture.java` | raw captures for `dla summarize` (package `dla.harness`): heap dumps, class histograms, flight recording, non-heap reports |
| `DlaHarness.java` | loop driver template (package `@@DLA_PACKAGE@@`, class `@@DLA_CLASS@@`) |
| `MANIFEST.MF` | jar manifest (`Premain-Class: dla.harness.DlaAgent`) |

## Build (done by dla at gen/run time, JDK 11+)

```
javac -d agent/classes DlaAgent.java DlaDump.java DlaCapture.java
jar --create --file dla-agent.jar --manifest MANIFEST.MF -C agent/classes .
```

## Inject

Append to any existing value (never replace it; the `Picked up JAVA_TOOL_OPTIONS` stderr line is
expected):

```
JAVA_TOOL_OPTIONS="$JAVA_TOOL_OPTIONS -javaagent:<abs>/dla-agent.jar -XX:NativeMemoryTracking=summary"
```

plus the environment contract of `harness/README.md`. Tool JVMs (javac, jar, jshell, maven,
gradle, ant, sbt, detected from `sun.java.command`) are skipped, so build steps may inherit the
variable. Driver mode adds `-XX:StartFlightRecording=filename=<harness>/out/<pid>.jfr,settings=profile`.

## What the exit dump records

- `values` after `System.gc()` twice: `jvm.heap_used`, `jvm.heap_committed`, `jvm.nonheap_used`,
  `jvm.nonheap_committed`, `jvm.threads`, `jvm.classes_loaded`, `jvm.nmt_committed` (only when NMT
  is on, otherwise a limitation), and `os.rss`, `os.rss_anon`, `os.rss_file`, `os.rss_shmem`,
  `os.swap`, `os.vm_data`, `os.vm_hwm`, `os.threads` read from `/proc/self/status`.
- `values_pre_gc`: `jvm.heap_used`, `jvm.heap_committed`, `os.rss` just before the collections.
- `types`: top 20 classes by instance count from `gcClassHistogram` (array names as `byte[]`).
- `coverage`: `{"kind":"classes","items":[...]}`; classes loaded from `DLA_PROJECT` first, then
  other non-JDK classes of the application and library loaders; capped at 4000.
- `probes`: `JvmClass` yields one record per static field of reference type (primitives, strings,
  numbers, enums and `Class` constants are skipped); `JvmStatic` yields one record. `size` is the
  size of a Collection/Map/array, `reachable` the objects reached by a bounded iterative walk
  (max 2,000,000), `bytes` a shallow estimate assuming compressed oops. The walk descends into
  arrays, Collection/Map contents (through their API, so JDK-internal nodes are not counted) and
  reference fields of application classes; it counts but does not descend into enums, strings,
  boxed numbers and references, and skips `Class`, `ClassLoader`, `Thread`, reflection, `Module`
  and `java.lang.invoke` objects. A class that is not loaded gives `error: "class not loaded: ..."`.
- `purge`: `trim-native-heap` runs the `systemTrimNativeHeap` diagnostic command and records the
  `os.*` values before and after; `gc` records the values before the forced collection and after.

## Raw captures (`DLA_CAPTURE`)

`dla harness run <H> --capture heap,histogram,jfr,nonheap` sets `DLA_CAPTURE` and `DLA_CAPTURE_DIR`
(`out/<label>/capture`). `DlaCapture` writes into `<dir>/<pid>/` (the directories of other
processes, such as the unit-test JVM a build script starts, are deleted after the run):

| File | Written | Source |
|---|---|---|
| `histogram-<n>.txt` | every checkpoint (driver) or at exit (command mode) | `DiagnosticCommand` `gcClassHistogram`, the text `jcmd GC.class_histogram` prints (live objects only: it forces a collection) |
| `heap-<n>.hprof` | the selected checkpoints (driver: `DLA_CAPTURE_DUMPS`, default 4, from the end of the warm-up to the last round) or at exit (command mode, for the points `dla harness run` selects) | `HotSpotDiagnosticMXBean.dumpHeap(file, true)` (live objects, mode 0600) |
| `recording.jfr` | at JVM exit | a recording started in `premain` before the workload (`settings=profile`, `path-to-gc-roots=true`, `dumponexit=true`): `jdk.OldObjectSample` only covers objects allocated while it runs |
| `threads.json` | last checkpoint or exit | `Thread.dump_to_file -format=json` (`threadDumpToFile`); needs JDK 19+, older JDKs record an error and write no file |
| `classloader-stats.txt` | same | `VM.classloader_stats` |
| `nmt-summary.txt`, `nmt-diff.txt` | same | `VM.native_memory summary` and `summary.diff` against a baseline taken at the end of the warm-up (driver) or at `premain` (command mode); needs `-XX:NativeMemoryTracking=summary`, which the injection always adds |

A dump costs about 20 ms and 10 to 40 MB for the fixtures; each capture's size and time are in the
`captures` array of the record. Heap dumps from separate processes (command mode) cannot be
compared as a series by `dla summarize heap-graph` (it refuses them as `dumps-not-comparable`, and
`dla harness run` prints no `heap-graph` command for them); a driver takes all of them in one process.
The printed `dla summarize` commands are one per op and file kind: `type-histogram-diff` on the
histograms, `alloc-stack-diff` on `recording.jfr` (reading it needs the `jfr` tool; the JVM writes
the recording itself), and `non-heap` separately on `threads.json`, `classloader-stats.txt` and
`nmt-diff.txt`. In a Kotlin driver run the harness puts `kotlin-stdlib` on the javac and run class
paths (`$KOTLIN_HOME/lib`, next to `kotlinc`, or the version the build script names in `~/.m2`).

## Driver

`java -cp <scratch classes>:<agent classes or jar> <package>.<class>`; compile it with the agent
classes on the classpath (`javac -cp agent/classes ...`) into the scratch build output. The driver
writes one `checkpoint` record per round via `DlaDump.checkpoint(unitsDone, lastRound)`.
`dla harness gen` writes no JUnit 5 variant yet: the plain driver `main` is used and the manifest
records a gap noting that `junit-platform-console-standalone` was not used.

### Placeholders

| Placeholder | Where | Meaning |
|---|---|---|
| `@@DLA_PACKAGE@@` | `DlaHarness.java` | package of the workload (the `package` line is removed when empty) |
| `@@DLA_CLASS@@` | `DlaHarness.java` | driver class name (file is written as `<class>.java`) |
| `@@DLA_STEP@@` | `DlaHarness.java` | units per round (default for `DLA_STEP`) |
| `@@DLA_WARMUP@@` | `DlaHarness.java` | warm-up rounds (default for `DLA_WARMUP`) |
| `@@DLA_ROUNDS@@` | `DlaHarness.java` | measured rounds (default for `DLA_ROUNDS`) |
| `@@DLA_REPEAT@@` | `DlaHarness.java` | repeated key space size (default for `DLA_REPEAT`) |

### Hooks

| Hook | Signature | Fill with |
|---|---|---|
| `DLA-HOOK setup` | `static void setup() throws Exception` | construct the services/caches under test (package-private API is usable); keep references in static fields |
| `DLA-HOOK unit` | `static void unit(int i, int key) throws Exception` | one unit of the workload for `key`; derive ids and payloads from `key` so a repeated key reproduces an earlier request |
