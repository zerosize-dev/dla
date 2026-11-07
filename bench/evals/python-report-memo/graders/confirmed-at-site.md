---
type: regex
target: trace
pattern: 'State: confirmed[\s\S]{0,600}?Site: `(reports/builder\.py:1[45]|reports/service\.py:15)`'
---
The report printed by `dla report` lists a finding in state confirmed whose site is the
memoised method (reports/builder.py lines 14 or 15) or the per-request builder
(reports/service.py line 15).
