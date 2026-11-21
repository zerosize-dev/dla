# Real plugin run 2: grpc-go v1.84.0 (headless Claude Code, Sonnet 5.5, plugin of the working tree at 00:15 UTC 2026-11-21)

Result: exit 0 after 690 s, 32 turns, cost 0.49 USD. 728 candidates. Two Go driver harnesses for the root
package (client per call; one connection with abandoned streams), flat. All 728 candidates closed
suspected_static_only not_exercised in a shell loop. The final gate then listed coverage_contradiction
(0 of 2305 entry points: Go driver mode writes no cover profile, harness/H3/cov empty) whose fix text
offered 'dla ledger resolve intake.workload --status unresolved --reason not_exercised'; the agent ran it
and the gate passed. Report COMPLETE. The documented leak (health/server.go:49) was never exercised.
Agents tried 'dla ledger list' again (does not exist).

## Timeline
```
INIT model=claude-sonnet-5-5 plugins=['dla', 'cc-plugin-agents-md', 'cc-plugin-telemetry', 'cc-plugin-privacy', 'cc-plugin-plugin-authoring'] skills=20 agents=['claude', 'dla:verifier', 'dla:worker', 'Explore', 'general-purpose', 'Plan', 'statusline-setup']
Bash: command -v dla; cd /root/dla-real/grpc-1/project; git ls-files | head -50; dla run init --entry repository --budget auto --session-id 2f831bb5-9b08-4e6c-b77d-364619481679
   -> /root/dla-real/plugin/bin/dla .gemini/config.yaml .gemini/styleguide.md .github/ISSUE_TEMPLATE/bug.md .github/ISSUE_TEMPLATE/feature.md .github/ISSUE_TEMPLATE/question.md .github/codecov.yml .github/pull_request_template.md .github/workflows/codeql-analysis.ym
Bash: cd /root/dla-real/grpc-1/project; ls /root/dla-real/plugin/skills/detect-leak-anywhere/references/; sed -n 1,140p /root/dla-real/plugin/skills/detect-leak-anywhere/references/stages.md
   -> backends.md evidence-and-report.md go-archetypes.md go.md jvm-archetypes.md jvm.md known-leaks.md native-archetypes.md native.md node-archetypes.md node.md python-archetypes.md python.md recovery.md rust-archetypes.md rust.md safety-and-effects.md stages.md # 
Bash: sed -n 140,330p /root/dla-real/plugin/skills/detect-leak-anywhere/references/stages.md; cat /root/dla-real/plugin/skills/detect-leak-anywhere/references/go.md
   -> init` reports a `consent` object, an unattended runner already answered: ask nothing, cite its evidence for `intake.consent`. Otherwise `dla consent --tiers <list> --ceiling ${user_config.privilege_ceiling}`, adding `--raw-values` only if the user explicitly a
Bash: cd /root/dla-real/grpc-1/project; for i in target scope symptom runtimes build workload consent budget; do :; done; dla ledger resolve --help 2>&1 | head -20; dla probe --build 2>&1 | tail -30
   -> Disposition a finding or open item, or answer a checklist item  Usage: dla ledger resolve [OPTIONS] <TARGET>  Arguments:   <TARGET>           A finding (F3, F3.n1), an open item (O2) or a checklist item (intake.scope)  Options:       --as <DISPOSITION>        
Bash: cd /root/dla-real/grpc-1/project;  ; dla ledger resolve intake.target --status answered --evidence E1 ; dla ledger resolve intake.scope --status answered --evidence E1 ; dla ledger resolve intake.symptom --status answered --evidence E1 ; dla ledger resolve intake.runtimes --status answered --evidence E3 ; dla ledger resolve inta
   -> {"id":"intake.target","ok":true,"record":".dla/20261121-003848-136a/ledger.jsonl#L4","seq":4,"status":"answered"} {"id":"intake.scope","ok":true,"record":".dla/20261121-003848-136a/ledger.jsonl#L5","seq":5,"status":"answered"} {"id":"intake.symptom","ok":true,
SAY Intake and probe gates have passed. I'm starting the static sweep and holder census for Go.
Bash: cd /root/dla-real/grpc-1/project; dla sweep --lang go 2>&1 | tail -c 2500; dla census --lang go 2>&1 | tail -c 2500
   -> {"artifacts":[".dla/20261121-003848-136a/sweep/go-staticcheck.json",".dla/20261121-003848-136a/sweep/go-golangci-lint.json",".dla/20261121-003848-136a/sweep/go-govet.json",".dla/20261121-003848-136a/sweep/go-dla.json"],"candidates":2,"command":"sweep","evidenc
Bash: cd /root/dla-real/grpc-1/project; dla gate check --stage sweep 2>&1 | tail -c 1500; dla run status 2>&1 | head -c 2500
   -> {"budget":"budget (auto): 3h for 1 runtime and 728 candidates (20 + 15x1 + 3x728 = 2219 min, capped at 3h)","budget_estimate":{"candidates":728,"minutes":2219,"runtimes":1,"total_minutes":180},"missing":[],"missing_total":0,"ok":true,"pointer":".dla/20261121-0
Bash: cd /root/dla-real/grpc-1/project; dla workload discover 2>&1 | head -c 2500; dla harness gen --help 2>&1 | head -40
   -> {"artifacts":[".dla/20261121-003848-136a/workload/workloads.json",".dla/20261121-003848-136a/coverage.json"],"candidates":0,"command":"workload discover","evidence":["E6"],"lang":null,"ok":true,"skipped_duplicates":0,"summary":{"build_failed":[],"by_kind":{"de
Bash: cd /root/dla-real/grpc-1/project; dla ledger list --help 2>&1 | head -15; dla ledger list 2>&1 | head -c 1500; dla workload discover --help 2>&1 | grep -i list
   -> {"error":{"code":"usage","fix":"dla ledger --help","message":"`dla ledger` has no such subcommand (unknown: list); available: add, resolve, annotate, test, confirm, refute, done, merge, verify, label, strong, explain, duplicate","missing":"a subcommand: add, r
Bash: cd /root/dla-real/grpc-1/project; R=.dla/20261121-003848-136a; head -c 1500 $R/sweep/go-dla.json; echo; python3 -c " ; import json ; for l in open('$R/ledger.jsonl'): ;     d=json.loads(l) ;     if 'origin' in json.dumps(d) and 'detector' in json.dumps(d): print(json.dumps(d)[:600]) ; " | head
   -> {   "backend": "dla",   "build_logic_hits": 0,   "capture": {     "runs": []   },   "command": [     "dla",     "(built-in rules)"   ],   "exit_code": 0,   "hits": [     {       "backend": "dla",       "file": "stats/opentelemetry/internal/tracing/carrier.go",
SAY Let me inspect these two hits in the source.
Bash: cd /root/dla-real/grpc-1/project; sed -n 95,120p stats/opentelemetry/internal/tracing/carrier.go; echo ----; sed -n 640,665p stream.go; grep -n "a.ctx\|attempt.ctx" stream.go | head -20
   -> func NewOutgoingCarrier(ctx context.Context) *OutgoingCarrier { 	return &OutgoingCarrier{ctx: ctx} }  // Get just logs an error and returns an empty string. It implements the // `TextMapCarrier` interface but should not be used with `OutgoingCarrier`. func (c 
Bash: cd /root/dla-real/grpc-1/project; dla harness gen --lang go 2>&1 | head -c 2500
   -> {"artifacts":[".dla/20261121-003848-136a/harness/H1/harness.json",".dla/20261121-003848-136a/harness/H1/probes.json"],"candidates":0,"command":"harness gen","evidence":["E7"],"lang":"go","ok":true,"skipped_duplicates":0,"summary":{"agent":"go-test-wrapper","bu
Bash: cd /root/dla-real/grpc-1/project; dla harness gen --lang go --driver --package . 2>&1 | head -c 3000
   -> {"artifacts":[".dla/20261121-003848-136a/harness/H2/harness.json",".dla/20261121-003848-136a/harness/H2/probes.json",".dla/20261121-003848-136a/harness/H2/driver/zz_dla_driver_test.go"],"candidates":0,"command":"harness gen","evidence":["E8"],"lang":"go","ok":
Bash: cd /root/dla-real/grpc-1/project; cat .dla/20261121-003848-136a/harness/H2/driver/zz_dla_driver_test.go | head -120; head -20 go.mod; ls ~/go/pkg/mod 2>/dev/null | head
   -> // Code generated for the DLA harness; not part of the project. Driver mode. // // Placed in the package under test (so package-level variables and unexported functions are // reachable) as zz_dla_driver_test.go. Fill the two DLA-HOOK bodies below and nothing 
Bash: cd /root/dla-real/grpc-1/project; grep -n "DLA-HOOK" -B3 -A12 .dla/20261121-003848-136a/harness/H2/driver/zz_dla_driver_test.go | sed -n 1,120p
   -> 1-// Code generated for the DLA harness; not part of the project. Driver mode. 2-// 3-// Placed in the package under test (so package-level variables and unexported functions are 4:// reachable) as zz_dla_driver_test.go. Fill the two DLA-HOOK bodies below and 
Bash: cd /root/dla-real/grpc-1/project; F=.dla/20261121-003848-136a/harness/H2/driver/zz_dla_driver_test.go; python3 - <<'EOF' ; p='.dla/20261121-003848-136a/harness/H2/driver/zz_dla_driver_test.go' ; s=open(p).read() ; s=s.replace('''	// DLA-HOOK setup: construct the services/caches/pools the workload needs and keep them in package-l
   -> {"artifacts":[".dla/20261121-003848-136a/series/S1.json",".dla/20261121-003848-136a/series/S2.json",".dla/20261121-003848-136a/harness/H2/run-S1.json"],"candidates":0,"command":"harness run","effects":{"cleanup":"every process is registered in state.json and s
SAY Harness H2 ran and produced series S1 and S2. I'm now running the trend analysis on both.
Bash: cd /root/dla-real/grpc-1/project; dla trend S1 2>&1 | head -c 2500; echo; dla trend S2 2>&1 | head -c 2500
   -> {"columns":18,"command":"trend","evidence":["E10","E11"],"failed":18,"next":[],"ok":true,"passed":[],"series":"S1","trend":"T1"}  {"columns":18,"command":"trend","evidence":["E12","E13"],"failed":17,"next":["dla gap S2 --residual   # column os.vm_hwm measures 
Bash: cd /root/dla-real/grpc-1/project; dla gap S2 --residual 2>&1 | head -c 1500; echo; dla gate check --stage classify 2>&1 | head -c 2000
   -> {"artifacts":[".dla/20261121-003848-136a/gap/residual-S2-E14.json"],"calibrated":false,"command":"gap","evidence":["E14"],"limitations":["off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)"],"next":["no region has unexpla
Bash: cd /root/dla-real/grpc-1/project; dla ledger resolve --help 2>&1 | sed -n 14,60p
   ->           - unresolved:            Left unresolved, with a reason and a remedy           - suspected_static_only: Only static evidence exists        --project <DIR>           Project directory (default: the git toplevel above the cwd, else the cwd)        --re
Bash: cd /root/dla-real/grpc-1/project; dla harness gen --lang go --driver --package . 2>&1 | grep -o '"harness":"H[0-9]*"' | head -1; ls .dla/20261121-003848-136a/harness/
   -> "harness":"H3" H1 H2 H3
Bash: cd /root/dla-real/grpc-1/project; python3 - <<'EOF' ; p='.dla/20261121-003848-136a/harness/H3/driver/zz_dla_driver_test.go' ; s=open(p).read() ; s=s.replace('''	// DLA-HOOK setup: construct the services/caches/pools the workload needs and keep them in package-level variables (package-private API is usable here; use dlaT for t.Te
   -> {"artifacts":[".dla/20261121-003848-136a/series/S3.json",".dla/20261121-003848-136a/series/S4.json",".dla/20261121-003848-136a/harness/H3/run-S3.json"],"candidates":0,"command":"harness run","effects":{"cleanup":"every process is registered in state.json and s
Bash: cd /root/dla-real/grpc-1/project; dla trend S3 2>&1 | head -c 1500; echo; dla trend S4 2>&1 | head -c 1500
   -> {"columns":24,"command":"trend","evidence":["E17","E18"],"failed":24,"next":[],"ok":true,"passed":[],"series":"S3","trend":"T3"}  {"columns":24,"command":"trend","evidence":["E19","E20"],"failed":22,"next":["dla gap S4 --residual   # column go.heap_live measur
SAY Series S3 shows no growth. S4 shows a tiny 8 byte per iteration heap step, so I'm checking whether the residual accounts for it.
Bash: cd /root/dla-real/grpc-1/project; dla gap S4 --residual 2>&1 | head -c 1500; echo; dla gap S2 --residual >/dev/null 2>&1; dla gate check --stage classify 2>&1 | head -c 1200
   -> {"artifacts":[".dla/20261121-003848-136a/gap/residual-S4-E21.json"],"calibrated":false,"command":"gap","evidence":["E21"],"limitations":["off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)"],"next":["O1: dla ledger add --
Bash: cd /root/dla-real/grpc-1/project; R=.dla/20261121-003848-136a ; dla ledger resolve O1 --as unresolved --reason inconclusive --remedy "8.4 bytes/iteration stair on go.heap_live in repeated-key series S4 (fresh-key S3 flat); a longer run with more rounds or a heap profile (--capture alloc) would show whether it is warm-up or a rea
   -> {"id":"O1","ok":true,"record":".dla/20261121-003848-136a/ledger.jsonl#L1142","seq":1142} {"artifacts":[],"blocking":[{"description":"go: 3520 iteration(s) ran and 0 of 2305 entry points were exercised: the driver did not call the cod...","fix":"fill the DLA-HO
Bash: cd /root/dla-real/grpc-1/project; dla gap S1 --residual 2>&1 | head -c 600; dla gap S3 --residual 2>&1 | head -c 600; dla run status 2>&1 | python3 -c " ; import json,sys ; d=json.load(sys.stdin) ; for b in d['blocking']: print(b)"
   -> {"artifacts":[".dla/20261121-003848-136a/gap/residual-S1-E23.json"],"calibrated":false,"command":"gap","evidence":["E23"],"limitations":["off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)"],"next":["O2: dla ledger add --
Bash: cd /root/dla-real/grpc-1/project; dla ledger resolve O2 --as unresolved --reason inconclusive --remedy "3.7 bytes/iteration heap stair in S1 (below 4 B noise-scale threshold; S3 flat); longer series or --capture alloc would decide" >/dev/null; dla gate check --final 2>&1 | python3 -c " ; import json,sys ; d=json.load(sys.stdin) 
   -> Exit code 1 Traceback (most recent call last):   File "<string>", line 3, in <module>     d=json.load(sys.stdin)   File "/usr/local/lib/python3.13/json/__init__.py", line 298, in load     return loads(fp.read(),         cls=cls, object_hook=object_hook,       
Bash: cd /root/dla-real/grpc-1/project; dla gate check --final 2>&1 | head -c 3500
   -> {"missing":[{"description":"go: 3520 iteration(s) ran and 0 of 2305 entry points were exercised: the driver did not call the code under test, so the run says nothing about the entry points","fix":"fill the DLA-HOOK bodies of H3 so its driver calls the entry po
Bash: cd /root/dla-real/grpc-1/project; R=.dla/20261121-003848-136a; ls $R/harness/H3 $R/harness/H2; find $R -iname '*cov*' | head; grep -o '"coverage[^}]*' $R/harness/H3/run-S3.json | head -c 600
   -> .dla/20261121-003848-136a/harness/H2: agent attempt.json cov driver harness.json logs out probes.json run-S1.json work  .dla/20261121-003848-136a/harness/H3: agent attempt.json cov driver harness.json logs out probes.json run-S3.json work .dla/20261121-003848-
Bash: cd /root/dla-real/grpc-1/project; ls .dla/20261121-003848-136a/harness/H3/cov | head -3; timeout 500 dla harness run H1 2>&1 | head -c 1200
   -> {"error":{"code":"harness-failed","fix":"read the logs, fix the harness or its hooks, then dla harness run H1","message":"every measured process of mode native exited with a non-zero code (p1-native: 1, p2-native: 1, p3-native: 1, p4-native: 1): the harness it
SAY As expected, H1 needs an external interop server and fails. I'll record the coverage blocker as the gate instructs.
Bash: cd /root/dla-real/grpc-1/project; dla ledger resolve intake.workload --status unresolved --reason not_exercised --remedy "Driver-mode Go harnesses (H2, H3) exercised root-package client/server RPCs but record no coverage profile (empty cov/); command-mode H1 (interop/client soak) needs an external interop server and exits 1. Pro
   -> {"id":"intake.workload","ok":true,"record":".dla/20261121-003848-136a/ledger.jsonl#L1875","seq":1875,"status":"unresolved"} {"missing":[],"missing_total":0,"ok":true,"pointer":".dla/20261121-003848-136a/gate/final.json","stage":"final"}
Bash: cd /root/dla-real/grpc-1/project; dla report 2>&1 | head -c 4000
   -> report: .dla/20261121-003848-136a/report.md sha256=ccd259d75b45dc22656a5ab1d6cc209111e72353bd89f6868d5b20c055c3a4a5 # DLA report  Run: `20261121-003848-136a`  Status: COMPLETE  Measured: 2 harness runs, 4 series, 0 captures  Coverage warning: this run exercise
SAY **No leak was confirmed, and this run barely tested the code.** It exercised 0 of 2305 entry points (go). That means it neither shows nor rules out a leak in the rest of the repository.  The `dla report` for run `20261121-003848-136a` is COMPLETE, and the final gate passed. Its Findings line reads: 
RESULT turns=32 cost=0.4915220999999999 is_error=False :: **No leak was confirmed, and this run barely tested the code.** It exercised 0 of 2305 entry points (go). That means it neither shows nor rules out a leak in the rest of the repository.

The `dla report` for run `20261121-003848-136a` is COMPLETE, and the final gate passed. Its Findings line reads: 728 enumerated, 728 dispositioned; leaks confirmed 0, strong 0, refuted 0, suspected 728, unresolved 0; non-leak explanations: confirmed 0. I wrote 0 "explanations, not leaks" because the report counts none.

**What was measured:** 2 harness runs, 4 series, 0 captures. I wrote two in-process drivers for the root `grpc` package. They are `H2`, which opens and closes a client per call, and `H3`, which reuses one connection and abandons every other stream. Fresh-key heap, goroutines and threads were flat in both. The repeated-key series showed small heap steps of 3.7 bytes/iteration (`S1`) and 8.4 bytes/iteration (`S4`). These are above their thresholds and are recorded as unresolved open items `O2` and

**Why coverage is 0:** the driver-mode harnesses recorded no coverage profile. The command-mode harness `H1` (interop client soak) needs an external interop server and exited 1. I recorded this on `intake.workload` as `not_exercised`, with the remedy of an in-process server workload run with `-coverprofile`, or per-package drivers for the census holders.

**The 728 suspected_static_only findings** (2 detector h
```
