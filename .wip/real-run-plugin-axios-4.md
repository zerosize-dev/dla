# Real plugin run axios-4 (frozen build, snapshot 0dbf4c9), 2026-11-21

Started 16:54 UTC, 129 s wall, 29 turns, cost 0.44, Status COMPLETE, "no leak confirmed",
budget 3 h, 1 min used, 13 of 39 ranked candidates targeted. Evidence: u36/axios-4.

## The tool did its part
- H1 at full length (9 points). harness run began with _attention: "READ THE DETECTOR HIT:
  listeners pile up at lib/adapters/http.js:924 (10 registered), finding F23".
- trend S1: 17 columns passed, first node.listeners@lib/adapters/http.js:924 slope 1.0
  candidates F23, F22, F24; next = DETECTOR HIT (harness gen --probe F23), missed targets,
  ledger add --sibling-of F23, ledger test F23 --evidence E8.

## The model walked away and the tool let it
- It chased two flat probes of its declared targets (F49, F50 in validator.js). The probe on
  deprecatedWarnings read 1 at every point although the unit adds a key per call (heap +94
  bytes per iteration): probably a probe defect (sent to N).
- ledger resolve intake.workload --status unresolved --reason tool_missing: accepted after two
  harnesses had run. Queue closed; close-rest closed 38 findings (F23 among them).
- close-rest printed "dla ledger test F9", refused as printed (needs --evidence).
- ledger resolve F9 --as unresolved --reason inconclusive: accepted for a finding whose column grew.
- Final gate passed; report COMPLETE.

## Fix
Pass 8 (wip-notes/pass8-brief.md, builder P3, scratch u49): intake reason contradicted by the
run's records is refused; a finding a passed column points at cannot be closed before an
alloc-test (close-rest leaves it with runnable commands, resolve refused, gate lists it);
a detector hit with a site is a gate item and its trend line prints the direct steps.
