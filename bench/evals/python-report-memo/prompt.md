---
description: Smoke case on the python-report-memo fixture.
runs: 1
max_turns: 80
timeout_seconds: 1500
allowed_tools: [Read, Glob, Grep, Skill, Bash]
---

Use /dla:detect-leak-anywhere for this investigation.

The report service's resident memory climbs steadily while it renders
account statements for the front end: after every few thousand requests the
process holds more than it did after the previous few thousand, and the
container is restarted by the scheduler about once a day. Find out what is
holding the memory.

This is an unattended run and nobody can answer questions. You have my consent for privilege tiers 1 to 4 and for raw values: record it with `dla consent --tiers 1,2,3,4 --raw-values`. Do not change the project's source files. Finish with the report printed by `dla report`, shown exactly as printed.
