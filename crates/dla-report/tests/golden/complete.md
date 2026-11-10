# DLA report

Run: `20260101-000000-abcd`

Status: COMPLETE

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 3 enumerated, 3 dispositioned; confirmed 1, strong 0, refuted 1, suspected 0, unresolved 1

Residual: not measured

Coverage: not measured

## Findings

### F1: **UNCLASSIFIED**

- State: confirmed
- Evidence classes: growth
- Holder: container-field
- Owner: `Cache.entries`
- Region: heap
- Shape: steady
- Locus: app
- Site: `src/cache.py:42`
- Writers: `src/cache.py:42`, `src/loader.py:17`
- Removers: `src/cache.py:90`
- Annotation: nothing evicts on the request path
- Verified: no
- Repair class: bound/evict
- Supporting evidence: E2
- Explanation (observed): Entries are never evicted.

### F3: **UNCLASSIFIED**

- State: unresolved(no_harness)
- Evidence classes: static
- Site: `src/net.py:7`
- Reason: no_harness
- Remedy: write a harness for net.py
- Supporting evidence: E4

## Ruled out

- **F2** (UNCLASSIFIED): discriminator E3
  - Ruled out by: discriminator E3
  - Hypothesis: N-WARM
  - Refuted if: growth is flat after warmup
  - Contradicting evidence: E3
  - Originally supported by: E4

## Blind spots and unknowns

### Coverage gaps

No coverage manifest recorded.

### Runtime blind spots

No coverage manifest recorded.

### Entry points not exercisable

No coverage manifest recorded.

### Evidence limitations and unknowns

- E2: partial coverage (40 of 100 examined)
- E2: limitation: sampled every 5s | misses short bursts
- E2: unknown: Is the cache bounded? Resolve by: read CacheConfig

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
| E1 | run-config | static | dla 0.1.0 | repo abc123 | — |
| E2 | growth-series | live | pyprobe 2.1 | process 4242 @ 2026-01-01T00:00:00Z | raw/growth.json |
| E3 | discriminator | test | dla 0.1.0 | repo abc123 | — |
| E4 | static-hit | static | scan 1.0 | artifact src/a\|b.py (0123456789ab) | — |
