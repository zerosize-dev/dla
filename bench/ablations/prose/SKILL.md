---
name: detect-leak-anywhere
description: Diagnose memory leaks in any runtime. Use when someone reports growing memory, a rising RSS, an out-of-memory kill or OOMKilled pod, a heap that never shrinks, a slow leak, file descriptor or thread growth, or asks to find, prove or rule out a leak in JVM/Kotlin, Go, Rust, Node.js/TypeScript, Python or native/FFI code. Diagnosis only; it never edits project source.
---

# Detect Leak Anywhere (DLA), prose edition

This is a diagnosis method for memory leaks. It measures, attributes and reports, and it never edits project source. There is no helper binary in this environment: you work with the tools of the runtime itself (a profiler, a heap or type histogram, `/proc`, `ps`, the language's own counters, a small script) and you keep your own notes in a scratch directory outside the source tree.

Work carefully and at high effort. The job is verification, and an explanation that merely sounds plausible is not a result.

## Rules, as practice

1. **Measure before you explain.** Do not name a cause before you have seen growth in a number: RSS, heap size, an object or entry count, a thread or goroutine count, file descriptors. Find the workload first, run it at two or more sizes, and look at the shape of each series (steady slope, step, plateau).
2. **Every growth series gets a non-leak hypothesis next to the leak hypothesis.** The non-leak candidates are allocator retention (memory freed but not returned to the OS), deferred collection (garbage waiting for a collector), a warm-up cache, a pool or buffer that settles at its high-water mark, a workload that scales with its input, retention that is intended, and the measuring instrument itself. For each hypothesis, write down the result that would refute it before you run the test.
3. **A claim needs a site, a holder and a demonstration.** A leak is confirmed only when you have (a) growth that keeps its slope over a long enough run, (b) the `file:line` that retains the memory or allocates it without release, (c) the holder, meaning what keeps it reachable, and (d) a non-leak alternative that your measurements ruled out. If you only read the code and could not run it, the honest verdict is "suspected".
4. **Resemblance is not evidence.** A familiar-looking pattern opens a hypothesis; it never closes one. Test it.
5. **UNCLASSIFIED is a result, not a failure.** When no common pattern fits, or the evidence a pattern needs is missing, state the holder and the mechanism in your own words and call the finding `UNCLASSIFIED`. A finding with a stated holder and mechanism and no label is a success, and it is reported with the same weight as a labelled one. Never force a label onto a leak it does not fit: a forced label is a mistake.
6. **Say what you could not do.** A missing tool, a build that fails or a workload you could not find is a gap to state, never a silent skip. When nothing can be run, report the leak as suspected, with the reason.
7. **Never edit project source.** Scratch copies, scratch scripts and temporary instrumentation outside the source tree are fine.

## Stages

Go through them in order and do not skip ahead.

1. **Intake.** Read the task, then the repository: language and runtime, how to build, how to run, entry points, tests. Nobody can answer questions, so decide for yourself and note your assumptions.
2. **Probe.** Find out what this machine can measure: the runtime version, available profilers, memory counters, whether the project builds. Missing tools become stated gaps.
3. **Static sweep and holder census.** List the holders that live as long as the process: static fields, singletons, module globals, registries and listener tables, collection fields of long-lived objects, thread-local storage, spawned threads or tasks, acquired native handles. For each, read the code that adds to it and the code that removes from it. Every holder with an add path and no matching removal path is a candidate.
4. **Classify growth.** Locate or write a workload that drives the code through many requests or iterations and accepts a count. Run it at several sizes and record the measurements for every series (heap, non-heap, off-heap, threads, descriptors). Then look for growth that no candidate explains yet.
5. **Confirm.** For each candidate, state the expected observation and the observation that would refute it, then run the test: count the holder's entries, take two histograms, compare thread dumps or goroutine counts, rerun with the suspected add path disabled in a scratch copy. Pair every leak hypothesis with its non-leak sibling and test both.
6. **Attribute and explain.** Pin the retaining site and the holder. Write the lifecycle in plain words: what is created, what holds it, what should have released, removed or stopped it and does not. Name the missing pairing, not a workaround such as a weak reference. A second, independent kind of evidence (a retention path, or the growth vanishing when the add path is removed in a scratch copy) makes the finding stronger.
7. **Rule out non-leaks.** For growth that turns out to plateau, drop after collection, follow the workload size or stop after warm-up, record the observation that rules the leak out: the entry count that stops at a fixed number, the slope of the second half of the run, the value that falls back after a forced collection or an allocator trim.
8. **Final check.** Walk the list of candidates and series once more. Every one ends as confirmed, suspected, or ruled out with its observation. Nothing stays open without a stated reason.
9. **Report.** Write the conclusions to `./findings.json` as described below, then summarise them in your answer, with the unfinished parts listed.

If the runtime has reference notes next to this file (`${CLAUDE_SKILL_DIR}/references/`), read the one for your runtime only after stage 4 has produced measurements.

## Writing findings.json

When you are done, write `./findings.json` in the project root:

```
{"findings": [{"site": "path/to/file:line", "holder": "...", "region": "...", "verdict": "...",
  "mechanism": "...", "cause": "...", "evidence": "...", "discriminator": "..."}]}
```

- `site`: project-relative `file:line` where the memory is retained, or allocated and never released.
- `holder`: one of `static/global` (a static field, singleton or module global), `registration-table` (a listener, timer, callback or subscription list), `container-field` (a collection field of a long-lived object), `thread-local` (thread-local or async-context storage), `blocked-stack` (a thread, goroutine, task or coroutine that never finishes), `closure/continuation` (a captured scope), `native-handle/global-ref` (native memory, a handle or a global reference outside the managed heap), `cycle-only` (objects reachable only from each other).
- `region`: `heap`, `off-heap` or `non-heap`.
- `verdict`: `leak-confirmed` (you demonstrated the growth by running or measuring), `leak-suspected` (you saw it in the code but did not demonstrate it), or `not-a-leak` (growth you investigated and ruled out).
- `discriminator`: for `not-a-leak`, the observation that rules the leak out.
- `cause` is optional free text or a label; `evidence` says what you ran or measured.

List every leak you suspect and every growth you ruled out. Do not call anything `leak-confirmed` that you did not demonstrate.
