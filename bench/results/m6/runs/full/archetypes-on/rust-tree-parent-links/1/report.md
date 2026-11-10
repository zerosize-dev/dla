# DLA report

Run: `20261110-203449-97cd`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 8 enumerated, 8 dispositioned; confirmed 4, strong 0, refuted 4, suspected 0, unresolved 0

Residual: heap 5268.498106060606 bytes/n unattributed of 5268.498106060606 total (threshold 1317.1245265151515) ABOVE THRESHOLD, see O2; heap 5262.361363636363 bytes/n unattributed of 5262.361363636363 total (threshold 1315.5903409090909) ABOVE THRESHOLD, see O2; heap 5260.929331779332 bytes/n unattributed of 5260.929331779332 total (threshold 1315.232332944833) ABOVE THRESHOLD, see O2; heap 5262.361363636363 bytes/n unattributed of 5262.361363636363 total (threshold 1315.5903409090909) ABOVE THRESHOLD, see O2; heap 5260.929331779332 bytes/n unattributed of 5260.929331779332 total (threshold 1315.232332944833) ABOVE THRESHOLD, see O2; heap 5262.361363636363 bytes/n unattributed of 5262.361363636363 total (threshold 1315.5903409090909) ABOVE THRESHOLD, see O2; heap 5260.929331779332 bytes/n unattributed of 5260.929331779332 total (threshold 1315.232332944833) ABOVE THRESHOLD, see O2; heap 5262.361363636363 bytes/n unattributed of 5262.361363636363 total (threshold 1315.5903409090909) ABOVE THRESHOLD, see O2; heap 5260.929331779332 bytes/n unattributed of 5260.929331779332 total (threshold 1315.232332944833) ABOVE THRESHOLD, see O3

Coverage: rust 1760 iterations, 0 of 10 entry points exercised

## Findings

### F1: **L-CYCLE**

- State: confirmed
- Label: L-CYCLE (evidence: E10 unit-count-series, E15 discriminator)
- Growth: `native.live_blocks@src/section.rs:16` 40.54 count/n (steady, gate pass, E10)
- Evidence classes: context, discriminator, growth, static
- Holder: cycle-only
- Region: heap
- Shape: steady
- Site: `src/section.rs:10`
- Writers: `src/section.rps:10`, `src/section.rs:28`
- Verified: DISAGREES (site: finding `src/section.rs:10` vs verifier `src/section.rs:27`; bundle fd1cf28dfb2c); finding mechanism: Detector hit for Section.parent at src/section.rs:10 (cycle-only holder): the strong Rc back-link, set in append at src/section.rs:28, closes a cycle with the children Vec and is never released.; verifier mechanism: Section.parent is a strong Rc to the parent while the parent's children Vec holds strong Rc to the child; each parsed document tree forms parent<->child Rc cycles, so dropping the root never frees any section (no Weak). Every request leaks the whole tree.
- Sibling hypotheses: F1.n1
- Suggested archetype (unconfirmed): L-CYCLE
- Supporting evidence: E4, E8, E10, E15
- Explanation (inferred): Detector hit for Section.parent at src/section.rs:10 (cycle-only holder): the strong Rc back-link, set in append at src/section.rs:28, closes a cycle with the children Vec and is never released.
- Explanation structure: adds: src/section.rs:28; removal: never

### F2: **L-CYCLE**

- State: confirmed
- Label: L-CYCLE (evidence: E10 unit-count-series, E16 discriminator)
- Growth: `native.live_blocks@src/section.rs:16` 40.54 count/n (steady, gate pass, E10)
- Evidence classes: context, discriminator, growth, static
- Holder: cycle-only
- Region: heap
- Shape: steady
- Site: `src/section.rs:11`
- Writers: `src/section.rs:29`
- Verified: DISAGREES (site: finding `src/section.rs:11` vs verifier `src/section.rs:27`; bundle f0e0ba9a2bd9); finding mechanism: Detector hit for Section.children at src/section.rs:11 (cycle-only holder): strong Rc children pushed at src/section.rs:29 form a cycle with the parent link and are never released.; verifier mechanism: Section::append stores a strong Rc to the parent in child.parent while the parent's children Vec holds a strong Rc to the child; each parsed document tree is a reference cycle, never freed when the request ends, so live blocks grow per document
- Sibling hypotheses: F2.n1
- Suggested archetype (unconfirmed): L-CYCLE
- Supporting evidence: E4, E8, E10, E16
- Explanation (inferred): Detector hit for Section.children at src/section.rs:11 (cycle-only holder): strong Rc children pushed at src/section.rs:29 form a cycle with the parent link and are never released.
- Explanation structure: adds: src/section.rs:29; removal: never

### F3: **L-CYCLE**

- State: confirmed
- Label: L-CYCLE (evidence: E10 unit-count-series, E11 discriminator)
- Growth: `native.live_blocks@src/section.rs:16` 40.54 count/n (steady, gate pass, E10)
- Evidence classes: context, discriminator, growth, static
- Holder: cycle-only
- Owner: `src.section.Section.parent`
- Region: heap
- Shape: steady
- Site: `src/section.rs:10`
- Writers: `src/section.rs:28`
- Annotation: Child holds strong Rc to parent while parent holds strong Rc to child: cycle never reaches zero
- Verified: DISAGREES (site: finding `src/section.rs:10` vs verifier `src/section.rs:27`; bundle 66b965ab376b); finding mechanism: Section.parent (src/section.rs:10, cycle-only holder) is set as a strong Rc by Section::append at src/section.rs:28 while the parent's children Vec holds the child, so each parsed document's tree is a cycle and is never freed.; verifier mechanism: Section::append stores a strong Rc parent in child.parent while the parent holds the child in children; every parsed document tree is a parent<->child Rc cycle, so dropping the root never frees it and each request leaks its whole tree
- Sibling hypotheses: F3.n1
- Supporting evidence: E5, E8, E10, E11
- Explanation (inferred): Section.parent (src/section.rs:10, cycle-only holder) is set as a strong Rc by Section::append at src/section.rs:28 while the parent's children Vec holds the child, so each parsed document's tree is a cycle and is never freed.
- Explanation structure: adds: src/section.rs:28; removal: never

### F4: **L-CYCLE**

- State: confirmed
- Label: L-CYCLE (evidence: E10 unit-count-series, E14 discriminator)
- Growth: `native.live_blocks@src/section.rs:16` 40.54 count/n (steady, gate pass, E10)
- Evidence classes: context, discriminator, growth, static
- Holder: cycle-only
- Owner: `src.section.Section.children`
- Region: heap
- Shape: steady
- Site: `src/section.rs:11`
- Writers: `src/section.rs:28`, `src/section.rs:29`
- Annotation: heuristic name match over the project's sources
- Verified: DISAGREES (site: finding `src/section.rs:11` vs verifier `src/section.rs:27`; bundle f48c32b37cf1); finding mechanism: Section.children (src/section.rs:11, cycle-only holder) gets each child pushed at src/section.rs:29 while the child's parent link holds the parent strongly, so the Rc cycle is never freed after the response.; verifier mechanism: Section.append stores a strong Rc parent in child.parent while parent.children holds strong Rc to child; each per-document tree is an Rc cycle, never freed when the document is dropped, so live blocks grow linearly per request
- Sibling hypotheses: F4.n1
- Supporting evidence: E5, E8, E10, E14
- Explanation (inferred): Section.children (src/section.rs:11, cycle-only holder) gets each child pushed at src/section.rs:29 while the child's parent link holds the parent strongly, so the Rc cycle is never freed after the response.
- Explanation structure: adds: src/section.rs:29; removal: never

## Ruled out

- **F1.n1** (UNCLASSIFIED): discriminator E15
  - Ruled out by: plateau+purge (leak) E15
  - Hypothesis: N-WARM: growth is warm-up, plateaus
  - Refuted if: growth continues over the extra cycles and survives purge
  - Contradicting evidence: E15
  - Originally supported by: E10
- **F2.n1** (UNCLASSIFIED): discriminator E16
  - Ruled out by: plateau+purge (leak) E16
  - Hypothesis: N-WARM: growth is warm-up, plateaus
  - Refuted if: growth continues over the extra cycles and survives purge
  - Contradicting evidence: E16
  - Originally supported by: E10
- **F3.n1** (UNCLASSIFIED): discriminator E11
  - Ruled out by: plateau+purge (leak) E11
  - Hypothesis: N-WARM: growth is warm-up, plateaus
  - Refuted if: growth continues over the extra cycles and survives purge
  - Contradicting evidence: E11
  - Originally supported by: E10
- **F4.n1** (UNCLASSIFIED): discriminator E14
  - Ruled out by: plateau+purge (leak) E14
  - Hypothesis: N-WARM: growth is warm-up, plateaus
  - Refuted if: growth continues over the extra cycles and survives purge
  - Contradicting evidence: E14
  - Originally supported by: E10

## Blind spots and unknowns

### Coverage gaps

- rust/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- rust/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)
- rust/rapx: blocked (cannot copy the project: Permission denied (os error 13))

### Runtime blind spots

- rust: the allocator shim does not record which entry points ran

### Entry points not exercisable

- rust: `src/parser.rs::ParseError.fmt`: no_harness

### Evidence limitations and unknowns

- E5: partial coverage
- E5: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E5: limitation: writers and removers are textual name matches
- E6: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E6: limitation: workloads were discovered by reading files; none was run
- E7: limitation: harness written; nothing was run (dla harness run measures it)
- E7: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E8: partial coverage (9 of 9 examined)
- E8: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E8: limitation: exercised entry points are not measured for rust: the allocator shim sees allocations, not calls
- E9: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E9: limitation: gate thresholds are provisional (not calibrated)
- E10: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E10: limitation: gate thresholds are provisional (not calibrated)
- E11: partial coverage (9 of 26 examined)
- E11: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E11: limitation: points 1 to 9 come from S1; the extra points from S2
- E11: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E11: limitation: gate and plateau thresholds are provisional (not calibrated)
- E12: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E12: limitation: gate thresholds are provisional (not calibrated)
- E13: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E13: limitation: gate thresholds are provisional (not calibrated)
- E14: partial coverage (9 of 32 examined)
- E14: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E14: limitation: points 1 to 9 come from S1; the extra points from S4
- E14: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E14: limitation: gate and plateau thresholds are provisional (not calibrated)
- E15: partial coverage (9 of 32 examined)
- E15: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E15: limitation: points 1 to 9 come from S1; the extra points from S6
- E15: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E15: limitation: gate and plateau thresholds are provisional (not calibrated)
- E16: partial coverage (9 of 32 examined)
- E16: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E16: limitation: points 1 to 9 come from S1; the extra points from S8
- E16: limitation: scratch copy: 23 path(s) not copied (not a regular file)
- E16: limitation: gate and plateau thresholds are provisional (not calibrated)
- E17: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E17: limitation: gate thresholds are provisional (not calibrated)
- E18: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E18: limitation: gate thresholds are provisional (not calibrated)
- E19: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E19: limitation: gate thresholds are provisional (not calibrated)
- E20: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: 548 live bytes sit in allocations with no frame in the project (library-internal or without debug info)
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: partial coverage (1 of 3 examined)
- E23: limitation: F1: growth measured as a count; bytes not attributed
- E23: limitation: F2: growth measured as a count; bytes not attributed
- E23: limitation: F3: growth measured as a count; bytes not attributed
- E23: limitation: F4: growth measured as a count; bytes not attributed
- E23: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E23: limitation: threshold is provisional (not calibrated)
- E24: partial coverage (1 of 3 examined)
- E24: limitation: F1: growth evidence is from S1, not S2; not subtracted
- E24: limitation: F2: growth evidence is from S1, not S2; not subtracted
- E24: limitation: F3: growth evidence is from S1, not S2; not subtracted
- E24: limitation: F4: growth evidence is from S1, not S2; not subtracted
- E24: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E24: limitation: threshold is provisional (not calibrated)
- E25: partial coverage (1 of 3 examined)
- E25: limitation: F1: growth evidence is from S1, not S3; not subtracted
- E25: limitation: F2: growth evidence is from S1, not S3; not subtracted
- E25: limitation: F3: growth evidence is from S1, not S3; not subtracted
- E25: limitation: F4: growth evidence is from S1, not S3; not subtracted
- E25: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E25: limitation: threshold is provisional (not calibrated)
- E26: partial coverage (1 of 3 examined)
- E26: limitation: F1: growth evidence is from S1, not S4; not subtracted
- E26: limitation: F2: growth evidence is from S1, not S4; not subtracted
- E26: limitation: F3: growth evidence is from S1, not S4; not subtracted
- E26: limitation: F4: growth evidence is from S1, not S4; not subtracted
- E26: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E26: limitation: threshold is provisional (not calibrated)
- E27: partial coverage (1 of 3 examined)
- E27: limitation: F1: growth evidence is from S1, not S5; not subtracted
- E27: limitation: F2: growth evidence is from S1, not S5; not subtracted
- E27: limitation: F3: growth evidence is from S1, not S5; not subtracted
- E27: limitation: F4: growth evidence is from S1, not S5; not subtracted
- E27: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E27: limitation: threshold is provisional (not calibrated)
- E28: partial coverage (1 of 3 examined)
- E28: limitation: F1: growth evidence is from S1, not S6; not subtracted
- E28: limitation: F2: growth evidence is from S1, not S6; not subtracted
- E28: limitation: F3: growth evidence is from S1, not S6; not subtracted
- E28: limitation: F4: growth evidence is from S1, not S6; not subtracted
- E28: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E28: limitation: threshold is provisional (not calibrated)
- E29: partial coverage (1 of 3 examined)
- E29: limitation: F1: growth evidence is from S1, not S7; not subtracted
- E29: limitation: F2: growth evidence is from S1, not S7; not subtracted
- E29: limitation: F3: growth evidence is from S1, not S7; not subtracted
- E29: limitation: F4: growth evidence is from S1, not S7; not subtracted
- E29: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E29: limitation: threshold is provisional (not calibrated)
- E30: partial coverage (1 of 3 examined)
- E30: limitation: F1: growth evidence is from S1, not S8; not subtracted
- E30: limitation: F2: growth evidence is from S1, not S8; not subtracted
- E30: limitation: F3: growth evidence is from S1, not S8; not subtracted
- E30: limitation: F4: growth evidence is from S1, not S8; not subtracted
- E30: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E30: limitation: threshold is provisional (not calibrated)
- E31: partial coverage (1 of 3 examined)
- E31: limitation: F1: growth measured as a count; bytes not attributed
- E31: limitation: F2: growth measured as a count; bytes not attributed
- E31: limitation: F3: growth measured as a count; bytes not attributed
- E31: limitation: F4: growth measured as a count; bytes not attributed
- E31: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E31: limitation: threshold is provisional (not calibrated)
- E32: partial coverage (1 of 3 examined)
- E32: limitation: F1: growth evidence is from S1, not S2; not subtracted
- E32: limitation: F2: growth evidence is from S1, not S2; not subtracted
- E32: limitation: F3: growth evidence is from S1, not S2; not subtracted
- E32: limitation: F4: growth evidence is from S1, not S2; not subtracted
- E32: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E32: limitation: threshold is provisional (not calibrated)
- E33: partial coverage (1 of 3 examined)
- E33: limitation: F1: growth evidence is from S1, not S3; not subtracted
- E33: limitation: F2: growth evidence is from S1, not S3; not subtracted
- E33: limitation: F3: growth evidence is from S1, not S3; not subtracted
- E33: limitation: F4: growth evidence is from S1, not S3; not subtracted
- E33: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E33: limitation: threshold is provisional (not calibrated)
- E34: partial coverage (1 of 3 examined)
- E34: limitation: F1: growth evidence is from S1, not S4; not subtracted
- E34: limitation: F2: growth evidence is from S1, not S4; not subtracted
- E34: limitation: F3: growth evidence is from S1, not S4; not subtracted
- E34: limitation: F4: growth evidence is from S1, not S4; not subtracted
- E34: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E34: limitation: threshold is provisional (not calibrated)
- E35: partial coverage (1 of 3 examined)
- E35: limitation: F1: growth evidence is from S1, not S5; not subtracted
- E35: limitation: F2: growth evidence is from S1, not S5; not subtracted
- E35: limitation: F3: growth evidence is from S1, not S5; not subtracted
- E35: limitation: F4: growth evidence is from S1, not S5; not subtracted
- E35: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E35: limitation: threshold is provisional (not calibrated)
- E36: partial coverage (1 of 3 examined)
- E36: limitation: F1: growth evidence is from S1, not S6; not subtracted
- E36: limitation: F2: growth evidence is from S1, not S6; not subtracted
- E36: limitation: F3: growth evidence is from S1, not S6; not subtracted
- E36: limitation: F4: growth evidence is from S1, not S6; not subtracted
- E36: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E36: limitation: threshold is provisional (not calibrated)
- E37: partial coverage (1 of 3 examined)
- E37: limitation: F1: growth evidence is from S1, not S7; not subtracted
- E37: limitation: F2: growth evidence is from S1, not S7; not subtracted
- E37: limitation: F3: growth evidence is from S1, not S7; not subtracted
- E37: limitation: F4: growth evidence is from S1, not S7; not subtracted
- E37: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E37: limitation: threshold is provisional (not calibrated)
- E38: partial coverage (1 of 3 examined)
- E38: limitation: F1: growth evidence is from S1, not S8; not subtracted
- E38: limitation: F2: growth evidence is from S1, not S8; not subtracted
- E38: limitation: F3: growth evidence is from S1, not S8; not subtracted
- E38: limitation: F4: growth evidence is from S1, not S8; not subtracted
- E38: limitation: off-heap: no gap series for S8; run dla gap S8 first (node.external is used when present)
- E38: limitation: threshold is provisional (not calibrated)
- E39: partial coverage (1 of 3 examined)
- E39: limitation: F1: growth evidence is from S1, not S9; not subtracted
- E39: limitation: F2: growth evidence is from S1, not S9; not subtracted
- E39: limitation: F3: growth evidence is from S1, not S9; not subtracted
- E39: limitation: F4: growth evidence is from S1, not S9; not subtracted
- E39: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E39: limitation: threshold is provisional (not calibrated)

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
| E1 | run-config | static | dla 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | — |
| E2 | run-config | static | dla 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | — |
| E3 | probe | static | dla-probe 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | probe/probe.json |
| E4 | static-hit | static | dla 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | sweep/rust-dla.json |
| E5 | census-holder | static | dla-census 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | census/rust.json |
| E6 | workload | static | dla 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | workload/workloads.json |
| E7 | workload | static | dla 0.1.0 | repo 6e55f6655f40b8af9f3dfd1130d4a4f2deed005f | harness/H1/harness.json |
| E8 | workload | test | dla 0.1.0 | artifact harness/H1/harness.json (0700c0ef7cc4) | harness/H1/run-S1.json |
| E9 | growth-series | test | dla 0.1.0 | artifact series/S1.json (649e8318ca35) | trend/T1.json |
| E10 | unit-count-series | test | dla 0.1.0 | artifact series/S1.json (649e8318ca35) | trend/T1.json |
| E11 | discriminator | test | dla 0.1.0 | artifact series/S3.json (85bbf7f14a5c) | alloc/A1.json |
| E12 | growth-series | test | dla 0.1.0 | artifact series/S2.json (0b3717ad46cf) | trend/T3.json |
| E13 | unit-count-series | test | dla 0.1.0 | artifact series/S2.json (0b3717ad46cf) | trend/T3.json |
| E14 | discriminator | test | dla 0.1.0 | artifact series/S5.json (c4d68e166028) | alloc/A2.json |
| E15 | discriminator | test | dla 0.1.0 | artifact series/S7.json (7ac6f60fad86) | alloc/A3.json |
| E16 | discriminator | test | dla 0.1.0 | artifact series/S9.json (fb1365b1ac2c) | alloc/A4.json |
| E17 | growth-series | test | dla 0.1.0 | artifact series/S4.json (8f56d9501603) | trend/T7.json |
| E18 | unit-count-series | test | dla 0.1.0 | artifact series/S4.json (8f56d9501603) | trend/T7.json |
| E19 | growth-series | test | dla 0.1.0 | artifact series/S6.json (d80265627fe9) | trend/T8.json |
| E20 | unit-count-series | test | dla 0.1.0 | artifact series/S6.json (d80265627fe9) | trend/T8.json |
| E21 | growth-series | test | dla 0.1.0 | artifact series/S8.json (de917561d07e) | trend/T9.json |
| E22 | unit-count-series | test | dla 0.1.0 | artifact series/S8.json (de917561d07e) | trend/T9.json |
| E23 | residual | test | dla 0.1.0 | artifact series/S1.json (649e8318ca35) | gap/residual-S1-E23.json |
| E24 | residual | test | dla 0.1.0 | artifact series/S2.json (0b3717ad46cf) | gap/residual-S2-E24.json |
| E25 | residual | test | dla 0.1.0 | artifact series/S3.json (85bbf7f14a5c) | gap/residual-S3-E25.json |
| E26 | residual | test | dla 0.1.0 | artifact series/S4.json (8f56d9501603) | gap/residual-S4-E26.json |
| E27 | residual | test | dla 0.1.0 | artifact series/S5.json (c4d68e166028) | gap/residual-S5-E27.json |
| E28 | residual | test | dla 0.1.0 | artifact series/S6.json (d80265627fe9) | gap/residual-S6-E28.json |
| E29 | residual | test | dla 0.1.0 | artifact series/S7.json (7ac6f60fad86) | gap/residual-S7-E29.json |
| E30 | residual | test | dla 0.1.0 | artifact series/S8.json (de917561d07e) | gap/residual-S8-E30.json |
| E31 | residual | test | dla 0.1.0 | artifact series/S1.json (649e8318ca35) | gap/residual-S1-E31.json |
| E32 | residual | test | dla 0.1.0 | artifact series/S2.json (0b3717ad46cf) | gap/residual-S2-E32.json |
| E33 | residual | test | dla 0.1.0 | artifact series/S3.json (85bbf7f14a5c) | gap/residual-S3-E33.json |
| E34 | residual | test | dla 0.1.0 | artifact series/S4.json (8f56d9501603) | gap/residual-S4-E34.json |
| E35 | residual | test | dla 0.1.0 | artifact series/S5.json (c4d68e166028) | gap/residual-S5-E35.json |
| E36 | residual | test | dla 0.1.0 | artifact series/S6.json (d80265627fe9) | gap/residual-S6-E36.json |
| E37 | residual | test | dla 0.1.0 | artifact series/S7.json (7ac6f60fad86) | gap/residual-S7-E37.json |
| E38 | residual | test | dla 0.1.0 | artifact series/S8.json (de917561d07e) | gap/residual-S8-E38.json |
| E39 | residual | test | dla 0.1.0 | artifact series/S9.json (fb1365b1ac2c) | gap/residual-S9-E39.json |
