# Safety and effects

What DLA may do, what each command needs, and how it cleans up. Read this before asking the
consent question and before running a command that starts or touches a process.

## DLA never edits project source

Everything DLA builds or runs happens on a copy. `dla harness gen` copies the project (tracked
files in a git repository, otherwise all files except build and dependency directories) to
`.dla/<run>/harness/H<n>/work/`, the scratch worktree. Builds, agents, allocator shims, generated
drivers and any intervention edit live there or elsewhere under `.dla/<run>/harness/H<n>/`. The
project directory itself is only read. `dla` also appends `.dla/` to `.gitignore` the first time.

## Privilege tiers

One consent per run covers a set of tiers (`dla consent --tiers 1-4`, or fewer). The tiers
escalate by intrusiveness: each one adds a kind of access the one before it does not have.

| Tier | Covers |
|---|---|
| 1 | Build and run project code in a scratch copy under the run directory: builds, tests, harness loops, preload agents, allocator shims and in-process probes in processes DLA starts, intervention edits in the scratch copy, installing analyzers into the plugin data directory |
| 2 | Observe a running local process DLA did not start: `/proc` or libproc sampling of a PID, jcmd attach (class histogram, thread dump, native memory summary, JFR recordings, which hold JVM arguments and environment variables), Node inspector (SIGUSR1) memory figures, Python remote exec, reading a Go pprof endpoint |
| 3 | Debugger or kernel level: ptrace attach (gdb, heaptrack `--pid`, goref), memray attach (gdb injection below Python 3.14, `sys.remote_exec` on 3.14; it records allocation stacks, not contents), eBPF/bpftrace, perf, valgrind with vgdb, another process's fd table, GPU counters; may need root |
| 4 | Capture memory contents: heap dumps (jcmd `GC.heap_dump`), heap snapshots (the Node inspector; peak extra memory about 12x the heap in use, and the target is blocked for seconds), core dumps. Outputs may contain secrets and stay in `.dla/` |

Raw values in summaries are a separate consent (`--raw-values`). Without it the model sees only
the shape of a value (length, character pattern, salted hash).

Ask once, as one question for the whole run. A second consent in the same run is refused. When a
command needs a tier the consent did not include, it is refused and the refusal names the tier.
If no consent exists the fix is `dla consent --tiers 1-4`; if consent was already given, the fix
is a new run (`dla run init`, then `dla consent --tiers <given tiers plus the needed one>`).
Never work around a refusal by running the tool by hand.

## Which command needs which tier

| Command | Tier | Notes |
|---|---|---|
| `dla run init`, `consent`, `workload discover`, `census`, `sweep`, `trend`, `gap`, `ledger ...`, `report` | none | Read project files and the run directory; `sweep` runs the installed analyzers on the project, except the Rust RAPx backend (`cargo check` runs build scripts), which needs tier 1 |
| `dla harness gen` | none | Writes only under the run directory (the scratch copy, the manifest, the agent files, the generated driver) |
| `dla probe --build` | 1 | Runs each runtime's build |
| `dla install <backend>` | 1 | Installs into the plugin data directory or through the project's package manager, never into project source |
| `dla harness run <H>` | 1 | Builds and runs the workload in the scratch copy with an agent, shim or driver injected |
| `dla alloc-test <H>` | 1 | Re-runs the harness with extra points and a purge |
| `dla sample --pid <P>` | 1 for a process DLA started (it is in the run's process registry), 2 for any other process | The OS figures need only read access to `/proc` or the platform process API |
| `dla sample` with `node-inspect` on a process DLA did not start | 2 | If the process has no inspector DLA sends it `SIGUSR1`, which opens one on port 9229 and stays open until the process exits. DLA signals only an executable named `node` or `nodejs`, and only when `--counters node-inspect` was asked for |
| `dla sample` with `python-remote` | 2 | The target runs a small generated script through `sys.remote_exec`; needs Python 3.14 or later in the target |
| `dla sample` with `jcmd` or `go-pprof=<url>` | 2 | jcmd attaches to the JVM (it needs a writable `/tmp` for the attach socket); the Go backend sends HTTP GETs to the pprof endpoint |
| `dla attach` with `memray`, `heaptrack` or `goref` | 3 | ptrace-level attach; allocation stacks, not contents. heaptrack `--pid` is refused while `kernel.yama.ptrace_scope` is above 0. Other debugger and kernel tracing (eBPF, perf, valgrind) is not in `dla` yet |
| `dla attach jcmd` with `histogram`, `thread-dump`, `nmt` or `jfr`; `dla attach node-inspector --op memory` | 2 | JFR recordings may hold JVM arguments and environment variables |
| `dla attach jcmd --op heap-dump`; `dla attach node-inspector --op heap-snapshot` | 4 | Memory contents; may hold secrets and stay in `.dla/`. A Node snapshot needs about 12x the heap in use and blocks the target for seconds |
| `dla summarize <op>` | none | Reads captured files (HPROF, histograms, JFR, valgrind, heaptrack and the other formats of `references/backends.md`); a raw `heaptrack.*` recording runs `heaptrack_print`, and a `.jfr` runs `jfr print`, both read-only |

`dla consent --ceiling N` (the plugin's privilege ceiling setting) caps which tiers may be
granted; offer only tiers up to the ceiling.

## The effects statement

Before a command touches a process it did not build, it states what it is about to do. `dla
sample --pid` prints it to stderr and puts it in its JSON `effects` field:

| Field | Meaning |
|---|---|
| `target` | The process (pid and command) and the tier in force |
| `expected_signal` | The columns the run expects to produce |
| `stopping` | The stopping condition: the process exits, or the duration passes, and never past the run budget |
| `effects` | What DLA does to the target (read-only observation; a signal; a script the target runs; an attach) |
| `cleanup` | What is left behind (nothing, or an inspector that stays open until exit) |

`dla attach` prints the same statement as a preflight, plus free disk, available memory and the expected dump size and peak extra memory; it refuses with `insufficient-resources` (exit 2) when the dump times 1.2 exceeds free disk or the peak exceeds 0.8 of available memory. Read it before relying on the sample. If `effects` names something the user did not consent to,
stop and record the item `unresolved` with reason `privilege_declined`.

`dla harness run` and `dla alloc-test` start only processes DLA builds in the scratch copy. Their
effects are in the manifest (`harness/H<n>/harness.json`: the build and run commands, the
environment, the agent, the timeout) and in the `workload` evidence; read the manifest before the
first run of a harness you did not just generate.

## Processes DLA starts

Every process a harness run starts is recorded in `state.json` under `processes` (pid, start time,
kind, command, start and stop times) and marked stopped when it exits. The build step has a 600
second limit and each measured process a 300 second limit by default (`timeout_secs` in the
manifest); on a timeout DLA kills the whole process group. Their output goes to
`harness/H<n>/logs/`. The preload agents and the allocator shim are injected through environment
variables of the measured process only (`JAVA_TOOL_OPTIONS`, `PYTHONPATH`, `NODE_OPTIONS`,
`LD_PRELOAD`), never of the build.

The allocator shim slows allocation-heavy programs a lot (tens of times in a worst case with many
threads). If a run times out for that reason, generate the harness again with a smaller `--step`;
the timeout is not a `harness gen` option.

## Cleanup

- A process DLA started is not left running: it ends with its command or by the timeout kill.
- `dla sample --pid` leaves the sampled process running. The one lasting side effect is an inspector
  opened by `SIGUSR1`, which closes when that process exits.
- Scratch copies and logs stay under `.dla/<run>/harness/` until the user deletes the run directory.
  They can be large; tell the user which run directory to remove when the investigation is over.
- `dla run abort` stops the processes DLA started (after checking pid and start time), runs the cleanup actions of attaches and marks the run aborted; it never kills a target DLA did not start and does not delete anything.
