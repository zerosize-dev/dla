# Real plugin run aiohttp-3 (2026-11-21 08:26 to 10:10 UTC), aiohttp 3.10.10

Binary = working tree at 08:24 (ranking, trigger step, Go work; = the combined snapshot minus
most report fixes). Model per run.py default. 1 h 44 min wall, report COMPLETE, final gate
passed. Kept: scratch u36/aiohttp-3/ (report.md, ledger.jsonl, H11 driver, its VARIES.txt);
run directory /root/dla-real/aiohttp-3/project/.dla/ (cache deleted).

Result: the documented leak is found, reproduced and verified, by a model that was not told
the cause.
- F8 `aiohttp/web_app.py:82` (the unbounded cache on `_build_middlewares`): grew in the first
  batch about 7 minutes in, confirmed, label L-COLLECT.
- Trigger: the model ran `dla harness gen --lang python --driver --entry` on its own for the
  confirmed findings. Its entry-path driver H11: a real server (an application with a
  middleware, a sub-application, routes), a client session; per unit four requests: a routed
  path, a sub-application path, an unknown path (`/nope<n>`) and an unsupported method.
  Report: "Trigger: reproduced through the public API in H11 (varies: URL path, method,
  User-Agent, Referer, X-K header and body of each HTTP request a remote client sends to a
  public aiohttp web server)".
- The model's annotation states the real mechanism: "for 404/405 match_info.handler is a
  bound SystemRoute._handle of a SystemRoute created per request, so every such request is a
  new cache key".
- Blind verifier: "Verified: agrees (verifier site aiohttp/web_app.py:82, holder
  static/global, label L-COLLECT)". (In aiohttp-2 the verifier disagreed on the same finding.)
- The other three confirmed holders are named for what they are: F9 (format cache) and F83
  (payload registry) "Trigger: not found" after two entry-path harnesses each, verifier
  N-INTENT; F189 (cookie jar) "Trigger: not checked".
- Queue: "68 of 68 ranked candidates targeted in 13 harnesses: grew 3, executed 56, not
  reached 9; 0 not targeted".
- Final answer: "One of the four confirmed leaks is a real trigger through aiohttp's public
  API. The other three are disputed or unproven."

Seen, to improve
- Residual header: 30 series above threshold (os columns of driver runs); the header is
  bounded now, but 37 residual rows were still made series by series (the one-command fix is
  in the next build).
- F189 "Trigger: not checked (no harness series that grew is tied to it)": it was confirmed
  through alloc stacks, not a probe; the trigger item does not ask for it.
- "unresolved 61" of 194: many measured-but-undecided candidates.
