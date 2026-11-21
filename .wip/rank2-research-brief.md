# Ranking of candidates, second step: research brief (read-only on the repo)

Repo /home/user/dla (branch main, working tree holds uncommitted work of other agents: do NOT
edit, commit, stash or reset anything in it; you only read it). 
sp = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad
Your scratch dir: `<sp>/u30/` (write scripts and the report there; delete nothing outside it;
never `rm -rf "$VAR/..."`). No model names, nothing committed.

## Why
DLA lists static leak candidates (rule hits and a "census" of holders: containers, registration
tables, static/global variables with their writer and remover sites) and now makes the agent
measure them in rank order, 8 per generated workload (`dla ledger list`, `dla harness gen
--driver --next`). A run has time for a few batches, so only about the first 16 to 24 ranked
candidates get a workload. The current order (crates/dla-core/src/rank.rs: tiers 1 to 6, then
findings with two origins first, then ledger order) is too coarse on real projects:

| project (tree) | runtime | candidates ranked | documented leak site | its rank now |
|---|---|---|---|---|
| aiohttp 3.10.10 (`<sp>/u29/aiohttp`) | python | 92 | `aiohttp/web_app.py:82` (lru_cache on `_build_middlewares`) | 1 |
| grpc-go v1.84.0 (`<sp>/u29/grpc`) | go | 298 | `health/server.go:49` (`Server.updates`: outer map entry per service name never deleted; the only deletes are on the inner map) | 159 (tier 4: "has removers") |
| spring-data-commons 4.1.1 (`<sp>/u29/spring`) | jvm | 479 | `src/main/java/org/springframework/data/core/TypedPropertyPaths.java:53` (static `resolved` map) | 325 (tier 4: 6 writers and 1 remover, most matched by NAME in other files) |
| axios (`<sp>/u18/axios/proj`, static stage not yet rerun) | node | ? | `lib/adapters/http.js:908-925` | ? |

Each tree under `<sp>/u29/<name>` has a finished static stage: `.dla/<run>/census/<lang>.json`
(entries with holder, kind, owner, site, writers, removers), `.dla/<run>/sweep/*.json`,
`.dla/<run>/ledger.jsonl`, and `<sp>/u29/<name>-list.jsonl` is the output of
`dla ledger list --all` (first line totals, then one JSON row per finding in rank order with
rank, tier, origin, holder, owner, site, writers, removers). For axios run
`bash <sp>/u29/static.sh axios node` first (it copies `<sp>/u18/axios/proj` to
`<sp>/u29/axios` and writes `<sp>/u29/axios-list.jsonl`; the binary it uses is
`<sp>/u28/Q1/bench/dla`, call it with `DLA_PLUGIN_ROOT=/home/user/dla`).

Things already seen in these lists (verify, do not take on trust):
- A delete on an ELEMENT of the holder (`delete(s.updates[service], stream)`) is counted as a
  remover of the holder; a write to an element (`s.updates[service][stream] = update`) may be
  counted as a writer.
- The JVM source census matches writers and removers of a `private static` field by name in
  other files (`Lazy.java:66` for `TypedPropertyPaths.resolved`).
- Package variables that are only reassigned or written once at start-up
  (`clientconn.emptyServiceConfig`, `credentials.tls.tls12ForbiddenCipherSuites`,
  `grpclog.internal.grpclog.LoggerV2Impl`) and Python `TypeVar` assignments
  (`aiohttp.client._RetType`) sit in tier 2 as "static/global with writers and no removers".
- Tier 1 on spring is 28 hits of one rule on instance fields of configuration objects.

## What to deliver
A measured proposal for the order, as `<sp>/u30/report.md`, built from data, not from argument:
1. A script (Python, in `<sp>/u30/`) that reads a project's census, sweep results and
   `-list.jsonl` plus the source lines of each writer and remover, computes per candidate a set
   of generic static signals, and prints the rank of the documented site under any scoring
   scheme given as weights. Signals to compute at least (add your own if the data supports
   them, and say for each how reliably it can be computed from a syntax tree, per language):
   - lifetime: static/global (process-long) vs field of an object vs local/closure;
   - holder is a real container (map, set, list, queue, cache type; not a scalar, pointer,
     logger, TypeVar, function value, sync.Pool, constant table);
   - writer kind: keyed insertion or append/add (growth) vs plain reassignment of the variable;
   - the key or the appended value derives from a parameter of the enclosing function (data
     flows in from a caller) vs a constant or a value fixed at start-up;
   - writers only in initialisers (Go `init`, package-level `var =`, constructors and `New*`
     functions, Java static initialisers and constructors, Python module top level and
     `__init__`, JS module top level) vs in functions that run per call or per request;
   - removers on the holder itself vs only on an element of it; removers matched by name in
     another file for a private or unexported holder (not really its remover);
   - visibility: the writer's function is part of the public surface (exported name, not under
     `internal/`, `_private` module, package-private class) or reachable only internally;
   - weak or bounded container types (WeakHashMap, LRU with a size, bounded channel, ring
     buffer) and what they bound (keys only, or values too);
   - a DLA rule hit at the site, and which rule.
2. For each of the four projects: the documented site's rank under the current order and under
   each scheme you try, and what is still ahead of it (the first 16 under your best scheme,
   one line each, with your judgement after reading their code: plausible leak, bounded by
   design, or noise). Read the code of those; a scheme that puts noise first is not better.
3. A guard against fitting the order to these four sites: the benchmark fixtures. For every
   fixture under `/home/user/dla/bench/fixtures/*/` the answer key (find it: a YAML or JSON
   with the leak sites next to the fixture's project) names the true leak sites. Run the static
   stage on a copy of at least 16 fixtures spread over the runtimes (copy the fixture's
   `project` directory to `<sp>/u30/fx/<name>`, `git init` it, then the commands of
   `<sp>/u29/static.sh`; some fixtures have several runtimes: run sweep and census for each
   runtime the probe reports) and report, per fixture, the rank of each true leak site now and
   under your best scheme. The scheme must not push a fixture's true site below rank 8 where it
   is within 8 now. Also report non-leak twins (fixtures whose key says there is no leak): what
   ranks first there.
4. The recommended scheme, stated exactly enough to implement: each signal's definition, how
   the census of each language can compute it (read `/home/user/dla/crates/dla-census/src/`
   to see what each language's census already extracts and name the functions that would
   change), the order or weights, and what must be stored on a census entry or a ledger
   annotation. Say which signals you could only approximate in your script (regex on source
   lines) and how far the approximation was off where you checked by hand.
5. What you could not establish.

## Rules
- Read-only on /home/user/dla. Never edit fixtures or answer keys. Do not run `dla-bench`.
- The order must stay a pure function of a finding and its census data (no network, no model).
- Machine has 4 CPUs shared with two other jobs: run one static stage at a time.
- Disk: if `df --output=avail -B1M /` is below 2500, delete your own fixture copies under
  `<sp>/u30/fx/` by literal path.
- Return at most 60 lines: the table of ranks (now and proposed) for the four projects, the
  fixture guard result, the recommended scheme in short, and what is unverified. The full
  detail goes into `<sp>/u30/report.md`.
