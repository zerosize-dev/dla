# What DLA cannot see

DLA reports what it measured and what it could not. This page lists the limits you should know
before trusting a report, first the ones that apply to every run, then the blind spots of each
runtime.

**Keeping this page honest.** Each runtime reference shipped with the skill declares its own blind
spots, and DLA copies them into the coverage manifest of every run so the report can show them.
This page must list every one of them. When a runtime reference gains or changes a blind spot, this
page changes in the same commit. A blind spot that appears in a reference but not here is a bug in
the documentation.

## Limits that apply to every run

- **Deployed environments.** DLA works only with local code, local builds and local artifacts. It
  never connects to staging, production or any remote service. A leak that appears only under
  production traffic, data volume, configuration or library versions can be missed or
  mis-sized by a local reproduction. DLA marks such findings as not exercised rather than
  guessing.
- **Secrets and values.** DLA does not read credentials, and by default it shows the model only the
  shape of values it reads from a target (length, character pattern and a salted hash). A leak
  whose diagnosis needs the actual value cannot be resolved with redaction on, and the report says
  so.
- **CPU, latency and throughput.** DLA looks at memory and at resources that hold on to it
  (handles, threads, tasks). A performance problem with no memory signature is out of scope.
- **Workloads it cannot run.** Growth that needs a particular input, a long soak or an external
  dependency cannot be confirmed unless a local workload exercises it. Entry points that could not
  be exercised are listed in the report, and the affected findings stay unresolved or
  suspected from static evidence only.
- **Hooks can be bypassed.** The plugin's hooks and the skill's rules guide the session, but a
  command run through the Bash tool, a different tool, or a session started without the plugin is
  not checked by them. The binary's gates protect the ledger and the report; they cannot stop a
  session from ignoring DLA altogether.
- **Hooks fail open.** A Stop hook that crashes, times out, cannot find the binary or prints
  something unparsable lets the turn end normally. A missing or failed check never blocks the
  session, so an absent check is not evidence that the conclusion was verified.
- **Measurement perturbs memory.** Heap snapshots, profilers, sanitizers and forced collections
  change what they measure: they allocate, slow the target down or alter timing. Growth that
  appears only while an instrument is on is reported as a possible artefact of the measurement
  rather than as a leak.
- **Static analysis is thin where leaks are common.** Detectors find handle and registration
  mistakes well. They find few leaks that come from long-lived containers, stale references or
  captured scope in garbage-collected languages, which is where the dynamic half of DLA matters most.
- **Unclassified is a valid result.** A confirmed leak whose mechanism fits no known pattern is
  reported as a novel mechanism with its holder and the model's description. Known patterns
  are hypotheses to test, never a list DLA must choose from.
- **Heuristics and model judgement.** The explanation attached to a finding is tagged `observed`
  or `inferred`. Inferred statements come from the investigating model and carry its mistakes,
  including a tendency to prefer familiar-looking causes. The blind verifier reduces this; it does
  not remove it, because it shares the session's model.

## JVM (Java, Kotlin, Scala)

- **Off-heap and non-heap memory is not in a heap dump.** Direct buffers, memory-mapped files,
  native allocations, thread stacks, file descriptors and metaspace need separate tools. Growth
  outside the heap is routed to those tools, and where none is available the report says the
  region was not examined.
- **Native allocation stacks may not include Java frames.** Whether native-memory profiling
  attributes an allocation to the Java code that caused it depends on the profiler and mode, and
  is unverified (see the verification backlog).
- **A heap dump is a stop-the-world snapshot of one moment.** It shows what is reachable then, not
  why it accumulated, and taking it can itself change the heap and the process.
- **Static detectors miss helper-method and framework cases.** Resource-leak rules often lose track
  of a closeable once it is wrapped or handed to a helper, and leaks that go through framework
  lifecycle callbacks are rarely visible to them.
- **Coroutine leaks need the debug probes installed.** Kotlin coroutines suspended forever are
  visible only if the coroutine debug agent was active while they were created, and creation
  stacks add overhead. Compiler-generated field names in continuations are not stable across
  Kotlin versions.
- **Class loader and redeploy leaks need a reload loop.** A single run of the application does not
  show that an old class loader stays reachable after a reload.
- **High-cardinality metrics are caught only when the tag value is derived from request data
  visibly in code.** Random suffixes and dynamic names slip through built-in detectors.

## Go

- **The leaked-goroutine profile misses primitives reachable from globals.** A goroutine blocked on
  a channel or lock that a global variable still references is not reported as leaked, because
  the runtime cannot prove nothing will ever wake it. Such goroutines appear only as growth in the
  ordinary goroutine profile.
- **The leaked-goroutine profile depends on the Go version.** It is on by default only in recent
  releases and needs an experiment flag and a rebuild in the previous one. Older toolchains get
  profile diffs instead, which are weaker evidence.
- **The built-in deadlock detector catches few blocking bugs.** It reports only when every
  goroutine is blocked, so a leak in a running server is invisible to it.
- **Heap profiles blame the allocation site, not the holder.** A small subslice that pins a large
  buffer is attributed to the line that allocated the buffer. Naming the root that keeps it alive
  needs a reference-chain tool, and support for the newest Go releases is unverified.
- **Maps and pools keep their peak capacity.** A map that grew and was emptied keeps its memory.
  A flat entry count with high memory is a high-water mark, not necessarily a leak, and heap
  profiles alone do not tell the two apart.
- **No linter covers the commonest goroutine leaks.** A goroutine that waits for a stop that never
  comes is found only by running the code.
- **Memory held by cgo or by the runtime outside the Go heap** is not in a heap profile.

## Rust

- **There is no live, typed retention view.** No tool walks a running Rust heap and prints the
  chain of owners that keeps an object alive. Retention is inferred from allocation-site growth
  and from code reading, and the coverage manifest says so on every Rust run.
- **Leak checkers report only memory that is unreachable from statics.** Miri and LeakSanitizer
  find memory that nothing points to at exit. A leak held by a static collection, an interner,
  a long-lived channel or a registry is still reachable and is invisible to them.
- **Reference-count cycles are confirmed only in tests.** Reproducing a cycle needs a test that
  runs under Miri or a leak sanitizer, and sanitizer support depends on a nightly toolchain.
  A running service cannot be checked this way.
- **Allocation profiling groups by allocation site, not by owner.** Growth can be located but the
  structure that holds it has to be reasoned out.
- **Async tasks that never finish** are visible as a growing task count only if the runtime exposes
  one or the code is instrumented; otherwise they look like ordinary heap growth.
- **Unbounded channels and caches** grow without any unsafe or leaking code, so lints for forgotten
  or leaked values do not flag them. Whether a mem-forget style lint exists for a given pattern is
  unverified.

## Node.js and TypeScript

- **Heap snapshots can double the heap.** Taking one needs memory comparable to the heap itself
  and can push a near-limit process over it. Snapshot-based growth is reported as possibly caused
  by the measurement until a lighter signal agrees.
- **Collection must be forced for comparisons to mean anything.** Without exposing the garbage
  collector, differences between snapshots include garbage that was merely not yet collected.
- **Native memory is outside the heap.** Buffers, array buffers, native addons and memory used by
  the runtime itself grow while the JavaScript heap stays flat. DLA detects the pattern and routes
  it to the native checks, but it does not see inside the addon.
- **Async context stores hide the real holder.** A timer or listener that retains a whole request
  through an async-local store shows the store in the path; the cause is the registration, and the
  path alone does not say so.
- **Browser-side leaks need a browser.** Front-end code is analysed through a headless browser
  scenario that someone must write; DLA cannot invent user interaction that exercises the leak.
- **Library leaks are common and look like application code.** A large share of web-application
  leaks sit in dependencies, and attributing them requires comparing versions against known
  issues, which only covers the issues DLA's data knows.
- **Heap analysis tools lag the runtime.** Snapshot parsers may fail on a newer Node release before
  they are updated.

## Python

- **Cycles that the collector has not run yet look like leaks.** CPython reclaims most reference
  cycles, but some versions let many accumulate before a full collection. Only a forced collection
  separates a pile-up from a cycle that survives, and a short run may not run one.
- **Objects tracked by the collector are the only ones visible.** Memory held by extension modules,
  by native libraries or in memory pools that the interpreter does not return to the system is
  outside what object-graph tools can show.
- **Allocation tracing slows the process and changes timing.** Tracing allocation sites with the
  standard library adds overhead and may hide races or timing-dependent growth.
- **Reference-count errors in C extensions** are visible only as native growth. Python-level tools
  cannot show which extension call leaked.
- **Task and thread leaks in asynchronous code** are visible only if the loop's tasks can be
  enumerated while the leak is happening.
- **Context variables and caches on methods** retain objects through places a plain instance
  census does not look.

## Native code (C, C++, FFI)

- **Leak checkers see only unreachable memory.** A leak held by a global table, a cache or a
  singleton is reachable at exit and is not reported. Growth of reachable memory needs allocation
  profiling and comparison over time.
- **Sanitizer builds need a rebuild.** Leak and address sanitizers require compiling the target
  with them, which may not be possible for third-party binaries, and they change memory layout and
  timing.
- **Allocator behaviour looks like a leak.** Freed memory held by the allocator (arenas,
  fragmentation, delayed return to the system) raises resident size with no live allocation
  growing. DLA checks live allocations against resident size first, but cannot always trigger a
  trim to confirm.
- **Static analyzer leak rules are path-sensitive and give false results in both directions.**
  Their exact rule names and precision are unverified; treat a hit as a lead.
- **GPU and device memory** is a separate region with vendor tools; it is not covered by the host
  tools above.
- **Memory handed across a boundary is easy to lose track of.** References held by a foreign
  runtime (a JVM global reference, a Python object count, a Go cgo handle) are seen only from the
  side that holds them, so a leak that spans both needs both runtimes' tools in one run.
