---
description: Smoke case on the native-vendor-codec fixture.
runs: 1
max_turns: 80
timeout_seconds: 1500
allowed_tools: [Read, Glob, Grep, Skill, Bash]
---

Use /dla:detect-leak-anywhere for this investigation.

The capture tool's memory grows in proportion to the number of packets it
decodes; during a long recording session it reaches several gigabytes. The
on-call engineer wants to know what accumulates and where.

This is an unattended run and nobody can answer questions. You have my consent for privilege tiers 1 to 4 and for raw values: record it with `dla consent --tiers 1,2,3,4 --raw-values`. Do not change the project's source files. Finish with the report printed by `dla report`, shown exactly as printed.
