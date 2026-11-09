# DLA report

Run: `20261109-200906-a53f`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 4 enumerated, 4 dispositioned; confirmed 0, strong 0, refuted 4, suspected 0, unresolved 0

Residual: not measured

Coverage: jvm 8880 iterations, 15 of 15 entry points exercised

## Findings

No findings to report.

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (non-leak) E12
  - Contradicting evidence: E12
  - Originally supported by: E6
- **F2** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (non-leak) E12
  - Contradicting evidence: E12
  - Originally supported by: E6
- **F3** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (non-leak) E12
  - Contradicting evidence: E12
  - Originally supported by: E6
- **F4** (UNCLASSIFIED): discriminator E12
  - Ruled out by: plateau+purge (non-leak) E12
  - Contradicting evidence: E12
  - Originally supported by: E6

## Blind spots and unknowns

### Coverage gaps

- jvm/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- jvm/codeql: missing (codeql is not on PATH or in the tools directory). Install hint: dla install codeql
- jvm/errorprone: missing (Error Prone jars not found in the tools directory (or DLA_ERRORPRONE_HOME)). Install hint: dla install errorprone
- jvm/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)
- jvm/sonar: missing (sonar-scanner is not installed and Sonar rules need a SonarQube or SonarCloud server). Install hint: install sonar-scanner, run a SonarQube or SonarCloud server and set SONAR_HOST_URL and SONAR_TOKEN (dla install sonar prints the steps)
- jvm/valgrind: blocked (Valgrind is not run on the JVM: the virtual machine's own allocations drown the project's; use the allocator shim, jcmd NMT or `dla attach bpftrace`)

### Runtime blind spots

None recorded.

### Entry points not exercisable

None recorded.

### Evidence limitations and unknowns

- E4: partial coverage
- E4: limitation: no auxiliary classpath: library calls are not modelled
- E6: partial coverage
- E6: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E6: limitation: writers and removers are textual name matches
- E7: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E7: limitation: workloads were discovered by reading files; none was run
- E8: limitation: harness written; nothing was run (dla harness run measures it)
- E8: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E8: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E9: partial coverage (9 of 9 examined)
- E9: limitation: command-mode heap dumps come from separate processes: dla summarize heap-graph compares dumps of one process only, so use a driver (dla harness gen --driver) for a dump series
- E9: limitation: cgroup.*: /proc/21/cgroup unreadable (No such file or directory (os error 2))
- E9: limitation: os.fds: /proc/21/fd unreadable (No such file or directory (os error 2))
- E9: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E9: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E10: limitation: cgroup.*: /proc/21/cgroup unreadable (No such file or directory (os error 2))
- E10: limitation: os.fds: /proc/21/fd unreadable (No such file or directory (os error 2))
- E10: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E10: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: limitation: cgroup.*: /proc/21/cgroup unreadable (No such file or directory (os error 2))
- E11: limitation: os.fds: /proc/21/fd unreadable (No such file or directory (os error 2))
- E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E11: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: cgroup.*: /proc/21/cgroup unreadable (No such file or directory (os error 2))
- E12: limitation: os.fds: /proc/21/fd unreadable (No such file or directory (os error 2))
- E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: points 1 to 9 come from S1; the extra points from S2
- E12: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E12: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E12: limitation: gate and plateau thresholds are provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | — |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | — |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | — |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | — |
| `intake.build` | Does the project build locally? | answered | E1 | — |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | — |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E1 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | — |
| E2 | run-config | static | dla 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | probe/probe.json |
| E4 | static-hit | static | spotbugs unknown | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | sweep/jvm-spotbugs.json |
| E5 | static-hit | static | dla 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | sweep/jvm-dla.json |
| E6 | census-holder | static | dla-census 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | census/jvm.json |
| E7 | workload | static | dla 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | workload/workloads.json |
| E8 | workload | static | dla 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | harness/H1/harness.json |
| E9 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (5103cac37bbc) | harness/H1/run-S1.json |
| E10 | growth-series | test | dla 0.1.0 | artifact series/S1.json (7bbd8c52f7af) | trend/T1.json |
| E11 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (7bbd8c52f7af) | trend/T1.json |
| E12 | discriminator | test | dla 0.1.0 | artifact series/S3.json (1f8b92994b22) | alloc/A1.json |
