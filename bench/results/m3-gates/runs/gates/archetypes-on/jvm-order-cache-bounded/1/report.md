# DLA report

Run: `20261108-124700-6b30`

Status: COMPLETE

Findings: 8 enumerated, 8 dispositioned; confirmed 4, strong 0, refuted 4, suspected 0, unresolved 0

Residual: not measured

Coverage: jvm 8880 iterations, 15 of 15 entry points exercised

## Findings

### F1.n1: Hypothesis N-WARM: growth in F1 is JVM warm-up / bounded cache fill that plateaus (UNCLASSIFIED)

- State: confirmed
- Explained by N-WARM (discriminator E14)
- Evidence classes: context, discriminator
- Sibling hypothesis of: F1
- Hypothesis: N-WARM: growth in F1 is JVM warm-up / bounded cache fill that plateaus
- Refuted if: heap and probe sizes keep growing after the 1000-entry cap is reached and after purge
- Supporting evidence: E10, E14

### F2.n1: Hypothesis N-WARM: growth in F2 is JVM warm-up / bounded cache fill that plateaus (UNCLASSIFIED)

- State: confirmed
- Explained by N-WARM (discriminator E15)
- Evidence classes: context, discriminator
- Sibling hypothesis of: F2
- Hypothesis: N-WARM: growth in F2 is JVM warm-up / bounded cache fill that plateaus
- Refuted if: heap and probe sizes keep growing after the 1000-entry cap is reached and after purge
- Supporting evidence: E10, E15

### F3.n1: Hypothesis N-WARM: growth in F3 is JVM warm-up / bounded cache fill that plateaus (UNCLASSIFIED)

- State: confirmed
- Explained by N-WARM (discriminator E16)
- Evidence classes: context, discriminator
- Sibling hypothesis of: F3
- Hypothesis: N-WARM: growth in F3 is JVM warm-up / bounded cache fill that plateaus
- Refuted if: heap and probe sizes keep growing after the 1000-entry cap is reached and after purge
- Supporting evidence: E10, E16

### F4.n1: Hypothesis N-WARM: growth in F4 is JVM warm-up / bounded cache fill that plateaus (UNCLASSIFIED)

- State: confirmed
- Explained by N-WARM (discriminator E17)
- Evidence classes: context, discriminator
- Sibling hypothesis of: F4
- Hypothesis: N-WARM: growth in F4 is JVM warm-up / bounded cache fill that plateaus
- Refuted if: heap and probe sizes keep growing after the 1000-entry cap is reached and after purge
- Supporting evidence: E10, E17

## Ruled out

- **F1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (non-leak) E14
  - Contradicting evidence: E14
  - Originally supported by: E7, E9
- **F2** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (non-leak) E15
  - Contradicting evidence: E15
  - Originally supported by: E7, E9
- **F3** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (non-leak) E16
  - Contradicting evidence: E16
  - Originally supported by: E7, E9
- **F4** (UNCLASSIFIED): discriminator E17
  - Ruled out by: plateau+purge (non-leak) E17
  - Contradicting evidence: E17
  - Originally supported by: E7, E9

## Blind spots and unknowns

### Coverage gaps

- jvm/codeql: missing (codeql is not on PATH or in the tools directory). Install hint: dla install codeql
- jvm/errorprone: missing (Error Prone jars not found in the tools directory (or DLA_ERRORPRONE_HOME)). Install hint: dla install errorprone
- jvm/sonar: missing (sonar-scanner is not installed and Sonar rules need a SonarQube or SonarCloud server). Install hint: install sonar-scanner, run a SonarQube or SonarCloud server and set SONAR_HOST_URL and SONAR_TOKEN (dla install sonar prints the steps)

### Runtime blind spots

None recorded.

### Entry points not exercisable

None recorded.

### Evidence limitations and unknowns

- E5: partial coverage
- E5: limitation: no auxiliary classpath: library calls are not modelled
- E7: partial coverage
- E7: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E7: limitation: writers and removers are textual name matches
- E8: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E8: limitation: workloads were discovered by reading files; none was run
- E9: limitation: harness written; nothing was run (dla harness run measures it)
- E9: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E9: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E10: partial coverage (9 of 9 examined)
- E10: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E10: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E11: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E11: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E11: limitation: gate thresholds are provisional (not calibrated)
- E12: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E12: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E13: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E13: limitation: points 1 to 9 come from S1; the extra points from S2
- E13: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E13: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E13: limitation: gate and plateau thresholds are provisional (not calibrated)
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E14: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E14: limitation: points 1 to 9 come from S1; the extra points from S4
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E15: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E15: limitation: points 1 to 9 come from S1; the extra points from S6
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E16: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E16: limitation: points 1 to 9 come from S1; the extra points from S8
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: jvm probe bytes are a shallow estimate assuming compressed oops
- E17: limitation: os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in
- E17: limitation: points 1 to 9 come from S1; the extra points from S10
- E17: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E17: limitation: note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts
- E17: limitation: gate and plateau thresholds are provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E2 | order service JVM heap growth; replay via shop.Main; consent from runner |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E2 | order service JVM heap growth; replay via shop.Main; consent from runner |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E2 | order service JVM heap growth; replay via shop.Main; consent from runner |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E2 | order service JVM heap growth; replay via shop.Main; consent from runner |
| `intake.build` | Does the project build locally? | answered | E2 | order service JVM heap growth; replay via shop.Main; consent from runner |
| `intake.workload` | What can exercise the suspected path? | answered | E2 | order service JVM heap growth; replay via shop.Main; consent from runner |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | order service JVM heap growth; replay via shop.Main; consent from runner |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E2 | order service JVM heap growth; replay via shop.Main; consent from runner |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | — |
| E2 | run-config | static | dla 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | probe/probe.json |
| E4 | probe | static | dla-probe 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | probe/probe.json |
| E5 | static-hit | static | spotbugs unknown | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | sweep/spotbugs.json |
| E6 | static-hit | static | dla 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | sweep/dla.json |
| E7 | census-holder | static | dla-census 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | census/jvm.json |
| E8 | workload | static | dla 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | workload/workloads.json |
| E9 | workload | static | dla 0.1.0 | repo 15fae5ea51753aaf07e9ebb26ed246401e264ff1 | harness/H1/harness.json |
| E10 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (bdd1ad3c5a18) | harness/H1/run-S1.json |
| E11 | growth-series | test | dla 0.1.0 | artifact series/S1.json (90fe7de532d7) | trend/T1.json |
| E12 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (90fe7de532d7) | trend/T1.json |
| E13 | discriminator | test | dla 0.1.0 | artifact series/S3.json (c99a4749a524) | alloc/A1.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S5.json (2dbc0dec15c7) | alloc/A2.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S7.json (525d93d5e569) | alloc/A3.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S9.json (830857fa622d) | alloc/A4.json |
| E17 | discriminator | test | dla 0.1.0 | artifact series/S11.json (e4742e3e529f) | alloc/A5.json |
